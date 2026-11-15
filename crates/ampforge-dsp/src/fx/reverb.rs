//! crates/ampforge-dsp/src/fx/reverb.rs
//! Processadores de Reverb (Tube Spring com "Drip", Vintage Plate, Hall, Chamber, Shimmer)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReverbModel {
    TubeSpringReverb,
    StudioVintagePlate,
    ConcertHall,
    Chamber,
    ShimmerCloudAmbient,
}

const BASE_COMB_LENS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const BASE_AP_LENS: [usize; 4] = [556, 441, 341, 225];
const SHIMMER_BUF_LEN: usize = 4096;

pub struct ReverbProcessor {
    pub model: ReverbModel,
    sample_rate: f32,
    decay_time_s: f32,
    predelay_ms: f32,
    damping: f32,
    mix_pct: f32,
    spring_tension: f32,
    bypassed: bool,
    spillover_enabled: bool,
    // Pre-allocated delay lines (sized for up to 96 kHz + spring tension)
    comb_buffers: Vec<Vec<f32>>,
    comb_active_lens: [usize; 8],
    comb_positions: [usize; 8],
    comb_feedbacks: [f32; 8],
    damp_states: [f32; 8],
    // 4 series Schroeder all-pass diffusers
    ap_buffers: Vec<Vec<f32>>,
    ap_active_lens: [usize; 4],
    ap_positions: [usize; 4],
    // Spring dispersive all-pass chain
    spring_disp_x1: [f32; 8],
    spring_disp_y1: [f32; 8],
    // Pre-allocated +12 semitone shimmer granular buffer
    shimmer_buf: Vec<f32>,
    shimmer_write_pos: usize,
    shimmer_phase: f32,
    shimmer_last_out: f32,
    predelay_buffer: Vec<f32>,
    predelay_pos: usize,
    // Contador O(1) para invalidar caudas sem memset no callback RT
    samples_since_clear: usize,
}

impl ReverbProcessor {
    pub fn new(model: ReverbModel) -> Self {
        let mut buffers = Vec::with_capacity(8);
        for &len in &BASE_COMB_LENS {
            // 4x capacity covers 96 kHz (2x) + loose spring tension scaling (1.6x)
            buffers.push(vec![0.0f32; len * 4 + 32]);
        }
        let mut ap_buffers = Vec::with_capacity(4);
        for &len in &BASE_AP_LENS {
            ap_buffers.push(vec![0.0f32; len * 2 + 32]);
        }

        let mut p = Self {
            model,
            sample_rate: 48000.0,
            decay_time_s: 2.5,
            predelay_ms: 20.0,
            damping: 4.0,
            mix_pct: 35.0,
            spring_tension: 5.0,
            bypassed: false,
            spillover_enabled: false,
            comb_buffers: buffers,
            comb_active_lens: BASE_COMB_LENS,
            comb_positions: [0; 8],
            comb_feedbacks: [0.85; 8],
            damp_states: [0.0; 8],
            ap_buffers,
            ap_active_lens: BASE_AP_LENS,
            ap_positions: [0; 4],
            spring_disp_x1: [0.0; 8],
            spring_disp_y1: [0.0; 8],
            shimmer_buf: vec![0.0; SHIMMER_BUF_LEN],
            shimmer_write_pos: 0,
            shimmer_phase: 0.0,
            shimmer_last_out: 0.0,
            predelay_buffer: vec![0.0f32; 14400], // até 150 ms a 96 kHz
            predelay_pos: 0,
            samples_since_clear: usize::MAX,
        };
        p.update_comb_feedback();
        p
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.update_comb_feedback();
    }

    pub fn decay_time_s(&self) -> f32 {
        self.decay_time_s
    }

    pub fn set_decay_time_s(&mut self, val: f32) {
        self.decay_time_s = val.clamp(0.1, 30.0);
        self.update_comb_feedback();
    }

    pub fn predelay_ms(&self) -> f32 {
        self.predelay_ms
    }

    pub fn set_predelay_ms(&mut self, val: f32) {
        self.predelay_ms = val.clamp(0.0, 150.0);
    }

    pub fn damping(&self) -> f32 {
        self.damping
    }

    pub fn set_damping(&mut self, val: f32) {
        self.damping = val.clamp(0.0, 10.0);
    }

    pub fn mix_pct(&self) -> f32 {
        self.mix_pct
    }

    pub fn set_mix_pct(&mut self, val: f32) {
        self.mix_pct = val.clamp(0.0, 100.0);
    }

    pub fn spring_tension(&self) -> f32 {
        self.spring_tension
    }

    pub fn set_spring_tension(&mut self, val: f32) {
        self.spring_tension = val.clamp(0.0, 10.0);
        self.update_comb_feedback();
    }

    pub fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    pub fn reset_state(&mut self) {
        self.clear_tail();
    }

    pub fn set_spillover_enabled(&mut self, enabled: bool) {
        self.spillover_enabled = enabled;
    }

    pub fn is_spillover_enabled(&self) -> bool {
        self.spillover_enabled
    }

    pub fn clear_tail(&mut self) {
        // O(1): evita memset dos buffers de delay/comb/allpass no callback RT
        self.samples_since_clear = 0;
        self.damp_states.fill(0.0);
        self.spring_disp_x1.fill(0.0);
        self.spring_disp_y1.fill(0.0);
        self.shimmer_last_out = 0.0;
    }

    fn update_comb_feedback(&mut self) {
        let sr_scale = self.sample_rate / 48000.0;
        let model_scale = match self.model {
            ReverbModel::TubeSpringReverb => 0.60 * (1.0 + (10.0 - self.spring_tension) * 0.12),
            ReverbModel::Chamber => 0.80,
            ReverbModel::StudioVintagePlate => 0.95,
            ReverbModel::ConcertHall => 1.15,
            ReverbModel::ShimmerCloudAmbient => 1.10,
        };
        let rt60 = self.decay_time_s.max(0.1);
        for c in 0..8 {
            let cap = self.comb_buffers[c].len();
            let active = ((BASE_COMB_LENS[c] as f32 * sr_scale * model_scale).round() as usize)
                .clamp(16, cap);
            self.comb_active_lens[c] = active;
            self.comb_positions[c] %= active;
            let loop_time_s = active as f32 / self.sample_rate;
            let exponent = -3.0 * (loop_time_s / rt60);
            self.comb_feedbacks[c] = 10.0f32.powf(exponent).clamp(0.05, 0.992);
        }
        for a in 0..4 {
            let cap = self.ap_buffers[a].len();
            let active = ((BASE_AP_LENS[a] as f32 * sr_scale).round() as usize).clamp(8, cap);
            self.ap_active_lens[a] = active;
            self.ap_positions[a] %= active;
        }
    }

    pub fn has_shimmer_octave_generation(&self) -> bool {
        matches!(self.model, ReverbModel::ShimmerCloudAmbient)
    }

    #[inline(always)]
    fn read_shimmer_linear(&self, delay_samples: f32) -> f32 {
        let d = delay_samples.clamp(1.0, (SHIMMER_BUF_LEN - 2) as f32);
        if (d.ceil() as usize) > self.samples_since_clear {
            return 0.0;
        }
        let pos = (self.shimmer_write_pos as f32 + SHIMMER_BUF_LEN as f32 - d)
            % SHIMMER_BUF_LEN as f32;
        let i0 = pos.floor() as usize % SHIMMER_BUF_LEN;
        let i1 = (i0 + 1) % SHIMMER_BUF_LEN;
        let frac = pos - pos.floor();
        self.shimmer_buf[i0] * (1.0 - frac) + self.shimmer_buf[i1] * frac
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        if self.mix_pct <= 0.0 {
            output.copy_from_slice(input);
            return;
        }

        let wet_ratio = self.mix_pct / 100.0;
        let dry_ratio = 1.0 - wet_ratio;

        let num_combs = match self.model {
            ReverbModel::TubeSpringReverb => 4,
            _ => 8,
        };
        let comb_norm = 1.0 / (num_combs as f32).sqrt();
        let damp_coeff = (self.damping / 10.0) * 0.45;
        let predelay_len = self.predelay_buffer.len();
        let predelay_samples = (self.predelay_ms * 0.001 * self.sample_rate) as usize;
        let spring_disp_a = (0.45 + 0.04 * (10.0 - self.spring_tension)).clamp(0.1, 0.88);
        let shimmer_grain = (1536.0 * (self.sample_rate / 48000.0)).clamp(256.0, 2048.0);
        let shimmer_inc = 1.0 / shimmer_grain;

        for i in 0..input.len() {
            let in_sample = if self.bypassed && self.spillover_enabled {
                0.0
            } else if self.bypassed {
                output[i] = input[i];
                continue;
            } else {
                input[i]
            };

            // Predelay circular buffer
            self.predelay_buffer[self.predelay_pos] = in_sample;
            let pd = predelay_samples.min(predelay_len - 1);
            let delayed_in = if pd > self.samples_since_clear {
                0.0
            } else {
                let read_idx = (self.predelay_pos + predelay_len - pd) % predelay_len;
                self.predelay_buffer[read_idx]
            };
            self.predelay_pos = (self.predelay_pos + 1) % predelay_len;

            let tank_input = if self.has_shimmer_octave_generation() {
                delayed_in + self.shimmer_last_out * 0.55
            } else {
                delayed_in
            };

            let mut wet_sum = 0.0f32;
            for c in 0..num_combs {
                let len = self.comb_active_lens[c];
                let pos = self.comb_positions[c];
                let out_c = if self.samples_since_clear < len {
                    0.0
                } else {
                    self.comb_buffers[c][pos]
                };
                wet_sum += out_c;

                // One-pole low-pass damping in loop
                let damp_out = (1.0 - damp_coeff) * out_c + damp_coeff * self.damp_states[c];
                self.damp_states[c] = damp_out;

                let fb_sig = damp_out * self.comb_feedbacks[c];
                self.comb_buffers[c][pos] = (tank_input + fb_sig).clamp(-4.0, 4.0);
                self.comb_positions[c] = if pos + 1 >= len { 0 } else { pos + 1 };
            }

            let mut diffused = wet_sum * comb_norm;

            if self.model == ReverbModel::TubeSpringReverb {
                // Dispersive spring all-pass chain controlled by spring_tension
                for s in 0..8 {
                    let y = -spring_disp_a * diffused
                        + self.spring_disp_x1[s]
                        + spring_disp_a * self.spring_disp_y1[s];
                    self.spring_disp_x1[s] = diffused;
                    self.spring_disp_y1[s] = y;
                    diffused = y;
                }
            } else {
                // 4 series Schroeder all-pass diffusers for dense plate/hall/chamber/shimmer tails
                let ap_g = 0.5f32;
                for a in 0..4 {
                    let len = self.ap_active_lens[a];
                    let pos = self.ap_positions[a];
                    let buf_out = if self.samples_since_clear < len {
                        0.0
                    } else {
                        self.ap_buffers[a][pos]
                    };
                    let y = -ap_g * diffused + buf_out;
                    self.ap_buffers[a][pos] = (diffused + ap_g * y).clamp(-4.0, 4.0);
                    self.ap_positions[a] = if pos + 1 >= len { 0 } else { pos + 1 };
                    diffused = y;
                }
            }

            if self.has_shimmer_octave_generation() {
                // Zero-alloc dual-grain +12 semitone pitch shifter fed by diffused tail
                self.shimmer_buf[self.shimmer_write_pos] = diffused;
                let p0 = self.shimmer_phase;
                let p1 = (p0 + 0.5) % 1.0;
                let w0 = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * p0).cos();
                let w1 = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * p1).cos();
                let d0 = shimmer_grain * (1.0 - p0) + 1.0;
                let d1 = shimmer_grain * (1.0 - p1) + 1.0;
                let shifted =
                    self.read_shimmer_linear(d0) * w0 + self.read_shimmer_linear(d1) * w1;
                self.shimmer_last_out = shifted;
                self.shimmer_write_pos = (self.shimmer_write_pos + 1) % SHIMMER_BUF_LEN;
                self.shimmer_phase = (self.shimmer_phase + shimmer_inc) % 1.0;
                diffused = diffused * 0.65 + shifted * 0.55;
            }

            self.samples_since_clear = self.samples_since_clear.saturating_add(1);
            if self.bypassed && self.spillover_enabled {
                output[i] = input[i] + diffused * wet_ratio;
            } else {
                output[i] = input[i] * dry_ratio + diffused * wet_ratio;
            }
        }
    }
}
