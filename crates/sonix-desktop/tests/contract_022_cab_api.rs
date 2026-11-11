//! crates/sonix-desktop/tests/contract_022_cab_api.rs
//! Testes de Contrato Automatizados para Spec 022: API de Gabinete, Upload de IR e Bloqueio de Microfones
//! Cobre Critérios de Aceite CA-11 e CA-13

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use sonix_desktop::audio::{RealtimeDspChain, TelemetryBuffer};
use sonix_desktop::server::LocalAppServer;

fn http_request_bytes(
    port: u16,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> (u16, String) {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .expect("Falha ao conectar ao servidor local");

    let mut req = format!("{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n", method, path, port);
    for (k, v) in headers {
        req.push_str(&format!("{}: {}\r\n", k, v));
    }
    if !headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("Content-Length")) {
        req.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    req.push_str("\r\n");

    stream.write_all(req.as_bytes()).unwrap();
    if !body.is_empty() {
        stream.write_all(body).unwrap();
    }

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    let status_line = response.lines().next().unwrap_or("HTTP/1.1 500");
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);

    let resp_body = response.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (status_code, resp_body)
}

fn create_wav_bytes(sample_rate: u32, channels: u16, bits_per_sample: u16, num_samples: usize) -> Vec<u8> {
    let byte_rate = sample_rate * channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = channels * (bits_per_sample / 8);
    let data_len = num_samples * block_align as usize;
    let total_len = 36 + data_len;

    let mut b = Vec::with_capacity(44 + data_len);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(total_len as u32).to_le_bytes());
    b.extend_from_slice(b"WAVE");
    b.extend_from_slice(b"fmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&channels.to_le_bytes());
    b.extend_from_slice(&sample_rate.to_le_bytes());
    b.extend_from_slice(&byte_rate.to_le_bytes());
    b.extend_from_slice(&block_align.to_le_bytes());
    b.extend_from_slice(&bits_per_sample.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(data_len as u32).to_le_bytes());

    for i in 0..num_samples {
        let val = if i == 0 { 16000i16 } else { 0i16 };
        b.extend_from_slice(&val.to_le_bytes());
    }
    b
}

#[test]
fn test_ca_11_ir_upload_limits_and_invalid_files() {
    let dsp_chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_telemetry(0, Arc::new(TelemetryBuffer::new(512)), Some(dsp_chain))
        .expect("Falha ao ligar server");
    let port = server.port();
    let token = server.session_token();

    // 1. Upload de IR válida (100 ms a 48 kHz = 4800 amostras)
    let valid_wav = create_wav_bytes(48000, 1, 16, 4800);
    let (code_ok, resp_ok) = http_request_bytes(
        port,
        "POST",
        "/api/cab/ir",
        &[("Content-Type", "audio/wav"), ("X-Sonix-Token", &token)],
        &valid_wav,
    );
    assert_eq!(code_ok, 200, "Upload de IR válida deve retornar 200: {}", resp_ok);
    assert!(resp_ok.contains("\"loaded\":true") || resp_ok.contains("\"loaded\": true"));

    // 2. Upload de IR excedendo 500 ms (600 ms a 48 kHz = 28800 amostras)
    let too_long_wav = create_wav_bytes(48000, 1, 16, 28800);
    let (code_long, resp_long) = http_request_bytes(
        port,
        "POST",
        "/api/cab/ir",
        &[("Content-Type", "audio/wav"), ("X-Sonix-Token", &token)],
        &too_long_wav,
    );
    assert_eq!(code_long, 400, "Upload de IR > 500 ms deve retornar 400 Bad Request");
    assert!(resp_long.contains("ir_too_long"), "Mensagem deve conter ir_too_long: {}", resp_long);

    // 3. Upload de IR com channels = 0
    let mut bad_channel_wav = valid_wav.clone();
    bad_channel_wav[22] = 0;
    bad_channel_wav[23] = 0;
    let (code_bad_ch, _resp_bad_ch) = http_request_bytes(
        port,
        "POST",
        "/api/cab/ir",
        &[("Content-Type", "audio/wav"), ("X-Sonix-Token", &token)],
        &bad_channel_wav,
    );
    assert_eq!(code_bad_ch, 400, "IR com 0 canais deve retornar 400 Bad Request");
}

#[test]
fn test_ca_13_ir_active_blocks_mic_params() {
    let dsp_chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_telemetry(0, Arc::new(TelemetryBuffer::new(512)), Some(dsp_chain))
        .expect("Falha ao ligar server");
    let port = server.port();
    let token = server.session_token();

    // 1. Ativa a IR (cab_1.ir_enabled = true)
    let (code_enable_ir, resp_enable_ir) = http_request_bytes(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        br#"{"cab_1.ir_enabled": true}"#,
    );
    assert_eq!(code_enable_ir, 200, "Ativar ir_enabled deve retornar 200: {}", resp_enable_ir);

    // 2. Com IR ativa, envia alteração de mic -> deve constar em ignored com motivo ir_active
    let (code_mic, resp_mic) = http_request_bytes(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        br#"{"cab_1.mic1.position": 0.8, "cab_1.mic1.distance_cm": 15.0}"#,
    );
    assert_eq!(code_mic, 200);
    let json_mic: serde_json::Value = serde_json::from_str(&resp_mic).unwrap();
    let ignored_arr = json_mic["ignored"].as_array().expect("ignored deve ser array");
    assert!(
        ignored_arr.iter().any(|v| v == "cab_1.mic1.position"),
        "cab_1.mic1.position deve constar em ignored quando IR está ativa"
    );
    assert!(
        ignored_arr.iter().any(|v| v == "cab_1.mic1.distance_cm"),
        "cab_1.mic1.distance_cm deve constar em ignored quando IR está ativa"
    );

    let reasons = json_mic.get("ignored_reasons").and_then(|r| r.as_object());
    assert!(
        reasons.is_some() && reasons.unwrap().get("cab_1.mic1.position").and_then(|v| v.as_str()) == Some("ir_active"),
        "ignored_reasons deve apontar ir_active para cab_1.mic1.position: {:?}", json_mic
    );

    // 3. Desativa IR (cab_1.ir_enabled = false)
    let (code_disable_ir, _) = http_request_bytes(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        br#"{"cab_1.ir_enabled": false}"#,
    );
    assert_eq!(code_disable_ir, 200);

    // 4. Sem IR ativa, envia alteração de mic -> deve ser aceita em applied
    let (code_mic_ok, resp_mic_ok) = http_request_bytes(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        br#"{"cab_1.mic1.position": 0.4}"#,
    );
    assert_eq!(code_mic_ok, 200);
    let json_mic_ok: serde_json::Value = serde_json::from_str(&resp_mic_ok).unwrap();
    let applied_arr = json_mic_ok["applied"].as_array().expect("applied deve ser array");
    assert!(
        applied_arr.iter().any(|v| v == "cab_1.mic1.position"),
        "cab_1.mic1.position deve constar em applied quando IR está inativa: {:?}", json_mic_ok
    );
}
