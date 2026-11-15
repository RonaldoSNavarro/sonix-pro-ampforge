//! Testes de contrato da Spec 034 CA-3, CA-4 e CA-9 — Configuração de dispositivo de áudio no shell Tauri

use serde_json::json;
use sonix_app_lib::commands::configure_audio_device_with_opener;
use sonix_desktop::api::{ApiContext, AppState};
use sonix_desktop::audio::{
    AudioDriverError, AudioStatusTelemetry, RealtimeDspChain, RealtimeDuplexEngine, TelemetryBuffer,
};
use std::sync::{Arc, Mutex};

fn mock_telemetry(sr: u32, period: u32) -> AudioStatusTelemetry {
    AudioStatusTelemetry {
        backend: "wasapi".to_string(),
        mode: "wasapi_shared".to_string(),
        fallback_reason: None,
        raw_processing: false,
        sample_rate: sr,
        input_period_frames: period,
        output_period_frames: period,
        input_device: "LineIn".to_string(),
        output_device: "Speakers".to_string(),
        driver_input_latency_ms: 2.5,
        driver_output_latency_ms: 2.5,
        cpu_avg_pct: 4.0,
        cpu_peak_pct: 6.0,
        underruns: 0,
        overruns: 0,
        panics: 0,
        adc_clips: 0,
        measured_roundtrip_ms: None,
        estimated_roundtrip_ms: 5.33,
        algorithmic_latency_samples: 16,
    }
}

fn build_context() -> (
    ApiContext,
    Arc<RealtimeDuplexEngine>,
    Arc<Mutex<RealtimeDspChain>>,
) {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 256);
    let pq = chain.param_queue().clone();
    let dsp = Arc::new(Mutex::new(chain));
    let tel_arc = Arc::new(TelemetryBuffer::new(256));
    let audio_tel = Arc::new(Mutex::new(Some(mock_telemetry(48000, 256))));
    let app_state = Arc::new(Mutex::new(AppState::default()));

    let ctx = ApiContext::new(
        audio_tel,
        app_state,
        tel_arc,
        Some(dsp.clone()),
        Some(pq),
        None,
        None,
    );
    let duplex = Arc::new(RealtimeDuplexEngine::with_target_period(4096, 256));
    (ctx, duplex, dsp)
}

#[test]
fn test_ca_04_configure_audio_accepts_buffer_and_buffer_size_and_rolls_back_on_error() {
    let (ctx, duplex, dsp) = build_context();
    let active_streams = Arc::new(Mutex::new(None));

    // 1. Erro ao abrir dispositivo inexistente deve retornar Err explícito e preservar audio_telemetry anterior (48000 Hz)
    let res_err = configure_audio_device_with_opener(
        &json!({
            "driver": "wasapi",
            "input_device": "NoSuchInput",
            "output_device": "NoSuchOutput",
            "sample_rate": 96000,
            "buffer": 128
        }),
        &ctx,
        &duplex,
        &dsp,
        &active_streams,
        |_d, _dsp, _tel, _in_dev, _out_dev, req_sr, req_buf| {
            assert_eq!(req_sr, Some(96000));
            assert_eq!(req_buf, Some(128));
            Err(AudioDriverError::InitFailed("Dispositivo inexistente".to_string()))
        },
    );
    assert!(res_err.is_err(), "CA-4: erro ao abrir dispositivo deve retornar Err explícito");
    assert!(
        res_err.unwrap_err().contains("Dispositivo inexistente"),
        "CA-4: mensagem de erro real deve ser preservada"
    );

    {
        let at = ctx.audio_telemetry.lock().unwrap();
        assert_eq!(
            at.as_ref().unwrap().sample_rate,
            48000,
            "CA-4: falha ao abrir dispositivo não pode mutar sample_rate ativo"
        );
    }

    // 2. "buffer_size" também deve ser aceito e propagar para duplex/dsp_chain/audio_telemetry em sucesso
    let res_ok = configure_audio_device_with_opener(
        &json!({
            "input_device": "LineIn",
            "output_device": "Speakers",
            "sample_rate": 96000,
            "buffer_size": 512
        }),
        &ctx,
        &duplex,
        &dsp,
        &active_streams,
        |_d, _dsp, _tel, _in_dev, _out_dev, req_sr, req_buf| {
            assert_eq!(req_sr, Some(96000));
            assert_eq!(req_buf, Some(512));
            Ok((None, mock_telemetry(96000, 512)))
        },
    );
    assert!(res_ok.is_ok(), "CA-4/CA-9: configure com buffer_size deve ter sucesso: {:?}", res_ok);

    let at = ctx.audio_telemetry.lock().unwrap();
    assert_eq!(at.as_ref().unwrap().sample_rate, 96000);
    assert_eq!(at.as_ref().unwrap().output_period_frames, 512);
    drop(at);

    let sr_bits = dsp
        .lock()
        .unwrap()
        .shared_sample_rate_bits()
        .load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(f32::from_bits(sr_bits), 96000.0, "CA-9: sample_rate propagado ao RealtimeDspChain");
}
