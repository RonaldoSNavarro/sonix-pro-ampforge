//! Testes de contrato API da Spec 030 — Parâmetros e Validação do Noise Gate Completo (CA-4)

use sonix_desktop::audio::RealtimeDspChain;
use sonix_desktop::server::LocalAppServer;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

fn http_request(
    port: u16,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &str,
) -> (u16, String) {
    let mut stream =
        TcpStream::connect(("127.0.0.1", port)).expect("Falha ao conectar no servidor local");
    let content_len = body.len();
    let mut req_str = format!(
        "{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: {}\r\n",
        method, path, port, content_len
    );
    for (k, v) in headers {
        req_str.push_str(&format!("{}: {}\r\n", k, v));
    }
    req_str.push_str("Connection: close\r\n\r\n");
    req_str.push_str(body);

    stream.write_all(req_str.as_bytes()).unwrap();
    let mut resp_str = String::new();
    stream.read_to_string(&mut resp_str).unwrap();

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
fn test_ca_04_gate_params_schema_validation_and_atomic_rejection() {
    let dsp = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_dsp(0, dsp).unwrap();
    let port = server.port();
    let token = server.session_token();
    let origin = format!("http://127.0.0.1:{}", port);

    // 1. Estado inicial deve expor todas as chaves gate.* da Spec 030
    let (code_init, init_body) = http_request(port, "GET", "/api/state", &[], "");
    assert_eq!(code_init, 200);
    let init_state: serde_json::Value = serde_json::from_str(&init_body).unwrap();
    for key in [
        "gate.enabled",
        "gate.mode",
        "gate.threshold_db",
        "gate.hysteresis_db",
        "gate.attack_ms",
        "gate.hold_ms",
        "gate.release_ms",
        "gate.range_db",
        "gate.key_hpf_hz",
        "gate.key_lpf_hz",
    ] {
        assert!(init_state.get(key).is_some(), "GET /api/state deve conter {}", key);
    }

    // 2. Atualização válida de todos os parâmetros gate.*
    let valid_payload = serde_json::json!({
        "gate.enabled": true,
        "gate.mode": "classic",
        "gate.threshold_db": -42.0,
        "gate.hysteresis_db": 6.0,
        "gate.attack_ms": 2.5,
        "gate.hold_ms": 80.0,
        "gate.release_ms": 150.0,
        "gate.range_db": -35.0,
        "gate.key_hpf_hz": 120.0,
        "gate.key_lpf_hz": 4500.0
    })
    .to_string();

    let (code_ok, ok_body) = http_request(
        port,
        "POST",
        "/api/params",
        &[
            ("Origin", &origin),
            ("Content-Type", "application/json"),
            ("X-Sonix-Token", &token),
        ],
        &valid_payload,
    );
    assert_eq!(code_ok, 200, "POST /api/params gate válido deve retornar 200: {}", ok_body);

    let (_, updated_body) = http_request(port, "GET", "/api/state", &[], "");
    let updated: serde_json::Value = serde_json::from_str(&updated_body).unwrap();
    assert_eq!(updated["gate.mode"], "classic");
    assert_eq!(updated["gate.hysteresis_db"], 6.0);
    assert_eq!(updated["gate.hold_ms"], 80.0);
    assert_eq!(updated["gate.range_db"], -35.0);
    assert_eq!(updated["gate.key_hpf_hz"], 120.0);
    assert_eq!(updated["gate.key_lpf_hz"], 4500.0);

    // 3. Rejeição atômica de valores fora de faixa com 400 Bad Request sem mutar estado anterior
    let invalid_payloads = [
        r#"{"gate.mode": "unknown"}"#,
        r#"{"gate.hysteresis_db": 15.0}"#,
        r#"{"gate.attack_ms": 0.01}"#,
        r#"{"gate.hold_ms": 600.0}"#,
        r#"{"gate.release_ms": 1500.0}"#,
        r#"{"gate.range_db": -100.0}"#,
        r#"{"gate.key_hpf_hz": 10.0}"#,
        r#"{"gate.key_lpf_hz": 15000.0}"#,
        r#"{"gate.threshold_db": -30.0, "gate.hold_ms": 999.0}"#,
    ];

    for bad in invalid_payloads {
        let (code_bad, _) = http_request(
            port,
            "POST",
            "/api/params",
            &[
                ("Origin", &origin),
                ("Content-Type", "application/json"),
                ("X-Sonix-Token", &token),
            ],
            bad,
        );
        assert_eq!(code_bad, 400, "Payload inválido {} deve retornar 400", bad);
    }

    let (_, after_bad_body) = http_request(port, "GET", "/api/state", &[], "");
    let after_bad: serde_json::Value = serde_json::from_str(&after_bad_body).unwrap();
    assert_eq!(
        after_bad["gate.threshold_db"], -42.0,
        "Falha atômica não pode mutar parâmetros válidos"
    );
}
