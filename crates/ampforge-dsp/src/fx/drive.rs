//! crates/ampforge-dsp/src/fx/drive.rs
//! Pedais de Overdrive, Distorção e Fuzz White-Box com Topologias Distintas e Troca de Diodos (Spec 023 & Spec 032)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveModel {
    GreenScreamer808,
    GoldenKloneCentaur,
    BlackRodentRat,
    OrangeBossDs,
    FuzzFaceSiGe,
    BigMuffPi,
}

impl DriveModel {
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "ts808" | "green_screamer_808" => Some(Self::GreenScreamer808),
            "klon" | "klon_centaur" | "golden_klone_centaur" => Some(Self::GoldenKloneCentaur),
            "rat" | "proco_rat" | "black_rodent_rat" => Some(Self::BlackRodentRat),
            "ds1" | "boss_ds1" | "orange_boss_ds" => Some(Self::OrangeBossDs),
            "fuzzface" | "fuzz_face" | "fuzz_face_sige" => Some(Self::FuzzFaceSiGe),
            "muff" | "big_muff_pi" => Some(Self::BigMuffPi),
            _ => None,
        }
    }
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

    #[allow(dead_code)]
    pub fn low_shelf(fc: f32, gain_db: f32, sr: f32) -> Self {
        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * (fc / sr).clamp(0.001, 0.49);
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / std::f32::consts::SQRT_2;
        let sqrt_a_2_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + sqrt_a_2_alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - sqrt_a_2_alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + sqrt_a_2_alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - sqrt_a_2_alpha;

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

    pub fn lowpass(fc: f32, q: f32, sr: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * (fc / sr).clamp(0.0005, 0.49);
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let a0 = 1.0 + alpha;
        Self {
            b0: ((1.0 - cos_w0) * 0.5) / a0,
            b1: (1.0 - cos_w0) / a0,
            b2: ((1.0 - cos_w0) * 0.5) / a0,
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn first_order_lowpass(fc: f32, sr: f32) -> Self {
        let ratio = (fc / sr).clamp(0.0005, 0.48);
        let k = (std::f32::consts::PI * ratio).tan();
        let norm = 1.0 + k;
        let b0 = k / norm;
        Self {
            b0,
            b1: b0,
            b2: 0.0,
            a1: (k - 1.0) / norm,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn first_order_highpass(fc: f32, sr: f32) -> Self {
        let ratio = (fc / sr).clamp(0.0005, 0.48);
        let k = (std::f32::consts::PI * ratio).tan();
        let norm = 1.0 + k;
        let b0 = 1.0 / norm;
        Self {
            b0,
            b1: -b0,
            b2: 0.0,
            a1: (k - 1.0) / norm,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn highpass(fc: f32, q: f32, sr: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * (fc / sr).clamp(0.0005, 0.49);
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let a0 = 1.0 + alpha;
        Self {
            b0: ((1.0 + cos_w0) * 0.5) / a0,
            b1: -(1.0 + cos_w0) / a0,
            b2: ((1.0 + cos_w0) * 0.5) / a0,
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn peaking(fc: f32, gain_db: f32, q: f32, sr: f32) -> Self {
        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * (fc / sr).clamp(0.0005, 0.49);
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let a0 = 1.0 + alpha / a;
        Self {
            b0: (1.0 + alpha * a) / a0,
            b1: (-2.0 * cos_w0) / a0,
            b2: (1.0 - alpha * a) / a0,
            a1: (-2.0 * cos_w0) / a0,
            a2: (1.0 - alpha / a) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn update_coeffs(&mut self, other: Self) {
        self.b0 = other.b0;
        self.b1 = other.b1;
        self.b2 = other.b2;
        self.a1 = other.a1;
        self.a2 = other.a2;
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

#[derive(Debug, Clone, Copy, Default)]
struct DriveBiquad64 {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl DriveBiquad64 {
    #[inline(always)]
    fn process(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = if y.is_finite() { y } else { 0.0 };
        self.y1
    }
}

pub struct DriveProcessor {
    model: DriveModel,
    clipping_diode: ClippingDiode,
    gain: f32,
    tone: f32,
    level: f32,
    bias: f32,
    sample_rate: f32,
    // Estados TS808 (Spec 023)
    ts_gain_biquad: DriveBiquad,
    ts_tone_biquad: DriveBiquad,
    // Klon Centaur
    klon_mid_biquad: DriveBiquad,
    klon_treble_biquad: DriveBiquad,
    // ProCo RAT LM308
    rat_slew_state: f32,
    rat_filter_biquad: DriveBiquad,
    // Big Muff Pi (4 estágios + Passive Tone Stack scoop)
    muff_hpf_biquad: DriveBiquad,
    muff_tone_lp_biquad: DriveBiquad,
    muff_tone_hp_biquad: DriveBiquad,
    // Fuzz Face Ge (sag/bias assimetria)
    ff_sag_env: f32,
    ff_tone_biquad: DriveBiquad,
    // Boss DS-1 (pre-hpf + V-curve tilt)
    ds1_pre_hpf: DriveBiquad,
    ds1_mid_notch: DriveBiquad,
    ds1_tone_shelf: DriveBiquad,
    // 8x Oversampling Anti-Aliasing (Spec 041 CA-5, zero-alloc, 64-bit precision)
    os_hist: [f32; 3],
    os_ai_lp: [DriveBiquad64; 4],
    os_aa_lp: [DriveBiquad64; 6],
}

impl DriveProcessor {
    pub fn new(model: DriveModel) -> Self {
        let mut p = Self {
            model,
            clipping_diode: ClippingDiode::SiliconSymmetrical,
            gain: 5.0,
            tone: 5.0,
            level: 5.0,
            bias: 5.0,
            sample_rate: 48000.0,
            ts_gain_biquad: DriveBiquad::default(),
            ts_tone_biquad: DriveBiquad::default(),
            klon_mid_biquad: DriveBiquad::default(),
            klon_treble_biquad: DriveBiquad::default(),
            rat_slew_state: 0.0,
            rat_filter_biquad: DriveBiquad::default(),
            muff_hpf_biquad: DriveBiquad::default(),
            muff_tone_lp_biquad: DriveBiquad::default(),
            muff_tone_hp_biquad: DriveBiquad::default(),
            ff_sag_env: 0.0,
            ff_tone_biquad: DriveBiquad::default(),
            ds1_pre_hpf: DriveBiquad::default(),
            ds1_mid_notch: DriveBiquad::default(),
            ds1_tone_shelf: DriveBiquad::default(),
            os_hist: [0.0; 3],
            os_ai_lp: [DriveBiquad64::default(); 4],
            os_aa_lp: [DriveBiquad64::default(); 6],
        };
        p.update_filters();
        p
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.update_filters();
    }

    pub fn model(&self) -> DriveModel {
        self.model
    }

    pub fn set_model(&mut self, model: DriveModel) {
        self.model = model;
        self.update_filters();
    }

    pub fn clipping_diode(&self) -> ClippingDiode {
        self.clipping_diode
    }

    pub fn set_clipping_diode(&mut self, diode: ClippingDiode) {
        self.clipping_diode = diode;
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(0.0, 10.0);
        self.update_filters();
    }

    pub fn set_tone(&mut self, tone: f32) {
        self.tone = tone.clamp(0.0, 10.0);
        self.update_filters();
    }

    pub fn set_level(&mut self, level: f32) {
        self.level = level.clamp(0.0, 10.0);
    }

    pub fn bias(&self) -> f32 {
        self.bias
    }

    pub fn set_bias(&mut self, bias: f32) {
        self.bias = bias.clamp(0.0, 10.0);
    }

    fn diode_thresholds(&self) -> (f32, f32) {
        match self.clipping_diode {
            ClippingDiode::Germanium1N34A => (0.24, 0.24),
            ClippingDiode::SiliconSymmetrical => (0.45, 0.45),
            ClippingDiode::AsymmetricalSiLed => (0.45, 1.35),
            ClippingDiode::MosfetClip => (0.72, 0.72),
            ClippingDiode::HardOpamp => (1.80, 1.80),
        }
    }

    fn update_filters(&mut self) {
        let sr = self.sample_rate;
        let g_norm = self.gain / 10.0;
        let t_norm = self.tone / 10.0;

        // 1. TS808: HPF de 1ª ordem em 720 Hz na realimentação não-inversora (4.7k + 47nF, Spec 041 CA-6)
        self.ts_gain_biquad.update_coeffs(DriveBiquad::first_order_highpass(720.0, sr));
        let tone_gain_db = -7.5 + 1.5 * self.tone;
        self.ts_tone_biquad.update_coeffs(DriveBiquad::high_shelf(1800.0, tone_gain_db, sr));

        // 2. Klon Centaur: hump médio em ~1050 Hz e shelf ativo de Treble em ~2400 Hz
        self.klon_mid_biquad.update_coeffs(DriveBiquad::peaking(1050.0, 1.5 + 3.5 * g_norm, 0.9, sr));
        self.klon_treble_biquad.update_coeffs(DriveBiquad::high_shelf(2400.0, -7.0 + 14.0 * t_norm, sr));

        // 3. ProCo RAT: Filter INVERTIDO de 1ª ordem (6 dB/oit, 0 = aberto ~14 kHz, 10 = escuro ~600 Hz)
        let rat_fc = 14000.0 * (600.0f32 / 14000.0f32).powf(t_norm);
        self.rat_filter_biquad.update_coeffs(DriveBiquad::first_order_lowpass(rat_fc, sr));

        // 12th-order Chebyshev Type I decimation LPF (6 biquads, 0.2 dB ripple) @ 8x OS (Spec 041 CA-5)
        let sr_os = (sr as f64) * 8.0;
        let fp = 17800.0f64.min((sr as f64) * 0.37);
        let n_order = 12.0f64;
        let eps = (10.0f64.powf(0.20 / 10.0) - 1.0).sqrt();
        let gamma = ((1.0 / eps) + (1.0 / (eps * eps) + 1.0).sqrt()).ln() / n_order;
        let sinh_g = gamma.sinh();
        let cosh_g = gamma.cosh();
        let k_warp = (std::f64::consts::PI * fp / sr_os).tan();
        let k2 = k_warp * k_warp;
        // 8th-order Butterworth anti-imaging LPF @ 8x OS (suppresses 43 kHz image before clipping)
        let ai_fc = 19000.0f64.min((sr as f64) * 0.40);
        let ai_qs = [0.50979558f64, 0.60134489, 0.89997622, 2.56291545];
        for (idx, &q) in ai_qs.iter().enumerate() {
            let w0 = 2.0 * std::f64::consts::PI * (ai_fc / sr_os).clamp(0.0005, 0.49);
            let cos_w0 = w0.cos();
            let alpha = w0.sin() / (2.0 * q);
            let a0 = 1.0 + alpha;
            self.os_ai_lp[idx].b0 = ((1.0 - cos_w0) * 0.5) / a0;
            self.os_ai_lp[idx].b1 = (1.0 - cos_w0) / a0;
            self.os_ai_lp[idx].b2 = ((1.0 - cos_w0) * 0.5) / a0;
            self.os_ai_lp[idx].a1 = (-2.0 * cos_w0) / a0;
            self.os_ai_lp[idx].a2 = (1.0 - alpha) / a0;
        }
        for idx in 0..6 {
            let theta = std::f64::consts::PI * (2.0 * (idx as f64) + 1.0) / (2.0 * n_order);
            let sigma = -sinh_g * theta.sin();
            let omega = cosh_g * theta.cos();
            let r2 = sigma * sigma + omega * omega;
            let b_ana = -2.0 * sigma;
            let a0 = 1.0 + b_ana * k_warp + r2 * k2;
            let b0 = (r2 * k2) / a0;
            self.os_aa_lp[idx].b0 = b0;
            self.os_aa_lp[idx].b1 = 2.0 * b0;
            self.os_aa_lp[idx].b2 = b0;
            self.os_aa_lp[idx].a1 = (2.0 * (r2 * k2 - 1.0)) / a0;
            self.os_aa_lp[idx].a2 = (1.0 - b_ana * k_warp + r2 * k2) / a0;
        }

        // 4. Big Muff Pi: HPF de entrada + Tone Stack passivo (LP 360 Hz / HP 2200 Hz -> notch ~950 Hz)
        self.muff_hpf_biquad.update_coeffs(DriveBiquad::highpass(95.0, 0.7071, sr));
        self.muff_tone_lp_biquad.update_coeffs(DriveBiquad::lowpass(360.0, 0.65, sr));
        self.muff_tone_hp_biquad.update_coeffs(DriveBiquad::highpass(2200.0, 0.65, sr));

        // 5. Fuzz Face: LPF suave pós-germânio
        let ff_fc = 1400.0 + t_norm * 3200.0;
        self.ff_tone_biquad.update_coeffs(DriveBiquad::lowpass(ff_fc, 0.7071, sr));

        // 6. Boss DS-1: pré-ênfase + V-curve característica
        self.ds1_pre_hpf.update_coeffs(DriveBiquad::highpass(240.0, 0.7071, sr));
        self.ds1_mid_notch.update_coeffs(DriveBiquad::peaking(720.0, -4.5, 0.85, sr));
        self.ds1_tone_shelf.update_coeffs(DriveBiquad::high_shelf(2100.0, -6.5 + 13.0 * t_norm, sr));
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());

        match self.model {
            DriveModel::GreenScreamer808 => self.process_ts808(input, output),
            DriveModel::GoldenKloneCentaur => self.process_klon(input, output),
            DriveModel::BlackRodentRat => self.process_rat(input, output),
            DriveModel::BigMuffPi => self.process_big_muff(input, output),
            DriveModel::FuzzFaceSiGe => self.process_fuzz_face(input, output),
            DriveModel::OrangeBossDs => self.process_ds1(input, output),
        }
    }

    fn out_level_taper(&self) -> f32 {
        if self.level <= 0.001 {
            0.0
        } else {
            (self.level / 10.0).powf(2.0) * 1.5
        }
    }

    #[inline(always)]
    fn oversample_4x(&mut self, x: f32, mut shaper: impl FnMut(f32) -> f32) -> f32 {
        // 4-point Catmull-Rom cubic interpolation to suppress upsampling images before clipping
        let xm1 = self.os_hist[0];
        let x0 = self.os_hist[1];
        let x1 = self.os_hist[2];
        let x2 = x;
        self.os_hist[0] = x0;
        self.os_hist[1] = x1;
        self.os_hist[2] = x2;

        let c0 = x0;
        let c1 = 0.5 * (x1 - xm1);
        let c2 = xm1 - 2.5 * x0 + 2.0 * x1 - 0.5 * x2;
        let c3 = 0.5 * (x2 - xm1) + 1.5 * (x0 - x1);

        let mut out = 0.0f64;
        for k in 1..=8 {
            let t = (k as f32) * 0.125;
            let mut u = (c0 + t * (c1 + t * (c2 + t * c3))) as f64;
            for ai in &mut self.os_ai_lp {
                u = ai.process(u);
            }
            let mut d = shaper(u as f32) as f64;
            for lpf in &mut self.os_aa_lp {
                d = lpf.process(d);
            }
            if k == 8 {
                out = d;
            }
        }
        out as f32
    }

    /// Processador TS808 fiel ao circuito analógico não-inversor (Spec 023, Spec 041 CA-5/CA-6):
    /// V_out = V_in + V_clipped(HP_720(V_in) * drive_gain)
    fn process_ts808(&mut self, input: &[f32], output: &mut [f32]) {
        let (vf_pos, vf_neg) = match self.clipping_diode {
            ClippingDiode::Germanium1N34A => (0.20f32, 0.20f32),
            ClippingDiode::SiliconSymmetrical => (0.35f32, 0.35f32),
            ClippingDiode::AsymmetricalSiLed => (1.20f32, 0.35f32),
            ClippingDiode::MosfetClip => (0.60f32, 0.60f32),
            ClippingDiode::HardOpamp => (1.80f32, 1.80f32),
        };
        let drive_gain = 0.8 + (self.gain / 10.0).powf(1.6) * 14.0;
        let out_gain = self.out_level_taper();

        for i in 0..input.len() {
            let x = input[i];
            let x_hp = self.ts_gain_biquad.process(x);
            let clipped = self.oversample_4x(x_hp, |u| {
                let d = u * drive_gain;
                if d.abs() < 1e-12 {
                    0.0
                } else if d >= 0.0 {
                    vf_pos * (d / vf_pos).tanh()
                } else {
                    -vf_neg * ((-d) / vf_neg).tanh()
                }
            });
            let summed = x + clipped;
            let toned = self.ts_tone_biquad.process(summed);
            output[i] = toned * out_gain;
        }
    }

    /// Klon Centaur: potenciômetro duplo clean-blend + clipping suave de germânio + 18V charge pump headroom
    fn process_klon(&mut self, input: &[f32], output: &mut [f32]) {
        let g = (self.gain / 10.0).clamp(0.0, 1.0);
        let clean_blend = (1.0 - g * 0.78).clamp(0.12, 1.0);
        let drive_blend = (0.18 + g * 0.82).clamp(0.18, 1.0);
        let pre_gain = 1.0 + g * 16.0;
        let (vf_p, vf_n) = self.diode_thresholds();
        // Klon possui headroom estendido + diodos germânio suaves com assimetria branda
        let vp = (vf_p * 1.45).max(0.40);
        let vn = (vf_n * 1.60).max(0.45);
        let out_gain = self.out_level_taper() * 1.15;

        for i in 0..input.len() {
            let x = input[i];
            let shaped = self.klon_mid_biquad.process(x) * pre_gain;
            let dirty = self.oversample_4x(shaped, |u| {
                if u >= 0.0 {
                    vp * (u / vp).tanh()
                } else {
                    -vn * ((-u) / vn).tanh()
                }
            });
            let summed = x * clean_blend + (dirty * 0.55) * drive_blend;
            let toned = self.klon_treble_biquad.process(summed);
            output[i] = (toned * out_gain).clamp(-1.5, 1.5);
        }
    }

    /// ProCo RAT: slew-rate limiting LM308 + hard-clipping para terra + filtro passivo INVERTIDO (1ª ordem)
    fn process_rat(&mut self, input: &[f32], output: &mut [f32]) {
        let g = (self.gain / 10.0).clamp(0.0, 1.0);
        let pre_gain = 2.0 + g * 38.0;
        let (vf_p, vf_n) = self.diode_thresholds();
        let vp = (vf_p * 0.72).max(0.22);
        let vn = (vf_n * 0.78).max(0.24);
        // Slew limit escalado ao sample rate 8x OS (~0.9 V/us relativo ao ganho)
        let max_slew = 1.2 * (48000.0 / self.sample_rate).clamp(0.25, 2.0);
        let out_gain = self.out_level_taper();

        for i in 0..input.len() {
            let amplified = input[i] * pre_gain;
            let mut slew_state = self.rat_slew_state;
            let clipped = self.oversample_4x(amplified, |u| {
                let raw_delta = u - slew_state;
                let delta = max_slew * (raw_delta / max_slew).tanh();
                slew_state = slew_state * 0.999 + delta;
                let slewed = slew_state;
                if slewed >= 0.0 {
                    let v = slewed / vp;
                    vp * (v / (1.0 + v * v).sqrt()) * 1.15
                } else {
                    let v = (-slewed) / vn;
                    -vn * (v / (1.0 + v * v).sqrt()) * 1.15
                }
            });
            self.rat_slew_state = slew_state;

            let filtered = self.rat_filter_biquad.process(clipped);
            output[i] = (filtered * out_gain).clamp(-1.5, 1.5);
        }
    }

    /// Electro-Harmonix Big Muff Pi: 3 estágios transistorizados em cascata (4x OS) + Tone Stack com notch de médios
    fn process_big_muff(&mut self, input: &[f32], output: &mut [f32]) {
        let g = (self.gain / 10.0).clamp(0.0, 1.0);
        let sustain = 2.5 + g * 18.0;
        let t_blend = (self.tone / 10.0).clamp(0.0, 1.0);
        let (vf_p, vf_n) = self.diode_thresholds();
        let stage_knee = ((vf_p + vf_n) * 0.5).clamp(0.25, 1.2);
        let out_gain = self.out_level_taper() * 1.35;

        for i in 0..input.len() {
            let pre = self.muff_hpf_biquad.process(input[i]) * sustain;
            let s = self.oversample_4x(pre, |mut u| {
                for st in 0..3 {
                    let drive = if st == 0 { 1.4 } else { 2.2 };
                    u = stage_knee * ((u * drive) / stage_knee).tanh();
                }
                u
            });
            // Passive twin RC Tone Stack (LP + HP blend gera notch natural ~950 Hz em Tone = 5.0)
            let lp = self.muff_tone_lp_biquad.process(s);
            let hp = self.muff_tone_hp_biquad.process(s);
            let stacked = (1.0 - t_blend) * lp + t_blend * hp;
            output[i] = (stacked * out_gain).clamp(-1.5, 1.5);
        }
    }

    /// Dallas Arbiter Fuzz Face Ge: 2 estágios PNP acoplados com assimetria de germânio e Bias Trim (4x OS)
    fn process_fuzz_face(&mut self, input: &[f32], output: &mut [f32]) {
        let g = (self.gain / 10.0).clamp(0.0, 1.0);
        let pre_gain = 1.6 + g * 12.0;
        // Bias (5.0 = ponto doce assimétrico clássico; < 3.0 ou > 7.5 aumenta assimetria e gating "velcro")
        let bias_offset = (self.bias - 5.0) * 0.14 + 0.12;
        let gate_floor = ((self.bias - 5.0).abs() - 1.5).max(0.0) * 0.018;
        let rel_coeff = (-1.0 / (self.sample_rate * 0.018)).exp();
        let out_gain = self.out_level_taper() * 1.15;

        for i in 0..input.len() {
            let x = input[i];
            let ax = x.abs();
            self.ff_sag_env = self.ff_sag_env * rel_coeff + ax * (1.0 - rel_coeff);

            let gated_in = if gate_floor > 0.0 && ax < gate_floor {
                x * (ax / gate_floor)
            } else {
                x
            };

            let u_in = gated_in * pre_gain;
            let dc_shift = bias_offset + 0.10 * self.ff_sag_env;
            let fuzz = self.oversample_4x(u_in, |u| {
                let clip = |v: f32| -> f32 {
                    if v >= 0.0 {
                        0.55 * (v / 0.55).tanh()
                    } else {
                        -0.16 * ((-v) / 0.16).tanh()
                    }
                };
                clip(u + dc_shift) - clip(dc_shift)
            });
            let toned = self.ff_tone_biquad.process(fuzz);
            output[i] = (toned * out_gain).clamp(-1.5, 1.5);
        }
    }

    /// Boss DS-1: op-amp + hard diode clipping assimétrico (4x OS) + curva de Tone em V
    fn process_ds1(&mut self, input: &[f32], output: &mut [f32]) {
        let g = (self.gain / 10.0).clamp(0.0, 1.0);
        let pre_gain = 3.0 + g * 40.0;
        let (vf_p, vf_n) = self.diode_thresholds();
        let vp = (vf_p * 0.85).clamp(0.25, 1.2);
        let vn = (vf_n * 1.15).clamp(0.32, 1.6);
        let out_gain = self.out_level_taper();

        for i in 0..input.len() {
            let x_hp = self.ds1_pre_hpf.process(input[i]) * pre_gain;
            let clipped = self.oversample_4x(x_hp, |u_in| {
                if u_in >= 0.0 {
                    let u = u_in / vp;
                    vp * (u / (1.0 + u.abs() * 0.85))
                } else {
                    let u = (-u_in) / vn;
                    -vn * (u / (1.0 + u.abs() * 0.95))
                }
            });
            let scooped = self.ds1_mid_notch.process(clipped);
            let toned = self.ds1_tone_shelf.process(scooped);
            output[i] = (toned * out_gain).clamp(-1.5, 1.5);
        }
    }
}
