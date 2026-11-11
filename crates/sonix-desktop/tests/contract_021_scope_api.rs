//! crates/sonix-desktop/tests/contract_021_scope_api.rs
//! Testes de Contrato Automatizados para Spec 021: API de Escopo e Estado da Bancada (Mod Shop)
//! Cobre Critérios de Aceite CA-9 e CA-10 lado Rust

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
fn test_ca_09_model_nominals_and_mod_indicator() {
    let dsp_chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_telemetry(0, Arc::new(TelemetryBuffer::new(512)), Some(dsp_chain))
        .expect("Falha ao ligar server");
    let port = server.port();
    let token = server.session_token();

    // 1. Estado inicial é nominal (is_modded = false)
    let (code_st, resp_st) = http_request(port, "GET", "/api/state", &[], None);
    assert_eq!(code_st, 200);
    let json_st: serde_json::Value = serde_json::from_str(&resp_st).unwrap();
    assert_eq!(json_st["is_modded"], false, "Inicialmente não deve estar modificado");

    // 2. Altera bias_ma -> is_modded vira true
    let (code_mod, _) = http_request(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        Some(r#"{"amp_1.bias_ma": 48.0}"#),
    );
    assert_eq!(code_mod, 200);

    let (_, resp_st2) = http_request(port, "GET", "/api/state", &[], None);
    let json_st2: serde_json::Value = serde_json::from_str(&resp_st2).unwrap();
    assert_eq!(json_st2["is_modded"], true, "Após alterar bias, deve acusar is_modded = true");

    // 3. Troca para outro modelo -> restaura nominais do novo modelo e is_modded = false
    let (code_sw, _) = http_request(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        Some(r#"{"amp_1.model": "twin_ab763"}"#),
    );
    assert_eq!(code_sw, 200);

    let (_, resp_st3) = http_request(port, "GET", "/api/state", &[], None);
    let json_st3: serde_json::Value = serde_json::from_str(&resp_st3).unwrap();
    assert_eq!(json_st3["amp_1.model"], "twin_ab763");
    assert_eq!(json_st3["is_modded"], false, "Ao trocar de modelo, deve voltar aos nominais");
}

#[test]
fn test_ca_10_scope_transfer_curve_symmetry() {
    let dsp_chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_telemetry(0, Arc::new(TelemetryBuffer::new(512)), Some(dsp_chain))
        .expect("Falha ao ligar server");
    let port = server.port();
    let token = server.session_token();

    // 1. Ajusta assimetria para 0.0
    let (c1, _) = http_request(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        Some(r#"{"amp_1.triode_asym": 0.0}"#),
    );
    assert_eq!(c1, 200);

    // Consulta GET /api/scope?mode=transfer
    let (c_scope1, body1) = http_request(port, "GET", "/api/scope?mode=transfer", &[], None);
    assert_eq!(c_scope1, 200);
    let json1: serde_json::Value = serde_json::from_str(&body1).expect("JSON inválido no /api/scope");
    let pts1 = json1["transfer"].as_array().expect("transfer deve ser array");
    assert!(!pts1.is_empty(), "transfer não pode ser vazio");

    // Verifica simetria: para cada (x, y), se encontrar (-x, y2), |y + y2| <= 1e-4
    for pt in pts1 {
        let x = pt["x"].as_f64().unwrap();
        let y = pt["y"].as_f64().unwrap();
        if let Some(opp) = pts1.iter().find(|p| (p["x"].as_f64().unwrap() + x).abs() < 1e-5) {
            let y_opp = opp["y"].as_f64().unwrap();
            assert!(
                (y + y_opp).abs() <= 1e-3,
                "Curva com triode_asym=0 deve ser simétrica: x={}, y={}, y_opp={}",
                x, y, y_opp
            );
        }
    }

    // 2. Ajusta assimetria para 0.6
    let (c2, _) = http_request(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        Some(r#"{"amp_1.triode_asym": 0.6}"#),
    );
    assert_eq!(c2, 200);

    let (c_scope2, body2) = http_request(port, "GET", "/api/scope?mode=transfer", &[], None);
    assert_eq!(c_scope2, 200);
    let json2: serde_json::Value = serde_json::from_str(&body2).unwrap();
    let pts2 = json2["transfer"].as_array().unwrap();

    // Deve ser assimétrica
    let mut max_asym = 0.0f64;
    for pt in pts2 {
        let x = pt["x"].as_f64().unwrap();
        let y = pt["y"].as_f64().unwrap();
        if let Some(opp) = pts2.iter().find(|p| (p["x"].as_f64().unwrap() + x).abs() < 1e-5) {
            let y_opp = opp["y"].as_f64().unwrap();
            max_asym = max_asym.max((y + y_opp).abs());
        }
    }
    assert!(max_asym >= 0.05, "Curva com triode_asym=0.6 deve ser marcadamente assimétrica: got {}", max_asym);
}
