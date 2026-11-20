//! Teste de auditoria 2026-11-20 com feature `fault-injection` (AUD-04)

use std::sync::{Arc, Mutex};

use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{
    recover_dsp_chain_if_faulted, DesktopAudioHost, RealtimeDspChain, RealtimeDuplexEngine,
    TelemetryBuffer,
};

#[test]
fn aud_04_panico_no_callback_vira_silencio_e_recupera() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    let pq = chain.param_queue();
    let dsp = Arc::new(Mutex::new(chain));
    let telemetry = Arc::new(TelemetryBuffer::new(512));
    let duplex = Arc::new(RealtimeDuplexEngine::new(4096));

    let ctx = ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        telemetry.clone(),
        Some(dsp.clone()),
        Some(pq),
        None,
        None,
    );

    let (st_ok, _, _) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"gate.enabled": false, "amp_1.standby": false}"#,
    );
    assert_eq!(st_ok, 200);

    // Injeta pânico real dentro de RealtimeDspChain::process via fault-injection
    let (st_inj, _, body_inj) = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"debug.inject_panic": true}"#,
    );
    assert_eq!(
        st_inj,
        200,
        "debug.inject_panic deveria retornar 200: {}",
        String::from_utf8_lossy(&body_inj)
    );

    let input_block: Vec<f32> = (0..128)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / 48000.0)).sin() * 0.25)
        .collect();
    duplex.push_captured_samples(&input_block);

    let mut out_interleaved = vec![0.99_f32; 256];
    let mut mono_in = vec![0.0_f32; 128];
    let mut mono_out = vec![0.0_f32; 128];
    DesktopAudioHost::process_output_callback_f32(
        &mut out_interleaved,
        2,
        48000.0,
        &mut mono_in,
        &mut mono_out,
        &duplex,
        &dsp,
        &telemetry,
        None,
    );

    assert!(
        out_interleaved.iter().all(|&s| s == 0.0),
        "AUD-04: bloco com pânico no callback deve sair totalmente zerado"
    );
    assert!(
        telemetry.is_engine_faulted(),
        "AUD-04: engine_faulted deve ligar após pânico no callback"
    );
    assert_eq!(telemetry.panic_count(), 1);
    assert!(
        dsp.try_lock().is_ok(),
        "AUD-04: Mutex<RealtimeDspChain> NÃO pode ficar envenenado"
    );

    // Rotina de recuperação reconstrói a cadeia e limpa a flag
    let recovered = recover_dsp_chain_if_faulted(&ctx);
    assert_eq!(recovered, Some(1));
    assert!(!telemetry.is_engine_faulted());
    assert_eq!(telemetry.recovery_count(), 1);

    // Próximo bloco com entrada não nula sai com áudio normal não nulo
    duplex.push_captured_samples(&input_block);
    let mut next_interleaved = vec![0.0_f32; 256];
    DesktopAudioHost::process_output_callback_f32(
        &mut next_interleaved,
        2,
        48000.0,
        &mut mono_in,
        &mut mono_out,
        &duplex,
        &dsp,
        &telemetry,
        None,
    );
    assert!(
        next_interleaved.iter().any(|&s| s.abs() > 1e-4),
        "AUD-04: após recuperação, o bloco seguinte com sinal deve produzir saída não nula"
    );
}
