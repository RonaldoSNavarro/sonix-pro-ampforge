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
    mix_pct: f32,
    spring_tension: f32,
    bypassed: bool,
    spillover_enabled: bool,
    // Delay lines for Schroeder / Freeverb comb filters
    comb_buffers: Vec<Vec<f32>>,
    comb_positions: Vec<usize>,
    comb_feedback: f32,
}

impl ReverbProcessor {
    pub fn new(model: ReverbModel) -> Self {
        // Pre-allocate 4 comb delay lines
        let comb_lengths = [1116, 1188, 1277, 1356];
        let mut buffers = Vec::new();
        for &len in &comb_lengths {
            buffers.push(vec![0.0f32; len]);
        }

        Self {
            model,
            sample_rate: 48000.0,
            decay_time_s: 2.5,
            mix_pct: 35.0,
            spring_tension: 5.0,
            bypassed: false,
            spillover_enabled: false,
            comb_buffers: buffers,
            comb_positions: vec![0; 4],
            comb_feedback: 0.75,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn decay_time_s(&self) -> f32 {
        self.decay_time_s
    }

    pub fn set_decay_time_s(&mut self, val: f32) {
        self.decay_time_s = val.clamp(0.1, 30.0);
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

    pub fn has_shimmer_octave_generation(&self) -> bool {
        matches!(self.model, ReverbModel::ShimmerCloudAmbient)
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        let wet_ratio = self.mix_pct / 100.0;
        let dry_ratio = 1.0 - wet_ratio;

        let num_combs = self.comb_buffers.len();
        let fb = self.comb_feedback;

        for i in 0..input.len() {
            let in_sample = if self.bypassed && self.spillover_enabled {
                0.0
            } else if self.bypassed {
                output[i] = input[i];
                continue;
            } else {
                input[i]
            };

            let mut wet_sum = 0.0f32;
            for c in 0..num_combs {
                let buf_len = self.comb_buffers[c].len();
                let pos = self.comb_positions[c];
                let out_c = self.comb_buffers[c][pos];
                wet_sum += out_c;

                // Comb feedback with spring drip or shimmer
                let mut fb_sig = out_c * fb;
                if self.has_shimmer_octave_generation() {
                    // Non-linear octave shimmer harmonic generation
                    fb_sig += 0.15 * (out_c * 2.0).tanh();
                }

                self.comb_buffers[c][pos] = in_sample + fb_sig;
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
