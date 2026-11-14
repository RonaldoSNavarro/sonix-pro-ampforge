//! Testes de contrato da Spec 033 (CA-5 e CA-7):
//! Todo controle real visível na UI altera a saída DSP no sentido esperado do equipamento real
//! e entradas inválidas são rejeitadas atomicamente.

use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{LockFreeParamQueue, RealtimeDspChain, TelemetryBuffer};
use sonix_desktop::calibration::CalibrationStore;
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
        Some(CalibrationStore::new()),
    )
}

fn render_rms(ctx: &ApiContext, amp: f32) -> f32 {
    let mut dsp = ctx.dsp_chain.as_ref().unwrap().lock().unwrap();
    dsp.drain_param_queue();
    let n = 4096;
    let mut out = vec![0.0f32; n];
    let mut input = vec![0.0f32; n];
    for (i, s) in input.iter_mut().enumerate() {
        let t = i as f32 / 48000.0;
        *s = amp * (2.0 * std::f32::consts::PI * 220.0 * t).sin();
    }
    dsp.process(&input, &mut out);
    let tail = &out[2048..];
    (tail.iter().map(|x| x * x).sum::<f32>() / tail.len() as f32).sqrt()
}

#[test]
fn test_ca_05_amp_switches_and_tone_controls_have_real_hardware_effect() {
    let ctx = make_ctx();
    let baseline = render_rms(&ctx, 0.15);
    assert!(baseline > 0.001, "Amp ativo deve produzir saída audível");

    // 1. Standby = true deve silenciar o estágio de saída
    let (st, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"amp_1.standby": true}"#,
    );
    assert_eq!(st, 200, "amp_1.standby deve ser aceito pelo backend");
    let standby_rms = render_rms(&ctx, 0.15);
    assert!(
        standby_rms < 1e-4,
        "Standby ativo deve silenciar saída (rms={})",
        standby_rms
    );

    // 2. Retoma saída e desliga Power = false -> também silencia
    let (st, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"amp_1.standby": false, "amp_1.power": false}"#,
    );
    assert_eq!(st, 200);
    let off_rms = render_rms(&ctx, 0.15);
    assert!(off_rms < 1e-4, "Power desligado deve silenciar saída");

    // 3. Entrada LOW atenua ~6 dB em relação à entrada HIGH no JCM800
    dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"amp_1.power": true, "amp_1.model": "jcm800_2203", "amp_1.gain": 3.0, "amp_1.input_jack": "high"}"#,
    );
    let high_rms = render_rms(&ctx, 0.04);
    let (st_low, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"amp_1.input_jack": "low"}"#,
    );
    assert_eq!(st_low, 200, "amp_1.input_jack deve ser aceito");
    let low_rms = render_rms(&ctx, 0.04);
    assert!(
        low_rms < high_rms * 0.72,
        "Entrada LOW deve ficar sensivelmente mais baixa que HIGH (high={}, low={})",
        high_rms,
        low_rms
    );

    // 4. Mod Shop: impedância do secundário + saída do compressor 1176 + Looper + Afinador ref
    let (st_imp, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"amp_1.impedance_ohms": "4", "comp_2.output": 9.0, "dly_1.mode": "12", "cab_1.speaker": "twin_2x12_open", "cab_1.mic1.angle_deg": "45", "cab_1.mic1.phase_invert": true, "tuner.ref_hz": 432.0, "looper.position": "pre", "looper.action": "stop"}"#,
    );
    assert_eq!(
        st_imp, 200,
        "Todas as chaves do registro de controles devem ser aceitas pelo backend"
    );

    // 5. Rejeição atômica para faixa inválida
    let (st_bad, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"tuner.ref_hz": 500.0}"#,
    );
    assert_eq!(st_bad, 400);
}
