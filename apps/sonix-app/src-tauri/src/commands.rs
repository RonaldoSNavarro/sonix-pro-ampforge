//! Comandos Tauri IPC tipados com paridade estrita ao despachante puro `sonix_desktop::api::dispatch`.

use crate::streams::{ScopeFrame, StreamCoordinator, TelemetryFrame, TunerFrame};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sonix_desktop::api::{dispatch, ApiContext};
use sonix_desktop::audio::{
    AudioDriverError, AudioStatusTelemetry, InputChannel, RealtimeDspChain,
    RealtimeDuplexAudioStreams, RealtimeDuplexEngine, TelemetryBuffer,
    STREAM_ERR_DEVICE_NOT_AVAILABLE, STREAM_ERR_OTHER,
};
use sonix_desktop::DesktopAudioHost;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tauri::ipc::Channel;
use tauri::State;

/// Configuração ativa de dispositivo de áudio para recuperação automática (AUD-05).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDeviceConfig {
    pub mode: String,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub sample_rate: Option<u32>,
    pub buffer_size: Option<u32>,
}

impl Default for AudioDeviceConfig {
    fn default() -> Self {
        Self {
            mode: "wasapi_shared".to_string(),
            input_device: None,
            output_device: None,
            sample_rate: None,
            buffer_size: None,
        }
    }
}

/// Supervisor assíncrono que observa erros de stream CPAL a >= 10 Hz e reabre automaticamente (AUD-05).
pub struct AudioDeviceSupervisor {
    running: Arc<AtomicBool>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl AudioDeviceSupervisor {
    pub fn start_with_opener<T, F, E>(
        api_ctx: ApiContext,
        duplex: Arc<RealtimeDuplexEngine>,
        dsp_chain: Arc<Mutex<RealtimeDspChain>>,
        active_streams: Arc<Mutex<Option<T>>>,
        config: Arc<Mutex<AudioDeviceConfig>>,
        mut opener: F,
        mut on_state_change: E,
    ) -> Self
    where
        T: Send + 'static,
        F: FnMut(
                Arc<RealtimeDuplexEngine>,
                Arc<Mutex<RealtimeDspChain>>,
                Arc<TelemetryBuffer>,
                &AudioDeviceConfig,
            ) -> Result<(Option<T>, AudioStatusTelemetry), AudioDriverError>
            + Send
            + 'static,
        E: FnMut(&str) + Send + 'static,
    {
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        let handle = thread::spawn(move || {
            let mut reopen_pending = false;
            let mut last_attempt = Instant::now();

            while r.load(Ordering::Relaxed) {
                let err_code = api_ctx.telemetry.take_stream_error_code();
                if err_code == STREAM_ERR_DEVICE_NOT_AVAILABLE {
                    if let Ok(mut guard) = active_streams.lock() {
                        *guard = None;
                    }
                    if !api_ctx.telemetry.is_device_lost() {
                        api_ctx.telemetry.set_device_lost(true);
                        on_state_change("lost");
                    }
                    reopen_pending = true;
                    last_attempt = Instant::now();
                } else if err_code == STREAM_ERR_OTHER {
                    eprintln!("SONIX AUDIO WARN: erro no stream CPAL (Other); tentando reabrir stream");
                    if let Ok(mut guard) = active_streams.lock() {
                        *guard = None;
                    }
                    let cfg = config
                        .lock()
                        .map(|g| g.clone())
                        .unwrap_or_default();
                    match opener(
                        duplex.clone(),
                        dsp_chain.clone(),
                        api_ctx.telemetry.clone(),
                        &cfg,
                    ) {
                        Ok((new_stream, actual_tele)) => {
                            duplex.set_target_period(actual_tele.output_period_frames as usize);
                            if let Ok(mut guard) = active_streams.lock() {
                                *guard = new_stream;
                            }
                            if let Ok(mut guard) = api_ctx.audio_telemetry.lock() {
                                *guard = Some(actual_tele);
                            }
                            api_ctx.telemetry.set_device_lost(false);
                            on_state_change("ok");
                            reopen_pending = false;
                        }
                        Err(e) => {
                            eprintln!("SONIX AUDIO WARN: reabertura imediata falhou ({}); entrando em recuperacao periodica", e);
                            if !api_ctx.telemetry.is_device_lost() {
                                api_ctx.telemetry.set_device_lost(true);
                                on_state_change("lost");
                            }
                            reopen_pending = true;
                            last_attempt = Instant::now();
                        }
                    }
                }

                if reopen_pending && last_attempt.elapsed() >= Duration::from_secs(1) {
                    last_attempt = Instant::now();
                    let cfg = config
                        .lock()
                        .map(|g| g.clone())
                        .unwrap_or_default();
                    if let Ok((new_stream, actual_tele)) = opener(
                        duplex.clone(),
                        dsp_chain.clone(),
                        api_ctx.telemetry.clone(),
                        &cfg,
                    ) {
                        duplex.set_target_period(actual_tele.output_period_frames as usize);
                        if let Ok(mut guard) = active_streams.lock() {
                            *guard = new_stream;
                        }
                        if let Ok(mut guard) = api_ctx.audio_telemetry.lock() {
                            *guard = Some(actual_tele);
                        }
                        api_ctx.telemetry.set_device_lost(false);
                        on_state_change("ok");
                        reopen_pending = false;
                    }
                }

                thread::sleep(Duration::from_millis(25));
            }
        });

        Self {
            running,
            worker: Mutex::new(Some(handle)),
        }
    }

    pub fn stop_and_join(&self) {
        self.running.store(false, Ordering::SeqCst);
        if let Ok(mut guard) = self.worker.lock() {
            if let Some(h) = guard.take() {
                let _ = h.join();
            }
        }
    }
}

impl Drop for AudioDeviceSupervisor {
    fn drop(&mut self) {
        self.stop_and_join();
    }
}

/// Controle da thread fallback quando o app inicia sem dispositivo real (Spec 049 CA-5).
pub struct FallbackWorkerControl {
    pub running: Arc<AtomicBool>,
    pub worker: Mutex<Option<JoinHandle<()>>>,
}

impl FallbackWorkerControl {
    pub fn stop_and_join(&self) {
        self.running.store(false, Ordering::SeqCst);
        if let Ok(mut guard) = self.worker.lock() {
            if let Some(handle) = guard.take() {
                handle.thread().unpark();
                let _ = handle.join();
            }
        }
    }
}

/// Handles ativos do motor de áudio gerenciados pelo runtime Tauri (Spec 034, CA-3/CA-4/CA-9).
pub struct AudioRuntimeControl {
    pub duplex: Arc<RealtimeDuplexEngine>,
    pub dsp_chain: Arc<Mutex<RealtimeDspChain>>,
    pub active_streams: Arc<Mutex<Option<RealtimeDuplexAudioStreams>>>,
    pub fallback: Arc<FallbackWorkerControl>,
    pub device_config: Arc<Mutex<AudioDeviceConfig>>,
}

/// Resposta bruta espelhando `(status, content_type, body)` do despachante puro `api::dispatch`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    pub status: u16,
    pub content_type: String,
    pub body: String,
}

/// Estado compartilhado gerenciado pelo runtime Tauri.
pub struct AppRuntime {
    pub api_ctx: ApiContext,
    pub streams: Arc<StreamCoordinator>,
    pub audio_ctrl: Option<Arc<AudioRuntimeControl>>,
    pub midi_host: Arc<Mutex<sonix_desktop::midi::DesktopMidiHost>>,
    pub midi_router: Arc<Mutex<sonix_desktop::midi::MidiRouter>>,
    pub _keep_alive: Arc<dyn std::any::Any + Send + Sync>,
}

/// Executa o despachante puro sem abrir nenhum socket TCP/HTTP.
pub fn invoke_dispatch(
    ctx: &ApiContext,
    method: &str,
    route_with_query: &str,
    body: &[u8],
) -> CommandResponse {
    let (status, content_type, bytes) = dispatch(ctx, route_with_query, method, body);
    CommandResponse {
        status,
        content_type: content_type.to_string(),
        body: String::from_utf8_lossy(&bytes).into_owned(),
    }
}

fn response_to_result(resp: CommandResponse) -> Result<Value, String> {
    let parsed: Value = serde_json::from_str(&resp.body)
        .unwrap_or_else(|_| Value::String(resp.body.clone()));
    if resp.status >= 400 {
        let msg = parsed
            .get("error")
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("IPC status {}: {}", resp.status, resp.body));
        // Spec 051 T-02: o IPC respondeu; a UI precisa distinguir rejeição de queda do IPC.
        let prefix = if resp.status < 500 { "E_VALIDATION:" } else { "E_ENGINE:" };
        Err(format!("{prefix}{msg}"))
    } else {
        Ok(parsed)
    }
}

#[tauri::command]
pub fn get_status(state: State<'_, AppRuntime>) -> Result<Value, String> {
    response_to_result(invoke_dispatch(&state.api_ctx, "GET", "/api/status", b""))
}

#[tauri::command]
pub fn get_audio_status(state: State<'_, AppRuntime>) -> Result<Value, String> {
    response_to_result(invoke_dispatch(
        &state.api_ctx,
        "GET",
        "/api/audio/status",
        b"",
    ))
}

#[tauri::command]
pub fn get_telemetry(state: State<'_, AppRuntime>) -> Result<Value, String> {
    response_to_result(invoke_dispatch(&state.api_ctx, "GET", "/api/telemetry", b""))
}

#[tauri::command]
pub fn get_devices(state: State<'_, AppRuntime>) -> Result<Value, String> {
    response_to_result(invoke_dispatch(&state.api_ctx, "GET", "/api/devices", b""))
}

#[tauri::command]
pub fn get_state(state: State<'_, AppRuntime>) -> Result<Value, String> {
    response_to_result(invoke_dispatch(&state.api_ctx, "GET", "/api/state", b""))
}

#[tauri::command]
pub fn get_tuner(state: State<'_, AppRuntime>) -> Result<Value, String> {
    response_to_result(invoke_dispatch(&state.api_ctx, "GET", "/api/tuner", b""))
}

#[tauri::command]
pub fn get_graph(state: State<'_, AppRuntime>) -> Result<Value, String> {
    response_to_result(invoke_dispatch(&state.api_ctx, "GET", "/api/graph", b""))
}

#[tauri::command]
pub fn get_scope(mode: Option<String>, state: State<'_, AppRuntime>) -> Result<Value, String> {
    let m = mode.as_deref().unwrap_or("transfer");
    let route = format!("/api/scope?mode={}", m);
    response_to_result(invoke_dispatch(&state.api_ctx, "GET", &route, b""))
}

#[tauri::command]
pub fn set_params(params: Value, state: State<'_, AppRuntime>) -> Result<Value, String> {
    let body = serde_json::to_vec(&params).map_err(|e| e.to_string())?;
    response_to_result(invoke_dispatch(&state.api_ctx, "POST", "/api/params", &body))
}

#[tauri::command]
pub fn load_preset(payload: Value, state: State<'_, AppRuntime>) -> Result<Value, String> {
    let body = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    response_to_result(invoke_dispatch(
        &state.api_ctx,
        "POST",
        "/api/presets/load",
        &body,
    ))
}

#[tauri::command]
pub fn save_preset(payload: Value, state: State<'_, AppRuntime>) -> Result<Value, String> {
    let body = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    response_to_result(invoke_dispatch(
        &state.api_ctx,
        "POST",
        "/api/presets/save",
        &body,
    ))
}

#[tauri::command]
pub fn import_preset(payload: Value, state: State<'_, AppRuntime>) -> Result<Value, String> {
    let body = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    response_to_result(invoke_dispatch(
        &state.api_ctx,
        "POST",
        "/api/presets/import",
        &body,
    ))
}

#[tauri::command]
pub fn export_preset(id: String, state: State<'_, AppRuntime>) -> Result<Value, String> {
    let route = format!("/api/presets/{}/export", id);
    response_to_result(invoke_dispatch(&state.api_ctx, "GET", &route, b""))
}

#[tauri::command]
pub fn list_presets(state: State<'_, AppRuntime>) -> Result<Value, String> {
    response_to_result(invoke_dispatch(&state.api_ctx, "GET", "/api/presets", b""))
}

#[tauri::command]
pub fn select_scene(payload: Value, state: State<'_, AppRuntime>) -> Result<Value, String> {
    let body = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    response_to_result(invoke_dispatch(
        &state.api_ctx,
        "POST",
        "/api/scenes/select",
        &body,
    ))
}

#[tauri::command]
pub fn load_ir(payload: Value, state: State<'_, AppRuntime>) -> Result<Value, String> {
    let body = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    response_to_result(invoke_dispatch(&state.api_ctx, "POST", "/api/cab/ir", &body))
}

#[tauri::command]
pub fn set_input_di(payload: Value, state: State<'_, AppRuntime>) -> Result<Value, String> {
    let body = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    response_to_result(invoke_dispatch(
        &state.api_ctx,
        "POST",
        "/api/input/di",
        &body,
    ))
}

#[tauri::command]
pub fn calibrate_input(payload: Value, state: State<'_, AppRuntime>) -> Result<Value, String> {
    let body = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    response_to_result(invoke_dispatch(
        &state.api_ctx,
        "POST",
        "/api/input/calibrate",
        &body,
    ))
}

pub fn run_reamp_ipc(ctx: &ApiContext, payload: &Value) -> Result<Value, String> {
    let body = serde_json::to_vec(payload).map_err(|e| e.to_string())?;
    response_to_result(invoke_dispatch(ctx, "POST", "/api/reamp/run", &body))
}

#[tauri::command]
pub fn run_reamp(payload: Value, state: State<'_, AppRuntime>) -> Result<Value, String> {
    run_reamp_ipc(&state.api_ctx, &payload)
}

#[tauri::command]
pub fn measure_latency(state: State<'_, AppRuntime>) -> Result<Value, String> {
    let status = get_audio_status(state)?;
    let in_ms = status["driver_input_latency_ms"].as_f64().unwrap_or(2.5);
    let out_ms = status["driver_output_latency_ms"].as_f64().unwrap_or(2.5);
    let sr = status["sample_rate"].as_f64().unwrap_or(48000.0).max(1.0);
    let algo_samples = status["algorithmic_latency_samples"].as_f64().unwrap_or(16.0);
    let algo_ms = (algo_samples / sr) * 1000.0;
    let estimated_ms = status["estimated_roundtrip_ms"]
        .as_f64()
        .unwrap_or(in_ms + out_ms + algo_ms);
    let measured_ms = status["measured_roundtrip_ms"].as_f64();
    Ok(json!({
        "ok": true,
        "driver_input_latency_ms": in_ms,
        "driver_output_latency_ms": out_ms,
        "algorithmic_latency_ms": (algo_ms * 100.0).round() / 100.0,
        "estimated_roundtrip_ms": (estimated_ms * 100.0).round() / 100.0,
        "measured_roundtrip_ms": measured_ms.map(|v| (v * 100.0).round() / 100.0),
        "sample_rate": sr as u32
    }))
}

/// Configura dispositivos/taxa/buffer de áudio reabrindo o stream real e preservando o anterior se falhar (Spec 034, CA-3, CA-4, CA-9).
pub fn configure_audio_device_with_opener<F>(
    payload: &Value,
    api_ctx: &ApiContext,
    duplex: &Arc<RealtimeDuplexEngine>,
    dsp_chain: &Arc<Mutex<RealtimeDspChain>>,
    active_streams: &Arc<Mutex<Option<RealtimeDuplexAudioStreams>>>,
    opener: F,
) -> Result<Value, String>
where
    F: FnMut(
        Arc<RealtimeDuplexEngine>,
        Arc<Mutex<RealtimeDspChain>>,
        Arc<TelemetryBuffer>,
        Option<&str>,
        Option<&str>,
        Option<u32>,
        Option<u32>,
    ) -> Result<(Option<RealtimeDuplexAudioStreams>, AudioStatusTelemetry), AudioDriverError>,
{
    configure_audio_device_generic(payload, api_ctx, duplex, dsp_chain, active_streams, None, opener)
}

pub fn configure_audio_device_generic<T, F>(
    payload: &Value,
    api_ctx: &ApiContext,
    duplex: &Arc<RealtimeDuplexEngine>,
    dsp_chain: &Arc<Mutex<RealtimeDspChain>>,
    active_streams: &Arc<Mutex<Option<T>>>,
    fallback: Option<&FallbackWorkerControl>,
    mut opener: F,
) -> Result<Value, String>
where
    F: FnMut(
        Arc<RealtimeDuplexEngine>,
        Arc<Mutex<RealtimeDspChain>>,
        Arc<TelemetryBuffer>,
        Option<&str>,
        Option<&str>,
        Option<u32>,
        Option<u32>,
    ) -> Result<(Option<T>, AudioStatusTelemetry), AudioDriverError>,
{
    let host = DesktopAudioHost::new();
    let known_devices = host.enumerate_devices();
    let backend = payload
        .get("backend")
        .and_then(|v| v.as_str())
        .unwrap_or("wasapi");
    let requested_mode = payload
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or(backend);
    host.validate_requested_mode(requested_mode)?;
    let sample_rate = payload
        .get("sample_rate")
        .and_then(|v| v.as_u64())
        .unwrap_or(48000) as u32;
    if !matches!(sample_rate, 44100 | 48000 | 88200 | 96000) {
        return Err(format!("E_VALIDATION:Taxa de amostragem não suportada: {} Hz", sample_rate));
    }
    let buffer_size = payload
        .get("buffer_size")
        .or_else(|| payload.get("buffer"))
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    if let Some(bs) = buffer_size {
        if !matches!(bs, 32 | 64 | 128 | 256 | 441 | 448 | 480 | 512 | 882 | 960 | 1024 | 2048)
            && !(32..=2048).contains(&bs)
        {
            return Err(format!("E_VALIDATION:Buffer size inválido: {} samples", bs));
        }
    }
    let input_device = payload
        .get("input_device")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let output_device = payload
        .get("output_device")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let input_channel = payload
        .get("input_channel")
        .and_then(|v| v.as_str())
        .unwrap_or("left")
        .to_string();

    let prev_telemetry = api_ctx
        .audio_telemetry
        .lock()
        .ok()
        .and_then(|g| g.clone());

    // CA-5: Para e faz join da thread fallback antes de iniciar o stream real.
    if let Some(fb) = fallback {
        fb.stop_and_join();
    }

    // CA-4: Encerra o stream antigo ANTES de tentar abrir o novo (evita conflito de modo exclusivo / SPSC).
    let had_prev_stream = if let Ok(mut stream_guard) = active_streams.lock() {
        let old = stream_guard.take();
        let existed = old.is_some();
        drop(old);
        existed
    } else {
        false
    };

    duplex.set_input_channel(InputChannel::from_str_opt(&input_channel));
    duplex.set_target_period(buffer_size.unwrap_or(128) as usize);
    // AUD-06: prepara a cadeia com o stream antigo já fechado e ANTES do novo callback rodar.
    if let Ok(mut d) = dsp_chain.lock() {
        d.prepare(
            sample_rate as f32,
            (buffer_size.unwrap_or(2048) as usize).max(2048),
        );
    }

    let open_res = opener(
        duplex.clone(),
        dsp_chain.clone(),
        api_ctx.telemetry.clone(),
        input_device.as_deref(),
        output_device.as_deref(),
        Some(sample_rate),
        buffer_size,
    );

    let (new_streams, actual_telemetry) = match open_res {
        Ok(ok) => ok,
        Err(e) => {
            // CA-4: Se o novo falhar e havia stream anterior, restaura o stream anterior com mesma configuração.
            if had_prev_stream {
                if let Some(prev) = prev_telemetry {
                    if let Ok(mut d) = dsp_chain.lock() {
                        d.prepare(
                            prev.sample_rate as f32,
                            (prev.output_period_frames as usize).max(2048),
                        );
                    }
                    if let Ok((restored_stream, restored_tele)) = opener(
                        duplex.clone(),
                        dsp_chain.clone(),
                        api_ctx.telemetry.clone(),
                        Some(prev.input_device.as_str()),
                        Some(prev.output_device.as_str()),
                        Some(prev.sample_rate),
                        Some(prev.output_period_frames),
                    ) {
                        if let Ok(mut stream_guard) = active_streams.lock() {
                            *stream_guard = restored_stream;
                        }
                        if let Ok(mut guard) = api_ctx.audio_telemetry.lock() {
                            *guard = Some(restored_tele);
                        }
                    }
                }
            }
            return Err(format!("{}", e));
        }
    };

    duplex.set_target_period(actual_telemetry.output_period_frames as usize);

    if let Ok(mut stream_guard) = active_streams.lock() {
        *stream_guard = new_streams;
    }

    if let Ok(mut guard) = api_ctx.audio_telemetry.lock() {
        let mut updated = actual_telemetry.clone();
        updated.backend = backend.to_string();
        *guard = Some(updated);
    }

    Ok(json!({
        "ok": true,
        "backend": backend,
        "sample_rate": actual_telemetry.sample_rate,
        "buffer_size": actual_telemetry.output_period_frames,
        "input_device": actual_telemetry.input_device,
        "output_device": actual_telemetry.output_device,
        "input_channel": input_channel,
        "available_devices": known_devices.len()
    }))
}

#[tauri::command]
pub fn set_audio_device(payload: Value, state: State<'_, AppRuntime>) -> Result<Value, String> {
    let Some(ref ctrl) = state.audio_ctrl else {
        return Err("Controle de áudio não inicializado".to_string());
    };
    let host = DesktopAudioHost::new();
    let backend = payload
        .get("backend")
        .and_then(|v| v.as_str())
        .unwrap_or("wasapi");
    let requested_mode_str = payload
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or(backend);
    let mode_enum = host.validate_requested_mode(requested_mode_str)?;
    let res = configure_audio_device_generic(
        &payload,
        &state.api_ctx,
        &ctrl.duplex,
        &ctrl.dsp_chain,
        &ctrl.active_streams,
        Some(ctrl.fallback.as_ref()),
        |duplex, dsp_chain, telemetry, in_dev, out_dev, sr, buf| {
            let streams = DesktopAudioHost::start_realtime_duplex_with_mode(
                duplex, dsp_chain, telemetry, mode_enum, in_dev, out_dev, sr, buf,
            )?;
            let tele = streams.telemetry.clone();
            Ok((Some(streams), tele))
        },
    );
    if let Ok(ref val) = res {
        if let Ok(mut cfg) = ctrl.device_config.lock() {
            cfg.mode = requested_mode_str.to_string();
            cfg.input_device = val.get("input_device").and_then(|v| v.as_str()).map(ToString::to_string);
            cfg.output_device = val.get("output_device").and_then(|v| v.as_str()).map(ToString::to_string);
            cfg.sample_rate = val.get("sample_rate").and_then(|v| v.as_u64()).map(|v| v as u32);
            cfg.buffer_size = val.get("buffer_size").and_then(|v| v.as_u64()).map(|v| v as u32);
        }
        state.api_ctx.telemetry.set_device_lost(false);
    }
    res
}

#[tauri::command]
pub fn set_stream_visible(
    visible: bool,
    scope_mode: Option<String>,
    state: State<'_, AppRuntime>,
) -> Result<Value, String> {
    state.streams.set_visible(visible);
    if let Some(m) = scope_mode {
        state.streams.set_scope_mode(&m);
    }
    Ok(json!({ "visible": state.streams.is_visible() }))
}

#[tauri::command]
pub fn start_streams(
    app: tauri::AppHandle,
    on_telemetry: Channel<TelemetryFrame>,
    on_tuner: Channel<TunerFrame>,
    on_scope: Channel<ScopeFrame>,
    state: State<'_, AppRuntime>,
) -> Result<Value, String> {
    state.streams.start_channels(
        state.api_ctx.clone(),
        on_telemetry,
        on_tuner,
        on_scope,
        Some(app),
    );
    Ok(json!({ "ok": true, "interval_ms": 33 }))
}

#[tauri::command]
pub fn list_midi_ports(state: State<'_, AppRuntime>) -> Result<Value, String> {
    let host = state
        .midi_host
        .lock()
        .map_err(|_| "E_ENGINE: falha ao acessar host MIDI".to_string())?;
    let ports = host.enumerate_input_ports();
    let connected = host.connected_port().map(ToString::to_string);
    Ok(json!({
        "ports": ports,
        "connected_port": connected,
    }))
}

#[tauri::command]
pub fn connect_midi_port(port_name: String, state: State<'_, AppRuntime>) -> Result<Value, String> {
    let mut host = state
        .midi_host
        .lock()
        .map_err(|_| "E_ENGINE: falha ao acessar host MIDI".to_string())?;
    host.connect_port_with_router(&port_name, Arc::clone(&state.midi_router))
        .map_err(|e| format!("E_VALIDATION: {e}"))?;
    Ok(json!({
        "ok": true,
        "connected_port": port_name,
    }))
}

#[tauri::command]
pub fn disconnect_midi_port(state: State<'_, AppRuntime>) -> Result<Value, String> {
    let mut host = state
        .midi_host
        .lock()
        .map_err(|_| "E_ENGINE: falha ao acessar host MIDI".to_string())?;
    host.disconnect();
    Ok(json!({
        "ok": true,
        "connected_port": Value::Null,
    }))
}
