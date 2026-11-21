//! crates/sonix-desktop/tests/contract_035_ui_backend_contracts.rs
//! Testes de contrato da Spec 035: Endpoints /api/presets, /api/presets/load, /api/scope, /api/devices
//! e geração/verificação das fixtures de contrato em tests/fixtures/ipc-schemas/*.json.

use serde_json::{json, Value};
use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{
    AudioBackendType, AudioDeviceEnumerator, AudioStatusTelemetry, DesktopAudioDevice,
    RealtimeDspChain, TelemetryBuffer,
};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

struct FixtureAudioEnumerator;

impl AudioDeviceEnumerator for FixtureAudioEnumerator {
    fn enumerate_devices(&self) -> Vec<DesktopAudioDevice> {
        vec![DesktopAudioDevice {
            id: "fixture_in_1".to_string(),
            name: "Focusrite USB".to_string(),
            backend: AudioBackendType::Wasapi,
            is_input: true,
            supported_sample_rates: vec![44100, 48000],
            supported_buffer_sizes: vec![64, 128, 256],
        }]
    }
}

fn build_ctx() -> ApiContext {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 256);
    let pq = chain.param_queue().clone();
    let mut ctx = ApiContext::new(
        Arc::new(Mutex::new(Some(AudioStatusTelemetry {
            backend: "wasapi".to_string(),
            mode: "wasapi_shared".to_string(),
            fallback_reason: None,
            raw_processing: false,
            sample_rate: 48000,
            input_period_frames: 128,
            output_period_frames: 128,
            input_device: "Focusrite USB".to_string(),
            output_device: "Focusrite USB".to_string(),
            driver_input_latency_ms: 2.67,
            driver_output_latency_ms: 2.67,
            cpu_avg_pct: 6.5,
            cpu_peak_pct: 9.1,
            underruns: 0,
            overruns: 0,
            panics: 0,
            adc_clips: 0,
            measured_roundtrip_ms: None,
            estimated_roundtrip_ms: 5.34,
            algorithmic_latency_samples: 0,
            pitch_wet_latency_ms: 0.0,
        }))),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(256)),
        Some(Arc::new(Mutex::new(chain))),
        Some(pq),
        None,
        None,
    );
    ctx.device_enumerator = Arc::new(FixtureAudioEnumerator);
    ctx
}

#[test]
fn test_ca_04_all_scope_modes_offered_by_ui_return_200() {
    let ctx = build_ctx();
    for mode in ["waveform", "transfer", "tonestack"] {
        let route = format!("/api/scope?mode={}", mode);
        let (code, _ct, body) = dispatch(&ctx, &route, "GET", b"");
        assert_eq!(
            code, 200,
            "CA-4: GET {} deve retornar 200, obtido {}: {}",
            route, code, String::from_utf8_lossy(&body)
        );
    }

    let (bad_code, _, _) = dispatch(&ctx, "/api/scope?mode=modo_invalido", "GET", b"");
    assert_eq!(bad_code, 400);
}

#[test]
fn test_ca_05_presets_list_returns_object_with_presets_array() {
    let ctx = build_ctx();
    let (code, _ct, body) = dispatch(&ctx, "/api/presets", "GET", b"");
    assert_eq!(code, 200);
    let parsed: Value = serde_json::from_slice(&body).unwrap();
    let list = parsed["presets"]
        .as_array()
        .expect("CA-5: GET /api/presets deve retornar objeto { presets: [...] }");
    assert!(!list.is_empty(), "CA-5: Deve haver presets no catalogo do backend");
}

#[test]
fn test_ca_06_load_preset_returns_status_loaded_and_state() {
    let ctx = build_ctx();
    let (code, _ct, body) = dispatch(&ctx, "/api/presets/load", "POST", br#"{"id":"recto"}"#);
    assert_eq!(code, 200);
    let parsed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["status"], "ok");
    assert_eq!(parsed["loaded"], "recto");
    let st = parsed["state"]
        .as_object()
        .expect("CA-6: POST /api/presets/load deve devolver campo 'state' com AppState aplicado");
    assert_eq!(
        st.get("amp_1.model").and_then(|v| v.as_str()),
        Some("dual_rect"),
        "CA-6: state deve refletir amp_1.model=dual_rect"
    );
}

#[test]
fn test_ca_07_devices_returns_object_with_devices_array() {
    let ctx = build_ctx();
    let (code, _ct, body) = dispatch(&ctx, "/api/devices", "GET", b"");
    assert_eq!(code, 200);
    let parsed: Value = serde_json::from_slice(&body).unwrap();
    let devs = parsed["devices"]
        .as_array()
        .expect("CA-7: GET /api/devices deve devolver { devices: [...] }");
    assert_eq!(devs.len(), 1);
    assert_eq!(devs[0]["name"], "Focusrite USB");
}

#[test]
fn test_ca_08_generate_and_verify_rust_ipc_schema_fixtures() {
    let ctx = build_ctx();
    ctx.telemetry.update_peaks(-18.5, -12.0, 37.5);
    ctx.telemetry.update_gate_telemetry(sonix_desktop::audio::GateState::Closed, -18.2);
    ctx.telemetry.push_waveform(&[0.0, 0.25, -0.25, 0.5]);

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let fixtures_dir = root.join("tests/fixtures/ipc-schemas");
    fs::create_dir_all(&fixtures_dir).unwrap();

    let routes = [
        ("telemetry.json", "GET", "/api/telemetry", b"".as_slice()),
        ("audio_status.json", "GET", "/api/audio/status", b"".as_slice()),
        ("devices.json", "GET", "/api/devices", b"".as_slice()),
        ("scope_transfer.json", "GET", "/api/scope?mode=transfer", b"".as_slice()),
        ("scope_waveform.json", "GET", "/api/scope?mode=waveform", b"".as_slice()),
        ("scope_tonestack.json", "GET", "/api/scope?mode=tonestack", b"".as_slice()),
        ("presets_list.json", "GET", "/api/presets", b"".as_slice()),
        ("presets_load.json", "POST", "/api/presets/load", br#"{"id":"recto"}"#.as_slice()),
    ];

    for (fname, method, route, req_body) in routes {
        let (code, _ct, body) = dispatch(&ctx, route, method, req_body);
        assert_eq!(code, 200, "CA-8: Rota {} falhou com {}", route, code);
        let val: Value = serde_json::from_slice(&body).unwrap();
        let pretty = serde_json::to_string_pretty(&val).unwrap();
        fs::write(fixtures_dir.join(fname), format!("{}\n", pretty)).unwrap();
    }

    let manifest = json!({
        "generated_by": "contract_035_ui_backend_contracts",
        "files": routes.iter().map(|(f, ..)| *f).collect::<Vec<_>>()
    });
    fs::write(
        fixtures_dir.join("manifest.json"),
        format!("{}\n", serde_json::to_string_pretty(&manifest).unwrap()),
    )
    .unwrap();
}
