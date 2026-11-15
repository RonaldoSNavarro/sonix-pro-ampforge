//! crates/sonix-desktop/tests/contract_049_asio_input_reamp.rs
//! Testes de Contrato para Spec 049 (Parte F — CA-36, CA-37, CA-38, CA-39):
//! - CA-36: Feature opcional `asio` em `sonix-desktop` (sem SDK no repo).
//! - CA-37: `set_audio_device({mode:"asio"})` sem driver retorna `mode_unavailable` sem derrubar WASAPI.
//! - CA-38: `input.mode = "instrument" | "line"` equaliza -10 dBV vs +4 dBu no mesmo nível nominal (±1 dB).
//! - CA-39: Reamp mede atraso por loopback e alinha gravação contra a DI com erro <= 1 amostra + metadados.

use serde_json::json;
use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{AudioMode, DesktopAudioHost, RealtimeDspChain, TelemetryBuffer};
use sonix_desktop::reamp::{align_reamp_recording, measure_loopback_delay_samples, ReampSessionMetadata};
use std::sync::{Arc, Mutex};

fn make_ctx() -> (ApiContext, Arc<Mutex<RealtimeDspChain>>) {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 1024);
    let dsp = Arc::new(Mutex::new(chain));
    let pq = dsp.lock().unwrap().param_queue();
    let ctx = ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(128)),
        Some(dsp.clone()),
        Some(pq),
        None,
        None,
    );
    (ctx, dsp)
}

#[test]
fn ca37_asio_unavailable_returns_mode_unavailable_and_preserves_wasapi() {
    let host = DesktopAudioHost::new();
    // Sem feature `asio` compilada (ou sem driver), validate_requested_mode("asio") retorna mode_unavailable
    if !cfg!(feature = "asio") {
        let err = host
            .validate_requested_mode("asio")
            .expect_err("CA-37: sem feature asio, modo asio deve retornar mode_unavailable");
        assert!(
            err.contains("mode_unavailable"),
            "CA-37: erro deve conter 'mode_unavailable', obtido: {}",
            err
        );
    }
    // WASAPI continua disponível
    assert!(
        host.has_mode_support(AudioMode::WasapiShared),
        "CA-37: WASAPI deve continuar disponível"
    );
    assert!(host.validate_requested_mode("wasapi").is_ok());
}

#[test]
fn ca38_input_mode_instrument_vs_line_matches_nominal_level_within_1db() {
    let (mut ctx, dsp) = make_ctx();
    let sr = 48000.0f32;
    let n = 2048usize;

    // Desabilita gate/amp/cab para medir exatamente o nível que chega ao primeiro bloco
    let base_params = json!({
        "gate.enabled": false,
        "drive.enabled": false,
        "amp_1.enabled": false,
        "cab_1.enabled": false,
        "eq_1.enabled": false,
        "comp_2.enabled": false,
        "mod_2.enabled": false,
        "dly_1.enabled": false,
        "rev_1.enabled": false,
        "input.boost": 1.0
    });
    let (st, _, _) = dispatch(
        &mut ctx,
        "/api/params",
        "POST",
        &serde_json::to_vec(&base_params).unwrap(),
    );
    assert_eq!(st, 200);

    // 1. Modo instrument: sinal nominal -10 dBV (0.316228 Vrms)
    let (st_inst, _, body_inst) = dispatch(
        &mut ctx,
        "/api/params",
        "POST",
        br#"{"input.mode":"instrument"}"#,
    );
    assert_eq!(st_inst, 200, "CA-38: {:?}", String::from_utf8_lossy(&body_inst));

    let v_inst = 0.316228f32;
    let in_inst: Vec<f32> = (0..n)
        .map(|i| v_inst * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sr).sin())
        .collect();
    let mut out_inst = vec![0.0f32; n];
    {
        let mut c = dsp.lock().unwrap();
        c.process(&in_inst, &mut out_inst);
    }
    let rms_inst = (out_inst[512..].iter().map(|x| x * x).sum::<f32>() / (n - 512) as f32).sqrt();

    // 2. Modo line: sinal nominal +4 dBu (1.2276 Vrms)
    let (st_line, _, body_line) = dispatch(
        &mut ctx,
        "/api/params",
        "POST",
        br#"{"input.mode":"line"}"#,
    );
    assert_eq!(st_line, 200, "CA-38: {:?}", String::from_utf8_lossy(&body_line));

    let v_line = 1.2276f32;
    let in_line: Vec<f32> = (0..n)
        .map(|i| v_line * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sr).sin())
        .collect();
    let mut out_line = vec![0.0f32; n];
    {
        let mut c = dsp.lock().unwrap();
        c.process(&in_line, &mut out_line);
    }
    let rms_line = (out_line[512..].iter().map(|x| x * x).sum::<f32>() / (n - 512) as f32).sqrt();

    let diff_db = 20.0 * (rms_line / rms_inst.max(1e-12)).log10().abs();
    assert!(
        diff_db <= 1.0,
        "CA-38: nível nominal entre instrument (-10 dBV) e line (+4 dBu) diferiu {:.3} dB (> 1 dB)",
        diff_db
    );
}

#[test]
fn ca39_reamp_loopback_measures_delay_and_aligns_within_1_sample() {
    let n = 1024usize;
    let true_delay = 173usize;

    // Pulso de loopback (chirp/impulso curto)
    let mut probe = vec![0.0f32; 512];
    probe[16] = 0.9;
    probe[17] = -0.45;
    let mut loopback_rec = vec![0.0f32; 1024];
    for (i, &s) in probe.iter().enumerate() {
        if i + true_delay < loopback_rec.len() {
            loopback_rec[i + true_delay] = s;
        }
    }

    let measured = measure_loopback_delay_samples(&probe, &loopback_rec);
    assert!(
        (measured as isize - true_delay as isize).abs() <= 1,
        "CA-39: atraso medido={} diferente do real={} (> 1 amostra)",
        measured,
        true_delay
    );

    // DI e retorno atrasado pelo mesmo roundtrip
    let di: Vec<f32> = (0..n)
        .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 330.0 * i as f32 / 48000.0).sin())
        .collect();
    let mut reamp_rec = vec![0.0f32; n + true_delay + 64];
    for i in 0..n {
        reamp_rec[i + true_delay] = di[i];
    }

    let aligned = align_reamp_recording(&reamp_rec, measured, n);
    assert_eq!(aligned.len(), n);
    let max_err = aligned
        .iter()
        .zip(di.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        max_err < 1e-5,
        "CA-39: erro após alinhamento = {} (esperado ~0)",
        max_err
    );

    let meta = ReampSessionMetadata {
        measured_delay_samples: measured,
        backend: "wasapi".to_string(),
        driver: "WASAPI Shared".to_string(),
        sample_rate: 48000,
        buffer_size: 128,
        di_source: "di_take_01.wav".to_string(),
    };
    let json = serde_json::to_value(&meta).unwrap();
    assert_eq!(json["measured_delay_samples"], measured);
    assert_eq!(json["backend"], "wasapi");
    assert_eq!(json["sample_rate"], 48000);
    assert_eq!(json["buffer_size"], 128);
}
