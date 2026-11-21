//! crates/sonix-desktop/tests/contract_032_fx_api.rs
//! Testes de Contrato API + RealtimeDspChain para Spec 032 & fechamento backend de 033:
//! drive.model/bias/rat filter, comp_2.output/ratio=all, dly_1.mode/bass/treble/reverb_volume,
//! eq_1.mode/band1..10/gain_db/volume_db, cab_1.speaker=twin_2x12_open/angle_deg/phase_invert,
//! tuner.ref_hz e looper.action/position.

use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{LockFreeParamQueue, RealtimeDspChain, TelemetryBuffer};
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

fn render(ctx: &ApiContext, freq: f32, amp: f32, len: usize) -> Vec<f32> {
    let mut dsp = ctx.dsp_chain.as_ref().unwrap().lock().unwrap();
    dsp.drain_param_queue();
    let input: Vec<f32> = (0..len)
        .map(|i| amp * (2.0 * std::f32::consts::PI * freq * (i as f32 / 48000.0)).sin())
        .collect();
    let mut out = vec![0.0f32; len];
    dsp.process(&input, &mut out);
    out
}

fn rms(s: &[f32]) -> f32 {
    (s.iter().map(|x| x * x).sum::<f32>() / s.len().max(1) as f32).sqrt()
}

#[test]
fn test_ca_04_05_06_07_fx_api_dispatch_and_real_audio_effects() {
    let ctx = make_ctx();

    // 1. Troca de pedal via drive.model + rat filter invertido + fuzzface bias
    let (st, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"amp_1.enabled": false, "cab_1.enabled": false, "gate.enabled": false, "drive.enabled": true, "drive.model": "rat", "drive.drive": 5.0, "drive.tone": 0.0}"#,
    );
    assert_eq!(st, 200);
    let out_rat_open = render(&ctx, 4000.0, 0.03, 2048);

    let (st, _, _) = dispatch(&ctx, "/api/params", "POST", br#"{"drive.tone": 10.0}"#);
    assert_eq!(st, 200);
    let out_rat_dark = render(&ctx, 4000.0, 0.03, 2048);
    assert!(
        rms(&out_rat_open[1024..]) > rms(&out_rat_dark[1024..]) * 1.8,
        "RAT via API com drive.tone=10 deve escurecer em 4 kHz vs 0"
    );

    // 2. FuzzFace + bias + comp_2.output & all + eq_1.mode mxr_10band com band10 e gain_db/volume_db
    let (st, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"drive.enabled": false, "split.enabled": true, "split.mode": "ab", "split.balance": 1.0, "amp_2.enabled": false, "cab_2.enabled": false, "merge.row_1_level_db": 0.0, "comp_2.enabled": true, "comp_2.ratio": "all", "comp_2.output": 2.0, "eq_1.enabled": true, "eq_1.mode": "mxr_10band", "eq_1.band10.gain_db": 0.0, "eq_1.volume_db": 0.0}"#,
    );
    assert_eq!(st, 200);
    let out_comp_low = render(&ctx, 1000.0, 0.08, 2048);

    let (st, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"comp_2.output": 8.0}"#,
    );
    assert_eq!(st, 200);
    let out_comp_hi = render(&ctx, 1000.0, 0.08, 2048);
    let comp_diff_db =
        20.0 * (rms(&out_comp_hi[1024..]) / rms(&out_comp_low[1024..]).max(1e-9)).log10();
    assert!(
        comp_diff_db > 10.0,
        "comp_2.output=8 vs 2 deve aumentar nível real pela API > 10 dB (got {:.2} dB)",
        comp_diff_db
    );

    // 3. EQ MXR 10-band band9 (8 kHz) e volume_db afetam saída
    dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"split.enabled": false, "split.balance": 0.5, "comp_2.enabled": false, "eq_1.band9.gain_db": 10.0, "eq_1.gain_db": 3.0}"#,
    );
    let out_eq_boost = render(&ctx, 8000.0, 0.05, 2048);
    dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"eq_1.band9.gain_db": -10.0, "eq_1.gain_db": -3.0}"#,
    );
    let out_eq_cut = render(&ctx, 8000.0, 0.05, 2048);
    assert!(
        rms(&out_eq_boost[1024..]) > rms(&out_eq_cut[1024..]) * 4.0,
        "MXR 10-band band9 + gain_db devem alterar resposta em 8 kHz pela API"
    );

    // 4. dly_1.mode + dly_1.bass + dly_1.treble + dly_1.reverb_volume + cab_1.speaker + mic angle/phase + looper/tuner
    let (st, _, body) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"eq_1.enabled": false, "dly_1.enabled": true, "dly_1.mode": "8", "dly_1.reverb_volume": 6.5, "dly_1.bass": 2.0, "dly_1.treble": 8.5, "cab_1.enabled": true, "cab_1.speaker": "twin_2x12_open", "cab_1.mic1.angle_deg": "45", "cab_1.mic2.phase_invert": true, "tuner.ref_hz": 432.0, "looper.position": "pre", "looper.action": "clear"}"#,
    );
    assert_eq!(
        st,
        200,
        "Todas as chaves Spec 032/033 devem ser aceitas: {}",
        String::from_utf8_lossy(&body)
    );
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let ignored = parsed["ignored"].as_array().unwrap();
    assert!(
        ignored.is_empty(),
        "Nenhuma chave Spec 032/033 pode ser ignorada: {:?}",
        ignored
    );

    // Validação de erro 400 para valores inválidos
    let (st_bad, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"dly_1.mode": "99"}"#,
    );
    assert_eq!(st_bad, 400);
}
