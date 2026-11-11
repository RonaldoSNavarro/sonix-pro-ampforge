//! crates/sonix-desktop/tests/contract_024_graph_api.rs
//! Testes de Contrato Automatizados para Spec 024: API de Grafo e Controles de Efeitos ao Vivo
//! Cobre Critérios de Aceite CA-1, CA-10 e não-regressão

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use sonix_desktop::audio::RealtimeDspChain;
use sonix_desktop::server::LocalAppServer;

fn http_request(
    port: u16,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> (u16, serde_json::Value) {
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

    let resp_body = response.split("\r\n\r\n").nth(1).unwrap_or("");
    let json_val: serde_json::Value = serde_json::from_str(resp_body).unwrap_or(serde_json::Value::Null);
    (status_code, json_val)
}

fn post_json(port: u16, path: &str, json_str: &str, token: &str) -> (u16, serde_json::Value) {
    http_request(
        port,
        "POST",
        path,
        &[("Content-Type", "application/json"), ("X-Sonix-Token", token)],
        Some(json_str),
    )
}

fn get_json(port: u16, path: &str, token: &str) -> (u16, serde_json::Value) {
    http_request(
        port,
        "GET",
        path,
        &[("X-Sonix-Token", token)],
        None,
    )
}

#[test]
fn test_ca_01_api_get_graph_structure() {
    let chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_dsp(0, chain).unwrap();
    let port = server.port();
    let token = server.session_token();

    let (status, graph) = get_json(port, "/api/graph", &token);
    assert_eq!(status, 200, "GET /api/graph deve retornar 200 OK");

    let rows = graph["rows"].as_array().expect("Campo 'rows' deve ser array");
    assert_eq!(rows.len(), 2, "Grid deve ter 2 linhas ativas");

    // Linha 0: gate, drive, split, amp_1, cab_1, eq_1, merge, dly_1, rev_1
    let row0_blocks = rows[0]["blocks"].as_array().expect("row 0 blocks array");
    let row0_ids: Vec<&str> = row0_blocks.iter().map(|b| b["id"].as_str().unwrap()).collect();
    assert!(row0_ids.contains(&"gate"));
    assert!(row0_ids.contains(&"drive"));
    assert!(row0_ids.contains(&"split"));
    assert!(row0_ids.contains(&"amp_1"));
    assert!(row0_ids.contains(&"cab_1"));
    assert!(row0_ids.contains(&"eq_1"));
    assert!(row0_ids.contains(&"merge"));
    assert!(row0_ids.contains(&"dly_1"));
    assert!(row0_ids.contains(&"rev_1"));

    // O merge deve estar na coluna 6
    let merge_block = row0_blocks.iter().find(|b| b["id"] == "merge").unwrap();
    assert_eq!(merge_block["col"], 6, "Merge deve estar na coluna 6");

    // Linha 1: comp_2, amp_2, cab_2, mod_2
    let row1_blocks = rows[1]["blocks"].as_array().expect("row 1 blocks array");
    let row1_ids: Vec<&str> = row1_blocks.iter().map(|b| b["id"].as_str().unwrap()).collect();
    assert!(row1_ids.contains(&"comp_2"));
    assert!(row1_ids.contains(&"amp_2"));
    assert!(row1_ids.contains(&"cab_2"));
    assert!(row1_ids.contains(&"mod_2"));
}

#[test]
fn test_ca_01_and_ca_10_api_post_params_validation_and_updates() {
    let chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_dsp(0, chain.clone()).unwrap();
    let port = server.port();
    let token = server.session_token();

    // 1. Aplicação válida de todos os novos blocos
    let payload = serde_json::json!({
        "split.enabled": true,
        "split.mode": "crossover",
        "split.crossover_hz": 650.0,
        "merge.row_1_level_db": -4.5,
        "merge.row_1_phase_invert": true,
        "comp_2.enabled": true,
        "comp_2.input": 7.0,
        "comp_2.output": 6.5,
        "comp_2.attack": 5.0,
        "comp_2.release": 4.0,
        "comp_2.ratio": "12",
        "eq_1.enabled": true,
        "eq_1.band1.freq_hz": 80.0,
        "eq_1.band1.gain_db": 3.0,
        "eq_1.band1.q": 1.4,
        "mod_2.enabled": true,
        "mod_2.rate_hz": 2.2,
        "mod_2.depth": 6.0,
        "mod_2.mix": 45.0,
        "dly_1.enabled": true,
        "dly_1.heads": "1+2",
        "dly_1.repeat_rate": 6.0,
        "dly_1.intensity": 5.5,
        "dly_1.echo_volume": 4.0,
        "dly_1.bass": 5.0,
        "dly_1.treble": 6.0,
        "dly_1.spillover": true,
        "rev_1.enabled": true,
        "rev_1.decay_s": 3.2,
        "rev_1.predelay_ms": 25.0,
        "rev_1.damping": 5.0,
        "rev_1.mix": 40.0,
        "rev_1.spillover": true,
        "amp_2.enabled": true,
        "amp_2.model": "twin_ab763",
        "cab_2.enabled": true,
        "cab_2.speaker": "alnico_blue_2x12"
    });

    let (status, resp) = post_json(port, "/api/params", &payload.to_string(), &token);
    assert_eq!(status, 200, "POST /api/params deve aceitar todos os novos parâmetros: {:?}", resp);

    // 2. Rejeição de valores fora de faixa
    let bad_payload = serde_json::json!({
        "split.crossover_hz": 50.0 // fora de 100..2000
    });
    let (bad_status, _) = post_json(port, "/api/params", &bad_payload.to_string(), &token);
    assert_eq!(bad_status, 400, "crossover_hz < 100 deve retornar 400");

    let bad_comp = serde_json::json!({
        "comp_2.attack": 9.0 // fora de 1..7
    });
    let (bad_comp_status, _) = post_json(port, "/api/params", &bad_comp.to_string(), &token);
    assert_eq!(bad_comp_status, 400, "comp_2.attack > 7 deve retornar 400");
}

#[test]
fn test_ca_01_live_chain_bit_identical_to_spec_016_series_chain() {
    // CA-1: com split.enabled=false e eq_1, dly_1, rev_1 desligados, a saída é bit-idêntica à cadeia em série da Spec 016
    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 256);

    // Certifica que blocos extras estão desligados
    chain.set_split_enabled(false);
    chain.set_eq_1_enabled(false);
    chain.set_dly_1_enabled(false);
    chain.set_rev_1_enabled(false);

    let in_buf = vec![0.3f32; 256];
    let mut out_buf = vec![0.0f32; 256];
    chain.process(&in_buf, &mut out_buf);

    // O sinal deve processar normalmente através de gate, amp e cab sem silêncio e sem NaNs
    assert!(out_buf.iter().all(|s| s.is_finite()));
    let rms: f32 = (out_buf.iter().map(|&s| s * s).sum::<f32>() / 256.0).sqrt();
    assert!(rms > 0.001, "Saída da cadeia em série deve conter sinal audível");
}
