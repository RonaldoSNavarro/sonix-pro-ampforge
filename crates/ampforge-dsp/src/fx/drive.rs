//! crates/ampforge-dsp/src/fx/drive.rs
//! Pedais de Overdrive, Distorção e Fuzz White-Box com Troca de Diodos no Mod Shop

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveModel {
    GreenScreamer808,
    GoldenKloneCentaur,
    BlackRodentRat,
    OrangeBossDs,
    FuzzFaceSiGe,
    BigMuffPi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClippingDiode {
    SiliconSymmetrical,
    AsymmetricalSiLed,
    Germanium1N34A,
    MosfetClip,
    HardOpamp,
}

pub struct DriveProcessor {
    model: DriveModel,
    clipping_diode: ClippingDiode,
    gain: f32,
    tone: f32,
    level: f32,
    sample_rate: f32,
}

impl DriveProcessor {
    pub fn new(model: DriveModel) -> Self {
        Self {
            model,
            clipping_diode: ClippingDiode::SiliconSymmetrical,
            gain: 5.0,
            tone: 5.0,
            level: 5.0,
            sample_rate: 48000.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn model(&self) -> DriveModel {
        self.model
    }

    pub fn set_model(&mut self, model: DriveModel) {
        self.model = model;
    }

    pub fn clipping_diode(&self) -> ClippingDiode {
        self.clipping_diode
    }

    pub fn set_clipping_diode(&mut self, diode: ClippingDiode) {
        self.clipping_diode = diode;
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(0.0, 10.0);
    }

    pub fn set_tone(&mut self, tone: f32) {
        self.tone = tone.clamp(0.0, 10.0);
    }

    pub fn set_level(&mut self, level: f32) {
        self.level = level.clamp(0.0, 10.0);
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        let pre_gain = 1.0 + (self.gain / 10.0) * 12.0;

        // Diode forward voltages (Vf_pos, Vf_neg)
        let (vf_pos, vf_neg) = match self.clipping_diode {
            ClippingDiode::Germanium1N34A => (0.30f32, 0.30f32),
            ClippingDiode::SiliconSymmetrical => (0.65f32, 0.65f32),
            ClippingDiode::AsymmetricalSiLed => (0.65f32, 1.80f32),
            ClippingDiode::MosfetClip => (0.90f32, 0.90f32),
            ClippingDiode::HardOpamp => (2.50f32, 2.50f32),
        };

        for i in 0..input.len() {
            let s = input[i] * pre_gain;
            // Asymmetrical / Symmetrical non-linear diode clipping
            let clipped = if s >= 0.0 {
                vf_pos * (s / vf_pos).tanh()
            } else {
                -vf_neg * ((-s) / vf_neg).tanh()
            };
            output[i] = clipped;
        }
    }
}
