//! crates/ampforge-dsp/src/fx/drive.rs
//! Pedais de Overdrive, Distorção e Fuzz White-Box com Troca de Diodos no Mod Shop (Spec 023)

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

#[derive(Debug, Clone, Copy)]
struct DriveBiquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Default for DriveBiquad {
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
}

impl DriveBiquad {
    pub fn high_shelf(fc: f32, gain_db: f32, sr: f32) -> Self {
        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * (fc / sr).clamp(0.001, 0.49);
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / std::f32::consts::SQRT_2;
        let sqrt_a_2_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + sqrt_a_2_alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - sqrt_a_2_alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + sqrt_a_2_alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - sqrt_a_2_alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    #[inline(always)]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

pub struct DriveProcessor {
    model: DriveModel,
    clipping_diode: ClippingDiode,
    gain: f32,
    tone: f32,
    level: f32,
    sample_rate: f32,
    // Estado do filtro genérico anterior
    tone_state: f32,
    // Estados específicos da modelagem TS808 fiel ao circuito (Spec 023)
    ts_gain_biquad: DriveBiquad,
    ts_tone_biquad: DriveBiquad,
}

impl DriveProcessor {
    pub fn new(model: DriveModel) -> Self {
        let mut p = Self {
            model,
            clipping_diode: ClippingDiode::SiliconSymmetrical,
            gain: 5.0,
            tone: 5.0,
            level: 5.0,
            sample_rate: 48000.0,
            tone_state: 0.0,
            ts_gain_biquad: DriveBiquad::default(),
            ts_tone_biquad: DriveBiquad::default(),
        };
        p.update_ts808_filters();
        p
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.update_ts808_filters();
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
        self.update_ts808_filters();
    }

    pub fn set_tone(&mut self, tone: f32) {
        self.tone = tone.clamp(0.0, 10.0);
        self.update_ts808_filters();
    }

    pub fn set_level(&mut self, level: f32) {
        self.level = level.clamp(0.0, 10.0);
    }

    fn update_ts808_filters(&mut self) {
        // 1. Passa-altas de ganho do TS808 (médios amplificados de +6.2 dB até +24.0 dB, graves em 80 Hz a 0 dB)
        let mid_gain_db = 6.2 + 17.8 * (self.gain / 10.0).powf(1.8);
        self.ts_gain_biquad = DriveBiquad::high_shelf(480.0, mid_gain_db, self.sample_rate);

        // 2. Controle de Tone ativo centrado em 1800 Hz (-7.5 dB a +7.5 dB)
        let tone_gain_db = -7.5 + 1.5 * self.tone;
        self.ts_tone_biquad = DriveBiquad::high_shelf(1800.0, tone_gain_db, self.sample_rate);
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());

        match self.model {
            DriveModel::GreenScreamer808 => {
                self.process_ts808(input, output);
            }
            _ => {
                self.process_generic(input, output);
            }
        }
    }

    /// Processador TS808 fiel ao circuito analógico clássico (Spec 023)
    fn process_ts808(&mut self, input: &[f32], output: &mut [f32]) {
        // Tensão de condução dos diodos selecionados (comportamento fiel ao Mod Shop / Spec 006 / Spec 023)
        let (vf_pos, vf_neg) = match self.clipping_diode {
            ClippingDiode::Germanium1N34A => (0.20f32, 0.20f32),
            ClippingDiode::SiliconSymmetrical => (0.35f32, 0.35f32),
            ClippingDiode::AsymmetricalSiLed => (0.35f32, 1.20f32),
            ClippingDiode::MosfetClip => (0.60f32, 0.60f32),
            ClippingDiode::HardOpamp => (1.80f32, 1.80f32),
        };

        // Level de saída: taper de áudio logarítmico (level=0 silencia >= 60 dB)
        let out_gain = if self.level <= 0.001 {
            0.0
        } else {
            (self.level / 10.0).powf(2.0) * 1.5
        };

        for i in 0..input.len() {
            let x = input[i];

            // 1. Passa-altas de ganho do op-amp TS808 (graves permanecem unitários, médios/agudos recebem drive)
            let x_boosted = self.ts_gain_biquad.process(x);

            // 2. Clipping por diodos antiparalelos (simétrico para SiliconSymmetrical f(-x) = -f(x))
            let clipped = if x_boosted.abs() < 1e-12 {
                0.0
            } else if x_boosted >= 0.0 {
                vf_pos * (x_boosted / vf_pos).tanh()
            } else {
                -vf_neg * ((-x_boosted) / vf_neg).tanh()
            };

            // 3. Estágio de Tone ativo
            let toned = self.ts_tone_biquad.process(clipped);

            // 4. Level passivo linear de saída (não altera THD)
            output[i] = toned * out_gain;
        }
    }

    /// Implementação comportamental genérica preservada para os demais modelos de drive
    fn process_generic(&mut self, input: &[f32], output: &mut [f32]) {
        let pre_gain = 1.0 + (self.gain / 10.0) * 35.0;

        let (vf_pos, vf_neg) = match self.clipping_diode {
            ClippingDiode::Germanium1N34A => (0.30f32, 0.30f32),
            ClippingDiode::SiliconSymmetrical => (0.65f32, 0.65f32),
            ClippingDiode::AsymmetricalSiLed => (0.65f32, 1.80f32),
            ClippingDiode::MosfetClip => (0.90f32, 0.90f32),
            ClippingDiode::HardOpamp => (2.50f32, 2.50f32),
        };

        let tone_ratio = self.tone / 10.0;
        let cutoff_hz = 1200.0 + tone_ratio * 4500.0;
        let dt = 1.0 / self.sample_rate;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
        let alpha = (dt / (rc + dt)).clamp(0.01, 0.99);

        let level_db = -12.0 + (self.level / 10.0) * 16.0;
        let out_gain = 10.0f32.powf(level_db / 20.0);

        for i in 0..input.len() {
            let s = input[i] * pre_gain;
            let clipped = if s >= 0.0 {
                vf_pos * (s / vf_pos).tanh()
            } else {
                -vf_neg * ((-s) / vf_neg).tanh()
            };

            self.tone_state += alpha * (clipped - self.tone_state);
            let toned = self.tone_state + (clipped - self.tone_state) * (tone_ratio * 0.9);

            output[i] = toned * out_gain;
        }
    }
}
