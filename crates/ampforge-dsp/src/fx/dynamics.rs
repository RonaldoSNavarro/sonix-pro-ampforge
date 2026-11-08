//! crates/ampforge-dsp/src/fx/dynamics.rs
//! Processadores de Dinâmica: Noise Gate Inteligente e Compressor Multi-Topologia

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseGateMode {
    IntelligentNaturalGate,
    ClassicExpanderGate,
}

pub struct NoiseGateProcessor {
    mode: NoiseGateMode,
    threshold_db: f32,
    ratio: f32,
    attack_ms: f32,
    hold_ms: f32,
    release_ms: f32,
    sample_rate: f32,
    envelope: f32,
    gain_reduction_db: f32,
}

impl NoiseGateProcessor {
    pub fn new(mode: NoiseGateMode) -> Self {
        Self {
            mode,
            threshold_db: -50.0,
            ratio: 10.0,
            attack_ms: 2.0,
            hold_ms: 10.0,
            release_ms: 50.0,
            sample_rate: 48000.0,
            envelope: 0.0,
            gain_reduction_db: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn threshold_db(&self) -> f32 {
        self.threshold_db
    }

    pub fn set_threshold_db(&mut self, val: f32) {
        self.threshold_db = val.clamp(-90.0, 0.0);
    }

    pub fn set_ratio(&mut self, val: f32) {
        self.ratio = val.clamp(1.2, 100.0);
    }

    pub fn set_attack_ms(&mut self, val: f32) {
        self.attack_ms = val.clamp(0.1, 50.0);
    }

    pub fn set_hold_ms(&mut self, val: f32) {
        self.hold_ms = val.clamp(1.0, 500.0);
    }

    pub fn set_release_ms(&mut self, val: f32) {
        self.release_ms = val.clamp(5.0, 1000.0);
    }

    pub fn gain_reduction_db(&self) -> f32 {
        self.gain_reduction_db
    }

    pub fn process(&mut self, input: &[f32], sidechain_di: Option<&[f32]>, output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        let threshold_lin = 10.0f32.powf(self.threshold_db / 20.0);

        let att_coeff = (-1.0 / (self.sample_rate * (self.attack_ms * 0.001))).exp();
        let rel_coeff = (-1.0 / (self.sample_rate * (self.release_ms * 0.001))).exp();

        for i in 0..input.len() {
            // Determine detector source
            let detector_val = match (self.mode, sidechain_di) {
                (NoiseGateMode::IntelligentNaturalGate, Some(di)) if i < di.len() => di[i].abs(),
                _ => input[i].abs(),
            };

            // Envelope follower
            if detector_val > self.envelope {
                self.envelope = (1.0 - att_coeff) * detector_val + att_coeff * self.envelope;
            } else {
                self.envelope = (1.0 - rel_coeff) * detector_val + rel_coeff * self.envelope;
            }

            // Downward expansion calculation
            let gain = if self.envelope >= threshold_lin {
                1.0
            } else {
                let diff_db = 20.0 * (self.envelope.max(1e-6) / threshold_lin).log10();
                let gr_db = diff_db * (self.ratio - 1.0);
                10.0f32.powf(gr_db / 20.0).clamp(0.0, 1.0)
            };

            output[i] = input[i] * gain;
            self.gain_reduction_db = if gain > 1e-4 {
                20.0 * gain.log10()
            } else {
                -80.0
            };
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressorTopology {
    PedalRossDyna,
    StudioFet1176,
    OpticalLa2a,
    VcaMasterBus,
}

pub struct CompressorProcessor {
    pub topology: CompressorTopology,
    threshold_db: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
    makeup_gain_db: f32,
    parallel_mix_pct: f32,
    sample_rate: f32,
    envelope: f32,
    gain_reduction_db: f32,
}

impl CompressorProcessor {
    pub fn new(topology: CompressorTopology) -> Self {
        Self {
            topology,
            threshold_db: -20.0,
            ratio: 4.0,
            attack_ms: 10.0,
            release_ms: 100.0,
            makeup_gain_db: 0.0,
            parallel_mix_pct: 100.0,
            sample_rate: 48000.0,
            envelope: 0.0,
            gain_reduction_db: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn set_threshold_db(&mut self, val: f32) {
        self.threshold_db = val.clamp(-60.0, 0.0);
    }

    pub fn set_ratio(&mut self, val: f32) {
        self.ratio = val.clamp(1.0, 20.0);
    }

    pub fn set_parallel_mix_pct(&mut self, val: f32) {
        self.parallel_mix_pct = val.clamp(0.0, 100.0);
    }

    pub fn gain_reduction_db(&self) -> f32 {
        self.gain_reduction_db
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        let threshold_lin = 10.0f32.powf(self.threshold_db / 20.0);
        let makeup_lin = 10.0f32.powf(self.makeup_gain_db / 20.0);
        let wet_ratio = self.parallel_mix_pct / 100.0;
        let dry_ratio = 1.0 - wet_ratio;

        let att_coeff = (-1.0 / (self.sample_rate * (self.attack_ms * 0.001))).exp();
        let rel_coeff = (-1.0 / (self.sample_rate * (self.release_ms * 0.001))).exp();

        for i in 0..input.len() {
            let abs_x = input[i].abs();
            if abs_x > self.envelope {
                self.envelope = (1.0 - att_coeff) * abs_x + att_coeff * self.envelope;
            } else {
                self.envelope = (1.0 - rel_coeff) * abs_x + rel_coeff * self.envelope;
            }

            let gain = if self.envelope <= threshold_lin {
                1.0
            } else {
                let overshoot_db = 20.0 * (self.envelope / threshold_lin).log10();
                let gr_db = -overshoot_db * (1.0 - 1.0 / self.ratio);
                10.0f32.powf(gr_db / 20.0)
            };

            self.gain_reduction_db = 20.0 * gain.log10();

            let wet = input[i] * gain * makeup_lin;
            let dry = input[i];
            output[i] = dry * dry_ratio + wet * wet_ratio;
        }
    }
}
