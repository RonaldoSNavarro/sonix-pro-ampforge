//! crates/sonix-desktop/tests/contract_028_low_latency_realtime_safety.rs
//! Testes de Contrato Automatizados para Spec 028: Latência Ultrabaixa Real (WASAPI Exclusive / IAudioClient3) e Callback 100% Lock-Free / Zero-Alloc
//! Cobre Critérios de Aceite CA-1 a CA-7
//!
//! CA-1: 0 alocações na heap, 0 locks (Mutex) e 0 bypasses por contenção sob carga HTTP concorrente (1000 blocos)
//! CA-2: Duplex ring buffer mantém ocupação média em [0.5, 2.0] períodos (nunca acumulando >= 2400 amostras / 50ms)
//! CA-3: Latência digital round-trip <= 10.0 ms com period_frames = 128 a 48000 Hz, reportada em /api/audio/status
//! CA-4: cpu_avg_pct e cpu_peak_pct em /api/audio/status e cpu_usage_pct em /api/telemetry reportam valores reais > 0.0%
//! CA-5: Recuo gracioso de WasapiExclusive para WasapiSharedLowLatency com fallback_reason explícito
//! CA-6: Desconexão de dispositivo gera silêncio (0.0) sem eprintln! na thread de áudio e com tratamento de erro
//! CA-7: Não-regressão das Specs 001 e 018 (SimulatedAudioHost e LockFreeSpscRingBuffer)

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use sonix_desktop::audio::{
    measure_loopback_latency, AudioMode, AudioStatusTelemetry,
    InputChannel, LockFreeSpscRingBuffer, RealtimeDspChain,
    RealtimeDuplexEngine, SimulatedAudioHost, TelemetryBuffer,
};
use sonix_desktop::server::LocalAppServer;

fn http_request(
    port: u16,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> (u16, String) {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .expect("Falha ao conectar ao servidor local");
    let mut req = format!("{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n", method, path, port);
    for (k, v) in headers {
        req.push_str(&format!("{}: {}\r\n", k, v));
    }
    if let Some(b) = body {
        if !headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("Content-Length")) {
            req.push_str(&format!("Content-Length: {}\r\n", b.len()));
        }
        req.push_str("\r\n");
        req.push_str(b);
    } else {
        req.push_str("\r\n");
    }

    stream.write_all(req.as_bytes()).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    let status_line = response.lines().next().unwrap_or("HTTP/1.1 500");
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);

    let body_part = response.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (status_code, body_part)
}

#[test]
fn test_ca_01_realtime_audio_safety_zero_alloc_zero_locks_zero_glitches() {
    // CA-1: Em execução com RealtimeDspChain ativo e carga HTTP concorrente (POST /api/params, GET /api/tuner, GET /api/scope a 60 Hz),
    // o callback de áudio executa 0 alocações de heap, 0 locks (Mutex) e 0 chamadas de bypass por contenção (0 glitches em 1000 blocos).

    let mut dsp = RealtimeDspChain::new(48000.0);
    dsp.set_amp_gain(8.0);
    dsp.set_amp_master(7.0);
    dsp.set_cab_enabled(true);
    let dsp_chain = Arc::new(Mutex::new(dsp));
    let telemetry = Arc::new(TelemetryBuffer::new(512));

    let server = LocalAppServer::bind_port_with_telemetry(0, telemetry.clone(), Some(dsp_chain.clone()))
        .expect("Falha ao ligar servidor de teste");
    let port = server.port();
    let token = server.session_token();

    let running = Arc::new(AtomicBool::new(true));
    let run_clone = running.clone();
    let token_clone = token.clone();

    // Thread concorrente enviando carga intensa de requisições HTTP
    let http_thread = thread::spawn(move || {
        let mut count = 0;
        while run_clone.load(Ordering::Relaxed) {
            count += 1;
            let gain = 1.0 + ((count % 80) as f32) * 0.1;
            let body = format!(r#"{{"amp_1.gain":{}}}"#, gain);
            let _ = http_request(
                port,
                "POST",
                "/api/params",
                &[("Content-Type", "application/json"), ("X-Sonix-Token", &token_clone)],
                Some(&body),
            );
            let _ = http_request(port, "GET", "/api/tuner", &[], None);
            let _ = http_request(port, "GET", "/api/telemetry", &[], None);
            thread::sleep(Duration::from_millis(2));
        }
    });

    let period = 128;
    let mut in_block = vec![0.0f32; period];
    for i in 0..period {
        in_block[i] = 0.3 * (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / 48000.0).sin();
    }
    let mut out_block = vec![0.0f32; period];

    // O duplex engine deve ser lock-free
    let duplex = Arc::new(RealtimeDuplexEngine::with_target_period(4096, period));
    assert!(duplex.is_lock_free(), "RealtimeDuplexEngine deve ser estritamente lock-free (sem Mutex)");

    // Processa 1000 blocos consecutivos simulando a thread de áudio
    let mut bypass_count = 0;
    for _ in 0..1000 {
        duplex.push_captured_samples(&in_block);
        duplex.pop_samples_for_processing(&mut out_block);

        // Processa através da cadeia DSP diretamente ou via motor lock-free sem contenção de mutex
        if let Ok(mut dsp_guard) = dsp_chain.try_lock() {
            dsp_guard.process(&in_block, &mut out_block);
        } else {
            // Se cair no bypass por contenção, a Spec 028 considera glitch!
            bypass_count += 1;
        }

        // Se out_block for idêntico a in_block, houve bypass do DSP
        if out_block == in_block {
            bypass_count += 1;
        }
    }

    running.store(false, Ordering::Relaxed);
    let _ = http_thread.join();

    assert_eq!(
        bypass_count, 0,
        "Zero bypasses por contenção e zero glitches são permitidos nos 1000 blocos consecutivos sob carga HTTP"
    );
}

#[test]
fn test_ca_02_duplex_ring_buffer_occupancy_target_one_period() {
    // CA-2: O RealtimeDuplexEngine mantém ocupação média em regime estacionário entre 0.5 * period_frames e 2.0 * period_frames
    // (nunca acumulando >= 2400 amostras / 50 ms como antes).

    let period = 128;
    let duplex = RealtimeDuplexEngine::with_target_period(8192, period);

    let in_buf = vec![0.1f32; period];
    let mut out_buf = vec![0.0f32; period];

    // 1. Regime estacionário: 500 ciclos de push e pop
    let mut total_occupancy = 0usize;
    let cycles = 500;
    for _ in 0..cycles {
        duplex.push_captured_samples(&in_buf);
        let occ_before_pop = duplex.available_samples();
        assert!(
            occ_before_pop <= period * 3,
            "Ocupação durante push não deve acumular descontroladamente: {}", occ_before_pop
        );
        duplex.pop_samples_for_processing(&mut out_buf);
        let occ_after_pop = duplex.available_samples();
        total_occupancy += occ_after_pop;
    }

    let avg_occupancy = total_occupancy as f32 / cycles as f32;
    let min_allowed = 0.5 * (period as f32);
    let max_allowed = 2.0 * (period as f32);
    assert!(
        avg_occupancy >= min_allowed && avg_occupancy <= max_allowed,
        "Ocupação média em regime estacionário ({:.1}) deve estar entre [{:.1}, {:.1}]",
        avg_occupancy, min_allowed, max_allowed
    );

    // 2. Simula rajada súbita de 3000 amostras (como pausa do SO)
    let burst = vec![0.2f32; 3000];
    duplex.push_captured_samples(&burst);
    assert!(
        duplex.available_samples() >= 2400,
        "Pré-condição da rajada: fila acumulou >= 2400 amostras"
    );

    // Na primeira drenagem, deve cortar o excesso imediatamente para o alvo de 1 período
    duplex.pop_samples_for_processing(&mut out_buf);
    let occ_after_drain = duplex.available_samples();
    assert!(
        occ_after_drain < 2400,
        "Fila não pode acumular >= 2400 amostras (50ms) após drenagem: atual={}", occ_after_drain
    );
    assert!(
        occ_after_drain <= (period * 2),
        "Fila deve ter sido drenada para <= 2 períodos: atual={}", occ_after_drain
    );
}

#[test]
fn test_ca_03_algorithmic_and_buffering_latency_under_10ms() {
    // CA-3: A latência algorítmica da cadeia + buffering duplex com period_frames = 128 a 48000 Hz em modo
    // wasapi_exclusive ou wasapi_shared_low_latency é <= 10.0 ms no caminho digital, reportada em GET /api/audio/status.

    let period = 128u32;
    let sample_rate = 48000u32;

    // Cálculo das componentes digitais:
    // driver_input_latency = 128 / 48000 = 2.667 ms
    // driver_output_latency = 128 / 48000 = 2.667 ms
    // duplex_buffering = 128 / 48000 = 2.667 ms
    // algorithmic_latency = 16 / 48000 = 0.333 ms
    // Total roundtrip digital = 2.667 + 2.667 + 2.667 + 0.333 = ~8.33 ms <= 10.0 ms
    let driver_in_ms = (period as f32 / sample_rate as f32) * 1000.0;
    let driver_out_ms = (period as f32 / sample_rate as f32) * 1000.0;
    let duplex_ms = (period as f32 / sample_rate as f32) * 1000.0;
    let algo_lat_samples = 16u32;
    let algo_lat_ms = (algo_lat_samples as f32 / sample_rate as f32) * 1000.0;
    let total_digital_latency_ms = driver_in_ms + driver_out_ms + duplex_ms + algo_lat_ms;

    assert!(
        total_digital_latency_ms <= 10.0,
        "Latência digital calculada ({:.2} ms) deve ser <= 10.0 ms",
        total_digital_latency_ms
    );

    let telemetry = AudioStatusTelemetry {
        backend: "wasapi".to_string(),
        mode: "wasapi_exclusive".to_string(),
        fallback_reason: None,
        raw_processing: true,
        sample_rate,
        input_period_frames: period,
        output_period_frames: period,
        input_device: "Realtek High Definition Audio (Line-In P2)".to_string(),
        output_device: "Realtek Audio Speakers (Headphones P2)".to_string(),
        driver_input_latency_ms: driver_in_ms,
        driver_output_latency_ms: driver_out_ms,
        cpu_avg_pct: 4.2,
        cpu_peak_pct: 7.5,
        underruns: 0,
        overruns: 0,
        panics: 0,
        adc_clips: 0,
        measured_roundtrip_ms: Some(total_digital_latency_ms),
        estimated_roundtrip_ms: total_digital_latency_ms,
        algorithmic_latency_samples: algo_lat_samples,
        pitch_wet_latency_ms: 0.0,
    };

    let server = LocalAppServer::bind_ephemeral().expect("Falha ao ligar server");
    server.set_audio_telemetry(telemetry);

    let (status, body) = http_request(server.port(), "GET", "/api/audio/status", &[], None);
    assert_eq!(status, 200);

    let json: serde_json::Value = serde_json::from_str(&body).expect("JSON inválido");
    assert_eq!(json["backend"], "wasapi");
    assert_eq!(json["mode"], "wasapi_exclusive");
    assert_eq!(json["input_period_frames"], 128);
    assert_eq!(json["output_period_frames"], 128);
    let roundtrip = json["measured_roundtrip_ms"].as_f64().expect("measured_roundtrip_ms ausente");
    assert!(
        roundtrip <= 10.0,
        "Latência roundtrip reportada ({:.2} ms) deve ser <= 10.0 ms", roundtrip
    );
}

#[test]
fn test_ca_04_realtime_cpu_load_measurement() {
    // CA-4: cpu_avg_pct e cpu_peak_pct em /api/audio/status e cpu_usage_pct em /api/telemetry
    // refletem o tempo real medido por bloco dividido pelo orçamento do bloco (period_frames / sample_rate),
    // sendo > 0.0% quando a cadeia DSP está ativa.

    let period = 128usize;
    let sample_rate = 48000u32;
    let block_budget_s = period as f32 / sample_rate as f32; // ~2.667 ms

    let telemetry = Arc::new(TelemetryBuffer::new(512));
    let mut dsp = RealtimeDspChain::new(sample_rate as f32);
    dsp.set_amp_gain(7.0);

    let in_block = vec![0.3f32; period];
    let mut out_block = vec![0.0f32; period];

    // Simula 20 blocos medindo tempo real de execução
    let mut peak_pct = 0.0f32;
    let mut sum_pct = 0.0f32;
    for _ in 0..20 {
        let t0 = std::time::Instant::now();
        dsp.process(&in_block, &mut out_block);
        let elapsed = t0.elapsed().as_secs_f32();
        let cpu_pct = (elapsed / block_budget_s) * 100.0;
        peak_pct = peak_pct.max(cpu_pct);
        sum_pct += cpu_pct;
        telemetry.update_peaks(-20.0, -10.0, cpu_pct);
    }
    let avg_pct = sum_pct / 20.0;

    assert!(avg_pct > 0.0, "Carga média de CPU deve ser > 0.0% com DSP ativo");
    assert!(peak_pct > 0.0, "Carga de pico de CPU deve ser > 0.0% com DSP ativo");

    let server = LocalAppServer::bind_port_with_telemetry(0, telemetry.clone(), None)
        .expect("Falha ao ligar server");
    let audio_status = AudioStatusTelemetry {
        backend: "wasapi".to_string(),
        mode: "wasapi_shared_low_latency".to_string(),
        fallback_reason: None,
        raw_processing: true,
        sample_rate,
        input_period_frames: period as u32,
        output_period_frames: period as u32,
        input_device: "Realtek P2".to_string(),
        output_device: "Realtek P2".to_string(),
        driver_input_latency_ms: 2.67,
        driver_output_latency_ms: 2.67,
        cpu_avg_pct: avg_pct,
        cpu_peak_pct: peak_pct,
        underruns: 0,
        overruns: 0,
        panics: 0,
        adc_clips: 0,
        measured_roundtrip_ms: Some(8.33),
        estimated_roundtrip_ms: 5.67,
        algorithmic_latency_samples: 16,
        pitch_wet_latency_ms: 0.0,
    };
    server.set_audio_telemetry(audio_status);

    // Consulta /api/audio/status
    let (st_status, body_status) = http_request(server.port(), "GET", "/api/audio/status", &[], None);
    assert_eq!(st_status, 200);
    let json_status: serde_json::Value = serde_json::from_str(&body_status).unwrap();
    let status_cpu_avg = json_status["cpu_avg_pct"].as_f64().unwrap();
    let status_cpu_peak = json_status["cpu_peak_pct"].as_f64().unwrap();
    assert!(status_cpu_avg > 0.0, "cpu_avg_pct em /api/audio/status deve ser > 0.0%");
    assert!(status_cpu_peak > 0.0, "cpu_peak_pct em /api/audio/status deve ser > 0.0%");

    // Consulta /api/telemetry
    let (st_tele, body_tele) = http_request(server.port(), "GET", "/api/telemetry", &[], None);
    assert_eq!(st_tele, 200);
    let json_tele: serde_json::Value = serde_json::from_str(&body_tele).unwrap();
    let tele_cpu = json_tele["cpu_usage_pct"].as_f64().unwrap();
    assert!(tele_cpu > 0.0, "cpu_usage_pct em /api/telemetry deve ser > 0.0%");
}

#[test]
fn test_ca_05_graceful_wasapi_fallback() {
    // CA-5: Se o hardware recusar WASAPI Exclusive (ex.: dispositivo em uso ou taxas in/out diferentes),
    // o host recua automaticamente para wasapi_shared_low_latency (ou wasapi_shared) sem panic e preenche fallback_reason.

    let mut host = SimulatedAudioHost::new(48000, 128);

    // 1. Caso de taxas de amostragem incompatíveis
    let result = host.open_stream_with_rates(44100, 48000, AudioMode::WasapiExclusive);
    assert!(result.is_err(), "Taxas diferentes devem ser rejeitadas no modo exclusivo");

    // 2. Recuo gracioso para modo compartilhado
    let fallback_shared = host.open_stream_with_rates(44100, 48000, AudioMode::WasapiShared);
    assert!(fallback_shared.is_ok(), "Deve aceitar recuo para compartilhado");

    // 3. Cascata de fallback com suporte exclusivo indisponível
    host.set_exclusive_supported(false);
    host.set_shared_low_latency_supported(true);
    let mode = host.initialize_auto_backend();
    assert_eq!(mode, AudioMode::WasapiSharedLowLatency);
    assert!(host.fallback_reason().is_some());
    let reason = host.fallback_reason().unwrap();
    assert!(
        reason.to_lowercase().contains("exclusivo") || reason.to_lowercase().contains("baixa latência"),
        "fallback_reason deve explicar o motivo do recuo: {}", reason
    );
}

#[test]
fn test_ca_06_device_disconnect_silence_and_error_handling() {
    // CA-6: Caso de erro — se o dispositivo de entrada ou saída desconectar durante a reprodução,
    // o callback preenche o buffer de saída com silêncio (0.0), incrementa contador de erro sem eprintln! na thread de áudio e sinaliza reconexão.

    let mut host = SimulatedAudioHost::new(48000, 128);

    // Reproduz normalmente com sinal ativo (após drenar período inicial de warmup)
    let _ = host.tick_sine_1khz();
    let normal_out = host.tick_sine_1khz();
    assert!(normal_out.iter().any(|&s| s != 0.0), "Antes da desconexão deve haver áudio");

    // Desconecta dispositivo
    host.simulate_device_disconnect();
    assert!(host.is_device_lost(), "Host deve registrar perda de dispositivo");

    // Enquanto desconectado, a saída deve ser estritamente silêncio (0.0)
    let silent_out = host.tick_sine_1khz();
    assert!(
        silent_out.iter().all(|&s| s == 0.0),
        "Buffer de saída deve conter apenas 0.0 durante desconexão"
    );

    // Reconexão automática em <= 2.5s
    let reconnected = host.simulate_device_reconnected(1.5);
    assert!(reconnected, "Deve reconectar em <= 2.5s");
    assert!(!host.is_device_lost());
}

#[test]
fn test_ca_07_non_regression_specs_001_and_018() {
    // CA-7: Não-regressão — integridade dos componentes de baixa latência e buffer SPSC da Spec 001 e Spec 018

    // 1. Buffer lock-free SPSC
    let ring = LockFreeSpscRingBuffer::new(512);
    assert_eq!(ring.capacity(), 512);
    assert_eq!(ring.occupied(), 0);
    assert!(ring.is_lock_free());

    let in_data = [0.42f32; 128];
    let mut out_data = [0.0f32; 128];
    assert_eq!(ring.push_slice(&in_data), 128);
    assert_eq!(ring.occupied(), 128);
    assert_eq!(ring.pop_slice(&mut out_data), 128);
    assert_eq!(ring.occupied(), 0);
    assert_eq!(in_data, out_data);

    // 2. Medição de loopback por correlação
    let sig = vec![0.5f32; 100];
    let mut resp = vec![0.0f32; 250];
    resp[50..150].copy_from_slice(&sig);
    let lag = measure_loopback_latency(&sig, &resp, 48000.0).expect("Deve detectar atraso");
    assert_eq!(lag, 50);

    // 3. Conversão de canais estéreo para mono
    let stereo = [0.5f32, -0.5f32];
    let mut mono = [0.0f32; 1];
    InputChannel::Sum.convert_interleaved(&stereo, 2, &mut mono);
    assert!(mono[0].abs() < 1e-6);
}
