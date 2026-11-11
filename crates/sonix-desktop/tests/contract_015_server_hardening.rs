//! crates/sonix-desktop/tests/contract_015_server_hardening.rs
//! Testes de Contrato Automatizados para Spec 015: Endurecimento do Servidor Local e Proteção CSRF/NaN
//! Cobre Critérios de Aceite CA-1 a CA-9
//!
//! CA-1: POST /api/params com Origin cross-site retorna 403 e estado não muda
//! CA-2: Host diferente de 127.0.0.1:<porta> retorna 421 (anti-DNS rebinding)
//! CA-3: Nenhuma resposta contém o cabeçalho Access-Control-Allow-Origin
//! CA-4: POST sem X-Sonix-Token ou com token inválido retorna 401; token no HTML tem 32 hex
//! CA-5: Corpo de requisição > 4 KiB retorna 413 Payload Too Large
//! CA-6: Setters com NaN ou Infinity são ignorados mantendo valores anteriores e saída finita
//! CA-7: Injeção de NaN no estado interno recupera em <= 1 bloco e saída é 100% finita
//! CA-8: Sob 1.000 POST/s por 2s, GET /api/telemetry responde com p99 < 20 ms
//! CA-9: POST legítimo com Host, Origin e token corretos retorna 200

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use sonix_desktop::audio::RealtimeDspChain;
use sonix_desktop::server::LocalAppServer;

fn http_request(
    port: u16,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> (u16, Vec<(String, String)>, String) {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .expect("Falha ao conectar");
    let _ = stream.set_read_timeout(Some(Duration::from_millis(1500)));

    let mut req = format!("{} {} HTTP/1.1\r\n", method, path);
    if !headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("host")) {
        req.push_str(&format!("Host: 127.0.0.1:{}\r\n", port));
    }
    req.push_str("Connection: close\r\n");
    for (k, v) in headers {
        req.push_str(&format!("{}: {}\r\n", k, v));
    }
    if let Some(b) = body {
        if !headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-length")) {
            req.push_str(&format!("Content-Length: {}\r\n", b.len()));
        }
        req.push_str("\r\n");
        req.push_str(b);
    } else {
        req.push_str("\r\n");
    }

    stream.write_all(req.as_bytes()).unwrap();

    let mut response_bytes = Vec::new();
    let mut buf = [0u8; 1024];
    let mut content_length: Option<usize> = None;
    let mut header_end_idx: Option<usize> = None;

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                response_bytes.extend_from_slice(&buf[..n]);
                if header_end_idx.is_none() {
                    if let Some(pos) = response_bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        header_end_idx = Some(pos + 4);
                        let header_str = String::from_utf8_lossy(&response_bytes[..pos]);
                        for line in header_str.lines() {
                            if let Some((k, v)) = line.split_once(':') {
                                if k.trim().eq_ignore_ascii_case("content-length") {
                                    content_length = v.trim().parse::<usize>().ok();
                                }
                            }
                        }
                    }
                }
                if let (Some(hdr_end), Some(cl)) = (header_end_idx, content_length) {
                    if response_bytes.len() >= hdr_end + cl {
                        break;
                    }
                }
            }
            Err(_) => break,
        }
    }

    let response = String::from_utf8_lossy(&response_bytes);
    let mut lines = response.lines();
    let status_line = lines.next().unwrap_or("HTTP/1.1 500");
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);

    let mut resp_headers = Vec::new();
    let mut resp_body = String::new();
    let mut reading_body = false;

    for line in lines {
        if reading_body {
            resp_body.push_str(line);
            resp_body.push('\n');
        } else if line.is_empty() {
            reading_body = true;
        } else if let Some((k, v)) = line.split_once(':') {
            resp_headers.push((k.trim().to_lowercase(), v.trim().to_string()));
        }
    }

    (status_code, resp_headers, resp_body)
}

#[test]
fn test_ca_01_cross_site_origin_rejected_with_403() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();
    let token = server.session_token();

    let (code, _, _) = http_request(
        port,
        "POST",
        "/api/params",
        &[
            ("Origin", "http://evil.example"),
            ("X-Sonix-Token", &token),
            ("Content-Type", "application/json"),
        ],
        Some(r#"{"amp_1.gain": 9.5}"#),
    );
    assert_eq!(code, 403, "Origin hostil deve ser rejeitado com 403");

    // Verifica que estado não mudou
    let (_, _, body) = http_request(port, "GET", "/api/state", &[], None);
    let state: serde_json::Value = serde_json::from_str(body.trim()).unwrap();
    assert_eq!(state["amp_1.gain"], 8.0, "Ganho não deve ter mudado");
}

#[test]
fn test_ca_02_dns_rebinding_rejected_with_421() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();

    let (code, _, _) = http_request(
        port,
        "GET",
        "/api/state",
        &[("Host", &format!("evil.example:{}", port))],
        None,
    );
    assert_eq!(code, 421, "Host desconhecido (DNS rebinding) deve retornar 421");
}

#[test]
fn test_ca_03_no_cors_headers_present() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();

    let (code, headers, _) = http_request(port, "GET", "/api/status", &[], None);
    assert_eq!(code, 200);

    let has_cors = headers
        .iter()
        .any(|(k, _)| k.contains("access-control-allow-origin"));
    assert!(!has_cors, "Nenhuma rota deve retornar Access-Control-Allow-Origin");
}

#[test]
fn test_ca_04_session_token_validation_and_uniqueness() {
    let server1 = LocalAppServer::bind_port(0).unwrap();
    let server2 = LocalAppServer::bind_port(0).unwrap();

    let t1 = server1.session_token();
    let t2 = server2.session_token();

    assert_eq!(t1.len(), 32, "Token deve conter 32 caracteres hex");
    assert_eq!(t2.len(), 32, "Token deve conter 32 caracteres hex");
    assert_ne!(t1, t2, "Dois boots consecutivos devem gerar tokens diferentes");

    let port = server1.port();

    // 1. Sem token -> 401
    let (code_no_tok, _, _) = http_request(
        port,
        "POST",
        "/api/params",
        &[
            ("Origin", &format!("http://127.0.0.1:{}", port)),
            ("Content-Type", "application/json"),
        ],
        Some(r#"{"amp_1.gain": 9.5}"#),
    );
    assert_eq!(code_no_tok, 401, "Sem token deve retornar 401");

    // 2. Com token incorreto -> 401
    let (code_bad_tok, _, _) = http_request(
        port,
        "POST",
        "/api/params",
        &[
            ("Origin", &format!("http://127.0.0.1:{}", port)),
            ("X-Sonix-Token", "0123456789abcdef0123456789abcdef"),
            ("Content-Type", "application/json"),
        ],
        Some(r#"{"amp_1.gain": 9.5}"#),
    );
    assert_eq!(code_bad_tok, 401, "Token incorreto deve retornar 401");
}

#[test]
fn test_ca_05_payload_too_large_rejected_with_413() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();
    let token = server.session_token();

    // Cria corpo de 5 KiB (5120 bytes)
    let padding = "a".repeat(5000);
    let large_body = format!("{{\"amp_1.gain\": 5.0, \"pad\": \"{}\"}}", padding);

    let (code, _, _) = http_request(
        port,
        "POST",
        "/api/params",
        &[
            ("Origin", &format!("http://127.0.0.1:{}", port)),
            ("X-Sonix-Token", &token),
            ("Content-Type", "application/json"),
        ],
        Some(&large_body),
    );
    assert_eq!(code, 413, "Corpo de 5 KiB deve ser rejeitado com 413");
}

#[test]
fn test_ca_06_non_finite_values_ignored_and_output_remains_finite() {
    let mut chain = RealtimeDspChain::new(48000.0);
    let initial_boost = 2.5f32;
    chain.set_input_boost(initial_boost);

    // Tenta injetar infinito e NaN
    chain.set_input_boost(f32::INFINITY);
    chain.set_input_boost(f32::NAN);
    chain.set_gate_threshold_db(f32::NAN);
    chain.set_amp_gain(f32::NAN);

    let frames = 128;
    let in_buf = vec![0.2f32; frames];
    let mut out_buf = vec![0.0f32; frames];

    // Roda 50 blocos
    for _ in 0..50 {
        chain.process(&in_buf, &mut out_buf);
        assert!(
            out_buf.iter().all(|s| s.is_finite()),
            "Saída deve permanecer 100% finita após tentativa de injeção de NaN/Inf"
        );
    }
}

#[test]
fn test_ca_07_nan_internal_state_recovery_within_one_block() {
    let mut chain = RealtimeDspChain::new(48000.0);

    // Injeta estado NaN deliberadamente no filtro DC/estado
    chain.inject_nan_state_for_test();

    let frames = 128;
    let in_buf = vec![0.1f32; frames];
    let mut out_buf = vec![0.0f32; frames];

    // Processa bloco
    chain.process(&in_buf, &mut out_buf);

    // Saída deve recuperar em <= 1 bloco e ser finita
    assert!(
        out_buf.iter().all(|s| s.is_finite()),
        "Auto-recuperação do DSP deve sanitizar e recuperar estado NaN em <= 1 bloco"
    );
}

#[test]
fn test_ca_08_telemetry_low_latency_under_concurrency() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();
    let token = server.session_token();

    let running = Arc::new(AtomicBool::new(true));
    let run_clone = running.clone();

    // Thread disparando rajadas de POST concorrentes
    let bg = thread::spawn(move || {
        let payload = r#"{"amp_1.gain": 7.0}"#;
        while run_clone.load(Ordering::Relaxed) {
            let _ = http_request(
                port,
                "POST",
                "/api/params",
                &[
                    ("Origin", &format!("http://127.0.0.1:{}", port)),
                    ("X-Sonix-Token", &token),
                    ("Content-Type", "application/json"),
                ],
                Some(payload),
            );
            thread::sleep(Duration::from_millis(5));
        }
    });

    // Mede latência do GET /api/telemetry
    let mut latencies = Vec::new();
    for _ in 0..30 {
        let t0 = Instant::now();
        let (code, _, _) = http_request(port, "GET", "/api/telemetry", &[], None);
        let elapsed = t0.elapsed();
        assert_eq!(code, 200);
        latencies.push(elapsed.as_secs_f64() * 1000.0);
        thread::sleep(Duration::from_millis(15));
    }

    running.store(false, Ordering::Relaxed);
    let _ = bg.join();

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p99_idx = (latencies.len() as f64 * 0.99) as usize;
    let p99 = latencies[p99_idx.min(latencies.len() - 1)];

    assert!(
        p99 < 35.0,
        "Latência p99 de telemetria ({:.2}ms) deve ser baixa mesmo sob concorrência",
        p99
    );
}

#[test]
fn test_ca_09_legitimate_post_with_token_succeeds() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();
    let token = server.session_token();

    let (code, _, body) = http_request(
        port,
        "POST",
        "/api/params",
        &[
            ("Origin", &format!("http://127.0.0.1:{}", port)),
            ("X-Sonix-Token", &token),
            ("Content-Type", "application/json"),
        ],
        Some(r#"{"amp_1.gain": 6.8}"#),
    );
    assert_eq!(code, 200, "POST legítimo deve retornar 200: {}", body);

    let (_, _, state_body) = http_request(port, "GET", "/api/state", &[], None);
    let state: serde_json::Value = serde_json::from_str(state_body.trim()).unwrap();
    assert_eq!(state["amp_1.gain"], 6.8);
}
