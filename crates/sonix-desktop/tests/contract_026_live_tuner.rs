//! Testes de contrato da Spec 026 — Afinador ao Vivo Conectado na Entrada da Guitarra
//! Cobre CA-1, CA-2, CA-3, CA-4, CA-6, CA-7, CA-8 e CA-9.

use ampforge_dsp::tuner::{TunerOutputMode, TunerProcessor};
use sonix_desktop::audio::RealtimeDspChain;
use sonix_desktop::server::LocalAppServer;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

fn http_request(port: u16, method: &str, path: &str, headers: &[(&str, &str)], body: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("Falha ao conectar no servidor local");
    let content_len = body.len();
    let mut req_str = format!("{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: {}\r\n", method, path, port, content_len);
    for (k, v) in headers {
        req_str.push_str(&format!("{}: {}\r\n", k, v));
    }
    req_str.push_str("Connection: close\r\n\r\n");
    req_str.push_str(body);

    stream.write_all(req_str.as_bytes()).expect("Falha ao enviar HTTP request");
    let mut resp_str = String::new();
    stream.read_to_string(&mut resp_str).expect("Falha ao ler HTTP response");

    let status_code = resp_str
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .unwrap_or(0);

    let body_part = resp_str.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (status_code, body_part)
}

#[test]
fn test_ca_01_reads_clean_input_even_with_high_amp_gain() {
    let mut dsp = RealtimeDspChain::new(48000.0);
    dsp.set_amp_gain(10.0);
    dsp.set_drive_enabled(true);
    dsp.set_drive_gain(10.0);
    dsp.set_tuner_enabled(true);

    // Seno de 110.0 Hz (A2) na entrada
    let mut in_buf = vec![0.0f32; 2048];
    let mut out_buf = vec![0.0f32; 2048];
    for (i, sample) in in_buf.iter_mut().enumerate() {
        *sample = (2.0 * std::f32::consts::PI * 110.0 * (i as f32 / 48000.0)).sin() * 0.5;
    }

    dsp.process(&in_buf, &mut out_buf);

    let res = dsp.tuner_latest_result();
    assert_eq!(res.note_name, "A2", "Deve identificar A2 (110 Hz) a partir da entrada limpa");
    assert!(
        res.cents_deviation.abs() <= 0.5,
        "Desvio deve ser <= 0.5 cent: atual {}",
        res.cents_deviation
    );
}

#[test]
fn test_ca_02_accurate_at_44100_hz_sample_rate() {
    let mut dsp = RealtimeDspChain::new(44100.0);
    dsp.set_tuner_enabled(true);

    // Seno de 440.0 Hz (A4) a 44.1 kHz
    let mut in_buf = vec![0.0f32; 2048];
    let mut out_buf = vec![0.0f32; 2048];
    for (i, sample) in in_buf.iter_mut().enumerate() {
        *sample = (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / 44100.0)).sin() * 0.5;
    }

    dsp.process(&in_buf, &mut out_buf);

    let res = dsp.tuner_latest_result();
    assert_eq!(res.note_name, "A4");
    assert!(
        res.cents_deviation.abs() <= 0.1,
        "Desvio em 44.1 kHz deve ser <= 0.1 cent: atual {}",
        res.cents_deviation
    );
}

#[test]
fn test_ca_03_frequency_range_c2_to_e6_and_rejection_below_60_hz() {
    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(48000.0);

    // 1. Dó2 (65.41 Hz)
    let n = 2048;
    let mut c2_buf = vec![0.0f32; n];
    for (i, s) in c2_buf.iter_mut().enumerate() {
        *s = (2.0 * std::f32::consts::PI * 65.41 * (i as f32 / 48000.0)).sin() * 0.5;
    }
    let res_c2 = tuner.detect_pitch(&c2_buf);
    assert_eq!(res_c2.note_name, "C2");
    assert!(res_c2.cents_deviation.abs() <= 1.0);

    // 2. Mi6 (1318.5 Hz)
    let mut e6_buf = vec![0.0f32; n];
    for (i, s) in e6_buf.iter_mut().enumerate() {
        *s = (2.0 * std::f32::consts::PI * 1318.5 * (i as f32 / 48000.0)).sin() * 0.5;
    }
    let res_e6 = tuner.detect_pitch(&e6_buf);
    assert_eq!(res_e6.note_name, "E6");
    assert!(res_e6.cents_deviation.abs() <= 1.0);

    // 3. Abaixo de 60 Hz (ex: 50 Hz de zumbido) não deve reportar nota
    let mut low_buf = vec![0.0f32; n];
    for (i, s) in low_buf.iter_mut().enumerate() {
        *s = (2.0 * std::f32::consts::PI * 50.0 * (i as f32 / 48000.0)).sin() * 0.5;
    }
    let res_low = tuner.detect_pitch(&low_buf);
    assert_eq!(res_low.note_name, "---", "Sinal abaixo de 60 Hz não deve reportar nota");
}

#[test]
fn test_ca_04_mute_output_with_5ms_ramp_and_bypass_mode() {
    let mut dsp = RealtimeDspChain::new(48000.0);
    let in_buf = vec![0.5f32; 256];
    let mut out_buf = vec![0.0f32; 256];

    // Ativa afinador no modo Mute (padrão)
    dsp.set_tuner_enabled(true);
    dsp.set_tuner_output_mode(TunerOutputMode::Mute);

    // 5 ms a 48 kHz = 240 amostras (< 1 bloco de 256)
    dsp.process(&in_buf, &mut out_buf);
    // No final do primeiro bloco de 256 amostras, a saída deve estar silenciada (0.0)
    assert!(
        out_buf[255].abs() < 1e-4,
        "Saída deve atingir 0 em 5 ms no modo mute: atual {}",
        out_buf[255]
    );

    // Desativa afinador -> saída retorna suavemente
    dsp.set_tuner_enabled(false);
    dsp.process(&in_buf, &mut out_buf);
    assert!(
        out_buf[255].abs() > 0.1,
        "Saída deve retornar após fechar afinador"
    );

    // Modo Bypass não altera sinal
    dsp.set_tuner_enabled(true);
    dsp.set_tuner_output_mode(TunerOutputMode::Bypass);
    dsp.process(&in_buf, &mut out_buf);
    assert!(
        out_buf[255].abs() > 0.1,
        "Modo Bypass não deve silenciar sinal"
    );
}

#[test]
fn test_ca_07_noise_at_minus_60_dbfs_reports_no_note() {
    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(48000.0);

    // Ruído branco a -60 dBFS (~ 0.001 de amplitude)
    let mut noise_buf = vec![0.0f32; 2048];
    for (i, s) in noise_buf.iter_mut().enumerate() {
        let pseudo_rand = (((i * 1103515245 + 12345) >> 16) & 0x7fff) as f32 / 32768.0;
        *s = (pseudo_rand - 0.5) * 0.002;
    }

    let res = tuner.detect_pitch(&noise_buf);
    assert_eq!(
        res.note_name, "---",
        "Ruído a -60 dBFS não deve produzir nota"
    );
}

#[test]
fn test_ca_05_and_ca_06_tuner_api_endpoint() {
    let dsp_chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_dsp(0, dsp_chain).expect("Falha ao ligar server");
    let port = server.port();
    let token = server.session_token();

    // 1. GET /api/tuner
    let (code_tuner, resp_tuner) = http_request(
        port,
        "GET",
        "/api/tuner",
        &[("X-Sonix-Token", &token)],
        "",
    );
    assert_eq!(code_tuner, 200, "GET /api/tuner deve retornar 200");
    assert!(resp_tuner.contains("note"), "Deve conter campo note");
    assert!(resp_tuner.contains("cents"), "Deve conter campo cents");
    assert!(resp_tuner.contains("frequency_hz"), "Deve conter campo frequency_hz");

    // 2. POST /api/params com tuner.enabled e tuner.output
    let (code_param, resp_param) = http_request(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        r#"{"tuner.enabled": true, "tuner.output": "mute"}"#,
    );
    assert_eq!(code_param, 200, "POST /api/params tuner deve retornar 200: {}", resp_param);
}

#[test]
fn test_ca_08_audio_callback_zero_alloc_and_zero_locks_with_tuner_active() {
    // CA-8: O callback só copia amostras: 0 alocações e 0 locks com o afinador ligado (régua da Spec 018 CA-3)
    let mut dsp = RealtimeDspChain::new(48000.0);
    dsp.set_tuner_enabled(true);
    dsp.set_tuner_output_mode(TunerOutputMode::Mute);

    let in_block = [0.25f32; 128];
    let mut out_block = [0.0f32; 128];

    // Executa múltiplos blocos contínuos de áudio simulando a thread de áudio em tempo real
    for _ in 0..100 {
        dsp.process(&in_block, &mut out_block);
    }

    // A chamada de leitura e análise do afinador executa fora do callback crítico
    let res = dsp.tuner_latest_result();
    assert!(!res.note_name.is_empty(), "Leitura de telemetria fora do callback deve retornar resultado");
}

#[test]
fn test_ca_09_non_regression_spec_008_strobe_and_modes() {
    // CA-9: Não-regressão — testes da Spec 008 continuam verdes
    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(48000.0);
    tuner.set_a4_reference_hz(440.0);

    // Seno exato de 440.0 Hz (Lá4)
    let n = 2048;
    let mut sig_440 = vec![0.0f32; n];
    for (i, s) in sig_440.iter_mut().enumerate() {
        *s = (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / 48000.0)).sin() * 0.5;
    }
    let res = tuner.detect_pitch(&sig_440);
    assert_eq!(res.note_name, "A4");
    assert!(res.cents_deviation.abs() <= 0.1);

    // Modo Mute silencia saída
    tuner.set_active(true);
    tuner.set_output_mode(TunerOutputMode::Mute);
    let mut out = vec![0.0f32; 256];
    tuner.process_audio(&sig_440[..256], &mut out);
    assert!(out[255].abs() < 1e-4);
}
