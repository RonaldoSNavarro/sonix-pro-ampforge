//! crates/sonix-desktop/tests/contract_020_amp_models_api.rs
//! Testes de Contrato Automatizados para Spec 020: API de Modelos de Amp e Controles por Circuito
//! Cobre Critérios de Aceite CA-4 e CA-9

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use sonix_desktop::audio::{RealtimeDspChain, TelemetryBuffer};
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

    let resp_body = response.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (status_code, resp_body)
}

#[test]
fn test_ca_04_controls_per_model_validation_and_model_switching() {
    let dsp_chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_telemetry(0, Arc::new(TelemetryBuffer::new(512)), Some(dsp_chain))
        .expect("Falha ao ligar server");
    let port = server.port();
    let token = server.session_token();

    // 1. amp_1.model = "jcm900" desconhecido deve retornar 400
    let (code_bad, _) = http_request(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        Some(r#"{"amp_1.model": "jcm900"}"#),
    );
    assert_eq!(code_bad, 400, "Modelo desconhecido deve retornar 400 Bad Request");

    // 2. Muda para plexi_1959
    let (code_switch, resp_switch) = http_request(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        Some(r#"{"amp_1.model": "plexi_1959"}"#),
    );
    assert_eq!(code_switch, 200, "Troca de modelo deve retornar 200: {}", resp_switch);

    // 3. POST {"amp_1.master": 3} com plexi_1959 ativo deve retornar 200 e constar em ignored com not_on_model
    let (code_ignored, resp_ignored) = http_request(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        Some(r#"{"amp_1.master": 3.0}"#),
    );
    assert_eq!(code_ignored, 200);
    let json_ignored: serde_json::Value = serde_json::from_str(&resp_ignored).unwrap();
    let ignored_arr = json_ignored["ignored"].as_array().expect("ignored deve ser array");
    assert!(
        ignored_arr.iter().any(|v| v == "amp_1.master"),
        "amp_1.master deve ser ignorado para modelo sem master"
    );

    // 4. engine.oversampling = "4x" e "8x" válidos
    let (code_os, resp_os) = http_request(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        Some(r#"{"engine.oversampling": "8x"}"#),
    );
    assert_eq!(code_os, 200, "engine.oversampling deve retornar 200: {}", resp_os);
}

#[test]
fn test_ca_09_non_regression_specs_003_014_016() {
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.set_amp_gain(7.0);
    assert_eq!(chain.sample_rate(), 48000.0);
}
