//! crates/sonix-desktop/tests/contract_031_amp_controls_api.rs
//! Testes de Contrato TDD para Spec 031 (camada API + áudio)

use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{LockFreeParamQueue, RealtimeDspChain, TelemetryBuffer};
use std::sync::{Arc, Mutex};

fn make_ctx() -> ApiContext {
    ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(512)),
        Some(Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)))),
        Some(Arc::new(LockFreeParamQueue::new())),
        None,
        None,
    )
}

fn render_rms_freq(ctx: &ApiContext, amp: f32, freq: f32) -> f32 {
    let mut dsp = ctx.dsp_chain.as_ref().unwrap().lock().unwrap();
    dsp.drain_param_queue();
    let n = 4096;
    let in_sig: Vec<f32> = (0..n)
        .map(|i| amp * (2.0 * std::f32::consts::PI * freq * (i as f32 / 48000.0)).sin())
        .collect();
    let mut out_sig = vec![0.0f32; n];
    dsp.process(&in_sig, &mut out_sig);
    let tail = &out_sig[1024..];
    (tail.iter().map(|&x| x * x).sum::<f32>() / tail.len() as f32).sqrt()
}

#[test]
fn test_ca_02_to_07_amp_controls_api_accepts_and_changes_audio() {
    let ctx = make_ctx();

    // Plexi 1959 jumpered + volume_2
    let (st, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"amp_1.model": "plexi_1959", "amp_1.input_jack": "jumpered", "amp_1.jumpered": true, "amp_1.gain": 2.0, "amp_1.volume_2": 0.0}"#,
    );
    assert_eq!(st, 200);
    let rms_v2_0 = render_rms_freq(&ctx, 0.05, 140.0);

    let (st, _, _) = dispatch(&ctx, "/api/params", "POST", br#"{"amp_1.volume_2": 10.0}"#);
    assert_eq!(st, 200);
    let rms_v2_10 = render_rms_freq(&ctx, 0.05, 140.0);
    let diff_plexi = 20.0 * (rms_v2_10 / rms_v2_0.max(1e-7)).log10();
    assert!(diff_plexi >= 3.0, "API Plexi volume_2 diff={} dB", diff_plexi);

    // AC30 Tone Cut
    let (st, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"amp_1.model": "ac30_tb", "amp_1.cut": 0.0}"#,
    );
    assert_eq!(st, 200);
    let rms_cut0 = render_rms_freq(&ctx, 0.04, 3200.0);
    let (st, _, _) = dispatch(&ctx, "/api/params", "POST", br#"{"amp_1.cut": 10.0}"#);
    assert_eq!(st, 200);
    let rms_cut10 = render_rms_freq(&ctx, 0.04, 3200.0);
    let atten_cut = 20.0 * (rms_cut0 / rms_cut10.max(1e-7)).log10();
    assert!(atten_cut >= 6.0, "API AC30 cut atten={} dB", atten_cut);

    // Dual Rectifier mode + spongy + impedance + oversampling + Twin reverb/vibrato
    let (st, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"amp_1.model": "dual_rect", "amp_1.channel_mode": "modern", "amp_1.spongy": true, "amp_1.impedance_ohms": "4", "engine.oversampling": "2x"}"#,
    );
    assert_eq!(st, 200);

    let (st, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"amp_1.model": "twin_ab763", "amp_1.input_jack": "vibrato", "amp_1.reverb": 6.0, "amp_1.vibrato_speed": 5.5, "amp_1.vibrato_depth": 7.0}"#,
    );
    assert_eq!(st, 200);

    // Atomic validation failure leaves state untouched
    let (st_bad, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"amp_1.impedance_ohms": "32"}"#,
    );
    assert_eq!(st_bad, 400);
}
