//! Testes de auditoria 2026-11-20 no shell Tauri — Segurança (AUD-01)

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use sonix_app_lib::commands::{invoke_dispatch, run_reamp_ipc};
use sonix_desktop::api::{ApiContext, AppState};
use sonix_desktop::audio::{LockFreeParamQueue, RealtimeDspChain, TelemetryBuffer};

fn make_ctx() -> ApiContext {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 256);
    let dsp = Arc::new(Mutex::new(chain));
    let pq = Arc::new(LockFreeParamQueue::new());
    ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(128)),
        Some(dsp),
        Some(pq),
        None,
        None,
    )
}

#[test]
fn aud_01_ipc_reamp_recusa_session_path() {
    let mut ctx = make_ctx();
    let uniq = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp_dir = std::env::temp_dir().join(format!("sonix_tauri_aud01_{}_{}", std::process::id(), uniq));
    let target = tmp_dir.join("x").join("y.json");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    ctx.set_data_dir(tmp_dir.clone());

    let payload = serde_json::json!({
        "probe": [0.5, -0.25],
        "loopback_rec": [0.0, 0.5, -0.25],
        "reamp_rec": [0.1, 0.2, 0.3],
        "session_path": target.to_string_lossy(),
    });
    let body = serde_json::to_vec(&payload).unwrap();
    let resp = invoke_dispatch(&ctx, "POST", "/api/reamp/run", &body);
    let ipc_res = run_reamp_ipc(&ctx, &payload);
    let existed = target.exists();

    assert_eq!(
        resp.status, 400,
        "AUD-01 IPC: session_path deve ser recusado com 400, retornou {} ({})",
        resp.status, resp.body
    );
    assert!(ipc_res.is_err(), "AUD-01 IPC: run_reamp_ipc deve recusar session_path");
    assert!(
        !existed,
        "AUD-01 IPC: arquivo arbitrário {:?} não pode existir",
        target
    );

    let ok_payload = serde_json::json!({
        "probe": [0.5, -0.25],
        "loopback_rec": [0.0, 0.5, -0.25],
        "reamp_rec": [0.1, 0.2, 0.3],
        "session_name": "tauri_take_01"
    });
    let ok_res = run_reamp_ipc(&ctx, &ok_payload).expect("run_reamp_ipc validado");
    let sf = ok_res["session_file"].as_str().expect("session_file presente");
    assert!(std::path::Path::new(sf).starts_with(tmp_dir.join("reamp")));
    assert!(std::path::Path::new(sf).exists());
    let _ = std::fs::remove_dir_all(&tmp_dir);
}
