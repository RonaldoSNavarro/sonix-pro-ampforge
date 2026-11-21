//! crates/sonix-desktop/src/api.rs
//! Camada pura de roteamento e despacho de comandos/telemetria da API (independente de HTTP).
//! Compartilhada entre comandos Tauri (IPC nativo) e o servidor HTTP de desenvolvimento (`dev-http`).

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use ampforge_dsp::cab::{MicModel, SpeakerCabModel};
use ampforge_dsp::fx::drive::DriveModel;
use ampforge_dsp::fx::eq::EqMode;
use ampforge_dsp::fx::wah::WahMode;
use ampforge_dsp::looper::LooperPosition;
use ampforge_dsp::modshop::{PowerTube, PreampTube, RectifierType, ToneStackTopology};
use ampforge_dsp::tuner::{split_note_and_octave, TunerPitchResult, TunerProcessor};

use crate::audio::{
    AudioDeviceEnumerator, AudioMode, AudioStatusTelemetry, ChainSplitMode, CleanInputRing,
    CpalAudioEnumerator, DesktopAudioHost, DspParamUpdate, LockFreeParamQueue,
    LockFreeResourceQueue, LooperActionCmd, PreparedDspResource, RealtimeDspChain,
    RealtimeDuplexEngine, TelemetryBuffer,
};
use crate::calibration::{calibrate_input, CalibrationStore};

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

fn parse_mod_model(s: &str) -> Option<ampforge_dsp::fx::modulation::ModModel> {
    use ampforge_dsp::fx::modulation::ModModel;
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

fn parse_pitch_model(s: &str) -> Option<ampforge_dsp::fx::pitch::PitchModel> {
    use ampforge_dsp::fx::pitch::PitchModel;
    match s {
        "PedalWhammy" | "whammy" => Some(PitchModel::PedalWhammy),
        "PolyphonicDropCapo" | "drop_capo" => Some(PitchModel::PolyphonicDropCapo),
        "IntelligentDualHarmonizer" | "harmonizer" => Some(PitchModel::IntelligentDualHarmonizer),
        "MicroPitchDetune" | "micro_detune" => Some(PitchModel::MicroPitchDetune),
        _ => None,
    }
}

fn parse_pitch_key(s: &str) -> Option<ampforge_dsp::fx::pitch::PitchKey> {
    use ampforge_dsp::fx::pitch::PitchKey;
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

fn parse_pitch_scale(s: &str) -> Option<ampforge_dsp::fx::pitch::ScaleType> {
    use ampforge_dsp::fx::pitch::ScaleType;
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

fn default_input_mode() -> String {
    "instrument".to_string()
}

/// Estado persistente e observável da cadeia e interface (tabela única de parâmetros da Spec 014).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppState {
    #[serde(rename = "amp_1.gain")]
    pub amp_gain: f32,
    #[serde(rename = "amp_1.overdrive")]
    pub amp_overdrive: f32,
    #[serde(rename = "amp_1.bass")]
    pub amp_bass: f32,
    #[serde(rename = "amp_1.middle")]
    pub amp_middle: f32,
    #[serde(rename = "amp_1.treble")]
    pub amp_treble: f32,
    #[serde(rename = "amp_1.presence")]
    pub amp_presence: f32,
    #[serde(rename = "amp_1.depth")]
    pub amp_depth: f32,
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
    // Parâmetros de Bancada / Mod Shop (Spec 021, CA-9 & CA-10)
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
    // Spec 024: Efeitos e Caminho Paralelo
    #[serde(default, rename = "split.enabled")]
    pub split_enabled: bool,
    #[serde(default = "default_split_mode", rename = "split.mode")]
    pub split_mode: String,
    #[serde(default = "default_crossover_hz", rename = "split.crossover_hz")]
    pub split_crossover_hz: f32,
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
    pub channels: std::collections::HashMap<
        String,
        std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>>,
    >,
    #[serde(skip, default = "default_dag_topology")]
    pub dag_topology: crate::audio::CompiledDagTopology,
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

fn default_curve_slope() -> f32 {
    0.5
}

fn default_footswitch_mode() -> String {
    "HYBRID_STOMP_SCENE".to_string()
}

fn default_modifiers() -> Vec<ModifierBindingDto> {
    crate::midi::DesktopMidiHost::default_midi_modifiers()
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SetlistDto {
    pub id: String,
    pub name: String,
    pub presets: Vec<String>,
}

fn default_dag_topology() -> crate::audio::CompiledDagTopology {
    crate::audio::CompiledDagTopology::default_layout(1)
}

fn default_scenes() -> Vec<sonix_core::persistence::SceneDto> {
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

fn default_grid() -> sonix_core::persistence::GridDto {
    sonix_core::persistence::GridDto {
        rows: 4,
        cols: 12,
        blocks: Vec::new(),
    }
}

/// Retorna os valores nominais de chassi para cada modelo de amplificador.
pub fn nominal_chassis_for_model(model: &str) -> (&'static str, &'static str, f32, f32, &'static str, f32, f32, f32, f32, &'static str) {
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
}

fn default_amp_jack() -> String {
    "high".to_string()
}

fn default_amp_channel_mode() -> String {
    "modern".to_string()
}

fn default_amp_impedance() -> String {
    "16".to_string()
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            amp_gain: 8.0,
            amp_overdrive: 5.0,
            amp_bass: 5.0,
            amp_middle: 5.0,
            amp_treble: 5.0,
            amp_presence: 5.0,
            amp_depth: 5.0,
            amp_master: 7.0,
            amp_volume_2: 0.0,
            amp_cut: 5.0,
            amp_model: "jcm800_2203".to_string(),
            engine_oversampling: "2x".to_string(),
            input_boost: 2.5,
            gate_threshold_db: -65.0,
            input_source: "device".to_string(),
            input_mode: "instrument".to_string(),
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
            channels: std::collections::HashMap::new(),
            dag_topology: default_dag_topology(),
            footswitch_mode: default_footswitch_mode(),
            modifiers: default_modifiers(),
            setlist_active_index: 0,
        }
    }
}

/// Estado de sessão mutável (calibração e scratch do afinador fora da thread de áudio).
pub struct CalibrationSession {
    pub noise_samples: Vec<f32>,
    pub cursor_pos: u64,
    pub tuner_enabled_at: u64,
    pub tuner_processor: TunerProcessor,
    pub tuner_scratch: Vec<f32>,
}

impl Default for CalibrationSession {
    fn default() -> Self {
        Self {
            noise_samples: Vec::new(),
            cursor_pos: 0,
            tuner_enabled_at: 0,
            tuner_processor: TunerProcessor::new(),
            tuner_scratch: vec![0.0; 4096],
        }
    }
}

/// Contexto compartilhado de estado, telemetria e motor DSP para o roteador puro da API.
#[derive(Clone)]
pub struct ApiContext {
    pub audio_telemetry: Arc<Mutex<Option<AudioStatusTelemetry>>>,
    pub app_state: Arc<Mutex<AppState>>,
    pub telemetry: Arc<TelemetryBuffer>,
    pub dsp_chain: Option<Arc<Mutex<RealtimeDspChain>>>,
    pub param_queue: Option<Arc<LockFreeParamQueue>>,
    pub resource_queue: Option<Arc<LockFreeResourceQueue>>,
    pub shared_adc_clips: Option<Arc<AtomicU64>>,
    pub shared_sample_rate_bits: Option<Arc<AtomicU32>>,
    pub shared_algo_latency_samples: Option<Arc<AtomicU32>>,
    pub shared_b_plus_v_bits: Option<Arc<AtomicU32>>,
    pub clean_input_ring: Option<Arc<CleanInputRing>>,
    pub scope_output_ring: Option<Arc<CleanInputRing>>,
    pub calibration_store: Option<CalibrationStore>,
    pub active_device_id: Option<String>,
    pub calibration_session: Arc<Mutex<CalibrationSession>>,
    pub device_enumerator: Arc<dyn AudioDeviceEnumerator>,
    pub duplex_engine: Option<Arc<RealtimeDuplexEngine>>,
    pub custom_presets_dir: Option<std::path::PathBuf>,
    pub data_dir: Option<std::path::PathBuf>,
}

/// Tabela canônica de rotas da API local (`(METHOD, PATH)`).
/// Toda requisição é despachada exclusivamente por esta tabela (AUD-01).
pub static ROUTES: &[(&str, &str)] = &[
    ("GET", "/api/status"),
    ("GET", "/api/audio/status"),
    ("GET", "/api/telemetry"),
    ("GET", "/api/devices"),
    ("GET", "/api/state"),
    ("GET", "/api/tuner"),
    ("GET", "/api/graph"),
    ("POST", "/api/graph"),
    ("GET", "/api/scope"),
    ("POST", "/api/params"),
    ("POST", "/api/input/di"),
    ("POST", "/api/input/calibrate"),
    ("POST", "/api/cab/ir"),
    ("GET", "/api/presets"),
    ("POST", "/api/presets/load"),
    ("POST", "/api/presets/save"),
    ("POST", "/api/presets/import"),
    ("GET", "/api/presets/*/export"),
    ("GET", "/api/setlist"),
    ("POST", "/api/setlist"),
    ("POST", "/api/setlist/next"),
    ("POST", "/api/setlist/prev"),
    ("GET", "/api/modifiers"),
    ("POST", "/api/modifiers"),
    ("DELETE", "/api/modifiers"),
    ("GET", "/api/footswitch"),
    ("POST", "/api/footswitch/press"),
    ("POST", "/api/scenes/select"),
    ("POST", "/api/reamp/run"),
];

pub fn route_matches(pattern: &str, path: &str) -> bool {
    if pattern == path {
        return true;
    }
    if pattern == "/api/presets/*/export" {
        return path.starts_with("/api/presets/")
            && path.ends_with("/export")
            && path.len() > "/api/presets//export".len();
    }
    false
}

impl ApiContext {
    pub fn new(
        audio_telemetry: Arc<Mutex<Option<AudioStatusTelemetry>>>,
        app_state: Arc<Mutex<AppState>>,
        telemetry: Arc<TelemetryBuffer>,
        dsp_chain: Option<Arc<Mutex<RealtimeDspChain>>>,
        param_queue: Option<Arc<LockFreeParamQueue>>,
        device_id: Option<&str>,
        calibration_store: Option<CalibrationStore>,
    ) -> Self {
        let mut resource_queue = None;
        let mut shared_adc_clips = None;
        let mut shared_sample_rate_bits = None;
        let mut shared_algo_latency_samples = None;
        let mut shared_b_plus_v_bits = None;
        let mut clean_input_ring = None;
        let mut scope_output_ring = None;

        if let Some(ref d) = dsp_chain {
            if let Ok(mut chain) = d.lock() {
                if let Some(ref pq) = param_queue {
                    chain.set_param_queue(pq.clone());
                }
                resource_queue = Some(chain.resource_queue());
                shared_adc_clips = Some(chain.adc_clips_handle());
                shared_sample_rate_bits = Some(chain.shared_sample_rate_bits());
                shared_algo_latency_samples = Some(chain.shared_algo_latency_samples());
                shared_b_plus_v_bits = Some(chain.shared_b_plus_v_bits());
                clean_input_ring = Some(chain.clean_input_ring());
                scope_output_ring = Some(chain.scope_output_ring());
            }
        }

        Self {
            audio_telemetry,
            app_state,
            telemetry,
            dsp_chain,
            param_queue,
            resource_queue,
            shared_adc_clips,
            shared_sample_rate_bits,
            shared_algo_latency_samples,
            shared_b_plus_v_bits,
            clean_input_ring,
            scope_output_ring,
            calibration_store,
            active_device_id: device_id.map(|s| s.to_string()),
            calibration_session: Arc::new(Mutex::new(CalibrationSession::default())),
            device_enumerator: Arc::new(CpalAudioEnumerator::new()),
            duplex_engine: None,
            custom_presets_dir: None,
            data_dir: None,
        }
    }

    pub fn set_presets_dir(&mut self, dir: std::path::PathBuf) {
        self.custom_presets_dir = Some(dir);
    }

    pub fn set_data_dir(&mut self, dir: std::path::PathBuf) {
        self.data_dir = Some(dir);
    }
}

fn json_response<T: serde::Serialize>(status: u16, value: &T) -> (u16, &'static str, Vec<u8>) {
    match serde_json::to_vec(value) {
        Ok(bytes) => (status, "application/json", bytes),
        Err(e) => (
            500,
            "application/json",
            format!("{{\"error\":\"E_INTERNAL: {}\"}}", e).into_bytes(),
        ),
    }
}

fn json_error_response(status: u16, msg: &str) -> (u16, &'static str, Vec<u8>) {
    let payload = serde_json::json!({ "error": msg });
    match serde_json::to_vec(&payload) {
        Ok(bytes) => (status, "application/json", bytes),
        Err(_) => (
            500,
            "application/json",
            br#"{"error":"E_INTERNAL: serialization failed"}"#.to_vec(),
        ),
    }
}

pub fn compile_modifiers_table(modifiers: &[ModifierBindingDto]) -> crate::audio::CompiledModifierTable {
    let mut table = crate::audio::CompiledModifierTable::empty();
    for m in modifiers.iter().take(16) {
        if let (Some((target, _, _)), Some(source)) = (
            crate::audio::ModifierTargetParam::from_str_name(&m.target_param),
            crate::audio::CompiledModifierSource::from_str_name(&m.source),
        ) {
            let idx = table.len;
            table.bindings[idx] = crate::audio::CompiledModifierBinding {
                active: true,
                target,
                source,
                min_val: m.min_val,
                max_val: m.max_val,
                curve_slope: m.curve_slope,
                damping_ms: m.damping_ms,
            };
            table.len += 1;
        }
    }
    table
}

pub fn app_state_to_dsp_updates(st: &AppState) -> Vec<DspParamUpdate> {
    let mut updates = Vec::new();
    if let Some(m) = ampforge_dsp::amp::find_model_by_name(&st.amp_model) {
        updates.push(DspParamUpdate::AmpModel(m));
    }
    if let Some(m) = ampforge_dsp::amp::find_model_by_name(&st.amp_2_model) {
        updates.push(DspParamUpdate::Amp2Model(m));
    }
    if let Some(spk) = parse_speaker_model(&st.cab_speaker) {
        updates.push(DspParamUpdate::Cab1Speaker(spk));
    }
    if let Some(spk) = parse_speaker_model(&st.cab_2_speaker) {
        updates.push(DspParamUpdate::Cab2Speaker(spk));
    }
    if let Ok(serde_json::Value::Object(map)) = serde_json::to_value(st) {
        for (k, v) in &map {
            if matches!(
                k.as_str(),
                "amp_1.model" | "amp_2.model" | "cab_1.speaker" | "cab_2.speaker" | "looper.action"
            ) {
                continue;
            }
            json_kv_to_dsp_updates(k.as_str(), v, &mut updates);
        }
    }
    for (idx, &gain_db) in st.eq_1_bands_db.iter().enumerate() {
        updates.push(DspParamUpdate::Eq1BandGain(idx, gain_db));
    }
    updates
}

/// Despachante puro da API (sem dependência de socket HTTP).
/// Retorna `(status_http, content_type, body_bytes)`.
pub fn dispatch(
    ctx: &ApiContext,
    route_with_query: &str,
    method: &str,
    body: &[u8],
) -> (u16, &'static str, Vec<u8>) {
    let path = route_with_query.split('?').next().unwrap_or("/");
    let matched_route = ROUTES
        .iter()
        .find(|(m, rp)| *m == method && route_matches(rp, path))
        .map(|(_, rp)| *rp);
    let Some(matched_path) = matched_route else {
        if ROUTES.iter().any(|(_, rp)| route_matches(rp, path)) {
            return (405, "text/plain", b"Method Not Allowed".to_vec());
        }
        return (404, "text/plain", b"Not Found".to_vec());
    };

    match matched_path {
        "/api/status" => {
            let backend = ctx
                .audio_telemetry
                .lock()
                .ok()
                .and_then(|g| g.as_ref().map(|t| t.backend.clone()))
                .unwrap_or_else(|| "wasapi".to_string());
            let body = serde_json::json!({
                "status": "online",
                "engine": "AmpForge RealTime",
                "version": "1.0.0",
                "backend": backend
            });
            json_response(200, &body)
        }
        "/api/audio/status" => {
            let clips = ctx
                .shared_adc_clips
                .as_ref()
                .map(|c| c.load(Ordering::Relaxed))
                .unwrap_or(0);
            let algo_lat = ctx
                .shared_algo_latency_samples
                .as_ref()
                .map(|a| a.load(Ordering::Relaxed))
                .unwrap_or_else(|| {
                    ctx.app_state
                        .lock()
                        .ok()
                        .map(|st| match st.engine_oversampling.as_str() {
                            "2x" => 4,
                            "4x" => 8,
                            _ => 16,
                        })
                        .unwrap_or(4)
                });
            let contention_silences = ctx.telemetry.dsp_contention_silence_blocks();
            let live_underruns = ctx.duplex_engine.as_ref().map(|d| d.underruns());
            let live_overruns = ctx.duplex_engine.as_ref().map(|d| d.overruns());
            let host = DesktopAudioHost::new();
            let caps = serde_json::json!({
                "wasapi_shared": host.has_mode_support(AudioMode::WasapiShared),
                "wasapi_shared_low_latency": host.has_mode_support(AudioMode::WasapiSharedLowLatency),
                "wasapi_exclusive": host.has_mode_support(AudioMode::WasapiExclusive),
                "asio": host.has_mode_support(AudioMode::Asio),
            });

            let mut val = if let Ok(mut guard) = ctx.audio_telemetry.lock() {
                if let Some(ref mut t) = *guard {
                    let prev_algo = t.algorithmic_latency_samples;
                    t.adc_clips = clips;
                    t.algorithmic_latency_samples = algo_lat;
                    if let Some(u) = live_underruns {
                        t.underruns = u;
                    }
                    if let Some(o) = live_overruns {
                        t.overruns = o;
                    }
                    let obs_in = ctx.telemetry.observed_input_period_frames();
                    let obs_out = ctx.telemetry.observed_output_period_frames();
                    let mut period_changed = false;
                    if obs_in > 0 && obs_in != t.input_period_frames {
                        t.input_period_frames = obs_in;
                        t.driver_input_latency_ms =
                            (obs_in as f32 / t.sample_rate.max(1) as f32) * 1000.0;
                        period_changed = true;
                    }
                    if obs_out > 0 && obs_out != t.output_period_frames {
                        t.output_period_frames = obs_out;
                        t.driver_output_latency_ms =
                            (obs_out as f32 / t.sample_rate.max(1) as f32) * 1000.0;
                        period_changed = true;
                    }
                    if t.estimated_roundtrip_ms <= 0.0 || period_changed {
                        let algo_ms = (algo_lat as f32 / t.sample_rate.max(1) as f32) * 1000.0;
                        let duplex_ms = (t.output_period_frames as f32 / t.sample_rate.max(1) as f32) * 1000.0;
                        t.estimated_roundtrip_ms =
                            t.driver_input_latency_ms + t.driver_output_latency_ms + duplex_ms + algo_ms;
                    } else if prev_algo != algo_lat {
                        let delta_ms =
                            ((algo_lat as f32 - prev_algo as f32) / t.sample_rate.max(1) as f32)
                                * 1000.0;
                        t.estimated_roundtrip_ms = (t.estimated_roundtrip_ms + delta_ms).max(0.0);
                    }
                    let mut v = serde_json::to_value(&*t).unwrap_or_else(|_| serde_json::json!({}));
                    if let Some(obj) = v.as_object_mut() {
                        for key in [
                            "cpu_avg_pct",
                            "cpu_peak_pct",
                            "driver_input_latency_ms",
                            "driver_output_latency_ms",
                            "estimated_roundtrip_ms",
                        ] {
                            if let Some(num) = obj.get(key).and_then(|x| x.as_f64()) {
                                let rounded = (num * 1000.0).round() / 1000.0;
                                obj.insert(key.to_string(), serde_json::json!(rounded));
                            }
                        }
                    }
                    v
                } else {
                    let fallback_sr = ctx
                        .shared_sample_rate_bits
                        .as_ref()
                        .map(|b| f32::from_bits(b.load(Ordering::Relaxed)).round() as u32)
                        .filter(|&sr| sr > 0)
                        .unwrap_or(48000);
                    let period = 128u32;
                    let driver_ms = (period as f32 / fallback_sr as f32) * 1000.0;
                    let algo_ms = (algo_lat as f32 / fallback_sr as f32) * 1000.0;
                    serde_json::json!({
                        "backend": "wasapi",
                        "mode": "wasapi_shared",
                        "raw_processing": false,
                        "sample_rate": fallback_sr,
                        "input_period_frames": period,
                        "output_period_frames": period,
                        "input_device": "",
                        "output_device": "",
                        "driver_input_latency_ms": driver_ms,
                        "driver_output_latency_ms": driver_ms,
                        "cpu_avg_pct": 0.0,
                        "cpu_peak_pct": 0.0,
                        "underruns": live_underruns.unwrap_or(0),
                        "overruns": live_overruns.unwrap_or(0),
                        "panics": 0,
                        "adc_clips": clips,
                        "measured_roundtrip_ms": serde_json::Value::Null,
                        "estimated_roundtrip_ms": driver_ms * 3.0 + algo_ms,
                        "algorithmic_latency_samples": algo_lat,
                    })
                }
            } else {
                serde_json::json!({
                    "adc_clips": clips,
                    "underruns": live_underruns.unwrap_or(0),
                    "overruns": live_overruns.unwrap_or(0),
                    "algorithmic_latency_samples": algo_lat,
                    "measured_roundtrip_ms": serde_json::Value::Null,
                })
            };

            if let Some(obj) = val.as_object_mut() {
                let ov_mode = ctx
                    .app_state
                    .lock()
                    .map(|st| st.engine_oversampling.clone())
                    .unwrap_or_else(|_| "2x".to_string());
                obj.insert(
                    "oversampling".to_string(),
                    serde_json::json!(ov_mode),
                );
                obj.insert(
                    "dsp_contention_silence_blocks".to_string(),
                    serde_json::json!(contention_silences),
                );
                obj.insert(
                    "panics".to_string(),
                    serde_json::json!(ctx.telemetry.panic_count()),
                );
                obj.insert(
                    "engine_faulted".to_string(),
                    serde_json::json!(ctx.telemetry.is_engine_faulted()),
                );
                obj.insert(
                    "recovery_count".to_string(),
                    serde_json::json!(ctx.telemetry.recovery_count()),
                );
                obj.insert(
                    "device_state".to_string(),
                    serde_json::json!(ctx.telemetry.device_state_str()),
                );
                obj.insert(
                    "connected".to_string(),
                    serde_json::json!(!ctx.telemetry.is_device_lost()),
                );
                obj.insert("mode_capabilities".to_string(), caps);
            }
            json_response(200, &val)
        }
        "/api/telemetry" => {
            let mut snap = ctx.telemetry.snapshot();
            if let Some(ref b_bits) = ctx.shared_b_plus_v_bits {
                let bv = f32::from_bits(b_bits.load(Ordering::Relaxed));
                if bv.is_finite() && bv > 0.0 {
                    snap.b_plus_v = bv;
                }
            }
            let json = serde_json::to_string(&snap).unwrap_or_else(|_| "{}".to_string());
            (200, "application/json", json.into_bytes())
        }
        "/api/devices" => {
            let devices = ctx.device_enumerator.enumerate_devices();
            let mapped: Vec<serde_json::Value> = devices
                .into_iter()
                .map(|d| {
                    serde_json::json!({
                        "id": d.id,
                        "name": d.name,
                        "backend": format!("{:?}", d.backend).to_ascii_lowercase(),
                        "is_input": d.is_input,
                        "is_output": !d.is_input,
                        "supported_sample_rates": d.supported_sample_rates,
                        "supported_buffer_sizes": d.supported_buffer_sizes,
                    })
                })
                .collect();
            let resp = serde_json::json!({ "devices": mapped });
            json_response(200, &resp)
        }
        "/api/state" => {
            if method == "GET" {
                match ctx.app_state.lock() {
                    Ok(st) => json_response(200, &*st),
                    Err(_) => json_error_response(500, "E_INTERNAL: app_state lock poisoned"),
                }
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/tuner" => {
            if method == "GET" {
                let (is_muted, is_active, ref_hz) = if let Ok(st) = ctx.app_state.lock() {
                    (
                        st.tuner_output == "mute" && st.tuner_enabled,
                        st.tuner_enabled,
                        st.tuner_ref_hz.clamp(430.0, 450.0),
                    )
                } else {
                    (false, false, 440.0)
                };
                let res = if is_active {
                    if let Some(ref ring) = ctx.clean_input_ring {
                        if let Ok(mut session) = ctx.calibration_session.lock() {
                            let sr = ring.sample_rate();
                            session.tuner_processor.set_sample_rate(sr);
                            session.tuner_processor.set_a4_reference_hz(ref_hz);
                            let CalibrationSession {
                                ref mut tuner_processor,
                                ref mut tuner_scratch,
                                ..
                            } = *session;
                            // Lê a janela mais recente do ring buffer para responder mesmo se tuner.enabled foi ligado antes da chegada do áudio
                            let n = ring.read_latest_since(0, tuner_scratch);
                            if n >= 1024 {
                                tuner_processor.detect_pitch(&tuner_scratch[..n])
                            } else {
                                TunerPitchResult {
                                    note_name: "---".to_string(),
                                    cents_deviation: 0.0,
                                    frequency_hz: 0.0,
                                }
                            }
                        } else {
                            TunerPitchResult {
                                note_name: "---".to_string(),
                                cents_deviation: 0.0,
                                frequency_hz: 0.0,
                            }
                        }
                    } else if let Some(ref dsp_ref) = ctx.dsp_chain {
                        if let Ok(mut dsp) = dsp_ref.try_lock() {
                            dsp.tuner_latest_result()
                        } else {
                            TunerPitchResult {
                                note_name: "---".to_string(),
                                cents_deviation: 0.0,
                                frequency_hz: 0.0,
                            }
                        }
                    } else {
                        TunerPitchResult {
                            note_name: "---".to_string(),
                            cents_deviation: 0.0,
                            frequency_hz: 0.0,
                        }
                    }
                } else {
                    TunerPitchResult {
                        note_name: "---".to_string(),
                        cents_deviation: 0.0,
                        frequency_hz: 0.0,
                    }
                };
                let signal_present = res.note_name != "---" && res.frequency_hz > 0.0;
                let (note_letter, octave) = if signal_present {
                    split_note_and_octave(&res.note_name)
                } else {
                    ("---", None)
                };
                let resp = serde_json::json!({
                    "note": if signal_present { res.note_name.as_str() } else { "---" },
                    "note_letter": note_letter,
                    "octave": octave,
                    "frequency_hz": if signal_present { res.frequency_hz } else { 0.0 },
                    "cents": if signal_present { res.cents_deviation } else { 0.0 },
                    "confidence": if signal_present { 1.0 } else { 0.0 },
                    "in_tune": signal_present && res.cents_deviation.abs() <= 1.0,
                    "signal_present": signal_present,
                    "active": is_active,
                    "enabled": is_active,
                    "muted": is_muted,
                    "ref_hz": ref_hz,
                });
                json_response(200, &resp)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/graph" => {
            if method == "GET" {
                let (
                    gate_en,
                    drive_en,
                    drive_model,
                    amp_en,
                    amp_model,
                    cab_en,
                    cab_speaker,
                    split_en,
                    comp_en,
                    eq_en,
                    mod_en,
                    dly_en,
                    rev_en,
                    amp2_en,
                    amp2_model,
                    cab2_en,
                    cab2_speaker,
                    wah_en,
                    dag_topo,
                ) = if let Ok(st) = ctx.app_state.lock() {
                    (
                        st.gate_enabled,
                        st.drive_enabled,
                        st.drive_model.clone(),
                        st.amp_enabled,
                        st.amp_model.clone(),
                        st.cab_enabled,
                        st.cab_speaker.clone(),
                        st.split_enabled,
                        st.comp_2_enabled,
                        st.eq_1_enabled,
                        st.mod_2_enabled,
                        st.dly_1_enabled,
                        st.rev_1_enabled,
                        st.amp_2_enabled,
                        st.amp_2_model.clone(),
                        st.cab_2_enabled,
                        st.cab_2_speaker.clone(),
                        st.wah_1_enabled,
                        st.dag_topology,
                    )
                } else {
                    (
                        true,
                        true,
                        "screamer_808".to_string(),
                        true,
                        "jcm800_2203".to_string(),
                        true,
                        "greenback_4x12".to_string(),
                        false,
                        false,
                        false,
                        false,
                        false,
                        false,
                        false,
                        "twin_ab763".to_string(),
                        false,
                        "alnico_blue_2x12".to_string(),
                        false,
                        default_dag_topology(),
                    )
                };

                let is_block_enabled = |k: crate::audio::DagBlockKind| -> bool {
                    use crate::audio::DagBlockKind::*;
                    match k {
                        Gate => gate_en,
                        Wah1 => wah_en,
                        Pitch1 => false,
                        Drive => drive_en,
                        Split => split_en,
                        Amp1 => amp_en,
                        Cab1 => cab_en,
                        Eq1 => eq_en,
                        Comp2 => comp_en,
                        Amp2 => amp2_en,
                        Cab2 => cab2_en,
                        Mod2 => mod_en,
                        Merge => true,
                        Dly1 => dly_en,
                        Rev1 => rev_en,
                    }
                };
                let slots_json: Vec<serde_json::Value> = (0..48usize)
                    .map(|idx| {
                        let row = idx / 12;
                        let col = idx % 12;
                        let blk = dag_topo.slots[idx];
                        serde_json::json!({
                            "row": row,
                            "col": col,
                            "block": blk.map(|b| b.as_str()),
                            "enabled": blk.map(is_block_enabled).unwrap_or(false),
                        })
                    })
                    .collect();

                let amp_display_name = |m: &str| match m {
                    "jcm800_2203" => "Brit 800 2203",
                    "plexi_1959" => "Plexi 1959",
                    "twin_ab763" => "US Twin Clean",
                    "dual_rect" => "Dual Rectifier",
                    "ac30_tb" => "AC30 Top Boost",
                    _ => "Brit 800 2203",
                };
                let drive_display_name = |m: &str| match m {
                    "screamer_808" => "Screamer 808",
                    "rat_dist" | "rat" => "RAT Distortion",
                    "sd1_overdrive" | "sd1" => "SD-1 Overdrive",
                    "klon_centaur" | "klon" => "Klon Centaur",
                    _ => "Screamer 808",
                };
                let cab_display_name = |s: &str| match s {
                    "greenback_4x12" | "greenback_4x12_closed" => "4x12 Greenback",
                    "v30_4x12" | "v30_4x12_closed" => "4x12 V30",
                    "alnico_blue_2x12" | "alnico_blue_2x12_open" => "2x12 Alnico",
                    "twin_2x12_open" | "twin_2x12" | "jensen_c12n_2x12_open" => "2x12 Jensen",
                    _ => "4x12 Greenback",
                };

                let graph = serde_json::json!({
                    "topology_id": dag_topo.id,
                    "slots": slots_json,
                    "rows": [
                        {
                            "index": 0,
                            "blocks": [
                                {"id": "gate", "col": 0, "name": "Natural Gate", "enabled": gate_en},
                                {"id": "drive", "col": 1, "name": drive_display_name(&drive_model), "enabled": drive_en},
                                {"id": "split", "col": 2, "name": "A/B Crossover", "enabled": split_en},
                                {"id": "amp_1", "col": 3, "name": amp_display_name(&amp_model), "enabled": amp_en},
                                {"id": "cab_1", "col": 4, "name": cab_display_name(&cab_speaker), "enabled": cab_en},
                                {"id": "eq_1", "col": 5, "name": "8-Band Studio", "enabled": eq_en},
                                {"id": "merge", "col": 6, "name": "Mixer Merge", "enabled": true},
                                {"id": "dly_1", "col": 7, "name": "Tape Echo 201", "enabled": dly_en},
                                {"id": "rev_1", "col": 8, "name": "Vintage Plate", "enabled": rev_en}
                            ]
                        },
                        {
                            "index": 1,
                            "blocks": [
                                {"id": "comp_2", "col": 2, "name": "FET 1176 Comp", "enabled": comp_en},
                                {"id": "amp_2", "col": 3, "name": amp_display_name(&amp2_model), "enabled": amp2_en},
                                {"id": "cab_2", "col": 4, "name": cab_display_name(&cab2_speaker), "enabled": cab2_en},
                                {"id": "mod_2", "col": 5, "name": "Tri-Chorus", "enabled": mod_en}
                            ]
                        }
                    ]
                });
                json_response(200, &graph)
            } else if method == "POST" {
                let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) else {
                    return json_error_response(400, "E_VALIDATION: JSON inválido em /api/graph");
                };
                if let Some(conns) = v.get("connections").and_then(|c| c.as_array()) {
                    if conns.len() > 256 {
                        return (400, "application/json", br#"{"error":"Muitas conexoes em /api/graph"}"#.to_vec());
                    }
                    let mut dag = ampforge_dsp::dag::DagGraph::new();
                    for c in conns {
                        let fr = c.get("from").and_then(|x| x.get("row")).and_then(|x| x.as_u64());
                        let fc = c.get("from").and_then(|x| x.get("col")).and_then(|x| x.as_u64());
                        let tr = c.get("to").and_then(|x| x.get("row")).and_then(|x| x.as_u64());
                        let tc = c.get("to").and_then(|x| x.get("col")).and_then(|x| x.as_u64());
                        let (Some(fr), Some(fc), Some(tr), Some(tc)) = (fr, fc, tr, tc) else {
                            return json_error_response(400, "E_VALIDATION: Conexão inválida em /api/graph");
                        };
                        if fr >= 4 || fc >= 12 || tr >= 4 || tc >= 12 {
                            return json_error_response(400, "E_VALIDATION: Coordenada fora da grade 4x12");
                        }
                        let (Ok(from_slot), Ok(to_slot)) = (
                            ampforge_dsp::dag::SlotId::new(fr as usize, fc as usize),
                            ampforge_dsp::dag::SlotId::new(tr as usize, tc as usize),
                        ) else {
                            return json_error_response(400, "E_VALIDATION: Coordenada fora da grade 4x12");
                        };
                        if let Err(err) = dag.add_connection(from_slot, to_slot) {
                            return json_error_response(400, &format!("DagCycleError: {:?}", err));
                        }
                    }
                }
                let mut new_topo_opt = None;
                if let Ok(mut st) = ctx.app_state.lock() {
                    let mut next = st.dag_topology;
                    if let Some(slots) = v.get("slots").and_then(|s| s.as_array()) {
                        if slots.len() > 48 {
                            return (400, "application/json", br#"{"error":"Slots demais em /api/graph"}"#.to_vec());
                        }
                        for item in slots {
                            let Some(r) = item.get("row").and_then(|x| x.as_u64()) else {
                                return (400, "application/json", b"{\"error\":\"row ausente\"}".to_vec());
                            };
                            let Some(c) = item.get("col").and_then(|x| x.as_u64()) else {
                                return (400, "application/json", b"{\"error\":\"col ausente\"}".to_vec());
                            };
                            if r >= 4 || c >= 12 {
                                return (400, "application/json", b"{\"error\":\"Coordenada fora da grade 4x12\"}".to_vec());
                            }
                            let idx = (r as usize) * 12 + (c as usize);
                            match item.get("block") {
                                Some(serde_json::Value::Null) | None => {
                                    next.slots[idx] = None;
                                }
                                Some(serde_json::Value::String(s)) if s.is_empty() => {
                                    next.slots[idx] = None;
                                }
                                Some(serde_json::Value::String(s)) => {
                                    let Some(kind) = crate::audio::DagBlockKind::from_str_name(s) else {
                                        return json_error_response(400, &format!("Bloco desconhecido: {}", s));
                                    };
                                    for slot in next.slots.iter_mut() {
                                        if *slot == Some(kind) {
                                            *slot = None;
                                        }
                                    }
                                    next.slots[idx] = Some(kind);
                                }
                                Some(_) => {
                                    return (400, "application/json", "{\"error\":\"block inválido\"}".as_bytes().to_vec());
                                }
                            }
                        }
                    }
                    next.id = st.dag_topology.id.wrapping_add(1);
                    st.dag_topology = next;
                    new_topo_opt = Some(next);
                }
                if let Some(topo) = new_topo_opt {
                    if let Some(ref rq) = ctx.resource_queue {
                        let _ = rq.push_topology(Box::new(topo));
                    }
                    json_response(200, &serde_json::json!({"ok": true, "topology_id": topo.id}))
                } else {
                    (500, "application/json", b"{\"error\":\"Falha ao atualizar topologia\"}".to_vec())
                }
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/scope" => {
            if method == "GET" {
                let mode = route_with_query
                    .split('?')
                    .nth(1)
                    .and_then(|q| {
                        q.split('&').find_map(|pair| {
                            let mut parts = pair.split('=');
                            if parts.next()? == "mode" {
                                parts.next()
                            } else {
                                None
                            }
                        })
                    })
                    .unwrap_or("transfer");

                let (amp_model_str, preamp_tube_str, asym, plate_v, ts_str, bass, middle, treble, cab_speaker_str) =
                    if let Ok(st) = ctx.app_state.lock() {
                        (
                            st.amp_model.clone(),
                            st.preamp_tube.clone(),
                            st.triode_asym,
                            st.preamp_plate_v,
                            st.tonestack.clone(),
                            st.amp_bass,
                            st.amp_middle,
                            st.amp_treble,
                            st.cab_speaker.clone(),
                        )
                    } else {
                        (
                            "jcm800_2203".to_string(),
                            "12ax7".to_string(),
                            0.20,
                            280.0,
                            "model".to_string(),
                            5.0,
                            5.0,
                            5.0,
                            "v30_4x12".to_string(),
                        )
                    };

                let amp_id = match amp_model_str.as_str() {
                    "plexi_1959" => ampforge_dsp::amp::AmpModelId::BritPlexi1959,
                    "jcm800_2203" => ampforge_dsp::amp::AmpModelId::BritJcm800_2203,
                    "twin_ab763" | "twin_blackface_65" => ampforge_dsp::amp::AmpModelId::UsTwinBlackface65,
                    "dual_rect" | "dual_recto_rev_f" => ampforge_dsp::amp::AmpModelId::CaliDualRectoRevF,
                    "ac30_tb" | "ac30_top_boost" => ampforge_dsp::amp::AmpModelId::UkAc30TopBoost,
                    _ => ampforge_dsp::amp::AmpModelId::BritJcm800_2203,
                };

                match mode {
                    "transfer" => {
                        let tube = parse_preamp_tube(&preamp_tube_str);
                        let n = 65;
                        let mut pts = Vec::with_capacity(n);
                        for i in 0..n {
                            let x = -2.0 + (4.0 * i as f64 / (n - 1) as f64);
                            let y = ampforge_dsp::amp::static_preamp_transfer(
                                x,
                                amp_id,
                                tube,
                                asym as f64,
                                plate_v as f64,
                            );
                            pts.push(serde_json::json!({
                                "x": x,
                                "y": y
                            }));
                        }
                        let resp = serde_json::json!({ "transfer": pts });
                        json_response(200, &resp)
                    }
                    "waveform" => {
                        let max_n = 1024usize;
                        let mut di_buf = vec![0.0f32; max_n];
                        let mut out_buf = vec![0.0f32; max_n];
                        let read_di = ctx
                            .clean_input_ring
                            .as_ref()
                            .map(|ring| ring.read_latest_since(0, &mut di_buf))
                            .unwrap_or(0);
                        let read_out = ctx
                            .scope_output_ring
                            .as_ref()
                            .map(|ring| ring.read_latest_since(0, &mut out_buf))
                            .unwrap_or(0);
                        let di_peak = di_buf[..read_di]
                            .iter()
                            .fold(0.0f32, |m, &v| m.max(v.abs()));
                        let out_peak = out_buf[..read_out]
                            .iter()
                            .fold(0.0f32, |m, &v| m.max(v.abs()));
                        let di_active = read_di > 0 && di_peak > 1e-4;
                        let out_active = read_out > 0 && out_peak > 1e-4;
                        let has_signal = di_active || out_active;
                        let di_json: Vec<f64> = if has_signal {
                            di_buf[..read_di].iter().map(|&v| v as f64).collect()
                        } else {
                            Vec::new()
                        };
                        let out_json: Vec<f64> = if has_signal {
                            out_buf[..read_out].iter().map(|&v| v as f64).collect()
                        } else {
                            Vec::new()
                        };
                        let resp = serde_json::json!({
                            "no_signal": !has_signal,
                            "di_active": di_active,
                            "output_active": out_active,
                            "waveform": {
                                "di": di_json,
                                "output": out_json
                            }
                        });
                        json_response(200, &resp)
                    }
                    "tonestack" => {
                        let parsed_ts = parse_tonestack(&ts_str);
                        let topo = if parsed_ts != ToneStackTopology::Model {
                            parsed_ts
                        } else {
                            match amp_id {
                                ampforge_dsp::amp::AmpModelId::BritPlexi1959
                                | ampforge_dsp::amp::AmpModelId::BritJcm800_2203 => {
                                    ToneStackTopology::MarshallCathodeFollower
                                }
                                ampforge_dsp::amp::AmpModelId::UsTwinBlackface65 => {
                                    ToneStackTopology::FenderTmbMidScoop
                                }
                                ampforge_dsp::amp::AmpModelId::UkAc30TopBoost => {
                                    ToneStackTopology::VoxTopBoost
                                }
                                ampforge_dsp::amp::AmpModelId::CaliDualRectoRevF => {
                                    ToneStackTopology::MesaMarkPreGain
                                }
                            }
                        };
                        let cab_model = match cab_speaker_str.as_str() {
                            "greenback_4x12" => SpeakerCabModel::Greenback4x12Closed,
                            "v30_4x12" => SpeakerCabModel::V304x12Closed,
                            "alnico_blue_2x12" => SpeakerCabModel::AlnicoBlue2x12Open,
                            "twin_2x12" => SpeakerCabModel::Twin2x12Open,
                            _ => SpeakerCabModel::V304x12Closed,
                        };
                        let imp_curve = cab_model.default_impedance_curve();
                        let f0 = imp_curve.low_resonance_hz as f64;
                        let q = imp_curve.resonance_q as f64;
                        let res_gain_db = imp_curve.low_resonance_gain_db as f64;
                        let ind_db = imp_curve.high_inductance_rise_db as f64;

                        let n = 50;
                        let mut ts = Vec::with_capacity(n);
                        let mut z_curve = Vec::with_capacity(n);
                        for i in 0..n {
                            let f = 20.0 * (1000.0f64).powf(i as f64 / (n - 1) as f64);
                            let db = ampforge_dsp::tonestack::analog_tonestack_magnitude_db(
                                topo, bass, middle, treble, f,
                            );
                            let detune = q * (f / f0 - f0 / f);
                            let res_shape = 1.0 / (1.0 + detune * detune);
                            let ind_shape = (f / 20000.0).clamp(0.0, 1.0);
                            let z_db = res_gain_db * res_shape + ind_db * ind_shape;
                            let z = 8.0 * 10.0f64.powf(z_db / 20.0);
                            ts.push(serde_json::json!({"f": f, "db": db}));
                            z_curve.push(serde_json::json!({"f": f, "z": z}));
                        }
                        let resp = serde_json::json!({
                            "tonestack": ts,
                            "speaker_z": z_curve
                        });
                        json_response(200, &resp)
                    }
                    _ => (400, "application/json", b"{\"error\":\"Modo de osciloscopio invalido\"}".to_vec()),
                }
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/params" => {
            if method == "POST" {
                let res = handle_post_params(body, &ctx.app_state, &ctx.dsp_chain, &ctx.param_queue);
                if res.0 == 200 {
                    if let Ok(st) = ctx.app_state.try_lock() {
                        if st.tuner_enabled {
                            if let (Some(ref ring), Ok(mut sess)) =
                                (&ctx.clean_input_ring, ctx.calibration_session.lock())
                            {
                                if sess.tuner_enabled_at == 0 {
                                    sess.tuner_enabled_at = ring.total_written();
                                }
                            }
                        } else if let Ok(mut sess) = ctx.calibration_session.lock() {
                            sess.tuner_enabled_at = 0;
                        }
                    }
                }
                res
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/input/di" => {
            if method == "POST" {
                handle_post_di(body, ctx)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/input/calibrate" => {
            if method == "POST" {
                handle_post_input_calibrate(body, ctx)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/cab/ir" => {
            if method == "POST" {
                handle_post_cab_ir(body, ctx)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/presets" => {
            if method == "GET" {
                handle_get_presets()
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/presets/load" => {
            if method == "POST" {
                handle_post_presets_load(body, ctx)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/presets/save" => {
            if method == "POST" {
                handle_post_presets_save(body, &ctx.app_state, ctx.custom_presets_dir.as_deref())
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/setlist" => {
            if method == "GET" {
                handle_get_setlist(ctx)
            } else if method == "POST" {
                handle_post_setlist(body, ctx)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/setlist/next" => {
            if method == "POST" {
                handle_post_setlist_step(1, ctx)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/setlist/prev" => {
            if method == "POST" {
                handle_post_setlist_step(-1, ctx)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/modifiers" => {
            if method == "GET" {
                handle_get_modifiers(ctx)
            } else if method == "POST" {
                handle_post_modifiers(body, ctx)
            } else if method == "DELETE" {
                handle_delete_modifiers(body, ctx)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/footswitch" => {
            if method == "GET" {
                handle_get_footswitch(ctx)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/footswitch/press" => {
            if method == "POST" {
                handle_post_footswitch_press(body, ctx)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/presets/import" => {
            if method == "POST" {
                handle_post_presets_import(body, ctx)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/scenes/select" => {
            if method == "POST" {
                handle_post_scenes_select(body, ctx)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/reamp/run" => {
            if method == "POST" {
                handle_post_reamp_run(body, ctx)
            } else {
                (405, "text/plain", b"Method Not Allowed".to_vec())
            }
        }
        "/api/presets/*/export" => {
            let id = &path["/api/presets/".len()..path.len() - "/export".len()];
            handle_get_presets_export(id)
        }
        _ => (404, "text/plain", b"Not Found".to_vec()),
    }
}

fn parse_speaker_model(s: &str) -> Option<SpeakerCabModel> {
    match s {
        "greenback_4x12" | "greenback_4x12_closed" => Some(SpeakerCabModel::Greenback4x12Closed),
        "v30_4x12" | "v30_4x12_closed" => Some(SpeakerCabModel::V304x12Closed),
        "alnico_blue_2x12" | "alnico_blue_2x12_open" => Some(SpeakerCabModel::AlnicoBlue2x12Open),
        "twin_2x12_open" | "twin_2x12" | "jensen_c12n_2x12_open" => Some(SpeakerCabModel::Twin2x12Open),
        _ => None,
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

fn parse_mic_angle_deg(v: &serde_json::Value) -> Option<f32> {
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

fn parse_preamp_tube(s: &str) -> PreampTube {
    PreampTube::from_str_name(s).unwrap_or(PreampTube::Ax7A)
}

fn parse_power_tube(s: &str) -> PowerTube {
    match s {
        "6l6gc" => PowerTube::Tube6L6Gc,
        "kt88" => PowerTube::Kt88,
        "el84_variant" | "el84" => PowerTube::El84,
        _ => PowerTube::El34,
    }
}

fn parse_rectifier(s: &str) -> RectifierType {
    match s {
        "gz34" => RectifierType::Gz34_5Ar4,
        "5u4gb" => RectifierType::Dual5U4Gb,
        _ => RectifierType::SolidStateDiode,
    }
}

fn parse_tonestack(s: &str) -> ToneStackTopology {
    match s {
        "marshall" => ToneStackTopology::MarshallCathodeFollower,
        "fender" => ToneStackTopology::FenderTmbMidScoop,
        "vox" => ToneStackTopology::VoxTopBoost,
        "baxandall" => ToneStackTopology::PassiveJamesBaxandall,
        _ => ToneStackTopology::Model,
    }
}

/// Traduz um par JSON `(chave, valor)` para comandos Copy lock-free `DspParamUpdate` (Spec 034, CA-1/CA-2).
fn json_kv_to_dsp_updates(k: &str, v: &serde_json::Value, out: &mut Vec<DspParamUpdate>) {
    match k {
        "amp_1.gain" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpGain(num as f32)); },
        "amp_1.overdrive" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpOverdrive(num as f32)); },
        "amp_1.bass" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpBass(num as f32)); },
        "amp_1.middle" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpMiddle(num as f32)); },
        "amp_1.treble" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpTreble(num as f32)); },
        "amp_1.presence" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpPresence(num as f32)); },
        "amp_1.depth" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpDepth(num as f32)); },
        "amp_1.master" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpMaster(num as f32)); },
        "amp_1.volume_2" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpVolume2(num as f32)); },
        "amp_1.cut" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpCut(num as f32)); },
        "amp_1.reverb" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpReverb(num as f32)); },
        "amp_1.vibrato_speed" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpVibratoSpeed(num as f32)); },
        "amp_1.vibrato_depth" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpVibratoDepth(num as f32)); },
        "amp_1.power" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::AmpPower(b)); },
        "amp_1.standby" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::AmpStandby(b)); },
        "amp_1.jumpered" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::AmpJumpered(b)); },
        "amp_1.spongy" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::AmpSpongy(b)); },
        "amp_1.bright" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::AmpBright(b)); },
        "amp_1.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::AmpEnabled(b)); },
        "amp_1.input_jack" => if let Some(s) = v.as_str() {
            if let Some(j) = ampforge_dsp::amp::AmpInputJack::from_str_name(s) {
                out.push(DspParamUpdate::AmpInputJack(j));
            }
        },
        "amp_1.channel_mode" => if let Some(s) = v.as_str() {
            if let Some(m) = ampforge_dsp::amp::AmpChannelMode::from_str_name(s) {
                out.push(DspParamUpdate::AmpChannelMode(m));
            }
        },
        "amp_1.impedance_ohms" => if let Some(s) = v.as_str() {
            let ohms = match s { "4" => 4, "8" => 8, _ => 16 };
            out.push(DspParamUpdate::AmpImpedanceOhms(ohms));
        },
        "amp_1.model" => if let Some(s) = v.as_str() {
            if let Some(m) = ampforge_dsp::amp::find_model_by_name(s) {
                out.push(DspParamUpdate::AmpModel(m));
            }
        },
        "amp_1.bias_ma" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpBiasMa(num as f32)); },
        "amp_1.sag" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpSag(num as f32)); },
        "amp_1.nfb" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpNfb(num as f32)); },
        "amp_1.xformer_sat" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpXformerSat(num as f32)); },
        "amp_1.preamp_plate_v" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpPreampPlateV(num as f32)); },
        "amp_1.triode_asym" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::AmpTriodeAsym(num as f32)); },
        "amp_1.preamp_tube" => if let Some(s) = v.as_str() { out.push(DspParamUpdate::AmpPreampTube(parse_preamp_tube(s))); },
        "amp_1.power_tube" => if let Some(s) = v.as_str() { out.push(DspParamUpdate::AmpPowerTube(parse_power_tube(s))); },
        "amp_1.rectifier" => if let Some(s) = v.as_str() { out.push(DspParamUpdate::AmpRectifier(parse_rectifier(s))); },
        "amp_1.tonestack" => if let Some(s) = v.as_str() { out.push(DspParamUpdate::AmpToneStack(parse_tonestack(s))); },
        "engine.oversampling" => if let Some(s) = v.as_str() {
            let factor = match s { "2x" => 2, "4x" => 4, _ => 8 };
            out.push(DspParamUpdate::EngineOversampling(factor));
        },
        "input.boost" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::InputBoost(num as f32)); },
        "input.source" => if let Some(s) = v.as_str() {
            let code = match s { "test_riff" => 1, "di_file" => 2, _ => 0 };
            out.push(DspParamUpdate::InputSource(code));
        },
        "input.mode" => if let Some(s) = v.as_str() {
            out.push(DspParamUpdate::InputModeLine(s == "line"));
        },
        "drive.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::DriveEnabled(b)); },
        "drive.model" => if let Some(s) = v.as_str() {
            if let Some(m) = DriveModel::from_str_name(s) {
                out.push(DspParamUpdate::DriveModel(m));
            }
        },
        "drive.drive" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::DriveGain(num as f32)); },
        "drive.tone" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::DriveTone(num as f32)); },
        "drive.level" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::DriveLevel(num as f32)); },
        "drive.bias" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::DriveBias(num as f32)); },
        "gate.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::GateEnabled(b)); },
        "gate.threshold_db" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::GateThreshold(num as f32)); },
        "gate.mode" => if let Some(s) = v.as_str() {
            let m = if s == "classic" {
                ampforge_dsp::fx::NoiseGateMode::ClassicExpanderGate
            } else {
                ampforge_dsp::fx::NoiseGateMode::IntelligentNaturalGate
            };
            out.push(DspParamUpdate::GateMode(m));
        },
        "gate.hysteresis_db" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::GateHysteresis(num as f32)); },
        "gate.attack_ms" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::GateAttack(num as f32)); },
        "gate.hold_ms" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::GateHold(num as f32)); },
        "gate.release_ms" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::GateRelease(num as f32)); },
        "gate.range_db" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::GateRange(num as f32)); },
        "gate.key_hpf_hz" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::GateKeyHpf(num as f32)); },
        "gate.key_lpf_hz" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::GateKeyLpf(num as f32)); },
        "cab_1.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::CabEnabled(b)); },
        "cab_1.speaker" | "cab_1.model" => if let Some(s) = v.as_str() {
            if let Some(spk) = parse_speaker_model(s) {
                out.push(DspParamUpdate::Cab1Speaker(spk));
            }
        },
        "cab_1.resonance_hz" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::CabResonanceHz(num as f32)); },
        "cab_1.hicut_khz" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::CabHicutKhz(num as f32)); },
        "cab_1.ir_enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::CabIrEnabled(b)); },
        "cab_1.mic1.model" => if let Some(s) = v.as_str() {
            if let Some(m) = parse_mic_model(s) { out.push(DspParamUpdate::CabMic1Model(m)); }
        },
        "cab_1.mic1.position" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::CabMic1Position(num as f32)); },
        "cab_1.mic1.distance_cm" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::CabMic1DistanceCm(num as f32)); },
        "cab_1.mic1.level_db" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::CabMic1LevelDb(num as f32)); },
        "cab_1.mic1.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::CabMic1Enabled(b)); },
        "cab_1.mic1.angle_deg" => if let Some(deg) = parse_mic_angle_deg(v) { out.push(DspParamUpdate::Cab1Mic1AngleDeg(deg)); },
        "cab_1.mic1.phase_invert" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::Cab1Mic1PhaseInvert(b)); },
        "cab_1.mic2.model" => if let Some(s) = v.as_str() {
            if let Some(m) = parse_mic_model(s) { out.push(DspParamUpdate::CabMic2Model(m)); }
        },
        "cab_1.mic2.position" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::CabMic2Position(num as f32)); },
        "cab_1.mic2.distance_cm" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::CabMic2DistanceCm(num as f32)); },
        "cab_1.mic2.level_db" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::CabMic2LevelDb(num as f32)); },
        "cab_1.mic2.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::CabMic2Enabled(b)); },
        "cab_1.mic2.angle_deg" => if let Some(deg) = parse_mic_angle_deg(v) { out.push(DspParamUpdate::Cab1Mic2AngleDeg(deg)); },
        "cab_1.mic2.phase_invert" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::Cab1Mic2PhaseInvert(b)); },
        "split.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::SplitEnabled(b)); },
        "split.mode" => if let Some(s) = v.as_str() { out.push(DspParamUpdate::SplitMode(ChainSplitMode::from_str_name(s))); },
        "split.crossover_hz" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::SplitCrossoverHz(num as f32)); },
        "merge.row_1_level_db" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::MergeRow1LevelDb(num as f32)); },
        "merge.row_1_phase_invert" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::MergeRow1PhaseInvert(b)); },
        "comp_2.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::Comp2Enabled(b)); },
        "comp_2.input" => if let Some(num) = v.as_f64() {
            let pre_gain_db = ((num as f32).clamp(0.0, 10.0) - 5.0) * 4.0;
            out.push(DspParamUpdate::Comp2ThresholdDb(-20.0 - pre_gain_db));
        },
        "comp_2.output" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Comp2Output(num as f32)); },
        "comp_2.attack" => if let Some(num) = v.as_f64() {
            let att = (num as f32).clamp(1.0, 7.0);
            let att_ms = 0.8f32 * (0.02f32 / 0.8f32).powf((att - 1.0) / 6.0);
            out.push(DspParamUpdate::Comp2AttackMs(att_ms));
        },
        "comp_2.release" => if let Some(num) = v.as_f64() {
            let rel = (num as f32).clamp(1.0, 7.0);
            let rel_ms = 1100.0f32 * (50.0f32 / 1100.0f32).powf((rel - 1.0) / 6.0);
            out.push(DspParamUpdate::Comp2ReleaseMs(rel_ms));
        },
        "comp_2.ratio" => if let Some(s) = v.as_str() {
            let r = match s { "8" => 8.0, "12" => 12.0, "20" | "all" => 20.0, _ => 4.0 };
            out.push(DspParamUpdate::Comp2Ratio(r));
            out.push(DspParamUpdate::Comp2AllButtons(s == "all"));
        },
        "eq_1.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::Eq1Enabled(b)); },
        "eq_1.mode" => if let Some(s) = v.as_str() {
            if let Some(m) = parse_eq_mode(s) { out.push(DspParamUpdate::Eq1Mode(m)); }
        },
        "eq_1.gain_db" | "eq_1.input_gain_db" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Eq1InputGainDb(num as f32)); },
        "eq_1.volume_db" | "eq_1.output_level_db" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Eq1OutputLevelDb(num as f32)); },
        "mod_2.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::Mod2Enabled(b)); },
        "mod_2.model" => if let Some(s) = v.as_str().and_then(parse_mod_model) { out.push(DspParamUpdate::Mod2Model(s)); },
        "mod_2.rate_hz" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Mod2RateHz(num as f32)); },
        "mod_2.depth" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Mod2Depth(num as f32)); },
        "mod_2.mix" | "mod_2.mix_pct" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Mod2Mix(num as f32)); },
        "dly_1.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::Dly1Enabled(b)); },
        "dly_1.mode" => {
            if let Some(m) = v.as_str().and_then(|s| s.parse::<u8>().ok()).or_else(|| v.as_u64().and_then(|n| u8::try_from(n).ok())) {
                out.push(DspParamUpdate::Dly1Mode(m));
            }
        }
        "dly_1.repeat_rate" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Dly1RepeatRate(num as f32)); },
        "dly_1.intensity" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Dly1Intensity(num as f32)); },
        "dly_1.echo_volume" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Dly1EchoVolume(num as f32)); },
        "dly_1.reverb_volume" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Dly1ReverbVolume(num as f32)); },
        "dly_1.bass" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Dly1Bass(num as f32)); },
        "dly_1.treble" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Dly1Treble(num as f32)); },
        "dly_1.spillover" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::Dly1Spillover(b)); },
        "rev_1.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::Rev1Enabled(b)); },
        "rev_1.decay_s" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Rev1DecayS(num as f32)); },
        "rev_1.predelay_ms" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Rev1PredelayMs(num as f32)); },
        "rev_1.damping" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Rev1Damping(num as f32)); },
        "rev_1.mix" | "rev_1.mix_pct" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Rev1Mix(num as f32)); },
        "rev_1.spillover" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::Rev1Spillover(b)); },
        "amp_2.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::Amp2Enabled(b)); },
        "amp_2.model" => if let Some(s) = v.as_str() {
            if let Some(m) = ampforge_dsp::amp::find_model_by_name(s) {
                out.push(DspParamUpdate::Amp2Model(m));
            }
        },
        "amp_2.gain" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Amp2Gain(num as f32)); },
        "amp_2.bass" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Amp2Bass(num as f32)); },
        "amp_2.middle" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Amp2Middle(num as f32)); },
        "amp_2.treble" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Amp2Treble(num as f32)); },
        "amp_2.master" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Amp2Master(num as f32)); },
        "wah_1.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::Wah1Enabled(b)); },
        "wah_1.position" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Wah1Position(num as f32)); },
        "wah_1.q" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Wah1Q(num as f32)); },
        "wah_1.mode" => if let Some(s) = v.as_str() {
            if let Some(m) = WahMode::from_str_name(s) {
                out.push(DspParamUpdate::Wah1Mode(m));
            }
        },
        "cab_2.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::Cab2Enabled(b)); },
        "cab_2.speaker" | "cab_2.model" => if let Some(s) = v.as_str() {
            if let Some(spk) = parse_speaker_model(s) {
                out.push(DspParamUpdate::Cab2Speaker(spk));
            }
        },
        "cab_2.resonance_hz" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Cab2ResonanceHz(num as f32)); },
        "cab_2.hicut_khz" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Cab2HicutKhz(num as f32)); },
        "master.level_db" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::MasterLevelDb(num as f32)); },
        "palco.exp_1" => if let Some(num) = v.as_f64() {
            let norm = if num > 10.0 { (num as f32) / 100.0 } else { (num as f32) / 10.0 };
            out.push(DspParamUpdate::Exp1Value(norm.clamp(0.0, 1.0)));
        },
        "tuner.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::TunerEnabled(b)); },
        "tuner.output" => if let Some(s) = v.as_str() {
            let m = if s == "bypass" {
                ampforge_dsp::tuner::TunerOutputMode::Bypass
            } else {
                ampforge_dsp::tuner::TunerOutputMode::Mute
            };
            out.push(DspParamUpdate::TunerOutputMode(m));
        },
        "tuner.ref_hz" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::TunerRefHz(num as f32)); },
        "looper.position" => if let Some(s) = v.as_str() {
            let pos = if s == "pre" { LooperPosition::PreChain } else { LooperPosition::PostChain };
            out.push(DspParamUpdate::LooperPosition(pos));
        },
        "looper.action" => if let Some(s) = v.as_str() {
            if let Some(cmd) = parse_looper_action(s) { out.push(DspParamUpdate::LooperAction(cmd)); }
        },
        "pitch_1.enabled" => if let Some(b) = v.as_bool() { out.push(DspParamUpdate::Pitch1Enabled(b)); },
        "pitch_1.model" => if let Some(s) = v.as_str() {
            if let Some(m) = parse_pitch_model(s) { out.push(DspParamUpdate::Pitch1Model(m)); }
        },
        "pitch_1.semitones" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Pitch1Semitones(num as f32)); },
        "pitch_1.key" => if let Some(s) = v.as_str() {
            if let Some(k) = parse_pitch_key(s) { out.push(DspParamUpdate::Pitch1Key(k)); }
        },
        "pitch_1.scale" => if let Some(s) = v.as_str() {
            if let Some(sc) = parse_pitch_scale(s) { out.push(DspParamUpdate::Pitch1Scale(sc)); }
        },
        "pitch_1.mix" | "pitch_1.mix_pct" => if let Some(num) = v.as_f64() { out.push(DspParamUpdate::Pitch1Mix(num as f32)); },
        #[cfg(feature = "fault-injection")]
        "debug.inject_panic" => if v.as_bool() == Some(true) { out.push(DspParamUpdate::DebugInjectPanic); },
        _ if k.starts_with("eq_1.band") => {
            let rest = &k["eq_1.band".len()..];
            let mut parts = rest.split('.');
            if let (Some(num_s), Some(prop), Some(num)) = (parts.next(), parts.next(), v.as_f64()) {
                if let Ok(n) = num_s.parse::<usize>() {
                    if (1..=10).contains(&n) {
                        let idx = n - 1;
                        match prop {
                            "freq_hz" => out.push(DspParamUpdate::Eq1BandFreq(idx, num as f32)),
                            "gain_db" => out.push(DspParamUpdate::Eq1BandGain(idx, num as f32)),
                            "q" => out.push(DspParamUpdate::Eq1BandQ(idx, num as f32)),
                            _ => {}
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

/// Valida e aplica parâmetros de forma atômica em `AppState` e na fila SPSC/cadeia DSP.
pub fn handle_post_params(
    body: &[u8],
    app_state_clone: &Arc<Mutex<AppState>>,
    dsp_chain_clone: &Option<Arc<Mutex<RealtimeDspChain>>>,
    param_queue_clone: &Option<Arc<LockFreeParamQueue>>,
) -> (u16, &'static str, Vec<u8>) {
    if body.len() > 4096 {
        return (413, "text/plain", b"Payload Too Large".to_vec());
    }

    let body_str = String::from_utf8_lossy(body);

    match serde_json::from_str::<serde_json::Value>(&body_str) {
        Ok(serde_json::Value::Object(map)) => {
            if map.len() > 256 {
                return (
                    400,
                    "application/json",
                    br#"{"error":"Patch /api/params excede limite de 256 chaves"}"#.to_vec(),
                );
            }
            let mut validation_error: Option<String> = None;
            let mut applied = Vec::new();
            let mut ignored = Vec::new();
            let mut ignored_reasons = serde_json::Map::new();

            let ir_active = if let Some(ir_val) = map.get("cab_1.ir_enabled") {
                ir_val.as_bool().unwrap_or(false)
            } else {
                app_state_clone.lock().map(|st| st.cab_ir_enabled).unwrap_or(false)
            };

            let active_model = if let Some(m_val) = map.get("amp_1.model") {
                if let Some(m_str) = m_val.as_str() {
                    if !["jcm800_2203", "plexi_1959", "twin_ab763", "dual_rect", "ac30_tb"].contains(&m_str) {
                        return (400, "application/json", b"{\"error\":\"Modelo desconhecido\"}".to_vec());
                    }
                    m_str.to_string()
                } else {
                    return (400, "application/json", b"{\"error\":\"amp_1.model deve ser string\"}".to_vec());
                }
            } else {
                app_state_clone
                    .lock()
                    .map(|st| st.amp_model.clone())
                    .unwrap_or_else(|_| "jcm800_2203".to_string())
            };

            let is_param_on_model = |model: &str, key: &str| -> bool {
                match key {
                    "amp_1.master" => model != "plexi_1959" && model != "twin_ab763",
                    "amp_1.middle" => model != "ac30_tb",
                    "amp_1.volume_2" => model == "plexi_1959",
                    "amp_1.cut" => model == "ac30_tb",
                    _ => true,
                }
            };

            for (k, v) in &map {
                if k.starts_with("cab_1.mic") {
                    if ir_active {
                        ignored.push(k.clone());
                        ignored_reasons.insert(k.clone(), serde_json::Value::String("ir_active".to_string()));
                        continue;
                    }

                    match k.as_str() {
                        "cab_1.mic1.model" | "cab_1.mic2.model" => {
                            if let Some(s) = v.as_str() {
                                if ["sm57", "md421", "m88", "r121", "m160", "u87", "c414"].contains(&s) {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some(format!("{} modelo de microfone inválido", k));
                                    break;
                                }
                            } else {
                                validation_error = Some(format!("{} deve ser string", k));
                                break;
                            }
                        }
                        "cab_1.mic1.position" | "cab_1.mic2.position" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 1.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some(format!("{} fora da faixa 0.0..1.0", k));
                                    break;
                                }
                            } else {
                                validation_error = Some(format!("{} deve ser número", k));
                                break;
                            }
                        }
                        "cab_1.mic1.distance_cm" | "cab_1.mic2.distance_cm" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 30.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some(format!("{} fora da faixa 0.0..30.0", k));
                                    break;
                                }
                            } else {
                                validation_error = Some(format!("{} deve ser número", k));
                                break;
                            }
                        }
                        "cab_1.mic1.level_db" | "cab_1.mic2.level_db" => {
                            if let Some(num) = v.as_f64() {
                                if num >= -60.0 && num <= 12.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some(format!("{} fora da faixa -60.0..12.0", k));
                                    break;
                                }
                            } else {
                                validation_error = Some(format!("{} deve ser número", k));
                                break;
                            }
                        }
                        "cab_1.mic1.enabled" | "cab_1.mic2.enabled" => {
                            if v.is_boolean() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} deve ser booleano", k));
                                break;
                            }
                        }
                        "cab_1.mic1.angle_deg" | "cab_1.mic2.angle_deg" => {
                            if parse_mic_angle_deg(v).is_some() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} deve ser '0' ou '45'", k));
                                break;
                            }
                        }
                        "cab_1.mic1.phase_invert" | "cab_1.mic2.phase_invert" => {
                            if v.is_boolean() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} deve ser booleano", k));
                                break;
                            }
                        }
                        _ => {
                            ignored.push(k.clone());
                        }
                    }
                    continue;
                }

                match k.as_str() {
                    "amp_1.gain" | "amp_1.overdrive" | "amp_1.bass"
                    | "amp_1.middle" | "amp_1.treble"
                    | "amp_1.presence" | "amp_1.depth"
                    | "amp_1.master" | "amp_1.volume_2" | "amp_1.cut"
                    | "amp_1.reverb" | "amp_1.vibrato_speed" | "amp_1.vibrato_depth" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.0 && num <= 10.0 {
                                if is_param_on_model(&active_model, k) {
                                    applied.push(k.clone());
                                } else {
                                    ignored.push(k.clone());
                                }
                            } else {
                                validation_error = Some(format!("{} fora da faixa 0..10", k));
                                break;
                            }
                        } else {
                            validation_error = Some(format!("{} deve ser número", k));
                            break;
                        }
                    }
                    "drive.drive" | "drive.tone" | "drive.level" | "drive.bias" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.0 && num <= 10.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} fora da faixa 0..10", k));
                                break;
                            }
                        } else {
                            validation_error = Some(format!("{} deve ser número", k));
                            break;
                        }
                    }
                    "drive.model" => {
                        if let Some(s) = v.as_str() {
                            if DriveModel::from_str_name(s).is_some() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("drive.model inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("drive.model deve ser string".to_string());
                            break;
                        }
                    }
                    "amp_1.model" => {
                        applied.push(k.clone());
                    }
                    "engine.oversampling" => {
                        if let Some(s) = v.as_str() {
                            if s == "2x" || s == "4x" || s == "8x" {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("engine.oversampling fora dos valores permitidos ('2x', '4x', '8x')".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("engine.oversampling deve ser string".to_string());
                            break;
                        }
                    }
                    "input.boost" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.1 && num <= 8.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("input.boost fora da faixa 0.1..8.0".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("input.boost deve ser número".to_string());
                            break;
                        }
                    }
                    "gate.threshold_db" => {
                        if let Some(num) = v.as_f64() {
                            if num >= -90.0 && num <= 0.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("gate.threshold_db fora da faixa -90..0".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("gate.threshold_db deve ser número".to_string());
                            break;
                        }
                    }
                    "gate.mode" => {
                        if let Some(s) = v.as_str() {
                            if s == "natural" || s == "classic" {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("gate.mode inválido (use 'natural' ou 'classic')".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("gate.mode deve ser string".to_string());
                            break;
                        }
                    }
                    "gate.hysteresis_db" => {
                        if let Some(num) = v.as_f64() {
                            if (0.0..=12.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("gate.hysteresis_db fora da faixa 0..12".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("gate.hysteresis_db deve ser número".to_string());
                            break;
                        }
                    }
                    "gate.attack_ms" => {
                        if let Some(num) = v.as_f64() {
                            if (0.1..=50.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("gate.attack_ms fora da faixa 0.1..50".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("gate.attack_ms deve ser número".to_string());
                            break;
                        }
                    }
                    "gate.hold_ms" => {
                        if let Some(num) = v.as_f64() {
                            if (0.0..=500.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("gate.hold_ms fora da faixa 0..500".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("gate.hold_ms deve ser número".to_string());
                            break;
                        }
                    }
                    "gate.release_ms" => {
                        if let Some(num) = v.as_f64() {
                            if (5.0..=1000.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("gate.release_ms fora da faixa 5..1000".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("gate.release_ms deve ser número".to_string());
                            break;
                        }
                    }
                    "gate.range_db" => {
                        if let Some(num) = v.as_f64() {
                            if (-90.0..=0.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("gate.range_db fora da faixa -90..0".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("gate.range_db deve ser número".to_string());
                            break;
                        }
                    }
                    "gate.key_hpf_hz" => {
                        if let Some(num) = v.as_f64() {
                            if (20.0..=500.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("gate.key_hpf_hz fora da faixa 20..500".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("gate.key_hpf_hz deve ser número".to_string());
                            break;
                        }
                    }
                    "gate.key_lpf_hz" => {
                        if let Some(num) = v.as_f64() {
                            if (1000.0..=12000.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("gate.key_lpf_hz fora da faixa 1000..12000".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("gate.key_lpf_hz deve ser número".to_string());
                            break;
                        }
                    }
                    "amp_1.bright" => {
                        if v.is_boolean() {
                            if is_param_on_model(&active_model, k) {
                                applied.push(k.clone());
                            } else {
                                ignored.push(k.clone());
                            }
                        } else {
                            validation_error = Some(format!("{} deve ser booleano", k));
                            break;
                        }
                    }
                    "drive.enabled" | "gate.enabled"
                    | "amp_1.enabled" | "cab_1.enabled"
                    | "amp_1.power" | "amp_1.standby"
                    | "amp_1.jumpered" | "amp_1.spongy" => {
                        if v.is_boolean() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some(format!("{} deve ser booleano", k));
                            break;
                        }
                    }
                    "amp_1.input_jack" => {
                        if let Some(s) = v.as_str() {
                            if ampforge_dsp::amp::AmpInputJack::from_str_name(s).is_some() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("amp_1.input_jack inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("amp_1.input_jack deve ser string".to_string());
                            break;
                        }
                    }
                    "amp_1.channel_mode" => {
                        if let Some(s) = v.as_str() {
                            if ampforge_dsp::amp::AmpChannelMode::from_str_name(s).is_some() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("amp_1.channel_mode inválido (clean|pushed|vintage|modern)".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("amp_1.channel_mode deve ser string".to_string());
                            break;
                        }
                    }
                    "amp_1.impedance_ohms" => {
                        if let Some(s) = v.as_str() {
                            if matches!(s, "4" | "8" | "16") {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("amp_1.impedance_ohms inválido (use '4', '8' ou '16')".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("amp_1.impedance_ohms deve ser string".to_string());
                            break;
                        }
                    }
                    "input.source" => {
                        if let Some(s) = v.as_str() {
                            if s == "device" || s == "test_riff" || s == "di_file" {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("input.source fora dos valores permitidos ('device', 'test_riff', 'di_file')".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("input.source deve ser string".to_string());
                            break;
                        }
                    }
                    "input.mode" => {
                        if let Some(s) = v.as_str() {
                            if s == "instrument" || s == "line" {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("input.mode fora dos valores permitidos ('instrument', 'line')".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("input.mode deve ser string".to_string());
                            break;
                        }
                    }
                    "amp_1.bias_ma" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 20.0 && num <= 55.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("amp_1.bias_ma fora da faixa 20..55".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("amp_1.bias_ma deve ser número".to_string());
                            break;
                        }
                    }
                    "amp_1.sag" | "amp_1.nfb" | "amp_1.xformer_sat" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.0 && num <= 10.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} fora da faixa 0..10", k));
                                break;
                            }
                        } else {
                            validation_error = Some(format!("{} deve ser número", k));
                            break;
                        }
                    }
                    "amp_1.preamp_plate_v" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 180.0 && num <= 340.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("amp_1.preamp_plate_v fora da faixa 180..340".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("amp_1.preamp_plate_v deve ser número".to_string());
                            break;
                        }
                    }
                    "amp_1.triode_asym" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.0 && num <= 0.60 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("amp_1.triode_asym fora da faixa 0..0.60".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("amp_1.triode_asym deve ser número".to_string());
                            break;
                        }
                    }
                    "amp_1.preamp_tube" => {
                        if let Some(s) = v.as_str() {
                            if PreampTube::from_str_name(s).is_some() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("amp_1.preamp_tube inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("amp_1.preamp_tube deve ser string".to_string());
                            break;
                        }
                    }
                    "amp_1.power_tube" => {
                        if let Some(s) = v.as_str() {
                            if ["el34", "6l6gc", "kt88", "el84_variant"].contains(&s) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("amp_1.power_tube inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("amp_1.power_tube deve ser string".to_string());
                            break;
                        }
                    }
                    "amp_1.rectifier" => {
                        if let Some(s) = v.as_str() {
                            if ["diode", "gz34", "5u4gb"].contains(&s) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("amp_1.rectifier inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("amp_1.rectifier deve ser string".to_string());
                            break;
                        }
                    }
                    "amp_1.tonestack" => {
                        if let Some(s) = v.as_str() {
                            if ["model", "marshall", "fender", "vox", "baxandall"].contains(&s) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("amp_1.tonestack inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("amp_1.tonestack deve ser string".to_string());
                            break;
                        }
                    }
                    "cab_1.speaker" | "cab_1.model" => {
                        if let Some(s) = v.as_str() {
                            if parse_speaker_model(s).is_some() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("cab_1.speaker inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("cab_1.speaker deve ser string".to_string());
                            break;
                        }
                    }
                    "cab_1.resonance_hz" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 65.0 && num <= 135.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("cab_1.resonance_hz fora da faixa 65..135".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("cab_1.resonance_hz deve ser número".to_string());
                            break;
                        }
                    }
                    "cab_1.hicut_khz" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 3.5 && num <= 18.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("cab_1.hicut_khz fora da faixa 3.5..18.0".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("cab_1.hicut_khz deve ser número".to_string());
                            break;
                        }
                    }
                    "cab_1.ir_enabled" => {
                        if v.is_boolean() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("cab_1.ir_enabled deve ser booleano".to_string());
                            break;
                        }
                    }
                    "split.enabled" => {
                        if v.is_boolean() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("split.enabled deve ser booleano".to_string());
                            break;
                        }
                    }
                    "split.mode" => {
                        if let Some(s) = v.as_str() {
                            if matches!(
                                s.to_ascii_lowercase().as_str(),
                                "ab" | "crossover" | "dynamic_split" | "dynamic"
                            ) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("split.mode inválido ('ab', 'crossover' ou 'dynamic_split')".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("split.mode deve ser string".to_string());
                            break;
                        }
                    }
                    "split.crossover_hz" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 100.0 && num <= 2000.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("split.crossover_hz fora da faixa 100..2000".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("split.crossover_hz deve ser número".to_string());
                            break;
                        }
                    }
                    "split.ratio" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.0 && num <= 1.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("split.ratio fora da faixa 0..1".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("split.ratio deve ser número".to_string());
                            break;
                        }
                    }
                    "merge.row_1_level_db" => {
                        if let Some(num) = v.as_f64() {
                            if num >= -60.0 && num <= 12.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("merge.row_1_level_db fora da faixa -60..12".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("merge.row_1_level_db deve ser número".to_string());
                            break;
                        }
                    }
                    "merge.row_1_phase_invert" => {
                        if v.is_boolean() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("merge.row_1_phase_invert deve ser booleano".to_string());
                            break;
                        }
                    }
                    "comp_2.enabled" => {
                        if v.is_boolean() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("comp_2.enabled deve ser booleano".to_string());
                            break;
                        }
                    }
                    "comp_2.input" | "comp_2.output" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.0 && num <= 10.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} fora da faixa 0..10", k));
                                break;
                            }
                        } else {
                            validation_error = Some(format!("{} deve ser número", k));
                            break;
                        }
                    }
                    "comp_2.attack" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 1.0 && num <= 7.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("comp_2.attack fora da faixa 1..7".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("comp_2.attack deve ser número".to_string());
                            break;
                        }
                    }
                    "comp_2.release" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 1.0 && num <= 7.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("comp_2.release fora da faixa 1..7".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("comp_2.release deve ser número".to_string());
                            break;
                        }
                    }
                    "comp_2.ratio" => {
                        if let Some(s) = v.as_str() {
                            if ["4", "8", "12", "20", "all"].contains(&s) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("comp_2.ratio inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("comp_2.ratio deve ser string".to_string());
                            break;
                        }
                    }
                    "eq_1.enabled" => {
                        if v.is_boolean() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("eq_1.enabled deve ser booleano".to_string());
                            break;
                        }
                    }
                    "eq_1.mode" => {
                        if let Some(s) = v.as_str() {
                            if parse_eq_mode(s).is_some() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("eq_1.mode inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("eq_1.mode deve ser string".to_string());
                            break;
                        }
                    }
                    "eq_1.gain_db" | "eq_1.input_gain_db" | "eq_1.volume_db" | "eq_1.output_level_db" => {
                        if let Some(num) = v.as_f64() {
                            if (-12.0..=12.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} fora da faixa -12..12", k));
                                break;
                            }
                        } else {
                            validation_error = Some(format!("{} deve ser número", k));
                            break;
                        }
                    }
                    "mod_2.enabled" => {
                        if v.is_boolean() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("mod_2.enabled deve ser booleano".to_string());
                            break;
                        }
                    }
                    "mod_2.model" => {
                        if v.as_str().and_then(parse_mod_model).is_some() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("mod_2.model inválido".to_string());
                            break;
                        }
                    }
                    "pitch_1.enabled" => {
                        if v.is_boolean() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("pitch_1.enabled deve ser booleano".to_string());
                            break;
                        }
                    }
                    "pitch_1.model" => {
                        if v.as_str().and_then(parse_pitch_model).is_some() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("pitch_1.model inválido".to_string());
                            break;
                        }
                    }
                    "pitch_1.semitones" => {
                        if let Some(num) = v.as_f64() {
                            if num >= -24.0 && num <= 24.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("pitch_1.semitones fora da faixa -24..24".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("pitch_1.semitones deve ser número".to_string());
                            break;
                        }
                    }
                    "pitch_1.key" => {
                        if v.as_str().and_then(parse_pitch_key).is_some() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("pitch_1.key inválido".to_string());
                            break;
                        }
                    }
                    "pitch_1.scale" => {
                        if v.as_str().and_then(parse_pitch_scale).is_some() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("pitch_1.scale inválido".to_string());
                            break;
                        }
                    }
                    "pitch_1.mix" | "pitch_1.mix_pct" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.0 && num <= 100.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} fora da faixa 0..100", k));
                                break;
                            }
                        } else {
                            validation_error = Some(format!("{} deve ser número", k));
                            break;
                        }
                    }
                    "mod_2.rate_hz" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.05 && num <= 10.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("mod_2.rate_hz fora da faixa 0.05..10".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("mod_2.rate_hz deve ser número".to_string());
                            break;
                        }
                    }
                    "mod_2.depth" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.0 && num <= 10.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("mod_2.depth fora da faixa 0..10".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("mod_2.depth deve ser número".to_string());
                            break;
                        }
                    }
                    "mod_2.mix" | "mod_2.mix_pct" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.0 && num <= 100.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("mod_2.mix fora da faixa 0..100".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("mod_2.mix deve ser número".to_string());
                            break;
                        }
                    }
                    "dly_1.enabled" | "dly_1.spillover" => {
                        if v.is_boolean() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some(format!("{} deve ser booleano", k));
                            break;
                        }
                    }
                    "dly_1.mode" => {
                        let valid = v.as_str()
                            .and_then(|s| s.parse::<u8>().ok())
                            .or_else(|| v.as_u64().and_then(|n| u8::try_from(n).ok()))
                            .is_some_and(|m| (1..=12).contains(&m));
                        if valid {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("dly_1.mode deve estar entre 1 e 12".to_string());
                            break;
                        }
                    }
                    "dly_1.heads" => {
                        if let Some(s) = v.as_str() {
                            if ["1", "2", "3", "1+2", "1+3", "2+3", "all"].contains(&s) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("dly_1.heads inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("dly_1.heads deve ser string".to_string());
                            break;
                        }
                    }
                    "dly_1.repeat_rate" | "dly_1.intensity" | "dly_1.echo_volume"
                    | "dly_1.reverb_volume" | "dly_1.bass" | "dly_1.treble" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.0 && num <= 10.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} fora da faixa 0..10", k));
                                break;
                            }
                        } else {
                            validation_error = Some(format!("{} deve ser número", k));
                            break;
                        }
                    }
                    "rev_1.enabled" | "rev_1.spillover" => {
                        if v.is_boolean() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some(format!("{} deve ser booleano", k));
                            break;
                        }
                    }
                    "rev_1.decay_s" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.1 && num <= 20.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("rev_1.decay_s fora da faixa 0.1..20".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("rev_1.decay_s deve ser número".to_string());
                            break;
                        }
                    }
                    "rev_1.predelay_ms" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.0 && num <= 250.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("rev_1.predelay_ms fora da faixa 0..250".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("rev_1.predelay_ms deve ser número".to_string());
                            break;
                        }
                    }
                    "rev_1.damping" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.0 && num <= 10.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("rev_1.damping fora da faixa 0..10".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("rev_1.damping deve ser número".to_string());
                            break;
                        }
                    }
                    "rev_1.mix" | "rev_1.mix_pct" => {
                        if let Some(num) = v.as_f64() {
                            if num >= 0.0 && num <= 100.0 {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("rev_1.mix fora da faixa 0..100".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("rev_1.mix deve ser número".to_string());
                            break;
                        }
                    }
                    "amp_2.enabled" | "cab_2.enabled" | "wah_1.enabled" => {
                        if v.is_boolean() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some(format!("{} deve ser booleano", k));
                            break;
                        }
                    }
                    "amp_2.model" => {
                        if let Some(s) = v.as_str() {
                            if ["jcm800_2203", "plexi_1959", "twin_ab763", "dual_rect", "ac30_tb"].contains(&s) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("amp_2.model inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("amp_2.model deve ser string".to_string());
                            break;
                        }
                    }
                    "amp_2.gain" | "amp_2.bass" | "amp_2.middle" | "amp_2.treble" | "amp_2.master" => {
                        if let Some(num) = v.as_f64() {
                            if (0.0..=10.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} fora da faixa 0..10", k));
                                break;
                            }
                        } else {
                            validation_error = Some(format!("{} deve ser número", k));
                            break;
                        }
                    }
                    "wah_1.position" => {
                        if let Some(num) = v.as_f64() {
                            if (0.0..=1.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("wah_1.position fora da faixa 0..1".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("wah_1.position deve ser número".to_string());
                            break;
                        }
                    }
                    "wah_1.q" => {
                        if let Some(num) = v.as_f64() {
                            if (1.0..=10.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("wah_1.q fora da faixa 1..10".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("wah_1.q deve ser número".to_string());
                            break;
                        }
                    }
                    "wah_1.mode" => {
                        if let Some(s) = v.as_str() {
                            if WahMode::from_str_name(s).is_some() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("wah_1.mode inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("wah_1.mode deve ser string".to_string());
                            break;
                        }
                    }
                    "cab_2.speaker" | "cab_2.model" => {
                        if let Some(s) = v.as_str() {
                            if parse_speaker_model(s).is_some() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("cab_2.speaker inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("cab_2.speaker deve ser string".to_string());
                            break;
                        }
                    }
                    "cab_2.resonance_hz" => {
                        if let Some(num) = v.as_f64() {
                            if (50.0..=160.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("cab_2.resonance_hz fora da faixa 50..160".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("cab_2.resonance_hz deve ser número".to_string());
                            break;
                        }
                    }
                    "cab_2.hicut_khz" => {
                        if let Some(num) = v.as_f64() {
                            if (2.0..=20.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("cab_2.hicut_khz fora da faixa 2.0..20.0".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("cab_2.hicut_khz deve ser número".to_string());
                            break;
                        }
                    }
                    "master.level_db" => {
                        if let Some(num) = v.as_f64() {
                            if (-60.0..=12.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("master.level_db fora da faixa -60..12".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("master.level_db deve ser número".to_string());
                            break;
                        }
                    }
                    "palco.exp_1" => {
                        if let Some(num) = v.as_f64() {
                            if (0.0..=100.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("palco.exp_1 fora da faixa 0..100".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("palco.exp_1 deve ser número".to_string());
                            break;
                        }
                    }
                    k if k.starts_with("eq_1.band") => {
                        let rest = &k["eq_1.band".len()..];
                        let mut parts = rest.split('.');
                        if let (Some(num_s), Some(prop)) = (parts.next(), parts.next()) {
                            if let Ok(n) = num_s.parse::<usize>() {
                                if (1..=10).contains(&n) {
                                    match prop {
                                        "freq_hz" => {
                                            if let Some(num) = v.as_f64() {
                                                if num >= 20.0 && num <= 20000.0 {
                                                    applied.push(k.to_string());
                                                } else {
                                                    validation_error = Some(format!("{} fora da faixa 20..20000", k));
                                                    break;
                                                }
                                            } else {
                                                validation_error = Some(format!("{} deve ser número", k));
                                                break;
                                            }
                                        }
                                        "gain_db" => {
                                            if let Some(num) = v.as_f64() {
                                                if num >= -15.0 && num <= 15.0 {
                                                    applied.push(k.to_string());
                                                } else {
                                                    validation_error = Some(format!("{} fora da faixa -15..15", k));
                                                    break;
                                                }
                                            } else {
                                                validation_error = Some(format!("{} deve ser número", k));
                                                break;
                                            }
                                        }
                                        "q" => {
                                            if let Some(num) = v.as_f64() {
                                                if num >= 0.3 && num <= 10.0 {
                                                    applied.push(k.to_string());
                                                } else {
                                                    validation_error = Some(format!("{} fora da faixa 0.3..10", k));
                                                    break;
                                                }
                                            } else {
                                                validation_error = Some(format!("{} deve ser número", k));
                                                break;
                                            }
                                        }
                                        _ => {
                                            ignored.push(k.to_string());
                                        }
                                    }
                                    continue;
                                }
                            }
                        }
                        ignored.push(k.to_string());
                    }
                    "scene.active" => {
                        if let Some(s) = v.as_str() {
                            if ["A", "B", "C", "D", "E", "F", "G", "H"].contains(&s) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("scene.active fora de A..H".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("scene.active deve ser string".to_string());
                            break;
                        }
                    }
                    k if k.ends_with(".channel") => {
                        if let Some(s) = v.as_str() {
                            if ["A", "B", "C", "D"].contains(&s) {
                                applied.push(k.to_string());
                            } else {
                                validation_error = Some(format!("{} fora de A..D", k));
                                break;
                            }
                        } else {
                            validation_error = Some(format!("{} deve ser string", k));
                            break;
                        }
                    }
                    "footswitch.mode" => {
                        if let Some(s) = v.as_str() {
                            if matches!(
                                s.to_ascii_uppercase().as_str(),
                                "PRESET" | "SCENE" | "STOMP" | "HYBRID_STOMP_SCENE" | "SCENES"
                            ) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("footswitch.mode inválido (PRESET|SCENE|STOMP|HYBRID_STOMP_SCENE)".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("footswitch.mode deve ser string".to_string());
                            break;
                        }
                    }
                    "tuner.enabled" => {
                        if v.is_boolean() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("tuner.enabled deve ser booleano".to_string());
                            break;
                        }
                    }
                    "tuner.output" => {
                        if let Some(s) = v.as_str() {
                            if s == "mute" || s == "bypass" {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("tuner.output deve ser mute ou bypass".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("tuner.output deve ser string".to_string());
                            break;
                        }
                    }
                    "tuner.ref_hz" => {
                        if let Some(num) = v.as_f64() {
                            if (430.0..=450.0).contains(&num) {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("tuner.ref_hz fora da faixa 430..450".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("tuner.ref_hz deve ser número".to_string());
                            break;
                        }
                    }
                    "looper.position" => {
                        if let Some(s) = v.as_str() {
                            if s == "pre" || s == "post" {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("looper.position deve ser 'pre' ou 'post'".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("looper.position deve ser string".to_string());
                            break;
                        }
                    }
                    "looper.action" => {
                        if let Some(s) = v.as_str() {
                            if parse_looper_action(s).is_some() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("looper.action inválido".to_string());
                                break;
                            }
                        } else {
                            validation_error = Some("looper.action deve ser string".to_string());
                            break;
                        }
                    }
                    #[cfg(feature = "fault-injection")]
                    "debug.inject_panic" => {
                        if v.is_boolean() {
                            applied.push(k.clone());
                        } else {
                            validation_error = Some("debug.inject_panic deve ser booleano".to_string());
                            break;
                        }
                    }
                    _ => {
                        ignored.push(k.clone());
                    }
                }
            }

            if let Some(err) = validation_error {
                json_error_response(400, &format!("E_VALIDATION: {}", err))
            } else {
                // Constrói o lote SPSC antes de mutar AppState (Spec 034, CA-1, CA-2, CA-10)
                let mut updates = Vec::with_capacity(applied.len() + 2);
                for (k, v) in &map {
                    if !applied.contains(k) {
                        continue;
                    }
                    json_kv_to_dsp_updates(k.as_str(), v, &mut updates);
                }

                if let Some(ref pq) = param_queue_clone {
                    if !updates.is_empty() && pq.push_batch(&updates).is_err() {
                        return (
                            503,
                            "application/json",
                            br#"{"error":"queue_full","retry_after_ms":5}"#.to_vec(),
                        );
                    }
                } else if let Some(ref dsp_ref) = dsp_chain_clone {
                    if let Ok(mut dsp) = dsp_ref.try_lock() {
                        for u in &updates {
                            dsp.apply_param_update(*u);
                        }
                    }
                }

                if let Ok(mut st) = app_state_clone.lock() {
                    if let Some(m_val) = map.get("amp_1.model").and_then(|v| v.as_str()) {
                        let nom = nominal_chassis_for_model(m_val);
                        st.amp_model = m_val.to_string();
                        st.preamp_tube = nom.0.to_string();
                        st.power_tube = nom.1.to_string();
                        st.bias_ma = nom.2;
                        st.sag = nom.3;
                        st.rectifier = nom.4.to_string();
                        st.nfb = nom.5;
                        st.xformer_sat = nom.6;
                        st.preamp_plate_v = nom.7;
                        st.triode_asym = nom.8;
                        st.tonestack = nom.9.to_string();
                        st.is_modded = false;
                    }

                    for (k, v) in &map {
                        if !applied.contains(k) {
                            continue;
                        }
                        match k.as_str() {
                            "amp_1.gain" => if let Some(n) = v.as_f64() { st.amp_gain = n as f32; },
                            "amp_1.overdrive" => if let Some(n) = v.as_f64() { st.amp_overdrive = n as f32; },
                            "amp_1.bass" => if let Some(n) = v.as_f64() { st.amp_bass = n as f32; },
                            "amp_1.middle" => if let Some(n) = v.as_f64() { st.amp_middle = n as f32; },
                            "amp_1.treble" => if let Some(n) = v.as_f64() { st.amp_treble = n as f32; },
                            "amp_1.presence" => if let Some(n) = v.as_f64() { st.amp_presence = n as f32; },
                            "amp_1.depth" => if let Some(n) = v.as_f64() { st.amp_depth = n as f32; },
                            "amp_1.master" => if let Some(n) = v.as_f64() { st.amp_master = n as f32; },
                            "amp_1.volume_2" => if let Some(n) = v.as_f64() { st.amp_volume_2 = n as f32; },
                            "amp_1.cut" => if let Some(n) = v.as_f64() { st.amp_cut = n as f32; },
                            "amp_1.reverb" => if let Some(n) = v.as_f64() { st.amp_reverb = n as f32; },
                            "amp_1.vibrato_speed" => if let Some(n) = v.as_f64() { st.amp_vibrato_speed = n as f32; },
                            "amp_1.vibrato_depth" => if let Some(n) = v.as_f64() { st.amp_vibrato_depth = n as f32; },
                            "amp_1.power" => if let Some(b) = v.as_bool() { st.amp_power = b; },
                            "amp_1.standby" => if let Some(b) = v.as_bool() { st.amp_standby = b; },
                            "amp_1.jumpered" => if let Some(b) = v.as_bool() { st.amp_jumpered = b; },
                            "amp_1.spongy" => if let Some(b) = v.as_bool() { st.amp_spongy = b; },
                            "amp_1.input_jack" => if let Some(s) = v.as_str() { st.amp_input_jack = s.to_string(); },
                            "amp_1.channel_mode" => if let Some(s) = v.as_str() { st.amp_channel_mode = s.to_string(); },
                            "amp_1.impedance_ohms" => if let Some(s) = v.as_str() { st.amp_impedance_ohms = s.to_string(); },
                            "amp_1.model" => {}
                            "engine.oversampling" => if let Some(s) = v.as_str() { st.engine_oversampling = s.to_string(); },
                            "input.boost" => if let Some(n) = v.as_f64() { st.input_boost = n as f32; },
                            "gate.threshold_db" => if let Some(n) = v.as_f64() { st.gate_threshold_db = n as f32; },
                            "amp_1.bright" => if let Some(b) = v.as_bool() { st.amp_bright = b; },
                            "drive.enabled" => if let Some(b) = v.as_bool() { st.drive_enabled = b; },
                            "drive.model" => if let Some(s) = v.as_str() { st.drive_model = s.to_string(); },
                            "drive.drive" => if let Some(n) = v.as_f64() { st.drive_drive = n as f32; },
                            "drive.tone" => if let Some(n) = v.as_f64() { st.drive_tone = n as f32; },
                            "drive.level" => if let Some(n) = v.as_f64() { st.drive_level = n as f32; },
                            "drive.bias" => if let Some(n) = v.as_f64() { st.drive_bias = n as f32; },
                            "gate.enabled" => if let Some(b) = v.as_bool() { st.gate_enabled = b; },
                            "gate.mode" => if let Some(s) = v.as_str() { st.gate_mode = s.to_string(); },
                            "gate.hysteresis_db" => if let Some(n) = v.as_f64() { st.gate_hysteresis_db = n as f32; },
                            "gate.attack_ms" => if let Some(n) = v.as_f64() { st.gate_attack_ms = n as f32; },
                            "gate.hold_ms" => if let Some(n) = v.as_f64() { st.gate_hold_ms = n as f32; },
                            "gate.release_ms" => if let Some(n) = v.as_f64() { st.gate_release_ms = n as f32; },
                            "gate.range_db" => if let Some(n) = v.as_f64() { st.gate_range_db = n as f32; },
                            "gate.key_hpf_hz" => if let Some(n) = v.as_f64() { st.gate_key_hpf_hz = n as f32; },
                            "gate.key_lpf_hz" => if let Some(n) = v.as_f64() { st.gate_key_lpf_hz = n as f32; },
                            "amp_1.enabled" => if let Some(b) = v.as_bool() { st.amp_enabled = b; },
                            "cab_1.enabled" => if let Some(b) = v.as_bool() { st.cab_enabled = b; },
                            "input.source" => if let Some(s) = v.as_str() { st.input_source = s.to_string(); },
                            "input.mode" => if let Some(s) = v.as_str() { st.input_mode = s.to_string(); },
                            "amp_1.bias_ma" => if let Some(n) = v.as_f64() { st.bias_ma = n as f32; },
                            "amp_1.sag" => if let Some(n) = v.as_f64() { st.sag = n as f32; },
                            "amp_1.nfb" => if let Some(n) = v.as_f64() { st.nfb = n as f32; },
                            "amp_1.xformer_sat" => if let Some(n) = v.as_f64() { st.xformer_sat = n as f32; },
                            "amp_1.preamp_plate_v" => if let Some(n) = v.as_f64() { st.preamp_plate_v = n as f32; },
                            "amp_1.triode_asym" => if let Some(n) = v.as_f64() { st.triode_asym = n as f32; },
                            "amp_1.preamp_tube" => if let Some(s) = v.as_str() { st.preamp_tube = s.to_string(); },
                            "amp_1.power_tube" => if let Some(s) = v.as_str() { st.power_tube = s.to_string(); },
                            "amp_1.rectifier" => if let Some(s) = v.as_str() { st.rectifier = s.to_string(); },
                            "amp_1.tonestack" => if let Some(s) = v.as_str() { st.tonestack = s.to_string(); },
                            "cab_1.speaker" | "cab_1.model" => {
                                if let Some(s) = v.as_str() {
                                    st.cab_speaker = s.to_string();
                                }
                                st.cab_resonance_hz = match st.cab_speaker.as_str() {
                                    "greenback_4x12" => 98.0,
                                    "v30_4x12" => 108.0,
                                    "alnico_blue_2x12" => 82.0,
                                    "twin_2x12_open" => 90.0,
                                    _ => 98.0,
                                };
                            }
                            "cab_1.resonance_hz" => if let Some(n) = v.as_f64() { st.cab_resonance_hz = n as f32; },
                            "cab_1.hicut_khz" => if let Some(n) = v.as_f64() { st.cab_hicut_khz = n as f32; },
                            "cab_1.ir_enabled" => if let Some(b) = v.as_bool() { st.cab_ir_enabled = b; },
                            "cab_1.mic1.model" => if let Some(s) = v.as_str() { st.cab_mic1_model = s.to_string(); },
                            "cab_1.mic1.position" => if let Some(n) = v.as_f64() { st.cab_mic1_position = n as f32; },
                            "cab_1.mic1.distance_cm" => if let Some(n) = v.as_f64() { st.cab_mic1_distance_cm = n as f32; },
                            "cab_1.mic1.level_db" => if let Some(n) = v.as_f64() { st.cab_mic1_level_db = n as f32; },
                            "cab_1.mic1.enabled" => if let Some(b) = v.as_bool() { st.cab_mic1_enabled = b; },
                            "cab_1.mic1.angle_deg" => if let Some(deg) = parse_mic_angle_deg(v) {
                                st.cab_mic1_angle_deg = if deg >= 22.5 { "45".to_string() } else { "0".to_string() };
                            },
                            "cab_1.mic1.phase_invert" => if let Some(b) = v.as_bool() { st.cab_mic1_phase_invert = b; },
                            "cab_1.mic2.model" => if let Some(s) = v.as_str() { st.cab_mic2_model = s.to_string(); },
                            "cab_1.mic2.position" => if let Some(n) = v.as_f64() { st.cab_mic2_position = n as f32; },
                            "cab_1.mic2.distance_cm" => if let Some(n) = v.as_f64() { st.cab_mic2_distance_cm = n as f32; },
                            "cab_1.mic2.level_db" => if let Some(n) = v.as_f64() { st.cab_mic2_level_db = n as f32; },
                            "cab_1.mic2.enabled" => if let Some(b) = v.as_bool() { st.cab_mic2_enabled = b; },
                            "cab_1.mic2.angle_deg" => if let Some(deg) = parse_mic_angle_deg(v) {
                                st.cab_mic2_angle_deg = if deg >= 22.5 { "45".to_string() } else { "0".to_string() };
                            },
                            "cab_1.mic2.phase_invert" => if let Some(b) = v.as_bool() { st.cab_mic2_phase_invert = b; },
                            "split.enabled" => if let Some(b) = v.as_bool() { st.split_enabled = b; },
                            "split.mode" => if let Some(s) = v.as_str() { st.split_mode = s.to_string(); },
                            "split.crossover_hz" => if let Some(n) = v.as_f64() { st.split_crossover_hz = n as f32; },
                            "merge.row_1_level_db" => if let Some(n) = v.as_f64() { st.merge_row_1_level_db = n as f32; },
                            "merge.row_1_phase_invert" => if let Some(b) = v.as_bool() { st.merge_row_1_phase_invert = b; },
                            "comp_2.enabled" => if let Some(b) = v.as_bool() { st.comp_2_enabled = b; },
                            "comp_2.input" => if let Some(n) = v.as_f64() { st.comp_2_input = n as f32; },
                            "comp_2.output" => if let Some(n) = v.as_f64() { st.comp_2_output = n as f32; },
                            "comp_2.attack" => if let Some(n) = v.as_f64() { st.comp_2_attack = n as f32; },
                            "comp_2.release" => if let Some(n) = v.as_f64() { st.comp_2_release = n as f32; },
                            "comp_2.ratio" => if let Some(s) = v.as_str() { st.comp_2_ratio = s.to_string(); },
                            "eq_1.enabled" => if let Some(b) = v.as_bool() { st.eq_1_enabled = b; },
                            "eq_1.mode" => if let Some(s) = v.as_str() { st.eq_1_mode = s.to_string(); },
                            "eq_1.gain_db" | "eq_1.input_gain_db" => if let Some(n) = v.as_f64() { st.eq_1_gain_db = n as f32; },
                            "eq_1.volume_db" | "eq_1.output_level_db" => if let Some(n) = v.as_f64() { st.eq_1_volume_db = n as f32; },
                            k if k.starts_with("eq_1.band") && k.ends_with(".gain_db") => {
                                if let (Some(n), Some(val)) = (
                                    k["eq_1.band".len()..].split('.').next().and_then(|s| s.parse::<usize>().ok()),
                                    v.as_f64(),
                                ) {
                                    if (1..=10).contains(&n) {
                                        st.eq_1_bands_db[n - 1] = val as f32;
                                    }
                                }
                            }
                            "mod_2.enabled" => if let Some(b) = v.as_bool() { st.mod_2_enabled = b; },
                            "mod_2.model" => if let Some(s) = v.as_str() { st.mod_2_model = s.to_string(); },
                            "mod_2.rate_hz" => if let Some(n) = v.as_f64() { st.mod_2_rate_hz = n as f32; },
                            "mod_2.depth" => if let Some(n) = v.as_f64() { st.mod_2_depth = n as f32; },
                            "mod_2.mix" | "mod_2.mix_pct" => if let Some(n) = v.as_f64() { st.mod_2_mix = n as f32; },
                            "pitch_1.enabled" => if let Some(b) = v.as_bool() { st.pitch_1_enabled = b; },
                            "pitch_1.model" => if let Some(s) = v.as_str() { st.pitch_1_model = s.to_string(); },
                            "pitch_1.semitones" => if let Some(n) = v.as_f64() { st.pitch_1_semitones = n as f32; },
                            "pitch_1.key" => if let Some(s) = v.as_str() { st.pitch_1_key = s.to_string(); },
                            "pitch_1.scale" => if let Some(s) = v.as_str() { st.pitch_1_scale = s.to_string(); },
                            "pitch_1.mix" | "pitch_1.mix_pct" => if let Some(n) = v.as_f64() { st.pitch_1_mix = n as f32; },
                            "dly_1.enabled" => if let Some(b) = v.as_bool() { st.dly_1_enabled = b; },
                            "dly_1.mode" => {
                                if let Some(s) = v.as_str() {
                                    st.dly_1_mode = s.to_string();
                                } else if let Some(n) = v.as_u64() {
                                    st.dly_1_mode = n.to_string();
                                }
                            }
                            "dly_1.heads" => if let Some(s) = v.as_str() { st.dly_1_heads = s.to_string(); },
                            "dly_1.repeat_rate" => if let Some(n) = v.as_f64() { st.dly_1_repeat_rate = n as f32; },
                            "dly_1.intensity" => if let Some(n) = v.as_f64() { st.dly_1_intensity = n as f32; },
                            "dly_1.echo_volume" => if let Some(n) = v.as_f64() { st.dly_1_echo_volume = n as f32; },
                            "dly_1.reverb_volume" => if let Some(n) = v.as_f64() { st.dly_1_reverb_volume = n as f32; },
                            "dly_1.bass" => if let Some(n) = v.as_f64() { st.dly_1_bass = n as f32; },
                            "dly_1.treble" => if let Some(n) = v.as_f64() { st.dly_1_treble = n as f32; },
                            "dly_1.spillover" => if let Some(b) = v.as_bool() { st.dly_1_spillover = b; },
                            "rev_1.enabled" => if let Some(b) = v.as_bool() { st.rev_1_enabled = b; },
                            "rev_1.decay_s" => if let Some(n) = v.as_f64() { st.rev_1_decay_s = n as f32; },
                            "rev_1.predelay_ms" => if let Some(n) = v.as_f64() { st.rev_1_predelay_ms = n as f32; },
                            "rev_1.damping" => if let Some(n) = v.as_f64() { st.rev_1_damping = n as f32; },
                            "rev_1.mix" | "rev_1.mix_pct" => if let Some(n) = v.as_f64() { st.rev_1_mix = n as f32; },
                            "rev_1.spillover" => if let Some(b) = v.as_bool() { st.rev_1_spillover = b; },
                            "amp_2.enabled" => if let Some(b) = v.as_bool() { st.amp_2_enabled = b; },
                            "amp_2.model" => if let Some(s) = v.as_str() { st.amp_2_model = s.to_string(); },
                            "amp_2.gain" => if let Some(n) = v.as_f64() { st.amp_2_gain = n as f32; },
                            "amp_2.bass" => if let Some(n) = v.as_f64() { st.amp_2_bass = n as f32; },
                            "amp_2.middle" => if let Some(n) = v.as_f64() { st.amp_2_middle = n as f32; },
                            "amp_2.treble" => if let Some(n) = v.as_f64() { st.amp_2_treble = n as f32; },
                            "amp_2.master" => if let Some(n) = v.as_f64() { st.amp_2_master = n as f32; },
                            "wah_1.enabled" => if let Some(b) = v.as_bool() { st.wah_1_enabled = b; },
                            "wah_1.position" => if let Some(n) = v.as_f64() { st.wah_1_position = n as f32; },
                            "wah_1.q" => if let Some(n) = v.as_f64() { st.wah_1_q = n as f32; },
                            "wah_1.mode" => if let Some(s) = v.as_str() { st.wah_1_mode = s.to_string(); },
                            "cab_2.enabled" => if let Some(b) = v.as_bool() { st.cab_2_enabled = b; },
                            "cab_2.speaker" | "cab_2.model" => if let Some(s) = v.as_str() { st.cab_2_speaker = s.to_string(); },
                            "cab_2.resonance_hz" => if let Some(n) = v.as_f64() { st.cab_2_resonance_hz = n as f32; },
                            "cab_2.hicut_khz" => if let Some(n) = v.as_f64() { st.cab_2_hicut_khz = n as f32; },
                            "master.level_db" => if let Some(n) = v.as_f64() { st.master_level_db = n as f32; },
                            "palco.exp_1" => if let Some(n) = v.as_f64() { st.palco_exp_1 = n as f32; },
                            "scene.active" => if let Some(s) = v.as_str() { st.scene_active = s.to_string(); },
                            "footswitch.mode" => if let Some(s) = v.as_str() { st.footswitch_mode = s.to_string(); },
                            "tuner.enabled" => if let Some(b) = v.as_bool() { st.tuner_enabled = b; },
                            "tuner.output" => if let Some(s) = v.as_str() { st.tuner_output = s.to_string(); },
                            "tuner.ref_hz" => if let Some(n) = v.as_f64() { st.tuner_ref_hz = n as f32; },
                            "looper.position" => if let Some(s) = v.as_str() { st.looper_position = s.to_string(); },
                            "looper.action" => {
                                if let Some(act) = v.as_str() {
                                    match act {
                                        "reverse" => st.looper_reverse = !st.looper_reverse,
                                        "half_speed" => st.looper_half_speed = !st.looper_half_speed,
                                        "rec" => st.looper_action = "record".to_string(),
                                        other => st.looper_action = other.to_string(),
                                    }
                                }
                            }
                            _ => {}
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
                    resp_json["ignored_reasons"] = serde_json::Value::Object(ignored_reasons);
                }
                (200, "application/json", resp_json.to_string().into_bytes())
            }
        }
        _ => (400, "application/json", b"{\"error\":\"Corpo deve ser objeto JSON\"}".to_vec()),
    }
}

fn handle_post_di(
    body: &[u8],
    ctx: &ApiContext,
) -> (u16, &'static str, Vec<u8>) {
    if body.len() > 20 * 1024 * 1024 {
        return (413, "text/plain", b"Payload Too Large".to_vec());
    }

    let session_sr = ctx
        .shared_sample_rate_bits
        .as_ref()
        .map(|bits| f32::from_bits(bits.load(Ordering::Relaxed)).max(1.0))
        .unwrap_or(48000.0);

    let loader = ampforge_dsp::cab::WavIrLoader::new(session_sr);
    match loader.load_wav_unlimited(body) {
        Ok(ir) => {
            let samples = ir.samples().to_vec();
            if let Some(ref rq) = ctx.resource_queue {
                if rq.push(PreparedDspResource::DiSamples(samples)).is_err() {
                    return (
                        503,
                        "application/json",
                        br#"{"error":"queue_full","retry_after_ms":5}"#.to_vec(),
                    );
                }
                if let Some(ref pq) = ctx.param_queue {
                    if pq.push(DspParamUpdate::InputSource(2)).is_err() {
                        return (
                            503,
                            "application/json",
                            br#"{"error":"queue_full","retry_after_ms":5}"#.to_vec(),
                        );
                    }
                }
            } else if let Some(ref dsp_ref) = ctx.dsp_chain {
                if let Ok(mut dsp) = dsp_ref.lock() {
                    dsp.load_di_wav(samples);
                    dsp.set_input_source("di_file");
                }
            }
            if let Ok(mut st) = ctx.app_state.lock() {
                st.input_source = "di_file".to_string();
            }
            (200, "application/json", b"{\"status\":\"ok\",\"loaded\":true}".to_vec())
        }
        Err(e) => {
            (400, "application/json", format!("{{\"error\":\"WAV inválido: {:?}\"}}", e).into_bytes())
        }
    }
}

fn handle_post_cab_ir(
    body: &[u8],
    ctx: &ApiContext,
) -> (u16, &'static str, Vec<u8>) {
    if body.len() > 20 * 1024 * 1024 {
        return (413, "text/plain", b"Payload Too Large".to_vec());
    }

    let session_sr = ctx
        .shared_sample_rate_bits
        .as_ref()
        .map(|bits| f32::from_bits(bits.load(Ordering::Relaxed)).max(1.0))
        .unwrap_or(48000.0);

    let loader = ampforge_dsp::cab::WavIrLoader::new(session_sr);
    match loader.load_from_bytes_with_info(body) {
        Ok((ir, source_channels)) => {
            if let Some(ref rq) = ctx.resource_queue {
                if rq.push(PreparedDspResource::CabinetIr { slot: 0, ir }).is_err() {
                    return (
                        503,
                        "application/json",
                        br#"{"error":"queue_full","retry_after_ms":5}"#.to_vec(),
                    );
                }
                if let Some(ref pq) = ctx.param_queue {
                    if pq.push(DspParamUpdate::CabIrEnabled(true)).is_err() {
                        return (
                            503,
                            "application/json",
                            br#"{"error":"queue_full","retry_after_ms":5}"#.to_vec(),
                        );
                    }
                }
            } else if let Some(ref dsp_ref) = ctx.dsp_chain {
                if let Ok(mut dsp) = dsp_ref.lock() {
                    let _ = dsp.load_cab_ir(body);
                    dsp.set_cab_ir_enabled(true);
                }
            }
            if let Ok(mut st) = ctx.app_state.lock() {
                st.cab_ir_enabled = true;
            }
            let resp = serde_json::json!({
                "status": "ok",
                "loaded": true,
                "channels_used": 1,
                "source_channels": source_channels
            });
            json_response(200, &resp)
        }
        Err(e) => {
            let err_msg = format!("{}", e);
            let err_code = if err_msg.contains("ir_too_long") {
                "ir_too_long"
            } else {
                "invalid_wav"
            };
            let resp = serde_json::json!({
                "error": err_code,
                "message": err_msg,
                "limit_ms": 500
            });
            json_response(400, &resp)
        }
    }
}

/// Valida id de preset contra path traversal (Spec 046 CA-4): `^[a-z0-9][a-z0-9_-]{0,63}$`
pub fn is_valid_preset_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 {
        return false;
    }
    if !matches!(bytes[0], b'a'..=b'z' | b'0'..=b'9') {
        return false;
    }
    bytes
        .iter()
        .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-'))
}

const EMBEDDED_FACTORY_PRESETS: &[(&str, &str, &str)] = &[
    ("brit800", "Brit 800 Crunch", include_str!("../../../presets/factory/brit800.sonixpreset")),
    ("usdeluxe", "US Deluxe Clean", include_str!("../../../presets/factory/usdeluxe.sonixpreset")),
    ("recto", "Recto High Gain", include_str!("../../../presets/factory/recto.sonixpreset")),
    ("ac30", "AC30 Top Boost", include_str!("../../../presets/factory/ac30.sonixpreset")),
];

fn embedded_factory_preset_content(id: &str) -> Option<&'static str> {
    let clean_id = id.trim_end_matches(".sonixpreset");
    EMBEDDED_FACTORY_PRESETS
        .iter()
        .find(|(fid, _, _)| *fid == clean_id)
        .map(|(_, _, content)| *content)
}

fn find_preset_file_in(custom_dir: Option<&std::path::Path>, id: &str) -> Option<std::path::PathBuf> {
    let clean_id = id.trim_end_matches(".sonixpreset");
    if !is_valid_preset_id(clean_id) {
        return None;
    }
    if let Some(dir) = custom_dir {
        let p = dir.join(format!("{}.sonixpreset", clean_id));
        if p.exists() {
            return Some(p);
        }
    }
    let candidate_dirs = [
        "presets/factory",
        "../../presets/factory",
        "../presets/factory",
        "presets/user",
        "../../presets/user",
        "../presets/user",
    ];
    for dir in candidate_dirs {
        let p = std::path::PathBuf::from(format!("{}/{}.sonixpreset", dir, clean_id));
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        let appdata_path = std::path::PathBuf::from(appdata)
            .join("SonixPro")
            .join("presets")
            .join(format!("{}.sonixpreset", clean_id));
        if appdata_path.exists() {
            return Some(appdata_path);
        }
    }
    None
}

fn handle_get_presets() -> (u16, &'static str, Vec<u8>) {
    let mut list = Vec::new();
    for &(id, name, _) in EMBEDDED_FACTORY_PRESETS {
        list.push(serde_json::json!({
            "id": id,
            "name": name,
            "type": "factory"
        }));
    }

    let mut scan_dir = |dir: std::path::PathBuf| {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("sonixpreset") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if !EMBEDDED_FACTORY_PRESETS.iter().any(|(fid, _, _)| *fid == stem) {
                            list.push(serde_json::json!({
                                "id": stem,
                                "name": stem,
                                "type": "user"
                            }));
                        }
                    }
                }
            }
        }
    };
    scan_dir(std::path::PathBuf::from("presets/user"));
    if let Ok(appdata) = std::env::var("APPDATA") {
        scan_dir(std::path::PathBuf::from(appdata).join("SonixPro").join("presets"));
    }

    json_response(200, &serde_json::json!({ "presets": list }))
}

fn find_preset_file(id: &str) -> Option<std::path::PathBuf> {
    find_preset_file_in(None, id)
}

fn preset_exists_in(custom_dir: Option<&std::path::Path>, id: &str) -> bool {
    find_preset_file_in(custom_dir, id).is_some() || embedded_factory_preset_content(id).is_some()
}

fn handle_post_presets_load(
    body: &[u8],
    ctx: &ApiContext,
) -> (u16, &'static str, Vec<u8>) {
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) else {
        return (400, "application/json", b"{\"error\":\"invalid_json\"}".to_vec());
    };
    let Some(id) = v.get("id").and_then(|id_v| id_v.as_str()) else {
        return (400, "application/json", b"{\"error\":\"missing_id\"}".to_vec());
    };
    if !is_valid_preset_id(id) {
        return (400, "application/json", b"{\"error\":\"invalid_preset_id\"}".to_vec());
    }

    let content: String = if let Some(path) = find_preset_file_in(ctx.custom_presets_dir.as_deref(), id) {
        let Ok(c) = std::fs::read_to_string(&path) else {
            return (500, "application/json", b"{\"error\":\"failed_to_read_preset\"}".to_vec());
        };
        c
    } else if let Some(embedded) = embedded_factory_preset_content(id) {
        embedded.to_string()
    } else {
        return (404, "application/json", b"{\"error\":\"preset_not_found\"}".to_vec());
    };

    let Ok(doc) = sonix_core::persistence::SonixPresetDoc::from_json_str(&content) else {
        return (400, "application/json", b"{\"error\":\"invalid_preset_schema\"}".to_vec());
    };

    if let Some(ref params) = doc.params {
        if let Err(e) = apply_preset_params_queued(params, &ctx.app_state, &ctx.param_queue, &ctx.dsp_chain) {
            return match e {
                PresetApplyError::Validation(msg) => (
                    400,
                    "application/json",
                    format!("{{\"error\":\"invalid_preset_param\",\"message\":\"{}\"}}", msg).into_bytes(),
                ),
                PresetApplyError::QueueFull => (
                    503,
                    "application/json",
                    br#"{"error":"queue_full","retry_after_ms":5}"#.to_vec(),
                ),
            };
        }
    }

    let state_json = if let Ok(mut st) = ctx.app_state.lock() {
        st.active_preset_id = id.to_string();
        st.active_preset_name = doc.metadata.name.clone();
        if !doc.scenes.is_empty() {
            st.scenes = doc.scenes.clone();
        } else {
            st.scenes = default_scenes();
        }
        st.grid = doc.grid.clone();
        st.channels = doc.channels.clone().unwrap_or_default();
        if !doc
            .params
            .as_ref()
            .map_or(false, |p| p.contains_key("scene.active"))
        {
            let letters = ["A", "B", "C", "D", "E", "F", "G", "H"];
            if let Some(&l) = letters.get(doc.active_scene_index) {
                st.scene_active = l.to_string();
            }
        }
        serde_json::to_value(&*st).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    json_response(
        200,
        &serde_json::json!({
            "status": "ok",
            "loaded": id,
            "name": doc.metadata.name,
            "state": state_json
        }),
    )
}

fn handle_post_presets_save(
    body: &[u8],
    app_state_clone: &Arc<Mutex<AppState>>,
    custom_dir: Option<&std::path::Path>,
) -> (u16, &'static str, Vec<u8>) {
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) else {
        return (400, "application/json", b"{\"error\":\"invalid_json\"}".to_vec());
    };
    let Some(id) = v.get("id").and_then(|id_v| id_v.as_str()) else {
        return (400, "application/json", b"{\"error\":\"missing_id\"}".to_vec());
    };
    if !is_valid_preset_id(id) {
        return (400, "application/json", b"{\"error\":\"invalid_preset_id\"}".to_vec());
    }
    let name = v.get("name").and_then(|n| n.as_str()).unwrap_or(id);

    let save_dir = if let Some(dir) = custom_dir {
        let _ = std::fs::create_dir_all(dir);
        dir.to_path_buf()
    } else if let Ok(appdata) = std::env::var("APPDATA") {
        let p = std::path::PathBuf::from(appdata).join("SonixPro").join("presets");
        let _ = std::fs::create_dir_all(&p);
        p
    } else {
        let p = std::path::PathBuf::from("presets/user");
        let _ = std::fs::create_dir_all(&p);
        p
    };

    let file_path = save_dir.join(format!("{}.sonixpreset", id));
    if file_path.parent() != Some(save_dir.as_path()) {
        return (400, "application/json", b"{\"error\":\"invalid_preset_id\"}".to_vec());
    }

    let current_state = if let Ok(st) = app_state_clone.lock() {
        st.clone()
    } else {
        AppState::default()
    };

    let state_val = serde_json::to_value(&current_state).unwrap_or_default();
    let mut params_map = std::collections::HashMap::new();
    if let serde_json::Value::Object(map) = state_val {
        for (k, val) in map {
            if matches!(
                k.as_str(),
                "scenes"
                    | "grid"
                    | "channels"
                    | "preset.active"
                    | "preset.name"
                    | "amp_1.is_modded"
                    | "is_modded"
            ) {
                continue;
            }
            params_map.insert(k, val);
        }
    }

    let active_scene_index = ["A", "B", "C", "D", "E", "F", "G", "H"]
        .iter()
        .position(|&s| s == current_state.scene_active.as_str())
        .unwrap_or(0);

    let doc = sonix_core::persistence::SonixPresetDoc {
        schema: "https://sonixpro.local/schemas/sonixpreset-v1.json".to_string(),
        schema_version: "1.1.0".to_string(),
        metadata: sonix_core::persistence::PresetMetadata {
            id: id.to_string(),
            name: name.to_string(),
            author: "User".to_string(),
            category: "Custom".to_string(),
            tempo_bpm: 120.0,
            created_at: format_utc_iso8601_now(),
            footswitch_mode: "scenes".to_string(),
        },
        active_scene_index,
        scenes: current_state.scenes.clone(),
        grid: current_state.grid.clone(),
        modifiers: vec![],
        params: Some(params_map),
        channels: if current_state.channels.is_empty() {
            None
        } else {
            Some(current_state.channels.clone())
        },
    };

    let Ok(json_str) = doc.to_json_str() else {
        return (500, "application/json", b"{\"error\":\"failed_to_serialize\"}".to_vec());
    };

    if let Err(e) = std::fs::write(&file_path, json_str) {
        return (
            500,
            "application/json",
            format!("{{\"error\":\"failed_to_write_file\",\"message\":\"{}\"}}", e).into_bytes(),
        );
    }

    json_response(
        200,
        &serde_json::json!({
            "status": "ok",
            "saved": id
        }),
    )
}

fn handle_post_presets_import(
    body: &[u8],
    ctx: &ApiContext,
) -> (u16, &'static str, Vec<u8>) {
    if body.len() > 256 * 1024 {
        return (413, "application/json", b"{\"error\":\"preset_too_large\"}".to_vec());
    }
    let Ok(content) = std::str::from_utf8(body) else {
        return (400, "application/json", b"{\"error\":\"invalid_utf8\"}".to_vec());
    };

    let doc = match sonix_core::persistence::SonixPresetDoc::from_json_str(content) {
        Ok(d) => {
            if d.params.as_ref().map(|p| p.len() > 512).unwrap_or(false) {
                return (400, "application/json", b"{\"error\":\"too_many_preset_params\"}".to_vec());
            }
            d
        }
        Err(e) => {
            return (
                400,
                "application/json",
                format!("{{\"error\":\"invalid_preset\",\"message\":\"{}\"}}", e).into_bytes(),
            );
        }
    };

    if let Some(ref params) = doc.params {
        if let Err(e) = apply_preset_params_queued(params, &ctx.app_state, &ctx.param_queue, &ctx.dsp_chain) {
            return match e {
                PresetApplyError::Validation(msg) => (
                    400,
                    "application/json",
                    format!("{{\"error\":\"invalid_preset_param\",\"message\":\"{}\"}}", msg).into_bytes(),
                ),
                PresetApplyError::QueueFull => (
                    503,
                    "application/json",
                    br#"{"error":"queue_full","retry_after_ms":5}"#.to_vec(),
                ),
            };
        }
    }

    if let Ok(mut st) = ctx.app_state.lock() {
        st.active_preset_id = doc.metadata.id.clone();
        st.active_preset_name = doc.metadata.name.clone();
        if !doc.scenes.is_empty() {
            st.scenes = doc.scenes.clone();
        } else {
            st.scenes = default_scenes();
        }
        st.grid = doc.grid.clone();
        st.channels = doc.channels.clone().unwrap_or_default();
        if !doc
            .params
            .as_ref()
            .map_or(false, |p| p.contains_key("scene.active"))
        {
            let letters = ["A", "B", "C", "D", "E", "F", "G", "H"];
            if let Some(&l) = letters.get(doc.active_scene_index) {
                st.scene_active = l.to_string();
            }
        }
    }

    json_response(
        200,
        &serde_json::json!({
            "status": "ok",
            "imported": doc.metadata.id
        }),
    )
}

fn handle_get_presets_export(preset_id: &str) -> (u16, &'static str, Vec<u8>) {
    if !is_valid_preset_id(preset_id) {
        return (400, "application/json", b"{\"error\":\"invalid_preset_id\"}".to_vec());
    }
    let Some(path) = find_preset_file(preset_id) else {
        return (404, "application/json", b"{\"error\":\"preset_not_found\"}".to_vec());
    };
    let Ok(bytes) = std::fs::read(path) else {
        return (500, "application/json", b"{\"error\":\"failed_to_read\"}".to_vec());
    };
    (200, "application/json", bytes)
}

fn format_utc_iso8601_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86400);
    let sod = secs.rem_euclid(86400);
    let hh = sod / 3600;
    let mm = (sod % 3600) / 60;
    let ss = sod % 60;

    // Howard Hinnant civil_from_days
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, hh, mm, ss)
}

fn handle_post_scenes_select(
    body: &[u8],
    ctx: &ApiContext,
) -> (u16, &'static str, Vec<u8>) {
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) else {
        return (400, "application/json", b"{\"error\":\"invalid_json\"}".to_vec());
    };
    let letters = ["A", "B", "C", "D", "E", "F", "G", "H"];
    let (scene, scene_idx) = if let Some(s) = v.get("scene").and_then(|s| s.as_str()) {
        let Some(idx) = letters.iter().position(|&l| l == s) else {
            return (400, "application/json", b"{\"error\":\"invalid_scene\"}".to_vec());
        };
        (s.to_string(), idx)
    } else if let Some(idx_u) = v.get("index").and_then(|x| x.as_u64()) {
        if idx_u >= 8 {
            return (400, "application/json", b"{\"error\":\"scene_index_out_of_range\"}".to_vec());
        }
        (letters[idx_u as usize].to_string(), idx_u as usize)
    } else {
        return (400, "application/json", b"{\"error\":\"missing_scene\"}".to_vec());
    };

    let start = std::time::Instant::now();
    let mut scene_params: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();
    scene_params.insert(
        "scene.active".to_string(),
        serde_json::Value::String(scene.to_string()),
    );

    if let Ok(st) = ctx.app_state.lock() {
        let ch_letters = ["A", "B", "C", "D"];
        for block in &st.grid.blocks {
            let bypass = block.bypass_by_scene.get(scene_idx).copied().unwrap_or(false);
            scene_params.insert(
                format!("{}.enabled", block.instance_id),
                serde_json::Value::Bool(!bypass),
            );
            let ch_idx = block
                .active_channel_by_scene
                .get(scene_idx)
                .copied()
                .unwrap_or(0);
            if let Some(&ch_letter) = ch_letters.get(ch_idx) {
                if let Some(block_channels) = st.channels.get(&block.instance_id) {
                    if let Some(ch_params) = block_channels.get(ch_letter) {
                        for (k, val) in ch_params {
                            scene_params.insert(k.clone(), val.clone());
                        }
                    }
                }
            }
        }
    }

    match apply_preset_params_queued(&scene_params, &ctx.app_state, &ctx.param_queue, &ctx.dsp_chain)
    {
        Ok(()) => {}
        Err(PresetApplyError::Validation(msg)) => {
            return json_error_response(400, &msg);
        }
        Err(PresetApplyError::QueueFull) => {
            return (
                503,
                "application/json",
                br#"{"error":"queue_full","retry_after_ms":5}"#.to_vec(),
            );
        }
    }

    let elapsed_ms = (start.elapsed().as_micros() as f32 / 1000.0).max(0.1);
    let state_json = ctx
        .app_state
        .lock()
        .ok()
        .and_then(|st| serde_json::to_value(&*st).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    json_response(
        200,
        &serde_json::json!({
            "status": "ok",
            "scene": scene,
            "state": state_json,
            "transition_ms": (elapsed_ms * 10.0).round() / 10.0
        }),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresetApplyError {
    Validation(String),
    QueueFull,
}

fn validate_preset_param_kv(k: &str, v: &serde_json::Value) -> Result<(), String> {
    let is_known = matches!(
        k,
        "amp_1.gain"
            | "amp_1.overdrive"
            | "amp_1.bass"
            | "amp_1.middle"
            | "amp_1.treble"
            | "amp_1.presence"
            | "amp_1.depth"
            | "amp_1.master"
            | "amp_1.volume_2"
            | "amp_1.cut"
            | "amp_1.reverb"
            | "amp_1.vibrato_speed"
            | "amp_1.vibrato_depth"
            | "amp_1.power"
            | "amp_1.standby"
            | "amp_1.jumpered"
            | "amp_1.spongy"
            | "amp_1.input_jack"
            | "amp_1.channel_mode"
            | "amp_1.impedance_ohms"
            | "amp_1.model"
            | "amp_1.bright"
            | "amp_1.enabled"
            | "amp_1.bias_ma"
            | "amp_1.sag"
            | "amp_1.nfb"
            | "amp_1.xformer_sat"
            | "amp_1.preamp_plate_v"
            | "amp_1.triode_asym"
            | "amp_1.preamp_tube"
            | "amp_1.power_tube"
            | "amp_1.rectifier"
            | "amp_1.tonestack"
            | "amp_1.is_modded"
            | "is_modded"
            | "engine.oversampling"
            | "input.boost"
            | "input.source"
            | "input.mode"
            | "drive.enabled"
            | "drive.model"
            | "drive.drive"
            | "drive.tone"
            | "drive.level"
            | "drive.bias"
            | "gate.enabled"
            | "gate.threshold_db"
            | "gate.mode"
            | "gate.hysteresis_db"
            | "gate.attack_ms"
            | "gate.hold_ms"
            | "gate.release_ms"
            | "gate.range_db"
            | "gate.key_hpf_hz"
            | "gate.key_lpf_hz"
            | "cab_1.enabled"
            | "cab_1.speaker"
            | "cab_1.model"
            | "cab_1.resonance_hz"
            | "cab_1.hicut_khz"
            | "cab_1.ir_enabled"
            | "cab_1.mic1.model"
            | "cab_1.mic1.position"
            | "cab_1.mic1.distance_cm"
            | "cab_1.mic1.level_db"
            | "cab_1.mic1.enabled"
            | "cab_1.mic1.angle_deg"
            | "cab_1.mic1.phase_invert"
            | "cab_1.mic2.model"
            | "cab_1.mic2.position"
            | "cab_1.mic2.distance_cm"
            | "cab_1.mic2.level_db"
            | "cab_1.mic2.enabled"
            | "cab_1.mic2.angle_deg"
            | "cab_1.mic2.phase_invert"
            | "split.enabled"
            | "split.mode"
            | "split.balance"
            | "split.crossover_hz"
            | "merge.row_1_level_db"
            | "merge.row_1_phase_invert"
            | "merge.level_a_db"
            | "merge.level_b_db"
            | "merge.phase_invert_b"
            | "comp_2.enabled"
            | "comp_2.input"
            | "comp_2.output"
            | "comp_2.attack"
            | "comp_2.release"
            | "comp_2.ratio"
            | "eq_1.enabled"
            | "eq_1.mode"
            | "eq_1.gain_db"
            | "eq_1.input_gain_db"
            | "eq_1.volume_db"
            | "eq_1.output_level_db"
            | "eq_1.bands_db"
            | "mod_2.enabled"
            | "mod_2.model"
            | "mod_2.rate_hz"
            | "mod_2.depth"
            | "mod_2.mix"
            | "mod_2.mix_pct"
            | "dly_1.enabled"
            | "dly_1.mode"
            | "dly_1.heads"
            | "dly_1.time_ms"
            | "dly_1.feedback"
            | "dly_1.mix_pct"
            | "dly_1.repeat_rate"
            | "dly_1.intensity"
            | "dly_1.echo_volume"
            | "dly_1.reverb_volume"
            | "dly_1.bass"
            | "dly_1.treble"
            | "dly_1.spillover"
            | "rev_1.enabled"
            | "rev_1.decay_s"
            | "rev_1.predelay_ms"
            | "rev_1.damping"
            | "rev_1.mix"
            | "rev_1.mix_pct"
            | "rev_1.spillover"
            | "amp_2.enabled"
            | "amp_2.model"
            | "amp_2.gain"
            | "amp_2.bass"
            | "amp_2.middle"
            | "amp_2.treble"
            | "amp_2.master"
            | "wah_1.enabled"
            | "wah_1.position"
            | "wah_1.q"
            | "wah_1.mode"
            | "cab_2.enabled"
            | "cab_2.speaker"
            | "cab_2.model"
            | "cab_2.resonance_hz"
            | "cab_2.hicut_khz"
            | "master.level_db"
            | "palco.exp_1"
            | "preset.active"
            | "preset.name"
            | "scene.active"
            | "tuner.enabled"
            | "tuner.output"
            | "tuner.ref_hz"
            | "looper.position"
            | "looper.action"
            | "looper.reverse"
            | "looper.half_speed"
            | "pitch_1.enabled"
            | "pitch_1.model"
            | "pitch_1.semitones"
            | "pitch_1.key"
            | "pitch_1.scale"
            | "pitch_1.mix"
            | "pitch_1.mix_pct"
            | "footswitch.mode"
            | "modifiers"
    ) || (k.starts_with("eq_1.band") && (k.ends_with(".gain_db") || k.ends_with(".freq_hz") || k.ends_with(".q")));

    if !is_known {
        return Err(format!("Chave de parâmetro desconhecida: {}", k));
    }
    if k == "amp_1.model" {
        let Some(m) = v.as_str() else {
            return Err("amp_1.model deve ser string".to_string());
        };
        if !["jcm800_2203", "plexi_1959", "twin_ab763", "dual_rect", "ac30_tb"].contains(&m) {
            return Err(format!("amp_1.model inválido: {}", m));
        }
    }
    if k == "scene.active" {
        let Some(s) = v.as_str() else {
            return Err("scene.active deve ser string".to_string());
        };
        if !["A", "B", "C", "D", "E", "F", "G", "H"].contains(&s) {
            return Err(format!("scene.active inválido: {}", s));
        }
    }
    Ok(())
}

/// Aplica um mapa completo de parâmetros de preset em `AppState` e envia ao motor via fila SPSC sem bloquear o DSP.
pub fn apply_preset_params_queued(
    params: &std::collections::HashMap<String, serde_json::Value>,
    app_state: &Arc<Mutex<AppState>>,
    param_queue: &Option<Arc<LockFreeParamQueue>>,
    dsp_chain: &Option<Arc<Mutex<RealtimeDspChain>>>,
) -> Result<(), PresetApplyError> {
    for (k, v) in params {
        validate_preset_param_kv(k.as_str(), v).map_err(PresetApplyError::Validation)?;
    }

    let mut updates = Vec::with_capacity(params.len() + 2);
    updates.push(DspParamUpdate::BeginPresetCrossfade);
    for (k, v) in params {
        json_kv_to_dsp_updates(k.as_str(), v, &mut updates);
    }

    if let Some(ref pq) = param_queue {
        pq.push_batch(&updates).map_err(|_| PresetApplyError::QueueFull)?;
    } else if let Some(ref dsp_ref) = dsp_chain {
        if let Ok(mut dsp) = dsp_ref.lock() {
            for u in &updates {
                dsp.apply_param_update(*u);
            }
        }
    }

    if let Ok(mut st) = app_state.lock() {
        for (k, v) in params {
            match k.as_str() {
                "amp_1.gain" => if let Some(n) = v.as_f64() { st.amp_gain = n as f32; },
                "amp_1.overdrive" => if let Some(n) = v.as_f64() { st.amp_overdrive = n as f32; },
                "amp_1.bass" => if let Some(n) = v.as_f64() { st.amp_bass = n as f32; },
                "amp_1.middle" => if let Some(n) = v.as_f64() { st.amp_middle = n as f32; },
                "amp_1.treble" => if let Some(n) = v.as_f64() { st.amp_treble = n as f32; },
                "amp_1.presence" => if let Some(n) = v.as_f64() { st.amp_presence = n as f32; },
                "amp_1.depth" => if let Some(n) = v.as_f64() { st.amp_depth = n as f32; },
                "amp_1.master" => if let Some(n) = v.as_f64() { st.amp_master = n as f32; },
                "amp_1.volume_2" => if let Some(n) = v.as_f64() { st.amp_volume_2 = n as f32; },
                "amp_1.cut" => if let Some(n) = v.as_f64() { st.amp_cut = n as f32; },
                "amp_1.reverb" => if let Some(n) = v.as_f64() { st.amp_reverb = n as f32; },
                "amp_1.vibrato_speed" => if let Some(n) = v.as_f64() { st.amp_vibrato_speed = n as f32; },
                "amp_1.vibrato_depth" => if let Some(n) = v.as_f64() { st.amp_vibrato_depth = n as f32; },
                "amp_1.power" => if let Some(b) = v.as_bool() { st.amp_power = b; },
                "amp_1.standby" => if let Some(b) = v.as_bool() { st.amp_standby = b; },
                "amp_1.jumpered" => if let Some(b) = v.as_bool() { st.amp_jumpered = b; },
                "amp_1.spongy" => if let Some(b) = v.as_bool() { st.amp_spongy = b; },
                "amp_1.input_jack" => if let Some(s) = v.as_str() { st.amp_input_jack = s.to_string(); },
                "amp_1.channel_mode" => if let Some(s) = v.as_str() { st.amp_channel_mode = s.to_string(); },
                "amp_1.impedance_ohms" => if let Some(s) = v.as_str() { st.amp_impedance_ohms = s.to_string(); },
                "amp_1.model" => if let Some(s) = v.as_str() { st.amp_model = s.to_string(); },
                "amp_1.bright" => if let Some(b) = v.as_bool() { st.amp_bright = b; },
                "amp_1.enabled" => if let Some(b) = v.as_bool() { st.amp_enabled = b; },
                "amp_1.bias_ma" => if let Some(n) = v.as_f64() { st.bias_ma = n as f32; },
                "amp_1.sag" => if let Some(n) = v.as_f64() { st.sag = n as f32; },
                "amp_1.nfb" => if let Some(n) = v.as_f64() { st.nfb = n as f32; },
                "amp_1.xformer_sat" => if let Some(n) = v.as_f64() { st.xformer_sat = n as f32; },
                "amp_1.preamp_plate_v" => if let Some(n) = v.as_f64() { st.preamp_plate_v = n as f32; },
                "amp_1.triode_asym" => if let Some(n) = v.as_f64() { st.triode_asym = n as f32; },
                "amp_1.preamp_tube" => if let Some(s) = v.as_str() { st.preamp_tube = s.to_string(); },
                "amp_1.power_tube" => if let Some(s) = v.as_str() { st.power_tube = s.to_string(); },
                "amp_1.rectifier" => if let Some(s) = v.as_str() { st.rectifier = s.to_string(); },
                "amp_1.tonestack" => if let Some(s) = v.as_str() { st.tonestack = s.to_string(); },
                "engine.oversampling" => if let Some(s) = v.as_str() { st.engine_oversampling = s.to_string(); },
                "input.boost" => if let Some(n) = v.as_f64() { st.input_boost = n as f32; },
                "input.source" => if let Some(s) = v.as_str() { st.input_source = s.to_string(); },
                "input.mode" => if let Some(s) = v.as_str() { st.input_mode = s.to_string(); },
                "drive.enabled" => if let Some(b) = v.as_bool() { st.drive_enabled = b; },
                "drive.model" => if let Some(s) = v.as_str() { st.drive_model = s.to_string(); },
                "drive.drive" => if let Some(n) = v.as_f64() { st.drive_drive = n as f32; },
                "drive.tone" => if let Some(n) = v.as_f64() { st.drive_tone = n as f32; },
                "drive.level" => if let Some(n) = v.as_f64() { st.drive_level = n as f32; },
                "drive.bias" => if let Some(n) = v.as_f64() { st.drive_bias = n as f32; },
                "gate.enabled" => if let Some(b) = v.as_bool() { st.gate_enabled = b; },
                "gate.threshold_db" => if let Some(n) = v.as_f64() { st.gate_threshold_db = n as f32; },
                "cab_1.enabled" => if let Some(b) = v.as_bool() { st.cab_enabled = b; },
                "cab_1.speaker" | "cab_1.model" => if let Some(s) = v.as_str() { st.cab_speaker = s.to_string(); },
                "cab_1.resonance_hz" => if let Some(n) = v.as_f64() { st.cab_resonance_hz = n as f32; },
                "cab_1.hicut_khz" => if let Some(n) = v.as_f64() { st.cab_hicut_khz = n as f32; },
                "cab_1.ir_enabled" => if let Some(b) = v.as_bool() { st.cab_ir_enabled = b; },
                "cab_1.mic1.model" => if let Some(s) = v.as_str() { st.cab_mic1_model = s.to_string(); },
                "cab_1.mic1.position" => if let Some(n) = v.as_f64() { st.cab_mic1_position = n as f32; },
                "cab_1.mic1.distance_cm" => if let Some(n) = v.as_f64() { st.cab_mic1_distance_cm = n as f32; },
                "cab_1.mic1.level_db" => if let Some(n) = v.as_f64() { st.cab_mic1_level_db = n as f32; },
                "cab_1.mic1.enabled" => if let Some(b) = v.as_bool() { st.cab_mic1_enabled = b; },
                "cab_1.mic2.model" => if let Some(s) = v.as_str() { st.cab_mic2_model = s.to_string(); },
                "cab_1.mic2.position" => if let Some(n) = v.as_f64() { st.cab_mic2_position = n as f32; },
                "cab_1.mic2.distance_cm" => if let Some(n) = v.as_f64() { st.cab_mic2_distance_cm = n as f32; },
                "cab_1.mic2.level_db" => if let Some(n) = v.as_f64() { st.cab_mic2_level_db = n as f32; },
                "cab_1.mic2.enabled" => if let Some(b) = v.as_bool() { st.cab_mic2_enabled = b; },
                "split.enabled" => if let Some(b) = v.as_bool() { st.split_enabled = b; },
                "split.mode" => if let Some(s) = v.as_str() { st.split_mode = s.to_string(); },
                "split.crossover_hz" => if let Some(n) = v.as_f64() { st.split_crossover_hz = n as f32; },
                "merge.row_1_level_db" => if let Some(n) = v.as_f64() { st.merge_row_1_level_db = n as f32; },
                "merge.row_1_phase_invert" => if let Some(b) = v.as_bool() { st.merge_row_1_phase_invert = b; },
                "comp_2.enabled" => if let Some(b) = v.as_bool() { st.comp_2_enabled = b; },
                "comp_2.input" => if let Some(n) = v.as_f64() { st.comp_2_input = n as f32; },
                "comp_2.output" => if let Some(n) = v.as_f64() { st.comp_2_output = n as f32; },
                "comp_2.attack" => if let Some(n) = v.as_f64() { st.comp_2_attack = n as f32; },
                "comp_2.release" => if let Some(n) = v.as_f64() { st.comp_2_release = n as f32; },
                "comp_2.ratio" => if let Some(s) = v.as_str() { st.comp_2_ratio = s.to_string(); },
                "eq_1.enabled" => if let Some(b) = v.as_bool() { st.eq_1_enabled = b; },
                "eq_1.mode" => if let Some(s) = v.as_str() { st.eq_1_mode = s.to_string(); },
                "eq_1.gain_db" | "eq_1.input_gain_db" => if let Some(n) = v.as_f64() { st.eq_1_gain_db = n as f32; },
                "eq_1.volume_db" | "eq_1.output_level_db" => if let Some(n) = v.as_f64() { st.eq_1_volume_db = n as f32; },
                "mod_2.enabled" => if let Some(b) = v.as_bool() { st.mod_2_enabled = b; },
                "mod_2.model" => if let Some(s) = v.as_str() { if parse_mod_model(s).is_some() { st.mod_2_model = s.to_string(); } },
                "mod_2.rate_hz" => if let Some(n) = v.as_f64() { st.mod_2_rate_hz = n as f32; },
                "mod_2.depth" => if let Some(n) = v.as_f64() { st.mod_2_depth = n as f32; },
                "mod_2.mix" | "mod_2.mix_pct" => if let Some(n) = v.as_f64() { st.mod_2_mix = n as f32; },
                "pitch_1.enabled" => if let Some(b) = v.as_bool() { st.pitch_1_enabled = b; },
                "pitch_1.model" => if let Some(s) = v.as_str() { if parse_pitch_model(s).is_some() { st.pitch_1_model = s.to_string(); } },
                "pitch_1.semitones" => if let Some(n) = v.as_f64() { st.pitch_1_semitones = n as f32; },
                "pitch_1.key" => if let Some(s) = v.as_str() { if parse_pitch_key(s).is_some() { st.pitch_1_key = s.to_string(); } },
                "pitch_1.scale" => if let Some(s) = v.as_str() { if parse_pitch_scale(s).is_some() { st.pitch_1_scale = s.to_string(); } },
                "pitch_1.mix" | "pitch_1.mix_pct" => if let Some(n) = v.as_f64() { st.pitch_1_mix = n as f32; },
                "dly_1.enabled" => if let Some(b) = v.as_bool() { st.dly_1_enabled = b; },
                "dly_1.mode" => if let Some(s) = v.as_str() { st.dly_1_mode = s.to_string(); },
                "dly_1.heads" => if let Some(s) = v.as_str() { st.dly_1_heads = s.to_string(); },
                "dly_1.repeat_rate" => if let Some(n) = v.as_f64() { st.dly_1_repeat_rate = n as f32; },
                "dly_1.intensity" => if let Some(n) = v.as_f64() { st.dly_1_intensity = n as f32; },
                "dly_1.echo_volume" => if let Some(n) = v.as_f64() { st.dly_1_echo_volume = n as f32; },
                "dly_1.reverb_volume" => if let Some(n) = v.as_f64() { st.dly_1_reverb_volume = n as f32; },
                "dly_1.bass" => if let Some(n) = v.as_f64() { st.dly_1_bass = n as f32; },
                "dly_1.treble" => if let Some(n) = v.as_f64() { st.dly_1_treble = n as f32; },
                "dly_1.spillover" => if let Some(b) = v.as_bool() { st.dly_1_spillover = b; },
                "rev_1.enabled" => if let Some(b) = v.as_bool() { st.rev_1_enabled = b; },
                "rev_1.decay_s" => if let Some(n) = v.as_f64() { st.rev_1_decay_s = n as f32; },
                "rev_1.predelay_ms" => if let Some(n) = v.as_f64() { st.rev_1_predelay_ms = n as f32; },
                "rev_1.damping" => if let Some(n) = v.as_f64() { st.rev_1_damping = n as f32; },
                "rev_1.mix" | "rev_1.mix_pct" => if let Some(n) = v.as_f64() { st.rev_1_mix = n as f32; },
                "rev_1.spillover" => if let Some(b) = v.as_bool() { st.rev_1_spillover = b; },
                "amp_2.enabled" => if let Some(b) = v.as_bool() { st.amp_2_enabled = b; },
                "amp_2.model" => if let Some(s) = v.as_str() { st.amp_2_model = s.to_string(); },
                "amp_2.gain" => if let Some(n) = v.as_f64() { st.amp_2_gain = n as f32; },
                "amp_2.bass" => if let Some(n) = v.as_f64() { st.amp_2_bass = n as f32; },
                "amp_2.middle" => if let Some(n) = v.as_f64() { st.amp_2_middle = n as f32; },
                "amp_2.treble" => if let Some(n) = v.as_f64() { st.amp_2_treble = n as f32; },
                "amp_2.master" => if let Some(n) = v.as_f64() { st.amp_2_master = n as f32; },
                "wah_1.enabled" => if let Some(b) = v.as_bool() { st.wah_1_enabled = b; },
                "wah_1.position" => if let Some(n) = v.as_f64() { st.wah_1_position = n as f32; },
                "wah_1.q" => if let Some(n) = v.as_f64() { st.wah_1_q = n as f32; },
                "wah_1.mode" => if let Some(s) = v.as_str() { st.wah_1_mode = s.to_string(); },
                "cab_2.enabled" => if let Some(b) = v.as_bool() { st.cab_2_enabled = b; },
                "cab_2.speaker" | "cab_2.model" => if let Some(s) = v.as_str() { st.cab_2_speaker = s.to_string(); },
                "cab_2.resonance_hz" => if let Some(n) = v.as_f64() { st.cab_2_resonance_hz = n as f32; },
                "cab_2.hicut_khz" => if let Some(n) = v.as_f64() { st.cab_2_hicut_khz = n as f32; },
                "master.level_db" => if let Some(n) = v.as_f64() { st.master_level_db = n as f32; },
                "palco.exp_1" => if let Some(n) = v.as_f64() { st.palco_exp_1 = n as f32; },
                "scene.active" => if let Some(s) = v.as_str() { st.scene_active = s.to_string(); },
                "footswitch.mode" => if let Some(s) = v.as_str() { st.footswitch_mode = s.to_string(); },
                "tuner.enabled" => if let Some(b) = v.as_bool() { st.tuner_enabled = b; },
                "tuner.output" => if let Some(s) = v.as_str() { st.tuner_output = s.to_string(); },
                "tuner.ref_hz" => if let Some(n) = v.as_f64() { st.tuner_ref_hz = n as f32; },
                "looper.position" => if let Some(s) = v.as_str() { st.looper_position = s.to_string(); },
                "cab_1.mic1.angle_deg" => if let Some(deg) = parse_mic_angle_deg(v) {
                    st.cab_mic1_angle_deg = if deg >= 22.5 { "45".to_string() } else { "0".to_string() };
                },
                "cab_1.mic1.phase_invert" => if let Some(b) = v.as_bool() { st.cab_mic1_phase_invert = b; },
                "cab_1.mic2.angle_deg" => if let Some(deg) = parse_mic_angle_deg(v) {
                    st.cab_mic2_angle_deg = if deg >= 22.5 { "45".to_string() } else { "0".to_string() };
                },
                "cab_1.mic2.phase_invert" => if let Some(b) = v.as_bool() { st.cab_mic2_phase_invert = b; },
                k if k.starts_with("eq_1.band") && k.ends_with(".gain_db") => {
                    if let (Some(n), Some(val)) = (
                        k["eq_1.band".len()..].split('.').next().and_then(|s| s.parse::<usize>().ok()),
                        v.as_f64(),
                    ) {
                        if (1..=10).contains(&n) {
                            st.eq_1_bands_db[n - 1] = val as f32;
                        }
                    }
                }
                _ => {}
            }
        }
        st.is_modded = st.check_is_modded();
    }
    Ok(())
}

/// Aplica um mapa completo de parâmetros de preset em `AppState` e no `RealtimeDspChain`.
pub fn apply_preset_params(
    params: &std::collections::HashMap<String, serde_json::Value>,
    app_state: &Arc<Mutex<AppState>>,
    dsp_chain: &Option<Arc<Mutex<RealtimeDspChain>>>,
) {
    let pq = dsp_chain
        .as_ref()
        .and_then(|d| d.lock().ok().map(|chain| chain.param_queue()));
    let _ = apply_preset_params_queued(params, app_state, &pq, dsp_chain);
}

/// Executa captura real de ruído/palhetada no ring lock-free e aplica calibração de entrada (Spec 029).
pub fn handle_post_input_calibrate(body: &[u8], ctx: &ApiContext) -> (u16, &'static str, Vec<u8>) {
    let body_str = String::from_utf8_lossy(body);
    let parsed: serde_json::Value = match serde_json::from_str(&body_str) {
        Ok(v) => v,
        Err(_) => return (400, "application/json", br#"{"error":"JSON invalido"}"#.to_vec()),
    };

    let phase = parsed.get("phase").and_then(|v| v.as_str()).unwrap_or("");
    let sr = ctx
        .clean_input_ring
        .as_ref()
        .map(|r| r.sample_rate())
        .unwrap_or(48000.0);

    match phase {
        // Spec 051 T-01: marca o início da captura do assistente (descarta o que veio antes).
        "begin" => {
            let Ok(mut sess) = ctx.calibration_session.lock() else {
                return json_error_response(500, "E_INTERNAL: calibration_session lock failed");
            };
            sess.noise_samples.clear();
            sess.cursor_pos = ctx
                .clean_input_ring
                .as_ref()
                .map(|r| r.total_written())
                .unwrap_or(0);
            let resp = serde_json::json!({ "phase": "begin", "cursor": sess.cursor_pos });
            json_response(200, &resp)
        }
        "silence" => {
            let samples = if let Some(ref ring) = ctx.clean_input_ring {
                let Ok(mut sess) = ctx.calibration_session.lock() else {
                    return json_error_response(500, "E_INTERNAL: calibration_session lock failed");
                };
                let max_samples = (sr * 3.0).clamp(1024.0, 200_000.0) as usize;
                let captured = ring.read_range(sess.cursor_pos, max_samples);
                sess.cursor_pos = ring.total_written();
                sess.noise_samples = captured;
                sess.noise_samples.len()
            } else {
                0
            };
            let noise_rms_db = {
                let Ok(sess) = ctx.calibration_session.lock() else {
                    return json_error_response(500, "E_INTERNAL: calibration_session lock failed");
                };
                if sess.noise_samples.is_empty() {
                    -90.0f32
                } else {
                    let sq: f32 = sess.noise_samples.iter().map(|&s| s * s).sum();
                    let rms = (sq / sess.noise_samples.len() as f32).sqrt().max(1e-6);
                    20.0 * rms.log10()
                }
            };
            let resp = serde_json::json!({
                "phase": "silence",
                "captured_samples": samples,
                "noise_floor_rms_db": noise_rms_db,
            });
            json_response(200, &resp)
        }
        "strum" => {
            let (noise_samples, strum_samples) = {
                let Ok(mut sess) = ctx.calibration_session.lock() else {
                    return json_error_response(500, "E_INTERNAL: calibration_session lock failed");
                };
                let strum = if let Some(ref ring) = ctx.clean_input_ring {
                    let max_samples = (sr * 5.0).clamp(1024.0, 260_000.0) as usize;
                    let captured = ring.read_range(sess.cursor_pos, max_samples);
                    sess.cursor_pos = ring.total_written();
                    captured
                } else {
                    Vec::new()
                };
                (sess.noise_samples.clone(), strum)
            };

            let result = calibrate_input(&noise_samples, &strum_samples, sr);

            if let Ok(mut st) = ctx.app_state.lock() {
                st.input_boost = result.proposed_boost;
                st.gate_threshold_db = result.proposed_gate_threshold_db;
                st.amp_standby = false;
            }
            if let Some(ref pq) = ctx.param_queue {
                let _ = pq.push(DspParamUpdate::InputBoost(result.proposed_boost));
                let _ = pq.push(DspParamUpdate::GateThreshold(result.proposed_gate_threshold_db));
                let _ = pq.push(DspParamUpdate::AmpStandby(false));
            } else if let Some(ref dsp_ref) = ctx.dsp_chain {
                if let Ok(mut dsp) = dsp_ref.try_lock() {
                    dsp.set_input_boost(result.proposed_boost);
                    dsp.set_gate_threshold_db(result.proposed_gate_threshold_db);
                    dsp.set_amp_standby(false);
                }
            }
            if let Some(ref store) = ctx.calibration_store {
                let dev_id = ctx.active_device_id.as_deref().unwrap_or("default");
                store.save(dev_id, result.proposed_boost, result.proposed_gate_threshold_db);
            }

            json_response(200, &result)
        }
        _ => (
            400,
            "application/json",
            br#"{"error":"phase deve ser 'begin', 'silence' ou 'strum'"}"#.to_vec(),
        ),
    }
}

fn reamp_sessions_dir(ctx: &ApiContext) -> std::path::PathBuf {
    if let Some(ref dir) = ctx.data_dir {
        return dir.join("reamp");
    }
    if let Some(ref presets_dir) = ctx.custom_presets_dir {
        let base = presets_dir.parent().unwrap_or(presets_dir);
        return base.join("reamp");
    }
    if let Ok(appdata) = std::env::var("APPDATA") {
        return std::path::PathBuf::from(appdata).join("SonixPro").join("reamp");
    }
    std::env::temp_dir().join("sonix-pro").join("reamp")
}

fn handle_post_reamp_run(body: &[u8], ctx: &ApiContext) -> (u16, &'static str, Vec<u8>) {
    let val: serde_json::Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return (400, "application/json", br#"{"error":"JSON invalido"}"#.to_vec()),
    };

    if val.get("session_path").is_some() {
        return (
            400,
            "application/json",
            br#"{"error":"session_path nao e permitido; utilize session_name"}"#.to_vec(),
        );
    }

    let session_file_stem = if let Some(raw_name) = val.get("session_name") {
        let Some(name_str) = raw_name.as_str() else {
            return (400, "application/json", br#"{"error":"session_name invalido"}"#.to_vec());
        };
        let valid_name = !name_str.is_empty()
            && name_str.len() <= 64
            && name_str
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
        if !valid_name {
            return (400, "application/json", br#"{"error":"session_name invalido"}"#.to_vec());
        }
        name_str.to_string()
    } else {
        let unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        format!("sessao-{}", unix_ms)
    };

    let parse_f32_vec_bounded = |key: &str| -> Result<Option<Vec<f32>>, &'static str> {
        let Some(v) = val.get(key) else {
            return Ok(None);
        };
        let Some(arr) = v.as_array() else {
            return Err("vetor_invalido");
        };
        if arr.len() > crate::reamp::MAX_REAMP_SAMPLES {
            return Err("vetor_excede_limite");
        }
        let mut out = Vec::new();
        if out.try_reserve_exact(arr.len()).is_err() {
            return Err("memoria_insuficiente");
        }
        for item in arr {
            let Some(num) = item.as_f64() else {
                return Err("amostra_nao_numerica");
            };
            if !num.is_finite() {
                return Err("amostra_nao_finita");
            }
            out.push(num as f32);
        }
        Ok(Some(out))
    };

    let probe = match parse_f32_vec_bounded("probe") {
        Ok(v) => v.unwrap_or_default(),
        Err(msg) => {
            let body = serde_json::to_vec(&serde_json::json!({ "error": msg })).unwrap_or_default();
            return (400, "application/json", body);
        }
    };
    let loopback_rec = match parse_f32_vec_bounded("loopback_rec") {
        Ok(v) => v.unwrap_or_default(),
        Err(msg) => {
            let body = serde_json::to_vec(&serde_json::json!({ "error": msg })).unwrap_or_default();
            return (400, "application/json", body);
        }
    };
    let reamp_rec = match parse_f32_vec_bounded("reamp_rec") {
        Ok(v) => v.unwrap_or_default(),
        Err(msg) => {
            let body = serde_json::to_vec(&serde_json::json!({ "error": msg })).unwrap_or_default();
            return (400, "application/json", body);
        }
    };

    let di_len = if let Some(raw_len) = val.get("di_len") {
        let Some(n_u64) = raw_len.as_u64() else {
            return (400, "application/json", br#"{"error":"di_len invalido"}"#.to_vec());
        };
        let Ok(n_usize) = usize::try_from(n_u64) else {
            return (400, "application/json", br#"{"error":"di_len fora do limite"}"#.to_vec());
        };
        if n_usize > reamp_rec.len() || n_usize > crate::reamp::MAX_REAMP_SAMPLES {
            return (
                400,
                "application/json",
                br#"{"error":"di_len excede reamp_rec.len() ou limite maximo"}"#.to_vec(),
            );
        }
        n_usize
    } else {
        reamp_rec.len()
    };

    let di_source = val
        .get("di_source")
        .and_then(|v| v.as_str())
        .unwrap_or("di_buffer")
        .to_string();

    if probe.is_empty() || loopback_rec.is_empty() {
        return (
            400,
            "application/json",
            br#"{"error":"probe e loopback_rec obrigatorios para medir atraso de loopback"}"#.to_vec(),
        );
    }

    let measured_delay_samples =
        crate::reamp::measure_loopback_delay_samples(&probe, &loopback_rec);
    let aligned =
        crate::reamp::align_reamp_recording(&reamp_rec, measured_delay_samples, di_len);

    let (backend, driver, sample_rate, buffer_size) = ctx
        .audio_telemetry
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .map(|t| (t.backend, t.mode, t.sample_rate, t.output_period_frames))
        .unwrap_or_else(|| ("wasapi".to_string(), "wasapi_shared".to_string(), 48000, 128));

    let metadata = crate::reamp::ReampSessionMetadata {
        measured_delay_samples,
        backend,
        driver,
        sample_rate,
        buffer_size,
        di_source,
    };

    let session_file = reamp_sessions_dir(ctx).join(format!("{}.json", session_file_stem));
    if let Err(e) = crate::reamp::save_reamp_session_metadata(&session_file, &metadata) {
        let body = serde_json::to_vec(&serde_json::json!({ "error": e })).unwrap_or_default();
        return (500, "application/json", body);
    }

    let resp = serde_json::json!({
        "ok": true,
        "measured_delay_samples": measured_delay_samples,
        "aligned_samples": aligned,
        "metadata": metadata,
        "session_file": session_file.to_string_lossy(),
        "notice": "Normalmente é necessário um reamp box entre a saída de linha e a entrada do amplificador."
    });
    let body = serde_json::to_vec(&resp).unwrap_or_default();
    (200, "application/json", body)
}

fn setlist_store_path(custom_dir: Option<&std::path::Path>) -> std::path::PathBuf {
    if let Some(dir) = custom_dir {
        dir.join("setlists.json")
    } else if let Ok(appdata) = std::env::var("APPDATA") {
        std::path::PathBuf::from(appdata).join("SonixPro").join("setlists.json")
    } else {
        std::path::PathBuf::from("presets/user/setlists.json")
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct SetlistStoreFile {
    active_setlist_id: Option<String>,
    setlists: Vec<SetlistDto>,
}

fn read_setlist_store(custom_dir: Option<&std::path::Path>) -> SetlistStoreFile {
    let p = setlist_store_path(custom_dir);
    std::fs::read_to_string(&p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_setlist_store(custom_dir: Option<&std::path::Path>, store: &SetlistStoreFile) -> Result<(), String> {
    let p = setlist_store_path(custom_dir);
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    std::fs::write(&p, json).map_err(|e| e.to_string())
}

fn handle_get_setlist(ctx: &ApiContext) -> (u16, &'static str, Vec<u8>) {
    let store = read_setlist_store(ctx.custom_presets_dir.as_deref());
    let active = store
        .active_setlist_id
        .as_ref()
        .and_then(|id| store.setlists.iter().find(|s| &s.id == id))
        .or_else(|| store.setlists.first())
        .cloned();
    let active_idx = ctx.app_state.lock().map(|s| s.setlist_active_index).unwrap_or(0);
    json_response(
        200,
        &serde_json::json!({
            "active_setlist": active,
            "active_index": active_idx,
            "setlists": store.setlists,
        }),
    )
}

fn handle_post_setlist(body: &[u8], ctx: &ApiContext) -> (u16, &'static str, Vec<u8>) {
    let Ok(dto) = serde_json::from_slice::<SetlistDto>(body) else {
        return (400, "application/json", b"{\"error\":\"invalid_setlist_json\"}".to_vec());
    };
    if dto.id.trim().is_empty() || dto.id.len() > 128 || dto.presets.is_empty() || dto.presets.len() > 256 {
        return (400, "application/json", b"{\"error\":\"setlist_id_ou_presets_vazios_ou_excede_limite\"}".to_vec());
    }
    for pid in &dto.presets {
        if !preset_exists_in(ctx.custom_presets_dir.as_deref(), pid) {
            return json_error_response(400, &format!("Preset inexistente no setlist: {}", pid));
        }
    }
    let mut store = read_setlist_store(ctx.custom_presets_dir.as_deref());
    if let Some(existing) = store.setlists.iter_mut().find(|s| s.id == dto.id) {
        *existing = dto.clone();
    } else {
        store.setlists.push(dto.clone());
    }
    store.active_setlist_id = Some(dto.id.clone());
    if let Err(e) = write_setlist_store(ctx.custom_presets_dir.as_deref(), &store) {
        return json_error_response(500, &e);
    }
    if let Ok(mut st) = ctx.app_state.lock() {
        st.setlist_active_index = 0;
    }
    json_response(200, &serde_json::json!({"ok": true, "setlist": dto}))
}

fn handle_post_setlist_step(delta: isize, ctx: &ApiContext) -> (u16, &'static str, Vec<u8>) {
    let store = read_setlist_store(ctx.custom_presets_dir.as_deref());
    let Some(active) = store
        .active_setlist_id
        .as_ref()
        .and_then(|id| store.setlists.iter().find(|s| &s.id == id))
        .or_else(|| store.setlists.first())
    else {
        return (404, "application/json", b"{\"error\":\"no_active_setlist\"}".to_vec());
    };
    if active.presets.is_empty() {
        return (400, "application/json", b"{\"error\":\"empty_setlist\"}".to_vec());
    }
    let cur_idx = ctx.app_state.lock().map(|s| s.setlist_active_index).unwrap_or(0);
    let len = active.presets.len() as isize;
    let next_idx = ((cur_idx as isize + delta).rem_euclid(len)) as usize;
    let preset_id = &active.presets[next_idx];
    let Ok(load_body) = serde_json::to_vec(&serde_json::json!({ "id": preset_id })) else {
        return json_error_response(500, "E_INTERNAL: serialization failed");
    };
    let (st, ct, resp) = handle_post_presets_load(&load_body, ctx);
    if st != 200 {
        return (st, ct, resp);
    }
    if let Ok(mut st_lock) = ctx.app_state.lock() {
        st_lock.setlist_active_index = next_idx;
    }
    json_response(
        200,
        &serde_json::json!({
            "ok": true,
            "index": next_idx,
            "preset_id": preset_id,
            "loaded": true,
        }),
    )
}

fn compile_and_push_modifiers(modifiers: &[ModifierBindingDto], ctx: &ApiContext) {
    let mut table = crate::audio::CompiledModifierTable::empty();
    for m in modifiers.iter().take(16) {
        if let (Some((target, _, _)), Some(source)) = (
            crate::audio::ModifierTargetParam::from_str_name(&m.target_param),
            crate::audio::CompiledModifierSource::from_str_name(&m.source),
        ) {
            let idx = table.len;
            table.bindings[idx] = crate::audio::CompiledModifierBinding {
                active: true,
                target,
                source,
                min_val: m.min_val,
                max_val: m.max_val,
                curve_slope: m.curve_slope,
                damping_ms: m.damping_ms,
            };
            table.len += 1;
        }
    }
    if let Some(ref rq) = ctx.resource_queue {
        let _ = rq.push_modifiers(Box::new(table));
    }
}

fn handle_get_modifiers(ctx: &ApiContext) -> (u16, &'static str, Vec<u8>) {
    let mods = ctx
        .app_state
        .lock()
        .map(|s| s.modifiers.clone())
        .unwrap_or_default();
    json_response(200, &serde_json::json!({ "modifiers": mods }))
}

fn handle_post_modifiers(body: &[u8], ctx: &ApiContext) -> (u16, &'static str, Vec<u8>) {
    let Ok(dto) = serde_json::from_slice::<ModifierBindingDto>(body) else {
        return (400, "application/json", b"{\"error\":\"invalid_modifier_json\"}".to_vec());
    };
    if dto.id.trim().is_empty() || dto.id.len() > 64 {
        return (400, "application/json", b"{\"error\":\"modifier_id_invalido\"}".to_vec());
    }
    let Some((_, min_bound, max_bound)) = crate::audio::ModifierTargetParam::from_str_name(&dto.target_param) else {
        return json_error_response(
            400,
            &format!("Parâmetro inválido ou discreto para Modifier: {}", dto.target_param),
        );
    };
    if crate::audio::CompiledModifierSource::from_str_name(&dto.source).is_none() {
        return (400, "application/json", b"{\"error\":\"invalid_modifier_source\"}".to_vec());
    }
    if dto.min_val < min_bound || dto.max_val > max_bound || dto.min_val > dto.max_val {
        return json_error_response(
            400,
            &format!("min_val/max_val fora da faixa [{}, {}] de {}", min_bound, max_bound, dto.target_param),
        );
    }
    let snapshot = if let Ok(mut st) = ctx.app_state.lock() {
        if let Some(existing) = st.modifiers.iter_mut().find(|m| m.id == dto.id) {
            *existing = dto.clone();
        } else {
            if st.modifiers.len() >= 64 {
                return (
                    400,
                    "application/json",
                    b"{\"error\":\"limite_modifiers_atingido\"}".to_vec(),
                );
            }
            st.modifiers.push(dto.clone());
        }
        st.modifiers.clone()
    } else {
        return (500, "application/json", b"{\"error\":\"lock_failed\"}".to_vec());
    };
    compile_and_push_modifiers(&snapshot, ctx);
    json_response(200, &serde_json::json!({ "ok": true, "modifiers": snapshot }))
}

fn handle_delete_modifiers(body: &[u8], ctx: &ApiContext) -> (u16, &'static str, Vec<u8>) {
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) else {
        return (400, "application/json", b"{\"error\":\"invalid_json\"}".to_vec());
    };
    let Some(id) = v.get("id").and_then(|x| x.as_str()) else {
        return (400, "application/json", b"{\"error\":\"missing_id\"}".to_vec());
    };
    let snapshot = if let Ok(mut st) = ctx.app_state.lock() {
        st.modifiers.retain(|m| m.id != id);
        st.modifiers.clone()
    } else {
        return (500, "application/json", b"{\"error\":\"lock_failed\"}".to_vec());
    };
    compile_and_push_modifiers(&snapshot, ctx);
    json_response(200, &serde_json::json!({ "ok": true, "modifiers": snapshot }))
}

fn handle_get_footswitch(ctx: &ApiContext) -> (u16, &'static str, Vec<u8>) {
    let mode_str = ctx
        .app_state
        .lock()
        .map(|s| s.footswitch_mode.clone())
        .unwrap_or_else(|_| "HYBRID_STOMP_SCENE".to_string());
    let mode = match mode_str.to_ascii_uppercase().as_str() {
        "PRESET" => sonix_core::footswitch::FootswitchMode::Preset,
        "SCENE" | "SCENES" => sonix_core::footswitch::FootswitchMode::Scene,
        "STOMP" => sonix_core::footswitch::FootswitchMode::Stomp,
        _ => sonix_core::footswitch::FootswitchMode::HybridStompScene,
    };
    let mut arr = sonix_core::footswitch::FootswitchArray::new();
    arr.set_mode(mode);
    let units: Vec<serde_json::Value> = (0..8usize)
        .map(|i| {
            let u = arr.unit(i);
            serde_json::json!({
                "id": u.id,
                "oled_top_line": u.oled_top_line,
                "oled_main_text": u.oled_main_text,
                "color_hex": u.color_hex,
            })
        })
        .collect();
    json_response(200, &serde_json::json!({ "mode": mode_str, "units": units }))
}

fn handle_post_footswitch_press(body: &[u8], ctx: &ApiContext) -> (u16, &'static str, Vec<u8>) {
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(body) else {
        return (400, "application/json", b"{\"error\":\"invalid_json\"}".to_vec());
    };
    let Some(index) = v.get("index").and_then(|x| x.as_u64()) else {
        return (400, "application/json", b"{\"error\":\"missing_index\"}".to_vec());
    };
    if !(1..=8).contains(&index) {
        return (400, "application/json", b"{\"error\":\"footswitch_index_out_of_range\"}".to_vec());
    }
    let mode_str = ctx
        .app_state
        .lock()
        .map(|s| s.footswitch_mode.clone())
        .unwrap_or_else(|_| "HYBRID_STOMP_SCENE".to_string());
    let mode_up = mode_str.to_ascii_uppercase();
    let stomp_keys = [
        "gate.enabled",
        "drive.enabled",
        "amp_1.enabled",
        "cab_1.enabled",
        "eq_1.enabled",
        "comp_2.enabled",
        "dly_1.enabled",
        "rev_1.enabled",
    ];
    let scene_letters = ["A", "B", "C", "D", "E", "F", "G", "H"];

    match mode_up.as_str() {
        "PRESET" => {
            if index % 2 == 1 {
                handle_post_setlist_step(-1, ctx)
            } else {
                handle_post_setlist_step(1, ctx)
            }
        }
        "SCENE" | "SCENES" => {
            let letter = scene_letters[(index as usize) - 1];
            let Ok(req) = serde_json::to_vec(&serde_json::json!({ "scene": letter })) else {
                return json_error_response(500, "E_INTERNAL: serialization failed");
            };
            handle_post_scenes_select(&req, ctx)
        }
        "STOMP" => {
            let key = stomp_keys[(index as usize) - 1];
            let cur = ctx
                .app_state
                .lock()
                .ok()
                .and_then(|st| serde_json::to_value(&*st).ok())
                .and_then(|val| val.get(key).and_then(|b| b.as_bool()))
                .unwrap_or(false);
            let Ok(patch) = serde_json::to_vec(&serde_json::json!({ key: !cur })) else {
                return json_error_response(500, "E_INTERNAL: serialization failed");
            };
            dispatch(ctx, "/api/params", "POST", &patch)
        }
        _ => {
            // HYBRID_STOMP_SCENE: FS1..4 = Stomps, FS5..8 = Scenes A..D
            if index <= 4 {
                let key = stomp_keys[(index as usize) - 1];
                let cur = ctx
                    .app_state
                    .lock()
                    .ok()
                    .and_then(|st| serde_json::to_value(&*st).ok())
                    .and_then(|val| val.get(key).and_then(|b| b.as_bool()))
                    .unwrap_or(false);
                let Ok(patch) = serde_json::to_vec(&serde_json::json!({ key: !cur })) else {
                    return json_error_response(500, "E_INTERNAL: serialization failed");
                };
                dispatch(ctx, "/api/params", "POST", &patch)
            } else {
                let letter = scene_letters[(index as usize) - 5];
                let Ok(req) = serde_json::to_vec(&serde_json::json!({ "scene": letter })) else {
                    return json_error_response(500, "E_INTERNAL: serialization failed");
                };
                handle_post_scenes_select(&req, ctx)
            }
        }
    }
}
