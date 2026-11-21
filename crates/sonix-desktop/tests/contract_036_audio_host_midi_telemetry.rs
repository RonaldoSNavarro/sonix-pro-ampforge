//! crates/sonix-desktop/tests/contract_036_audio_host_midi_telemetry.rs
//! Testes de contrato da Spec 036: Host de áudio real, entrada linear, MIDI real e telemetria honesta.

use serde_json::Value;
use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{
    AudioBackendType, AudioDeviceEnumerator, AudioMode, AudioStatusTelemetry, DesktopAudioDevice,
    DesktopAudioHost, InputChannel, RealtimeDspChain, RealtimeDuplexEngine, SimulatedAudioHost,
    TelemetryBuffer,
};
use sonix_desktop::midi::{DesktopMidiHost, MidiPortEnumerator};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

struct MockAudioEnumerator {
    devices: Vec<DesktopAudioDevice>,
}

impl AudioDeviceEnumerator for MockAudioEnumerator {
    fn enumerate_devices(&self) -> Vec<DesktopAudioDevice> {
        self.devices.clone()
    }
}

struct MockMidiEnumerator {
    ports: Vec<String>,
}

impl MidiPortEnumerator for MockMidiEnumerator {
    fn enumerate_input_ports(&self) -> Vec<String> {
        self.ports.clone()
    }
}

fn collect_src_files(dir: &Path, exts: &[&str], out: &mut Vec<std::path::PathBuf>) {
    if !dir.exists() {
        return;
    }
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_src_files(&path, exts, out);
        } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if exts.contains(&ext) {
                out.push(path);
            }
        }
    }
}

fn base_context() -> ApiContext {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 256);
    let pq = chain.param_queue().clone();
    ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(256)),
        Some(Arc::new(Mutex::new(chain))),
        Some(pq),
        None,
        None,
    )
}

#[test]
fn test_ca_01_no_realtek_in_production_and_injected_enumerator_respected() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let mut prod_files = Vec::new();
    for entry in fs::read_dir(root.join("crates")).unwrap() {
        let p = entry.unwrap().path().join("src");
        collect_src_files(&p, &["rs"], &mut prod_files);
    }
    collect_src_files(&root.join("apps/sonix-app/src-tauri/src"), &["rs"], &mut prod_files);
    collect_src_files(&root.join("apps/sonix-app/ui/src"), &["ts"], &mut prod_files);

    assert!(!prod_files.is_empty());
    for file in prod_files {
        let txt = fs::read_to_string(&file).unwrap();
        assert!(
            !txt.to_lowercase().contains("realtek"),
            "CA-1: Literal proibido 'Realtek' encontrado em código de produção: {}",
            file.display()
        );
    }

    let focusrite = DesktopAudioDevice {
        id: "focusrite_usb_in".to_string(),
        name: "Focusrite USB".to_string(),
        backend: AudioBackendType::Wasapi,
        is_input: true,
        supported_sample_rates: vec![44100, 48000],
        supported_buffer_sizes: vec![64, 128, 256],
    };
    let enumerator = Arc::new(MockAudioEnumerator {
        devices: vec![focusrite],
    });
    let host = DesktopAudioHost::with_enumerator(enumerator.clone());
    let listed = host.enumerate_devices();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].name, "Focusrite USB");

    let mut ctx = base_context();
    ctx.device_enumerator = enumerator;
    let (status, _ct, body) = dispatch(&ctx, "/api/devices", "GET", b"");
    assert_eq!(status, 200);
    let parsed: Value = serde_json::from_slice(&body).unwrap();
    let arr = parsed["devices"].as_array().expect("CA-1: /api/devices deve retornar objeto { devices: [...] }");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "Focusrite USB");
}

#[test]
fn test_ca_02_ca_03_ca_04_honest_audio_status_raw_processing_and_roundtrip_telemetry() {
    let ctx = base_context();
    let tele = AudioStatusTelemetry {
        backend: "wasapi".to_string(),
        mode: "wasapi_shared".to_string(),
        fallback_reason: None,
        raw_processing: false,
        sample_rate: 44100,
        input_period_frames: 480,
        output_period_frames: 480,
        input_device: "Focusrite USB Input".to_string(),
        output_device: "Focusrite USB Output".to_string(),
        driver_input_latency_ms: 10.88,
        driver_output_latency_ms: 10.88,
        cpu_avg_pct: 4.5,
        cpu_peak_pct: 6.2,
        underruns: 0,
        overruns: 0,
        panics: 0,
        adc_clips: 0,
        measured_roundtrip_ms: None,
        estimated_roundtrip_ms: 21.76,
        algorithmic_latency_samples: 0,
        pitch_wet_latency_ms: 0.0,
    };
    *ctx.audio_telemetry.lock().unwrap() = Some(tele);

    let (code, _ct, body) = dispatch(&ctx, "/api/audio/status", "GET", b"");
    assert_eq!(code, 200);
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["backend"], "wasapi");
    assert_eq!(v["sample_rate"], 44100);
    assert_eq!(v["input_period_frames"], 480);
    assert_eq!(v["output_period_frames"], 480);
    assert_eq!(v["raw_processing"], false, "CA-3: raw_processing não pode ser true por decreto");
    assert!(v["measured_roundtrip_ms"].is_null(), "CA-4: sem loopback real, measured_roundtrip_ms deve ser null");
    assert!(
        v["estimated_roundtrip_ms"].as_f64().unwrap_or(0.0) > 20.0,
        "CA-4: estimated_roundtrip_ms deve trazer estimativa calculada"
    );

    // Também verifica que fallback sem stream ativo reporta raw_processing=false e measured_roundtrip_ms=null
    let ctx_empty = base_context();
    let (code2, _ct2, body2) = dispatch(&ctx_empty, "/api/audio/status", "GET", b"");
    assert_eq!(code2, 200);
    let v2: Value = serde_json::from_slice(&body2).unwrap();
    assert_eq!(v2["raw_processing"], false);
    assert!(v2["measured_roundtrip_ms"].is_null());
    assert!(v2["estimated_roundtrip_ms"].as_f64().is_some());
}

#[test]
fn test_ca_05_real_underruns_counted_on_empty_ring_pops() {
    let mut ctx = base_context();
    let duplex = Arc::new(RealtimeDuplexEngine::new(256));
    duplex.set_target_period(64);
    ctx.duplex_engine = Some(duplex.clone());

    let mut out = [0.0f32; 64];
    for _ in 0..3 {
        let popped = duplex.pop_samples_for_processing(&mut out);
        assert_eq!(popped, 0);
    }
    assert_eq!(duplex.underruns(), 3, "CA-5: 3 pops vazios devem incrementar underruns para 3");

    let (code, _ct, body) = dispatch(&ctx, "/api/audio/status", "GET", b"");
    assert_eq!(code, 200);
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["underruns"], 3, "CA-5: /api/audio/status deve refletir underruns reais");
}

#[test]
fn test_ca_06_input_channel_linear_selection_without_nonlinear_max_abs() {
    let n = 128usize;
    let mut stereo = vec![0.0f32; n * 2];
    let mut expected_left = vec![0.0f32; n];
    let mut expected_right = vec![0.0f32; n];
    for i in 0..n {
        let s = (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 48000.0).sin() * 0.5;
        stereo[i * 2] = s;
        stereo[i * 2 + 1] = -s;
        expected_left[i] = s;
        expected_right[i] = -s;
    }

    let telemetry = TelemetryBuffer::new(256);
    let mut mono_buf = vec![0.0f32; 256];

    // Sum: L + (-L) deve cancelar para < -90 dBFS
    let duplex_sum = RealtimeDuplexEngine::new(512);
    duplex_sum.set_input_channel(InputChannel::Sum);
    DesktopAudioHost::process_input_callback_f32(&stereo, 2, &mut mono_buf, &duplex_sum, &telemetry);
    let mut out_sum = vec![0.0f32; n];
    let popped = duplex_sum.pop_samples_raw(&mut out_sum);
    assert_eq!(popped, n);
    let peak_sum = out_sum.iter().fold(0.0f32, |m, &x| m.max(x.abs()));
    assert!(
        peak_sum < 3.16e-5,
        "CA-6: Soma L + (-L) deve cancelar abaixo de -90 dBFS, obtido {}",
        peak_sum
    );

    // Left: seno puro idêntico ao canal L (diferença < 0.1%)
    let duplex_l = RealtimeDuplexEngine::new(512);
    duplex_l.set_input_channel(InputChannel::Left);
    DesktopAudioHost::process_input_callback_f32(&stereo, 2, &mut mono_buf, &duplex_l, &telemetry);
    let mut out_l = vec![0.0f32; n];
    assert_eq!(duplex_l.pop_samples_raw(&mut out_l), n);
    for i in 0..n {
        assert!((out_l[i] - expected_left[i]).abs() < 1e-4, "CA-6: Left deve preservar L sem Distorção/max-abs");
    }

    // Right: seno invertido idêntico ao canal R
    let duplex_r = RealtimeDuplexEngine::new(512);
    duplex_r.set_input_channel(InputChannel::Right);
    DesktopAudioHost::process_input_callback_f32(&stereo, 2, &mut mono_buf, &duplex_r, &telemetry);
    let mut out_r = vec![0.0f32; n];
    assert_eq!(duplex_r.pop_samples_raw(&mut out_r), n);
    for i in 0..n {
        assert!((out_r[i] - expected_right[i]).abs() < 1e-4, "CA-6: Right deve preservar R exatamente");
    }
}

#[test]
fn test_ca_07_unavailable_exclusive_and_asio_modes_return_mode_unavailable() {
    let host = DesktopAudioHost::new();
    assert!(!host.has_mode_support(AudioMode::WasapiExclusive));
    assert!(!host.has_mode_support(AudioMode::Asio));

    let res_excl = host.validate_requested_mode("wasapi_exclusive");
    assert!(
        matches!(res_excl, Err(ref e) if e.contains("mode_unavailable")),
        "CA-7: Modo wasapi_exclusive deve recusar com mode_unavailable, obtido {:?}",
        res_excl
    );

    let res_asio = host.validate_requested_mode("asio");
    assert!(
        matches!(res_asio, Err(ref e) if e.contains("mode_unavailable")),
        "CA-7: Modo asio deve recusar com mode_unavailable, obtido {:?}",
        res_asio
    );
}

#[test]
fn test_ca_08_midi_enumerator_injection_and_cc_routing_via_param_table() {
    let midi_files = [
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/midi.rs"),
    ];
    for f in midi_files {
        let src = fs::read_to_string(f).unwrap();
        assert!(!src.contains("USB Floorboard MIDI In"), "CA-8: Porta MIDI hardcoded proibida");
        assert!(!src.contains("Expression Pedal Interface"), "CA-8: Porta MIDI hardcoded proibida");
    }

    let midi_host = DesktopMidiHost::with_enumerator(Arc::new(MockMidiEnumerator {
        ports: vec!["Morningstar MC6".to_string()],
    }));
    let ports = midi_host.enumerate_input_ports();
    assert_eq!(ports, vec!["Morningstar MC6".to_string()]);

    let ctx = base_context();
    let mut router = sonix_desktop::midi::MidiRouter::new(ctx.clone());
    // CC 7 = 64 -> amp_1.master = (64/127)*10.0 ~= 5.03937 (tolerância 0.05)
    let msg = router.on_message(&[0xB0, 0x07, 64]);
    assert_eq!(
        msg,
        sonix_desktop::midi::MidiMessageType::ControlChange {
            controller: 7,
            value: 64
        }
    );

    let master = ctx.app_state.lock().unwrap().amp_master;
    assert!(
        (master - 5.03937).abs() <= 0.05,
        "CA-8: CC 7=64 deve ajustar amp_1.master para ~5.04, obtido {}",
        master
    );
    let queued = ctx.param_queue.as_ref().unwrap().pop();
    assert!(queued.is_some(), "CA-8: Parâmetro MIDI validado deve ser enfileirado na LockFreeParamQueue do DSP");
}

#[test]
fn test_ca_09_empty_device_list_graceful_startup_and_test_riff_kept() {
    let mut ctx = base_context();
    ctx.device_enumerator = Arc::new(MockAudioEnumerator { devices: vec![] });

    let (code, _ct, body) = dispatch(&ctx, "/api/devices", "GET", b"");
    assert_eq!(code, 200);
    let parsed: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["devices"].as_array().unwrap().len(), 0);

    // test_riff continua selecionável e processável sem dispositivo físico
    let (p_code, _p_ct, p_body) = dispatch(&ctx, "/api/params", "POST", br#"{"input.source":"test_riff"}"#);
    assert_eq!(p_code, 200, "CA-9: test_riff deve permanecer ativo sem dispositivo físico: {}", String::from_utf8_lossy(&p_body));
    assert_eq!(ctx.app_state.lock().unwrap().input_source, "test_riff");
}

#[test]
fn test_ca_10_simulated_audio_host_non_regression() {
    let mut sim = SimulatedAudioHost::new(48000, 128);
    let out = sim.tick_sine_1khz();
    assert_eq!(out.len(), 128);
    let tele = sim.current_status_telemetry();
    assert_eq!(tele.sample_rate, 48000);
    assert_eq!(tele.input_period_frames, 128);
}
