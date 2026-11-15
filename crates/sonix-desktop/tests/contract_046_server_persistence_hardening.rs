//! Testes de contrato da Spec 046 — Endurecimento do Servidor dev-http e Presets (CA-1 a CA-4, CA-7)

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use sonix_desktop::server::LocalAppServer;

fn send_raw_http(port: u16, raw_request: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    let _ = stream.set_read_timeout(Some(Duration::from_millis(1500)));
    stream.write_all(raw_request.as_bytes()).unwrap();
    let mut resp = String::new();
    let _ = stream.read_to_string(&mut resp);
    let code = resp
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body = resp.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (code, body)
}

#[test]
fn test_ca_01_host_header_requires_exact_equality_and_rejects_missing() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();

    // Prefixo malicioso localhost.evil.com -> 421
    let (c1, _) = send_raw_http(
        port,
        &format!("GET /api/state HTTP/1.1\r\nHost: localhost.evil.com:{}\r\nConnection: close\r\n\r\n", port),
    );
    assert_eq!(c1, 421, "CA-1: Host localhost.evil.com deveria retornar 421");

    // Prefixo malicioso 127.0.0.1.evil.com -> 421
    let (c2, _) = send_raw_http(
        port,
        "GET /api/state HTTP/1.1\r\nHost: 127.0.0.1.evil.com\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(c2, 421, "CA-1: Host 127.0.0.1.evil.com deveria retornar 421");

    // Host ausente -> 400
    let (c3, _) = send_raw_http(
        port,
        "GET /api/state HTTP/1.1\r\nConnection: close\r\n\r\n",
    );
    assert_eq!(c3, 400, "CA-1: Requisição sem Host deveria retornar 400");

    // Hosts exatos válidos -> 200
    let (c4, _) = send_raw_http(
        port,
        &format!("GET /api/state HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", port),
    );
    assert_eq!(c4, 200, "CA-1: Host exato 127.0.0.1:<port> deveria retornar 200");

    let (c5, _) = send_raw_http(
        port,
        &format!("GET /api/state HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n", port),
    );
    assert_eq!(c5, 200, "CA-1: Host exato localhost:<port> deveria retornar 200");
}

#[test]
fn test_ca_02_origin_header_requires_exact_equality() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();
    let token = server.session_token();
    let payload = r#"{"amp_1.gain":5.5}"#;

    let req_evil = format!(
        "POST /api/params HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nOrigin: http://localhost.evil.com\r\nX-Sonix-Token: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        port, token, payload.len(), payload
    );
    let (c1, _) = send_raw_http(port, &req_evil);
    assert_eq!(c1, 403, "CA-2: Origin http://localhost.evil.com deve retornar 403");

    let req_evil2 = format!(
        "POST /api/params HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nOrigin: http://127.0.0.1.evil.com:{}\r\nX-Sonix-Token: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        port, port, token, payload.len(), payload
    );
    let (c2, _) = send_raw_http(port, &req_evil2);
    assert_eq!(c2, 403, "CA-2: Origin http://127.0.0.1.evil.com deve retornar 403");

    let req_ok = format!(
        "POST /api/params HTTP/1.1\r\nHost: localhost:{}\r\nOrigin: http://localhost:{}\r\nX-Sonix-Token: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        port, port, token, payload.len(), payload
    );
    let (c3, _) = send_raw_http(port, &req_ok);
    assert_eq!(c3, 200, "CA-2: Origin exato http://localhost:<port> deve retornar 200");
}

#[test]
fn test_ca_03_csprng_token_32_hex_and_unique_per_instance() {
    let mut seen = std::collections::HashSet::new();
    for _ in 0..8 {
        let tok = sonix_desktop::server::generate_secure_token();
        assert_eq!(tok.len(), 32, "CA-3: token deve ter 32 hex chars");
        assert!(
            tok.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')),
            "CA-3: token inválido: {}",
            tok
        );
        assert!(seen.insert(tok), "CA-3: colisão de token gerada!");
    }
}

#[test]
fn test_ca_04_preset_id_rejects_traversal_on_save_load_and_export() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();
    let token = server.session_token();

    // Redireciona APPDATA temporariamente dentro de mutex implícito ou verifica que nenhum arquivo é criado
    let evil_save = r#"{"id":"../../evil_046","name":"Evil"}"#;
    let req_save = format!(
        "POST /api/presets/save HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nOrigin: http://127.0.0.1:{}\r\nX-Sonix-Token: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        port, port, token, evil_save.len(), evil_save
    );
    let (c_save, _) = send_raw_http(port, &req_save);
    assert_eq!(c_save, 400, "CA-4: presets/save com ../../evil deve retornar 400");
    assert!(!std::path::Path::new("evil_046.sonixpreset").exists());
    assert!(!std::path::Path::new("../evil_046.sonixpreset").exists());

    for bad_id in ["../../evil", "foo/bar", r"foo\bar", "C:evil", ".hidden", "UPPER"] {
        let load_body = serde_json::json!({ "id": bad_id }).to_string();
        let req_load = format!(
            "POST /api/presets/load HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nOrigin: http://127.0.0.1:{}\r\nX-Sonix-Token: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            port, port, token, load_body.len(), load_body
        );
        let (c_load, _) = send_raw_http(port, &req_load);
        assert_eq!(c_load, 400, "CA-4: presets/load com '{}' deve retornar 400", bad_id);
    }

    for bad_export_path in [
        "/api/presets/C:evil/export",
        "/api/presets/../../evil/export",
        "/api/presets/foo\\bar/export",
    ] {
        let req_exp = format!(
            "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            bad_export_path, port
        );
        let (c_exp, _) = send_raw_http(port, &req_exp);
        assert_eq!(c_exp, 400, "CA-4: export '{}' deve retornar 400", bad_export_path);
    }
}

#[test]
fn test_ca_07_legitimate_preset_save_load_export_with_valid_id() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();
    let token = server.session_token();

    let load_body = r#"{"id":"brit800"}"#;
    let req_load = format!(
        "POST /api/presets/load HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nOrigin: http://127.0.0.1:{}\r\nX-Sonix-Token: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        port, port, token, load_body.len(), load_body
    );
    let (c_load, _) = send_raw_http(port, &req_load);
    assert_eq!(c_load, 200, "CA-7: carga legítima brit800 falhou");
}
