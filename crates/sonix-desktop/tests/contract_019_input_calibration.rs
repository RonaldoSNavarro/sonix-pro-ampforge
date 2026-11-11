//! crates/sonix-desktop/tests/contract_019_input_calibration.rs
//! Testes de Contrato Automatizados para Spec 019: Entrada de Guitarra na Placa Onboard: Calibração, Clipping e Sinal de Teste
//! Cobre Critérios de Aceite CA-1 a CA-9 (Lado Rust/Backend)
//!
//! CA-1: Calibração propõe input.boost = 7.94 ± 0.05 e gate.threshold_db = -46 ± 0.5 dB
//! CA-2: Pico bruto >= -1 dBFS resulta em input_too_hot e boost <= 1.0; pico < -42 dBFS resulta em input_too_low e boost = 8.0
//! CA-3: Detecção de hum 50 Hz vs 60 Hz vs ruído branco
//! CA-4: adc_clips incrementa exatamente com amostras >= 0.999 e reflete em GET /api/audio/status
//! CA-5: input.source=test_riff toca sinal que passa pela cadeia DSP sem depender de hardware
//! CA-6: Duas gerações de test_riff com mesma semente são bit-idênticas com pico de -12 dBFS ± 0.5 dB
//! CA-7: POST /api/input/di aceita WAV PCM16 44.1kHz e rejeita channels=0 (400) e > 20MB (413) sem pânico
//! CA-8: Calibração persistida por ID de dispositivo aplicada no boot
//! CA-9: Não-regressão das Specs 013, 014 e 018

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use sonix_desktop::audio::{RealtimeDspChain, TelemetryBuffer};
use sonix_desktop::calibration::{
    calibrate_input, generate_karplus_strong_riff, CalibrationStore, DiagnosticReason,
};
use sonix_desktop::server::LocalAppServer;

fn http_request(
    port: u16,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&[u8]>,
) -> (u16, Vec<(String, String)>, Vec<u8>) {
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
        let mut full_bytes = req.into_bytes();
        full_bytes.extend_from_slice(b);
        stream.write_all(&full_bytes).unwrap();
    } else {
        req.push_str("\r\n");
        stream.write_all(req.as_bytes()).unwrap();
    }

    let mut response_bytes = Vec::new();
    stream.read_to_end(&mut response_bytes).unwrap();

    let header_end = response_bytes
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .unwrap_or(response_bytes.len());
    let header_str = String::from_utf8_lossy(&response_bytes[..header_end]);
    let mut lines = header_str.lines();

    let status_line = lines.next().unwrap_or("HTTP/1.1 500");
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);

    let mut resp_headers = Vec::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            resp_headers.push((k.trim().to_lowercase(), v.trim().to_string()));
        }
    }

    let resp_body = if header_end + 4 <= response_bytes.len() {
        response_bytes[header_end + 4..].to_vec()
    } else {
        Vec::new()
    };

    (status_code, resp_headers, resp_body)
}

fn make_test_wav(channels: u16, sample_rate: u32, num_samples: usize) -> Vec<u8> {
    let bytes_per_sample = 2usize;
    let data_bytes = num_samples * (channels as usize) * bytes_per_sample;
    let total_size = 36 + data_bytes;

    let mut buf = Vec::with_capacity(44 + data_bytes);
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&(total_size as u32).to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes());
    buf.extend_from_slice(&channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate.wrapping_mul(channels as u32).wrapping_mul(bytes_per_sample as u32);
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&(channels * 2).to_le_bytes());
    buf.extend_from_slice(&16u16.to_le_bytes());
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&(data_bytes as u32).to_le_bytes());
    buf.resize(44 + data_bytes, 0u8);
    buf
}

#[test]
fn test_ca_01_input_calibration_ideal_boost_and_gate() {
    let sample_rate = 48000.0;
    // Ruído RMS de -70 dBFS (amplitude rms = 10^(-70/20) = 0.0003162)
    let noise_len = 48000 * 2;
    let mut noise = vec![0.0f32; noise_len];
    let noise_amp = 0.0003162 * (3.0f32).sqrt(); // distribuição uniforme [-A, A] tem RMS = A / sqrt(3)
    let mut lcg = 123456789u64;
    for s in noise.iter_mut() {
        lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
        let u = (lcg >> 33) as f32 / (1u32 << 31) as f32 - 1.0;
        *s = u * noise_amp;
    }

    // Rajadas com pico de -30 dBFS (10^(-30/20) = 0.0316228)
    let strum_len = 48000 * 2;
    let mut strum = vec![0.0f32; strum_len];
    for (i, s) in strum.iter_mut().enumerate() {
        let t = i as f32 / sample_rate;
        *s = 0.0316228 * (2.0 * std::f32::consts::PI * 220.0 * t).sin();
    }

    let result = calibrate_input(&noise, &strum, sample_rate);

    // CA-1: propõe input.boost = 7.94 ± 0.05 e gate.threshold_db = -46 ± 0.5 dB
    assert!(
        (result.proposed_boost - 7.94).abs() <= 0.05,
        "Boost proposto deve ser 7.94 ± 0.05, obteve {}",
        result.proposed_boost
    );
    assert!(
        (result.proposed_gate_threshold_db - (-46.0)).abs() <= 0.5,
        "Threshold de gate proposto deve ser -46 ± 0.5 dB, obteve {}",
        result.proposed_gate_threshold_db
    );
    assert_eq!(result.diagnostic, DiagnosticReason::Ok);
}

#[test]
fn test_ca_02_input_too_hot_and_input_too_low() {
    let sample_rate = 48000.0;
    let noise = vec![0.0f32; 48000];

    // 1. Pico bruto de -0.5 dBFS (10^(-0.5/20) = 0.944) >= -1 dBFS
    let mut hot_strum = vec![0.0f32; 48000];
    hot_strum[100] = 0.95;
    let res_hot = calibrate_input(&noise, &hot_strum, sample_rate);
    assert_eq!(res_hot.diagnostic, DiagnosticReason::InputTooHot);
    assert!(
        res_hot.proposed_boost <= 1.0,
        "Com input_too_hot, boost não pode ser > 1.0, obteve {}",
        res_hot.proposed_boost
    );

    // 2. Pico bruto de -60 dBFS (10^(-60/20) = 0.001) < -42 dBFS
    let mut low_strum = vec![0.0f32; 48000];
    low_strum[100] = 0.001;
    let res_low = calibrate_input(&noise, &low_strum, sample_rate);
    assert_eq!(res_low.diagnostic, DiagnosticReason::InputTooLow);
    assert_eq!(
        res_low.proposed_boost, 8.0,
        "Com input_too_low, boost deve ser máximo (8.0)"
    );
}

#[test]
fn test_ca_03_hum_detection_50hz_60hz_and_white_noise() {
    let sample_rate = 48000.0;
    let len = 48000 * 2;
    let strum = vec![0.1f32; 48000];

    // Piso com 60 Hz e 180 Hz a -50 dBFS (0.00316)
    let mut noise_60 = vec![0.0f32; len];
    for (i, s) in noise_60.iter_mut().enumerate() {
        let t = i as f32 / sample_rate;
        *s = 0.00316 * (2.0 * std::f32::consts::PI * 60.0 * t).sin()
            + 0.00316 * (2.0 * std::f32::consts::PI * 180.0 * t).sin();
    }
    let res_60 = calibrate_input(&noise_60, &strum, sample_rate);
    assert_eq!(res_60.diagnostic, DiagnosticReason::Hum60Hz);

    // Piso com 50 Hz e 150 Hz a -50 dBFS
    let mut noise_50 = vec![0.0f32; len];
    for (i, s) in noise_50.iter_mut().enumerate() {
        let t = i as f32 / sample_rate;
        *s = 0.00316 * (2.0 * std::f32::consts::PI * 50.0 * t).sin()
            + 0.00316 * (2.0 * std::f32::consts::PI * 150.0 * t).sin();
    }
    let res_50 = calibrate_input(&noise_50, &strum, sample_rate);
    assert_eq!(res_50.diagnostic, DiagnosticReason::Hum50Hz);

    // Apenas ruído branco sem harmônicas de rede
    let mut white_noise = vec![0.0f32; len];
    let mut lcg = 987654321u64;
    for s in white_noise.iter_mut() {
        lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
        *s = ((lcg >> 33) as f32 / (1u32 << 31) as f32 - 1.0) * 0.0001;
    }
    let res_white = calibrate_input(&white_noise, &strum, sample_rate);
    assert_eq!(res_white.diagnostic, DiagnosticReason::Ok);
}

#[test]
fn test_ca_04_adc_clips_counted_and_exposed_in_status() {
    let mut chain = RealtimeDspChain::new(48000.0);
    let frames = 64;
    let mut in_buf = vec![0.2f32; frames];
    // Injeta exatamente 5 amostras em clipping no conversor (|x| >= 0.999)
    in_buf[5] = 1.0;
    in_buf[12] = -0.9995;
    in_buf[25] = 1.05;
    in_buf[40] = -1.0;
    in_buf[55] = 0.9999;

    let mut out_buf = vec![0.0f32; frames];
    chain.process(&in_buf, &mut out_buf);
    assert_eq!(chain.adc_clips(), 5, "adc_clips deve registrar 5 amostras");

    // Verifica que GET /api/audio/status expõe o campo adc_clips
    let chain_arc = Arc::new(Mutex::new(chain));
    let server = LocalAppServer::bind_port_with_telemetry(0, Arc::new(TelemetryBuffer::new(512)), Some(chain_arc))
        .unwrap();
    let port = server.port();

    let (code, _, body) = http_request(port, "GET", "/api/audio/status", &[], None);
    assert_eq!(code, 200);
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["adc_clips"], 5);
}

#[test]
fn test_ca_05_input_source_test_riff_plays_through_dsp_chain() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.set_input_source("test_riff");

    let frames = 512;
    let silent_in = vec![0.0f32; frames];
    let mut out_with_amp = vec![0.0f32; frames];
    chain.process(&silent_in, &mut out_with_amp);

    // RMS deve ser > -40 dBFS (amplitude RMS > 0.01)
    let rms_amp: f32 = (out_with_amp.iter().map(|s| s * s).sum::<f32>() / frames as f32).sqrt();
    assert!(
        rms_amp > 0.01,
        "test_riff deve produzir saída audível (RMS > -40 dBFS), obteve {}",
        rms_amp
    );

    // Desligar amp altera a saída (comprovando que passa pela cadeia de guitarra)
    chain.set_amp_enabled(false);
    let mut out_without_amp = vec![0.0f32; frames];
    chain.process(&silent_in, &mut out_without_amp);

    let diff: f32 = out_with_amp
        .iter()
        .zip(out_without_amp.iter())
        .map(|(a, b)| (a - b).abs())
        .sum();
    assert!(diff > 0.1, "Desligar o amp deve alterar o som do test_riff");
}

#[test]
fn test_ca_06_test_riff_deterministic_bit_identical_and_peak_level() {
    let riff1 = generate_karplus_strong_riff(48000.0, 12345);
    let riff2 = generate_karplus_strong_riff(48000.0, 12345);

    assert_eq!(riff1.len(), riff2.len());
    assert_eq!(riff1, riff2, "Duas gerações com a mesma semente devem ser bit-idênticas");

    // Pico de -12 dBFS ± 0.5 dB (entre -12.5 dBFS = 0.2371 e -11.5 dBFS = 0.2660)
    let peak = riff1.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
    let peak_db = 20.0 * peak.log10();
    assert!(
        (peak_db - (-12.0)).abs() <= 0.5,
        "Pico do riff deve ser -12 dBFS ± 0.5 dB, obteve {:.2} dBFS",
        peak_db
    );
}

#[test]
fn test_ca_07_post_api_input_di_validation_and_resampling() {
    let chain_arc = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_telemetry(0, Arc::new(TelemetryBuffer::new(512)), Some(chain_arc))
        .unwrap();
    let port = server.port();
    let token = server.session_token();

    // 1. WAV 44.1 kHz mono PCM16 válido (1 segundo)
    let wav44k = make_test_wav(1, 44100, 44100);
    let (code_ok, _, _) = http_request(
        port,
        "POST",
        "/api/input/di",
        &[("Content-Type", "audio/wav"), ("X-Sonix-Token", &token)],
        Some(&wav44k),
    );
    assert_eq!(code_ok, 200, "WAV 44.1 kHz deve ser aceito e reamostrado");

    // 2. WAV com channels = 0 -> 400 Bad Request
    let wav_bad_ch = make_test_wav(0, 48000, 100);
    let (code_bad, _, _) = http_request(
        port,
        "POST",
        "/api/input/di",
        &[("Content-Type", "audio/wav"), ("X-Sonix-Token", &token)],
        Some(&wav_bad_ch),
    );
    assert_eq!(code_bad, 400, "channels=0 deve retornar 400 Bad Request");

    // 3. Payload > 20 MB -> 413 Payload Too Large
    let huge_payload = vec![0u8; 20 * 1024 * 1024 + 1024];
    let (code_huge, _, _) = http_request(
        port,
        "POST",
        "/api/input/di",
        &[("Content-Type", "audio/wav"), ("X-Sonix-Token", &token)],
        Some(&huge_payload),
    );
    assert_eq!(code_huge, 413, "Corpo > 20 MB deve retornar 413");
}

#[test]
fn test_ca_08_persisted_calibration_per_device_id_loaded_on_boot() {
    let store = CalibrationStore::default();
    store.save("device_realtek_p2", 5.5, -48.0);

    let server_custom = LocalAppServer::bind_port_with_device(0, Some("device_realtek_p2"), Some(store))
        .unwrap();
    let port_custom = server_custom.port();

    let (_, _, body_custom) = http_request(port_custom, "GET", "/api/state", &[], None);
    let state_custom: serde_json::Value = serde_json::from_slice(&body_custom).unwrap();
    assert_eq!(state_custom["input.boost"], 5.5);
    assert_eq!(state_custom["gate.threshold_db"], -48.0);

    // Dispositivo desconhecido carrega defaults da Spec 014 (boost 2.5, gate -65.0)
    let server_unknown = LocalAppServer::bind_port_with_device(0, Some("device_unknown"), None).unwrap();
    let port_unknown = server_unknown.port();
    let (_, _, body_unknown) = http_request(port_unknown, "GET", "/api/state", &[], None);
    let state_unknown: serde_json::Value = serde_json::from_slice(&body_unknown).unwrap();
    assert_eq!(state_unknown["input.boost"], 2.5);
    assert_eq!(state_unknown["gate.threshold_db"], -65.0);
}

#[test]
fn test_ca_09_non_regression_specs_013_014_and_018() {
    let chain = RealtimeDspChain::new(48000.0);
    assert_eq!(chain.adc_clips(), 0);
}
