//! crates/ampforge-dsp/src/fx/drive.rs
//! Pedais de Overdrive, Distorção e Fuzz White-Box com Topologias Distintas e Troca de Diodos (Spec 023 & Spec 032)

use crate::unidades::volts_para_fs;

/// Largura de banda de ganho unitário do amp-op LM308 compensado com Cc = 30 pF no ProCo RAT (AUD-29).
pub const DECISAO_SONIX_LM308_GBW: f32 = 1_000_000.0;

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
        let y = crate::denormal::flush_f32(
            self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2,
        );
        self.x2 = self.x1;
        self.x1 = crate::denormal::flush_f32(x);
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
        let y = crate::denormal::flush_f64(
            self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2,
        );
        self.x2 = self.x1;
        self.x1 = crate::denormal::flush_f64(x);
        self.y2 = self.y1;
        self.y1 = if y.is_finite() { y } else { 0.0 };
        self.y1
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct DriveIir3Order64 {
    b0: f64,
    b1: f64,
    b2: f64,
    b3: f64,
    a1: f64,
    a2: f64,
    a3: f64,
    z1: f64,
    z2: f64,
    z3: f64,
}

impl DriveIir3Order64 {
    fn update_coeffs(&mut self, b: [f64; 4], a: [f64; 4]) {
        let a0 = a[0];
        self.b0 = b[0] / a0;
        self.b1 = b[1] / a0;
        self.b2 = b[2] / a0;
        self.b3 = b[3] / a0;
        self.a1 = a[1] / a0;
        self.a2 = a[2] / a0;
        self.a3 = a[3] / a0;
    }

    #[inline(always)]
    fn process(&mut self, x: f64) -> f64 {
        let y = crate::denormal::flush_f64(self.b0 * x + self.z1);
        let yf = if y.is_finite() { y } else { 0.0 };
        self.z1 = crate::denormal::flush_f64(self.b1 * x - self.a1 * yf + self.z2);
        self.z2 = crate::denormal::flush_f64(self.b2 * x - self.a2 * yf + self.z3);
        self.z3 = crate::denormal::flush_f64(self.b3 * x - self.a3 * yf);
        yf
    }
}

#[inline]
fn bilinear_poly3(p0: f64, p1: f64, p2: f64, p3: f64, k: f64) -> [f64; 4] {
    let q1 = p1 * k;
    let q2 = p2 * k * k;
    let q3 = p3 * k * k * k;
    [
        p0 + q1 + q2 + q3,
        3.0 * p0 + q1 - q2 - 3.0 * q3,
        3.0 * p0 - q1 - q2 + 3.0 * q3,
        p0 - q1 + q2 - q3,
    ]
}

pub struct DriveProcessor {
    model: DriveModel,
    clipping_diode: ClippingDiode,
    gain: f32,
    tone: f32,
    level: f32,
    bias: f32,
    sample_rate: f32,
    // Estados TS808 (Spec 023, AUD-29: estágio G(s)=1+Zf/Zg e LPF pós-clip 723 Hz @ 16x subpasso)
    ts_fb_bq_os: DriveBiquad64,
    ts_post_lp_os: DriveBiquad64,
    ts_tone_biquad: DriveBiquad,
    // Klon Centaur
    klon_mid_biquad: DriveBiquad,
    klon_treble_biquad: DriveBiquad,
    // ProCo RAT LM308 (AUD-29: G_cl(s) 3ª ordem com GBW LM308 @ 32x subpasso + Filter passivo)
    rat_cl_iir_os: DriveIir3Order64,
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
    // 8x Oversampling Anti-Aliasing com subpassos internos opcionais (Spec 041 CA-5, zero-alloc, 64-bit)
    os_hist: [f32; 3],
    os_u_prev: f64,
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
            ts_fb_bq_os: DriveBiquad64::default(),
            ts_post_lp_os: DriveBiquad64::default(),
            ts_tone_biquad: DriveBiquad::default(),
            klon_mid_biquad: DriveBiquad::default(),
            klon_treble_biquad: DriveBiquad::default(),
            rat_cl_iir_os: DriveIir3Order64::default(),
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
            os_u_prev: 0.0,
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
        self.os_hist = [0.0; 3];
        self.os_u_prev = 0.0;
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
            ClippingDiode::Germanium1N34A => (volts_para_fs(0.30), volts_para_fs(0.30)),
            ClippingDiode::SiliconSymmetrical => (volts_para_fs(0.60), volts_para_fs(0.60)),
            ClippingDiode::AsymmetricalSiLed => (volts_para_fs(1.80), volts_para_fs(0.60)),
            ClippingDiode::MosfetClip => (volts_para_fs(0.75), volts_para_fs(0.75)),
            ClippingDiode::HardOpamp => (volts_para_fs(1.80), volts_para_fs(1.80)),
        }
    }

    fn update_filters(&mut self) {
        let sr = self.sample_rate;
        let g_norm = self.gain / 10.0;
        let t_norm = self.tone / 10.0;
        let sr_os = (sr as f64) * 8.0;

        // 1. TS808 (AUD-29): realimentação Zf/Zg e LPF pós-clip de 1k + 0.22 µF (723,43 Hz) @ 16x (subpasso 2x em 8x OS)
        let k_ts = 2.0 * sr_os * 2.0;
        let d_ts = g_norm.clamp(0.0, 1.0) as f64;
        let rf_ts = 51_000.0 + d_ts * 500_000.0;
        let cf_ts = 51e-12;
        let rg_ts = 4_700.0;
        let cg_ts = 0.047e-6;
        let tau_g = rg_ts * cg_ts;
        let tau_f = rf_ts * cf_ts;
        let dg0 = 1.0 + k_ts * tau_g;
        let dg1 = 1.0 - k_ts * tau_g;
        let df0 = 1.0 + k_ts * tau_f;
        let df1 = 1.0 - k_ts * tau_f;
        let a0_ts = dg0 * df0;
        self.ts_fb_bq_os.b0 = (rf_ts * cg_ts * k_ts) / a0_ts;
        self.ts_fb_bq_os.b1 = 0.0;
        self.ts_fb_bq_os.b2 = -self.ts_fb_bq_os.b0;
        self.ts_fb_bq_os.a1 = (dg0 * df1 + dg1 * df0) / a0_ts;
        self.ts_fb_bq_os.a2 = (dg1 * df1) / a0_ts;

        let tau_lp = 1_000.0 * 0.22e-6;
        let dl0 = 1.0 + k_ts * tau_lp;
        let dl1 = 1.0 - k_ts * tau_lp;
        self.ts_post_lp_os.b0 = 1.0 / dl0;
        self.ts_post_lp_os.b1 = 1.0 / dl0;
        self.ts_post_lp_os.b2 = 0.0;
        self.ts_post_lp_os.a1 = dl1 / dl0;
        self.ts_post_lp_os.a2 = 0.0;

        let tone_gain_db = -7.5 + 1.5 * self.tone;
        self.ts_tone_biquad.update_coeffs(DriveBiquad::high_shelf(1800.0, tone_gain_db, sr));

        // 2. Klon Centaur: hump médio em ~1050 Hz e shelf ativo de Treble em ~2400 Hz
        self.klon_mid_biquad.update_coeffs(DriveBiquad::peaking(1050.0, 1.5 + 3.5 * g_norm, 0.9, sr));
        self.klon_treble_biquad.update_coeffs(DriveBiquad::high_shelf(2400.0, -7.0 + 14.0 * t_norm, sr));

        // 3. ProCo RAT (AUD-29): estágio não-inversor 1+Rd/Zg com GBW LM308 @ 32x (subpasso 4x em 8x OS) + Filter passivo
        let k_rat = 2.0 * sr_os * 4.0;
        let d_rat = (g_norm.clamp(0.0, 1.0)) as f64;
        let rat_taper = if d_rat <= 0.0 {
            0.0
        } else {
            d_rat.powf(3.321928094887362) // Taper logarítmico 10% em meio curso
        };
        let rd_rat = 100_000.0 * rat_taper;
        let c1_rat = 2.2e-6;
        let r1_rat = 47.0;
        let c2_rat = 4.7e-6;
        let r2_rat = 560.0;
        let tau1 = r1_rat * c1_rat;
        let tau2 = r2_rat * c2_rat;
        let d1 = tau1 + tau2;
        let d2 = tau1 * tau2;
        let n1 = d1 + rd_rat * (c1_rat + c2_rat);
        let n2 = d2 + rd_rat * (c1_rat * tau2 + c2_rat * tau1);
        let w_gbw = 2.0 * std::f64::consts::PI * (DECISAO_SONIX_LM308_GBW as f64);
        let c1 = d1 + 1.0 / w_gbw;
        let c2 = d2 + n1 / w_gbw;
        let c3 = n2 / w_gbw;
        let b_rat = bilinear_poly3(1.0, n1, n2, 0.0, k_rat);
        let a_rat = bilinear_poly3(1.0, c1, c2, c3, k_rat);
        self.rat_cl_iir_os.update_coeffs(b_rat, a_rat);

        let rat_fc = 32150.0 * (600.0f32 / 32150.0f32).powf(t_norm);
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
    fn step_oversample_sub(
        os_hist: &mut [f32; 3],
        os_ai_lp: &mut [DriveBiquad64; 4],
        os_aa_lp: &mut [DriveBiquad64; 6],
        os_u_prev: &mut f64,
        substeps: usize,
        x: f32,
        mut shaper: impl FnMut(f64) -> f64,
    ) -> f32 {
        // 4-point Catmull-Rom cubic interpolation to suppress upsampling images before clipping
        let xm1 = os_hist[0];
        let x0 = os_hist[1];
        let x1 = os_hist[2];
        let x2 = x;
        os_hist[0] = x0;
        os_hist[1] = x1;
        os_hist[2] = x2;

        let c0 = x0;
        let c1 = 0.5 * (x1 - xm1);
        let c2 = xm1 - 2.5 * x0 + 2.0 * x1 - 0.5 * x2;
        let c3 = 0.5 * (x2 - xm1) + 1.5 * (x0 - x1);

        let inv_sub = 1.0 / (substeps.max(1) as f64);
        let mut out = 0.0f64;
        for k in 1..=8 {
            let t = (k as f32) * 0.125;
            let mut u = (c0 + t * (c1 + t * (c2 + t * c3))) as f64;
            for ai in os_ai_lp.iter_mut() {
                u = ai.process(u);
            }
            let mut d = if substeps <= 1 {
                *os_u_prev = crate::denormal::flush_f64(u);
                shaper(u)
            } else {
                let u_prev = *os_u_prev;
                *os_u_prev = crate::denormal::flush_f64(u);
                let mut acc = 0.0f64;
                let du = u - u_prev;
                for s in 1..=substeps {
                    let us = u_prev + du * ((s as f64) * inv_sub);
                    acc += shaper(us);
                }
                acc * inv_sub
            };
            for lpf in os_aa_lp.iter_mut() {
                d = lpf.process(d);
            }
            if k == 8 {
                out = d;
            }
        }
        out as f32
    }

    #[inline(always)]
    fn oversample_4x(&mut self, x: f32, mut shaper: impl FnMut(f32) -> f32) -> f32 {
        Self::step_oversample_sub(
            &mut self.os_hist,
            &mut self.os_ai_lp,
            &mut self.os_aa_lp,
            &mut self.os_u_prev,
            1,
            x,
            |u| shaper(u as f32) as f64,
        )
    }

    /// Processador TS808 fiel ao circuito analógico não-inversor (Spec 023, Spec 041, AUD-29):
    /// V_stage = V_in + clip_Vf( (Zf(s)/Zg(s)) * V_in ) -> LPF 723,43 Hz (1k + 0.22 µF) -> Tone -> Level
    fn process_ts808(&mut self, input: &[f32], output: &mut [f32]) {
        let (vf_pos, vf_neg) = self.diode_thresholds();
        let vf_p = (vf_pos as f64).max(1e-6);
        let vf_n = (vf_neg as f64).max(1e-6);
        let out_gain = self.out_level_taper();

        for i in 0..input.len() {
            let clipped = Self::step_oversample_sub(
                &mut self.os_hist,
                &mut self.os_ai_lp,
                &mut self.os_aa_lp,
                &mut self.os_u_prev,
                2,
                input[i],
                |us| {
                    let v_fb = self.ts_fb_bq_os.process(us);
                    let v_clip = if v_fb >= 0.0 {
                        let r = v_fb / vf_p;
                        v_fb / (1.0 + r.powi(8)).powf(0.125)
                    } else {
                        let r = (-v_fb) / vf_n;
                        v_fb / (1.0 + r.powi(8)).powf(0.125)
                    };
                    let v_stage = us + v_clip;
                    self.ts_post_lp_os.process(v_stage)
                },
            );
            let toned = self.ts_tone_biquad.process(clipped);
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

    /// ProCo RAT (AUD-29): ganho G_cl(s) com limite de GBW do LM308 + hard-clip 2 × 1N914 para terra + filtro INVERTIDO
    fn process_rat(&mut self, input: &[f32], output: &mut [f32]) {
        let (vf_p, vf_n) = self.diode_thresholds();
        let vp = (vf_p as f64).max(1e-6);
        let vn = (vf_n as f64).max(1e-6);
        let out_gain = self.out_level_taper();

        for i in 0..input.len() {
            let clipped = Self::step_oversample_sub(
                &mut self.os_hist,
                &mut self.os_ai_lp,
                &mut self.os_aa_lp,
                &mut self.os_u_prev,
                4,
                input[i],
                |us| {
                    let v = self.rat_cl_iir_os.process(us);
                    let vf = if v >= 0.0 { vp } else { vn };
                    let r = v.abs() / vf;
                    v / (1.0 + r.powi(8)).powf(0.125)
                },
            );
            let filtered = self.rat_filter_biquad.process(clipped);
            output[i] = (filtered * out_gain).clamp(-2.5, 2.5);
        }
    }

    /// Electro-Harmonix Big Muff Pi: 3 estágios transistorizados em cascata (16x subpasso) + Tone Stack com notch de médios
    fn process_big_muff(&mut self, input: &[f32], output: &mut [f32]) {
        let g = (self.gain / 10.0).clamp(0.0, 1.0);
        let sustain = 2.5 + g * 18.0;
        let t_blend = (self.tone / 10.0).clamp(0.0, 1.0);
        let (vf_p, vf_n) = self.diode_thresholds();
        let stage_knee = (((vf_p + vf_n) * 0.5).clamp(0.25, 1.2)) as f64;
        let out_gain = self.out_level_taper() * 1.35;

        for i in 0..input.len() {
            let pre = self.muff_hpf_biquad.process(input[i]) * sustain;
            let s = Self::step_oversample_sub(
                &mut self.os_hist,
                &mut self.os_ai_lp,
                &mut self.os_aa_lp,
                &mut self.os_u_prev,
                2,
                pre,
                |mut u| {
                    for st in 0..3 {
                        let drive = if st == 0 { 1.4 } else { 2.2 };
                        u = stage_knee * ((u * drive) / stage_knee).tanh();
                    }
                    u
                },
            );
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

    /// Boss DS-1 (AUD-29): op-amp + hard diode clipping simétrico por 2 × 1N4148 para terra (8x OS) + curva Tone em V
    fn process_ds1(&mut self, input: &[f32], output: &mut [f32]) {
        let g = (self.gain / 10.0).clamp(0.0, 1.0);
        let pre_gain = 3.0 + g * 40.0;
        let vf = volts_para_fs(0.60) as f64;
        let out_gain = self.out_level_taper();

        for i in 0..input.len() {
            let x_hp = self.ds1_pre_hpf.process(input[i]) * pre_gain;
            let clipped = Self::step_oversample_sub(
                &mut self.os_hist,
                &mut self.os_ai_lp,
                &mut self.os_aa_lp,
                &mut self.os_u_prev,
                2,
                x_hp,
                |u64| {
                    let r = u64.abs() / vf;
                    u64 / (1.0 + r.powi(8)).powf(0.125)
                },
            );
            let scooped = self.ds1_mid_notch.process(clipped);
            let toned = self.ds1_tone_shelf.process(scooped);
            output[i] = (toned * out_gain).clamp(-1.5, 1.5);
        }
    }
}
