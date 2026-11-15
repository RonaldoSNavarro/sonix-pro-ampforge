//! Comandos Tauri IPC tipados com paridade estrita ao despachante puro `sonix_desktop::api::dispatch`.

use crate::streams::{ScopeFrame, StreamCoordinator, TelemetryFrame, TunerFrame};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sonix_desktop::api::{dispatch, ApiContext};
use sonix_desktop::audio::{
    AudioDriverError, AudioStatusTelemetry, InputChannel, RealtimeDspChain,
    RealtimeDuplexAudioStreams, RealtimeDuplexEngine, TelemetryBuffer,
};
use sonix_desktop::DesktopAudioHost;
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri::State;

/// Handles ativos do motor de áudio gerenciados pelo runtime Tauri (Spec 034, CA-3/CA-4/CA-9).
pub struct AudioRuntimeControl {
    pub duplex: Arc<RealtimeDuplexEngine>,
    pub dsp_chain: Arc<Mutex<RealtimeDspChain>>,
    pub active_streams: Arc<Mutex<Option<RealtimeDuplexAudioStreams>>>,
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
        Err(msg)
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
    F: FnOnce(
        Arc<RealtimeDuplexEngine>,
        Arc<Mutex<RealtimeDspChain>>,
        Arc<TelemetryBuffer>,
        Option<&str>,
        Option<&str>,
        Option<u32>,
        Option<u32>,
    ) -> Result<(Option<RealtimeDuplexAudioStreams>, AudioStatusTelemetry), AudioDriverError>,
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
        return Err(format!("Taxa de amostragem não suportada: {} Hz", sample_rate));
    }
    let buffer_size = payload
        .get("buffer_size")
        .or_else(|| payload.get("buffer"))
        .and_then(|v| v.as_u64())
        .unwrap_or(128) as u32;
    if !matches!(buffer_size, 32 | 64 | 128 | 256 | 512 | 1024) {
        return Err(format!("Buffer size inválido: {} samples", buffer_size));
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
    duplex.set_input_channel(InputChannel::from_str_opt(&input_channel));

    // Tenta abrir o novo stream antes de desligar/sobrescrever o anterior (atomicidade com rollback em erro)
    let (new_streams, actual_telemetry) = opener(
        duplex.clone(),
        dsp_chain.clone(),
        api_ctx.telemetry.clone(),
        input_device.as_deref(),
        output_device.as_deref(),
        Some(sample_rate),
        Some(buffer_size),
    )
    .map_err(|e| format!("{}", e))?;

    duplex.set_target_period(actual_telemetry.output_period_frames as usize);
    if let Ok(mut d) = dsp_chain.lock() {
        d.prepare(
            actual_telemetry.sample_rate as f32,
            (actual_telemetry.output_period_frames as usize).max(2048),
        );
    }

    if let Ok(mut stream_guard) = active_streams.lock() {
        // Substitui stream antigo após abertura bem-sucedida do novo
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
    configure_audio_device_with_opener(
        &payload,
        &state.api_ctx,
        &ctrl.duplex,
        &ctrl.dsp_chain,
        &ctrl.active_streams,
        |duplex, dsp_chain, telemetry, in_dev, out_dev, sr, buf| {
            let streams = DesktopAudioHost::start_realtime_wasapi_duplex_with_config(
                duplex, dsp_chain, telemetry, in_dev, out_dev, sr, buf,
            )?;
            let tele = streams.telemetry.clone();
            Ok((Some(streams), tele))
        },
    )
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
    on_telemetry: Channel<TelemetryFrame>,
    on_tuner: Channel<TunerFrame>,
    on_scope: Channel<ScopeFrame>,
    state: State<'_, AppRuntime>,
) -> Result<Value, String> {
    state
        .streams
        .start_channels(state.api_ctx.clone(), on_telemetry, on_tuner, on_scope);
    Ok(json!({ "ok": true, "interval_ms": 33 }))
}
