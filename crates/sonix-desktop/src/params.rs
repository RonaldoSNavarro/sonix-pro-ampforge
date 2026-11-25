//! Tabela única de parâmetros reais (AUD-09 / Spec 014).
//! Fonte única de verdade para validação, tradução DSP (`DspParamUpdate`),
//! leitura/escrita em `AppState`, presets e geração de `params.generated.ts`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ampforge_dsp::amp::{find_model_by_name, Ac30VibTremMode, AmpChannelMode, AmpInputJack, AmpModelId};
use ampforge_dsp::cab::{MicModel, SpeakerCabModel};
use ampforge_dsp::fx::drive::DriveModel;
use ampforge_dsp::fx::eq::EqMode;
use ampforge_dsp::fx::modulation::ModModel;
use ampforge_dsp::fx::pitch::{PitchKey, PitchModel, ScaleType};
use ampforge_dsp::fx::wah::WahMode;
use ampforge_dsp::fx::NoiseGateMode;
use ampforge_dsp::looper::LooperPosition;
use ampforge_dsp::modshop::{PowerTube, PreampTube, RectifierType, ToneStackTopology};
use ampforge_dsp::tuner::TunerOutputMode;
use serde_json::Value;

use crate::audio::{
    ChainSplitMode, CompiledDagTopology, DspParamUpdate, LockFreeParamQueue, LooperActionCmd,
    RealtimeDspChain,
};

pub const KEY_SCENE_ACTIVE: &str = "scene.active";
pub const KEY_AMP_1_MODEL: &str = "amp_1.model";
pub const STOMP_PARAM_KEYS: [&str; 8] = [
    "gate.enabled",
    "drive.enabled",
    "amp_1.enabled",
    "cab_1.enabled",
    "eq_1.enabled",
    "comp_2.enabled",
    "dly_1.enabled",
    "rev_1.enabled",
];

const KEY_LOOPER_ACTION: &str = "looper.action";
const KEY_AMP_1_CHANNEL_MODE: &str = "amp_1.channel_mode";
const KEY_CAB_1_SPEAKER: &str = "cab_1.speaker";
const KEY_CAB_1_IR_ENABLED: &str = "cab_1.ir_enabled";
const KEY_EQ_1_BANDS_DB: &str = "eq_1.bands_db";
const PREFIX_CAB_1_MIC: &str = "cab_1.mic";
const PREFIX_EQ_1_BAND: &str = "eq_1.band";
const MODEL_DUAL_RECT: &str = "dual_rect";
const MODEL_JCM800: &str = "jcm800_2203";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamKind {
    Number { min: f32, max: f32, step: f32 },
    Bool,
    Enum(&'static [&'static str]),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamDefault {
    Number(f32),
    Bool(bool),
    Enum(&'static str),
}

pub struct ParamDesc {
    pub key: &'static str,
    pub aliases: &'static [&'static str],
    pub kind: ParamKind,
    pub default: ParamDefault,
    pub models: Option<&'static [AmpModelId]>,
    pub to_dsp: fn(&Value) -> Option<DspParamUpdate>,
    pub read_state: fn(&AppState) -> Value,
    pub write_state: fn(&mut AppState, &Value),
}

// ── Defaults compartilhados para AppState & Serde ────────────────────────────────

fn default_true() -> bool { true }
fn default_one() -> f32 { 1.0 }
fn default_four() -> f32 { 4.0 }
fn default_five() -> f32 { 5.0 }
fn default_seven() -> f32 { 7.0 }
fn default_half() -> f32 { 0.5 }
fn default_wah_mode() -> String { "crybaby_gcb95".to_string() }
fn default_forty() -> f32 { 40.0 }
fn default_fifty() -> f32 { 50.0 }
fn default_crossover_hz() -> f32 { 650.0 }
fn default_merge_level() -> f32 { -6.0 }
fn default_decay() -> f32 { 2.5 }
fn default_predelay() -> f32 { 20.0 }
fn default_split_mode() -> String { "ab".to_string() }
fn default_comp_ratio() -> String { "4".to_string() }
fn default_dly_heads() -> String { "1".to_string() }
fn default_twin() -> String { "twin_ab763".to_string() }
fn default_alnico() -> String { "alnico_blue_2x12".to_string() }
fn default_preset_active() -> String { "brit800".to_string() }
fn default_preset_name() -> String { "Brit 800 Crunch".to_string() }
fn default_scene_active() -> String { "A".to_string() }
fn default_mute() -> String { "mute".to_string() }
fn default_gate_mode() -> String { "natural".to_string() }
fn default_gate_hold() -> f32 { 35.0 }
fn default_gate_release() -> f32 { 80.0 }
fn default_gate_range() -> f32 { -70.0 }
fn default_gate_key_hpf() -> f32 { 80.0 }
fn default_gate_key_lpf() -> f32 { 6000.0 }
fn default_drive_model() -> String { "ts808".to_string() }
fn default_eq_mode() -> String { "eight_band".to_string() }
fn default_dly_mode() -> String { "1".to_string() }
fn default_mod_2_model() -> String { "AnalogBbdChorus".to_string() }
fn default_zero_str() -> String { "0".to_string() }
fn default_a4_hz() -> f32 { 440.0 }
fn default_looper_pos() -> String { "post".to_string() }
fn default_looper_action() -> String { "stop".to_string() }
fn default_cab_resonance() -> f32 { 90.0 }
fn default_cab_hicut() -> f32 { 9.0 }
fn default_exp_1() -> f32 { 62.0 }
fn default_hundred() -> f32 { 100.0 }
fn default_pitch_model() -> String { "whammy".to_string() }
fn default_pitch_key() -> String { "C".to_string() }
fn default_pitch_scale() -> String { "major".to_string() }
fn default_input_mode() -> String { "instrument".to_string() }
fn default_input_channel() -> String { "left".to_string() }
fn default_amp_jack() -> String { "high".to_string() }
fn default_amp_channel_mode() -> String { "modern".to_string() }
fn default_amp_impedance() -> String { "16".to_string() }
fn default_vib_trem_mode() -> String { "vibrato".to_string() }
fn default_curve_slope() -> f32 { 0.5 }
fn default_footswitch_mode() -> String { "HYBRID_STOMP_SCENE".to_string() }
fn default_split_dyn_thresh() -> f32 { -30.0 }
fn default_split_dyn_attack() -> f32 { 5.0 }
fn default_split_dyn_release() -> f32 { 80.0 }

const EQ_DEFAULT_BAND_FREQS_HZ: [f32; 10] = [
    31.25, 62.5, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];

fn default_eq_band_freqs() -> [f32; 10] {
    EQ_DEFAULT_BAND_FREQS_HZ
}

fn default_eq_band_qs() -> [f32; 10] {
    [1.0; 10]
}

fn default_modifiers() -> Vec<ModifierBindingDto> {
    crate::midi::DesktopMidiHost::default_midi_modifiers()
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect()
}

pub fn default_dag_topology() -> CompiledDagTopology {
    CompiledDagTopology::default_layout(1)
}

pub fn default_scenes() -> Vec<sonix_core::persistence::SceneDto> {
    ["A", "B", "C", "D", "E", "F", "G", "H"]
        .iter()
        .enumerate()
        .map(|(index, label)| sonix_core::persistence::SceneDto {
            index,
            id: format!("scene_{}", index),
            name: format!("Scene {}", label),
            color: "#ffc400".to_string(),
        })
        .collect()
}

pub fn default_grid() -> sonix_core::persistence::GridDto {
    sonix_core::persistence::GridDto {
        rows: 4,
        cols: 12,
        blocks: Vec::new(),
    }
}

/// Estado persistente e observável da cadeia e interface (tabela única de parâmetros da Spec 014).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppState {
    #[serde(rename = "amp_1.gain")]
    pub amp_gain: f32,
    #[serde(rename = "amp_1.bass")]
    pub amp_bass: f32,
    #[serde(rename = "amp_1.middle")]
    pub amp_middle: f32,
    #[serde(rename = "amp_1.treble")]
    pub amp_treble: f32,
    #[serde(rename = "amp_1.presence")]
    pub amp_presence: f32,
    #[serde(rename = "amp_1.master")]
    pub amp_master: f32,
    #[serde(rename = "amp_1.volume_2")]
    pub amp_volume_2: f32,
    #[serde(rename = "amp_1.cut")]
    pub amp_cut: f32,
    #[serde(rename = "amp_1.model")]
    pub amp_model: String,
    #[serde(rename = "engine.oversampling")]
    pub engine_oversampling: String,
    #[serde(rename = "input.boost")]
    pub input_boost: f32,
    #[serde(rename = "gate.threshold_db")]
    pub gate_threshold_db: f32,
    #[serde(rename = "input.source")]
    pub input_source: String,
    #[serde(default = "default_input_mode", rename = "input.mode")]
    pub input_mode: String,
    #[serde(default = "default_input_channel", rename = "input.channel")]
    pub input_channel: String,
    #[serde(rename = "amp_1.bright")]
    pub amp_bright: bool,
    #[serde(default = "default_true", rename = "amp_1.power")]
    pub amp_power: bool,
    #[serde(default, rename = "amp_1.standby")]
    pub amp_standby: bool,
    #[serde(default = "default_amp_jack", rename = "amp_1.input_jack")]
    pub amp_input_jack: String,
    #[serde(default, rename = "amp_1.jumpered")]
    pub amp_jumpered: bool,
    #[serde(default = "default_amp_channel_mode", rename = "amp_1.channel_mode")]
    pub amp_channel_mode: String,
    #[serde(default, rename = "amp_1.spongy")]
    pub amp_spongy: bool,
    #[serde(default, rename = "amp_1.reverb")]
    pub amp_reverb: f32,
    #[serde(default = "default_four", rename = "amp_1.vibrato_speed")]
    pub amp_vibrato_speed: f32,
    #[serde(default, rename = "amp_1.vibrato_depth")]
    pub amp_vibrato_depth: f32,
    #[serde(default = "default_vib_trem_mode", rename = "amp_1.vib_trem_mode")]
    pub amp_vib_trem_mode: String,
    #[serde(default = "default_amp_impedance", rename = "amp_1.impedance_ohms")]
    pub amp_impedance_ohms: String,
    #[serde(rename = "drive.enabled")]
    pub drive_enabled: bool,
    #[serde(default = "default_drive_model", rename = "drive.model")]
    pub drive_model: String,
    #[serde(rename = "drive.drive")]
    pub drive_drive: f32,
    #[serde(rename = "drive.tone")]
    pub drive_tone: f32,
    #[serde(rename = "drive.level")]
    pub drive_level: f32,
    #[serde(default = "default_five", rename = "drive.bias")]
    pub drive_bias: f32,
    #[serde(rename = "gate.enabled")]
    pub gate_enabled: bool,
    #[serde(rename = "gate.mode", default = "default_gate_mode")]
    pub gate_mode: String,
    #[serde(rename = "gate.hysteresis_db", default = "default_four")]
    pub gate_hysteresis_db: f32,
    #[serde(rename = "gate.attack_ms", default = "default_one")]
    pub gate_attack_ms: f32,
    #[serde(rename = "gate.hold_ms", default = "default_gate_hold")]
    pub gate_hold_ms: f32,
    #[serde(rename = "gate.release_ms", default = "default_gate_release")]
    pub gate_release_ms: f32,
    #[serde(rename = "gate.range_db", default = "default_gate_range")]
    pub gate_range_db: f32,
    #[serde(rename = "gate.key_hpf_hz", default = "default_gate_key_hpf")]
    pub gate_key_hpf_hz: f32,
    #[serde(rename = "gate.key_lpf_hz", default = "default_gate_key_lpf")]
    pub gate_key_lpf_hz: f32,
    #[serde(rename = "amp_1.enabled")]
    pub amp_enabled: bool,
    #[serde(rename = "cab_1.enabled")]
    pub cab_enabled: bool,
    #[serde(rename = "amp_1.preamp_tube")]
    pub preamp_tube: String,
    #[serde(rename = "amp_1.power_tube")]
    pub power_tube: String,
    #[serde(rename = "amp_1.bias_ma")]
    pub bias_ma: f32,
    #[serde(rename = "amp_1.sag")]
    pub sag: f32,
    #[serde(rename = "amp_1.rectifier")]
    pub rectifier: String,
    #[serde(rename = "amp_1.nfb")]
    pub nfb: f32,
    #[serde(rename = "amp_1.xformer_sat")]
    pub xformer_sat: f32,
    #[serde(rename = "amp_1.preamp_plate_v")]
    pub preamp_plate_v: f32,
    #[serde(rename = "amp_1.triode_asym")]
    pub triode_asym: f32,
    #[serde(rename = "amp_1.tonestack")]
    pub tonestack: String,
    pub is_modded: bool,
    #[serde(rename = "cab_1.speaker")]
    pub cab_speaker: String,
    #[serde(rename = "cab_1.resonance_hz")]
    pub cab_resonance_hz: f32,
    #[serde(rename = "cab_1.hicut_khz")]
    pub cab_hicut_khz: f32,
    #[serde(rename = "cab_1.ir_enabled")]
    pub cab_ir_enabled: bool,
    #[serde(rename = "cab_1.mic1.model")]
    pub cab_mic1_model: String,
    #[serde(rename = "cab_1.mic1.position")]
    pub cab_mic1_position: f32,
    #[serde(rename = "cab_1.mic1.distance_cm")]
    pub cab_mic1_distance_cm: f32,
    #[serde(rename = "cab_1.mic1.level_db")]
    pub cab_mic1_level_db: f32,
    #[serde(rename = "cab_1.mic1.enabled")]
    pub cab_mic1_enabled: bool,
    #[serde(default = "default_zero_str", rename = "cab_1.mic1.angle_deg")]
    pub cab_mic1_angle_deg: String,
    #[serde(default, rename = "cab_1.mic1.phase_invert")]
    pub cab_mic1_phase_invert: bool,
    #[serde(rename = "cab_1.mic2.model")]
    pub cab_mic2_model: String,
    #[serde(rename = "cab_1.mic2.position")]
    pub cab_mic2_position: f32,
    #[serde(rename = "cab_1.mic2.distance_cm")]
    pub cab_mic2_distance_cm: f32,
    #[serde(rename = "cab_1.mic2.level_db")]
    pub cab_mic2_level_db: f32,
    #[serde(rename = "cab_1.mic2.enabled")]
    pub cab_mic2_enabled: bool,
    #[serde(default = "default_zero_str", rename = "cab_1.mic2.angle_deg")]
    pub cab_mic2_angle_deg: String,
    #[serde(default, rename = "cab_1.mic2.phase_invert")]
    pub cab_mic2_phase_invert: bool,
    #[serde(default, rename = "split.enabled")]
    pub split_enabled: bool,
    #[serde(default = "default_split_mode", rename = "split.mode")]
    pub split_mode: String,
    #[serde(default = "default_crossover_hz", rename = "split.crossover_hz")]
    pub split_crossover_hz: f32,
    #[serde(default = "default_half", rename = "split.balance")]
    pub split_balance: f32,
    #[serde(default = "default_split_dyn_thresh", rename = "split.dyn_threshold_db")]
    pub split_dyn_threshold_db: f32,
    #[serde(default = "default_split_dyn_attack", rename = "split.dyn_attack_ms")]
    pub split_dyn_attack_ms: f32,
    #[serde(default = "default_split_dyn_release", rename = "split.dyn_release_ms")]
    pub split_dyn_release_ms: f32,
    #[serde(default = "default_merge_level", rename = "merge.row_1_level_db")]
    pub merge_row_1_level_db: f32,
    #[serde(default, rename = "merge.row_1_phase_invert")]
    pub merge_row_1_phase_invert: bool,
    #[serde(default, rename = "comp_2.enabled")]
    pub comp_2_enabled: bool,
    #[serde(default = "default_five", rename = "comp_2.input")]
    pub comp_2_input: f32,
    #[serde(default = "default_five", rename = "comp_2.output")]
    pub comp_2_output: f32,
    #[serde(default = "default_four", rename = "comp_2.attack")]
    pub comp_2_attack: f32,
    #[serde(default = "default_four", rename = "comp_2.release")]
    pub comp_2_release: f32,
    #[serde(default = "default_comp_ratio", rename = "comp_2.ratio")]
    pub comp_2_ratio: String,
    #[serde(default, rename = "eq_1.enabled")]
    pub eq_1_enabled: bool,
    #[serde(default = "default_eq_mode", rename = "eq_1.mode")]
    pub eq_1_mode: String,
    #[serde(default, rename = "eq_1.gain_db")]
    pub eq_1_gain_db: f32,
    #[serde(default, rename = "eq_1.volume_db")]
    pub eq_1_volume_db: f32,
    #[serde(default, rename = "eq_1.bands_db")]
    pub eq_1_bands_db: [f32; 10],
    #[serde(skip, default = "default_eq_band_freqs")]
    pub eq_1_band_freqs_hz: [f32; 10],
    #[serde(skip, default = "default_eq_band_qs")]
    pub eq_1_band_qs: [f32; 10],
    #[serde(default, rename = "mod_2.enabled")]
    pub mod_2_enabled: bool,
    #[serde(default = "default_mod_2_model", rename = "mod_2.model")]
    pub mod_2_model: String,
    #[serde(default = "default_one", rename = "mod_2.rate_hz")]
    pub mod_2_rate_hz: f32,
    #[serde(default = "default_five", rename = "mod_2.depth")]
    pub mod_2_depth: f32,
    #[serde(default = "default_fifty", rename = "mod_2.mix")]
    pub mod_2_mix: f32,
    #[serde(default, rename = "pitch_1.enabled")]
    pub pitch_1_enabled: bool,
    #[serde(default = "default_pitch_model", rename = "pitch_1.model")]
    pub pitch_1_model: String,
    #[serde(default, rename = "pitch_1.semitones")]
    pub pitch_1_semitones: f32,
    #[serde(default = "default_pitch_key", rename = "pitch_1.key")]
    pub pitch_1_key: String,
    #[serde(default = "default_pitch_scale", rename = "pitch_1.scale")]
    pub pitch_1_scale: String,
    #[serde(default = "default_hundred", rename = "pitch_1.mix")]
    pub pitch_1_mix: f32,
    #[serde(default, rename = "dly_1.enabled")]
    pub dly_1_enabled: bool,
    #[serde(default = "default_dly_mode", rename = "dly_1.mode")]
    pub dly_1_mode: String,
    #[serde(default = "default_dly_heads", rename = "dly_1.heads")]
    pub dly_1_heads: String,
    #[serde(default = "default_five", rename = "dly_1.repeat_rate")]
    pub dly_1_repeat_rate: f32,
    #[serde(default = "default_five", rename = "dly_1.intensity")]
    pub dly_1_intensity: f32,
    #[serde(default = "default_five", rename = "dly_1.echo_volume")]
    pub dly_1_echo_volume: f32,
    #[serde(default = "default_five", rename = "dly_1.reverb_volume")]
    pub dly_1_reverb_volume: f32,
    #[serde(default = "default_five", rename = "dly_1.bass")]
    pub dly_1_bass: f32,
    #[serde(default = "default_five", rename = "dly_1.treble")]
    pub dly_1_treble: f32,
    #[serde(default = "default_true", rename = "dly_1.spillover")]
    pub dly_1_spillover: bool,
    #[serde(default, rename = "rev_1.enabled")]
    pub rev_1_enabled: bool,
    #[serde(default = "default_decay", rename = "rev_1.decay_s")]
    pub rev_1_decay_s: f32,
    #[serde(default = "default_predelay", rename = "rev_1.predelay_ms")]
    pub rev_1_predelay_ms: f32,
    #[serde(default = "default_five", rename = "rev_1.damping")]
    pub rev_1_damping: f32,
    #[serde(default = "default_forty", rename = "rev_1.mix")]
    pub rev_1_mix: f32,
    #[serde(default = "default_true", rename = "rev_1.spillover")]
    pub rev_1_spillover: bool,
    #[serde(default, rename = "amp_2.enabled")]
    pub amp_2_enabled: bool,
    #[serde(default = "default_twin", rename = "amp_2.model")]
    pub amp_2_model: String,
    #[serde(default = "default_five", rename = "amp_2.gain")]
    pub amp_2_gain: f32,
    #[serde(default = "default_five", rename = "amp_2.bass")]
    pub amp_2_bass: f32,
    #[serde(default = "default_five", rename = "amp_2.middle")]
    pub amp_2_middle: f32,
    #[serde(default = "default_five", rename = "amp_2.treble")]
    pub amp_2_treble: f32,
    #[serde(default = "default_seven", rename = "amp_2.master")]
    pub amp_2_master: f32,
    #[serde(default, rename = "wah_1.enabled")]
    pub wah_1_enabled: bool,
    #[serde(default = "default_half", rename = "wah_1.position")]
    pub wah_1_position: f32,
    #[serde(default = "default_five", rename = "wah_1.q")]
    pub wah_1_q: f32,
    #[serde(default = "default_wah_mode", rename = "wah_1.mode")]
    pub wah_1_mode: String,
    #[serde(default, rename = "cab_2.enabled")]
    pub cab_2_enabled: bool,
    #[serde(default = "default_alnico", rename = "cab_2.speaker")]
    pub cab_2_speaker: String,
    #[serde(default = "default_cab_resonance", rename = "cab_2.resonance_hz")]
    pub cab_2_resonance_hz: f32,
    #[serde(default = "default_cab_hicut", rename = "cab_2.hicut_khz")]
    pub cab_2_hicut_khz: f32,
    #[serde(default, rename = "master.level_db")]
    pub master_level_db: f32,
    #[serde(default, rename = "looper.reverse")]
    pub looper_reverse: bool,
    #[serde(default, rename = "looper.half_speed")]
    pub looper_half_speed: bool,
    #[serde(default = "default_exp_1", rename = "palco.exp_1")]
    pub palco_exp_1: f32,
    #[serde(rename = "preset.active", default = "default_preset_active")]
    pub active_preset_id: String,
    #[serde(rename = "preset.name", default = "default_preset_name")]
    pub active_preset_name: String,
    #[serde(rename = "scene.active", default = "default_scene_active")]
    pub scene_active: String,
    #[serde(default, rename = "tuner.enabled")]
    pub tuner_enabled: bool,
    #[serde(rename = "tuner.output", default = "default_mute")]
    pub tuner_output: String,
    #[serde(default = "default_a4_hz", rename = "tuner.ref_hz")]
    pub tuner_ref_hz: f32,
    #[serde(default = "default_looper_pos", rename = "looper.position")]
    pub looper_position: String,
    #[serde(default = "default_looper_action", rename = "looper.action")]
    pub looper_action: String,
    #[serde(default = "default_scenes")]
    pub scenes: Vec<sonix_core::persistence::SceneDto>,
    #[serde(default = "default_grid")]
    pub grid: sonix_core::persistence::GridDto,
    #[serde(default)]
    pub channels: HashMap<String, HashMap<String, HashMap<String, Value>>>,
    #[serde(skip, default = "default_dag_topology")]
    pub dag_topology: CompiledDagTopology,
    #[serde(default = "default_footswitch_mode", rename = "footswitch.mode")]
    pub footswitch_mode: String,
    #[serde(default = "default_modifiers")]
    pub modifiers: Vec<ModifierBindingDto>,
    #[serde(skip)]
    pub setlist_active_index: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModifierBindingDto {
    pub id: String,
    pub target_param: String,
    pub source: String,
    pub min_val: f32,
    pub max_val: f32,
    #[serde(default = "default_curve_slope")]
    pub curve_slope: f32,
    #[serde(default)]
    pub damping_ms: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SetlistDto {
    pub id: String,
    pub name: String,
    pub presets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresetApplyError {
    Validation(String),
    QueueFull,
}

/// Retorna os valores nominais de chassi para cada modelo de amplificador.
pub fn nominal_chassis_for_model(
    model: &str,
) -> (
    &'static str,
    &'static str,
    f32,
    f32,
    &'static str,
    f32,
    f32,
    f32,
    f32,
    &'static str,
) {
    match model {
        "jcm800_2203" => ("12ax7", "el34", 38.0, 3.0, "diode", 5.0, 4.0, 280.0, 0.20, "model"),
        "plexi_1959" => ("12ax7", "el34", 36.0, 4.0, "diode", 6.0, 5.0, 250.0, 0.22, "model"),
        "twin_ab763" => ("12ax7", "6l6gc", 35.0, 2.0, "diode", 8.0, 2.0, 310.0, 0.15, "model"),
        "dual_rect" => ("12ax7", "6l6gc", 32.0, 5.0, "5u4gb", 3.0, 3.0, 290.0, 0.28, "model"),
        "ac30_tb" => ("12ax7", "el84_variant", 45.0, 6.0, "gz34", 0.0, 5.0, 240.0, 0.18, "model"),
        _ => ("12ax7", "el34", 38.0, 3.0, "diode", 5.0, 4.0, 280.0, 0.20, "model"),
    }
}

impl AppState {
    /// Verifica se algum parâmetro do chassi difere dos valores nominais do modelo ativo.
    pub fn check_is_modded(&self) -> bool {
        let nom = nominal_chassis_for_model(&self.amp_model);
        self.preamp_tube != nom.0
            || self.power_tube != nom.1
            || (self.bias_ma - nom.2).abs() > 0.01
            || (self.sag - nom.3).abs() > 0.01
            || self.rectifier != nom.4
            || (self.nfb - nom.5).abs() > 0.01
            || (self.xformer_sat - nom.6).abs() > 0.01
            || (self.preamp_plate_v - nom.7).abs() > 0.01
            || (self.triode_asym - nom.8).abs() > 0.01
            || self.tonestack != nom.9
    }

    pub fn apply_nominal_chassis_for_model(&mut self, model: &str) {
        let nom = nominal_chassis_for_model(model);
        self.amp_model = model.to_string();
        self.preamp_tube = nom.0.to_string();
        self.power_tube = nom.1.to_string();
        self.bias_ma = nom.2;
        self.sag = nom.3;
        self.rectifier = nom.4.to_string();
        self.nfb = nom.5;
        self.xformer_sat = nom.6;
        self.preamp_plate_v = nom.7;
        self.triode_asym = nom.8;
        self.tonestack = nom.9.to_string();
        self.is_modded = false;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            amp_gain: 8.0,
            amp_bass: 5.0,
            amp_middle: 5.0,
            amp_treble: 5.0,
            amp_presence: 5.0,
            amp_master: 7.0,
            amp_volume_2: 0.0,
            amp_cut: 5.0,
            amp_model: MODEL_JCM800.to_string(),
            engine_oversampling: "2x".to_string(),
            input_boost: 2.5,
            gate_threshold_db: -65.0,
            input_source: "device".to_string(),
            input_mode: "instrument".to_string(),
            input_channel: "left".to_string(),
            amp_bright: false,
            amp_power: true,
            amp_standby: false,
            amp_input_jack: "high".to_string(),
            amp_jumpered: false,
            amp_channel_mode: "modern".to_string(),
            amp_spongy: false,
            amp_reverb: 0.0,
            amp_vibrato_speed: 4.0,
            amp_vibrato_depth: 0.0,
            amp_vib_trem_mode: default_vib_trem_mode(),
            amp_impedance_ohms: "16".to_string(),
            drive_enabled: false,
            drive_model: "ts808".to_string(),
            drive_drive: 5.0,
            drive_tone: 5.0,
            drive_level: 5.0,
            drive_bias: 5.0,
            gate_enabled: true,
            gate_mode: "natural".to_string(),
            gate_hysteresis_db: 4.0,
            gate_attack_ms: 1.0,
            gate_hold_ms: 35.0,
            gate_release_ms: 80.0,
            gate_range_db: -70.0,
            gate_key_hpf_hz: 80.0,
            gate_key_lpf_hz: 6000.0,
            amp_enabled: true,
            cab_enabled: true,
            preamp_tube: "12ax7".to_string(),
            power_tube: "el34".to_string(),
            bias_ma: 38.0,
            sag: 3.0,
            rectifier: "diode".to_string(),
            nfb: 5.0,
            xformer_sat: 4.0,
            preamp_plate_v: 280.0,
            triode_asym: 0.20,
            tonestack: "model".to_string(),
            is_modded: false,
            cab_speaker: "greenback_4x12".to_string(),
            cab_resonance_hz: 98.0,
            cab_hicut_khz: 8.2,
            cab_ir_enabled: false,
            cab_mic1_model: "sm57".to_string(),
            cab_mic1_position: 0.2,
            cab_mic1_distance_cm: 2.0,
            cab_mic1_level_db: 0.0,
            cab_mic1_enabled: true,
            cab_mic1_angle_deg: "0".to_string(),
            cab_mic1_phase_invert: false,
            cab_mic2_model: "sm57".to_string(),
            cab_mic2_position: 0.5,
            cab_mic2_distance_cm: 5.0,
            cab_mic2_level_db: 0.0,
            cab_mic2_enabled: false,
            cab_mic2_angle_deg: "0".to_string(),
            cab_mic2_phase_invert: false,
            split_enabled: false,
            split_mode: "ab".to_string(),
            split_crossover_hz: 650.0,
            split_balance: 0.5,
            split_dyn_threshold_db: -30.0,
            split_dyn_attack_ms: 5.0,
            split_dyn_release_ms: 80.0,
            merge_row_1_level_db: -6.0,
            merge_row_1_phase_invert: false,
            comp_2_enabled: false,
            comp_2_input: 5.0,
            comp_2_output: 5.0,
            comp_2_attack: 4.0,
            comp_2_release: 4.0,
            comp_2_ratio: "4".to_string(),
            eq_1_enabled: false,
            eq_1_mode: "eight_band".to_string(),
            eq_1_gain_db: 0.0,
            eq_1_volume_db: 0.0,
            eq_1_bands_db: [0.0; 10],
            eq_1_band_freqs_hz: EQ_DEFAULT_BAND_FREQS_HZ,
            eq_1_band_qs: [1.0; 10],
            mod_2_enabled: false,
            mod_2_model: default_mod_2_model(),
            mod_2_rate_hz: 1.0,
            mod_2_depth: 5.0,
            mod_2_mix: 50.0,
            pitch_1_enabled: false,
            pitch_1_model: default_pitch_model(),
            pitch_1_semitones: 0.0,
            pitch_1_key: default_pitch_key(),
            pitch_1_scale: default_pitch_scale(),
            pitch_1_mix: 100.0,
            dly_1_enabled: false,
            dly_1_mode: "1".to_string(),
            dly_1_heads: "1".to_string(),
            dly_1_repeat_rate: 5.0,
            dly_1_intensity: 5.0,
            dly_1_echo_volume: 5.0,
            dly_1_reverb_volume: 5.0,
            dly_1_bass: 5.0,
            dly_1_treble: 5.0,
            dly_1_spillover: true,
            rev_1_enabled: false,
            rev_1_decay_s: 2.5,
            rev_1_predelay_ms: 20.0,
            rev_1_damping: 5.0,
            rev_1_mix: 40.0,
            rev_1_spillover: true,
            amp_2_enabled: false,
            amp_2_model: "twin_ab763".to_string(),
            amp_2_gain: 5.0,
            amp_2_bass: 5.0,
            amp_2_middle: 5.0,
            amp_2_treble: 5.0,
            amp_2_master: 7.0,
            wah_1_enabled: false,
            wah_1_position: 0.5,
            wah_1_q: 5.0,
            wah_1_mode: "crybaby_gcb95".to_string(),
            cab_2_enabled: false,
            cab_2_speaker: "alnico_blue_2x12".to_string(),
            cab_2_resonance_hz: 90.0,
            cab_2_hicut_khz: 9.0,
            master_level_db: 0.0,
            looper_reverse: false,
            looper_half_speed: false,
            palco_exp_1: 62.0,
            active_preset_id: "brit800".to_string(),
            active_preset_name: "Brit 800 Crunch".to_string(),
            scene_active: "A".to_string(),
            tuner_enabled: false,
            tuner_output: "mute".to_string(),
            tuner_ref_hz: 440.0,
            looper_position: "post".to_string(),
            looper_action: "stop".to_string(),
            scenes: default_scenes(),
            grid: default_grid(),
            channels: HashMap::new(),
            dag_topology: default_dag_topology(),
            footswitch_mode: default_footswitch_mode(),
            modifiers: default_modifiers(),
            setlist_active_index: 0,
        }
    }
}

// ── Parsers tipados compartilhados ───────────────────────────────────────────────

fn parse_speaker_model(s: &str) -> Option<SpeakerCabModel> {
    match s {
        "greenback_4x12" | "greenback_4x12_closed" => Some(SpeakerCabModel::Greenback4x12Closed),
        "v30_4x12" | "v30_4x12_closed" => Some(SpeakerCabModel::V304x12Closed),
        "alnico_blue_2x12" | "alnico_blue_2x12_open" => Some(SpeakerCabModel::AlnicoBlue2x12Open),
        "twin_2x12_open" | "twin_2x12" | "jensen_c12n_2x12_open" => Some(SpeakerCabModel::Twin2x12Open),
        _ => None,
    }
}

fn nominal_resonance_for_speaker(s: &str) -> f32 {
    match parse_speaker_model(s) {
        Some(SpeakerCabModel::V304x12Closed) => 108.0,
        Some(SpeakerCabModel::AlnicoBlue2x12Open) => 82.0,
        Some(SpeakerCabModel::Twin2x12Open) => 90.0,
        _ => 98.0,
    }
}

fn parse_eq_mode(s: &str) -> Option<EqMode> {
    match s {
        "eight_band" | "parametric_8" => Some(EqMode::Parametric8Band),
        "graphic_5_boogie" | "boogie_5" => Some(EqMode::Graphic5BandBoogie),
        "mxr_10band" | "graphic_10_mxr" => Some(EqMode::Graphic10BandPedal),
        _ => None,
    }
}

fn parse_looper_action(s: &str) -> Option<LooperActionCmd> {
    match s {
        "record" | "rec" => Some(LooperActionCmd::Record),
        "play" => Some(LooperActionCmd::Play),
        "overdub" => Some(LooperActionCmd::Overdub),
        "stop" => Some(LooperActionCmd::Stop),
        "undo" => Some(LooperActionCmd::Undo),
        "redo" => Some(LooperActionCmd::Redo),
        "reverse" => Some(LooperActionCmd::Reverse),
        "half_speed" => Some(LooperActionCmd::HalfSpeed),
        "clear" => Some(LooperActionCmd::Clear),
        _ => None,
    }
}

fn parse_mic_angle_deg(v: &Value) -> Option<f32> {
    if let Some(s) = v.as_str() {
        match s {
            "0" => Some(0.0),
            "45" => Some(45.0),
            _ => None,
        }
    } else if let Some(n) = v.as_f64() {
        if (n - 0.0).abs() < 1e-3 {
            Some(0.0)
        } else if (n - 45.0).abs() < 1e-3 {
            Some(45.0)
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_mic_model(s: &str) -> Option<MicModel> {
    match s {
        "sm57" => Some(MicModel::Sm57),
        "md421" => Some(MicModel::Md421),
        "m88" => Some(MicModel::M88),
        "r121" => Some(MicModel::R121),
        "m160" => Some(MicModel::M160),
        "u87" => Some(MicModel::U87),
        "c414" => Some(MicModel::C414),
        _ => None,
    }
}

fn parse_power_tube(s: &str) -> Option<PowerTube> {
    match s {
        "el34" => Some(PowerTube::El34),
        "6l6gc" => Some(PowerTube::Tube6L6Gc),
        "kt88" => Some(PowerTube::Kt88),
        "el84_variant" | "el84" => Some(PowerTube::El84),
        _ => None,
    }
}

fn parse_rectifier(s: &str) -> Option<RectifierType> {
    match s {
        "diode" => Some(RectifierType::SolidStateDiode),
        "gz34" => Some(RectifierType::Gz34_5Ar4),
        "5u4gb" => Some(RectifierType::Dual5U4Gb),
        _ => None,
    }
}

pub fn parse_preamp_tube_or_default(s: &str) -> PreampTube {
    PreampTube::from_str_name(s).unwrap_or(PreampTube::Ax7A)
}

pub fn parse_tonestack(s: &str) -> Option<ToneStackTopology> {
    match s {
        "model" => Some(ToneStackTopology::Model),
        "marshall" => Some(ToneStackTopology::MarshallCathodeFollower),
        "fender" => Some(ToneStackTopology::FenderTmbMidScoop),
        "vox" => Some(ToneStackTopology::VoxTopBoost),
        "baxandall" => Some(ToneStackTopology::PassiveJamesBaxandall),
        _ => None,
    }
}

pub fn parse_tonestack_or_default(s: &str) -> ToneStackTopology {
    parse_tonestack(s).unwrap_or(ToneStackTopology::Model)
}

fn parse_mod_model(s: &str) -> Option<ModModel> {
    match s {
        "AnalogBbdChorus" | "bbd_chorus" => Some(ModModel::AnalogBbdChorus),
        "StereoTriChorus" | "tri_chorus" => Some(ModModel::StereoTriChorus),
        "ThroughZeroFlanger" | "tz_flanger" => Some(ModModel::ThroughZeroFlanger),
        "Orange90Phaser" | "phaser_90" => Some(ModModel::Orange90Phaser),
        "OpticalBiasTremolo" | "optical_tremolo" => Some(ModModel::OpticalBiasTremolo),
        "PhotocellUnivibe" | "univibe" => Some(ModModel::PhotocellUnivibe),
        "RotarySpeaker" | "rotary" => Some(ModModel::RotarySpeaker),
        _ => None,
    }
}

fn parse_pitch_model(s: &str) -> Option<PitchModel> {
    match s {
        "PedalWhammy" | "whammy" => Some(PitchModel::PedalWhammy),
        "PolyphonicDropCapo" | "drop_capo" => Some(PitchModel::PolyphonicDropCapo),
        "IntelligentDualHarmonizer" | "harmonizer" => Some(PitchModel::IntelligentDualHarmonizer),
        "MicroPitchDetune" | "micro_detune" => Some(PitchModel::MicroPitchDetune),
        _ => None,
    }
}

fn parse_pitch_key(s: &str) -> Option<PitchKey> {
    match s {
        "C" => Some(PitchKey::C),
        "C#" | "Db" | "CSharp" => Some(PitchKey::CSharp),
        "D" => Some(PitchKey::D),
        "D#" | "Eb" | "DSharp" => Some(PitchKey::DSharp),
        "E" => Some(PitchKey::E),
        "F" => Some(PitchKey::F),
        "F#" | "Gb" | "FSharp" => Some(PitchKey::FSharp),
        "G" => Some(PitchKey::G),
        "G#" | "Ab" | "GSharp" => Some(PitchKey::GSharp),
        "A" => Some(PitchKey::A),
        "A#" | "Bb" | "ASharp" => Some(PitchKey::ASharp),
        "B" => Some(PitchKey::B),
        _ => None,
    }
}

fn parse_pitch_scale(s: &str) -> Option<ScaleType> {
    match s {
        "major" | "Major" => Some(ScaleType::Major),
        "natural_minor" | "NaturalMinor" => Some(ScaleType::NaturalMinor),
        "harmonic_minor" | "HarmonicMinor" => Some(ScaleType::HarmonicMinor),
        "melodic_minor" | "MelodicMinor" => Some(ScaleType::MelodicMinor),
        "dorian" | "Dorian" => Some(ScaleType::Dorian),
        "phrygian" | "Phrygian" => Some(ScaleType::Phrygian),
        "lydian" | "Lydian" => Some(ScaleType::Lydian),
        "mixolydian" | "Mixolydian" => Some(ScaleType::Mixolydian),
        "locrian" | "Locrian" => Some(ScaleType::Locrian),
        _ => None,
    }
}

fn parse_dly_mode_u8(v: &Value) -> Option<u8> {
    let m = v
        .as_str()
        .and_then(|s| s.parse::<u8>().ok())
        .or_else(|| v.as_u64().and_then(|n| u8::try_from(n).ok()))?;
    (1..=12).contains(&m).then_some(m)
}

fn parse_dly_heads_u8(s: &str) -> Option<u8> {
    match s {
        "1" => Some(1),
        "2" => Some(2),
        "3" => Some(3),
        "1+2" => Some(4),
        "1+3" => Some(5),
        "2+3" => Some(6),
        "all" => Some(7),
        _ => None,
    }
}

fn parse_scene_u8(s: &str) -> Option<u8> {
    ["A", "B", "C", "D", "E", "F", "G", "H"]
        .iter()
        .position(|&l| l == s)
        .map(|i| i as u8)
}

fn parse_footswitch_mode_u8(s: &str) -> Option<u8> {
    match s.to_ascii_uppercase().as_str() {
        "PRESET" => Some(0),
        "SCENE" | "SCENES" => Some(1),
        "STOMP" => Some(2),
        "HYBRID_STOMP_SCENE" => Some(3),
        _ => None,
    }
}

// ── Conversões f32 / bool / enum compactas por campo ─────────────────────────────

fn json_f32(n: f32) -> Value {
    if !n.is_finite() {
        return Value::Null;
    }
    serde_json::to_string(&n)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!(n))
}

fn json_bool(b: bool) -> Value {
    Value::Bool(b)
}

fn json_str(s: &str) -> Value {
    Value::String(s.to_string())
}

fn val_f32(v: &Value) -> Option<f32> {
    v.as_f64().filter(|f| f.is_finite()).map(|f| f as f32)
}

macro_rules! num_param {
    (
        $key:expr, $aliases:expr, $min:expr, $max:expr, $step:expr, $def:expr,
        $models:expr, $dsp:expr, $field:ident
    ) => {
        ParamDesc {
            key: $key,
            aliases: $aliases,
            kind: ParamKind::Number { min: $min, max: $max, step: $step },
            default: ParamDefault::Number($def),
            models: $models,
            to_dsp: |v| val_f32(v).map($dsp),
            read_state: |st| json_f32(st.$field),
            write_state: |st, v| {
                if let Some(n) = val_f32(v) {
                    st.$field = n;
                }
            },
        }
    };
}

macro_rules! bool_param {
    ($key:expr, $def:expr, $models:expr, $dsp:expr, $field:ident) => {
        ParamDesc {
            key: $key,
            aliases: &[],
            kind: ParamKind::Bool,
            default: ParamDefault::Bool($def),
            models: $models,
            to_dsp: |v| v.as_bool().map($dsp),
            read_state: |st| json_bool(st.$field),
            write_state: |st, v| {
                if let Some(b) = v.as_bool() {
                    st.$field = b;
                }
            },
        }
    };
}

macro_rules! eq_band_gain {
    ($n:literal, $idx:literal) => {
        ParamDesc {
            key: concat!("eq_1.band", stringify!($n), ".gain_db"),
            aliases: &[],
            kind: ParamKind::Number { min: -15.0, max: 15.0, step: 0.5 },
            default: ParamDefault::Number(0.0),
            models: None,
            to_dsp: |v| val_f32(v).map(|g| DspParamUpdate::Eq1BandGain($idx, g)),
            read_state: |st| json_f32(st.eq_1_bands_db[$idx]),
            write_state: |st, v| {
                if let Some(n) = val_f32(v) {
                    st.eq_1_bands_db[$idx] = n;
                }
            },
        }
    };
}

macro_rules! eq_band_freq {
    ($n:literal, $idx:literal, $def_freq:expr) => {
        ParamDesc {
            key: concat!("eq_1.band", stringify!($n), ".freq_hz"),
            aliases: &[],
            kind: ParamKind::Number { min: 20.0, max: 20000.0, step: 1.0 },
            default: ParamDefault::Number($def_freq),
            models: None,
            to_dsp: |v| val_f32(v).map(|f| DspParamUpdate::Eq1BandFreq($idx, f)),
            read_state: |st| json_f32(st.eq_1_band_freqs_hz[$idx]),
            write_state: |st, v| {
                if let Some(n) = val_f32(v) {
                    st.eq_1_band_freqs_hz[$idx] = n;
                }
            },
        }
    };
}

macro_rules! eq_band_q {
    ($n:literal, $idx:literal) => {
        ParamDesc {
            key: concat!("eq_1.band", stringify!($n), ".q"),
            aliases: &[],
            kind: ParamKind::Number { min: 0.3, max: 10.0, step: 0.1 },
            default: ParamDefault::Number(1.0),
            models: None,
            to_dsp: |v| val_f32(v).map(|q| DspParamUpdate::Eq1BandQ($idx, q)),
            read_state: |st| json_f32(st.eq_1_band_qs[$idx]),
            write_state: |st, v| {
                if let Some(n) = val_f32(v) {
                    st.eq_1_band_qs[$idx] = n;
                }
            },
        }
    };
}

const AMP_MODELS_ORDER: &[&str] = &[
    "plexi_1959",
    "jcm800_2203",
    "twin_ab763",
    "dual_rect",
    "ac30_tb",
];
const SPEAKER_MODELS: &[&str] = &[
    "greenback_4x12",
    "v30_4x12",
    "alnico_blue_2x12",
    "twin_2x12_open",
];
const MIC_MODELS: &[&str] = &["sm57", "md421", "m88", "r121", "m160", "u87", "c414"];
const MIC_ANGLES: &[&str] = &["0", "45"];
const SCENES_LIST: &[&str] = &["A", "B", "C", "D", "E", "F", "G", "H"];

const MODELS_PRESENCE: &[AmpModelId] = &[
    AmpModelId::BritPlexi1959,
    AmpModelId::BritJcm800_2203,
    AmpModelId::CaliDualRectoRevF,
];
const MODELS_MASTER: &[AmpModelId] = &[AmpModelId::BritJcm800_2203, AmpModelId::CaliDualRectoRevF];
const MODELS_MIDDLE: &[AmpModelId] = &[
    AmpModelId::BritPlexi1959,
    AmpModelId::BritJcm800_2203,
    AmpModelId::UsTwinBlackface65,
    AmpModelId::CaliDualRectoRevF,
];
const MODELS_VOLUME_2: &[AmpModelId] = &[AmpModelId::BritPlexi1959, AmpModelId::UkAc30TopBoost];
const MODELS_CUT: &[AmpModelId] = &[AmpModelId::UkAc30TopBoost];
const MODELS_REVERB: &[AmpModelId] = &[AmpModelId::UsTwinBlackface65];
const MODELS_VIBRATO: &[AmpModelId] = &[AmpModelId::UsTwinBlackface65, AmpModelId::UkAc30TopBoost];
const MODELS_AC30_ONLY: &[AmpModelId] = &[AmpModelId::UkAc30TopBoost];
const MODELS_JUMPERED: &[AmpModelId] = &[AmpModelId::BritPlexi1959];
const MODELS_SPONGY: &[AmpModelId] = &[AmpModelId::CaliDualRectoRevF];

/// Fonte única de verdade para todos os parâmetros do motor & interface.
/// REGRA DE ORDEM: `amp_1.model` precede todo `amp_1.*`; `cab_1.speaker` precede `cab_1.resonance_hz`.
pub static PARAMS: &[ParamDesc] = &[
    ParamDesc {
        key: "amp_1.model",
        aliases: &[],
        kind: ParamKind::Enum(AMP_MODELS_ORDER),
        default: ParamDefault::Enum(MODEL_JCM800),
        models: None,
        to_dsp: |v| v.as_str().and_then(find_model_by_name).map(DspParamUpdate::AmpModel),
        read_state: |st| json_str(&st.amp_model),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.amp_model = s.to_string();
            }
        },
    },
    bool_param!("amp_1.enabled", true, None, DspParamUpdate::AmpEnabled, amp_enabled),
    bool_param!("amp_1.power", true, None, DspParamUpdate::AmpPower, amp_power),
    bool_param!("amp_1.standby", false, None, DspParamUpdate::AmpStandby, amp_standby),
    bool_param!("amp_1.bright", false, None, DspParamUpdate::AmpBright, amp_bright),
    bool_param!(
        "amp_1.jumpered",
        false,
        Some(MODELS_JUMPERED),
        DspParamUpdate::AmpJumpered,
        amp_jumpered
    ),
    bool_param!(
        "amp_1.spongy",
        false,
        Some(MODELS_SPONGY),
        DspParamUpdate::AmpSpongy,
        amp_spongy
    ),
    ParamDesc {
        key: "amp_1.input_jack",
        aliases: &[],
        kind: ParamKind::Enum(&[
            "high",
            "low",
            "high1",
            "low1",
            "high2",
            "low2",
            "jumpered",
            "normal_high",
            "normal_low",
            "brilliant_high",
            "brilliant_low",
            "normal",
            "vibrato",
        ]),
        default: ParamDefault::Enum("high"),
        models: None,
        to_dsp: |v| v.as_str().and_then(AmpInputJack::from_str_name).map(DspParamUpdate::AmpInputJack),
        read_state: |st| json_str(&st.amp_input_jack),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.amp_input_jack = s.to_string();
            }
        },
    },
    ParamDesc {
        key: "amp_1.channel_mode",
        aliases: &[],
        kind: ParamKind::Enum(&["clean", "vintage", "modern"]),
        default: ParamDefault::Enum("modern"),
        models: None,
        to_dsp: |v| {
            v.as_str()
                .and_then(AmpChannelMode::from_str_name)
                .map(DspParamUpdate::AmpChannelMode)
        },
        read_state: |st| json_str(&st.amp_channel_mode),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.amp_channel_mode = s.to_string();
            }
        },
    },
    ParamDesc {
        key: "amp_1.impedance_ohms",
        aliases: &[],
        kind: ParamKind::Enum(&["4", "8", "16"]),
        default: ParamDefault::Enum("16"),
        models: None,
        to_dsp: |v| {
            v.as_str().and_then(|s| match s {
                "4" => Some(DspParamUpdate::AmpImpedanceOhms(4)),
                "8" => Some(DspParamUpdate::AmpImpedanceOhms(8)),
                "16" => Some(DspParamUpdate::AmpImpedanceOhms(16)),
                _ => None,
            })
        },
        read_state: |st| json_str(&st.amp_impedance_ohms),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.amp_impedance_ohms = s.to_string();
            }
        },
    },
    num_param!("amp_1.gain", &[], 0.0, 10.0, 0.1, 8.0, None, DspParamUpdate::AmpGain, amp_gain),
    num_param!("amp_1.bass", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::AmpBass, amp_bass),
    num_param!(
        "amp_1.middle",
        &[],
        0.0,
        10.0,
        0.1,
        5.0,
        Some(MODELS_MIDDLE),
        DspParamUpdate::AmpMiddle,
        amp_middle
    ),
    num_param!("amp_1.treble", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::AmpTreble, amp_treble),
    num_param!(
        "amp_1.presence",
        &[],
        0.0,
        10.0,
        0.1,
        5.0,
        Some(MODELS_PRESENCE),
        DspParamUpdate::AmpPresence,
        amp_presence
    ),
    num_param!(
        "amp_1.master",
        &[],
        0.0,
        10.0,
        0.1,
        7.0,
        Some(MODELS_MASTER),
        DspParamUpdate::AmpMaster,
        amp_master
    ),
    num_param!(
        "amp_1.volume_2",
        &[],
        0.0,
        10.0,
        0.1,
        0.0,
        Some(MODELS_VOLUME_2),
        DspParamUpdate::AmpVolume2,
        amp_volume_2
    ),
    num_param!(
        "amp_1.cut",
        &[],
        0.0,
        10.0,
        0.1,
        5.0,
        Some(MODELS_CUT),
        DspParamUpdate::AmpCut,
        amp_cut
    ),
    num_param!(
        "amp_1.reverb",
        &[],
        0.0,
        10.0,
        0.1,
        0.0,
        Some(MODELS_REVERB),
        DspParamUpdate::AmpReverb,
        amp_reverb
    ),
    num_param!(
        "amp_1.vibrato_speed",
        &[],
        0.0,
        10.0,
        0.1,
        4.0,
        Some(MODELS_VIBRATO),
        DspParamUpdate::AmpVibratoSpeed,
        amp_vibrato_speed
    ),
    num_param!(
        "amp_1.vibrato_depth",
        &[],
        0.0,
        10.0,
        0.1,
        0.0,
        Some(MODELS_VIBRATO),
        DspParamUpdate::AmpVibratoDepth,
        amp_vibrato_depth
    ),
    ParamDesc {
        key: "amp_1.vib_trem_mode",
        aliases: &[],
        kind: ParamKind::Enum(&["vibrato", "tremolo"]),
        default: ParamDefault::Enum("vibrato"),
        models: Some(MODELS_AC30_ONLY),
        to_dsp: |v| {
            v.as_str()
                .and_then(Ac30VibTremMode::from_str_name)
                .map(DspParamUpdate::AmpVibTremMode)
        },
        read_state: |st| json_str(&st.amp_vib_trem_mode),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.amp_vib_trem_mode = s.to_string();
            }
        },
    },
    ParamDesc {
        key: "amp_1.preamp_tube",
        aliases: &[],
        kind: ParamKind::Enum(&["12ax7", "ecc83", "12at7", "12ay7", "ef86", "ef86_variant"]),
        default: ParamDefault::Enum("12ax7"),
        models: None,
        to_dsp: |v| {
            v.as_str()
                .and_then(PreampTube::from_str_name)
                .map(DspParamUpdate::AmpPreampTube)
        },
        read_state: |st| json_str(&st.preamp_tube),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.preamp_tube = s.to_string();
            }
        },
    },
    ParamDesc {
        key: "amp_1.power_tube",
        aliases: &[],
        kind: ParamKind::Enum(&["el34", "6l6gc", "kt88", "el84_variant"]),
        default: ParamDefault::Enum("el34"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_power_tube).map(DspParamUpdate::AmpPowerTube),
        read_state: |st| json_str(&st.power_tube),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.power_tube = s.to_string();
            }
        },
    },
    ParamDesc {
        key: "amp_1.rectifier",
        aliases: &[],
        kind: ParamKind::Enum(&["diode", "gz34", "5u4gb"]),
        default: ParamDefault::Enum("diode"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_rectifier).map(DspParamUpdate::AmpRectifier),
        read_state: |st| json_str(&st.rectifier),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.rectifier = s.to_string();
            }
        },
    },
    ParamDesc {
        key: "amp_1.tonestack",
        aliases: &[],
        kind: ParamKind::Enum(&["model", "marshall", "fender", "vox", "baxandall"]),
        default: ParamDefault::Enum("model"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_tonestack).map(DspParamUpdate::AmpToneStack),
        read_state: |st| json_str(&st.tonestack),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.tonestack = s.to_string();
            }
        },
    },
    num_param!("amp_1.bias_ma", &[], 20.0, 55.0, 0.5, 38.0, None, DspParamUpdate::AmpBiasMa, bias_ma),
    num_param!("amp_1.sag", &[], 0.0, 10.0, 0.1, 3.0, None, DspParamUpdate::AmpSag, sag),
    num_param!("amp_1.nfb", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::AmpNfb, nfb),
    num_param!(
        "amp_1.xformer_sat",
        &[],
        0.0,
        10.0,
        0.1,
        4.0,
        None,
        DspParamUpdate::AmpXformerSat,
        xformer_sat
    ),
    num_param!(
        "amp_1.preamp_plate_v",
        &[],
        180.0,
        340.0,
        1.0,
        280.0,
        None,
        DspParamUpdate::AmpPreampPlateV,
        preamp_plate_v
    ),
    num_param!(
        "amp_1.triode_asym",
        &[],
        0.0,
        0.6,
        0.01,
        0.2,
        None,
        DspParamUpdate::AmpTriodeAsym,
        triode_asym
    ),
    // ── Engine / Input ───────────────────────────────────────────────────────────
    ParamDesc {
        key: "engine.oversampling",
        aliases: &[],
        kind: ParamKind::Enum(&["2x", "4x", "8x"]),
        default: ParamDefault::Enum("2x"),
        models: None,
        to_dsp: |v| {
            v.as_str().and_then(|s| match s {
                "2x" => Some(DspParamUpdate::EngineOversampling(2)),
                "4x" => Some(DspParamUpdate::EngineOversampling(4)),
                "8x" => Some(DspParamUpdate::EngineOversampling(8)),
                _ => None,
            })
        },
        read_state: |st| json_str(&st.engine_oversampling),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.engine_oversampling = s.to_string();
            }
        },
    },
    num_param!("input.boost", &[], 0.1, 8.0, 0.05, 2.5, None, DspParamUpdate::InputBoost, input_boost),
    ParamDesc {
        key: "input.source",
        aliases: &[],
        kind: ParamKind::Enum(&["device", "test_riff", "di_file"]),
        default: ParamDefault::Enum("device"),
        models: None,
        to_dsp: |v| {
            v.as_str().and_then(|s| match s {
                "device" => Some(DspParamUpdate::InputSource(0)),
                "test_riff" => Some(DspParamUpdate::InputSource(1)),
                "di_file" => Some(DspParamUpdate::InputSource(2)),
                _ => None,
            })
        },
        read_state: |st| json_str(&st.input_source),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.input_source = s.to_string();
            }
        },
    },
    ParamDesc {
        key: "input.mode",
        aliases: &[],
        kind: ParamKind::Enum(&["instrument", "pc_line_in", "line"]),
        default: ParamDefault::Enum("instrument"),
        models: None,
        to_dsp: |v| {
            v.as_str().and_then(|s| match s {
                "instrument" | "pc_line_in" => Some(DspParamUpdate::InputModeLine(false)),
                "line" => Some(DspParamUpdate::InputModeLine(true)),
                _ => None,
            })
        },
        read_state: |st| json_str(&st.input_mode),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.input_mode = s.to_string();
            }
        },
    },
    ParamDesc {
        key: "input.channel",
        aliases: &[],
        kind: ParamKind::Enum(&["left", "right", "sum"]),
        default: ParamDefault::Enum("left"),
        models: None,
        to_dsp: |v| {
            v.as_str().and_then(|s| match s {
                "left" => Some(DspParamUpdate::InputChannel(crate::audio::InputChannel::Left)),
                "right" => Some(DspParamUpdate::InputChannel(crate::audio::InputChannel::Right)),
                "sum" => Some(DspParamUpdate::InputChannel(crate::audio::InputChannel::Sum)),
                _ => None,
            })
        },
        read_state: |st| json_str(&st.input_channel),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.input_channel = s.to_string();
            }
        },
    },
    // ── Noise Gate ───────────────────────────────────────────────────────────────
    bool_param!("gate.enabled", true, None, DspParamUpdate::GateEnabled, gate_enabled),
    ParamDesc {
        key: "gate.mode",
        aliases: &[],
        kind: ParamKind::Enum(&["natural", "classic"]),
        default: ParamDefault::Enum("natural"),
        models: None,
        to_dsp: |v| {
            v.as_str().and_then(|s| match s {
                "natural" => Some(DspParamUpdate::GateMode(NoiseGateMode::IntelligentNaturalGate)),
                "classic" => Some(DspParamUpdate::GateMode(NoiseGateMode::ClassicExpanderGate)),
                _ => None,
            })
        },
        read_state: |st| json_str(&st.gate_mode),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.gate_mode = s.to_string();
            }
        },
    },
    num_param!(
        "gate.threshold_db",
        &[],
        -90.0,
        0.0,
        0.5,
        -65.0,
        None,
        DspParamUpdate::GateThreshold,
        gate_threshold_db
    ),
    num_param!(
        "gate.hysteresis_db",
        &[],
        0.0,
        12.0,
        0.5,
        4.0,
        None,
        DspParamUpdate::GateHysteresis,
        gate_hysteresis_db
    ),
    num_param!(
        "gate.attack_ms",
        &[],
        0.1,
        50.0,
        0.1,
        1.0,
        None,
        DspParamUpdate::GateAttack,
        gate_attack_ms
    ),
    num_param!(
        "gate.hold_ms",
        &[],
        0.0,
        500.0,
        1.0,
        35.0,
        None,
        DspParamUpdate::GateHold,
        gate_hold_ms
    ),
    num_param!(
        "gate.release_ms",
        &[],
        5.0,
        1000.0,
        1.0,
        80.0,
        None,
        DspParamUpdate::GateRelease,
        gate_release_ms
    ),
    num_param!(
        "gate.range_db",
        &[],
        -90.0,
        0.0,
        1.0,
        -70.0,
        None,
        DspParamUpdate::GateRange,
        gate_range_db
    ),
    num_param!(
        "gate.key_hpf_hz",
        &[],
        20.0,
        500.0,
        5.0,
        80.0,
        None,
        DspParamUpdate::GateKeyHpf,
        gate_key_hpf_hz
    ),
    num_param!(
        "gate.key_lpf_hz",
        &[],
        1000.0,
        12000.0,
        100.0,
        6000.0,
        None,
        DspParamUpdate::GateKeyLpf,
        gate_key_lpf_hz
    ),
    // ── Drive Pedal ──────────────────────────────────────────────────────────────
    bool_param!("drive.enabled", false, None, DspParamUpdate::DriveEnabled, drive_enabled),
    ParamDesc {
        key: "drive.model",
        aliases: &[],
        kind: ParamKind::Enum(&["ts808", "klon", "rat", "muff", "fuzzface", "ds1"]),
        default: ParamDefault::Enum("ts808"),
        models: None,
        to_dsp: |v| v.as_str().and_then(DriveModel::from_str_name).map(DspParamUpdate::DriveModel),
        read_state: |st| json_str(&st.drive_model),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.drive_model = s.to_string();
            }
        },
    },
    num_param!("drive.drive", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::DriveGain, drive_drive),
    num_param!("drive.tone", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::DriveTone, drive_tone),
    num_param!("drive.level", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::DriveLevel, drive_level),
    num_param!("drive.bias", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::DriveBias, drive_bias),
    // ── Cab 1 & Mics ─────────────────────────────────────────────────────────────
    bool_param!("cab_1.enabled", true, None, DspParamUpdate::CabEnabled, cab_enabled),
    ParamDesc {
        key: "cab_1.speaker",
        aliases: &["cab_1.model"],
        kind: ParamKind::Enum(SPEAKER_MODELS),
        default: ParamDefault::Enum("greenback_4x12"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_speaker_model).map(DspParamUpdate::Cab1Speaker),
        read_state: |st| json_str(&st.cab_speaker),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.cab_speaker = s.to_string();
            }
        },
    },
    num_param!(
        "cab_1.resonance_hz",
        &[],
        65.0,
        135.0,
        1.0,
        98.0,
        None,
        DspParamUpdate::CabResonanceHz,
        cab_resonance_hz
    ),
    num_param!(
        "cab_1.hicut_khz",
        &[],
        3.5,
        18.0,
        0.1,
        8.2,
        None,
        DspParamUpdate::CabHicutKhz,
        cab_hicut_khz
    ),
    bool_param!("cab_1.ir_enabled", false, None, DspParamUpdate::CabIrEnabled, cab_ir_enabled),
    bool_param!(
        "cab_1.mic1.enabled",
        true,
        None,
        DspParamUpdate::CabMic1Enabled,
        cab_mic1_enabled
    ),
    ParamDesc {
        key: "cab_1.mic1.model",
        aliases: &[],
        kind: ParamKind::Enum(MIC_MODELS),
        default: ParamDefault::Enum("sm57"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_mic_model).map(DspParamUpdate::CabMic1Model),
        read_state: |st| json_str(&st.cab_mic1_model),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.cab_mic1_model = s.to_string();
            }
        },
    },
    num_param!(
        "cab_1.mic1.position",
        &[],
        0.0,
        1.0,
        0.01,
        0.2,
        None,
        DspParamUpdate::CabMic1Position,
        cab_mic1_position
    ),
    num_param!(
        "cab_1.mic1.distance_cm",
        &[],
        0.0,
        30.0,
        0.5,
        2.0,
        None,
        DspParamUpdate::CabMic1DistanceCm,
        cab_mic1_distance_cm
    ),
    num_param!(
        "cab_1.mic1.level_db",
        &[],
        -60.0,
        12.0,
        0.5,
        0.0,
        None,
        DspParamUpdate::CabMic1LevelDb,
        cab_mic1_level_db
    ),
    ParamDesc {
        key: "cab_1.mic1.angle_deg",
        aliases: &[],
        kind: ParamKind::Enum(MIC_ANGLES),
        default: ParamDefault::Enum("0"),
        models: None,
        to_dsp: |v| parse_mic_angle_deg(v).map(DspParamUpdate::Cab1Mic1AngleDeg),
        read_state: |st| json_str(&st.cab_mic1_angle_deg),
        write_state: |st, v| {
            if let Some(deg) = parse_mic_angle_deg(v) {
                st.cab_mic1_angle_deg = if deg >= 22.5 { "45" } else { "0" }.to_string();
            }
        },
    },
    bool_param!(
        "cab_1.mic1.phase_invert",
        false,
        None,
        DspParamUpdate::Cab1Mic1PhaseInvert,
        cab_mic1_phase_invert
    ),
    bool_param!(
        "cab_1.mic2.enabled",
        false,
        None,
        DspParamUpdate::CabMic2Enabled,
        cab_mic2_enabled
    ),
    ParamDesc {
        key: "cab_1.mic2.model",
        aliases: &[],
        kind: ParamKind::Enum(MIC_MODELS),
        default: ParamDefault::Enum("sm57"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_mic_model).map(DspParamUpdate::CabMic2Model),
        read_state: |st| json_str(&st.cab_mic2_model),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.cab_mic2_model = s.to_string();
            }
        },
    },
    num_param!(
        "cab_1.mic2.position",
        &[],
        0.0,
        1.0,
        0.01,
        0.5,
        None,
        DspParamUpdate::CabMic2Position,
        cab_mic2_position
    ),
    num_param!(
        "cab_1.mic2.distance_cm",
        &[],
        0.0,
        30.0,
        0.5,
        5.0,
        None,
        DspParamUpdate::CabMic2DistanceCm,
        cab_mic2_distance_cm
    ),
    num_param!(
        "cab_1.mic2.level_db",
        &[],
        -60.0,
        12.0,
        0.5,
        0.0,
        None,
        DspParamUpdate::CabMic2LevelDb,
        cab_mic2_level_db
    ),
    ParamDesc {
        key: "cab_1.mic2.angle_deg",
        aliases: &[],
        kind: ParamKind::Enum(MIC_ANGLES),
        default: ParamDefault::Enum("0"),
        models: None,
        to_dsp: |v| parse_mic_angle_deg(v).map(DspParamUpdate::Cab1Mic2AngleDeg),
        read_state: |st| json_str(&st.cab_mic2_angle_deg),
        write_state: |st, v| {
            if let Some(deg) = parse_mic_angle_deg(v) {
                st.cab_mic2_angle_deg = if deg >= 22.5 { "45" } else { "0" }.to_string();
            }
        },
    },
    bool_param!(
        "cab_1.mic2.phase_invert",
        false,
        None,
        DspParamUpdate::Cab1Mic2PhaseInvert,
        cab_mic2_phase_invert
    ),
    // ── Split / Merge ────────────────────────────────────────────────────────────
    bool_param!("split.enabled", false, None, DspParamUpdate::SplitEnabled, split_enabled),
    ParamDesc {
        key: "split.mode",
        aliases: &[],
        kind: ParamKind::Enum(&["ab", "crossover", "dynamic_split"]),
        default: ParamDefault::Enum("ab"),
        models: None,
        to_dsp: |v| v.as_str().map(|s| DspParamUpdate::SplitMode(ChainSplitMode::from_str_name(s))),
        read_state: |st| json_str(&st.split_mode),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.split_mode = s.to_string();
            }
        },
    },
    num_param!(
        "split.crossover_hz",
        &[],
        100.0,
        2000.0,
        10.0,
        650.0,
        None,
        DspParamUpdate::SplitCrossoverHz,
        split_crossover_hz
    ),
    num_param!(
        "split.balance",
        &["split.ratio"],
        0.0,
        1.0,
        0.01,
        0.5,
        None,
        DspParamUpdate::SplitRatio,
        split_balance
    ),
    num_param!(
        "split.dyn_threshold_db",
        &[],
        -60.0,
        0.0,
        0.5,
        -30.0,
        None,
        DspParamUpdate::SplitDynThresholdDb,
        split_dyn_threshold_db
    ),
    num_param!(
        "split.dyn_attack_ms",
        &[],
        1.0,
        50.0,
        0.5,
        5.0,
        None,
        DspParamUpdate::SplitDynAttackMs,
        split_dyn_attack_ms
    ),
    num_param!(
        "split.dyn_release_ms",
        &[],
        10.0,
        500.0,
        5.0,
        80.0,
        None,
        DspParamUpdate::SplitDynReleaseMs,
        split_dyn_release_ms
    ),
    num_param!(
        "merge.row_1_level_db",
        &[],
        -60.0,
        12.0,
        0.5,
        -6.0,
        None,
        DspParamUpdate::MergeRow1LevelDb,
        merge_row_1_level_db
    ),
    bool_param!(
        "merge.row_1_phase_invert",
        false,
        None,
        DspParamUpdate::MergeRow1PhaseInvert,
        merge_row_1_phase_invert
    ),
    // ── Compressor 1176 (comp_2) ─────────────────────────────────────────────────
    bool_param!("comp_2.enabled", false, None, DspParamUpdate::Comp2Enabled, comp_2_enabled),
    ParamDesc {
        key: "comp_2.input",
        aliases: &[],
        kind: ParamKind::Number { min: 0.0, max: 10.0, step: 0.1 },
        default: ParamDefault::Number(5.0),
        models: None,
        to_dsp: |v| {
            val_f32(v).map(|input| {
                let pre_gain_db = (input.clamp(0.0, 10.0) - 5.0) * 4.0;
                DspParamUpdate::Comp2ThresholdDb(-20.0 - pre_gain_db)
            })
        },
        read_state: |st| json_f32(st.comp_2_input),
        write_state: |st, v| {
            if let Some(n) = val_f32(v) {
                st.comp_2_input = n;
            }
        },
    },
    ParamDesc {
        key: "comp_2.output",
        aliases: &[],
        kind: ParamKind::Number { min: 0.0, max: 10.0, step: 0.1 },
        default: ParamDefault::Number(5.0),
        models: None,
        to_dsp: |v| {
            val_f32(v).map(|output| {
                let gain_db = (output.clamp(0.0, 10.0) - 5.0) * 4.8;
                DspParamUpdate::Comp2MakeupGainDb(gain_db)
            })
        },
        read_state: |st| json_f32(st.comp_2_output),
        write_state: |st, v| {
            if let Some(n) = val_f32(v) {
                st.comp_2_output = n;
            }
        },
    },
    ParamDesc {
        key: "comp_2.attack",
        aliases: &[],
        kind: ParamKind::Number { min: 1.0, max: 7.0, step: 0.1 },
        default: ParamDefault::Number(4.0),
        models: None,
        to_dsp: |v| {
            val_f32(v).map(|att| {
                let att_c = att.clamp(1.0, 7.0);
                let attack_ms = 0.8f32 * (0.02f32 / 0.8f32).powf((att_c - 1.0) / 6.0);
                DspParamUpdate::Comp2AttackMs(attack_ms)
            })
        },
        read_state: |st| json_f32(st.comp_2_attack),
        write_state: |st, v| {
            if let Some(n) = val_f32(v) {
                st.comp_2_attack = n;
            }
        },
    },
    ParamDesc {
        key: "comp_2.release",
        aliases: &[],
        kind: ParamKind::Number { min: 1.0, max: 7.0, step: 0.1 },
        default: ParamDefault::Number(4.0),
        models: None,
        to_dsp: |v| {
            val_f32(v).map(|rel| {
                let rel_c = rel.clamp(1.0, 7.0);
                let release_ms = 1100.0f32 * (50.0f32 / 1100.0f32).powf((rel_c - 1.0) / 6.0);
                DspParamUpdate::Comp2ReleaseMs(release_ms)
            })
        },
        read_state: |st| json_f32(st.comp_2_release),
        write_state: |st, v| {
            if let Some(n) = val_f32(v) {
                st.comp_2_release = n;
            }
        },
    },
    ParamDesc {
        key: "comp_2.ratio",
        aliases: &[],
        kind: ParamKind::Enum(&["4", "8", "12", "20", "all"]),
        default: ParamDefault::Enum("4"),
        models: None,
        to_dsp: |v| {
            v.as_str().and_then(|s| match s {
                "4" => Some(DspParamUpdate::Comp2Ratio { ratio: 4.0, all_buttons: false }),
                "8" => Some(DspParamUpdate::Comp2Ratio { ratio: 8.0, all_buttons: false }),
                "12" => Some(DspParamUpdate::Comp2Ratio { ratio: 12.0, all_buttons: false }),
                "20" => Some(DspParamUpdate::Comp2Ratio { ratio: 20.0, all_buttons: false }),
                "all" => Some(DspParamUpdate::Comp2Ratio { ratio: 20.0, all_buttons: true }),
                _ => None,
            })
        },
        read_state: |st| json_str(&st.comp_2_ratio),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.comp_2_ratio = s.to_string();
            }
        },
    },
    // ── Equalizer (eq_1) ─────────────────────────────────────────────────────────
    bool_param!("eq_1.enabled", false, None, DspParamUpdate::Eq1Enabled, eq_1_enabled),
    ParamDesc {
        key: "eq_1.mode",
        aliases: &[],
        kind: ParamKind::Enum(&["eight_band", "graphic_5_boogie", "mxr_10band"]),
        default: ParamDefault::Enum("eight_band"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_eq_mode).map(DspParamUpdate::Eq1Mode),
        read_state: |st| json_str(&st.eq_1_mode),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.eq_1_mode = s.to_string();
            }
        },
    },
    num_param!(
        "eq_1.gain_db",
        &["eq_1.input_gain_db"],
        -12.0,
        12.0,
        0.5,
        0.0,
        None,
        DspParamUpdate::Eq1InputGainDb,
        eq_1_gain_db
    ),
    num_param!(
        "eq_1.volume_db",
        &["eq_1.output_level_db"],
        -12.0,
        12.0,
        0.5,
        0.0,
        None,
        DspParamUpdate::Eq1OutputLevelDb,
        eq_1_volume_db
    ),
    eq_band_gain!(1, 0), eq_band_freq!(1, 0, 31.25), eq_band_q!(1, 0),
    eq_band_gain!(2, 1), eq_band_freq!(2, 1, 62.5), eq_band_q!(2, 1),
    eq_band_gain!(3, 2), eq_band_freq!(3, 2, 125.0), eq_band_q!(3, 2),
    eq_band_gain!(4, 3), eq_band_freq!(4, 3, 250.0), eq_band_q!(4, 3),
    eq_band_gain!(5, 4), eq_band_freq!(5, 4, 500.0), eq_band_q!(5, 4),
    eq_band_gain!(6, 5), eq_band_freq!(6, 5, 1000.0), eq_band_q!(6, 5),
    eq_band_gain!(7, 6), eq_band_freq!(7, 6, 2000.0), eq_band_q!(7, 6),
    eq_band_gain!(8, 7), eq_band_freq!(8, 7, 4000.0), eq_band_q!(8, 7),
    eq_band_gain!(9, 8), eq_band_freq!(9, 8, 8000.0), eq_band_q!(9, 8),
    eq_band_gain!(10, 9), eq_band_freq!(10, 9, 16000.0), eq_band_q!(10, 9),
    // ── Modulação (mod_2) ────────────────────────────────────────────────────────
    bool_param!("mod_2.enabled", false, None, DspParamUpdate::Mod2Enabled, mod_2_enabled),
    ParamDesc {
        key: "mod_2.model",
        aliases: &[],
        kind: ParamKind::Enum(&[
            "AnalogBbdChorus",
            "StereoTriChorus",
            "ThroughZeroFlanger",
            "Orange90Phaser",
            "OpticalBiasTremolo",
            "PhotocellUnivibe",
            "RotarySpeaker",
        ]),
        default: ParamDefault::Enum("AnalogBbdChorus"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_mod_model).map(DspParamUpdate::Mod2Model),
        read_state: |st| json_str(&st.mod_2_model),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.mod_2_model = s.to_string();
            }
        },
    },
    num_param!("mod_2.rate_hz", &[], 0.05, 10.0, 0.05, 1.0, None, DspParamUpdate::Mod2RateHz, mod_2_rate_hz),
    num_param!("mod_2.depth", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::Mod2Depth, mod_2_depth),
    num_param!(
        "mod_2.mix",
        &["mod_2.mix_pct"],
        0.0,
        100.0,
        1.0,
        50.0,
        None,
        DspParamUpdate::Mod2Mix,
        mod_2_mix
    ),
    // ── Pitch (pitch_1) ──────────────────────────────────────────────────────────
    bool_param!("pitch_1.enabled", false, None, DspParamUpdate::Pitch1Enabled, pitch_1_enabled),
    ParamDesc {
        key: "pitch_1.model",
        aliases: &[],
        kind: ParamKind::Enum(&["whammy", "drop_capo", "harmonizer", "micro_detune"]),
        default: ParamDefault::Enum("whammy"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_pitch_model).map(DspParamUpdate::Pitch1Model),
        read_state: |st| json_str(&st.pitch_1_model),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.pitch_1_model = s.to_string();
            }
        },
    },
    num_param!(
        "pitch_1.semitones",
        &[],
        -24.0,
        24.0,
        1.0,
        0.0,
        None,
        DspParamUpdate::Pitch1Semitones,
        pitch_1_semitones
    ),
    ParamDesc {
        key: "pitch_1.key",
        aliases: &[],
        kind: ParamKind::Enum(&["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"]),
        default: ParamDefault::Enum("C"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_pitch_key).map(DspParamUpdate::Pitch1Key),
        read_state: |st| json_str(&st.pitch_1_key),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.pitch_1_key = s.to_string();
            }
        },
    },
    ParamDesc {
        key: "pitch_1.scale",
        aliases: &[],
        kind: ParamKind::Enum(&[
            "major",
            "natural_minor",
            "harmonic_minor",
            "melodic_minor",
            "dorian",
            "phrygian",
            "lydian",
            "mixolydian",
            "locrian",
        ]),
        default: ParamDefault::Enum("major"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_pitch_scale).map(DspParamUpdate::Pitch1Scale),
        read_state: |st| json_str(&st.pitch_1_scale),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.pitch_1_scale = s.to_string();
            }
        },
    },
    num_param!(
        "pitch_1.mix",
        &["pitch_1.mix_pct"],
        0.0,
        100.0,
        1.0,
        100.0,
        None,
        DspParamUpdate::Pitch1Mix,
        pitch_1_mix
    ),
    // ── Delay RE-201 (dly_1) ─────────────────────────────────────────────────────
    bool_param!("dly_1.enabled", false, None, DspParamUpdate::Dly1Enabled, dly_1_enabled),
    ParamDesc {
        key: "dly_1.mode",
        aliases: &[],
        kind: ParamKind::Enum(&["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12"]),
        default: ParamDefault::Enum("1"),
        models: None,
        to_dsp: |v| parse_dly_mode_u8(v).map(DspParamUpdate::Dly1Mode),
        read_state: |st| json_str(&st.dly_1_mode),
        write_state: |st, v| {
            if let Some(m) = parse_dly_mode_u8(v) {
                st.dly_1_mode = m.to_string();
            }
        },
    },
    ParamDesc {
        key: "dly_1.heads",
        aliases: &[],
        kind: ParamKind::Enum(&["1", "2", "3", "1+2", "1+3", "2+3", "all"]),
        default: ParamDefault::Enum("1"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_dly_heads_u8).map(DspParamUpdate::Dly1Heads),
        read_state: |st| json_str(&st.dly_1_heads),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.dly_1_heads = s.to_string();
            }
        },
    },
    num_param!(
        "dly_1.repeat_rate",
        &[],
        0.0,
        10.0,
        0.1,
        5.0,
        None,
        DspParamUpdate::Dly1RepeatRate,
        dly_1_repeat_rate
    ),
    num_param!(
        "dly_1.intensity",
        &[],
        0.0,
        10.0,
        0.1,
        5.0,
        None,
        DspParamUpdate::Dly1Intensity,
        dly_1_intensity
    ),
    num_param!(
        "dly_1.echo_volume",
        &[],
        0.0,
        10.0,
        0.1,
        5.0,
        None,
        DspParamUpdate::Dly1EchoVolume,
        dly_1_echo_volume
    ),
    num_param!(
        "dly_1.reverb_volume",
        &[],
        0.0,
        10.0,
        0.1,
        5.0,
        None,
        DspParamUpdate::Dly1ReverbVolume,
        dly_1_reverb_volume
    ),
    num_param!("dly_1.bass", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::Dly1Bass, dly_1_bass),
    num_param!("dly_1.treble", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::Dly1Treble, dly_1_treble),
    bool_param!("dly_1.spillover", true, None, DspParamUpdate::Dly1Spillover, dly_1_spillover),
    // ── Reverb (rev_1) ───────────────────────────────────────────────────────────
    bool_param!("rev_1.enabled", false, None, DspParamUpdate::Rev1Enabled, rev_1_enabled),
    num_param!("rev_1.decay_s", &[], 0.1, 20.0, 0.1, 2.5, None, DspParamUpdate::Rev1DecayS, rev_1_decay_s),
    num_param!(
        "rev_1.predelay_ms",
        &[],
        0.0,
        250.0,
        1.0,
        20.0,
        None,
        DspParamUpdate::Rev1PredelayMs,
        rev_1_predelay_ms
    ),
    num_param!("rev_1.damping", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::Rev1Damping, rev_1_damping),
    num_param!(
        "rev_1.mix",
        &["rev_1.mix_pct"],
        0.0,
        100.0,
        1.0,
        40.0,
        None,
        DspParamUpdate::Rev1Mix,
        rev_1_mix
    ),
    bool_param!("rev_1.spillover", true, None, DspParamUpdate::Rev1Spillover, rev_1_spillover),
    // ── Row B: Amp 2 / Cab 2 / Wah ───────────────────────────────────────────────
    bool_param!("amp_2.enabled", false, None, DspParamUpdate::Amp2Enabled, amp_2_enabled),
    ParamDesc {
        key: "amp_2.model",
        aliases: &[],
        kind: ParamKind::Enum(AMP_MODELS_ORDER),
        default: ParamDefault::Enum("twin_ab763"),
        models: None,
        to_dsp: |v| v.as_str().and_then(find_model_by_name).map(DspParamUpdate::Amp2Model),
        read_state: |st| json_str(&st.amp_2_model),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.amp_2_model = s.to_string();
            }
        },
    },
    num_param!("amp_2.gain", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::Amp2Gain, amp_2_gain),
    num_param!("amp_2.bass", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::Amp2Bass, amp_2_bass),
    num_param!("amp_2.middle", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::Amp2Middle, amp_2_middle),
    num_param!("amp_2.treble", &[], 0.0, 10.0, 0.1, 5.0, None, DspParamUpdate::Amp2Treble, amp_2_treble),
    num_param!("amp_2.master", &[], 0.0, 10.0, 0.1, 7.0, None, DspParamUpdate::Amp2Master, amp_2_master),
    bool_param!("wah_1.enabled", false, None, DspParamUpdate::Wah1Enabled, wah_1_enabled),
    num_param!("wah_1.position", &[], 0.0, 1.0, 0.01, 0.5, None, DspParamUpdate::Wah1Position, wah_1_position),
    num_param!("wah_1.q", &[], 1.0, 10.0, 0.1, 5.0, None, DspParamUpdate::Wah1Q, wah_1_q),
    ParamDesc {
        key: "wah_1.mode",
        aliases: &[],
        kind: ParamKind::Enum(&["crybaby_gcb95", "clyde_mccoy", "auto_envelope"]),
        default: ParamDefault::Enum("crybaby_gcb95"),
        models: None,
        to_dsp: |v| v.as_str().and_then(WahMode::from_str_name).map(DspParamUpdate::Wah1Mode),
        read_state: |st| json_str(&st.wah_1_mode),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.wah_1_mode = s.to_string();
            }
        },
    },
    bool_param!("cab_2.enabled", false, None, DspParamUpdate::Cab2Enabled, cab_2_enabled),
    ParamDesc {
        key: "cab_2.speaker",
        aliases: &["cab_2.model"],
        kind: ParamKind::Enum(SPEAKER_MODELS),
        default: ParamDefault::Enum("alnico_blue_2x12"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_speaker_model).map(DspParamUpdate::Cab2Speaker),
        read_state: |st| json_str(&st.cab_2_speaker),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.cab_2_speaker = s.to_string();
            }
        },
    },
    num_param!(
        "cab_2.resonance_hz",
        &[],
        50.0,
        160.0,
        1.0,
        90.0,
        None,
        DspParamUpdate::Cab2ResonanceHz,
        cab_2_resonance_hz
    ),
    num_param!(
        "cab_2.hicut_khz",
        &[],
        2.0,
        20.0,
        0.1,
        9.0,
        None,
        DspParamUpdate::Cab2HicutKhz,
        cab_2_hicut_khz
    ),
    // ── Master / Palco / Cena / Tuner / Looper ───────────────────────────────────
    num_param!(
        "master.level_db",
        &[],
        -60.0,
        12.0,
        0.5,
        0.0,
        None,
        DspParamUpdate::MasterLevelDb,
        master_level_db
    ),
    ParamDesc {
        key: "palco.exp_1",
        aliases: &[],
        kind: ParamKind::Number { min: 0.0, max: 100.0, step: 1.0 },
        default: ParamDefault::Number(62.0),
        models: None,
        to_dsp: |v| {
            val_f32(v).map(|num| {
                let norm = if num > 10.0 { num / 100.0 } else { num / 10.0 };
                DspParamUpdate::Exp1Value(norm.clamp(0.0, 1.0))
            })
        },
        read_state: |st| json_f32(st.palco_exp_1),
        write_state: |st, v| {
            if let Some(n) = val_f32(v) {
                st.palco_exp_1 = n;
            }
        },
    },
    ParamDesc {
        key: "scene.active",
        aliases: &[],
        kind: ParamKind::Enum(SCENES_LIST),
        default: ParamDefault::Enum("A"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_scene_u8).map(DspParamUpdate::ActiveScene),
        read_state: |st| json_str(&st.scene_active),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.scene_active = s.to_string();
            }
        },
    },
    ParamDesc {
        key: "footswitch.mode",
        aliases: &[],
        kind: ParamKind::Enum(&["PRESET", "SCENE", "STOMP", "HYBRID_STOMP_SCENE"]),
        default: ParamDefault::Enum("HYBRID_STOMP_SCENE"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_footswitch_mode_u8).map(DspParamUpdate::FootswitchMode),
        read_state: |st| json_str(&st.footswitch_mode),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.footswitch_mode = s.to_string();
            }
        },
    },
    bool_param!("tuner.enabled", false, None, DspParamUpdate::TunerEnabled, tuner_enabled),
    ParamDesc {
        key: "tuner.output",
        aliases: &[],
        kind: ParamKind::Enum(&["mute", "bypass"]),
        default: ParamDefault::Enum("mute"),
        models: None,
        to_dsp: |v| {
            v.as_str().and_then(|s| match s {
                "mute" => Some(DspParamUpdate::TunerOutputMode(TunerOutputMode::Mute)),
                "bypass" => Some(DspParamUpdate::TunerOutputMode(TunerOutputMode::Bypass)),
                _ => None,
            })
        },
        read_state: |st| json_str(&st.tuner_output),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.tuner_output = s.to_string();
            }
        },
    },
    num_param!("tuner.ref_hz", &[], 430.0, 450.0, 0.5, 440.0, None, DspParamUpdate::TunerRefHz, tuner_ref_hz),
    ParamDesc {
        key: "looper.position",
        aliases: &[],
        kind: ParamKind::Enum(&["pre", "post"]),
        default: ParamDefault::Enum("post"),
        models: None,
        to_dsp: |v| {
            v.as_str().and_then(|s| match s {
                "pre" => Some(DspParamUpdate::LooperPosition(LooperPosition::PreChain)),
                "post" => Some(DspParamUpdate::LooperPosition(LooperPosition::PostChain)),
                _ => None,
            })
        },
        read_state: |st| json_str(&st.looper_position),
        write_state: |st, v| {
            if let Some(s) = v.as_str() {
                st.looper_position = s.to_string();
            }
        },
    },
    ParamDesc {
        key: "looper.action",
        aliases: &[],
        kind: ParamKind::Enum(&[
            "record",
            "play",
            "overdub",
            "stop",
            "undo",
            "redo",
            "reverse",
            "half_speed",
            "clear",
        ]),
        default: ParamDefault::Enum("stop"),
        models: None,
        to_dsp: |v| v.as_str().and_then(parse_looper_action).map(DspParamUpdate::LooperAction),
        read_state: |st| json_str(&st.looper_action),
        write_state: |st, v| {
            if let Some(act) = v.as_str() {
                match act {
                    "reverse" => st.looper_reverse = !st.looper_reverse,
                    "half_speed" => st.looper_half_speed = !st.looper_half_speed,
                    "rec" => st.looper_action = "record".to_string(),
                    other => st.looper_action = other.to_string(),
                }
            }
        },
    },
];

// ── Lookup, Validação & Serialização derivados de PARAMS ─────────────────────────

pub fn find_param_with_index(key: &str) -> Option<(usize, &'static ParamDesc)> {
    PARAMS
        .iter()
        .enumerate()
        .find(|(_, d)| d.key == key || d.aliases.contains(&key))
}

pub fn find_param(key: &str) -> Option<&'static ParamDesc> {
    find_param_with_index(key).map(|(_, d)| d)
}

pub fn read_param_bool(st: &AppState, key: &str) -> bool {
    find_param(key)
        .and_then(|d| (d.read_state)(st).as_bool())
        .unwrap_or(false)
}

pub fn is_amp_param_valid_for_model(model: &str, key: &str) -> bool {
    let Some(desc) = find_param(key) else {
        return true;
    };
    let Some(allowed) = desc.models else {
        return true;
    };
    let Some(model_id) = find_model_by_name(model) else {
        return false;
    };
    allowed.contains(&model_id)
}

fn validate_param_value(desc: &ParamDesc, key: &str, v: &Value) -> Result<(), String> {
    match desc.kind {
        ParamKind::Bool => {
            if v.is_boolean() {
                Ok(())
            } else {
                Err(format!("{} deve ser booleano", key))
            }
        }
        ParamKind::Number { min, max, .. } => {
            let Some(num) = v.as_f64() else {
                return Err(format!("{} deve ser número", key));
            };
            if !num.is_finite() || num < (min as f64) - 1e-4 || num > (max as f64) + 1e-4 {
                return Err(format!("{} fora da faixa {}..{}", key, min, max));
            }
            Ok(())
        }
        ParamKind::Enum(_) => {
            if (desc.to_dsp)(v).is_some() {
                Ok(())
            } else if v.is_string() || (desc.key == "dly_1.mode" && v.is_u64()) || (desc.key.ends_with(".angle_deg") && v.is_number()) {
                Err(format!("{} inválido", key))
            } else {
                Err(format!("{} deve ser string", key))
            }
        }
    }
}

pub fn state_to_json(st: &AppState) -> Value {
    let mut val = serde_json::to_value(st).unwrap_or_else(|_| Value::Object(serde_json::Map::new()));
    if let Value::Object(ref mut map) = val {
        for desc in PARAMS {
            map.insert(desc.key.to_string(), (desc.read_state)(st));
        }
    }
    val
}

pub fn export_preset_params(st: &AppState) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    for desc in PARAMS {
        if !is_amp_param_valid_for_model(&st.amp_model, desc.key) {
            continue;
        }
        if desc.key == KEY_AMP_1_CHANNEL_MODE && st.amp_model != MODEL_DUAL_RECT {
            continue;
        }
        map.insert(desc.key.to_string(), (desc.read_state)(st));
    }
    if let Ok(arr) = serde_json::to_value(st.eq_1_bands_db) {
        map.insert(KEY_EQ_1_BANDS_DB.to_string(), arr);
    }
    map
}

pub fn app_state_to_dsp_updates(st: &AppState) -> Vec<DspParamUpdate> {
    let mut updates = Vec::with_capacity(PARAMS.len());
    for desc in PARAMS {
        if desc.key == KEY_LOOPER_ACTION {
            continue;
        }
        let val = (desc.read_state)(st);
        if let Some(u) = (desc.to_dsp)(&val) {
            updates.push(u);
        }
    }
    updates
}

fn json_error_400(msg: &str) -> (u16, &'static str, Vec<u8>) {
    let body = serde_json::json!({ "error": msg });
    (400, "application/json", body.to_string().into_bytes())
}

fn is_legacy_preset_only_key(key: &str) -> bool {
    matches!(
        key,
        "is_modded"
            | "amp_1.is_modded"
            | "preset.active"
            | "preset.name"
            | "looper.reverse"
            | "looper.half_speed"
            | "modifiers"
            | "eq_1.bands_db"
            | "split.balance"
            | "merge.level_a_db"
            | "merge.level_b_db"
            | "merge.phase_invert_b"
            | "dly_1.time_ms"
            | "dly_1.feedback"
            | "dly_1.mix_pct"
    )
}

fn legacy_preset_kv_to_dsp(key: &str, v: &Value) -> Option<DspParamUpdate> {
    match key {
        "dly_1.time_ms" => val_f32(v).map(DspParamUpdate::Dly1TimeMs),
        "dly_1.feedback" => val_f32(v).map(DspParamUpdate::Dly1Feedback),
        "dly_1.mix_pct" => val_f32(v).map(DspParamUpdate::Dly1Mix),
        _ => None,
    }
}

/// Valida e aplica parâmetros de forma atômica em `AppState` e na fila SPSC/cadeia DSP.
pub fn handle_post_params(
    body: &[u8],
    app_state_clone: &Arc<Mutex<AppState>>,
    _dsp_chain_clone: &Option<Arc<Mutex<RealtimeDspChain>>>,
    param_queue_clone: &Option<Arc<LockFreeParamQueue>>,
) -> (u16, &'static str, Vec<u8>) {
    if body.len() > 4096 {
        return (413, "text/plain", b"Payload Too Large".to_vec());
    }

    let body_str = String::from_utf8_lossy(body);
    let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&body_str) else {
        return (400, "application/json", b"{\"error\":\"Corpo deve ser objeto JSON\"}".to_vec());
    };

    if map.len() > 256 {
        return (
            400,
            "application/json",
            br#"{"error":"Patch /api/params excede limite de 256 chaves"}"#.to_vec(),
        );
    }

    let ir_active = map
        .get(KEY_CAB_1_IR_ENABLED)
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| {
            app_state_clone
                .lock()
                .map(|st| st.cab_ir_enabled)
                .unwrap_or(false)
        });

    let active_model = if let Some(m_val) = map.get(KEY_AMP_1_MODEL) {
        let Some(m_str) = m_val.as_str() else {
            return (400, "application/json", b"{\"error\":\"amp_1.model deve ser string\"}".to_vec());
        };
        if !AMP_MODELS_ORDER.contains(&m_str) {
            return (400, "application/json", b"{\"error\":\"Modelo desconhecido\"}".to_vec());
        }
        m_str.to_string()
    } else {
        app_state_clone
            .lock()
            .map(|st| st.amp_model.clone())
            .unwrap_or_else(|_| MODEL_JCM800.to_string())
    };

    let mut applied: Vec<String> = Vec::new();
    let mut ignored: Vec<String> = Vec::new();
    let mut ignored_reasons = serde_json::Map::new();
    let mut matched: Vec<(usize, &'static ParamDesc, &Value)> = Vec::new();
    #[cfg(feature = "fault-injection")]
    let mut inject_panic = false;

    for (k, v) in &map {
        if k.starts_with(PREFIX_CAB_1_MIC) && ir_active {
            ignored.push(k.clone());
            ignored_reasons.insert(k.clone(), Value::String("ir_active".to_string()));
            continue;
        }

        if let Some((idx, desc)) = find_param_with_index(k.as_str()) {
            if let Err(err) = validate_param_value(desc, k.as_str(), v) {
                return json_error_400(&format!("E_VALIDATION: {}", err));
            }
            if !is_amp_param_valid_for_model(&active_model, desc.key) {
                return json_error_400(&format!(
                    "E_VALIDATION: {} não existe no modelo {}",
                    k, active_model
                ));
            }
            applied.push(k.clone());
            matched.push((idx, desc, v));
            continue;
        }

        if k.ends_with(".channel") {
            let Some(s) = v.as_str() else {
                return json_error_400(&format!("E_VALIDATION: {} deve ser string", k));
            };
            if ["A", "B", "C", "D"].contains(&s) {
                applied.push(k.clone());
                continue;
            }
            return json_error_400(&format!("E_VALIDATION: {} fora de A..D", k));
        }

        #[cfg(feature = "fault-injection")]
        if k == "debug.inject_panic" {
            if let Some(b) = v.as_bool() {
                applied.push(k.clone());
                inject_panic = b;
                continue;
            }
            return json_error_400("E_VALIDATION: debug.inject_panic deve ser booleano");
        }

        ignored.push(k.clone());
    }

    // Ordena conforme PARAMS: amp_1.model antes do chassi/painel, cab_1.speaker antes de resonance_hz.
    matched.sort_by_key(|(idx, _, _)| *idx);

    let mut updates = Vec::with_capacity(matched.len() + 2);
    for (_, desc, v) in &matched {
        if let Some(u) = (desc.to_dsp)(v) {
            updates.push(u);
        }
    }
    #[cfg(feature = "fault-injection")]
    if inject_panic {
        updates.push(DspParamUpdate::DebugInjectPanic);
    }

    if let Some(ref pq) = param_queue_clone {
        if !updates.is_empty() && pq.push_batch(&updates).is_err() {
            return (
                503,
                "application/json",
                br#"{"error":"queue_full","retry_after_ms":5}"#.to_vec(),
            );
        }
    }

    if let Ok(mut st) = app_state_clone.lock() {
        if let Some(m_val) = map.get(KEY_AMP_1_MODEL).and_then(|v| v.as_str()) {
            st.apply_nominal_chassis_for_model(m_val);
        }

        for (_, desc, v) in matched {
            if desc.key == KEY_AMP_1_MODEL {
                continue;
            }
            (desc.write_state)(&mut st, v);
            if desc.key == KEY_CAB_1_SPEAKER {
                if let Some(s) = v.as_str() {
                    st.cab_resonance_hz = nominal_resonance_for_speaker(s);
                }
            }
        }

        st.is_modded = st.check_is_modded();
    }

    let mut resp_json = serde_json::json!({
        "status": "ok",
        "applied": applied,
        "ignored": ignored
    });
    if !ignored_reasons.is_empty() {
        resp_json["ignored_reasons"] = Value::Object(ignored_reasons);
    }
    (200, "application/json", resp_json.to_string().into_bytes())
}

/// Aplica um mapa completo de parâmetros de preset em `AppState` e envia ao motor via fila SPSC.
pub fn apply_preset_params_queued(
    params: &HashMap<String, Value>,
    app_state: &Arc<Mutex<AppState>>,
    param_queue: &Option<Arc<LockFreeParamQueue>>,
    _dsp_chain: &Option<Arc<Mutex<RealtimeDspChain>>>,
) -> Result<Vec<String>, PresetApplyError> {
    let target_model = params
        .get(KEY_AMP_1_MODEL)
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| {
            app_state
                .lock()
                .map(|st| st.amp_model.clone())
                .unwrap_or_else(|_| MODEL_JCM800.to_string())
        });

    let mut ignored_keys: Vec<String> = Vec::new();
    let mut matched: Vec<(usize, &'static ParamDesc, &Value)> = Vec::with_capacity(params.len());
    let mut legacy: Vec<(&String, &Value)> = Vec::new();
    let mut fallback_recto_channel_mode = false;

    for (k, v) in params {
        let key = k.as_str();
        if matches!(key, "amp_1.overdrive" | "amp_1.depth")
            || !is_amp_param_valid_for_model(&target_model, key)
            || (key == KEY_AMP_1_CHANNEL_MODE && target_model != MODEL_DUAL_RECT)
        {
            ignored_keys.push(k.clone());
            continue;
        }
        if key == KEY_AMP_1_CHANNEL_MODE
            && target_model == MODEL_DUAL_RECT
            && v.as_str().map(|s| s.eq_ignore_ascii_case("pushed")).unwrap_or(false)
        {
            ignored_keys.push(k.clone());
            fallback_recto_channel_mode = true;
            continue;
        }

        if let Some((idx, desc)) = find_param_with_index(key) {
            validate_preset_param_kv_desc(desc, key, v).map_err(PresetApplyError::Validation)?;
            matched.push((idx, desc, v));
            continue;
        }

        if is_legacy_preset_only_key(key) {
            legacy.push((k, v));
            continue;
        }

        return Err(PresetApplyError::Validation(format!(
            "Chave de parâmetro desconhecida: {}",
            key
        )));
    }
    ignored_keys.sort();
    matched.sort_by_key(|(idx, _, _)| *idx);

    let mut updates = Vec::with_capacity(matched.len() + 12);
    updates.push(DspParamUpdate::BeginPresetCrossfade);
    for (_, desc, v) in &matched {
        if let Some(u) = (desc.to_dsp)(v) {
            updates.push(u);
        }
    }
    if fallback_recto_channel_mode {
        updates.push(DspParamUpdate::AmpChannelMode(AmpChannelMode::Modern));
    }
    for (k, v) in &legacy {
        if k.as_str() == KEY_EQ_1_BANDS_DB {
            if let Some(arr) = v.as_array() {
                for (i, bv) in arr.iter().take(10).enumerate() {
                    if let Some(g) = val_f32(bv) {
                        let explicit_band = params.keys().any(|pk| {
                            pk.starts_with(PREFIX_EQ_1_BAND) && pk.contains(&format!("{}.gain_db", i + 1))
                        });
                        if !explicit_band {
                            updates.push(DspParamUpdate::Eq1BandGain(i, g));
                        }
                    }
                }
            }
        } else if let Some(u) = legacy_preset_kv_to_dsp(k.as_str(), v) {
            updates.push(u);
        }
    }

    if let Some(ref pq) = param_queue {
        pq.push_batch(&updates).map_err(|_| PresetApplyError::QueueFull)?;
    }

    if let Ok(mut st) = app_state.lock() {
        for (_, desc, v) in matched {
            (desc.write_state)(&mut st, v);
        }
        if fallback_recto_channel_mode {
            st.amp_channel_mode = "modern".to_string();
        }
        for (k, v) in legacy {
            if k.as_str() == KEY_EQ_1_BANDS_DB {
                if let Some(arr) = v.as_array() {
                    for (i, bv) in arr.iter().take(10).enumerate() {
                        if let Some(g) = val_f32(bv) {
                            st.eq_1_bands_db[i] = g;
                        }
                    }
                }
            }
        }
        st.is_modded = st.check_is_modded();
    }

    Ok(ignored_keys)
}

fn validate_preset_param_kv_desc(desc: &ParamDesc, key: &str, v: &Value) -> Result<(), String> {
    if desc.key == KEY_AMP_1_MODEL || desc.key == KEY_SCENE_ACTIVE {
        return validate_param_value(desc, key, v);
    }
    // Em presets antigos, chaves não-críticas preservam compatibilidade de carga tolerando aliases
    // mas mantendo rejeição explícita para tipos/faixas absurdos no modelo e na cena.
    let _ = desc;
    let _ = key;
    let _ = v;
    Ok(())
}

/// Aplica um mapa completo de parâmetros de preset em `AppState` e no `RealtimeDspChain`.
pub fn apply_preset_params(
    params: &HashMap<String, Value>,
    app_state: &Arc<Mutex<AppState>>,
    dsp_chain: &Option<Arc<Mutex<RealtimeDspChain>>>,
) {
    let pq = dsp_chain
        .as_ref()
        .and_then(|d| d.lock().ok().map(|chain| chain.param_queue()));
    let _ = apply_preset_params_queued(params, app_state, &pq, dsp_chain);
}

// ── Gerador TypeScript (`params.generated.ts`) ───────────────────────────────────

fn format_ts_f32(n: f32) -> String {
    let clean = if n.abs() < 1e-6 { 0.0 } else { n };
    let rounded = (clean as f64 * 10000.0).round() / 10000.0;
    if (rounded - rounded.round()).abs() < 1e-6 {
        format!("{}", rounded.round() as i64)
    } else {
        let s = format!("{:.4}", rounded);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn escape_ts_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

fn models_ts_suffix(models: Option<&'static [AmpModelId]>) -> String {
    let Some(list) = models else {
        return String::new();
    };
    let joined: Vec<String> = list
        .iter()
        .map(|m| format!("'{}'", escape_ts_str(m.as_str())))
        .collect();
    format!(", models: [{}]", joined.join(", "))
}

pub fn render_params_ts() -> String {
    let mut out = String::with_capacity(16 * 1024);
    out.push_str("// Arquivo gerado automaticamente por `cargo run -p sonix-desktop --example gerar_params_ts` (AUD-09).\n");
    out.push_str("// NÃO editar manualmente. Fonte de verdade: `crates/sonix-desktop/src/params.rs`.\n\n");
    out.push_str("export const ALL_AMP_MODELS = [\n");
    for m in AMP_MODELS_ORDER {
        out.push_str(&format!("  '{}',\n", m));
    }
    out.push_str("] as const;\n\n");
    out.push_str("export type AmpModelId = (typeof ALL_AMP_MODELS)[number];\n\n");
    out.push_str("export interface NumberParamSchema {\n");
    out.push_str("  readonly type: 'number';\n");
    out.push_str("  readonly range: readonly [number, number];\n");
    out.push_str("  readonly step: number;\n");
    out.push_str("  readonly defaultValue: number;\n");
    out.push_str("  readonly models?: readonly AmpModelId[];\n");
    out.push_str("}\n\n");
    out.push_str("export interface BoolParamSchema {\n");
    out.push_str("  readonly type: 'boolean';\n");
    out.push_str("  readonly defaultValue: boolean;\n");
    out.push_str("  readonly models?: readonly AmpModelId[];\n");
    out.push_str("}\n\n");
    out.push_str("export interface EnumParamSchema {\n");
    out.push_str("  readonly type: 'enum';\n");
    out.push_str("  readonly options: readonly string[];\n");
    out.push_str("  readonly defaultValue: string;\n");
    out.push_str("  readonly models?: readonly AmpModelId[];\n");
    out.push_str("}\n\n");
    out.push_str("export type ParamSchemaEntry = NumberParamSchema | BoolParamSchema | EnumParamSchema;\n\n");
    out.push_str("export const PARAMS_SCHEMA = {\n");

    for desc in PARAMS {
        let models_part = models_ts_suffix(desc.models);
        match (desc.kind, desc.default) {
            (ParamKind::Number { min, max, step }, ParamDefault::Number(def)) => {
                out.push_str(&format!(
                    "  '{}': {{ type: 'number', range: [{}, {}], step: {}, defaultValue: {}{} }},\n",
                    escape_ts_str(desc.key),
                    format_ts_f32(min),
                    format_ts_f32(max),
                    format_ts_f32(step),
                    format_ts_f32(def),
                    models_part,
                ));
            }
            (ParamKind::Bool, ParamDefault::Bool(def)) => {
                out.push_str(&format!(
                    "  '{}': {{ type: 'boolean', defaultValue: {}{} }},\n",
                    escape_ts_str(desc.key),
                    if def { "true" } else { "false" },
                    models_part,
                ));
            }
            (ParamKind::Enum(opts), ParamDefault::Enum(def)) => {
                let opts_joined: Vec<String> = opts
                    .iter()
                    .map(|o| format!("'{}'", escape_ts_str(o)))
                    .collect();
                out.push_str(&format!(
                    "  '{}': {{ type: 'enum', options: [{}], defaultValue: '{}'{} }},\n",
                    escape_ts_str(desc.key),
                    opts_joined.join(", "),
                    escape_ts_str(def),
                    models_part,
                ));
            }
            _ => {}
        }
    }

    out.push_str("} as const;\n\n");
    out.push_str("export type BackendParamKey = keyof typeof PARAMS_SCHEMA;\n");
    out
}
