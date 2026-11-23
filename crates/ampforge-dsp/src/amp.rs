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
    pub const ALL: [Self; 5] = [
        Self::BritJcm800_2203,
        Self::BritPlexi1959,
        Self::UsTwinBlackface65,
        Self::CaliDualRectoRevF,
        Self::UkAc30TopBoost,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BritJcm800_2203 => "jcm800_2203",
            Self::BritPlexi1959 => "plexi_1959",
            Self::UsTwinBlackface65 => "twin_ab763",
            Self::CaliDualRectoRevF => "dual_rect",
            Self::UkAc30TopBoost => "ac30_tb",
        }
    }

    pub fn has_presence(self) -> bool {
        matches!(
            self,
            Self::BritPlexi1959 | Self::BritJcm800_2203 | Self::CaliDualRectoRevF
        )
    }

    pub fn has_master(self) -> bool {
        matches!(self, Self::BritJcm800_2203 | Self::CaliDualRectoRevF)
    }

    pub fn has_middle(self) -> bool {
        !matches!(self, Self::UkAc30TopBoost)
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
pub enum Ac30VibTremMode {
    Vibrato,
    Tremolo,
}

impl Ac30VibTremMode {
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "vibrato" => Some(Self::Vibrato),
            "tremolo" => Some(Self::Tremolo),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Vibrato => "vibrato",
            Self::Tremolo => "tremolo",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmpChannelMode {
    Clean,
    Vintage,
    Modern,
}

impl AmpChannelMode {
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "clean" => Some(Self::Clean),
            "vintage" => Some(Self::Vintage),
            "modern" => Some(Self::Modern),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clean => "clean",
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
    pub bass: f32,
    pub middle: f32,
    pub treble: f32,
    pub presence: f32,
    pub master_volume: f32,
    pub output_level_db: f32,
    pub bright_switch: bool,
    pub channel_toggle: AmpChannelVoicing,
    pub volume_2: f32,
    pub cut: f32,
    pub reverb: f32,
    pub vibrato_speed: f32,
    pub vibrato_depth: f32,
    pub ac30_vib_trem_mode: Ac30VibTremMode,
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
            bass: 5.0,
            middle: 5.0,
            treble: 5.0,
            presence: 5.0,
            master_volume: 5.0,
            output_level_db: 0.0,
            bright_switch: false,
            channel_toggle: AmpChannelVoicing::Norm,
            volume_2: 0.0,
            cut: 5.0,
            reverb: 0.0,
            vibrato_speed: 4.0,
            vibrato_depth: 0.0,
            ac30_vib_trem_mode: Ac30VibTremMode::Vibrato,
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

    pub fn set_ac30_vib_trem_mode(&mut self, mode: Ac30VibTremMode) {
        self.ac30_vib_trem_mode = mode;
    }

    pub fn ac30_vib_trem_mode(&self) -> Ac30VibTremMode {
        self.ac30_vib_trem_mode
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

struct AmpCore {
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
    miller_lp2: f64,
    ac30_cut_lp: f64,
    jcm800_peaking: Jcm800PeakingNetwork,
    plexi_bright_cap: PlexiBrightCapFilter,
    passive_tonestack: PassiveToneStack,
    presence_filter: BiquadFilter,
    voicing_filter1: BiquadFilter,
    voicing_filter2: BiquadFilter,
    adaa_pre_prev: f64,
    adaa_s2_prev: f64,
    adaa_s3_prev: f64,
    adaa_s4_prev: f64,
    adaa_power_prev: f64,
    adaa_power_f_prev: f64,
    interp_hist: [f64; 8],
    aa_filters: [BiquadFilter; 4],
    ai_filters: [BiquadFilter; 4],
    oversampling_factor: usize,
    sag_state: f64,
    xformer_lf_state: f64,
    speaker_res_filter: BiquadFilter,
    speaker_resonance_hz: f32,
    lfo_phase: f64,
    standby_env: f64,
    smooth_tube_gain: f64,
    spring_reverb: SpringReverbTank,
    ac30_vib_buf: [f64; 512],
    ac30_vib_pos: usize,
    sm_bass: crate::smoother::ParamSmoother,
    sm_middle: crate::smoother::ParamSmoother,
    sm_treble: crate::smoother::ParamSmoother,
    sm_presence: crate::smoother::ParamSmoother,
    sm_master: crate::smoother::ParamSmoother,
    sm_volume_2: crate::smoother::ParamSmoother,
    sm_cut: crate::smoother::ParamSmoother,
    sm_speaker_res_hz: crate::smoother::ParamSmoother,
    has_processed: bool,
    cached_subblock: Option<AmpSubblockCache>,
}

#[derive(Clone, Copy)]
struct AmpSubblockCache {
    model_id: AmpModelId,
    os_factor: usize,
    fs_base: f64,
    sm_bass: f32,
    sm_middle: f32,
    sm_treble: f32,
    sm_presence: f32,
    sm_master: f32,
    sm_volume_2: f32,
    sm_cut: f32,
    sm_res_hz: f64,
    input_gain: f32,
    output_level_db: f32,
    channel_mode: AmpChannelMode,
    channel_toggle: AmpChannelVoicing,
    bright_switch: bool,
    input_jack: AmpInputJack,
    jumpered: bool,
    spongy: bool,
    impedance_ohms: u8,
    power: bool,
    standby: bool,
    vibrato_speed: f32,
    vibrato_depth: f32,
    ac30_vib_trem_mode: Ac30VibTremMode,
    reverb: f32,
    mod_nfb: f32,
    mod_tonestack: ToneStackTopology,
    mod_xformer_sat: f32,
    mod_sag: f32,
    mod_rectifier: RectifierType,
    mod_power_tube: PowerTube,
    mod_preamp_tube: PreampTube,
    mod_preamp_plate_v: f32,
    mod_triode_asym: f32,
    mod_bias_ma: f32,
    // Computed values
    ts_recovery: f64,
    eff_nfb: f64,
    pre_gain_lin: f64,
    vol2_lin: f64,
    lead_mult: f64,
    bright_effective_boost: f64,
    input_pad: f64,
    power_tube_headroom: f64,
    variac_scale: f64,
    spongy_power_atten: f64,
    plexi_ch1_on: bool,
    plexi_ch2_on: bool,
    power_saturation_drive: f64,
    target_tube_gain: f64,
    ef86_knee: f64,
    tube_alpha: f64,
    v1_headroom: f64,
    asym: f64,
    bias_ma_f: f64,
    crossover_depth: f64,
    bias_warm_drive: f64,
    alpha_bright: f64,
    alpha_ch2: f64,
    alpha_standby: f64,
    alpha_cut: f64,
    alpha_hp_loose: f64,
    alpha_hp_tight: f64,
    alpha_miller: f64,
    xf_val: f64,
    sag_amt: f64,
    ri: f64,
    alpha_sag_att: f64,
    alpha_sag_rel: f64,
    alpha_ac30_brilliant_hp: f64,
    out_lin: f64,
    live_target: f64,
    lfo_inc: f64,
    is_ac30_vib: bool,
    trem_depth: f64,
    rev_mix: f64,
}

pub struct WhiteBoxAmpProcessor {
    pub model_id: AmpModelId,
    amp: Box<AmpCore>,
    amp_transicao: Box<AmpCore>,
    transicao_ativa: bool,
    transicao_amostras_feitas: usize,
    transicao_total_amostras: usize,
    transicao_buf: [f32; 64],
    measured_latencies: [u32; 4],
}

/// Tanque do Fender Twin Reverb AB763: Accutronics 4AB3C1B, código de decay 'C' (longo, 2,75–4,0 s).
pub const DECISAO_SONIX_TWIN_RT60: f64 = 3.0;

#[derive(Debug, Clone)]
struct SpringReverbTank {
    lines: [[f64; 2048]; 4],
    lens: [usize; 4],
    idx: [usize; 4],
    lp: [f64; 4],
    lp_alpha: f64,
    feedback_g: f64,
}

impl Default for SpringReverbTank {
    fn default() -> Self {
        let lens = [347, 509, 683, 887];
        let avg_len_s = (lens.iter().sum::<usize>() as f64 / 4.0) / 48000.0;
        let feedback_g = 10.0f64.powf(-3.0 * avg_len_s / DECISAO_SONIX_TWIN_RT60) * 1.003;
        Self {
            lines: [[0.0; 2048]; 4],
            lens,
            idx: [0; 4],
            lp: [0.0; 4],
            lp_alpha: 1.0 - (-2.0 * std::f64::consts::PI * 6500.0 / 48000.0).exp(),
            feedback_g,
        }
    }
}

impl SpringReverbTank {
    fn reset(&mut self) {
        for line in &mut self.lines {
            line.fill(0.0);
        }
        self.idx = [0; 4];
        self.lp = [0.0; 4];
    }

    fn set_sample_rate(&mut self, sr: f32) {
        let fs = (sr as f64).max(1.0);
        let scale = (fs / 48000.0).clamp(0.5, 2.0);
        self.lp_alpha = (1.0 - (-2.0 * std::f64::consts::PI * 6500.0 / fs).exp()).clamp(0.01, 0.99);
        let base = [347.0, 509.0, 683.0, 887.0];
        let mut sum_lens = 0usize;
        for i in 0..4 {
            self.lens[i] = ((base[i] * scale).round() as usize).clamp(64, 2047);
            sum_lens += self.lens[i];
            if self.idx[i] >= self.lens[i] {
                self.idx[i] = 0;
            }
        }
        let avg_len_s = (sum_lens as f64 / 4.0) / fs;
        self.feedback_g = (10.0f64.powf(-3.0 * avg_len_s / DECISAO_SONIX_TWIN_RT60) * 1.003).clamp(0.5, 0.995);
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
        // Hadamard scattering matrix calibrated to Accutronics 4AB3C1B RT60 = 3.0 s
        let s_all = 0.5 * (d[0] + d[1] + d[2] + d[3]);
        let g = self.feedback_g;
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
    *state = crate::denormal::flush_f64(*state + alpha * hp);
    hp
}

pub use crate::biquad::BiquadFilter;

const TRIODE_12AX7_MU: f64 = 100.0;
const TRIODE_12AX7_RP_OHM: f64 = 62_500.0;
// Fonte: Dr. Tube Marshall JMP/JCM800 2203 schematic (1981) & Rob Robinette
// (https://robrobinette.com/How_the_Marshall_JCM800_Works.htm), Spec 055 §6.2/§7:
// V1a: Rp = 100 kΩ, Rl = 1 MΩ (pot Pre-Amp Volume), Rk = 2,7 kΩ || 0,68 µF (bypassado em 1 kHz -> Rk_sem_bypass = 0).
const JCM800_V1A_RA_OHM: f64 = 100_000.0;
const JCM800_V1A_RL_OHM: f64 = 1_000_000.0;
const JCM800_V1A_RK_UNBYPASSED_OHM: f64 = 0.0;
// DECISAO_SONIX_PREAMP_STAGE_NORM (2026-11-20): normalização grade-placa (~10x entre grade V1 ~1,5 V e estágio seguinte/PI ~15 V)
// usada enquanto triode_adaa1 opera normalizado em v1_headroom ~ 1,0; confirmável por calibração nodal SPICE/WDF completa.
const DECISAO_SONIX_PREAMP_STAGE_NORM: f64 = 10.0;

// DECISAO_SONIX_RECTO_STAGE_* (2026-11-20): esquema primário Rev F (`RF-1F`) é UNKNOWN em docs/amps/voicing-fontes.md §4;
// valores típicos de cascata 12AX7 de alto ganho (s2 e s4 com cátodo 1.8 kΩ || 1 µF bypassado acima de ~88 Hz -> Rk_sem_bypass = 0;
// cold clipper s3 com Rk = 39 kΩ sem capacitor de bypass), confirmáveis com esquema primário RF-1F.
const DECISAO_SONIX_RECTO_S2_RA: f64 = 100_000.0;
const DECISAO_SONIX_RECTO_S2_RL: f64 = 470_000.0;
const DECISAO_SONIX_RECTO_S2_RK_UNBYPASSED: f64 = 0.0;
const DECISAO_SONIX_RECTO_S3_RA: f64 = 100_000.0;
const DECISAO_SONIX_RECTO_S3_RL: f64 = 470_000.0;
const DECISAO_SONIX_RECTO_S3_RK_UNBYPASSED: f64 = 39_000.0;
const DECISAO_SONIX_RECTO_S4_RA: f64 = 100_000.0;
const DECISAO_SONIX_RECTO_S4_RL: f64 = 1_000_000.0;
const DECISAO_SONIX_RECTO_S4_RK_UNBYPASSED: f64 = 0.0;

#[inline]
fn triode_stage_gain(ra_ohm: f64, rl_ohm: f64, rk_unbypassed_ohm: f64) -> f64 {
    let r_ac = (ra_ohm * rl_ohm) / (ra_ohm + rl_ohm);
    (TRIODE_12AX7_MU * r_ac)
        / (r_ac + TRIODE_12AX7_RP_OHM + (TRIODE_12AX7_MU + 1.0) * rk_unbypassed_ohm)
}

// DECISAO_SONIX_RECTO_MODERN_PRESENCE (2026-11-21): evidência indireta da Mesa (cada modo tem seu pot de presence);
// esquema primário RF-1F PENDENTE (em Modern, NFB global = 0, mantendo o controle de presence atuante em 4 kHz).
const DECISAO_SONIX_RECTO_MODERN_PRESENCE: f64 = 1.0;

// DECISAO_SONIX_AC30_BRILLIANT_HP_HZ (2026-11-22): capacitor de acoplamento 0,001 µF (1000 pF) em 1 MΩ (fc = 159,15 Hz)
// no canal Brilliant do AC30/6; esquemas OS/056/OS/065 apresentam variantes de 500 pF a 1000 pF.
const DECISAO_SONIX_AC30_BRILLIANT_HP_HZ: f64 = 159.15;

// Fonte: Dr. Tube Marshall JMP/JCM800 2203 schematic (1981) & Rob Robinette
// (https://robrobinette.com/How_the_Marshall_JCM800_Works.htm), Spec 055 §6.2/§7:
// Divisor inter-estágio V1b -> V2a: R1 = 470 kΩ em paralelo com Cp = 470 pF, sobre Rl = 470 kΩ à terra.
const JCM800_PEAKING_R1_OHM: f64 = 470_000.0;
const JCM800_PEAKING_CP_F: f64 = 470e-12;
const JCM800_PEAKING_RL_OHM: f64 = 470_000.0;

#[derive(Debug, Clone, Copy)]
pub struct Jcm800PeakingNetwork {
    b0: f64,
    b1: f64,
    a1: f64,
    x1: f64,
    y1: f64,
    last_sr: f64,
}

impl Jcm800PeakingNetwork {
    pub fn new(sr: f64) -> Self {
        let mut s = Self {
            b0: 1.0,
            b1: 0.0,
            a1: 0.0,
            x1: 0.0,
            y1: 0.0,
            last_sr: -1.0,
        };
        s.update(sr);
        s
    }

    pub fn update(&mut self, sr: f64) {
        let fs = sr.max(8000.0);
        if (fs - self.last_sr).abs() < 0.5 {
            return;
        }
        self.last_sr = fs;
        let r1 = JCM800_PEAKING_R1_OHM;
        let rl = JCM800_PEAKING_RL_OHM;
        let cp = JCM800_PEAKING_CP_F;
        let rc = rl * r1 * cp;
        let w_z = 1.0 / (r1 * cp);
        let w_p = (rl + r1) / rc;
        let w0 = (w_z * w_p).sqrt().min(0.45 * std::f64::consts::PI * fs);
        let k = w0 / (w0 / (2.0 * fs)).tan();
        let b0_s = rl;
        let b1_s = rc;
        let a0_s = rl + r1;
        let a1_s = rc;
        let den = a0_s + a1_s * k;
        self.b0 = (b0_s + b1_s * k) / den;
        self.b1 = (b0_s - b1_s * k) / den;
        self.a1 = (a0_s - a1_s * k) / den;
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }

    #[inline(always)]
    pub fn process(&mut self, x: f64) -> f64 {
        let y = crate::denormal::flush_f64(self.b0 * x + self.b1 * self.x1 - self.a1 * self.y1);
        self.x1 = crate::denormal::flush_f64(x);
        self.y1 = y;
        y
    }
}

// Fonte: Marshall 1959HW Owners Manual (docs/ui-desktop/fontes/marshall-1959HW-owners-manual.pdf,
// p. 5, Item 8 — Tonal Note 5, PRIMARY_OFFICIAL) & Dr. Tube 1969 SL/A:
// Bright cap Cb = 0,005 µF (5000 pF) em paralelo ao segmento superior de Volume I (1 MΩ, taper 10% log).
const PLEXI_VOLUME1_POT_OHM: f64 = 1_000_000.0;
const PLEXI_BRIGHT_CAP_F: f64 = 5_000e-12;

#[derive(Debug, Clone, Copy)]
pub struct PlexiBrightCapFilter {
    b0: f64,
    b1: f64,
    a1: f64,
    x1: f64,
    y1: f64,
    last_vol: f64,
    last_sr: f64,
}

impl PlexiBrightCapFilter {
    pub fn new(sr: f64) -> Self {
        let mut s = Self {
            b0: 1.0,
            b1: 0.0,
            a1: 0.0,
            x1: 0.0,
            y1: 0.0,
            last_vol: -1.0,
            last_sr: -1.0,
        };
        s.update(5.0, sr);
        s
    }

    pub fn update(&mut self, vol_0_to_10: f64, sr: f64) {
        let fs = sr.max(8000.0);
        let u_vol = vol_0_to_10.clamp(0.0, 10.0);
        if (u_vol - self.last_vol).abs() < 1e-6 && (fs - self.last_sr).abs() < 0.5 {
            return;
        }
        self.last_vol = u_vol;
        self.last_sr = fs;

        let u = u_vol / 10.0;
        let t = (0.01 * 100.0f64.powf(u) - 0.01) / 0.99;
        if t <= 1e-9 {
            self.b0 = 0.0;
            self.b1 = 0.0;
            self.a1 = 0.0;
            return;
        }
        if t >= 1.0 - 1e-9 {
            self.b0 = 1.0;
            self.b1 = 0.0;
            self.a1 = 0.0;
            return;
        }
        let r_u = (1.0 - t) * PLEXI_VOLUME1_POT_OHM;
        let r_d = t * PLEXI_VOLUME1_POT_OHM;
        let rc = r_d * r_u * PLEXI_BRIGHT_CAP_F;
        let w_z = 1.0 / (r_u * PLEXI_BRIGHT_CAP_F);
        let w_p = PLEXI_VOLUME1_POT_OHM / rc;
        let w0 = (w_z * w_p).sqrt().min(0.45 * std::f64::consts::PI * fs);
        let k = w0 / (w0 / (2.0 * fs)).tan();
        let b0_s = r_d;
        let b1_s = rc;
        let a0_s = PLEXI_VOLUME1_POT_OHM;
        let a1_s = rc;
        let den = a0_s + a1_s * k;
        self.b0 = (b0_s + b1_s * k) / den;
        self.b1 = (b0_s - b1_s * k) / den;
        self.a1 = (a0_s - a1_s * k) / den;
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }

    #[inline(always)]
    pub fn process(&mut self, x: f64) -> f64 {
        let y = crate::denormal::flush_f64(self.b0 * x + self.b1 * self.x1 - self.a1 * self.y1);
        self.x1 = crate::denormal::flush_f64(x);
        self.y1 = y;
        y
    }
}

impl AmpCore {
    pub fn create_voicing_filters(model: AmpModelId, sr: f64) -> (BiquadFilter, BiquadFilter) {
        // Curvas físicas documentadas em docs/amps/voicing-fontes.md e esquemas elétricos originais
        match model {
            AmpModelId::BritJcm800_2203 => (
                BiquadFilter::identity(),
                BiquadFilter::identity(),
            ),
            AmpModelId::BritPlexi1959 => (
                BiquadFilter::peaking(2500.0, 2.6, 0.90, sr),
                BiquadFilter::low_shelf(160.0, 2.8, sr),
            ),
            AmpModelId::UsTwinBlackface65 => (
                BiquadFilter::high_shelf(4200.0, 3.2, sr),
                BiquadFilter::identity(),
            ),
            AmpModelId::CaliDualRectoRevF => (
                BiquadFilter::peaking(100.0, 4.5, 1.0, sr),
                BiquadFilter::peaking(550.0, -6.0, 0.8, sr),
            ),
            AmpModelId::UkAc30TopBoost => (
                BiquadFilter::peaking(950.0, 3.5, 1.0, sr),
                BiquadFilter::low_shelf(150.0, -3.5, sr),
            ),
        }
    }

    pub fn new(model_id: AmpModelId) -> Self {
        let default_os = 2usize;
        let sr = 48000.0 * (default_os as f32);
        let (v1, v2) = Self::create_voicing_filters(model_id, sr as f64);
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
            miller_lp2: 0.0,
            ac30_cut_lp: 0.0,
            jcm800_peaking: Jcm800PeakingNetwork::new(sr as f64),
            plexi_bright_cap: PlexiBrightCapFilter::new(sr as f64),
            passive_tonestack: PassiveToneStack::new(),
            presence_filter: BiquadFilter::default(),
            voicing_filter1: v1,
            voicing_filter2: v2,
            adaa_pre_prev: 0.0,
            adaa_s2_prev: 0.0,
            adaa_s3_prev: 0.0,
            adaa_s4_prev: 0.0,
            adaa_power_prev: 0.0,
            adaa_power_f_prev: 0.0,
            interp_hist: [0.0; 8],
            aa_filters: [BiquadFilter::default(); 4],
            ai_filters: [BiquadFilter::default(); 4],
            oversampling_factor: default_os,
            sag_state: 0.0,
            xformer_lf_state: 0.0,
            speaker_res_filter: BiquadFilter::default(),
            speaker_resonance_hz: 98.0,
            lfo_phase: 0.0,
            standby_env: 1.0,
            smooth_tube_gain: 1.0,
            spring_reverb: SpringReverbTank::default(),
            ac30_vib_buf: [0.0; 512],
            ac30_vib_pos: 0,
            sm_bass: crate::smoother::ParamSmoother::new(48000.0, 0.020, 5.0),
            sm_middle: crate::smoother::ParamSmoother::new(48000.0, 0.020, 5.0),
            sm_treble: crate::smoother::ParamSmoother::new(48000.0, 0.020, 5.0),
            sm_presence: crate::smoother::ParamSmoother::new(48000.0, 0.020, 5.0),
            sm_master: crate::smoother::ParamSmoother::new(48000.0, 0.020, 5.0),
            sm_volume_2: crate::smoother::ParamSmoother::new(48000.0, 0.020, 5.0),
            sm_cut: crate::smoother::ParamSmoother::new(48000.0, 0.020, 5.0),
            sm_speaker_res_hz: crate::smoother::ParamSmoother::new(48000.0, 0.020, 98.0),
            has_processed: false,
            cached_subblock: None,
        };
        s.update_aa_filters();
        s
    }

    fn set_smoother_sample_rate(&mut self, sample_rate: f32) {
        let sr = sample_rate.max(1.0) as f64;
        self.sm_bass.set_sample_rate(sr);
        self.sm_middle.set_sample_rate(sr);
        self.sm_treble.set_sample_rate(sr);
        self.sm_presence.set_sample_rate(sr);
        self.sm_master.set_sample_rate(sr);
        self.sm_volume_2.set_sample_rate(sr);
        self.sm_cut.set_sample_rate(sr);
        self.sm_speaker_res_hz.set_sample_rate(sr);
    }

    fn reset(&mut self) {
        self.bright_state = 0.0;
        self.bright2_state = 0.0;
        self.interstage_hp1 = 0.0;
        self.interstage_hp2 = 0.0;
        self.interstage_hp3 = 0.0;
        self.miller_lp1 = 0.0;
        self.miller_lp2 = 0.0;
        self.ac30_cut_lp = 0.0;
        self.jcm800_peaking.reset();
        self.plexi_bright_cap.reset();
        self.passive_tonestack = PassiveToneStack::new();
        self.presence_filter = BiquadFilter::default();
        let sr = (self.sample_rate as f64) * (self.oversampling_factor.max(2) as f64);
        self.jcm800_peaking.update(sr);
        self.plexi_bright_cap.update(self.panel.input_gain as f64, sr);
        let (v1, v2) = Self::create_voicing_filters(self.model_id, sr);
        self.voicing_filter1 = v1;
        self.voicing_filter2 = v2;
        self.adaa_pre_prev = 0.0;
        self.adaa_s2_prev = 0.0;
        self.adaa_s3_prev = 0.0;
        self.adaa_s4_prev = 0.0;
        self.adaa_power_prev = 0.0;
        self.adaa_power_f_prev = 0.0;
        self.interp_hist = [0.0; 8];
        self.update_aa_filters();
        self.sag_state = 0.0;
        self.xformer_lf_state = 0.0;
        self.speaker_res_filter = BiquadFilter::default();
        self.lfo_phase = 0.0;
        self.smooth_tube_gain = 1.0;
        self.spring_reverb.reset();
        self.spring_reverb.set_sample_rate(self.sample_rate);
        self.ac30_vib_buf.fill(0.0);
        self.ac30_vib_pos = 0;
        self.set_smoother_sample_rate(self.sample_rate);
        self.sm_bass.reset(self.panel.bass as f64);
        self.sm_middle.reset(self.panel.middle as f64);
        self.sm_treble.reset(self.panel.treble as f64);
        self.sm_presence.reset(self.panel.presence as f64);
        self.sm_master.reset(self.panel.master_volume as f64);
        self.sm_volume_2.reset(self.panel.volume_2 as f64);
        self.sm_cut.reset(self.panel.cut as f64);
        self.sm_speaker_res_hz.reset(self.speaker_resonance_hz as f64);
        self.has_processed = false;
        self.cached_subblock = None;
    }

    /// Returns the instantaneous B+ rail voltage including power-supply sag and variac scaling.
    pub fn current_b_plus_v(&self) -> f32 {
        let factor = (1.0 - self.sag_state as f32).max(0.15);
        self.mod_settings.preamp_plate_v * factor
    }

    #[inline]
    fn ganho_estagio1_pequeno_sinal(&self) -> f64 {
        match self.model_id {
            // Fonte: Dr. Tube Marshall JMP/JCM800 2203 schematic (1981) & Rob Robinette, Spec 055 §6.2/§7
            AmpModelId::BritJcm800_2203 => triode_stage_gain(
                JCM800_V1A_RA_OHM,
                JCM800_V1A_RL_OHM,
                JCM800_V1A_RK_UNBYPASSED_OHM,
            ),
            // Fonte: Dr. Tube 1969 SL/A schematic & Manual 1959HW, Spec 054 §7 (V1b Rk=2,7k||0,68uF)
            AmpModelId::BritPlexi1959 => triode_stage_gain(100_000.0, 1_000_000.0, 0.0),
            // Fonte: Rob Robinette AB763, Spec 056 §7 (V1a Rk=1,5k||22uF)
            AmpModelId::UsTwinBlackface65 => triode_stage_gain(100_000.0, 1_000_000.0, 0.0),
            // DECISAO_SONIX_RECTO_S1: 12AX7 ganho padrão (100k/1M, cátodo bypassado) até verificação RF-1F
            AmpModelId::CaliDualRectoRevF => triode_stage_gain(100_000.0, 1_000_000.0, 0.0),
            // Fonte: JMI AC30 Top Boost OS/010 (Rp=220k, Rk=1,5k||25uF)
            AmpModelId::UkAc30TopBoost => triode_stage_gain(220_000.0, 1_000_000.0, 0.0),
        }
    }

    fn update_aa_filters(&mut self) {
        let identity = BiquadFilter {
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
        if self.oversampling_factor <= 1 {
            self.aa_filters = [identity; 4];
            self.ai_filters = [identity; 4];
            return;
        }
        let sos = crate::os_filters::elliptic8_sos(self.sample_rate, self.oversampling_factor);
        for k in 0..4 {
            let b = BiquadFilter {
                b0: sos[k][0],
                b1: sos[k][1],
                b2: sos[k][2],
                a1: sos[k][3],
                a2: sos[k][4],
                x1: 0.0,
                x2: 0.0,
                y1: 0.0,
                y2: 0.0,
            };
            self.aa_filters[k] = b;
            self.ai_filters[k] = b;
        }
    }

    pub fn measure_aa_filter_magnitude_db(&self, freq_hz: f64) -> f64 {
        Self::measure_biquad_chain_db(&self.aa_filters, self.sample_rate, self.oversampling_factor, freq_hz)
    }

    pub fn measure_ai_filter_magnitude_db(&self, freq_hz: f64) -> f64 {
        Self::measure_biquad_chain_db(&self.ai_filters, self.sample_rate, self.oversampling_factor, freq_hz)
    }

    fn measure_biquad_chain_db(
        chain: &[BiquadFilter],
        sample_rate: f32,
        oversampling_factor: usize,
        freq_hz: f64,
    ) -> f64 {
        let fs_os = (sample_rate as f64) * (oversampling_factor.max(1) as f64);
        let w = 2.0 * std::f64::consts::PI * freq_hz / fs_os;
        let cos_w = w.cos();
        let sin_w = w.sin();
        let cos_2w = (2.0 * w).cos();
        let sin_2w = (2.0 * w).sin();
        let mut mag_sq = 1.0f64;
        for f in chain {
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
            AmpModelId::CaliDualRectoRevF => ToneStackTopology::RectoPostGain,
        }
    }

    #[inline]
    fn tonestack_recovery_gain(topo: ToneStackTopology) -> f64 {
        match topo {
            ToneStackTopology::MarshallCathodeFollower | ToneStackTopology::Model => 3.5,
            ToneStackTopology::FenderTmbMidScoop => 5.4,
            ToneStackTopology::VoxTopBoost => 4.2,
            ToneStackTopology::RectoPostGain => 1.2,
            ToneStackTopology::MesaMarkPreGain => 3.5,
            ToneStackTopology::PassiveJamesBaxandall => 2.6,
        }
    }

    #[inline]
    fn any_smoother_ramping(&self) -> bool {
        self.sm_bass.is_ramping()
            || self.sm_middle.is_ramping()
            || self.sm_treble.is_ramping()
            || self.sm_presence.is_ramping()
            || self.sm_master.is_ramping()
            || self.sm_volume_2.is_ramping()
            || self.sm_cut.is_ramping()
            || self.sm_speaker_res_hz.is_ramping()
    }

    fn process_core(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        if input.is_empty() {
            return;
        }
        if !self.has_processed {
            self.sm_bass.reset(self.panel.bass as f64);
            self.sm_middle.reset(self.panel.middle as f64);
            self.sm_treble.reset(self.panel.treble as f64);
            self.sm_presence.reset(self.panel.presence as f64);
            self.sm_master.reset(self.panel.master_volume as f64);
            self.sm_volume_2.reset(self.panel.volume_2 as f64);
            self.sm_cut.reset(self.panel.cut as f64);
            self.sm_speaker_res_hz
                .reset(self.speaker_resonance_hz as f64);
            self.has_processed = true;
        } else {
            self.sm_bass.set_target(self.panel.bass as f64);
            self.sm_middle.set_target(self.panel.middle as f64);
            self.sm_treble.set_target(self.panel.treble as f64);
            self.sm_presence.set_target(self.panel.presence as f64);
            self.sm_master.set_target(self.panel.master_volume as f64);
            self.sm_volume_2.set_target(self.panel.volume_2 as f64);
            self.sm_cut.set_target(self.panel.cut as f64);
            self.sm_speaker_res_hz
                .set_target(self.speaker_resonance_hz as f64);
        }

        if !self.any_smoother_ramping() {
            self.process_subblock(input, output);
            return;
        }

        let mut offset = 0usize;
        while offset < input.len() {
            let chunk = (input.len() - offset).min(4);
            self.sm_bass.advance(chunk);
            self.sm_middle.advance(chunk);
            self.sm_treble.advance(chunk);
            self.sm_presence.advance(chunk);
            self.sm_master.advance(chunk);
            self.sm_volume_2.advance(chunk);
            self.sm_cut.advance(chunk);
            self.sm_speaker_res_hz.advance(chunk);
            self.process_subblock(
                &input[offset..offset + chunk],
                &mut output[offset..offset + chunk],
            );
            offset += chunk;
        }
    }

    fn process_subblock(&mut self, input: &[f32], output: &mut [f32]) {
        let os_factor = self.oversampling_factor.max(1);
        let step_stride = if os_factor <= 1 { 8 } else { (8 / os_factor).max(1) };
        let fs_base = self.sample_rate as f64;
        let sr = fs_base * (os_factor as f64);

        let sm_bass = self.sm_bass.current() as f32;
        let sm_middle = self.sm_middle.current() as f32;
        let sm_treble = self.sm_treble.current() as f32;
        let sm_presence = self.sm_presence.current() as f32;
        let sm_master = self.sm_master.current() as f32;
        let sm_volume_2 = self.sm_volume_2.current() as f32;
        let sm_cut = self.sm_cut.current() as f32;
        let sm_res_hz = (self.sm_speaker_res_hz.current() as f32).clamp(50.0, 200.0) as f64;

        let cache_hit = if let Some(c) = self.cached_subblock {
            c.model_id == self.model_id
                && c.os_factor == os_factor
                && c.fs_base == fs_base
                && c.sm_bass == sm_bass
                && c.sm_middle == sm_middle
                && c.sm_treble == sm_treble
                && c.sm_presence == sm_presence
                && c.sm_master == sm_master
                && c.sm_volume_2 == sm_volume_2
                && c.sm_cut == sm_cut
                && c.sm_res_hz == sm_res_hz
                && c.input_gain == self.panel.input_gain
                && c.output_level_db == self.panel.output_level_db
                && c.channel_mode == self.panel.channel_mode
                && c.channel_toggle == self.panel.channel_toggle
                && c.bright_switch == self.panel.bright_switch
                && c.input_jack == self.panel.input_jack
                && c.jumpered == self.panel.jumpered
                && c.spongy == self.panel.spongy
                && c.impedance_ohms == self.panel.impedance_ohms
                && c.power == self.panel.power
                && c.standby == self.panel.standby
                && c.vibrato_speed == self.panel.vibrato_speed
                && c.vibrato_depth == self.panel.vibrato_depth
                && c.ac30_vib_trem_mode == self.panel.ac30_vib_trem_mode
                && c.reverb == self.panel.reverb
                && c.mod_nfb == self.mod_settings.nfb
                && c.mod_tonestack == self.mod_settings.tonestack
                && c.mod_xformer_sat == self.mod_settings.xformer_sat
                && c.mod_sag == self.mod_settings.sag
                && c.mod_rectifier == self.mod_settings.rectifier
                && c.mod_power_tube == self.mod_settings.power_tube
                && c.mod_preamp_tube == self.mod_settings.preamp_tube
                && c.mod_preamp_plate_v == self.mod_settings.preamp_plate_v
                && c.mod_triode_asym == self.mod_settings.triode_asym
                && c.mod_bias_ma == self.mod_settings.bias_ma
        } else {
            false
        };

        let c = if cache_hit {
            self.cached_subblock.unwrap()
        } else {
            let ts_topo = self.resolve_tonestack_topology(self.model_id);
            let ts_recovery = Self::tonestack_recovery_gain(ts_topo);
            self.passive_tonestack.update(
                ts_topo,
                sm_bass,
                sm_middle,
                sm_treble,
                sr,
            );
            self.jcm800_peaking.update(sr);
            self.plexi_bright_cap.update(self.panel.input_gain as f64, sr);

            // Effective NFB: Dual Rectifier Modern mode disconnects global NFB per Rev-F schematic
            let eff_nfb = if self.model_id == AmpModelId::CaliDualRectoRevF
                && self.panel.channel_mode == AmpChannelMode::Modern
            {
                0.0
            } else {
                self.mod_settings.nfb as f64
            };
            let nfb_scale = (eff_nfb / 10.0).clamp(0.0, 1.0);

            // Feedback loop filter: Presence (>2 kHz) modulated by NFB loop somente nos modelos com Presence real
            if self.model_id.has_presence() {
                let presence_nfb_scale = if self.model_id == AmpModelId::CaliDualRectoRevF
                    && self.panel.channel_mode == AmpChannelMode::Modern
                {
                    DECISAO_SONIX_RECTO_MODERN_PRESENCE
                } else {
                    nfb_scale
                };
                let pres_db = (sm_presence as f64 / 10.0) * 10.0 * presence_nfb_scale;
                self.presence_filter
                    .update_coeffs(BiquadFilter::peaking(4000.0, pres_db, 1.4, sr));
            }

            // Subtle power-section/transformer voicing per model
            let (v1_cur, v2_cur) = if self.mod_settings.tonestack != ToneStackTopology::Model {
                match self.mod_settings.tonestack {
                    ToneStackTopology::FenderTmbMidScoop => (
                        BiquadFilter::identity(),
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
                    ToneStackTopology::RectoPostGain | ToneStackTopology::MesaMarkPreGain => (
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

            let pre_gain_db = -44.0 + (self.panel.input_gain as f64 / 10.0) * 62.0;
            let pre_gain_lin = 10.0f64.powf(pre_gain_db / 20.0);

            let vol2_gain_db = -44.0 + (sm_volume_2 as f64 / 10.0) * 62.0;
            let vol2_lin = if sm_volume_2 <= 0.05 {
                0.0
            } else {
                10.0f64.powf(vol2_gain_db / 20.0)
            };

            let lead_mult = match self.panel.channel_toggle {
                AmpChannelVoicing::Norm => 1.0,
                AmpChannelVoicing::HighLead => 2.5,
            };

            let bright_effective_boost = if self.panel.bright_switch {
                let pot_ratio = (10.0 - self.panel.input_gain) / 10.0;
                (pot_ratio as f64) * 0.95
            } else {
                0.0
            };

            let input_pad = if self.panel.input_jack.is_low_pad() {
                0.5
            } else {
                1.0
            };

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

            let is_plexi_jumpered = self.panel.jumpered || self.panel.input_jack == AmpInputJack::Jumpered;
            let plexi_ch1_on = is_plexi_jumpered || !self.panel.input_jack.is_normal_channel();
            let plexi_ch2_on = is_plexi_jumpered || self.panel.input_jack.is_normal_channel();

            let calc_power_drive = |model: AmpModelId, gain: f32, vol2: f32, master: f32, ch_mode: AmpChannelMode| -> f64 {
                match model {
                    AmpModelId::BritPlexi1959 => {
                        let g1_drive = if plexi_ch1_on { (gain as f64 / 10.0) * 18.0 } else { 0.0 };
                        let g2_drive = if plexi_ch2_on { (vol2 as f64 / 10.0) * 6.0 } else { 0.0 };
                        (1.0 + g1_drive + g2_drive) / power_tube_headroom
                    }
                    AmpModelId::UsTwinBlackface65 => {
                        (0.55 + (gain as f64 / 10.0) * 1.65) / power_tube_headroom
                    }
                    AmpModelId::CaliDualRectoRevF => {
                        let master_pot = master as f64 / 10.0;
                        let m_mult = match ch_mode {
                            AmpChannelMode::Clean => 0.55,
                            AmpChannelMode::Vintage => 1.35,
                            AmpChannelMode::Modern => 1.75,
                        };
                        (0.02 + master_pot.powi(3) * 3.0 * m_mult) / power_tube_headroom
                    }
                    AmpModelId::UkAc30TopBoost => {
                        (1.2 + (gain as f64 / 10.0) * 5.0 + (vol2 as f64 / 10.0) * 1.6) / power_tube_headroom
                    }
                    AmpModelId::BritJcm800_2203 => {
                        let master_pot = master as f64 / 10.0;
                        (0.25 + master_pot.powi(3) * 28.0) / power_tube_headroom
                    }
                }
            };

            let power_saturation_drive = calc_power_drive(
                self.model_id,
                self.panel.input_gain,
                sm_volume_2,
                sm_master,
                self.panel.channel_mode,
            );

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
            let bias_warm_drive = 1.0 + ((bias_ma_f - 40.0).max(0.0) / 24.0).min(0.25);

            let alpha_bright = (1.0 - (-2.0 * std::f64::consts::PI * 1200.0 / fs_base.max(1.0)).exp()).clamp(0.001, 0.99);
            let alpha_ch2 = (1.0 - (-2.0 * std::f64::consts::PI * 310.0 / fs_base.max(1.0)).exp()).clamp(0.001, 0.99);
            let alpha_standby = (1.0 - (-1.0 / (fs_base.max(1.0) * 0.00014)).exp()).clamp(0.001, 0.99);

            let imp_factor = match self.panel.impedance_ohms {
                4 => 1.24,
                8 => 1.0,
                _ => 0.78,
            };
            let eff_nfb_damping = (eff_nfb / imp_factor).clamp(0.0, 10.0);
            let res_boost_db = ((10.0 - eff_nfb_damping) / 10.0) * 5.0 * imp_factor;
            let res_hz = sm_res_hz;
            self.speaker_res_filter
                .update_coeffs(BiquadFilter::peaking(res_hz, res_boost_db, 2.0, sr));

            let fc_cut_hz = 18000.0 * (750.0f64 / 18000.0).powf((sm_cut as f64 / 10.0).clamp(0.0, 1.0));
            let alpha_cut = (2.0 * std::f64::consts::PI * fc_cut_hz / sr).clamp(0.005, 0.99);

            let alpha_hp_loose = (2.0 * std::f64::consts::PI * 45.0 / sr).clamp(0.0001, 0.25);
            let alpha_hp_tight = (2.0 * std::f64::consts::PI * 115.0 / sr).clamp(0.0001, 0.35);
            let alpha_miller = (2.0 * std::f64::consts::PI * 7500.0 / sr).clamp(0.01, 0.90);

            let xf_val = self.mod_settings.xformer_sat as f64;
            let sag_amt = self.mod_settings.sag as f64;
            let ri = self.mod_settings.rectifier.internal_resistance();
            let tau_att = self.mod_settings.rectifier.attack_time_sec();
            let tau_rel = self.mod_settings.rectifier.release_time_sec();
            let alpha_sag_att = 1.0 - (-1.0 / (sr * tau_att)).exp();
            let alpha_sag_rel = 1.0 - (-1.0 / (sr * tau_rel)).exp();
            let alpha_ac30_brilliant_hp = 1.0
                - (-2.0 * std::f64::consts::PI * DECISAO_SONIX_AC30_BRILLIANT_HP_HZ / sr).exp();

            let out_lin = 10.0f64.powf(self.panel.output_level_db as f64 / 20.0);
            let live_target = if self.panel.power && !self.panel.standby { 1.0 } else { 0.0 };

            let lfo_hz = 1.5 + (self.panel.vibrato_speed as f64 / 10.0) * 8.0;
            let lfo_inc = (2.0 * std::f64::consts::PI * lfo_hz) / fs_base.max(1.0);
            let is_twin_vibrato_ch = self.model_id == AmpModelId::UsTwinBlackface65
                && !self.panel.input_jack.is_normal_channel();
            let is_ac30_vibtrem_ch = self.model_id == AmpModelId::UkAc30TopBoost
                && self.panel.input_jack == AmpInputJack::Vibrato;
            let is_ac30_trem = is_ac30_vibtrem_ch
                && self.panel.ac30_vib_trem_mode == Ac30VibTremMode::Tremolo
                && self.panel.vibrato_depth > 0.01;
            let is_ac30_vib = is_ac30_vibtrem_ch
                && self.panel.ac30_vib_trem_mode == Ac30VibTremMode::Vibrato
                && self.panel.vibrato_depth > 0.01;
            let trem_depth = if is_twin_vibrato_ch || is_ac30_trem {
                (self.panel.vibrato_depth as f64 / 10.0).clamp(0.0, 1.0) * 0.70
            } else {
                0.0
            };
            let rev_mix = if is_twin_vibrato_ch {
                (self.panel.reverb as f64 / 10.0).clamp(0.0, 1.0) * 3.10
            } else {
                0.0
            };

            let built = AmpSubblockCache {
                model_id: self.model_id,
                os_factor,
                fs_base,
                sm_bass,
                sm_middle,
                sm_treble,
                sm_presence,
                sm_master,
                sm_volume_2,
                sm_cut,
                sm_res_hz,
                input_gain: self.panel.input_gain,
                output_level_db: self.panel.output_level_db,
                channel_mode: self.panel.channel_mode,
                channel_toggle: self.panel.channel_toggle,
                bright_switch: self.panel.bright_switch,
                input_jack: self.panel.input_jack,
                jumpered: self.panel.jumpered,
                spongy: self.panel.spongy,
                impedance_ohms: self.panel.impedance_ohms,
                power: self.panel.power,
                standby: self.panel.standby,
                vibrato_speed: self.panel.vibrato_speed,
                vibrato_depth: self.panel.vibrato_depth,
                ac30_vib_trem_mode: self.panel.ac30_vib_trem_mode,
                reverb: self.panel.reverb,
                mod_nfb: self.mod_settings.nfb,
                mod_tonestack: self.mod_settings.tonestack,
                mod_xformer_sat: self.mod_settings.xformer_sat,
                mod_sag: self.mod_settings.sag,
                mod_rectifier: self.mod_settings.rectifier,
                mod_power_tube: self.mod_settings.power_tube,
                mod_preamp_tube: self.mod_settings.preamp_tube,
                mod_preamp_plate_v: self.mod_settings.preamp_plate_v,
                mod_triode_asym: self.mod_settings.triode_asym,
                mod_bias_ma: self.mod_settings.bias_ma,
                ts_recovery,
                eff_nfb,
                pre_gain_lin,
                vol2_lin,
                lead_mult,
                bright_effective_boost,
                input_pad,
                power_tube_headroom,
                variac_scale,
                spongy_power_atten,
                plexi_ch1_on,
                plexi_ch2_on,
                power_saturation_drive,
                target_tube_gain,
                ef86_knee,
                tube_alpha,
                v1_headroom,
                asym,
                bias_ma_f,
                crossover_depth,
                bias_warm_drive,
                alpha_bright,
                alpha_ch2,
                alpha_standby,
                alpha_cut,
                alpha_hp_loose,
                alpha_hp_tight,
                alpha_miller,
                xf_val,
                sag_amt,
                ri,
                alpha_sag_att,
                alpha_sag_rel,
                alpha_ac30_brilliant_hp,
                out_lin,
                live_target,
                lfo_inc,
                is_ac30_vib,
                trem_depth,
                rev_mix,
            };
            self.cached_subblock = Some(built);
            built
        };

        let ts_recovery = c.ts_recovery;
        let eff_nfb = c.eff_nfb;
        let pre_gain_lin = c.pre_gain_lin;
        let vol2_lin = c.vol2_lin;
        let lead_mult = c.lead_mult;
        let bright_effective_boost = c.bright_effective_boost;
        let input_pad = c.input_pad;
        let power_tube_headroom = c.power_tube_headroom;
        let variac_scale = c.variac_scale;
        let spongy_power_atten = c.spongy_power_atten;
        let plexi_ch1_on = c.plexi_ch1_on;
        let plexi_ch2_on = c.plexi_ch2_on;
        let power_saturation_drive = c.power_saturation_drive;
        let target_tube_gain = c.target_tube_gain;
        let ef86_knee = c.ef86_knee;
        let tube_alpha = c.tube_alpha;
        let v1_headroom = c.v1_headroom;
        let asym = c.asym;
        let bias_ma_f = c.bias_ma_f;
        let crossover_depth = c.crossover_depth;
        let crossover_width = 0.18;
        let bias_warm_drive = c.bias_warm_drive;
        let alpha_bright = c.alpha_bright;
        let alpha_ch2 = c.alpha_ch2;
        let alpha_standby = c.alpha_standby;
        let alpha_cut = c.alpha_cut;
        let alpha_hp_loose = c.alpha_hp_loose;
        let alpha_hp_tight = c.alpha_hp_tight;
        let alpha_miller = c.alpha_miller;
        let xf_val = c.xf_val;
        let sag_amt = c.sag_amt;
        let ri = c.ri;
        let alpha_sag_att = c.alpha_sag_att;
        let alpha_sag_rel = c.alpha_sag_rel;
        let alpha_ac30_brilliant_hp = c.alpha_ac30_brilliant_hp;
        let out_lin = c.out_lin;
        let live_target = c.live_target;
        let lfo_inc = c.lfo_inc;
        let is_ac30_vib = c.is_ac30_vib;
        let trem_depth = c.trem_depth;
        let rev_mix = c.rev_mix;

        // COR-12: constantes por sub-bloco içadas para fora do laço de amostras e de oversampling
        let v1b_drive_plexi = triode_stage_gain(100_000.0, 1_000_000.0, 0.0) / DECISAO_SONIX_PREAMP_STAGE_NORM;
        let v2a_gain_plexi = triode_stage_gain(100_000.0, 1_000_000.0, 820.0) / DECISAO_SONIX_PREAMP_STAGE_NORM;
        let v1a_gain_estagio1 = self.ganho_estagio1_pequeno_sinal();
        let alpha_grid_jcm = (2.0 * std::f64::consts::PI * 6400.0 / sr).clamp(0.01, 0.85);
        let alpha_miller_jcm = (2.0 * std::f64::consts::PI * 7000.0 / sr).clamp(0.01, 0.85);
        let v1b_gain_jcm = triode_stage_gain(100_000.0, 470_000.0, 10_000.0);
        let s2_gain_recto = triode_stage_gain(
            DECISAO_SONIX_RECTO_S2_RA,
            DECISAO_SONIX_RECTO_S2_RL,
            DECISAO_SONIX_RECTO_S2_RK_UNBYPASSED,
        ) / DECISAO_SONIX_PREAMP_STAGE_NORM;
        let s3_gain_recto = triode_stage_gain(
            DECISAO_SONIX_RECTO_S3_RA,
            DECISAO_SONIX_RECTO_S3_RL,
            DECISAO_SONIX_RECTO_S3_RK_UNBYPASSED,
        );
        let s4_gain_recto = triode_stage_gain(
            DECISAO_SONIX_RECTO_S4_RA,
            DECISAO_SONIX_RECTO_S4_RL,
            DECISAO_SONIX_RECTO_S4_RK_UNBYPASSED,
        ) / DECISAO_SONIX_PREAMP_STAGE_NORM;
        let tb_v2_gain_ac30 = triode_stage_gain(100_000.0, 220_000.0, 1_500.0) / DECISAO_SONIX_PREAMP_STAGE_NORM;
        let alpha_ac30_brilliant = alpha_ac30_brilliant_hp.clamp(0.0001, 0.5);
        let kp_base = (1.0 + 1.2 * asym) * ef86_knee;
        let kn_base = ((1.0 - 0.55 * asym) * ef86_knee).max(0.35);
        let kp_cold = (1.0 + 11.0 * asym) * ef86_knee;
        let kn_cold = ((1.0 - 2.6 * asym) * ef86_knee).max(0.35);
        let twin_headroom = v1_headroom * 2.50;
        let skip_post_tonestack = (self.model_id == AmpModelId::UsTwinBlackface65)
            || (self.model_id == AmpModelId::UkAc30TopBoost
                && self.panel.input_jack.is_normal_channel()
                && self.mod_settings.tonestack == ToneStackTopology::Model);
        let is_low_pad = self.panel.input_jack.is_low_pad();
        let is_ac30_norm = self.panel.input_jack.is_normal_channel();
        let (recto_stage_drive, recto_n_stages) = match self.panel.channel_mode {
            AmpChannelMode::Clean => (0.65, 2usize),
            AmpChannelMode::Vintage => (2.15, 4usize),
            AmpChannelMode::Modern => (3.10, 4usize),
        };
        let nfb_linearization = 1.0 / (1.0 + (eff_nfb / 10.0) * 1.15);
        let tube_out_scale = (0.65 + 0.35 * power_tube_headroom)
            * (0.93 + 0.07 * ((bias_ma_f - 20.0) / 35.0).clamp(0.0, 1.0));
        let power_out_scale = 0.30 * tube_out_scale * spongy_power_atten;
        let has_presence = self.model_id.has_presence();
        let lf_alpha = (150.0 * 2.0 * std::f64::consts::PI / sr).min(0.2);

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
            self.bright2_state =
                crate::denormal::flush_f64(self.bright2_state + alpha_ch2 * (s_in - self.bright2_state));
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

            let raw_drive_jcm = pre_gain_lin * lead_mult * tube_gain;
            let base_drive_jcm = {
                let r = raw_drive_jcm / 18.5;
                let r2 = r * r;
                raw_drive_jcm / (1.0 + r2 * r2).sqrt().sqrt()
            };

            for step in (0..8).step_by(step_stride) {
                let s = if os_factor <= 1 {
                    s_in
                } else {
                    let mut acc = if step == 0 {
                        (os_factor as f64) * s_in
                    } else {
                        0.0
                    };
                    for f in &mut self.ai_filters {
                        acc = f.process(acc);
                    }
                    acc
                };

                // 2. Authentic Per-Model Multi-Stage Preamp Topology
                let v_pre_out = match self.model_id {
                    AmpModelId::BritPlexi1959 => {
                        let ch1_sig = if plexi_ch1_on {
                            let g1 = (s * v1b_drive_plexi * lead_mult * tube_gain) / v1_headroom;
                            let v1b_out = triode_adaa1(g1, &mut self.adaa_pre_prev, kp_base, kn_base) * v1_headroom;
                            self.plexi_bright_cap.process(v1b_out)
                        } else {
                            0.0
                        };
                        let ch2_sig = if plexi_ch2_on {
                            let g2 = (s_ch2_base * tube_gain) / v1_headroom;
                            triode_adaa1(g2, &mut self.adaa_s2_prev, kp_base * 0.9, kn_base) * v1_headroom
                        } else {
                            0.0
                        };
                        let sum_v2 = one_pole_hpf(&mut self.interstage_hp1, 0.5 * (ch1_sig + ch2_sig), alpha_hp_loose);
                        triode_adaa1((sum_v2 * v2a_gain_plexi) / v1_headroom, &mut self.adaa_s3_prev, kp_base, kn_base)
                            * v1_headroom
                    }
                    AmpModelId::BritJcm800_2203 => {
                        let s_after_v1a = if is_low_pad {
                            s * (1.05 * raw_drive_jcm / v1a_gain_estagio1)
                        } else {
                            self.miller_lp2 = crate::denormal::flush_f64(
                                self.miller_lp2 + alpha_grid_jcm * (s - self.miller_lp2),
                            );
                            let s_v1a_in = self.miller_lp2;
                            triode_adaa1(
                                (s_v1a_in * 1.05 * base_drive_jcm) / v1_headroom,
                                &mut self.adaa_pre_prev,
                                kp_base,
                                kn_base,
                            ) * v1_headroom
                        };
                        let v1b_in = one_pole_hpf(&mut self.interstage_hp1, s_after_v1a, alpha_hp_tight);
                        let v1b = triode_adaa1((v1b_in * v1b_gain_jcm) / v1_headroom, &mut self.adaa_s2_prev, kp_cold, kn_cold)
                            * v1_headroom;
                        let v1b_peaked = self.jcm800_peaking.process(v1b);
                        self.miller_lp1 = crate::denormal::flush_f64(
                            self.miller_lp1 + alpha_miller_jcm * (v1b_peaked - self.miller_lp1),
                        );
                        let v2_in = one_pole_hpf(&mut self.interstage_hp2, self.miller_lp1, alpha_hp_loose);
                        triode_adaa1((v2_in * 1.0) / v1_headroom, &mut self.adaa_s3_prev, kp_base, kn_base)
                            * v1_headroom
                    }
                    AmpModelId::UsTwinBlackface65 => {
                        let g_v1a = (s * pre_gain_lin * 1.95 * lead_mult * tube_gain) / twin_headroom;
                        let v1a = triode_adaa1(g_v1a, &mut self.adaa_pre_prev, kp_base * 0.78, kn_base * 0.88) * twin_headroom;
                        let v1a_hp = one_pole_hpf(&mut self.interstage_hp1, v1a, alpha_hp_loose * 0.40);
                        let after_ts = self.passive_tonestack.process(v1a_hp) * ts_recovery;
                        let g_v1b = (after_ts * 1.75) / twin_headroom;
                        triode_adaa1(g_v1b, &mut self.adaa_s2_prev, kp_base * 0.82, kn_base * 0.90) * twin_headroom
                    }
                    AmpModelId::CaliDualRectoRevF => {
                        let hp_a = alpha_hp_loose;
                        let g1 = (s * pre_gain_lin * lead_mult * recto_stage_drive * tube_gain) / v1_headroom;
                        let s1 = triode_adaa1(g1, &mut self.adaa_pre_prev, kp_base, kn_base) * v1_headroom;
                        let hp1 = one_pole_hpf(&mut self.interstage_hp1, s1, hp_a);
                        let s2 = triode_adaa1((hp1 * s2_gain_recto) / v1_headroom, &mut self.adaa_s2_prev, kp_base * 1.25, kn_base * 0.85) * v1_headroom;
                        if recto_n_stages == 2 {
                            s2
                        } else {
                            let hp2 = one_pole_hpf(&mut self.interstage_hp2, s2, hp_a);
                            let s3 = triode_adaa1((hp2 * s3_gain_recto) / v1_headroom, &mut self.adaa_s3_prev, kp_base * 1.45, kn_base * 0.7) * v1_headroom;
                            if recto_n_stages == 3 {
                                s3
                            } else {
                                self.miller_lp1 = crate::denormal::flush_f64(
                                    self.miller_lp1 + alpha_miller * (s3 - self.miller_lp1),
                                );
                                triode_adaa1((self.miller_lp1 * s4_gain_recto) / v1_headroom, &mut self.adaa_s4_prev, kp_base * 1.15, kn_base * 0.9) * v1_headroom
                            }
                        }
                    }
                    AmpModelId::UkAc30TopBoost => {
                        let g1 = (s * pre_gain_lin * lead_mult * tube_gain * 1.05) / v1_headroom;
                        let v1 = triode_adaa1(g1, &mut self.adaa_pre_prev, kp_base * 1.05, kn_base * 0.92) * v1_headroom;
                        if is_ac30_norm {
                            v1
                        } else {
                            let hp1 = one_pole_hpf(&mut self.interstage_hp1, v1, alpha_ac30_brilliant);
                            triode_adaa1((hp1 * tb_v2_gain_ac30) / v1_headroom, &mut self.adaa_s2_prev, kp_base * 1.1, kn_base * 0.88)
                                * v1_headroom
                        }
                    }
                };

                // 3. Passive Interactive Tone Stack pós-pré (Twin AB763 já passou entre V1a e V1b; AC30 Normal bypassa Top Boost)
                let v_base = if skip_post_tonestack {
                    v_pre_out
                } else {
                    self.passive_tonestack.process(v_pre_out) * ts_recovery
                };

                rev_feed_acc += v_base;
                sub_count += 1;

                // Model Voicing
                let v_voiced = {
                    let va = self.voicing_filter1.process(v_base);
                    self.voicing_filter2.process(va)
                };

                // Twin Reverb vibrato channel mixes Spring Reverb before PI
                let v_with_rev = v_voiced + rev_tap_out * rev_mix;

                // AC30 Tone Cut shunt low-pass after LTP PI (and mild active on others if desired, primary on AC30)
                let v_post_cut = if self.model_id == AmpModelId::UkAc30TopBoost {
                    self.ac30_cut_lp = crate::denormal::flush_f64(
                        self.ac30_cut_lp + alpha_cut * (v_with_rev - self.ac30_cut_lp),
                    );
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
                    self.adaa_power_f_prev = f64::NAN;
                    power_in.tanh()
                } else {
                    let f1 = if power_in.abs() > 20.0 {
                        power_in.abs() - std::f64::consts::LN_2
                    } else {
                        power_in.cosh().ln()
                    };
                    let f0 = if !self.adaa_power_f_prev.is_nan() {
                        self.adaa_power_f_prev
                    } else if self.adaa_power_prev.abs() > 20.0 {
                        self.adaa_power_prev.abs() - std::f64::consts::LN_2
                    } else {
                        self.adaa_power_prev.cosh().ln()
                    };
                    self.adaa_power_f_prev = f1;
                    (f1 - f0) / dp
                };
                self.adaa_power_prev = power_in;

                // NFB Linearization on power stage distortion (Modern Dual Rect / AC30 use eff_nfb=0)
                let soft_rail = (power_in_raw * 0.6).tanh() / 0.6;
                let power_sat = soft_rail + (power_sat_raw - soft_rail) * nfb_linearization;

                // 5. Speaker Resonance (NFB + Secondary Impedance) + Presence + Power Tube Headroom/Bias
                let mut power_out = self
                    .speaker_res_filter
                    .process(power_sat * power_out_scale);
                if has_presence {
                    power_out = self.presence_filter.process(power_out);
                }

                // 6. Transformer Saturation (Feedforward Frequency-Dependent LF Core Saturation at 150 Hz)
                if xf_val > 0.01 {
                    self.xformer_lf_state = crate::denormal::flush_f64(
                        self.xformer_lf_state + lf_alpha * (power_sat - self.xformer_lf_state),
                    );
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
                    alpha_sag_att
                } else {
                    alpha_sag_rel
                };
                self.sag_state = crate::denormal::flush_f64(
                    self.sag_state + alpha_sag * (target_drop - self.sag_state),
                );
                let b_plus_factor = ((1.0 - self.sag_state) * variac_scale).max(0.15);

                // Safe output limiter inside oversampled loop (limiar 0.78 preserva ataque transiente; teto 0.85 evita Gibbs > 0.999 pós-AA)
                let lim_in = power_out * b_plus_factor * self.standby_env;
                let abs_s = lim_in.abs();
                let mut lim_out = if abs_s <= 0.78 {
                    lim_in
                } else {
                    lim_in.signum() * (0.78 + 0.07 * ((abs_s - 0.78) / 0.07).tanh())
                };

                // Anti-Aliasing Decimation Filter (apenas quando os_factor > 1)
                if os_factor > 1 {
                    for f in &mut self.aa_filters[..4] {
                        lim_out = f.process(lim_out);
                    }
                }

                last_sample = lim_out;
            }

            if sub_count > 0 {
                self.interstage_hp3 = crate::denormal::flush_f64(rev_feed_acc / (sub_count as f64));
            }

            if is_ac30_vib {
                self.ac30_vib_buf[self.ac30_vib_pos] = crate::denormal::flush_f64(last_sample);
                let depth_norm = (self.panel.vibrato_depth as f64 / 10.0).clamp(0.0, 1.0);
                let d_ms = 1.5 + 0.5 * depth_norm * self.lfo_phase.sin();
                let d_samp = (d_ms * 0.001 * fs_base).clamp(0.0, 510.0);
                let r_pos = (self.ac30_vib_pos as f64 + 512.0 - d_samp) % 512.0;
                let idx0 = (r_pos.floor() as usize) & 511;
                let idx1 = (idx0 + 1) & 511;
                let frac = r_pos - r_pos.floor();
                last_sample = self.ac30_vib_buf[idx0] * (1.0 - frac) + self.ac30_vib_buf[idx1] * frac;
                self.ac30_vib_pos = (self.ac30_vib_pos + 1) & 511;
            }

            let out_sample = (last_sample.clamp(-0.999, 0.999) * out_lin * self.standby_env).clamp(-1.0, 1.0);
            output[i] = if out_sample.is_finite() {
                out_sample as f32
            } else {
                0.0
            };
        }
    }
}

impl WhiteBoxAmpProcessor {
    pub fn create_voicing_filters(model: AmpModelId, sr: f64) -> (BiquadFilter, BiquadFilter) {
        AmpCore::create_voicing_filters(model, sr)
    }

    pub fn new(model_id: AmpModelId) -> Self {
        let measured_latencies = Self::compute_measured_latencies(48_000.0);
        Self {
            model_id,
            amp: Box::new(AmpCore::new(model_id)),
            amp_transicao: Box::new(AmpCore::new(model_id)),
            transicao_ativa: false,
            transicao_amostras_feitas: 0,
            transicao_total_amostras: 960,
            transicao_buf: [0.0; 64],
            measured_latencies,
        }
    }

    fn os_index(os_factor: usize) -> usize {
        match os_factor {
            1 => 0,
            2 => 1,
            4 => 2,
            _ => 3,
        }
    }

    fn measure_impulse_latency_for(sample_rate: f32, os_factor: usize) -> u32 {
        let mut core = Box::new(AmpCore::new(AmpModelId::BritJcm800_2203));
        core.sample_rate = sample_rate.max(1.0);
        core.oversampling_factor = match os_factor {
            1 => 1,
            2 => 2,
            4 => 4,
            _ => 8,
        };
        core.panel.input_gain = 1.0;
        core.panel.master_volume = 5.0;
        core.standby_env = 1.0;
        core.set_smoother_sample_rate(core.sample_rate);
        core.spring_reverb.set_sample_rate(core.sample_rate);
        core.update_aa_filters();
        let mut input = [0.0f32; 256];
        input[0] = 0.01;
        let mut output = [0.0f32; 256];
        core.process_core(&input[0..128], &mut output[0..128]);
        core.process_core(&input[128..256], &mut output[128..256]);
        crate::regua::atraso_por_impulso(&output) as u32
    }

    fn compute_measured_latencies(sample_rate: f32) -> [u32; 4] {
        [
            Self::measure_impulse_latency_for(sample_rate, 1),
            Self::measure_impulse_latency_for(sample_rate, 2),
            Self::measure_impulse_latency_for(sample_rate, 4),
            Self::measure_impulse_latency_for(sample_rate, 8),
        ]
    }

    pub fn set_power(&mut self, power: bool) {
        self.amp.panel.power = power;
        self.amp_transicao.panel.power = power;
    }

    pub fn set_standby(&mut self, standby: bool) {
        self.amp.panel.standby = standby;
        self.amp_transicao.panel.standby = standby;
    }

    pub fn set_input_jack(&mut self, jack: AmpInputJack) {
        self.amp.panel.input_jack = jack;
        self.amp.panel.jumpered = jack == AmpInputJack::Jumpered;
        self.amp_transicao.panel.input_jack = jack;
        self.amp_transicao.panel.jumpered = self.amp.panel.jumpered;
    }

    pub fn set_jumpered(&mut self, jumpered: bool) {
        self.amp.panel.jumpered = jumpered;
        if jumpered {
            self.amp.panel.input_jack = AmpInputJack::Jumpered;
        } else if self.amp.panel.input_jack == AmpInputJack::Jumpered {
            self.amp.panel.input_jack = AmpInputJack::High1;
        }
        self.amp_transicao.panel.jumpered = jumpered;
        self.amp_transicao.panel.input_jack = self.amp.panel.input_jack;
    }

    pub fn set_channel_mode(&mut self, mode: AmpChannelMode) {
        self.amp.panel.channel_mode = mode;
        self.amp_transicao.panel.channel_mode = mode;
    }

    pub fn set_spongy_power(&mut self, spongy: bool) {
        self.amp.panel.spongy = spongy;
        self.amp_transicao.panel.spongy = spongy;
    }

    pub fn set_secondary_impedance_ohms(&mut self, ohms: u8) {
        let clamped = match ohms {
            4 | 8 | 16 => ohms,
            _ => 16,
        };
        self.amp.panel.impedance_ohms = clamped;
        self.amp_transicao.panel.impedance_ohms = clamped;
    }

    pub fn set_preamp_tube_type(&mut self, tube: PreampTube) {
        self.amp.mod_settings.preamp_tube = tube;
    }

    pub fn set_power_tube_type(&mut self, tube: PowerTube) {
        self.amp.mod_settings.power_tube = tube;
    }

    pub fn set_power_tube_bias_ma(&mut self, ma: f32) {
        self.amp.mod_settings.bias_ma = ma.clamp(20.0, 55.0);
    }

    pub fn set_sag_amount(&mut self, sag: f32) {
        self.amp.mod_settings.sag = sag.clamp(0.0, 10.0);
    }

    pub fn set_rectifier_type(&mut self, rec: RectifierType) {
        self.amp.mod_settings.rectifier = rec;
    }

    /// Returns the instantaneous B+ rail voltage including power-supply sag and variac scaling.
    pub fn current_b_plus_v(&self) -> f32 {
        self.amp.current_b_plus_v()
    }

    pub fn ganho_estagio1_pequeno_sinal(&self) -> f64 {
        self.amp.ganho_estagio1_pequeno_sinal()
    }

    pub fn set_negative_feedback(&mut self, nfb: f32) {
        self.amp.mod_settings.nfb = nfb.clamp(0.0, 10.0);
    }

    pub fn set_transformer_saturation(&mut self, sat: f32) {
        self.amp.mod_settings.xformer_sat = sat.clamp(0.0, 10.0);
    }

    pub fn set_preamp_plate_voltage(&mut self, v: f32) {
        self.amp.mod_settings.preamp_plate_v = v.clamp(180.0, 340.0);
    }

    pub fn set_triode_asymmetry(&mut self, asym: f32) {
        self.amp.mod_settings.triode_asym = asym.clamp(0.0, 0.60);
    }

    pub fn set_tonestack_topology(&mut self, ts: ToneStackTopology) {
        self.amp.mod_settings.tonestack = ts;
    }

    pub fn set_target_model(&mut self, model: AmpModelId) {
        if !self.transicao_ativa && model == self.amp.model_id {
            return;
        }
        if self.transicao_ativa && model == self.amp_transicao.model_id {
            return;
        }
        self.amp_transicao.model_id = model;
        self.amp_transicao.panel = self.amp.panel.clone();
        self.amp_transicao.mod_settings = nominal_mod_settings_for_model(model);
        self.amp_transicao.sample_rate = self.amp.sample_rate;
        self.amp_transicao.oversampling_factor = self.amp.oversampling_factor;
        self.amp_transicao.speaker_resonance_hz = self.amp.speaker_resonance_hz;
        self.amp_transicao.reset();
        self.amp_transicao.standby_env = self.amp.standby_env;

        self.transicao_ativa = true;
        self.transicao_amostras_feitas = 0;
        self.transicao_total_amostras = (self.amp.sample_rate * 0.020).round().max(1.0) as usize;
    }

    pub fn set_oversampling_factor(&mut self, factor: usize) {
        let new_factor = match factor {
            1 => 1,
            2 => 2,
            4 => 4,
            _ => 8,
        };
        if self.amp.oversampling_factor != new_factor {
            self.amp.oversampling_factor = new_factor;
            self.amp.update_aa_filters();
        }
        if self.amp_transicao.oversampling_factor != new_factor {
            self.amp_transicao.oversampling_factor = new_factor;
            self.amp_transicao.update_aa_filters();
        }
    }

    pub fn algorithmic_latency_samples(&self) -> u32 {
        self.measured_latencies[Self::os_index(self.amp.oversampling_factor)]
    }

    pub fn measure_aa_filter_magnitude_db(&self, freq_hz: f64) -> f64 {
        self.amp.measure_aa_filter_magnitude_db(freq_hz)
    }

    pub fn measure_ai_filter_magnitude_db(&self, freq_hz: f64) -> f64 {
        self.amp.measure_ai_filter_magnitude_db(freq_hz)
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        let sr = sample_rate.max(1.0);
        self.amp.sample_rate = sr;
        self.amp.set_smoother_sample_rate(sr);
        self.amp.spring_reverb.set_sample_rate(sr);
        self.amp.update_aa_filters();
        self.amp_transicao.sample_rate = sr;
        self.amp_transicao.set_smoother_sample_rate(sr);
        self.amp_transicao.spring_reverb.set_sample_rate(sr);
        self.amp_transicao.update_aa_filters();
        self.measured_latencies = Self::compute_measured_latencies(sr);
    }

    pub fn set_speaker_resonance_hz(&mut self, hz: f32) {
        let clamped = hz.clamp(50.0, 200.0);
        self.amp.speaker_resonance_hz = clamped;
        self.amp_transicao.speaker_resonance_hz = clamped;
    }

    pub fn speaker_resonance_hz(&self) -> f32 {
        self.amp.speaker_resonance_hz
    }

    pub fn panel(&self) -> &AmpFrontPanel {
        &self.amp.panel
    }

    pub fn panel_mut(&mut self) -> &mut AmpFrontPanel {
        &mut self.amp.panel
    }

    pub fn mod_settings(&self) -> &ChassisModSettings {
        &self.amp.mod_settings
    }

    pub fn mod_settings_mut(&mut self) -> &mut ChassisModSettings {
        &mut self.amp.mod_settings
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        if !self.transicao_ativa {
            self.amp.process_core(input, output);
            return;
        }

        // Mantém painel do alvo sincronizado com alterações em andamento
        self.amp_transicao.panel = self.amp.panel.clone();

        let total = self.transicao_total_amostras.max(1);
        let mut offset = 0usize;
        while offset < input.len() {
            if !self.transicao_ativa {
                self.amp.process_core(&input[offset..], &mut output[offset..]);
                break;
            }
            let restantes_trans = total.saturating_sub(self.transicao_amostras_feitas);
            if restantes_trans == 0 {
                std::mem::swap(&mut self.amp, &mut self.amp_transicao);
                self.model_id = self.amp.model_id;
                self.transicao_ativa = false;
                self.amp.process_core(&input[offset..], &mut output[offset..]);
                break;
            }
            let chunk = (input.len() - offset).min(64).min(restantes_trans);
            self.amp
                .process_core(&input[offset..offset + chunk], &mut output[offset..offset + chunk]);
            self.amp_transicao.process_core(
                &input[offset..offset + chunk],
                &mut self.transicao_buf[..chunk],
            );
            for k in 0..chunk {
                let prog = (self.transicao_amostras_feitas + k) as f32 / (total as f32);
                let theta = prog.clamp(0.0, 1.0) * std::f32::consts::FRAC_PI_2;
                let ga = theta.cos();
                let gb = theta.sin();
                output[offset + k] = output[offset + k] * ga + self.transicao_buf[k] * gb;
            }
            self.transicao_amostras_feitas += chunk;
            offset += chunk;
            if self.transicao_amostras_feitas >= total {
                std::mem::swap(&mut self.amp, &mut self.amp_transicao);
                self.model_id = self.amp.model_id;
                self.transicao_ativa = false;
            }
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
        let fs = self.amp.sample_rate;
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
