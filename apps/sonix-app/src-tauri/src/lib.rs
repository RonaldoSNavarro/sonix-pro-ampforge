//! Biblioteca da casca desktop Tauri v2 (Spec 033 Fase 2).
//! Embute o motor DSP `sonix-desktop` sem servidor HTTP/TCP nem dependência de navegador externo.

pub mod commands;
pub mod streams;

use commands::{AppRuntime, AudioRuntimeControl, FallbackWorkerControl};
use sonix_desktop::api::{ApiContext, AppState};
use sonix_desktop::audio::{
    DesktopAudioHost, RealtimeDspChain, RealtimeDuplexAudioStreams, RealtimeDuplexEngine,
    TelemetryBuffer,
};
use sonix_desktop::calibration::CalibrationStore;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use streams::StreamCoordinator;
use tauri::{Manager, WindowEvent};

static AUDIO_HOST_SINGLETON_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Reinicia o estado do guarda singleton exclusivo para testes de contrato isolados.
pub fn reset_audio_host_count_for_test() {
    AUDIO_HOST_SINGLETON_ACTIVE.store(false, Ordering::SeqCst);
}

/// Garante no máximo uma inicialização de host de áudio por instância (Spec 033, CA-11).
pub fn ensure_single_audio_host(started_count: &AtomicUsize) -> bool {
    if AUDIO_HOST_SINGLETON_ACTIVE
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        started_count.fetch_add(1, Ordering::SeqCst);
        true
    } else {
        false
    }
}

struct RuntimeKeepAlive {
    _streams: Arc<Mutex<Option<RealtimeDuplexAudioStreams>>>,
    fallback: Arc<FallbackWorkerControl>,
}

impl Drop for RuntimeKeepAlive {
    fn drop(&mut self) {
        self.fallback.stop_and_join();
        if let Ok(mut s) = self._streams.lock() {
            *s = None;
        }
        AUDIO_HOST_SINGLETON_ACTIVE.store(false, Ordering::SeqCst);
    }
}

fn build_runtime() -> AppRuntime {
    let calib_store = CalibrationStore::new();
    let calib = calib_store.get("default");
    build_runtime_with_calibration(calib_store, calib, true)
}

/// Constrói runtime para testes de contrato de inicialização sem abrir hardware WASAPI real (Spec 050, CA-2/CA-4).
pub fn build_runtime_for_startup_test(calib: Option<(f32, f32)>) -> AppRuntime {
    build_runtime_with_calibration(CalibrationStore::new(), calib, false)
}

fn build_runtime_with_calibration(
    calib_store: CalibrationStore,
    calib: Option<(f32, f32)>,
    start_audio_host: bool,
) -> AppRuntime {
    let audio_telemetry = Arc::new(Mutex::new(None));
    let duplex = Arc::new(RealtimeDuplexEngine::new(65536));
    let telemetry = Arc::new(TelemetryBuffer::new(512));
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 2048);
    let dsp_chain = Arc::new(Mutex::new(chain));
    let param_queue = dsp_chain
        .lock()
        .ok()
        .map(|chain| chain.param_queue());

    let mut initial_state = AppState::default();
    if let Some((boost, gate)) = calib {
        initial_state.input_boost = boost;
        initial_state.gate_threshold_db = gate;
        if let Ok(mut dsp) = dsp_chain.lock() {
            dsp.set_input_boost(boost);
            dsp.set_gate_threshold_db(gate);
        }
    } else {
        // Spec 050 CA-2: sem calibração prévia, inicia em Standby seguro para evitar chiado/microfonia imediata
        initial_state.amp_standby = true;
        if let Ok(mut dsp) = dsp_chain.lock() {
            dsp.set_amp_standby(true);
        }
    }

    let started_counter = AtomicUsize::new(0);
    let (active_streams, running, fallback_worker) = if start_audio_host && ensure_single_audio_host(&started_counter) {
        let streams = DesktopAudioHost::start_realtime_wasapi_duplex(
            duplex.clone(),
            dsp_chain.clone(),
            telemetry.clone(),
        )
        .ok();

        if let Some(ref s) = streams {
            duplex.set_target_period(s.telemetry.output_period_frames as usize);
            if let Ok(mut guard) = audio_telemetry.lock() {
                *guard = Some(s.telemetry.clone());
            }
        }

        let run_flag = Arc::new(AtomicBool::new(true));
        let worker = if streams.is_none() {
            let duplex_clone = duplex.clone();
            let telemetry_clone = telemetry.clone();
            let dsp_clone = dsp_chain.clone();
            let flag = run_flag.clone();
            Some(thread::spawn(move || {
                let mut in_buf = vec![0.0f32; 4096];
                let mut out_buf = vec![0.0f32; 4096];
                while flag.load(Ordering::Relaxed) {
                    thread::park_timeout(Duration::from_millis(3));
                    let period = duplex_clone.target_period().clamp(32, 4096);
                    duplex_clone.pop_samples_for_processing(&mut in_buf[..period]);
                    if let Ok(mut dsp) = dsp_clone.lock() {
                        dsp.process(&in_buf[..period], &mut out_buf[..period]);
                    }
                    let peak = out_buf[..period]
                        .iter()
                        .fold(0.0f32, |acc, &x| acc.max(x.abs()));
                    let peak_db = if peak > 1e-6 {
                        20.0 * peak.log10()
                    } else {
                        -120.0
                    };
                    telemetry_clone.update_peaks(peak_db - 6.0, peak_db, 1.2);
                    telemetry_clone.push_waveform(&out_buf[..period]);
                }
            }))
        } else {
            None
        };

        (Arc::new(Mutex::new(streams)), run_flag, worker)
    } else {
        (Arc::new(Mutex::new(None)), Arc::new(AtomicBool::new(false)), None)
    };

    let fallback_ctrl = Arc::new(FallbackWorkerControl {
        running,
        worker: Mutex::new(fallback_worker),
    });

    let audio_ctrl = Arc::new(AudioRuntimeControl {
        duplex,
        dsp_chain: dsp_chain.clone(),
        active_streams: active_streams.clone(),
        fallback: fallback_ctrl.clone(),
    });

    let mut api_ctx = ApiContext::new(
        audio_telemetry,
        Arc::new(Mutex::new(initial_state)),
        telemetry,
        Some(dsp_chain),
        param_queue,
        Some("default"),
        Some(calib_store),
    );
    api_ctx.duplex_engine = Some(audio_ctrl.duplex.clone());

    AppRuntime {
        api_ctx,
        streams: Arc::new(StreamCoordinator::new()),
        audio_ctrl: Some(audio_ctrl),
        _keep_alive: Arc::new(RuntimeKeepAlive {
            _streams: active_streams,
            fallback: fallback_ctrl,
        }),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let runtime = build_runtime();
    let streams_for_events = Arc::clone(&runtime.streams);

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(runtime)
        .on_window_event(move |window, event| match event {
            WindowEvent::Resized(_) => {
                if let Ok(minimized) = window.is_minimized() {
                    streams_for_events.set_visible(!minimized);
                }
            }
            WindowEvent::Focused(focused) => {
                if *focused {
                    streams_for_events.set_visible(true);
                }
            }
            WindowEvent::Destroyed => {
                streams_for_events.set_visible(false);
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_audio_status,
            commands::get_telemetry,
            commands::get_devices,
            commands::get_state,
            commands::get_tuner,
            commands::get_graph,
            commands::get_scope,
            commands::set_params,
            commands::load_preset,
            commands::save_preset,
            commands::import_preset,
            commands::export_preset,
            commands::list_presets,
            commands::select_scene,
            commands::load_ir,
            commands::set_input_di,
            commands::calibrate_input,
            commands::measure_latency,
            commands::set_audio_device,
            commands::set_stream_visible,
            commands::start_streams,
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao executar aplicação Tauri SONIX PRO // AMPFORGE");
}
