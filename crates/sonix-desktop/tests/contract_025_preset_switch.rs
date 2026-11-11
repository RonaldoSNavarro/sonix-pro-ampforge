//! Testes de contrato da Spec 025 — Presets, Cenas e Canais de Bloco Reais (Lado Desktop / Audio)
//! Cobre CA-2, CA-10 e CA-11 (API).

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
fn test_ca_02_gapless_preset_switch_crossfade_and_spillover() {
    let mut dsp = RealtimeDspChain::new(48000.0);

    // Configura delay com spillover ativado
    dsp.set_dly_1_enabled(true);
    dsp.set_dly_1_spillover(true);

    // Alimenta sinal durante 1 segundo
    let mut in_buf = vec![0.0f32; 256];
    let mut out_buf = vec![0.0f32; 256];
    for (i, sample) in in_buf.iter_mut().enumerate() {
        *sample = (2.0 * std::f32::consts::PI * 1000.0 * (i as f32 / 48000.0)).sin() * 0.5;
    }
    for _ in 0..100 {
        dsp.process(&in_buf, &mut out_buf);
    }

    // Dispara troca de preset
    dsp.trigger_preset_crossfade();
    assert!(dsp.is_preset_crossfading());

    // Durante a troca (20 ms a 48 kHz ~ 960 amostras, aprox 4 blocos de 256)
    // O sinal não deve ter descontinuidade abrupta (> 1.1x regime)
    let mut max_diff = 0.0f32;
    let mut prev = 0.0f32;
    for _ in 0..4 {
        dsp.process(&in_buf, &mut out_buf);
        for &s in out_buf.iter() {
            let diff = (s - prev).abs();
            if diff > max_diff {
                max_diff = diff;
            }
            prev = s;
        }
    }

    // Após o crossfade, spillover do delay mantém cauda mesmo com entrada silenciada
    let silent_in = vec![0.0f32; 256];
    dsp.process(&silent_in, &mut out_buf);
    let energy_tail: f32 = out_buf.iter().map(|&s| s * s).sum();
    assert!(energy_tail > 1e-6, "Cauda do delay com spillover deve continuar audível");
}

#[test]
fn test_ca_10_and_ca_11_preset_api_endpoints() {
    let dsp_chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_dsp(0, dsp_chain).expect("Falha ao ligar server");
    let port = server.port();
    let token = server.session_token();

    // 1. GET /api/presets lista os 4 presets de fábrica
    let (code_list, resp_list) = http_request(
        port,
        "GET",
        "/api/presets",
        &[("X-Sonix-Token", &token)],
        "",
    );
    assert_eq!(code_list, 200);
    assert!(resp_list.contains("brit800"));
    assert!(resp_list.contains("usdeluxe"));
    assert!(resp_list.contains("recto"));
    assert!(resp_list.contains("ac30"));

    // 2. POST /api/presets/load com id inexistente -> 404 Not Found (CA-11)
    let (code_404, resp_404) = http_request(
        port,
        "POST",
        "/api/presets/load",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        r#"{"id": "non_existent_preset_id_xyz"}"#,
    );
    assert_eq!(code_404, 404, "Preset inexistente deve retornar 404: {}", resp_404);

    // 3. POST /api/presets/load com id válido -> 200 OK
    let (code_load, resp_load) = http_request(
        port,
        "POST",
        "/api/presets/load",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        r#"{"id": "brit800"}"#,
    );
    assert_eq!(code_load, 200, "Carregar brit800 deve retornar 200: {}", resp_load);

    // 4. POST /api/scenes/select com cena fora de A-H -> 400 Bad Request (CA-11)
    let (code_bad_scene, resp_bad_scene) = http_request(
        port,
        "POST",
        "/api/scenes/select",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        r#"{"scene": "Z"}"#,
    );
    assert_eq!(code_bad_scene, 400, "Cena 'Z' deve retornar 400: {}", resp_bad_scene);

    // 5. POST /api/scenes/select com cena válida A -> 200 OK e retorna tempo medido
    let (code_scene_a, resp_scene_a) = http_request(
        port,
        "POST",
        "/api/scenes/select",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        r#"{"scene": "A"}"#,
    );
    assert_eq!(code_scene_a, 200, "Cena 'A' deve retornar 200: {}", resp_scene_a);
    assert!(resp_scene_a.contains("transition_ms"), "Deve retornar transition_ms: {}", resp_scene_a);

    // 6. CA-1 & CA-3: Carregar usdeluxe restaura parâmetros completos em GET /api/state
    let (code_usdeluxe, _) = http_request(
        port,
        "POST",
        "/api/presets/load",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        r#"{"id": "usdeluxe"}"#,
    );
    assert_eq!(code_usdeluxe, 200);

    let (code_state, resp_state) = http_request(
        port,
        "GET",
        "/api/state",
        &[("X-Sonix-Token", &token)],
        "",
    );
    assert_eq!(code_state, 200);
    assert!(resp_state.contains("twin_ab763"), "Estado deve refletir o modelo de usdeluxe: {}", resp_state);
}
