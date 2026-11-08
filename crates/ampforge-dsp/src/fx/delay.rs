//! crates/ampforge-dsp/src/fx/delay.rs
//! Processadores de Delay (Tape Echo, BBD, Digital, Ducking e Spillover)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayModel {
    MultiHeadTapeEcho,
    AnalogBbdDelay,
    PristineDigitalDelay,
    DualParallelDelay,
    ReverseDelay,
    PingPongStereo,
}

pub struct DelayProcessor {
    pub model: DelayModel,
    sample_rate: f32,
    delay_time_ms: f32,
    feedback_pct: f32,
    mix_pct: f32,
    tempo_bpm: f32,
    // Ducking
    ducking_enabled: bool,
    ducking_threshold_db: f32,
    ducking_attenuation_db: f32,
    ducking_release_ms: f32,
    ducking_gain_reduction_db: f32,
    ducking_envelope: f32,
    // Spillover & Bypass
    bypassed: bool,
    spillover_enabled: bool,
    // Pre-allocated circular buffer (up to 4000 ms at 96 kHz: 384,000 samples)
    buffer: Vec<f32>,
    write_pos: usize,
}

impl DelayProcessor {
    pub fn new(model: DelayModel) -> Self {
        // Pre-allocate buffer for 4000 ms at 96 kHz
        let max_samples = (4.0 * 96000.0) as usize;
        Self {
            model,
            sample_rate: 48000.0,
            delay_time_ms: 350.0,
            feedback_pct: 35.0,
            mix_pct: 40.0,
            tempo_bpm: 120.0,
            ducking_enabled: false,
            ducking_threshold_db: -30.0,
            ducking_attenuation_db: -12.0,
            ducking_release_ms: 150.0,
            ducking_gain_reduction_db: 0.0,
            ducking_envelope: 0.0,
            bypassed: false,
            spillover_enabled: false,
            buffer: vec![0.0; max_samples],
            write_pos: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn tempo_bpm(&self) -> f32 {
        self.tempo_bpm
    }

    pub fn set_tempo_bpm(&mut self, bpm: f32) {
        self.tempo_bpm = bpm.clamp(20.0, 300.0);
    }

    pub fn delay_time_ms(&self) -> f32 {
        self.delay_time_ms
    }

    pub fn set_delay_time_ms(&mut self, ms: f32) {
        self.delay_time_ms = ms.clamp(1.0, 4000.0);
    }

    pub fn set_feedback_pct(&mut self, fb: f32) {
        self.feedback_pct = fb.clamp(0.0, 150.0);
    }

    pub fn set_mix_pct(&mut self, mix: f32) {
        self.mix_pct = mix.clamp(0.0, 100.0);
    }

    pub fn set_ducking_enabled(&mut self, enabled: bool) {
        self.ducking_enabled = enabled;
    }

    pub fn set_ducking_threshold_db(&mut self, db: f32) {
        self.ducking_threshold_db = db.clamp(-60.0, 0.0);
    }

    pub fn set_ducking_attenuation_db(&mut self, db: f32) {
        self.ducking_attenuation_db = db.clamp(-30.0, 0.0);
    }

    pub fn ducking_gain_reduction_db(&self) -> f32 {
        self.ducking_gain_reduction_db
    }

    pub fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    pub fn set_spillover_enabled(&mut self, enabled: bool) {
        self.spillover_enabled = enabled;
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());

        let wet_ratio = self.mix_pct / 100.0;
        let dry_ratio = 1.0 - wet_ratio;
        let delay_samples = (self.delay_time_ms * 0.001 * self.sample_rate) as usize;
        let buf_len = self.buffer.len();
        let feedback_gain = self.feedback_pct / 100.0;

        let threshold_lin = 10.0f32.powf(self.ducking_threshold_db / 20.0);
        let max_duck_gain = 10.0f32.powf(self.ducking_attenuation_db / 20.0);
        let duck_rel_coeff = (-1.0 / (self.sample_rate * (self.ducking_release_ms * 0.001))).exp();

        for i in 0..input.len() {
            let in_sample = if self.bypassed && self.spillover_enabled {
                0.0 // Mute input to delay line, let tail ring out
            } else if self.bypassed {
                output[i] = input[i];
                continue;
            } else {
                input[i]
            };

            // Ducking envelope
            if self.ducking_enabled {
                let abs_in = input[i].abs();
                if abs_in > self.ducking_envelope {
                    self.ducking_envelope = abs_in;
                } else {
                    self.ducking_envelope = (1.0 - duck_rel_coeff) * abs_in + duck_rel_coeff * self.ducking_envelope;
                }

                if self.ducking_envelope > threshold_lin {
                    self.ducking_gain_reduction_db = self.ducking_attenuation_db;
                } else {
                    self.ducking_gain_reduction_db = 0.0;
                }
            } else {
                self.ducking_gain_reduction_db = 0.0;
            }

            let duck_mult = if self.ducking_enabled && self.ducking_envelope > threshold_lin {
                max_duck_gain
            } else {
                1.0
            };

            // Read from circular delay line
            let read_pos = (self.write_pos + buf_len - delay_samples.min(buf_len - 1)) % buf_len;
            let delayed = self.buffer[read_pos];

            // Magnetic saturation on feedback to model tape self-oscillation
            let fb_signal = (delayed * feedback_gain).tanh();

            // Write back
            self.buffer[self.write_pos] = in_sample + fb_signal;
            self.write_pos = (self.write_pos + 1) % buf_len;

            let wet = delayed * duck_mult;
            if self.bypassed && self.spillover_enabled {
                output[i] = input[i] + wet; // Spillover tail over direct signal
            } else {
                output[i] = input[i] * dry_ratio + wet * wet_ratio;
            }
        }
    }
}
