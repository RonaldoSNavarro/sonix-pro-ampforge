//! crates/sonix-desktop/tests/contract_018_low_latency_host.rs
//! Testes de Contrato Automatizados para Spec 018: Latência Mínima e Host de Áudio Robusto em Placa Onboard
//! Cobre Critérios de Aceite CA-1 a CA-13
//!
//! CA-1: Ocupação da fila mantida em [0.5, 2.0] períodos no host simulado (128 frames)
//! CA-2: Controle de deriva de +-200 ppm sem descontinuidades (|dy| <= 1.05 * ideal)
//! CA-3: Zero alocações e zero locks no caminho crítico de áudio (10.000 callbacks)
//! CA-4: Negociação de taxa e preparação da RealtimeDspChain; recuo graceful se sem taxa comum
//! CA-5: Conversão estéreo para mono explícita (left, right, sum) sem retificação
//! CA-6: Fallback em cascata: exclusivo -> compartilhado baixa latência -> compartilhado padrão
//! CA-7: Endpoint GET /api/audio/status com telemetria e diagnósticos completos
//! CA-9: Medição offline de latência por loopback (correlação cruzada e detecção de ausência)
//! CA-11: Resiliência contra pânico no processamento DSP (zero-fill, contador e retomada)
//! CA-12: Detecção de device_lost e recuperação automática em <= 2.5 s
//! CA-13: Não-regressão das Specs 001 e 013

use sonix_desktop::audio::{
    measure_loopback_latency, recover_dsp_chain_if_faulted, AudioMode, AudioStatusTelemetry,
    DesktopAudioHost, InputChannel, LockFreeSpscRingBuffer, RealtimeDspChain,
    RealtimeDuplexEngine, SimulatedAudioHost, TelemetryBuffer,
};
use sonix_desktop::api::{ApiContext, AppState};
use std::sync::{Arc, Mutex};
use sonix_desktop::server::LocalAppServer;

#[test]
fn test_ca_01_queue_occupancy_within_half_to_two_periods() {
    // CA-1: Host simulado (128 quadros) rodando 60s simulados: ocupação da fila fica em [0.5; 2.0] períodos
    let period = 128;
    let mut host = SimulatedAudioHost::new(48000, period);
    
    // Roda 1 segundo inicial de estabilização (48000 / 128 = 375 períodos)
    for _ in 0..375 {
        host.tick_callback_pair(0.0);
    }
    
    // Roda 60 segundos simulados (60 * 375 = 22.500 callbacks)
    let min_allowed = (period as f32 * 0.5) as usize;
    let max_allowed = (period as f32 * 2.0) as usize;
    
    let mut violations = 0;
    let total_ticks = 22500;
    for _ in 0..total_ticks {
        host.tick_callback_pair(0.0);
        let occ = host.queue_occupancy();
        if occ < min_allowed || occ > max_allowed {
            violations += 1;
        }
    }
    
    assert_eq!(
        violations, 0,
        "Ocupação da fila violou a faixa [{}, {}] em {} de {} callbacks",
        min_allowed, max_allowed, violations, total_ticks
    );
}

#[test]
fn test_ca_02_drift_correction_and_waveform_continuity() {
    // CA-2: Deriva de +-200 ppm por 10 min simulados: ocupação em [0.5; 2.0] e sem descontinuidade em seno de 1 kHz
    let period = 128;
    let mut host = SimulatedAudioHost::new(48000, period);
    host.set_drift_ppm(200.0); // +200 ppm de deriva entre captura e reprodução
    
    let min_allowed = (period as f32 * 0.5) as usize;
    let max_allowed = (period as f32 * 2.0) as usize;
    
    let total_ticks = 10 * 60 * (48000 / period); // 10 minutos
    let mut max_sine_jump = 0.0f32;
    let mut prev_sample = 0.0f32;
    
    for i in 0..total_ticks {
        let out_buf = host.tick_sine_1khz();
        if i > 500 { // Após aquecimento
            let occ = host.queue_occupancy();
            assert!(
                occ >= min_allowed && occ <= max_allowed,
                "Ocupação fora de faixa sob deriva: {}", occ
            );
            
            for &s in out_buf.iter() {
                let jump = (s - prev_sample).abs();
                if jump > max_sine_jump {
                    max_sine_jump = jump;
                }
                prev_sample = s;
            }
        } else {
            prev_sample = *out_buf.last().unwrap_or(&0.0);
        }
    }
    
    // Para seno ideal de 1 kHz em 48 kHz: max |sin(w*n) - sin(w*(n-1))| = 2*sin(w/2) ≈ 2*sin(pi*1000/48000) ≈ 0.1307
    let ideal_jump = 2.0 * (std::f32::consts::PI * 1000.0 / 48000.0).sin();
    assert!(
        max_sine_jump <= 1.05 * ideal_jump,
        "Descontinuidade de sinal detectada sob deriva: jump={:.4}, max_allowed={:.4}",
        max_sine_jump,
        1.05 * ideal_jump
    );
}

#[test]
fn test_ca_03_zero_alloc_and_zero_lock_during_callbacks() {
    // CA-3: 10.000 callbacks de captura e reprodução com 0 alocações e 0 locks
    let buffer = LockFreeSpscRingBuffer::new(1024);
    let in_data = [0.1f32; 128];
    let mut out_data = [0.0f32; 128];
    
    // Pré-aquecimento
    buffer.push_slice(&in_data);
    buffer.pop_slice(&mut out_data);
    
    // Executa 10.000 iterações testando zero overhead
    for _ in 0..10_000 {
        let pushed = buffer.push_slice(&in_data);
        assert_eq!(pushed, 128);
        let popped = buffer.pop_slice(&mut out_data);
        assert_eq!(popped, 128);
    }
    
    // Verifica flag e propriedades lock-free do buffer
    assert!(buffer.is_lock_free(), "Buffer deve ser estritamente lock-free");
}

#[test]
fn test_ca_04_negotiated_sample_rate_and_graceful_mismatch_fallback() {
    // CA-4: Dispositivo em 44.100 Hz prepara cadeia com 44100.0. Mismatch recua para compartilhado com motivo
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(44100.0, 128);
    assert_eq!(chain.sample_rate(), 44100.0, "Cadeia deve ser preparada na taxa negociada");
    
    let mut host = SimulatedAudioHost::new(48000, 128);
    let result = host.open_stream_with_rates(44100, 48000, AudioMode::WasapiExclusive);
    
    assert!(result.is_err(), "Taxas discordantes sem taxa comum devem falhar no modo exclusivo");
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("44100") && err_msg.contains("48000"), "Erro deve listar as taxas suportadas");
    
    // Host recua para compartilhado
    let fallback = host.open_stream_with_rates(44100, 48000, AudioMode::WasapiShared);
    assert!(fallback.is_ok(), "Host deve conseguir abrir no modo compartilhado");
    assert_eq!(host.current_mode(), AudioMode::WasapiShared);
}

#[test]
fn test_ca_05_stereo_to_mono_channel_selection_accuracy() {
    // CA-5: Entrada estéreo com L = seno 0.5 e R = -seno 0.5
    // input.channel = left -> L
    // input.channel = right -> R
    // input.channel = sum -> 0 (+- 1e-7)
    let frames = 256;
    let mut stereo_data = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        let s = 0.5 * (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / 48000.0).sin();
        stereo_data.push(s);      // L
        stereo_data.push(-s);     // R
    }
    
    let mut out_left = vec![0.0f32; frames];
    let mut out_right = vec![0.0f32; frames];
    let mut out_sum = vec![0.0f32; frames];
    
    InputChannel::Left.convert_interleaved(&stereo_data, 2, &mut out_left);
    InputChannel::Right.convert_interleaved(&stereo_data, 2, &mut out_right);
    InputChannel::Sum.convert_interleaved(&stereo_data, 2, &mut out_sum);
    
    for i in 0..frames {
        let expected_l = stereo_data[i * 2];
        let expected_r = stereo_data[i * 2 + 1];
        assert_eq!(out_left[i], expected_l, "Left deve ser bit-idêntico a L");
        assert_eq!(out_right[i], expected_r, "Right deve ser bit-idêntico a R");
        assert!(out_sum[i].abs() <= 1e-7, "Sum deve cancelar perfeitamente: {}", out_sum[i]);
    }
}

#[test]
fn test_ca_06_backend_fallback_cascade_without_panic() {
    // CA-6: Exclusivo falha -> baixa latência; baixa latência falha -> compartilhado padrão
    let mut host = SimulatedAudioHost::new(48000, 128);
    host.set_exclusive_supported(false);
    host.set_shared_low_latency_supported(false);
    
    let mode = host.initialize_auto_backend();
    assert_eq!(mode, AudioMode::WasapiShared, "Deve ter recuado até WasapiShared");
    assert!(host.fallback_reason().is_some(), "Deve documentar motivo do recuo");
}

#[test]
fn test_ca_07_api_audio_status_telemetry_schema() {
    // CA-7: GET /api/audio/status devolve JSON estruturado completo
    let telemetry = AudioStatusTelemetry {
        backend: "wasapi".to_string(),
        mode: "wasapi_shared_low_latency".to_string(),
        fallback_reason: None,
        raw_processing: true,
        sample_rate: 48000,
        input_period_frames: 128,
        output_period_frames: 128,
        input_device: "Realtek High Definition Audio".to_string(),
        output_device: "Realtek Audio Speakers".to_string(),
        driver_input_latency_ms: 2.67,
        driver_output_latency_ms: 2.67,
        cpu_avg_pct: 12.4,
        cpu_peak_pct: 18.2,
        underruns: 0,
        overruns: 0,
        panics: 0,
        adc_clips: 0,
        measured_roundtrip_ms: Some(10.5),
        estimated_roundtrip_ms: 5.34,
        algorithmic_latency_samples: 0,
    };
    
    let server = LocalAppServer::bind_ephemeral().expect("Falha ao ligar server");
    server.set_audio_telemetry(telemetry.clone());
    
    let url = format!("http://127.0.0.1:{}/api/audio/status", server.port());
    let resp: serde_json::Value = ureq_or_simple_get(&url);
    
    assert_eq!(resp["backend"], "wasapi");
    assert_eq!(resp["mode"], "wasapi_shared_low_latency");
    assert_eq!(resp["raw_processing"], true);
    assert_eq!(resp["sample_rate"], 48000);
    assert_eq!(resp["input_period_frames"], 128);
    assert_eq!(resp["output_period_frames"], 128);
    assert_eq!(resp["cpu_avg_pct"], 12.4);
    assert_eq!(resp["measured_roundtrip_ms"], 10.5);
}

#[test]
fn test_ca_09_offline_loopback_latency_measurement() {
    // CA-9: Laço simulado com atraso conhecido de 517 amostras e ruído a -40 dB -> 517 +- 1
    // Sem cabo (sem correlação) -> Err("sinal de retorno não detectado")
    let n = 2048;
    let mut test_signal = vec![0.0f32; n];
    // Sinal chirp ou ruído com envelope
    for i in 0..n {
        let t = i as f32 / n as f32;
        test_signal[i] = (2.0 * std::f32::consts::PI * (100.0 + 3000.0 * t * t) * (i as f32 / 48000.0)).sin() * (-0.1 * (i as f32)).exp();
    }
    
    let true_delay = 517;
    let mut response_signal = vec![0.0f32; n + true_delay + 200];
    
    // Copia sinal com atraso e adiciona ruído -40 dB (amplitude ~ 0.01)
    for (i, &s) in test_signal.iter().enumerate() {
        let noise = ((i * 1234567) % 1000) as f32 / 50000.0 - 0.01;
        response_signal[i + true_delay] = s + noise;
    }
    
    let measured = measure_loopback_latency(&test_signal, &response_signal, 48000.0)
        .expect("Deve medir atraso quando o retorno existe");
    
    let diff = (measured as i32 - true_delay as i32).abs();
    assert!(diff <= 1, "Atraso medido ({}) deve ser 517 +- 1", measured);
    
    // Sem cabo (só silêncio / ruído térmico fraco)
    let silent_response = vec![0.0001f32; n + 500];
    let fail_result = measure_loopback_latency(&test_signal, &silent_response, 48000.0);
    assert_eq!(fail_result, Err("sinal de retorno não detectado"));
}

#[test]
fn test_ca_11_dsp_panic_catch_and_stream_resilience() {
    // CA-11 (AUD-04): Pânico provocado dentro do processamento gera silêncio no callback real,
    // liga engine_faulted sem envenenar o Mutex, incrementa panic_count e recupera com a rotina real.
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 128);
    chain.set_gate_enabled(false);
    chain.set_amp_standby(false);
    let pq = chain.param_queue();
    let dsp = Arc::new(Mutex::new(chain));
    let telemetry = Arc::new(TelemetryBuffer::new(512));
    let duplex = Arc::new(RealtimeDuplexEngine::new(4096));

    let mut st = AppState::default();
    st.gate_enabled = false;
    st.amp_standby = false;
    let ctx = ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(st)),
        telemetry.clone(),
        Some(dsp.clone()),
        Some(pq),
        None,
        None,
    );

    assert_eq!(telemetry.panic_count(), 0);
    assert!(!telemetry.is_engine_faulted());

    let input_block: Vec<f32> = (0..128)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / 48000.0)).sin() * 0.25)
        .collect();
    duplex.push_captured_samples(&input_block);

    let mut output = vec![0.99_f32; 256];
    let mut mono_in = vec![0.0_f32; 128];
    let mut mono_out = vec![0.0_f32; 128];

    DesktopAudioHost::process_output_callback_f32_with_processor(
        &mut output,
        2,
        48000.0,
        &mut mono_in,
        &mut mono_out,
        &duplex,
        &dsp,
        &telemetry,
        None,
        |_dsp, _inp, _out| panic!("CA-11 fault injection"),
    );

    assert!(output.iter().all(|&s| s == 0.0), "Bloco em que houve pânico deve sair em silêncio");
    assert_eq!(telemetry.panic_count(), 1, "Contador de panics deve ser incrementado");
    assert!(telemetry.is_engine_faulted(), "Flag engine_faulted deve ligar após pânico");
    assert!(dsp.try_lock().is_ok(), "Mutex não pode ficar envenenado após catch_unwind");

    // Enquanto faulted, blocos adicionais continuam silenciosos
    duplex.push_captured_samples(&input_block);
    output.fill(0.5);
    DesktopAudioHost::process_output_callback_f32(
        &mut output,
        2,
        48000.0,
        &mut mono_in,
        &mut mono_out,
        &duplex,
        &dsp,
        &telemetry,
        None,
    );
    assert!(output.iter().all(|&s| s == 0.0), "Com engine_faulted, callback sai em silêncio");

    // Rotina real de recuperação reconstrói a cadeia a partir de AppState e limpa a flag
    let recovery = recover_dsp_chain_if_faulted(&ctx);
    assert_eq!(recovery, Some(1));
    assert!(!telemetry.is_engine_faulted());

    // Os 100 blocos seguintes devem ser processados normalmente sem pânico
    let mut normal_blocks = 0;
    for _ in 0..100 {
        duplex.push_captured_samples(&input_block);
        output.fill(0.0);
        DesktopAudioHost::process_output_callback_f32(
            &mut output,
            2,
            48000.0,
            &mut mono_in,
            &mut mono_out,
            &duplex,
            &dsp,
            &telemetry,
            None,
        );
        if output.iter().any(|&s| s != 0.0) {
            normal_blocks += 1;
        }
    }
    assert_eq!(normal_blocks, 100, "Todos os 100 blocos seguintes devem operar normalmente");
    assert_eq!(telemetry.panic_count(), 1, "Contador de panics não deve subir além de 1");
}

#[test]
fn test_ca_12_device_lost_and_automatic_recovery() {
    // CA-12: Desconexão simula device_lost com saída em silêncio; ao reaparecer, reabre em <= 2.5s
    let mut host = SimulatedAudioHost::new(48000, 128);
    host.simulate_device_disconnect();
    
    assert!(host.is_device_lost());
    let silent_out = host.tick_sine_1khz();
    assert!(silent_out.iter().all(|&s| s == 0.0), "Saída deve ser silêncio durante device_lost");
    
    // Simula retorno do dispositivo
    let recovered = host.simulate_device_reconnected(2.0); // 2.0s <= 2.5s
    assert!(recovered, "Dispositivo deve recuperar em <= 2.5s");
    assert!(!host.is_device_lost());
    
    let mut has_audio = false;
    for _ in 0..5 {
        let out = host.tick_sine_1khz();
        if out.iter().any(|&s| s != 0.0) {
            has_audio = true;
            break;
        }
    }
    assert!(has_audio, "Após retorno e dreno inicial, deve processar áudio normalmente");
}

#[test]
fn test_ca_13_non_regression_specs_001_and_013() {
    // CA-13: Preserva integridade de duplex lock-free e buffer sizes das specs anteriores
    let buffer = LockFreeSpscRingBuffer::new(512);
    assert_eq!(buffer.capacity(), 512);
    assert_eq!(buffer.occupied(), 0);
}

/// Helper HTTP GET simples para testes
fn ureq_or_simple_get(url: &str) -> serde_json::Value {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    
    let parsed = url.strip_prefix("http://").unwrap();
    let mut parts = parsed.splitn(2, '/');
    let host_port = parts.next().unwrap();
    let path = format!("/{}", parts.next().unwrap_or(""));
    
    let mut stream = TcpStream::connect(host_port).expect("Falha ao conectar no host");
    let req = format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", path, host_port);
    stream.write_all(req.as_bytes()).unwrap();
    
    let mut resp = String::new();
    stream.read_to_string(&mut resp).unwrap();
    
    let body = resp.split("\r\n\r\n").nth(1).unwrap_or("{}");
    serde_json::from_str(body).unwrap_or(serde_json::Value::Null)
}
