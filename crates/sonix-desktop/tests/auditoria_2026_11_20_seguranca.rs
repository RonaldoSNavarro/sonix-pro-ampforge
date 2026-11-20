//! Testes de auditoria 2026-11-20 — Segurança (AUD-01, AUD-03, AUD-04)

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sonix_desktop::server::LocalAppServer;

fn send_raw(port: u16, raw: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    let _ = stream.set_read_timeout(Some(Duration::from_millis(2000)));
    stream.write_all(raw.as_bytes()).expect("write");
    let mut resp = String::new();
    let _ = stream.read_to_string(&mut resp);
    let status = resp
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body = resp.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (status, body)
}

fn http_req(
    port: u16,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &str,
) -> (u16, String) {
    let mut raw = format!("{} {} HTTP/1.1\r\n", method, path);
    if !headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("host")) {
        raw.push_str(&format!("Host: 127.0.0.1:{}\r\n", port));
    }
    for (k, v) in headers {
        raw.push_str(&format!("{}: {}\r\n", k, v));
    }
    raw.push_str(&format!(
        "Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    ));
    send_raw(port, &raw)
}

#[test]
fn aud_01_post_simples_cross_site_rejeitado() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();

    for path in [
        "/api/reamp/run",
        "/api/modifiers",
        "/api/footswitch/press",
        "/api/graph",
    ] {
        let body = match path {
            "/api/reamp/run" => r#"{"probe":[0.1],"loopback_rec":[0.1],"reamp_rec":[0.1]}"#,
            "/api/modifiers" => r#"{"id":"x","target_param":"amp_1.gain","source":"EXP_PEDAL_1","min_val":0.0,"max_val":10.0}"#,
            "/api/footswitch/press" => r#"{"index":1}"#,
            _ => r#"{}"#,
        };
        let (code, resp_body) = http_req(
            port,
            "POST",
            path,
            &[
                ("Origin", "http://evil.example"),
                ("Content-Type", "text/plain"),
            ],
            body,
        );
        assert_eq!(
            code, 403,
            "AUD-01: POST cross-site sem token em {} deveria retornar 403, retornou {} ({})",
            path, code, resp_body
        );
    }
}

#[test]
fn aud_01_host_invalido_rejeitado() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();

    let (code_evil, _) = send_raw(
        port,
        &format!(
            "GET / HTTP/1.1\r\nHost: evil.example:{}\r\nConnection: close\r\n\r\n",
            port
        ),
    );
    assert_eq!(
        code_evil, 421,
        "AUD-01: GET / com Host evil.example deveria ser rejeitado (421), obteve {}",
        code_evil
    );

    // Host duplicado (smuggling): primeiro valido, segundo hostil -> deve ser rejeitado!
    let (code_dup, _) = send_raw(
        port,
        &format!(
            "GET / HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nHost: evil.example:{}\r\nConnection: close\r\n\r\n",
            port, port
        ),
    );
    assert_ne!(
        code_dup, 200,
        "AUD-01: GET / com múltiplos headers Host não pode retornar 200 (obteve {})",
        code_dup
    );
}

#[test]
fn aud_01_reamp_recusa_session_path() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();
    let token = server.session_token();
    let uniq = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp_dir = std::env::temp_dir().join(format!("sonix_aud01_sp_{}_{}", std::process::id(), uniq));
    let target = tmp_dir.join("x").join("y.json");
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let payload = serde_json::json!({
        "probe": [0.5, -0.25],
        "loopback_rec": [0.0, 0.5, -0.25],
        "reamp_rec": [0.1, 0.2, 0.3],
        "session_path": target.to_string_lossy(),
    })
    .to_string();

    let (code, body) = http_req(
        port,
        "POST",
        "/api/reamp/run",
        &[
            ("Origin", &format!("http://127.0.0.1:{}", port)),
            ("X-Sonix-Token", &token),
            ("Content-Type", "application/json"),
        ],
        &payload,
    );
    let existed = target.exists();
    let _ = std::fs::remove_dir_all(&tmp_dir);

    assert_eq!(
        code, 400,
        "AUD-01: session_path deve ser recusado com 400, retornou {} ({})",
        code, body
    );
    assert!(
        !existed,
        "AUD-01: arquivo arbitrário {:?} não pode existir",
        target
    );
}

#[test]
fn aud_01_di_len_absurdo_nao_derruba() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();
    let token = server.session_token();
    let payload = serde_json::json!({
        "probe": [0.5, -0.25],
        "loopback_rec": [0.0, 0.5, -0.25],
        "reamp_rec": [0.1, 0.2, 0.3],
        "di_len": 1u64 << 62,
    })
    .to_string();

    let (code, body) = http_req(
        port,
        "POST",
        "/api/reamp/run",
        &[
            ("Origin", &format!("http://127.0.0.1:{}", port)),
            ("X-Sonix-Token", &token),
            ("Content-Type", "application/json"),
        ],
        &payload,
    );
    assert_eq!(
        code, 400,
        "AUD-01: di_len absurdo deve retornar 400, retornou {} ({})",
        code, body
    );

    let (st_code, _) = send_raw(
        port,
        &format!(
            "GET /api/state HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            port
        ),
    );
    assert_eq!(st_code, 200, "AUD-01: servidor deve seguir vivo após di_len absurdo");
}

#[test]
fn aud_01_toda_rota_mutante_exige_token() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();

    let mut visited = 0usize;
    for &(route_method, pattern) in sonix_desktop::api::ROUTES {
        if route_method == "GET" || route_method == "HEAD" {
            continue;
        }
        visited += 1;
        let path = pattern.replace('*', "factory_clean");

        let (code_no_tok, body_no_tok) = http_req(
            port,
            route_method,
            &path,
            &[("Content-Type", "application/json")],
            "{}",
        );
        assert!(
            code_no_tok == 401 || code_no_tok == 403,
            "AUD-01: {} {} sem token deveria retornar 401/403, retornou {} ({})",
            route_method,
            path,
            code_no_tok,
            body_no_tok
        );

        let (code_cross, body_cross) = http_req(
            port,
            route_method,
            &path,
            &[
                ("Origin", "http://evil.example"),
                ("Content-Type", "text/plain"),
            ],
            "{}",
        );
        assert_eq!(
            code_cross, 403,
            "AUD-01: {} {} cross-site sem token deveria retornar 403, retornou {} ({})",
            route_method, path, code_cross, body_cross
        );
    }
    assert!(visited >= 10, "Esperava >= 10 rotas mutantes em ROUTES, achou {}", visited);
}

#[test]
fn aud_01_reamp_grava_na_pasta_do_app() {
    let uniq = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp_dir = std::env::temp_dir().join(format!("sonix_aud01_dir_{}_{}", std::process::id(), uniq));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let server = LocalAppServer::bind_port_with_data_dir(0, tmp_dir.clone()).unwrap();
    let port = server.port();
    let token = server.session_token();

    let payload = serde_json::json!({
        "probe": [0.5, -0.25],
        "loopback_rec": [0.0, 0.5, -0.25],
        "reamp_rec": [0.1, 0.2, 0.3, 0.4],
        "di_len": 4,
        "session_name": "take_aud01"
    })
    .to_string();

    let (code, body) = http_req(
        port,
        "POST",
        "/api/reamp/run",
        &[
            ("Origin", &format!("http://127.0.0.1:{}", port)),
            ("X-Sonix-Token", &token),
            ("Content-Type", "application/json"),
        ],
        &payload,
    );
    assert_eq!(code, 200, "AUD-01 reamp validado falhou: {} {}", code, body);
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    let session_file = parsed["session_file"]
        .as_str()
        .expect("Resposta deve conter session_file");
    let expected_parent = tmp_dir.join("reamp");
    assert!(
        std::path::Path::new(session_file).starts_with(&expected_parent),
        "session_file {:?} deveria ficar dentro de {:?}",
        session_file,
        expected_parent
    );
    let saved = std::fs::read_to_string(session_file).expect("session_file deveria existir");
    let meta: sonix_desktop::reamp::ReampSessionMetadata =
        serde_json::from_str(&saved).expect("JSON de metadados válido");
    assert_eq!(meta.measured_delay_samples, 1);

    let _ = std::fs::remove_dir_all(&tmp_dir);
}
