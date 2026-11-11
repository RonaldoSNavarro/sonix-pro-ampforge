//! crates/ampforge-dsp/src/amp.rs
//! Modelagem White-Box de Amplificadores Valvulados e Painel Frontal Autêntico

use crate::modshop::{PowerTube, PreampTube, RectifierType, ToneStackTopology};

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
    bass_filter: BiquadFilter,
    mid_filter: BiquadFilter,
    treb_filter: BiquadFilter,
    depth_filter: BiquadFilter,
    presence_filter: BiquadFilter,
    voicing_filter1: BiquadFilter,
    voicing_filter2: BiquadFilter,
    adaa_pre_prev: f64,
    adaa_power_prev: f64,
    interp_hist: [f64; 8],
    aa_filters: [BiquadFilter; 14],
    ai_filters: [BiquadFilter; 4],
    target_model_id: Option<AmpModelId>,
    target_voicing_filter1: BiquadFilter,
    target_voicing_filter2: BiquadFilter,
    model_crossfade: f32,
    crossfade_step: f32,
    oversampling_factor: usize,
    sag_state: f64,
    xformer_lf_state: f64,
    speaker_res_filter: BiquadFilter,
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
        match model {
            AmpModelId::BritJcm800_2203 => (
                BiquadFilter::peaking(1600.0, 3.5, 1.2, sr),
                BiquadFilter::low_shelf(120.0, -2.5, sr),
            ),
            AmpModelId::BritPlexi1959 => (
                BiquadFilter::peaking(2500.0, 6.0, 0.9, sr),
                BiquadFilter::peaking(120.0, 4.0, 1.0, sr),
            ),
            AmpModelId::UsTwinBlackface65 => (
                BiquadFilter::peaking(450.0, -8.0, 0.9, sr),
                BiquadFilter::high_shelf(4500.0, 4.0, sr),
            ),
            AmpModelId::CaliDualRectoRevF => (
                BiquadFilter::peaking(85.0, 4.0, 1.4, sr),
                BiquadFilter::peaking(750.0, -3.5, 1.1, sr),
            ),
            AmpModelId::UkAc30TopBoost => (
                BiquadFilter::peaking(4200.0, 5.0, 1.2, sr),
                BiquadFilter::low_shelf(150.0, -5.0, sr),
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
            bass_filter: BiquadFilter::default(),
            mid_filter: BiquadFilter::default(),
            treb_filter: BiquadFilter::default(),
            depth_filter: BiquadFilter::default(),
            presence_filter: BiquadFilter::default(),
            voicing_filter1: v1,
            voicing_filter2: v2,
            target_voicing_filter1: BiquadFilter::default(),
            target_voicing_filter2: BiquadFilter::default(),
            adaa_pre_prev: 0.0,
            adaa_power_prev: 0.0,
            interp_hist: [0.0; 8],
            aa_filters: [BiquadFilter::default(); 14],
            ai_filters: [BiquadFilter::default(); 4],
            target_model_id: None,
            model_crossfade: 0.0,
            crossfade_step: 1.0 / (48000.0 * 0.020),
            oversampling_factor: 8,
            sag_state: 0.0,
            xformer_lf_state: 0.0,
            speaker_res_filter: BiquadFilter::default(),
        };
        s.update_aa_filters();
        s
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
            let sr = self.sample_rate as f64 * 8.0;
            let (v1, v2) = Self::create_voicing_filters(model, sr);
            self.target_voicing_filter1 = v1;
            self.target_voicing_filter2 = v2;
            self.target_model_id = Some(model);
            self.model_crossfade = 0.0;
            self.crossfade_step = 1.0 / (self.sample_rate * 0.020).max(1.0);
            self.mod_settings = nominal_mod_settings_for_model(model);
        }
    }

    pub fn set_oversampling_factor(&mut self, factor: usize) {
        self.oversampling_factor = match factor {
            2 => 2,
            4 => 4,
            _ => 8,
        };
    }

    pub fn algorithmic_latency_samples(&self) -> u32 {
        match self.oversampling_factor {
            2 => 4,
            4 => 8,
            _ => 16,
        }
    }

    fn update_aa_filters(&mut self) {
        let fs_os = self.sample_rate as f64 * 8.0;
        let fc = 15000.0f64.min(self.sample_rate as f64 * 0.35);
        self.aa_filters[0] = BiquadFilter::lowpass(fc, 0.707, fs_os);
        self.aa_filters[1] = BiquadFilter::lowpass(fc, 0.707, fs_os);
        self.aa_filters[2] = BiquadFilter::notch(25000.0, 1.2, fs_os);
        self.aa_filters[3] = BiquadFilter::notch(25000.0, 1.2, fs_os);
        self.aa_filters[4] = BiquadFilter::notch(35000.0, 1.5, fs_os);
        self.aa_filters[5] = BiquadFilter::notch(35000.0, 1.5, fs_os);
        self.aa_filters[6] = BiquadFilter::notch(45000.0, 1.8, fs_os);
        self.aa_filters[7] = BiquadFilter::notch(55000.0, 1.8, fs_os);
        self.aa_filters[8] = BiquadFilter::notch(85000.0, 2.0, fs_os);
        self.aa_filters[9] = BiquadFilter::notch(95000.0, 2.0, fs_os);
        self.aa_filters[10] = BiquadFilter::notch(105000.0, 2.0, fs_os);
        self.aa_filters[11] = BiquadFilter::notch(115000.0, 2.0, fs_os);
        self.aa_filters[12] = BiquadFilter::notch(45000.0, 1.8, fs_os);
        self.aa_filters[13] = BiquadFilter::notch(95000.0, 2.0, fs_os);

        let fc_ai = 20000.0f64.min(self.sample_rate as f64 * 0.42);
        for i in 0..4 {
            self.ai_filters[i] = BiquadFilter::lowpass(fc_ai, 0.707, fs_os);
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.update_aa_filters();
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

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());

        let sr = self.sample_rate as f64 * 8.0;

        // Tone Stack Biquad Filters (Spec 016, CA-1 & CA-2)
        let bass_db = (self.panel.bass as f64 - 5.0) * 2.2;
        let mid_db = (self.panel.middle as f64 - 5.0) * 1.8;
        let treb_db = (self.panel.treble as f64 - 5.0) * 2.2;

        self.bass_filter.update_coeffs(BiquadFilter::low_shelf(120.0, bass_db, sr));
        self.mid_filter.update_coeffs(BiquadFilter::peaking(650.0, mid_db, 1.1, sr));
        self.treb_filter.update_coeffs(BiquadFilter::high_shelf(3500.0, treb_db, sr));

        // Feedback loop filters: Depth (65-130 Hz) & Presence (>2 kHz) (Spec 016, CA-7)
        let depth_db = (self.panel.depth as f64 / 10.0) * 6.5;
        let pres_db = (self.panel.presence as f64 / 10.0) * 6.5;
        self.depth_filter.update_coeffs(BiquadFilter::peaking(85.0, depth_db, 2.5, sr));
        self.presence_filter.update_coeffs(BiquadFilter::peaking(4200.0, pres_db, 2.2, sr));

        // Voicing por modelo de amp atualizado sem resetar estado (com suporte a override de tonestack)
        let (v1_cur, v2_cur) = if self.mod_settings.tonestack != ToneStackTopology::Model {
            match self.mod_settings.tonestack {
                ToneStackTopology::FenderTmbMidScoop => (
                    BiquadFilter::peaking(450.0, -4.5, 1.0, sr),
                    BiquadFilter::high_shelf(5500.0, 3.0, sr),
                ),
                ToneStackTopology::MarshallCathodeFollower => (
                    BiquadFilter::peaking(1600.0, 3.5, 1.2, sr),
                    BiquadFilter::low_shelf(120.0, -2.5, sr),
                ),
                ToneStackTopology::VoxTopBoost => (
                    BiquadFilter::peaking(2800.0, 3.5, 1.2, sr),
                    BiquadFilter::low_shelf(110.0, -4.0, sr),
                ),
                ToneStackTopology::PassiveJamesBaxandall => (
                    BiquadFilter::low_shelf(100.0, 2.0, sr),
                    BiquadFilter::high_shelf(4000.0, 2.0, sr),
                ),
                ToneStackTopology::MesaMarkPreGain => (
                    BiquadFilter::peaking(750.0, -3.5, 1.1, sr),
                    BiquadFilter::peaking(85.0, 4.0, 1.4, sr),
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

        let lead_mult = match self.panel.channel_toggle {
            AmpChannelVoicing::Norm => 1.0,
            AmpChannelVoicing::HighLead => 2.5,
        };

        // Bright switch physical circuit
        let bright_effective_boost = if self.panel.bright_switch {
            let pot_ratio = (10.0 - self.panel.input_gain) / 10.0;
            pot_ratio * 0.95
        } else {
            0.0
        };

        let calc_power_drive = |model: AmpModelId, gain: f32, master: f32| -> f64 {
            match model {
                AmpModelId::BritPlexi1959 => {
                    let master_pot = master as f64 / 10.0;
                    1.0 + master_pot.powi(3) * 35.0 + (gain as f64 / 10.0) * 1.5
                }
                AmpModelId::UsTwinBlackface65 => {
                    1.0 + (gain as f64 / 10.0) * 4.5
                }
                _ => {
                    let master_pot = master as f64 / 10.0;
                    1.0 + master_pot * 4.5
                }
            }
        };

        let cur_drive = calc_power_drive(self.model_id, self.panel.input_gain, self.panel.master_volume);
        let target_drive = if let Some(target) = self.target_model_id {
            calc_power_drive(target, self.panel.input_gain, self.panel.master_volume)
        } else {
            cur_drive
        };
        let alpha_f64 = self.model_crossfade as f64;
        let power_saturation_drive = (1.0 - alpha_f64) * cur_drive + alpha_f64 * target_drive;

        let tube_gain = match self.mod_settings.preamp_tube {
            PreampTube::Ax7A => 1.0f64,
            PreampTube::At7 => 0.60f64,
            PreampTube::Ay7 => 0.45f64,
            PreampTube::Ef86 => 1.15f64,
        };
        let v1_headroom = (self.mod_settings.preamp_plate_v as f64 / 250.0).max(0.1);
        let asym = self.mod_settings.triode_asym as f64;
        let bias_deficit = ((28.0 - self.mod_settings.bias_ma as f64) / 8.0).clamp(0.0, 1.0);
        let crossover_depth = bias_deficit * 0.70;
        let crossover_width = 0.025;

        // Speaker resonance peaking filter (NFB damping at 98 Hz)
        let res_boost_db = ((10.0 - self.mod_settings.nfb as f64) / 10.0) * 4.5;
        self.speaker_res_filter.update_coeffs(BiquadFilter::peaking(98.0, res_boost_db, 2.0, sr));
        let xf_val = self.mod_settings.xformer_sat as f64;
        let sag_amt = self.mod_settings.sag as f64;
        let ri = self.mod_settings.rectifier.internal_resistance();
        let tau_att = self.mod_settings.rectifier.attack_time_sec();
        let tau_rel = self.mod_settings.rectifier.release_time_sec();

        let out_lin = 10.0f64.powf(self.panel.output_level_db as f64 / 20.0);

        for i in 0..input.len() {
            let mut s_in = input[i] as f64;

            // 1. Input Bright Cap Filter at base rate (Spec 003 CA-3)
            if bright_effective_boost > 0.001 {
                let high_pass = s_in - self.bright_state;
                self.bright_state = 0.85 * self.bright_state + 0.15 * s_in;
                s_in += high_pass * (bright_effective_boost as f64 * 2.0);
            } else {
                self.bright_state = 0.85 * self.bright_state + 0.15 * s_in;
            }

            for k in 0..7 {
                self.interp_hist[k] = self.interp_hist[k + 1];
            }
            self.interp_hist[7] = s_in;

            let mut last_sample = 0.0f64;

            for step in 0..8 {
                let weights = &SINC_WEIGHTS[step];
                let mut s = 0.0f64;
                for k in 0..8 {
                    s += self.interp_hist[k] * weights[k];
                }
                for f in &mut self.ai_filters {
                    s = f.process(s);
                }

                // 2. Preamp Triode V1 Stage with ADAA, Tube Gain, Plate Voltage & Asymmetry
                let total_pre_gain = pre_gain_lin * od_gain_lin * lead_mult * tube_gain;
                let pre_in_raw = (s * total_pre_gain) / v1_headroom;
                let pre_in = if asym > 1e-4 {
                    if pre_in_raw >= 0.0 {
                        pre_in_raw * (1.0 + 0.5 * asym)
                    } else {
                        pre_in_raw * (1.0 - 0.5 * asym)
                    }
                } else {
                    pre_in_raw
                };
                let dx = pre_in - self.adaa_pre_prev;
                let v1_sat = if dx.abs() < 1e-5 {
                    pre_in / (1.0 + pre_in.abs())
                } else {
                    let ax = pre_in.abs();
                    let a_prev = self.adaa_pre_prev.abs();
                    let f1 = ax - (1.0 + ax).ln();
                    let f0 = a_prev - (1.0 + a_prev).ln();
                    (f1 - f0) / dx
                };
                self.adaa_pre_prev = pre_in;
                let v1_out = v1_sat * v1_headroom;

                // 3. Tone Stack Passivo de 3 Bandas + Voicing do Modelo
                let mut v_base = v1_out;
                v_base = self.bass_filter.process(v_base);
                v_base = self.mid_filter.process(v_base);
                v_base = self.treb_filter.process(v_base);

                let v = if self.target_model_id.is_some() {
                    let mut va = v_base;
                    va = self.voicing_filter1.process(va);
                    va = self.voicing_filter2.process(va);

                    let mut vb = v_base;
                    vb = self.target_voicing_filter1.process(vb);
                    vb = self.target_voicing_filter2.process(vb);

                    let alpha = self.model_crossfade as f64;
                    (1.0 - alpha) * va + alpha * vb
                } else {
                    let mut va = v_base;
                    va = self.voicing_filter1.process(va);
                    va = self.voicing_filter2.process(va);
                    va
                };

                // 4. Power Amp / Phase Inverter Stage with Bias Crossover Distortion & ADAA tanh
                let power_in_raw = v * power_saturation_drive;
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
                    let f1 = if power_in.abs() > 20.0 { power_in.abs() - std::f64::consts::LN_2 } else { power_in.cosh().ln() };
                    let f0 = if self.adaa_power_prev.abs() > 20.0 { self.adaa_power_prev.abs() - std::f64::consts::LN_2 } else { self.adaa_power_prev.cosh().ln() };
                    (f1 - f0) / dp
                };
                self.adaa_power_prev = power_in;

                // NFB Linearization on power stage distortion
                let nfb_linearization = 1.0 / (1.0 + (self.mod_settings.nfb as f64 / 10.0) * 3.0);
                let power_stage_dist = power_sat_raw - power_in_raw;
                let power_sat = power_in_raw + power_stage_dist * nfb_linearization;

                // 5. Speaker Resonance (NFB) + Negative Feedback Loop Filters: Depth & Presence
                let mut power_out = self.speaker_res_filter.process(power_sat * 0.38);
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
                let b_plus_factor = (1.0 - self.sag_state).max(0.15);

                // Safe output limiter inside 8x oversampled loop (Spec 016, CA-9 & CA-11)
                let lim_in = power_out * b_plus_factor;
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

                if step == 7 {
                    last_sample = lim_out;
                }
            }

            if self.target_model_id.is_some() {
                self.model_crossfade += self.crossfade_step;
                if self.model_crossfade >= 1.0 {
                    let target = self.target_model_id.take().unwrap();
                    self.model_id = target;
                    self.voicing_filter1 = self.target_voicing_filter1;
                    self.voicing_filter2 = self.target_voicing_filter2;
                    self.model_crossfade = 0.0;
                }
            }

            let out_sample = (last_sample.clamp(-0.999, 0.999) * out_lin).clamp(-1.0, 1.0);
            output[i] = out_sample as f32;
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
