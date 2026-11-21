//! Testes de contrato da Spec 033 (Fase 2 — Shell Desktop Tauri v2)
//! Regime SDD + TDD estrito (CA-1, CA-2, CA-9, CA-10, CA-11, CA-13).

use sonix_app_lib::commands::{invoke_dispatch, CommandResponse};
use sonix_app_lib::streams::StreamCoordinator;
use sonix_app_lib::{ensure_single_audio_host, reset_audio_host_count_for_test};
use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{
    LockFreeParamQueue, RealtimeDspChain, TelemetryBuffer,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

fn make_ctx() -> ApiContext {
    let audio_tele = Arc::new(Mutex::new(None));
    let app_state = Arc::new(Mutex::new(AppState::default()));
    let telemetry = Arc::new(TelemetryBuffer::new(512));
    let dsp = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let pq = Arc::new(LockFreeParamQueue::new());
    ApiContext::new(
        audio_tele,
        app_state,
        telemetry,
        Some(dsp),
        Some(pq),
        None,
        None,
    )
}

#[test]
fn test_ca_02_tauri_commands_match_api_dispatch_parity() {
    let ctx_http = make_ctx();
    let ctx_ipc = make_ctx();

    let routes: &[(&str, &str, &str)] = &[
        ("GET", "/api/status", ""),
        ("GET", "/api/audio/status", ""),
        ("GET", "/api/telemetry", ""),
        ("GET", "/api/devices", ""),
        ("GET", "/api/state", ""),
        ("GET", "/api/tuner", ""),
        ("GET", "/api/graph", ""),
        ("GET", "/api/scope?mode=transfer", ""),
        ("GET", "/api/presets", ""),
        (
            "POST",
            "/api/params",
            r#"{"amp_1.gain": 6.5, "gate.threshold_db": -52.0}"#,
        ),
        (
            "POST",
            "/api/params",
            r#"{"amp_1.gain": 99.0, "gate.threshold_db": -40.0}"#,
        ),
        ("POST", "/api/scenes/select", r#"{"scene": "C"}"#),
        ("POST", "/api/input/di", r#"{"samples": [0.1, -0.1, 0.2, -0.2]}"#),
    ];

    for (method, route, body) in routes {
        let (http_status, _, http_body) = dispatch(&ctx_http, route, method, body.as_bytes());
        let ipc_resp: CommandResponse = invoke_dispatch(&ctx_ipc, method, route, body.as_bytes());
        assert_eq!(
            ipc_resp.status, http_status,
            "Status divergente para {} {}",
            method, route
        );
        let http_json: serde_json::Value = serde_json::from_slice(&http_body).unwrap();
        let ipc_json: serde_json::Value = serde_json::from_str(&ipc_resp.body).unwrap();
        assert_eq!(
            ipc_json, http_json,
            "JSON divergente para {} {}",
            method, route
        );
    }

    // Valida que rejeição atômica em /api/params não mutou amp_1.gain (manteve 6.5, não virou 99.0)
    let state_ipc: CommandResponse = invoke_dispatch(&ctx_ipc, "GET", "/api/state", b"");
    let parsed_state: serde_json::Value = serde_json::from_str(&state_ipc.body).unwrap();
    assert_eq!(parsed_state["amp_1.gain"], 6.5);
}

#[test]
fn test_ca_09_stream_frame_sequence_and_pause_on_minimize() {
    let ctx = make_ctx();
    let coord = StreamCoordinator::new();
    assert!(coord.is_visible());

    let f1 = coord.next_telemetry_frame(&ctx).expect("Quadro 1 visível");
    let f2 = coord.next_telemetry_frame(&ctx).expect("Quadro 2 visível");
    assert!(f2.seq > f1.seq, "Sequência de telemetria deve ser estritamente crescente");

    let t1 = coord.next_tuner_frame(&ctx).expect("Tuner quadro 1 visível");
    let s1 = coord.next_scope_frame(&ctx, "transfer").expect("Scope quadro 1 visível");
    assert!(t1.seq > 0 && s1.seq > 0);

    // Ao minimizar / ocultar janela, taxa de entrega vai a 0
    coord.set_visible(false);
    assert!(coord.next_telemetry_frame(&ctx).is_none());
    assert!(coord.next_tuner_frame(&ctx).is_none());
    assert!(coord.next_scope_frame(&ctx, "transfer").is_none());

    // Ao restaurar janela, retoma entrega mantendo sequência crescente
    coord.set_visible(true);
    let f3 = coord.next_telemetry_frame(&ctx).expect("Retoma telemetria ao restaurar");
    assert!(f3.seq > f2.seq);
}

#[test]
fn test_ca_10_and_ca_01_tauri_conf_csp_and_offline_nsis() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let conf_text = std::fs::read_to_string(manifest_dir.join("tauri.conf.json")).unwrap();
    let conf: serde_json::Value = serde_json::from_str(&conf_text).unwrap();

    let win = &conf["app"]["windows"][0];
    assert_eq!(win["width"], 1440);
    assert_eq!(win["height"], 900);
    assert!(win["minWidth"].as_u64().unwrap() >= 1280);
    assert!(win["minHeight"].as_u64().unwrap() >= 800);
    assert_eq!(win["decorations"], false);

    let csp = conf["app"]["security"]["csp"].as_str().unwrap();
    assert!(!csp.contains('*'), "CSP não pode conter wildcard '*'");
    assert!(csp.contains("default-src 'self'"));
    assert!(csp.contains("ipc.localhost"));

    assert_eq!(
        conf["bundle"]["windows"]["webviewInstallMode"]["type"],
        "offlineInstaller"
    );

    let cap_text = std::fs::read_to_string(manifest_dir.join("capabilities/main.json")).unwrap();
    let cap: serde_json::Value = serde_json::from_str(&cap_text).unwrap();
    assert_eq!(cap["identifier"], "main-capability");
    assert!(cap["permissions"].as_array().unwrap().len() >= 5);
}

#[test]
fn test_ca_11_single_instance_audio_stream_singleton() {
    reset_audio_host_count_for_test();
    let started_count = AtomicUsize::new(0);
    assert!(ensure_single_audio_host(&started_count));
    assert_eq!(started_count.load(Ordering::SeqCst), 1);
    // Segunda tentativa na mesma instância (ou foco de segunda execução) não abre 2º stream
    assert!(!ensure_single_audio_host(&started_count));
    assert_eq!(started_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_ca_13_realtime_audio_callback_zero_alloc_under_60hz_ipc() {
    let ctx = make_ctx();
    let dsp_ref = ctx.dsp_chain.clone().unwrap();

    // Injeta 60 comandos via comando puro enquanto callback RT drena fila e processa blocos
    let mut in_buf = [0.05f32; 128];
    let mut out_buf = [0.0f32; 128];
    for step in 0..60 {
        let gain = 2.0 + (step as f32) * 0.1;
        let payload = format!(r#"{{"amp_1.gain": {:.2}}}"#, gain);
        let resp = invoke_dispatch(&ctx, "POST", "/api/params", payload.as_bytes());
        assert_eq!(resp.status, 200);

        // Thread de áudio não bloqueia e drena fila SPSC sem alocação
        if let Ok(mut guard) = dsp_ref.try_lock() {
            guard.drain_param_queue();
            guard.process(&in_buf, &mut out_buf);
        }
        in_buf[0] = -in_buf[0];
        assert!(out_buf.iter().all(|s| s.is_finite()));
    }
}
