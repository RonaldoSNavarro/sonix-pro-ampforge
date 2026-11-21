//! Testes de auditoria 2026-11-20 — Erros de stream CPAL e recuperação no Tauri host (AUD-05).
//! Proibido usar `SimulatedAudioHost`.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use sonix_app_lib::commands::{invoke_dispatch, AudioDeviceConfig, AudioDeviceSupervisor};
use sonix_desktop::api::{ApiContext, AppState};
use sonix_desktop::audio::{
    AudioDriverError, AudioStatusTelemetry, CpalError, CpalErrorKind, DesktopAudioHost,
    RealtimeDspChain, RealtimeDuplexEngine, TelemetryBuffer,
};

struct FakeStreamHandle {
    alive: Arc<AtomicBool>,
}

impl Drop for FakeStreamHandle {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
    }
}

fn mock_audio_telemetry() -> AudioStatusTelemetry {
    AudioStatusTelemetry {
        backend: "wasapi".to_string(),
        mode: "wasapi_shared".to_string(),
        fallback_reason: None,
        raw_processing: false,
        sample_rate: 48000,
        input_period_frames: 128,
        output_period_frames: 128,
        input_device: "InDevA".to_string(),
        output_device: "OutDevA".to_string(),
        driver_input_latency_ms: 2.67,
        driver_output_latency_ms: 2.67,
        cpu_avg_pct: 1.0,
        cpu_peak_pct: 2.0,
        underruns: 0,
        overruns: 0,
        panics: 0,
        adc_clips: 0,
        measured_roundtrip_ms: None,
        estimated_roundtrip_ms: 8.0,
        algorithmic_latency_samples: 4,
        pitch_wet_latency_ms: 0.0,
    }
}

fn setup_real_chain_ctx() -> (
    ApiContext,
    Arc<RealtimeDuplexEngine>,
    Arc<Mutex<RealtimeDspChain>>,
    Arc<TelemetryBuffer>,
) {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 2048);
    chain.set_gate_enabled(false);
    chain.set_amp_standby(false);
    let pq = chain.param_queue();
    let dsp = Arc::new(Mutex::new(chain));
    let telemetry = Arc::new(TelemetryBuffer::new(256));
    let duplex = Arc::new(RealtimeDuplexEngine::new(4096));

    let mut st = AppState::default();
    st.gate_enabled = false;
    st.amp_standby = false;
    let ctx = ApiContext::new(
        Arc::new(Mutex::new(Some(mock_audio_telemetry()))),
        Arc::new(Mutex::new(st)),
        telemetry.clone(),
        Some(dsp.clone()),
        Some(pq),
        None,
        None,
    );
    (ctx, duplex, dsp, telemetry)
}

#[test]
fn aud_05_erro_do_stream_marca_perda() {
    let (ctx, duplex, dsp, telemetry) = setup_real_chain_ctx();
    let initial_alive = Arc::new(AtomicBool::new(true));
    let active_streams = Arc::new(Mutex::new(Some(FakeStreamHandle {
        alive: initial_alive.clone(),
    })));
    let config = Arc::new(Mutex::new(AudioDeviceConfig {
        mode: "wasapi_shared".to_string(),
        input_device: Some("InDevA".to_string()),
        output_device: Some("OutDevA".to_string()),
        sample_rate: Some(48000),
        buffer_size: Some(128),
    }));

    let supervisor = AudioDeviceSupervisor::start_with_opener(
        ctx.clone(),
        duplex,
        dsp,
        active_streams.clone(),
        config,
        |_d, _dsp, _tele, _cfg| Err(AudioDriverError::DeviceDisconnected),
        |_| {},
    );

    let err_slot = telemetry.stream_error_code_handle();
    let t0 = Instant::now();
    DesktopAudioHost::record_cpal_stream_error(
        &err_slot,
        CpalError::new(CpalErrorKind::DeviceNotAvailable),
    );

    let mut lost_in_time = false;
    while t0.elapsed() <= Duration::from_millis(200) {
        if telemetry.is_device_lost()
            && telemetry.device_state_str() == "lost"
            && !initial_alive.load(Ordering::SeqCst)
        {
            lost_in_time = true;
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }

    assert!(
        lost_in_time,
        "AUD-05: estado deveria virar 'lost' em <= 200 ms (is_lost={}, alive={}, elapsed={:?})",
        telemetry.is_device_lost(),
        initial_alive.load(Ordering::SeqCst),
        t0.elapsed()
    );

    let resp = invoke_dispatch(&ctx, "GET", "/api/audio/status", b"");
    assert_eq!(resp.status, 200);
    let parsed: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
    assert_eq!(parsed["device_state"], "lost");
    assert_eq!(parsed["connected"], false);

    drop(supervisor);
}

#[test]
fn aud_05_reabre_quando_volta() {
    let (ctx, duplex, dsp, telemetry) = setup_real_chain_ctx();
    let old_alive = Arc::new(AtomicBool::new(true));
    let new_alive = Arc::new(AtomicBool::new(false));
    let active_streams = Arc::new(Mutex::new(Some(FakeStreamHandle {
        alive: old_alive.clone(),
    })));
    let config = Arc::new(Mutex::new(AudioDeviceConfig {
        mode: "wasapi_shared".to_string(),
        input_device: Some("InDevA".to_string()),
        output_device: Some("OutDevA".to_string()),
        sample_rate: Some(48000),
        buffer_size: Some(128),
    }));

    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_w = attempts.clone();
    let new_alive_w = new_alive.clone();
    let supervisor = AudioDeviceSupervisor::start_with_opener(
        ctx.clone(),
        duplex.clone(),
        dsp.clone(),
        active_streams.clone(),
        config,
        move |_d, _dsp, _tele, cfg| {
            assert_eq!(cfg.input_device.as_deref(), Some("InDevA"));
            assert_eq!(cfg.output_device.as_deref(), Some("OutDevA"));
            assert_eq!(cfg.sample_rate, Some(48000));
            assert_eq!(cfg.buffer_size, Some(128));
            let n = attempts_w.fetch_add(1, Ordering::SeqCst) + 1;
            if n <= 2 {
                Err(AudioDriverError::DeviceDisconnected)
            } else {
                new_alive_w.store(true, Ordering::SeqCst);
                Ok((
                    Some(FakeStreamHandle {
                        alive: new_alive_w.clone(),
                    }),
                    mock_audio_telemetry(),
                ))
            }
        },
        |_| {},
    );

    let err_slot = telemetry.stream_error_code_handle();
    let t0 = Instant::now();
    DesktopAudioHost::record_cpal_stream_error(
        &err_slot,
        CpalError::new(CpalErrorKind::DeviceNotAvailable),
    );

    let mut reopened = false;
    while t0.elapsed() <= Duration::from_millis(3500) {
        if !telemetry.is_device_lost()
            && telemetry.device_state_str() == "ok"
            && new_alive.load(Ordering::SeqCst)
        {
            reopened = true;
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    assert!(
        reopened,
        "AUD-05: dispositivo deveria reabrir e voltar a 'ok' em <= 3.5 s (attempts={}, state={}, elapsed={:?})",
        attempts.load(Ordering::SeqCst),
        telemetry.device_state_str(),
        t0.elapsed()
    );
    assert!(!old_alive.load(Ordering::SeqCst));
    assert_eq!(attempts.load(Ordering::SeqCst), 3);

    let resp = invoke_dispatch(&ctx, "GET", "/api/audio/status", b"");
    let parsed: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
    assert_eq!(parsed["device_state"], "ok");
    assert_eq!(parsed["connected"], true);

    // Após recuperação, a cadeia real volta a processar áudio não nulo
    let input_block: Vec<f32> = (0..128)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / 48000.0)).sin() * 0.25)
        .collect();
    duplex.push_captured_samples(&input_block);
    let mut mono_in = vec![0.0_f32; 128];
    let mut mono_out = vec![0.0_f32; 128];
    let mut out_interleaved = vec![0.0_f32; 256];
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
        out_interleaved.iter().any(|&s| s.abs() > 1e-4),
        "AUD-05: após reabertura, cadeia DSP deve voltar a produzir áudio"
    );

    drop(supervisor);
}
