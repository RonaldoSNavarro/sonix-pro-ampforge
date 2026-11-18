//! crates/ampforge-dsp/src/amp.rs
//! Modelagem White-Box de Amplificadores Valvulados e Painel Frontal Autêntico

use crate::modshop::{PowerTube, PreampTube, RectifierType, ToneStackTopology};
use crate::tonestack::PassiveToneStack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmpModelId {
    BritJcm800_2203,
    BritPlexi1959,
    UsTwinBlackface65,
    CaliDualRectoRevF,
    UkAc30TopBoost,
}

impl AmpModelId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BritJcm800_2203 => "jcm800_2203",
            Self::BritPlexi1959 => "plexi_1959",
            Self::UsTwinBlackface65 => "twin_ab763",
            Self::CaliDualRectoRevF => "dual_rect",
            Self::UkAc30TopBoost => "ac30_tb",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmpInputJack {
    High,
    Low,
    High1,
    Low1,
    High2,
    Low2,
    Jumpered,
    NormalHigh,
    NormalLow,
    BrilliantHigh,
    BrilliantLow,
    Normal,
    Vibrato,
}

impl AmpInputJack {
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "high" => Some(Self::High),
            "low" => Some(Self::Low),
            "high1" => Some(Self::High1),
            "low1" => Some(Self::Low1),
            "high2" => Some(Self::High2),
            "low2" => Some(Self::Low2),
            "jumpered" => Some(Self::Jumpered),
            "normal_high" => Some(Self::NormalHigh),
            "normal_low" => Some(Self::NormalLow),
            "brilliant_high" => Some(Self::BrilliantHigh),
            "brilliant_low" => Some(Self::BrilliantLow),
            "normal" => Some(Self::Normal),
            "vibrato" => Some(Self::Vibrato),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Low => "low",
            Self::High1 => "high1",
            Self::Low1 => "low1",
            Self::High2 => "high2",
            Self::Low2 => "low2",
            Self::Jumpered => "jumpered",
            Self::NormalHigh => "normal_high",
            Self::NormalLow => "normal_low",
            Self::BrilliantHigh => "brilliant_high",
            Self::BrilliantLow => "brilliant_low",
            Self::Normal => "normal",
            Self::Vibrato => "vibrato",
        }
    }

    #[inline]
    pub fn is_low_pad(&self) -> bool {
        matches!(
            self,
            Self::Low | Self::Low1 | Self::Low2 | Self::NormalLow | Self::BrilliantLow
        )
    }

    #[inline]
    pub fn is_normal_channel(&self) -> bool {
        matches!(
            self,
            Self::Normal | Self::NormalHigh | Self::NormalLow | Self::High2 | Self::Low2
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmpChannelMode {
    Clean,
    Pushed,
    Vintage,
    Modern,
}

impl AmpChannelMode {
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "clean" => Some(Self::Clean),
            "pushed" => Some(Self::Pushed),
            "vintage" => Some(Self::Vintage),
            "modern" => Some(Self::Modern),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Pushed => "pushed",
            Self::Vintage => "vintage",
            Self::Modern => "modern",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmpPanelTheme {
    BrushedBrass,
    BrushedChrome,
    BlackoutMetal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmpChannelVoicing {
    Norm,
    HighLead,
}

#[derive(Debug, Clone)]
pub struct AmpModelDescriptor {
    pub id: AmpModelId,
    pub name: &'static str,
    pub brand_text: &'static str,
    pub panel_theme: AmpPanelTheme,
}

#[derive(Debug, Clone)]
pub struct AmpFrontPanel {
    pub input_gain: f32,
    pub overdrive: f32,
    pub bass: f32,
    pub middle: f32,
    pub treble: f32,
    pub presence: f32,
    pub depth: f32,
    pub master_volume: f32,
    pub output_level_db: f32,
    pub bright_switch: bool,
    pub channel_toggle: AmpChannelVoicing,
    pub volume_2: f32,
    pub cut: f32,
    pub reverb: f32,
    pub vibrato_speed: f32,
    pub vibrato_depth: f32,
    pub power: bool,
    pub standby: bool,
    pub jumpered: bool,
    pub spongy: bool,
    pub input_jack: AmpInputJack,
    pub channel_mode: AmpChannelMode,
    pub impedance_ohms: u8,
}

impl Default for AmpFrontPanel {
    fn default() -> Self {
        Self {
            input_gain: 5.0,
            overdrive: 5.0,
            bass: 5.0,
            middle: 5.0,
            treble: 5.0,
            presence: 5.0,
            depth: 5.0,
            master_volume: 5.0,
            output_level_db: 0.0,
            bright_switch: false,
            channel_toggle: AmpChannelVoicing::Norm,
            volume_2: 0.0,
            cut: 5.0,
            reverb: 0.0,
            vibrato_speed: 4.0,
            vibrato_depth: 0.0,
            power: true,
            standby: false,
            jumpered: false,
            spongy: false,
            input_jack: AmpInputJack::High,
            channel_mode: AmpChannelMode::Modern,
            impedance_ohms: 16,
        }
    }
}

impl AmpFrontPanel {
    fn sanitize_val(val: f32, min: f32, max: f32, default_val: f32) -> f32 {
        if val.is_nan() {
            default_val
        } else {
            val.clamp(min, max)
        }
    }

    pub fn set_input_gain(&mut self, val: f32) {
        self.input_gain = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_overdrive(&mut self, val: f32) {
        self.overdrive = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_bass(&mut self, val: f32) {
        self.bass = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_middle(&mut self, val: f32) {
        self.middle = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_treble(&mut self, val: f32) {
        self.treble = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_presence(&mut self, val: f32) {
        self.presence = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_depth(&mut self, val: f32) {
        self.depth = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_master_volume(&mut self, val: f32) {
        self.master_volume = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_output_level_db(&mut self, val: f32) {
        self.output_level_db = Self::sanitize_val(val, -24.0, 12.0, 0.0);
    }

    pub fn set_volume_2(&mut self, val: f32) {
        self.volume_2 = Self::sanitize_val(val, 0.0, 10.0, 0.0);
    }

    pub fn set_cut(&mut self, val: f32) {
        self.cut = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_reverb(&mut self, val: f32) {
        self.reverb = Self::sanitize_val(val, 0.0, 10.0, 0.0);
    }

    pub fn set_vibrato_speed(&mut self, val: f32) {
        self.vibrato_speed = Self::sanitize_val(val, 0.0, 10.0, 4.0);
    }

    pub fn set_vibrato_depth(&mut self, val: f32) {
        self.vibrato_depth = Self::sanitize_val(val, 0.0, 10.0, 0.0);
    }
}

#[derive(Debug, Clone)]
pub struct ChassisModSettings {
    pub preamp_tube: PreampTube,
    pub power_tube: PowerTube,
    pub bias_ma: f32,
    pub sag: f32,
    pub rectifier: RectifierType,
    pub nfb: f32,
    pub xformer_sat: f32,
    pub preamp_plate_v: f32,
    pub triode_asym: f32,
    pub tonestack: ToneStackTopology,
    pub b_plus_voltage: f32,
    pub cathode_bias_resistor_ohm: f32,
}

impl Default for ChassisModSettings {
    fn default() -> Self {
        nominal_mod_settings_for_model(AmpModelId::BritJcm800_2203)
    }
}

pub fn nominal_mod_settings_for_model(model: AmpModelId) -> ChassisModSettings {
    match model {
        AmpModelId::BritJcm800_2203 => ChassisModSettings {
            preamp_tube: PreampTube::Ax7A,
            power_tube: PowerTube::El34,
            bias_ma: 38.0,
            sag: 3.0,
            rectifier: RectifierType::SolidStateDiode,
            nfb: 5.0,
            xformer_sat: 4.0,
            preamp_plate_v: 280.0,
            triode_asym: 0.20,
            tonestack: ToneStackTopology::Model,
            b_plus_voltage: 450.0,
            cathode_bias_resistor_ohm: 1500.0,
        },
        AmpModelId::BritPlexi1959 => ChassisModSettings {
            preamp_tube: PreampTube::Ax7A,
            power_tube: PowerTube::El34,
            bias_ma: 36.0,
            sag: 4.0,
            rectifier: RectifierType::SolidStateDiode,
            nfb: 6.0,
            xformer_sat: 5.0,
            preamp_plate_v: 250.0,
            triode_asym: 0.22,
            tonestack: ToneStackTopology::Model,
            b_plus_voltage: 460.0,
            cathode_bias_resistor_ohm: 1500.0,
        },
        AmpModelId::UsTwinBlackface65 => ChassisModSettings {
            preamp_tube: PreampTube::Ax7A,
            power_tube: PowerTube::Tube6L6Gc,
            bias_ma: 35.0,
            sag: 2.0,
            rectifier: RectifierType::SolidStateDiode,
            nfb: 8.0,
            xformer_sat: 2.0,
            preamp_plate_v: 310.0,
            triode_asym: 0.15,
            tonestack: ToneStackTopology::Model,
            b_plus_voltage: 440.0,
            cathode_bias_resistor_ohm: 1500.0,
        },
        AmpModelId::CaliDualRectoRevF => ChassisModSettings {
            preamp_tube: PreampTube::Ax7A,
            power_tube: PowerTube::Tube6L6Gc,
            bias_ma: 32.0,
            sag: 5.0,
            rectifier: RectifierType::Dual5U4Gb,
            nfb: 3.0,
            xformer_sat: 3.0,
            preamp_plate_v: 290.0,
            triode_asym: 0.28,
            tonestack: ToneStackTopology::Model,
            b_plus_voltage: 470.0,
            cathode_bias_resistor_ohm: 1500.0,
        },
        AmpModelId::UkAc30TopBoost => ChassisModSettings {
            preamp_tube: PreampTube::Ax7A,
            power_tube: PowerTube::El84,
            bias_ma: 45.0,
            sag: 6.0,
            rectifier: RectifierType::Gz34_5Ar4,
            nfb: 0.0,
            xformer_sat: 5.0,
            preamp_plate_v: 240.0,
            triode_asym: 0.18,
            tonestack: ToneStackTopology::Model,
            b_plus_voltage: 330.0,
            cathode_bias_resistor_ohm: 50.0,
        },
    }
}

pub struct AmpModelCatalog {
    models: Vec<AmpModelDescriptor>,
}

impl Default for AmpModelCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl AmpModelCatalog {
    pub fn new() -> Self {
        Self {
            models: vec![
                AmpModelDescriptor {
                    id: AmpModelId::BritJcm800_2203,
                    name: "brit_jcm800_2203",
                    brand_text: "MARSHALL™ / BRIT 800",
                    panel_theme: AmpPanelTheme::BrushedBrass,
                },
                AmpModelDescriptor {
                    id: AmpModelId::BritPlexi1959,
                    name: "brit_plexi_1959",
                    brand_text: "PLEXI 1959 SUPER LEAD",
                    panel_theme: AmpPanelTheme::BrushedBrass,
                },
                AmpModelDescriptor {
                    id: AmpModelId::UsTwinBlackface65,
                    name: "us_twin_blackface_65",
                    brand_text: "US '65 TWIN REVERB",
                    panel_theme: AmpPanelTheme::BrushedChrome,
                },
                AmpModelDescriptor {
                    id: AmpModelId::CaliDualRectoRevF,
                    name: "cali_dual_recto_revf",
                    brand_text: "CALI RECTIFIER REV-F",
                    panel_theme: AmpPanelTheme::BlackoutMetal,
                },
                AmpModelDescriptor {
                    id: AmpModelId::UkAc30TopBoost,
                    name: "uk_ac30_topboost",
                    brand_text: "UK AC30 TOP BOOST",
                    panel_theme: AmpPanelTheme::BrushedBrass,
                },
            ],
        }
    }

    pub fn get_descriptor(&self, id: AmpModelId) -> &AmpModelDescriptor {
        self.models
            .iter()
            .find(|m| m.id == id)
            .expect("Modelo de amplificador desconhecido")
    }

    pub fn find_model_by_name(&self, name: &str) -> Result<AmpModelId, String> {
        find_model_by_name(name).ok_or_else(|| {
            self.models
                .iter()
                .find(|m| m.name == name)
                .map(|m| m.id)
                .ok_or_else(|| format!("Modelo de amplificador '{}' não encontrado", name))
                .err()
                .unwrap_or_default()
        })
    }
}

pub fn find_model_by_name(name: &str) -> Option<AmpModelId> {
    match name {
        "jcm800_2203" | "brit_jcm800_2203" | "brit800" => Some(AmpModelId::BritJcm800_2203),
        "plexi_1959" | "brit_plexi_1959" | "plexi" => Some(AmpModelId::BritPlexi1959),
        "twin_ab763" | "us_twin_blackface_65" | "twin_reverb" | "usdeluxe" => Some(AmpModelId::UsTwinBlackface65),
        "dual_rect" | "cali_dual_recto_revf" | "dual_rectifier" | "rectifier_solo_100" | "recto" => Some(AmpModelId::CaliDualRectoRevF),
        "ac30_tb" | "uk_ac30_topboost" | "ac30" | "ac30_top_boost" => Some(AmpModelId::UkAc30TopBoost),
        _ => None,
    }
}

pub struct WhiteBoxAmpProcessor {
    pub model_id: AmpModelId,
    panel: AmpFrontPanel,
    mod_settings: ChassisModSettings,
    sample_rate: f32,
    // Internal state variables for IIR filters
    bright_state: f64,
    bright2_state: f64,
    interstage_hp1: f64,
    interstage_hp2: f64,
    interstage_hp3: f64,
    miller_lp1: f64,
    ac30_cut_lp: f64,
    passive_tonestack: PassiveToneStack,
    depth_filter: BiquadFilter,
    presence_filter: BiquadFilter,
    voicing_filter1: BiquadFilter,
    voicing_filter2: BiquadFilter,
    adaa_pre_prev: f64,
    adaa_s2_prev: f64,
    adaa_s3_prev: f64,
    adaa_s4_prev: f64,
    adaa_power_prev: f64,
    interp_hist: [f64; 8],
    aa_filters: [BiquadFilter; 14],
    ai_filters: [BiquadFilter; 4],
    target_model_id: Option<AmpModelId>,
    target_mod_settings: Option<ChassisModSettings>,
    target_voicing_filter1: BiquadFilter,
    target_voicing_filter2: BiquadFilter,
    model_crossfade: f32,
    crossfade_step: f32,
    oversampling_factor: usize,
    sag_state: f64,
    xformer_lf_state: f64,
    speaker_res_filter: BiquadFilter,
    speaker_resonance_hz: f32,
    lfo_phase: f64,
    standby_env: f64,
    smooth_tube_gain: f64,
    spring_reverb: SpringReverbTank,
}

#[derive(Debug, Clone)]
struct SpringReverbTank {
    lines: [[f64; 2048]; 4],
    lens: [usize; 4],
    idx: [usize; 4],
    lp: [f64; 4],
    lp_alpha: f64,
}

impl Default for SpringReverbTank {
    fn default() -> Self {
        Self {
            lines: [[0.0; 2048]; 4],
            lens: [347, 509, 683, 887],
            idx: [0; 4],
            lp: [0.0; 4],
            lp_alpha: 1.0 - (-2.0 * std::f64::consts::PI * 3650.0 / 48000.0).exp(),
        }
    }
}

impl SpringReverbTank {
    fn set_sample_rate(&mut self, sr: f32) {
        let fs = (sr as f64).max(1.0);
        let scale = (fs / 48000.0).clamp(0.5, 2.0);
        self.lp_alpha = (1.0 - (-2.0 * std::f64::consts::PI * 3650.0 / fs).exp()).clamp(0.01, 0.99);
        let base = [347.0, 509.0, 683.0, 887.0];
        for i in 0..4 {
            self.lens[i] = ((base[i] * scale).round() as usize).clamp(64, 2047);
            if self.idx[i] >= self.lens[i] {
                self.idx[i] = 0;
            }
        }
    }

    #[inline(always)]
    fn process_sample(&mut self, x: f64) -> f64 {
        let mut d = [0.0f64; 4];
        for k in 0..4 {
            let p = self.idx[k];
            let raw = self.lines[k][p];
            // High-frequency spring dispersion & damping
            self.lp[k] += self.lp_alpha * (raw - self.lp[k]);
            d[k] = self.lp[k];
        }
        // Hadamard scattering matrix with g = 0.895 for authentic long AB763 spring decay
        let s_all = 0.5 * (d[0] + d[1] + d[2] + d[3]);
        let g = 0.895;
        for k in 0..4 {
            let fb = g * (d[k] - s_all);
            let p = self.idx[k];
            self.lines[k][p] = (x * 0.65 + fb).clamp(-2.0, 2.0);
            self.idx[k] = if p + 1 >= self.lens[k] { 0 } else { p + 1 };
        }
        0.25 * (d[0] - d[1] + d[2] - d[3])
    }
}

#[inline(always)]
fn antideriv_triode_branch(w: f64, k: f64) -> f64 {
    let kw = k * w;
    if kw < 1e-4 {
        let w2 = w * w;
        0.5 * w2 - 0.125 * k * k * w2 * w2
    } else {
        ((1.0 + kw * kw).sqrt() - 1.0) / (k * k)
    }
}

#[inline(always)]
fn antideriv_triode(u: f64, k_p: f64, k_n: f64) -> f64 {
    if u >= 0.0 {
        antideriv_triode_branch(u, k_p)
    } else {
        antideriv_triode_branch(-u, k_n)
    }
}

#[inline(always)]
fn triode_adaa1(x: f64, prev: &mut f64, k_p: f64, k_n: f64) -> f64 {
    let dx = x - *prev;
    let y = if dx.abs() < 1e-5 {
        let mid = 0.5 * (x + *prev);
        let k = if mid >= 0.0 { k_p } else { k_n };
        mid / (1.0 + (k * mid) * (k * mid)).sqrt()
    } else {
        (antideriv_triode(x, k_p, k_n) - antideriv_triode(*prev, k_p, k_n)) / dx
    };
    *prev = x;
    y
}

/// Evaluates the static first-stage preamp triode transfer curve matching `AmpBlock::process`.
pub fn static_preamp_transfer(
    x: f64,
    model: AmpModelId,
    preamp_tube: PreampTube,
    asym: f64,
    plate_v: f64,
) -> f64 {
    let (tube_gain, ef86_knee) = match preamp_tube {
        PreampTube::Ax7A => (1.0f64, 1.0f64),
        PreampTube::At7 => (0.58f64, 0.85f64),
        PreampTube::Ay7 => (0.45f64, 0.78f64),
        PreampTube::Ef86 => (1.22f64, 1.35f64),
    };
    let v1_headroom = (plate_v / 250.0).max(0.1);
    let kp_base = (1.0 + 1.2 * asym) * ef86_knee;
    let kn_base = ((1.0 - 0.55 * asym) * ef86_knee).max(0.35);

    let (u, kp, kn, out_scale) = match model {
        AmpModelId::BritPlexi1959 => ((x * tube_gain) / v1_headroom, kp_base, kn_base, v1_headroom),
        AmpModelId::BritJcm800_2203 => (
            (x * 1.15 * tube_gain) / v1_headroom,
            kp_base * 1.15,
            kn_base * 0.90,
            v1_headroom,
        ),
        AmpModelId::UsTwinBlackface65 => {
            let th = v1_headroom * 1.35;
            ((x * tube_gain) / th, kp_base * 0.85, kn_base, th)
        }
        AmpModelId::CaliDualRectoRevF => (
            (x * 1.25 * tube_gain) / v1_headroom,
            kp_base * 1.25,
            kn_base * 0.85,
            v1_headroom,
        ),
        AmpModelId::UkAc30TopBoost => (
            (x * tube_gain) / v1_headroom,
            kp_base * 1.05,
            kn_base * 0.92,
            v1_headroom,
        ),
    };
    let k = if asym <= 1e-4 || u >= 0.0 { kp } else { kn };
    (u / (1.0 + (k * u) * (k * u)).sqrt()) * out_scale
}

#[inline(always)]
fn one_pole_hpf(state: &mut f64, x: f64, alpha: f64) -> f64 {
    let hp = x - *state;
    *state += alpha * hp;
    hp
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BiquadFilter {
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

impl BiquadFilter {
    #[inline(always)]
    pub fn update_coeffs(&mut self, other: BiquadFilter) {
        self.b0 = other.b0;
        self.b1 = other.b1;
        self.b2 = other.b2;
        self.a1 = other.a1;
        self.a2 = other.a2;
    }

    #[inline(always)]
    pub fn process(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    pub fn lowpass(freq_hz: f64, q: f64, sample_rate: f64) -> Self {
        let w0 = 2.0 * std::f64::consts::PI * (freq_hz / sample_rate).clamp(0.0001, 0.499);
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let cos_w = w0.cos();
        let b0 = (1.0 - cos_w) / 2.0;
        let b1 = 1.0 - cos_w;
        let b2 = (1.0 - cos_w) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha;
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

    pub fn notch(freq_hz: f64, q: f64, sample_rate: f64) -> Self {
        let w0 = 2.0 * std::f64::consts::PI * (freq_hz / sample_rate).clamp(0.0001, 0.499);
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let cos_w = w0.cos();
        let b0 = 1.0;
        let b1 = -2.0 * cos_w;
        let b2 = 1.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha;
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

    pub fn peaking(freq_hz: f64, gain_db: f64, q: f64, sample_rate: f64) -> Self {
        let a = 10.0f64.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f64::consts::PI * (freq_hz / sample_rate).clamp(0.0001, 0.499);
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let cos_w0 = w0.cos();

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a;

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

    pub fn low_shelf(freq_hz: f64, gain_db: f64, sample_rate: f64) -> Self {
        let a = 10.0f64.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f64::consts::PI * (freq_hz / sample_rate).clamp(0.0001, 0.499);
        let alpha = w0.sin() / 2.0 * 2.0f64.sqrt();
        let cos_w0 = w0.cos();

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha;

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

    pub fn high_shelf(freq_hz: f64, gain_db: f64, sample_rate: f64) -> Self {
        let a = 10.0f64.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f64::consts::PI * (freq_hz / sample_rate).clamp(0.0001, 0.499);
        let alpha = w0.sin() / 2.0 * 2.0f64.sqrt();
        let cos_w0 = w0.cos();

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha;

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
}

const SINC_WEIGHTS: [[f64; 8]; 8] = [
    [  0.00000000,  -0.00000000,   0.00000000,   1.00000000,   0.00000000,  -0.00000000,   0.00000000,   0.00000000], // step 0
    [ -0.00191376,   0.01676671,  -0.07812250,   0.97053650,   0.11443573,  -0.02536934,   0.00369401,  -0.00002736], // step 1
    [ -0.00241474,   0.02485426,  -0.12008976,   0.88586592,   0.25990285,  -0.05703832,   0.00913086,  -0.00021106], // step 2
    [ -0.00204519,   0.02578259,  -0.13065782,   0.75632385,   0.42577557,  -0.09018572,   0.01565918,  -0.00065248], // step 3
    [ -0.00132955,   0.02189682,  -0.11764955,   0.59708228,   0.59708228,  -0.11764955,   0.02189682,  -0.00132955], // step 4
    [ -0.00065248,   0.01565918,  -0.09018572,   0.42577557,   0.75632385,  -0.13065782,   0.02578259,  -0.00204519], // step 5
    [ -0.00021106,   0.00913086,  -0.05703832,   0.25990285,   0.88586592,  -0.12008976,   0.02485426,  -0.00241474], // step 6
    [ -0.00002736,   0.00369401,  -0.02536934,   0.11443573,   0.97053650,  -0.07812250,   0.01676671,  -0.00191376], // step 7
];

impl WhiteBoxAmpProcessor {
    pub fn create_voicing_filters(model: AmpModelId, sr: f64) -> (BiquadFilter, BiquadFilter) {
        // Curvas físicas documentadas em docs/amps/voicing-fontes.md
        match model {
            AmpModelId::BritJcm800_2203 => (
                BiquadFilter::peaking(1600.0, 2.0, 1.1, sr),
                BiquadFilter::low_shelf(110.0, -1.5, sr),
            ),
            AmpModelId::BritPlexi1959 => (
                BiquadFilter::peaking(2400.0, 2.5, 0.9, sr),
                BiquadFilter::low_shelf(120.0, 1.5, sr),
            ),
            AmpModelId::UsTwinBlackface65 => (
                BiquadFilter::high_shelf(4500.0, 1.5, sr),
                BiquadFilter::low_shelf(80.0, 0.5, sr),
            ),
            AmpModelId::CaliDualRectoRevF => (
                BiquadFilter::peaking(85.0, 2.0, 1.4, sr),
                BiquadFilter::peaking(750.0, -2.0, 1.1, sr),
            ),
            AmpModelId::UkAc30TopBoost => (
                BiquadFilter::peaking(3800.0, 2.5, 1.1, sr),
                BiquadFilter::low_shelf(140.0, -2.5, sr),
            ),
        }
    }

    pub fn new(model_id: AmpModelId) -> Self {
        let sr = 48000.0 * 8.0;
        let (v1, v2) = Self::create_voicing_filters(model_id, sr);
        let mut s = Self {
            model_id,
            panel: AmpFrontPanel::default(),
            mod_settings: nominal_mod_settings_for_model(model_id),
            sample_rate: 48000.0,
            bright_state: 0.0,
            bright2_state: 0.0,
            interstage_hp1: 0.0,
            interstage_hp2: 0.0,
            interstage_hp3: 0.0,
            miller_lp1: 0.0,
            ac30_cut_lp: 0.0,
            passive_tonestack: PassiveToneStack::new(),
            depth_filter: BiquadFilter::default(),
            presence_filter: BiquadFilter::default(),
            voicing_filter1: v1,
            voicing_filter2: v2,
            target_voicing_filter1: BiquadFilter::default(),
            target_voicing_filter2: BiquadFilter::default(),
            adaa_pre_prev: 0.0,
            adaa_s2_prev: 0.0,
            adaa_s3_prev: 0.0,
            adaa_s4_prev: 0.0,
            adaa_power_prev: 0.0,
            interp_hist: [0.0; 8],
            aa_filters: [BiquadFilter::default(); 14],
            ai_filters: [BiquadFilter::default(); 4],
            target_model_id: None,
            target_mod_settings: None,
            model_crossfade: 0.0,
            crossfade_step: 1.0 / (48000.0 * 0.020),
            oversampling_factor: 8,
            sag_state: 0.0,
            xformer_lf_state: 0.0,
            speaker_res_filter: BiquadFilter::default(),
            speaker_resonance_hz: 98.0,
            lfo_phase: 0.0,
            standby_env: 1.0,
            smooth_tube_gain: 1.0,
            spring_reverb: SpringReverbTank::default(),
        };
        s.update_aa_filters();
        s
    }

    pub fn set_power(&mut self, power: bool) {
        self.panel.power = power;
    }

    pub fn set_standby(&mut self, standby: bool) {
        self.panel.standby = standby;
    }

    pub fn set_input_jack(&mut self, jack: AmpInputJack) {
        self.panel.input_jack = jack;
        self.panel.jumpered = jack == AmpInputJack::Jumpered;
    }

    pub fn set_jumpered(&mut self, jumpered: bool) {
        self.panel.jumpered = jumpered;
        if jumpered {
            self.panel.input_jack = AmpInputJack::Jumpered;
        } else if self.panel.input_jack == AmpInputJack::Jumpered {
            self.panel.input_jack = AmpInputJack::High1;
        }
    }

    pub fn set_channel_mode(&mut self, mode: AmpChannelMode) {
        self.panel.channel_mode = mode;
    }

    pub fn set_spongy_power(&mut self, spongy: bool) {
        self.panel.spongy = spongy;
    }

    pub fn set_secondary_impedance_ohms(&mut self, ohms: u8) {
        self.panel.impedance_ohms = match ohms {
            4 | 8 | 16 => ohms,
            _ => 16,
        };
    }

    pub fn set_preamp_tube_type(&mut self, tube: PreampTube) {
        self.mod_settings.preamp_tube = tube;
    }

    pub fn set_power_tube_type(&mut self, tube: PowerTube) {
        self.mod_settings.power_tube = tube;
    }

    pub fn set_power_tube_bias_ma(&mut self, ma: f32) {
        self.mod_settings.bias_ma = ma.clamp(20.0, 55.0);
    }

    pub fn set_sag_amount(&mut self, sag: f32) {
        self.mod_settings.sag = sag.clamp(0.0, 10.0);
    }

    pub fn set_rectifier_type(&mut self, rec: RectifierType) {
        self.mod_settings.rectifier = rec;
    }

    /// Returns the instantaneous B+ rail voltage including power-supply sag and variac scaling.
    pub fn current_b_plus_v(&self) -> f32 {
        let factor = (1.0 - self.sag_state as f32).max(0.15);
        self.mod_settings.preamp_plate_v * factor
    }

    pub fn set_negative_feedback(&mut self, nfb: f32) {
        self.mod_settings.nfb = nfb.clamp(0.0, 10.0);
    }

    pub fn set_transformer_saturation(&mut self, sat: f32) {
        self.mod_settings.xformer_sat = sat.clamp(0.0, 10.0);
    }

    pub fn set_preamp_plate_voltage(&mut self, v: f32) {
        self.mod_settings.preamp_plate_v = v.clamp(180.0, 340.0);
    }

    pub fn set_triode_asymmetry(&mut self, asym: f32) {
        self.mod_settings.triode_asym = asym.clamp(0.0, 0.60);
    }

    pub fn set_tonestack_topology(&mut self, ts: ToneStackTopology) {
        self.mod_settings.tonestack = ts;
    }

    pub fn set_target_model(&mut self, model: AmpModelId) {
        if model != self.model_id {
            let sr = self.sample_rate as f64 * (self.oversampling_factor.max(2) as f64);
            let (v1, v2) = Self::create_voicing_filters(model, sr);
            self.target_voicing_filter1 = v1;
            self.target_voicing_filter2 = v2;
            self.target_model_id = Some(model);
            self.target_mod_settings = Some(nominal_mod_settings_for_model(model));
            self.model_crossfade = 0.0;
            self.crossfade_step = 1.0 / (self.sample_rate * 0.020).max(1.0);
        }
    }

    pub fn set_oversampling_factor(&mut self, factor: usize) {
        let new_factor = match factor {
            2 => 2,
            4 => 4,
            _ => 8,
        };
        if self.oversampling_factor != new_factor {
            self.oversampling_factor = new_factor;
            self.update_aa_filters();
        }
    }

    pub fn algorithmic_latency_samples(&self) -> u32 {
        match self.oversampling_factor {
            2 => 4,
            4 => 8,
            _ => 16,
        }
    }

    fn update_aa_filters(&mut self) {
        let os = self.oversampling_factor.max(2) as f64;
        let fs_base = self.sample_rate as f64;
        let fs_os = fs_base * os;

        // 14th-order Chebyshev Type I (7 cascaded biquads, 0.2 dB passband ripple)
        // Flat 20 Hz..18 kHz (ripple <= 0.25 dB) and >= 70 dB attenuation at fs_base / 2.
        let n_order = 14.0f64;
        let fp = 18300.0f64.min(fs_base * 0.385);
        let eps = (10.0f64.powf(0.20 / 10.0) - 1.0).sqrt();
        let gamma = ((1.0 / eps) + (1.0 / (eps * eps) + 1.0).sqrt()).ln() / n_order;
        let sinh_g = gamma.sinh();
        let cosh_g = gamma.cosh();
        let k_warp = (std::f64::consts::PI * fp / fs_os).tan();
        let k2 = k_warp * k_warp;

        for k in 0..7 {
            let theta = std::f64::consts::PI * (2.0 * (k as f64) + 1.0) / (2.0 * n_order);
            let sigma = -sinh_g * theta.sin();
            let omega = cosh_g * theta.cos();
            let r2 = sigma * sigma + omega * omega;
            let b_ana = -2.0 * sigma;
            let a0 = 1.0 + b_ana * k_warp + r2 * k2;
            let b0 = (r2 * k2) / a0;
            self.aa_filters[k] = BiquadFilter {
                b0,
                b1: 2.0 * b0,
                b2: b0,
                a1: (2.0 * (r2 * k2 - 1.0)) / a0,
                a2: (1.0 - b_ana * k_warp + r2 * k2) / a0,
                x1: 0.0,
                x2: 0.0,
                y1: 0.0,
                y2: 0.0,
            };
        }
        for k in 7..14 {
            self.aa_filters[k] = BiquadFilter {
                b0: 1.0,
                b1: 0.0,
                b2: 0.0,
                a1: 0.0,
                a2: 0.0,
                x1: 0.0,
                x2: 0.0,
                y1: 0.0,
                y2: 0.0,
            };
        }

        let fc_ai = 20000.0f64.min(fs_base * 0.42);
        for i in 0..4 {
            self.ai_filters[i] = BiquadFilter::lowpass(fc_ai, 0.707, fs_os);
        }
    }

    pub fn measure_aa_filter_magnitude_db(&self, freq_hz: f64) -> f64 {
        let fs_os = (self.sample_rate as f64) * (self.oversampling_factor.max(2) as f64);
        let w = 2.0 * std::f64::consts::PI * freq_hz / fs_os;
        let cos_w = w.cos();
        let sin_w = w.sin();
        let cos_2w = (2.0 * w).cos();
        let sin_2w = (2.0 * w).sin();
        let mut mag_sq = 1.0f64;
        for f in &self.aa_filters {
            let num_re = f.b0 + f.b1 * cos_w + f.b2 * cos_2w;
            let num_im = -f.b1 * sin_w - f.b2 * sin_2w;
            let den_re = 1.0 + f.a1 * cos_w + f.a2 * cos_2w;
            let den_im = -f.a1 * sin_w - f.a2 * sin_2w;
            let n2 = num_re * num_re + num_im * num_im;
            let d2 = (den_re * den_re + den_im * den_im).max(1e-24);
            mag_sq *= n2 / d2;
        }
        10.0 * mag_sq.max(1e-18).log10()
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.spring_reverb.set_sample_rate(self.sample_rate);
        self.update_aa_filters();
    }

    pub fn set_speaker_resonance_hz(&mut self, hz: f32) {
        self.speaker_resonance_hz = hz.clamp(50.0, 200.0);
    }

    pub fn speaker_resonance_hz(&self) -> f32 {
        self.speaker_resonance_hz
    }

    pub fn panel(&self) -> &AmpFrontPanel {
        &self.panel
    }

    pub fn panel_mut(&mut self) -> &mut AmpFrontPanel {
        &mut self.panel
    }

    pub fn mod_settings(&self) -> &ChassisModSettings {
        &self.mod_settings
    }

    pub fn mod_settings_mut(&mut self) -> &mut ChassisModSettings {
        &mut self.mod_settings
    }

    fn resolve_tonestack_topology(&self, model: AmpModelId) -> ToneStackTopology {
        if self.mod_settings.tonestack != ToneStackTopology::Model {
            return self.mod_settings.tonestack;
        }
        match model {
            AmpModelId::BritPlexi1959 | AmpModelId::BritJcm800_2203 => {
                ToneStackTopology::MarshallCathodeFollower
            }
            AmpModelId::UsTwinBlackface65 => ToneStackTopology::FenderTmbMidScoop,
            AmpModelId::UkAc30TopBoost => ToneStackTopology::VoxTopBoost,
            AmpModelId::CaliDualRectoRevF => ToneStackTopology::MesaMarkPreGain,
        }
    }

    #[inline]
    fn tonestack_recovery_gain(topo: ToneStackTopology) -> f64 {
        match topo {
            ToneStackTopology::MarshallCathodeFollower | ToneStackTopology::Model => 3.5,
            ToneStackTopology::FenderTmbMidScoop => 5.4,
            ToneStackTopology::VoxTopBoost => 4.2,
            ToneStackTopology::MesaMarkPreGain => 3.5,
            ToneStackTopology::PassiveJamesBaxandall => 2.6,
        }
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());

        let os_factor = self.oversampling_factor.max(2);
        let step_stride = (8 / os_factor).max(1);
        let fs_base = self.sample_rate as f64;
        let sr = fs_base * (os_factor as f64);

        let ts_topo = self.resolve_tonestack_topology(self.model_id);
        let ts_recovery = Self::tonestack_recovery_gain(ts_topo);
        self.passive_tonestack.update(
            ts_topo,
            self.panel.bass,
            self.panel.middle,
            self.panel.treble,
            sr,
        );

        // Effective NFB: Dual Rectifier Modern mode disconnects global NFB per Rev-F schematic
        let eff_nfb = if self.model_id == AmpModelId::CaliDualRectoRevF
            && self.panel.channel_mode == AmpChannelMode::Modern
        {
            0.0
        } else {
            self.mod_settings.nfb as f64
        };
        let nfb_scale = (eff_nfb / 10.0).clamp(0.0, 1.0);

        // Feedback loop filters: Depth (65-130 Hz) & Presence (>2 kHz) modulated by NFB loop (Spec 016 CA-7, Spec 039 CA-8)
        let depth_db = (self.panel.depth as f64 / 10.0) * 6.5 * nfb_scale;
        let pres_db = (self.panel.presence as f64 / 10.0) * 10.0 * nfb_scale;
        self.depth_filter.update_coeffs(BiquadFilter::peaking(85.0, depth_db, 2.5, sr));
        self.presence_filter.update_coeffs(BiquadFilter::peaking(4000.0, pres_db, 1.4, sr));

        // Subtle power-section/transformer voicing per model
        let (v1_cur, v2_cur) = if self.mod_settings.tonestack != ToneStackTopology::Model {
            match self.mod_settings.tonestack {
                ToneStackTopology::FenderTmbMidScoop => (
                    BiquadFilter::peaking(450.0, -3.5, 1.0, sr),
                    BiquadFilter::high_shelf(5500.0, 2.5, sr),
                ),
                ToneStackTopology::MarshallCathodeFollower => (
                    BiquadFilter::peaking(1600.0, 2.5, 1.2, sr),
                    BiquadFilter::low_shelf(120.0, -1.5, sr),
                ),
                ToneStackTopology::VoxTopBoost => (
                    BiquadFilter::peaking(2800.0, 2.5, 1.2, sr),
                    BiquadFilter::low_shelf(110.0, -3.0, sr),
                ),
                ToneStackTopology::PassiveJamesBaxandall => (
                    BiquadFilter::low_shelf(100.0, 1.5, sr),
                    BiquadFilter::high_shelf(4000.0, 1.5, sr),
                ),
                ToneStackTopology::MesaMarkPreGain => (
                    BiquadFilter::peaking(750.0, -2.5, 1.1, sr),
                    BiquadFilter::peaking(85.0, 3.0, 1.4, sr),
                ),
                ToneStackTopology::Model => Self::create_voicing_filters(self.model_id, sr),
            }
        } else {
            Self::create_voicing_filters(self.model_id, sr)
        };
        self.voicing_filter1.update_coeffs(v1_cur);
        self.voicing_filter2.update_coeffs(v2_cur);

        if let Some(target) = self.target_model_id {
            let (v1_tgt, v2_tgt) = Self::create_voicing_filters(target, sr);
            self.target_voicing_filter1.update_coeffs(v1_tgt);
            self.target_voicing_filter2.update_coeffs(v2_tgt);
        }

        let pre_gain_db = -44.0 + (self.panel.input_gain as f64 / 10.0) * 62.0;
        let pre_gain_lin = 10.0f64.powf(pre_gain_db / 20.0);

        let od_gain_db = (self.panel.overdrive as f64 / 10.0) * 36.0;
        let od_gain_lin = 10.0f64.powf(od_gain_db / 20.0);

        let vol2_gain_db = -44.0 + (self.panel.volume_2 as f64 / 10.0) * 62.0;
        let vol2_lin = if self.panel.volume_2 <= 0.05 {
            0.0
        } else {
            10.0f64.powf(vol2_gain_db / 20.0)
        };

        let lead_mult = match self.panel.channel_toggle {
            AmpChannelVoicing::Norm => 1.0,
            AmpChannelVoicing::HighLead => 2.5,
        };

        // Bright switch physical circuit + Plexi 5000pF bright cap on Vol I
        let bright_effective_boost = if self.panel.bright_switch || self.model_id == AmpModelId::BritPlexi1959 {
            let pot_ratio = (10.0 - self.panel.input_gain) / 10.0;
            let base = if self.panel.bright_switch { 0.95 } else { 0.35 };
            (pot_ratio as f64) * base
        } else {
            0.0
        };

        let input_pad = if self.panel.input_jack.is_low_pad() {
            0.50118723 // -6.0 dB
        } else {
            1.0
        };

        // Power tube & secondary impedance physical scaling
        let power_tube_headroom = match self.mod_settings.power_tube {
            PowerTube::El34 => 1.0,
            PowerTube::Tube6L6Gc => 1.18,
            PowerTube::Tube6V6 => 0.85,
            PowerTube::Kt88 => 1.35,
            PowerTube::El84 => 0.78,
        };
        let variac_scale = if self.model_id == AmpModelId::CaliDualRectoRevF && self.panel.spongy {
            0.80
        } else {
            1.0
        };
        let spongy_power_atten = if self.model_id == AmpModelId::CaliDualRectoRevF && self.panel.spongy {
            0.86
        } else {
            1.0
        };

        let calc_power_drive = |model: AmpModelId, gain: f32, vol2: f32, master: f32, ch_mode: AmpChannelMode| -> f64 {
            match model {
                AmpModelId::BritPlexi1959 => {
                    // Plexi 1959SLP has no Master Volume; power drive comes from Vol I & Vol II
                    (1.0 + (gain as f64 / 10.0) * 18.0 + (vol2 as f64 / 10.0) * 6.0) / power_tube_headroom
                }
                AmpModelId::UsTwinBlackface65 => {
                    (0.85 + (gain as f64 / 10.0) * 2.2) / power_tube_headroom
                }
                AmpModelId::CaliDualRectoRevF => {
                    let master_pot = master as f64 / 10.0;
                    let m_mult = match ch_mode {
                        AmpChannelMode::Clean => 0.55,
                        AmpChannelMode::Pushed => 0.95,
                        AmpChannelMode::Vintage => 1.35,
                        AmpChannelMode::Modern => 1.75,
                    };
                    (1.0 + master_pot * 4.2 * m_mult) / power_tube_headroom
                }
                AmpModelId::UkAc30TopBoost => {
                    let master_pot = master as f64 / 10.0;
                    (1.2 + master_pot * 5.0 + (gain as f64 / 10.0) * 1.6) / power_tube_headroom
                }
                AmpModelId::BritJcm800_2203 => {
                    let master_pot = master as f64 / 10.0;
                    (0.25 + master_pot.powi(3) * 28.0) / power_tube_headroom
                }
            }
        };

        let cur_drive = calc_power_drive(
            self.model_id,
            self.panel.input_gain,
            self.panel.volume_2,
            self.panel.master_volume,
            self.panel.channel_mode,
        );
        let target_drive = if let Some(target) = self.target_model_id {
            calc_power_drive(
                target,
                self.panel.input_gain,
                self.panel.volume_2,
                self.panel.master_volume,
                self.panel.channel_mode,
            )
        } else {
            cur_drive
        };
        let alpha_f64 = self.model_crossfade as f64;
        let power_saturation_drive = (1.0 - alpha_f64) * cur_drive + alpha_f64 * target_drive;

        let (target_tube_gain, ef86_knee) = match self.mod_settings.preamp_tube {
            PreampTube::Ax7A => (1.0f64, 1.0f64),
            PreampTube::At7 => (0.58f64, 0.85f64),
            PreampTube::Ay7 => (0.45f64, 0.78f64),
            PreampTube::Ef86 => (1.22f64, 1.35f64),
        };
        let tube_alpha = (1.0 / (fs_base * 0.008)).clamp(0.0005, 0.05);
        let v1_headroom = ((self.mod_settings.preamp_plate_v as f64 / 250.0) * variac_scale).max(0.1);
        let asym = self.mod_settings.triode_asym as f64;
        let bias_ma_f = self.mod_settings.bias_ma as f64;
        let bias_deficit = ((32.0 - bias_ma_f) / 12.0).clamp(0.0, 1.0);
        let crossover_depth = bias_deficit * 0.72;
        let crossover_width = 0.18;
        let bias_warm_drive = 1.0 + ((bias_ma_f - 40.0).max(0.0) / 24.0).min(0.25);

        // Sample-rate-invariant coefficients for base-rate filters (Spec 039 CA-4)
        let alpha_bright = (1.0 - (-2.0 * std::f64::consts::PI * 1200.0 / fs_base.max(1.0)).exp()).clamp(0.001, 0.99);
        let alpha_ch2 = (1.0 - (-2.0 * std::f64::consts::PI * 310.0 / fs_base.max(1.0)).exp()).clamp(0.001, 0.99);
        let alpha_standby = (1.0 - (-1.0 / (fs_base.max(1.0) * 0.00014)).exp()).clamp(0.001, 0.99);

        // Secondary output transformer impedance shifts damping factor and resonance height
        let imp_factor = match self.panel.impedance_ohms {
            4 => 0.78,
            8 => 1.0,
            _ => 1.24,
        };
        let res_boost_db = ((10.0 - eff_nfb) / 10.0) * 4.5 * imp_factor;
        let res_hz = self.speaker_resonance_hz.clamp(50.0, 200.0) as f64;
        self.speaker_res_filter
            .update_coeffs(BiquadFilter::peaking(res_hz, res_boost_db, 2.0, sr));

        // AC30 Tone Cut filter alpha (shunt LPF after LTP PI before EL84 power stage)
        let fc_cut_hz = 18000.0 * (750.0f64 / 18000.0).powf((self.panel.cut as f64 / 10.0).clamp(0.0, 1.0));
        let alpha_cut = (2.0 * std::f64::consts::PI * fc_cut_hz / sr).clamp(0.005, 0.99);

        // Interstage coupling alphas at OS sample rate
        let alpha_hp_loose = (2.0 * std::f64::consts::PI * 45.0 / sr).clamp(0.0001, 0.25);
        let alpha_hp_tight = (2.0 * std::f64::consts::PI * 115.0 / sr).clamp(0.0001, 0.35);
        let alpha_miller = (2.0 * std::f64::consts::PI * 7500.0 / sr).clamp(0.01, 0.90);

        let xf_val = self.mod_settings.xformer_sat as f64;
        let sag_amt = self.mod_settings.sag as f64;
        let ri = self.mod_settings.rectifier.internal_resistance();
        let tau_att = self.mod_settings.rectifier.attack_time_sec();
        let tau_rel = self.mod_settings.rectifier.release_time_sec();

        let out_lin = 10.0f64.powf(self.panel.output_level_db as f64 / 20.0);
        let live_target = if self.panel.power && !self.panel.standby { 1.0 } else { 0.0 };

        // Tremolo LFO frequency (1.5 Hz to 9.5 Hz)
        let lfo_hz = 1.5 + (self.panel.vibrato_speed as f64 / 10.0) * 8.0;
        let lfo_inc = (2.0 * std::f64::consts::PI * lfo_hz) / fs_base.max(1.0);
        let is_twin_vibrato_ch = self.model_id == AmpModelId::UsTwinBlackface65
            && !self.panel.input_jack.is_normal_channel();
        let is_ac30_vibrato = self.model_id == AmpModelId::UkAc30TopBoost
            && self.panel.vibrato_depth > 0.01;
        let trem_depth = if is_twin_vibrato_ch || is_ac30_vibrato {
            (self.panel.vibrato_depth as f64 / 10.0).clamp(0.0, 1.0) * 0.65
        } else {
            0.0
        };
        let rev_mix = if is_twin_vibrato_ch {
            (self.panel.reverb as f64 / 10.0).clamp(0.0, 1.0) * 1.45
        } else {
            0.0
        };

        for i in 0..input.len() {
            // Smooth PreampTube gain transition to eliminate switching clicks (Spec 021 CA-11)
            self.smooth_tube_gain += tube_alpha * (target_tube_gain - self.smooth_tube_gain);
            let tube_gain = self.smooth_tube_gain;

            // Smooth Power/Standby mute envelope (drops below 1e-6 inside ~2.5 ms)
            self.standby_env += alpha_standby * (live_target - self.standby_env);
            if self.standby_env < 1e-9 && live_target == 0.0 {
                self.standby_env = 0.0;
            }

            let raw_in = if input[i].is_finite() { input[i] as f64 } else { 0.0 };
            let mut s_in = raw_in * input_pad;

            // 1. Input Bright Cap Filter at base rate (Spec 003 CA-3, Spec 039 CA-4)
            self.bright_state += alpha_bright * (s_in - self.bright_state);
            if bright_effective_boost > 0.001 {
                let high_pass = s_in - self.bright_state;
                s_in += high_pass * (bright_effective_boost * 2.0);
            }

            // Keep parallel Normal channel low/mid contour state updated for Plexi Ch II
            // Note: s_in already includes input_pad (-6 dB on Low2), do not apply input_pad twice.
            self.bright2_state += alpha_ch2 * (s_in - self.bright2_state);
            let ch2_hp = s_in - self.bright2_state;
            let s_ch2_base = (s_in * 0.65 + self.bright2_state * 0.70 - ch2_hp * 0.10) * vol2_lin;

            for k in 0..7 {
                self.interp_hist[k] = self.interp_hist[k + 1];
            }
            self.interp_hist[7] = s_in;

            // Bias-vary Tremolo LFO modulation per base sample
            self.lfo_phase += lfo_inc;
            if self.lfo_phase >= 2.0 * std::f64::consts::PI {
                self.lfo_phase -= 2.0 * std::f64::consts::PI;
            }
            let trem_mod = if trem_depth > 1e-4 {
                1.0 - trem_depth * (0.5 + 0.5 * self.lfo_phase.sin())
            } else {
                1.0
            };

            let rev_tap_out = if rev_mix > 1e-4 {
                self.spring_reverb.process_sample(self.interstage_hp3)
            } else {
                0.0
            };

            let mut last_sample = 0.0f64;
            let mut rev_feed_acc = 0.0f64;
            let mut sub_count = 0usize;

            for step in (0..8).step_by(step_stride) {
                let weights = &SINC_WEIGHTS[step];
                let mut s = 0.0f64;
                for k in 0..8 {
                    s += self.interp_hist[k] * weights[k];
                }
                for f in &mut self.ai_filters {
                    s = f.process(s);
                }

                // 2. Authentic Per-Model Multi-Stage Preamp Topology
                let kp_base = (1.0 + 1.2 * asym) * ef86_knee;
                let kn_base = ((1.0 - 0.55 * asym) * ef86_knee).max(0.35);

                let v_pre_out = match self.model_id {
                    AmpModelId::BritPlexi1959 => {
                        let is_jumpered = self.panel.jumpered || self.panel.input_jack == AmpInputJack::Jumpered;
                        let ch1_on = is_jumpered || !self.panel.input_jack.is_normal_channel();
                        let ch2_on = is_jumpered || self.panel.input_jack.is_normal_channel();

                        let ch1_sig = if ch1_on {
                            let g1 = (s * pre_gain_lin * od_gain_lin * lead_mult * tube_gain) / v1_headroom;
                            triode_adaa1(g1, &mut self.adaa_pre_prev, kp_base, kn_base) * v1_headroom
                        } else {
                            0.0
                        };
                        let ch2_sig = if ch2_on {
                            let g2 = (s_ch2_base * od_gain_lin * tube_gain) / v1_headroom;
                            triode_adaa1(g2, &mut self.adaa_s2_prev, kp_base * 0.9, kn_base) * v1_headroom
                        } else {
                            0.0
                        };
                        // V2a + V2b Cathode Follower
                        let sum_v2 = one_pole_hpf(&mut self.interstage_hp1, ch1_sig + ch2_sig, alpha_hp_loose);
                        triode_adaa1((sum_v2 * 1.15) / v1_headroom, &mut self.adaa_s3_prev, kp_base, kn_base)
                            * v1_headroom
                    }
                    AmpModelId::BritJcm800_2203 => {
                        let is_low = self.panel.input_jack.is_low_pad();
                        let base_drive = pre_gain_lin * od_gain_lin * lead_mult * tube_gain;
                        // V1a stage (Low jack skips V1a excess gain while matching -6dB input pad)
                        let s_after_v1a = if is_low {
                            s * base_drive
                        } else {
                            triode_adaa1(
                                (s * 1.05 * base_drive) / v1_headroom,
                                &mut self.adaa_pre_prev,
                                kp_base,
                                kn_base,
                            ) * v1_headroom
                        };
                        // V1b Asymmetric Cold Clipper (Rk = 10k -> pronounced 2f0 when triode_asym > 0; symmetric if dialed to 0 in Mod Shop)
                        let v1b_in = one_pole_hpf(&mut self.interstage_hp1, s_after_v1a, alpha_hp_tight);
                        let kp_cold = (1.0 + 11.0 * asym) * ef86_knee;
                        let kn_cold = ((1.0 - 2.6 * asym) * ef86_knee).max(0.35);
                        let v1b = triode_adaa1((v1b_in * 1.08) / v1_headroom, &mut self.adaa_s2_prev, kp_cold, kn_cold)
                            * v1_headroom;
                        // V2a + V2b Cathode Follower
                        let v2_in = one_pole_hpf(&mut self.interstage_hp2, v1b, alpha_hp_loose);
                        triode_adaa1((v2_in * 1.0) / v1_headroom, &mut self.adaa_s3_prev, kp_base, kn_base)
                            * v1_headroom
                    }
                    AmpModelId::UsTwinBlackface65 => {
                        // High-headroom AB763 clean triode stages (Volume -> Bright -> Tone Stack)
                        let twin_headroom = v1_headroom * 1.55;
                        let g_twin = (s * pre_gain_lin * 0.72 * od_gain_lin * lead_mult * tube_gain) / twin_headroom;
                        let v1 = triode_adaa1(g_twin, &mut self.adaa_pre_prev, kp_base * 0.85, kn_base) * twin_headroom;
                        one_pole_hpf(&mut self.interstage_hp1, v1, alpha_hp_loose * 0.6)
                    }
                    AmpModelId::CaliDualRectoRevF => {
                        let (stage_drive, n_stages, use_tight_hp) = match self.panel.channel_mode {
                            AmpChannelMode::Clean => (0.65, 2usize, false),
                            AmpChannelMode::Pushed => (1.25, 3usize, false),
                            AmpChannelMode::Vintage => (2.15, 4usize, false),
                            AmpChannelMode::Modern => (3.10, 4usize, true),
                        };
                        let hp_a = if use_tight_hp { alpha_hp_tight } else { alpha_hp_loose };
                        let g1 = (s * pre_gain_lin * od_gain_lin * lead_mult * stage_drive * tube_gain) / v1_headroom;
                        let s1 = triode_adaa1(g1, &mut self.adaa_pre_prev, kp_base, kn_base) * v1_headroom;
                        let hp1 = one_pole_hpf(&mut self.interstage_hp1, s1, hp_a);
                        let s2 = triode_adaa1((hp1 * (0.9 + 0.35 * stage_drive)) / v1_headroom, &mut self.adaa_s2_prev, kp_base * 1.25, kn_base * 0.85) * v1_headroom;
                        if n_stages == 2 {
                            s2
                        } else {
                            let hp2 = one_pole_hpf(&mut self.interstage_hp2, s2, hp_a);
                            let s3 = triode_adaa1((hp2 * 1.85) / v1_headroom, &mut self.adaa_s3_prev, kp_base * 1.45, kn_base * 0.7) * v1_headroom;
                            if n_stages == 3 {
                                s3
                            } else {
                                self.miller_lp1 += alpha_miller * (s3 - self.miller_lp1);
                                let s4 = triode_adaa1((self.miller_lp1 * 2.1) / v1_headroom, &mut self.adaa_s4_prev, kp_base * 1.15, kn_base * 0.9) * v1_headroom;
                                s4
                            }
                        }
                    }
                    AmpModelId::UkAc30TopBoost => {
                        let is_norm = self.panel.input_jack.is_normal_channel();
                        let g1 = (s * pre_gain_lin * od_gain_lin * lead_mult * tube_gain * 1.05) / v1_headroom;
                        let v1 = triode_adaa1(g1, &mut self.adaa_pre_prev, kp_base * 1.05, kn_base * 0.92) * v1_headroom;
                        if is_norm {
                            v1
                        } else {
                            let hp1 = one_pole_hpf(&mut self.interstage_hp1, v1, alpha_hp_loose);
                            triode_adaa1((hp1 * 1.35) / v1_headroom, &mut self.adaa_s2_prev, kp_base * 1.1, kn_base * 0.88)
                                * v1_headroom
                        }
                    }
                };

                // 3. Passive Interactive Tone Stack (or Normal bypass on AC30 Normal channel)
                let skip_top_boost_stack = self.model_id == AmpModelId::UkAc30TopBoost
                    && self.panel.input_jack.is_normal_channel()
                    && self.mod_settings.tonestack == ToneStackTopology::Model;
                let v_base = if skip_top_boost_stack {
                    v_pre_out
                } else {
                    self.passive_tonestack.process(v_pre_out) * ts_recovery
                };

                rev_feed_acc += v_base;
                sub_count += 1;

                // Model Voicing & Glitch-Free Crossfade
                let v_voiced = if self.target_model_id.is_some() {
                    let mut va = self.voicing_filter1.process(v_base);
                    va = self.voicing_filter2.process(va);

                    let mut vb = self.target_voicing_filter1.process(v_base);
                    vb = self.target_voicing_filter2.process(vb);

                    let alpha = self.model_crossfade as f64;
                    (1.0 - alpha) * va + alpha * vb
                } else {
                    let mut va = self.voicing_filter1.process(v_base);
                    va = self.voicing_filter2.process(va);
                    va
                };

                // Twin Reverb vibrato channel mixes Spring Reverb before PI
                let v_with_rev = v_voiced + rev_tap_out * rev_mix;

                // AC30 Tone Cut shunt low-pass after LTP PI (and mild active on others if desired, primary on AC30)
                let v_post_cut = if self.model_id == AmpModelId::UkAc30TopBoost {
                    self.ac30_cut_lp += alpha_cut * (v_with_rev - self.ac30_cut_lp);
                    self.ac30_cut_lp
                } else {
                    v_with_rev
                };

                // 4. Power Amp / Phase Inverter Stage with Bias Crossover, Tremolo & ADAA tanh
                let power_in_raw = v_post_cut * power_saturation_drive * bias_warm_drive * trem_mod;
                let power_in = if crossover_depth > 1e-4 {
                    let dip = (-0.5 * (power_in_raw / crossover_width).powi(2)).exp();
                    power_in_raw * (1.0 - crossover_depth * dip)
                } else {
                    power_in_raw
                };
                let dp = power_in - self.adaa_power_prev;
                let power_sat_raw = if dp.abs() < 1e-5 {
                    power_in.tanh()
                } else {
                    let f1 = if power_in.abs() > 20.0 {
                        power_in.abs() - std::f64::consts::LN_2
                    } else {
                        power_in.cosh().ln()
                    };
                    let f0 = if self.adaa_power_prev.abs() > 20.0 {
                        self.adaa_power_prev.abs() - std::f64::consts::LN_2
                    } else {
                        self.adaa_power_prev.cosh().ln()
                    };
                    (f1 - f0) / dp
                };
                self.adaa_power_prev = power_in;

                // NFB Linearization on power stage distortion (Modern Dual Rect / AC30 use eff_nfb=0)
                let nfb_linearization = 1.0 / (1.0 + (eff_nfb / 10.0) * 3.0);
                let power_stage_dist = power_sat_raw - power_in_raw;
                let power_sat = power_in_raw + power_stage_dist * nfb_linearization;

                // 5. Speaker Resonance (NFB + Secondary Impedance) + Depth & Presence
                let mut power_out = self.speaker_res_filter.process(power_sat * 0.30 * spongy_power_atten);
                power_out = self.depth_filter.process(power_out);
                power_out = self.presence_filter.process(power_out);

                // 6. Transformer Saturation (Feedforward Frequency-Dependent LF Core Saturation at 150 Hz)
                if xf_val > 0.01 {
                    let lf_alpha = (150.0 * 2.0 * std::f64::consts::PI / sr).min(0.2);
                    self.xformer_lf_state += lf_alpha * (power_sat - self.xformer_lf_state);
                    let lf_flux = self.xformer_lf_state;
                    let lf_k = 6.0;
                    let sat_flux = (lf_flux * lf_k).tanh() / lf_k;
                    let lf_loss = (sat_flux - lf_flux) * (xf_val / 10.0) * 1.5;
                    power_out += lf_loss;
                }

                // Dynamic Sag calculation
                let target_drop = if sag_amt > 0.001 {
                    (sag_amt / 10.0) * ri * (power_out.abs() * 2.5).min(1.0)
                } else {
                    0.0
                };
                let alpha_sag = if target_drop > self.sag_state {
                    1.0 - (-1.0 / (sr * tau_att)).exp()
                } else {
                    1.0 - (-1.0 / (sr * tau_rel)).exp()
                };
                self.sag_state += alpha_sag * (target_drop - self.sag_state);
                let b_plus_factor = ((1.0 - self.sag_state) * variac_scale).max(0.15);

                // Safe output limiter inside oversampled loop
                let lim_in = power_out * b_plus_factor * self.standby_env;
                let abs_s = lim_in.abs();
                let mut lim_out = if abs_s <= 0.65 {
                    lim_in
                } else {
                    lim_in.signum() * (0.65 + 0.25 * ((abs_s - 0.65) / 0.25).tanh())
                };

                // Anti-Aliasing Decimation Filter
                for f in &mut self.aa_filters {
                    lim_out = f.process(lim_out);
                }

                last_sample = lim_out;
            }

            if sub_count > 0 {
                self.interstage_hp3 = rev_feed_acc / (sub_count as f64);
            }

            if self.target_model_id.is_some() {
                self.model_crossfade += self.crossfade_step;
                if self.model_crossfade >= 1.0 {
                    let target = self.target_model_id.take().unwrap();
                    self.model_id = target;
                    if let Some(ms) = self.target_mod_settings.take() {
                        self.mod_settings = ms;
                    }
                    self.voicing_filter1 = self.target_voicing_filter1;
                    self.voicing_filter2 = self.target_voicing_filter2;
                    self.model_crossfade = 0.0;
                }
            }

            let out_sample = (last_sample.clamp(-0.999, 0.999) * out_lin * self.standby_env).clamp(-1.0, 1.0);
            output[i] = if out_sample.is_finite() {
                out_sample as f32
            } else {
                0.0
            };
        }
    }

    /// Computes Total Harmonic Distortion (THD) ratio of a periodic signal
    pub fn compute_thd(&self, signal: &[f32]) -> f32 {
        if signal.is_empty() {
            return 0.0;
        }
        let mean: f32 = signal.iter().sum::<f32>() / signal.len() as f32;
        let ac: Vec<f32> = signal.iter().map(|&x| x - mean).collect();

        // Perform basic DFT at fundamental frequency (assuming ~1000 Hz with N=2048, sr=48000)
        let n = ac.len();
        let fs = self.sample_rate;
        let f0 = 1000.0;
        let bin_k = (f0 * n as f32 / fs).round() as usize;

        let calc_power_at_bin = |k: usize| -> f32 {
            let mut real = 0.0f32;
            let mut imag = 0.0f32;
            let angle = 2.0 * std::f32::consts::PI * (k as f32) / (n as f32);
            for (idx, &sample) in ac.iter().enumerate() {
                real += sample * (angle * idx as f32).cos();
                imag += sample * (angle * idx as f32).sin();
            }
            real * real + imag * imag
        };

        let p_fund = calc_power_at_bin(bin_k).max(1e-12);
        let mut p_harmonics = 0.0f32;
        for h in 2..=5 {
            let hk = bin_k * h;
            if hk < n / 2 {
                p_harmonics += calc_power_at_bin(hk);
            }
        }

        (p_harmonics / p_fund).sqrt()
    }
}
