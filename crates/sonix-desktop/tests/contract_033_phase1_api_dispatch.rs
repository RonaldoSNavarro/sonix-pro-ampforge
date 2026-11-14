//! crates/sonix-desktop/tests/contract_033_phase1_api_dispatch.rs
//! Verifica Fase 1 da Spec 033: `api::dispatch` funciona de forma pura, sem HTTP (`dev-http` desligado).

use std::sync::{Arc, Mutex};

use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{RealtimeDspChain, TelemetryBuffer};

fn test_ctx() -> ApiContext {
    let mut dsp = RealtimeDspChain::new(48000.0);
    dsp.set_amp_gain(8.0);
    let param_queue = dsp.param_queue();
    ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(256)),
        Some(Arc::new(Mutex::new(dsp))),
        Some(param_queue),
        None,
        None,
    )
}

#[test]
fn test_spec033_phase1_pure_dispatch_status_state_and_params_without_http() {
    let ctx = test_ctx();

    let (st_status, ct_status, body_status) = dispatch(&ctx, "/api/status", "GET", &[]);
    assert_eq!(st_status, 200);
    assert_eq!(ct_status, "application/json");
    let status_json: serde_json::Value = serde_json::from_slice(&body_status).unwrap();
    assert_eq!(status_json["status"], "online");

    // Mutação válida via api::dispatch
    let (st_post, _, body_post) =
        dispatch(&ctx, "/api/params", "POST", br#"{"amp_1.gain": 6.2}"#);
    assert_eq!(st_post, 200);
    let post_json: serde_json::Value = serde_json::from_slice(&body_post).unwrap();
    assert_eq!(post_json["status"], "ok");

    // Rejeição atômica (amp_1.gain fora de 0..10 não altera estado anterior)
    let (st_bad, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"amp_1.bass": 4.0, "amp_1.gain": 99.0}"#,
    );
    assert_eq!(st_bad, 400);

    let (st_state, _, body_state) = dispatch(&ctx, "/api/state", "GET", &[]);
    assert_eq!(st_state, 200);
    let state_json: serde_json::Value = serde_json::from_slice(&body_state).unwrap();
    assert!((state_json["amp_1.gain"].as_f64().unwrap() - 6.2).abs() < 1e-4);
    assert!((state_json["amp_1.bass"].as_f64().unwrap() - 5.0).abs() < 1e-4);

    // Rota desconhecida retorna 404
    let (st_404, _, _) = dispatch(&ctx, "/api/does_not_exist", "GET", &[]);
    assert_eq!(st_404, 404);
}
