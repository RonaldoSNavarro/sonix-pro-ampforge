//! crates/sonix-desktop/src/api.rs
//! Camada pura de roteamento e despacho de comandos/telemetria da API (independente de HTTP).
//! Compartilhada entre comandos Tauri (IPC nativo) e o servidor HTTP de desenvolvimento (`dev-http`).

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use ampforge_dsp::cab::SpeakerCabModel;
use ampforge_dsp::modshop::ToneStackTopology;
use ampforge_dsp::tuner::{split_note_and_octave, TunerPitchResult, TunerProcessor};

use crate::audio::{
    AudioDeviceEnumerator, AudioMode, AudioStatusTelemetry, CleanInputRing, CpalAudioEnumerator,
    DesktopAudioHost, DspParamUpdate, LockFreeParamQueue, LockFreeResourceQueue,
    PreparedDspResource, RealtimeDspChain, RealtimeDuplexEngine, TelemetryBuffer,
};
use crate::calibration::{calibrate_input, CalibrationStore};

pub use crate::params::{
    app_state_to_dsp_updates, apply_preset_params, apply_preset_params_queued,
    default_dag_topology, default_scenes, handle_post_params, is_amp_param_valid_for_model,
    nominal_chassis_for_model, AppState, ModifierBindingDto, PresetApplyError, SetlistDto,
};
use crate::params::{
    export_preset_params, parse_preamp_tube_or_default, parse_tonestack_or_default,
    read_param_bool, state_to_json, KEY_SCENE_ACTIVE, STOMP_PARAM_KEYS,
};

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
    pub shared_pitch_wet_latency_ms_bits: Option<Arc<AtomicU32>>,
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

#[inline]
fn sync_telemetry_output_muted(telemetry: &TelemetryBuffer, st: &AppState) {
    let muted = st.amp_standby || (st.tuner_enabled && st.tuner_output == "mute");
    telemetry.set_output_muted(muted);
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
        let mut resolved_param_queue = param_queue;
        let mut resource_queue = None;
        let mut shared_adc_clips = None;
        let mut shared_sample_rate_bits = None;
        let mut shared_algo_latency_samples = None;
        let mut shared_pitch_wet_latency_ms_bits = None;
        let mut shared_b_plus_v_bits = None;
        let mut clean_input_ring = None;
        let mut scope_output_ring = None;

        if let Some(ref d) = dsp_chain {
            if let Ok(mut chain) = d.lock() {
                if let Some(ref pq) = resolved_param_queue {
                    chain.set_param_queue(pq.clone());
                } else {
                    resolved_param_queue = Some(chain.param_queue());
                }
                telemetry.set_output_muted(chain.is_output_muted());
                resource_queue = Some(chain.resource_queue());
                shared_adc_clips = Some(chain.adc_clips_handle());
                shared_sample_rate_bits = Some(chain.shared_sample_rate_bits());
                shared_algo_latency_samples = Some(chain.shared_algo_latency_samples());
                shared_pitch_wet_latency_ms_bits = Some(chain.shared_pitch_wet_latency_ms_bits());
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
            param_queue: resolved_param_queue,
            resource_queue,
            shared_adc_clips,
            shared_sample_rate_bits,
            shared_algo_latency_samples,
            shared_pitch_wet_latency_ms_bits,
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
            let pitch_wet_ms = ctx
                .shared_pitch_wet_latency_ms_bits
                .as_ref()
                .map(|p| f32::from_bits(p.load(Ordering::Relaxed)))
                .unwrap_or(0.0);
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
                    t.pitch_wet_latency_ms = pitch_wet_ms;
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
                        "pitch_wet_latency_ms": (pitch_wet_ms * 1000.0).round() / 1000.0,
                    })
                }
            } else {
                serde_json::json!({
                    "adc_clips": clips,
                    "underruns": live_underruns.unwrap_or(0),
                    "overruns": live_overruns.unwrap_or(0),
                    "algorithmic_latency_samples": algo_lat,
                    "pitch_wet_latency_ms": (pitch_wet_ms * 1000.0).round() / 1000.0,
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
                    "pitch_wet_latency_ms".to_string(),
                    serde_json::json!((pitch_wet_ms * 1000.0).round() / 1000.0),
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
                    Ok(st) => json_response(200, &state_to_json(&st)),
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
                        let tube = parse_preamp_tube_or_default(&preamp_tube_str);
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
                        let parsed_ts = parse_tonestack_or_default(&ts_str);
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
                    if let Ok(st) = ctx.app_state.lock() {
                        sync_telemetry_output_muted(&ctx.telemetry, &st);
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
    ("ustwin", "US Twin Clean", include_str!("../../../presets/factory/usdeluxe.sonixpreset")),
    ("recto", "Recto High Gain", include_str!("../../../presets/factory/recto.sonixpreset")),
    ("ac30", "AC30 Top Boost", include_str!("../../../presets/factory/ac30.sonixpreset")),
];

fn canonical_factory_preset_id(id: &str) -> &str {
    if id == "usdeluxe" {
        "ustwin"
    } else {
        id
    }
}

fn embedded_factory_preset_content(id: &str) -> Option<&'static str> {
    let clean_id = canonical_factory_preset_id(id.trim_end_matches(".sonixpreset"));
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
    if canonical_factory_preset_id(clean_id) == "ustwin" {
        for dir in ["presets/factory", "../../presets/factory", "../presets/factory"] {
            let p = std::path::PathBuf::from(format!("{}/usdeluxe.sonixpreset", dir));
            if p.exists() {
                return Some(p);
            }
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
        if id == "ustwin" {
            list.push(serde_json::json!({
                "id": id,
                "name": name,
                "aliases": ["usdeluxe"],
                "type": "factory"
            }));
        } else {
            list.push(serde_json::json!({
                "id": id,
                "name": name,
                "type": "factory"
            }));
        }
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

    let ignored_keys = if let Some(ref params) = doc.params {
        match apply_preset_params_queued(params, &ctx.app_state, &ctx.param_queue, &ctx.dsp_chain) {
            Ok(ignored) => ignored,
            Err(e) => {
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
    } else {
        Vec::new()
    };

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
            .map_or(false, |p| p.contains_key(KEY_SCENE_ACTIVE))
        {
            let letters = ["A", "B", "C", "D", "E", "F", "G", "H"];
            if let Some(&l) = letters.get(doc.active_scene_index) {
                st.scene_active = l.to_string();
            }
        }
        sync_telemetry_output_muted(&ctx.telemetry, &st);
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
            "ignored_keys": ignored_keys,
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

    let params_map = export_preset_params(&current_state);

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

    let ignored_keys = if let Some(ref params) = doc.params {
        match apply_preset_params_queued(params, &ctx.app_state, &ctx.param_queue, &ctx.dsp_chain) {
            Ok(ignored) => ignored,
            Err(e) => {
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
    } else {
        Vec::new()
    };

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
            .map_or(false, |p| p.contains_key(KEY_SCENE_ACTIVE))
        {
            let letters = ["A", "B", "C", "D", "E", "F", "G", "H"];
            if let Some(&l) = letters.get(doc.active_scene_index) {
                st.scene_active = l.to_string();
            }
        }
        sync_telemetry_output_muted(&ctx.telemetry, &st);
    }

    json_response(
        200,
        &serde_json::json!({
            "status": "ok",
            "imported": doc.metadata.id,
            "ignored_keys": ignored_keys
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
        KEY_SCENE_ACTIVE.to_string(),
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
        Ok(_) => {}
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
        .and_then(|st| {
            sync_telemetry_output_muted(&ctx.telemetry, &st);
            serde_json::to_value(&*st).ok()
        })
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
                sync_telemetry_output_muted(&ctx.telemetry, &st);
            }
            if let Some(ref pq) = ctx.param_queue {
                let _ = pq.push(DspParamUpdate::InputBoost(result.proposed_boost));
                let _ = pq.push(DspParamUpdate::GateThreshold(result.proposed_gate_threshold_db));
                let _ = pq.push(DspParamUpdate::AmpStandby(false));
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
            let key = STOMP_PARAM_KEYS[(index as usize) - 1];
            let cur = ctx
                .app_state
                .lock()
                .ok()
                .map(|st| read_param_bool(&st, key))
                .unwrap_or(false);
            let Ok(patch) = serde_json::to_vec(&serde_json::json!({ key: !cur })) else {
                return json_error_response(500, "E_INTERNAL: serialization failed");
            };
            dispatch(ctx, "/api/params", "POST", &patch)
        }
        _ => {
            // HYBRID_STOMP_SCENE: FS1..4 = Stomps, FS5..8 = Scenes A..D
            if index <= 4 {
                let key = STOMP_PARAM_KEYS[(index as usize) - 1];
                let cur = ctx
                    .app_state
                    .lock()
                    .ok()
                    .map(|st| read_param_bool(&st, key))
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

/// Roteia um evento MIDI CC pelo mesmo caminho validado de `/api/params` e fila lock-free do DSP.
pub fn dispatch_midi_cc(ctx: &ApiContext, controller: u8, value: u8) -> Result<(), String> {
    let norm = (value.min(127) as f32) / 127.0;
    let upd = crate::audio::DspParamUpdate::MidiCcValue(controller, norm);
    if let Some(ref q) = ctx.param_queue {
        let _ = q.push(upd);
    } else if let Some(ref dsp) = ctx.dsp_chain {
        if let Ok(mut chain) = dsp.lock() {
            chain.set_modifier_source_value(
                crate::audio::CompiledModifierSource::MidiCc(controller),
                norm,
            );
        }
    }

    if let Some((key, val)) = crate::midi::DesktopMidiHost::map_cc_to_param(controller, value) {
        let patch = serde_json::json!({ key: val });
        let body = serde_json::to_vec(&patch).map_err(|e| e.to_string())?;
        let (code, _, resp) = dispatch(ctx, "/api/params", "POST", &body);
        if code >= 400 {
            return Err(format!(
                "Falha ao rotear MIDI CC {} via Spec 014 ({}): {}",
                controller,
                code,
                String::from_utf8_lossy(&resp)
            ));
        }
    }
    Ok(())
}
