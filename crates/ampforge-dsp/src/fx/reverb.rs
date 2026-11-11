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
    // Delay lines for Schroeder / Freeverb comb filters
    comb_buffers: Vec<Vec<f32>>,
    comb_positions: Vec<usize>,
    comb_feedback: f32,
    damp_states: Vec<f32>,
    predelay_buffer: Vec<f32>,
    predelay_pos: usize,
}

impl ReverbProcessor {
    pub fn new(model: ReverbModel) -> Self {
        // Pre-allocate 4 comb delay lines
        let comb_lengths = [1116, 1188, 1277, 1356];
        let mut buffers = Vec::new();
        for &len in &comb_lengths {
            buffers.push(vec![0.0f32; len]);
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
            comb_positions: vec![0; 4],
            comb_feedback: 0.85,
            damp_states: vec![0.0f32; 4],
            predelay_buffer: vec![0.0f32; 14400], // até 150 ms a 96 kHz
            predelay_pos: 0,
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
    }

    pub fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    pub fn set_spillover_enabled(&mut self, enabled: bool) {
        self.spillover_enabled = enabled;
    }

    fn update_comb_feedback(&mut self) {
        let avg_loop_samples = 1234.0f32;
        let loop_time_s = avg_loop_samples / self.sample_rate;
        let exponent = -3.0 * (loop_time_s / self.decay_time_s.max(0.1));
        self.comb_feedback = 10.0f32.powf(exponent).clamp(0.1, 0.98);
    }

    pub fn has_shimmer_octave_generation(&self) -> bool {
        matches!(self.model, ReverbModel::ShimmerCloudAmbient)
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        if self.mix_pct <= 0.0 {
            output.copy_from_slice(input);
            return;
        }

        let wet_ratio = self.mix_pct / 100.0;
        let dry_ratio = 1.0 - wet_ratio;

        let num_combs = self.comb_buffers.len();
        let fb = self.comb_feedback;
        let damp_coeff = (self.damping / 10.0) * 0.4;
        let predelay_len = self.predelay_buffer.len();
        let predelay_samples = (self.predelay_ms * 0.001 * self.sample_rate) as usize;

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
            let read_idx = (self.predelay_pos + predelay_len - predelay_samples.min(predelay_len - 1)) % predelay_len;
            let delayed_in = self.predelay_buffer[read_idx];
            self.predelay_pos = (self.predelay_pos + 1) % predelay_len;

            let mut wet_sum = 0.0f32;
            for c in 0..num_combs {
                let buf_len = self.comb_buffers[c].len();
                let pos = self.comb_positions[c];
                let out_c = self.comb_buffers[c][pos];
                wet_sum += out_c;

                // One-pole low-pass damping in loop
                let damp_out = (1.0 - damp_coeff) * out_c + damp_coeff * self.damp_states[c];
                self.damp_states[c] = damp_out;

                // Comb feedback with spring drip or shimmer
                let mut fb_sig = damp_out * fb;
                if self.has_shimmer_octave_generation() {
                    fb_sig += 0.15 * (out_c * 2.0).tanh();
                }

                self.comb_buffers[c][pos] = delayed_in + fb_sig;
                self.comb_positions[c] = (pos + 1) % buf_len;
            }

            wet_sum *= 0.25; // Normalize comb sum

            if self.bypassed && self.spillover_enabled {
                output[i] = input[i] + wet_sum;
            } else {
                output[i] = input[i] * dry_ratio + wet_sum * wet_ratio;
            }
        }
    }
}
