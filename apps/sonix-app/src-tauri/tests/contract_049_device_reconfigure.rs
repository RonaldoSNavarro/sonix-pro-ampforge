//! apps/sonix-app/src-tauri/tests/contract_049_device_reconfigure.rs
//! Testes de Contrato para Spec 049 (Parte A — CA-4, CA-5, CA-6):
//! - CA-4: Reconfiguração encerra stream antigo antes de abrir o novo; falha restaura stream anterior.
//! - CA-5: Thread fallback de lib.rs é parada e joined antes de iniciar stream real (<= 50 ms).
//! - CA-6: Estado global de ensure_single_audio_host é limpo no encerramento e permite reabertura.

use serde_json::json;
use sonix_app_lib::commands::{
    configure_audio_device_generic, FallbackWorkerControl,
};
use sonix_app_lib::{ensure_single_audio_host, reset_audio_host_count_for_test};
use sonix_desktop::api::{ApiContext, AppState};
use sonix_desktop::audio::{
    AudioDriverError, AudioStatusTelemetry, RealtimeDspChain, RealtimeDuplexEngine,
    TelemetryBuffer,
};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct MockStream {
    alive: Arc<AtomicBool>,
}

impl Drop for MockStream {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
    }
}

fn mock_telemetry(sr: u32, period: u32, in_dev: &str, out_dev: &str) -> AudioStatusTelemetry {
    AudioStatusTelemetry {
        backend: "wasapi".to_string(),
        mode: "wasapi_shared".to_string(),
        fallback_reason: None,
        raw_processing: false,
        sample_rate: sr,
        input_period_frames: period,
        output_period_frames: period,
        input_device: in_dev.to_string(),
        output_device: out_dev.to_string(),
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

fn build_context() -> (ApiContext, Arc<RealtimeDuplexEngine>, Arc<Mutex<RealtimeDspChain>>) {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 2048);
    let dsp = Arc::new(Mutex::new(chain));
    let pq = dsp.lock().unwrap().param_queue();
    let ctx = ApiContext::new(
        Arc::new(Mutex::new(Some(mock_telemetry(48000, 128, "InA", "OutA")))),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(128)),
        Some(dsp.clone()),
        Some(pq),
        None,
        None,
    );
    let duplex = Arc::new(RealtimeDuplexEngine::with_target_period(4096, 128));
    (ctx, duplex, dsp)
}

#[test]
fn ca4_reconfigure_stops_old_stream_before_opening_new_and_rolls_back_on_error() {
    let (ctx, duplex, dsp) = build_context();
    let old_alive = Arc::new(AtomicBool::new(true));
    let active_streams: Arc<Mutex<Option<MockStream>>> = Arc::new(Mutex::new(Some(MockStream {
        alive: old_alive.clone(),
    })));
    let saw_old_dropped_before_open = Arc::new(AtomicBool::new(false));
    let calls = Arc::new(AtomicUsize::new(0));
    let restored_alive = Arc::new(AtomicBool::new(false));

    let res = configure_audio_device_generic(
        &json!({
            "driver": "wasapi",
            "input_device": "BrokenIn",
            "output_device": "BrokenOut",
            "sample_rate": 96000,
            "buffer_size": 256
        }),
        &ctx,
        &duplex,
        &dsp,
        &active_streams,
        None,
        |_d, _dsp, _tel, in_dev, _out_dev, req_sr, req_buf| {
            let call_idx = calls.fetch_add(1, Ordering::SeqCst);
            if call_idx == 0 {
                // O stream antigo DEVE ter sido encerrado (dropped) ANTES de abrir o novo!
                saw_old_dropped_before_open.store(!old_alive.load(Ordering::SeqCst), Ordering::SeqCst);
                assert_eq!(in_dev, Some("BrokenIn"));
                assert_eq!(req_sr, Some(96000));
                Err(AudioDriverError::InitFailed("Falha simulada".to_string()))
            } else {
                // Rollback deve reabrir a configuração anterior (InA / OutA / 48000 / 128)
                assert_eq!(in_dev, Some("InA"));
                assert_eq!(req_sr, Some(48000));
                assert_eq!(req_buf, Some(128));
                restored_alive.store(true, Ordering::SeqCst);
                Ok((
                    Some(MockStream {
                        alive: restored_alive.clone(),
                    }),
                    mock_telemetry(48000, 128, "InA", "OutA"),
                ))
            }
        },
    );

    assert!(res.is_err(), "CA-4: deve retornar Err explícito quando novo stream falha");
    assert!(
        saw_old_dropped_before_open.load(Ordering::SeqCst),
        "CA-4: stream antigo deve ser encerrado ANTES de tentar abrir o novo"
    );
    assert!(
        restored_alive.load(Ordering::SeqCst) && active_streams.lock().unwrap().is_some(),
        "CA-4: stream anterior deve ser restaurado após falha na abertura do novo"
    );
}

#[test]
fn ca5_fallback_worker_stops_and_joins_within_50ms_on_set_audio_device() {
    let (ctx, duplex, dsp) = build_context();
    let active_streams: Arc<Mutex<Option<MockStream>>> = Arc::new(Mutex::new(None));

    // 1. Mede diretamente stop_and_join() <= 50 ms
    let running1 = Arc::new(AtomicBool::new(true));
    let started1 = Arc::new(AtomicBool::new(false));
    let r1 = running1.clone();
    let s1 = started1.clone();
    let h1 = thread::spawn(move || {
        s1.store(true, Ordering::SeqCst);
        while r1.load(Ordering::Relaxed) {
            thread::park_timeout(Duration::from_millis(3));
        }
    });
    while !started1.load(Ordering::SeqCst) {
        thread::yield_now();
    }
    let fb1 = FallbackWorkerControl {
        running: running1.clone(),
        worker: Mutex::new(Some(h1)),
    };
    let t0 = Instant::now();
    fb1.stop_and_join();
    let join_elapsed = t0.elapsed();
    assert!(
        join_elapsed <= Duration::from_millis(50),
        "CA-5: parada + join da thread fallback levou {:?} (> 50 ms)",
        join_elapsed
    );

    // 2. Verifica que configure_audio_device_generic para e faz join da thread fallback antes do opener
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();
    let handle = thread::spawn(move || {
        while running_clone.load(Ordering::Relaxed) {
            thread::park_timeout(Duration::from_millis(3));
        }
    });
    let fallback = FallbackWorkerControl {
        running: running.clone(),
        worker: Mutex::new(Some(handle)),
    };

    let res = configure_audio_device_generic(
        &json!({
            "driver": "wasapi",
            "input_device": "InA",
            "output_device": "OutA",
            "sample_rate": 48000,
            "buffer_size": 128
        }),
        &ctx,
        &duplex,
        &dsp,
        &active_streams,
        Some(&fallback),
        |_d, _dsp, _tel, _in_dev, _out_dev, _sr, _buf| {
            // Quando o opener do stream real roda, a thread fallback já deve estar parada
            assert!(
                !running.load(Ordering::SeqCst),
                "CA-5: fallback worker deve parar antes de iniciar o stream real"
            );
            Ok((
                Some(MockStream {
                    alive: Arc::new(AtomicBool::new(true)),
                }),
                mock_telemetry(48000, 128, "InA", "OutA"),
            ))
        },
    );
    assert!(res.is_ok(), "CA-5: {:?}", res);
    assert!(
        fallback.worker.lock().unwrap().is_none(),
        "CA-5: handle da thread fallback deve ter sido joined e limpo"
    );
}

#[test]
fn ca6_single_audio_host_resets_and_allows_reopen() {
    reset_audio_host_count_for_test();
    let count = AtomicUsize::new(0);
    assert!(ensure_single_audio_host(&count));
    assert!(!ensure_single_audio_host(&count), "Segunda chamada sem reset deve ser bloqueada");
    reset_audio_host_count_for_test();
    assert!(ensure_single_audio_host(&count), "CA-6: após limpeza deve permitir reabertura");
    reset_audio_host_count_for_test();
}

#[test]
fn test_configure_audio_device_accepts_wasapi_shared_480_frames() {
    let (ctx, duplex, dsp) = build_context();
    let active_streams: Arc<Mutex<Option<MockStream>>> = Arc::new(Mutex::new(None));
    let res = configure_audio_device_generic(
        &json!({
            "driver": "wasapi",
            "input_device": "InA",
            "output_device": "Alto-falantes",
            "sample_rate": 48000,
            "buffer_size": 480
        }),
        &ctx,
        &duplex,
        &dsp,
        &active_streams,
        None,
        |_d, _dsp, _tel, _in_dev, _out_dev, _sr, req_buf| {
            assert_eq!(req_buf, Some(480));
            Ok((
                Some(MockStream {
                    alive: Arc::new(AtomicBool::new(true)),
                }),
                mock_telemetry(48000, 480, "InA", "Alto-falantes"),
            ))
        },
    );
    assert!(res.is_ok(), "Deve aceitar período 480 de WASAPI Compartilhado: {:?}", res);
}

