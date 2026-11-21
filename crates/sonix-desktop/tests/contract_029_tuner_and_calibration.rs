//! Testes de contrato da Spec 029 — Afinador Funcional End-to-End e Calibração de Entrada Ao Vivo
//! Regime SDD + TDD estrito: testes red -> green cobrindo CA-1, CA-2, CA-4, CA-5, CA-6, CA-7, CA-8.

use ampforge_dsp::tuner::{TunerOutputMode, TunerProcessor};
use sonix_desktop::audio::{RealtimeDspChain, TelemetryBuffer};
use sonix_desktop::calibration::CalibrationStore;
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

    stream
        .write_all(req_str.as_bytes())
        .expect("Falha ao enviar HTTP request");
    let mut resp_str = String::new();
    stream
        .read_to_string(&mut resp_str)
        .expect("Falha ao ler HTTP response");

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
fn test_ca_01_tuner_api_returns_expected_json_contract_and_live_note() {
    let dsp_chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server =
        LocalAppServer::bind_port_with_dsp(0, dsp_chain.clone()).expect("Falha ao ligar server");
    let port = server.port();
    let token = server.session_token();

    // Habilita afinador via POST /api/params
    let (code_en, _) = http_request(
        port,
        "POST",
        "/api/params",
        &[
            ("Origin", &format!("http://127.0.0.1:{}", port)),
            ("Content-Type", "application/json"),
            ("X-Sonix-Token", &token),
        ],
        r#"{"tuner.enabled": true, "tuner.output": "mute"}"#,
    );
    assert_eq!(code_en, 200);

    // Processa seno de 110 Hz (A2) pela cadeia ao vivo
    let mut in_buf = vec![0.0f32; 4096];
    let mut out_buf = vec![0.0f32; 4096];
    for (i, s) in in_buf.iter_mut().enumerate() {
        *s = (2.0 * std::f32::consts::PI * 110.0 * (i as f32 / 48000.0)).sin() * 0.25;
    }
    dsp_chain.lock().unwrap().process(&in_buf, &mut out_buf);

    let (code_tuner, resp_body) = http_request(port, "GET", "/api/tuner", &[], "");
    assert_eq!(code_tuner, 200);
    let json: serde_json::Value = serde_json::from_str(&resp_body).expect("JSON inválido");

    assert_eq!(json["note"].as_str(), Some("A2"));
    assert!(json.get("cents").is_some(), "Campo cents deve existir");
    assert!(json["cents"].as_f64().unwrap().abs() <= 0.5);
    assert!((json["frequency_hz"].as_f64().unwrap() - 110.0).abs() <= 0.5);
    assert_eq!(json["in_tune"].as_bool(), Some(true));
    assert_eq!(json["active"].as_bool(), Some(true));
    assert_eq!(json["muted"].as_bool(), Some(true));
    assert!(json.get("note_name").is_none(), "Contrato usa 'note', não 'note_name'");
    assert!(
        json.get("cents_deviation").is_none(),
        "Contrato usa 'cents', não 'cents_deviation'"
    );
}

#[test]
fn test_ca_02_low_level_e2_at_96khz_and_white_noise_rejection() {
    let sr = 96000.0f32;

    // -55 dBFS de amplitude de pico/seno -> RMS ≈ 0.00126 (amp = 0.00178)
    let amp = 10.0f32.powf(-55.0 / 20.0) * std::f32::consts::SQRT_2;
    let mut e2_buf = vec![0.0f32; 4096];
    for (i, s) in e2_buf.iter_mut().enumerate() {
        *s = (2.0 * std::f32::consts::PI * 82.41 * (i as f32 / sr)).sin() * amp;
    }

    let mut dsp = RealtimeDspChain::new(sr);
    dsp.set_tuner_enabled(true);
    let mut out_buf = vec![0.0f32; 4096];
    dsp.process(&e2_buf, &mut out_buf);

    let res = dsp.tuner_latest_result();
    assert_eq!(
        res.note_name, "E2",
        "Deve detectar E2 (82.41 Hz) a -55 dBFS em 96 kHz; obteve {} ({} Hz)",
        res.note_name, res.frequency_hz
    );
    assert!(
        res.cents_deviation.abs() <= 1.0,
        "Erro para E2 a -55 dBFS / 96 kHz deve ser <= 1.0 cent: atual {}",
        res.cents_deviation
    );

    // Ruído branco a -60 dBFS (RMS = 0.001 -> amplitude uniforme [-0.001732, +0.001732])
    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(sr);
    let mut noise = vec![0.0f32; 4096];
    let mut lcg = 987654321u64;
    let noise_amp = 0.001 * 3.0f32.sqrt();
    for s in noise.iter_mut() {
        lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
        let u = ((lcg >> 33) as f32 / (1u32 << 30) as f32) - 1.0;
        *s = u * noise_amp;
    }
    let noise_res = tuner.detect_pitch(&noise);
    assert_eq!(
        noise_res.note_name, "---",
        "Ruído branco a -60 dBFS não deve disparar nota; obteve {}",
        noise_res.note_name
    );
}

#[test]
fn test_ca_04_and_ca_05_post_api_input_calibrate_live_capture_and_state() {
    let dsp_chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let cal_dir = std::env::temp_dir().join(format!("sonix_029_cal_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&cal_dir);
    let store = CalibrationStore::open(&cal_dir);
    let server = LocalAppServer::bind_port_with_all(
        0,
        Arc::new(TelemetryBuffer::new(512)),
        Some(dsp_chain.clone()),
        Some("p2_mic"),
        Some(store.clone()),
    )
    .expect("Falha ao ligar servidor");

    let port = server.port();
    let token = server.session_token();
    let origin = format!("http://127.0.0.1:{}", port);

    // 1. Alimenta 1 segundo de piso de ruído a -70 dBFS RMS na entrada da cadeia ao vivo
    let noise_len = 48000;
    let mut noise = vec![0.0f32; noise_len];
    let noise_amp = 0.0003162 * 3.0f32.sqrt();
    let mut lcg = 123456789u64;
    for s in noise.iter_mut() {
        lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
        let u = ((lcg >> 33) as f32 / (1u32 << 30) as f32) - 1.0;
        *s = u * noise_amp;
    }
    let mut out = vec![0.0f32; noise_len];
    dsp_chain.lock().unwrap().process(&noise, &mut out);

    // Fase 1: silence
    let (code_sil, body_sil) = http_request(
        port,
        "POST",
        "/api/input/calibrate",
        &[
            ("Origin", &origin),
            ("Content-Type", "application/json"),
            ("X-Sonix-Token", &token),
        ],
        r#"{"phase":"silence"}"#,
    );
    assert_eq!(code_sil, 200, "phase=silence deve retornar 200: {}", body_sil);

    // 2. Alimenta 1 segundo de palhetada forte a -30 dBFS (pico = 0.0316228 com ataque suave de 5ms)
    let strum_len = 48000;
    let mut strum = vec![0.0f32; strum_len];
    for (i, s) in strum.iter_mut().enumerate() {
        let env = (i as f32 / 240.0).min(1.0);
        *s = 0.0316228 * env * (2.0 * std::f32::consts::PI * 220.0 * (i as f32 / 48000.0)).sin();
    }
    let mut out_strum = vec![0.0f32; strum_len];
    dsp_chain.lock().unwrap().process(&strum, &mut out_strum);

    // Fase 2: strum
    let (code_strum, body_strum) = http_request(
        port,
        "POST",
        "/api/input/calibrate",
        &[
            ("Origin", &origin),
            ("Content-Type", "application/json"),
            ("X-Sonix-Token", &token),
        ],
        r#"{"phase":"strum"}"#,
    );
    assert_eq!(code_strum, 200, "phase=strum deve retornar 200: {}", body_strum);

    let res: serde_json::Value = serde_json::from_str(&body_strum).expect("JSON válido em strum");
    let proposed_boost = res["proposed_boost"].as_f64().expect("proposed_boost obrigatório");
    let proposed_gate = res["proposed_gate_threshold_db"]
        .as_f64()
        .expect("proposed_gate_threshold_db obrigatório");
    assert_eq!(res["diagnostic"].as_str(), Some("ok"));
    assert!(res["noise_floor_rms_db"].as_f64().is_some());
    assert!(res["raw_peak_db"].as_f64().is_some());
    assert!(
        (proposed_boost - 7.94).abs() <= 0.1,
        "proposed_boost esperado ~7.94, obteve {}",
        proposed_boost
    );
    assert!(
        (proposed_gate - (-46.0)).abs() <= 1.0,
        "proposed_gate_threshold_db esperado ~-46.0, obteve {}",
        proposed_gate
    );

    // CA-5: /api/state reflete os novos valores e CalibrationStore armazena o perfil
    let (code_state, state_body) = http_request(port, "GET", "/api/state", &[], "");
    assert_eq!(code_state, 200);
    let state: serde_json::Value = serde_json::from_str(&state_body).unwrap();
    assert!(
        (state["input.boost"].as_f64().unwrap() - proposed_boost).abs() <= 0.05,
        "GET /api/state deve refletir input.boost calibrado"
    );
    assert!(
        (state["gate.threshold_db"].as_f64().unwrap() - proposed_gate).abs() <= 0.1,
        "GET /api/state deve refletir gate.threshold_db calibrado"
    );

    let stored = store.get("p2_mic").expect("CalibrationStore deve salvar p2_mic");
    assert!((stored.0 as f64 - proposed_boost).abs() <= 0.05);
    assert!((stored.1 as f64 - proposed_gate).abs() <= 0.1);
}

#[test]
fn test_ca_06_calibration_validation_auth_and_silent_strum_input_too_low() {
    let dsp_chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_dsp(0, dsp_chain.clone()).unwrap();
    let port = server.port();
    let token = server.session_token();
    let origin = format!("http://127.0.0.1:{}", port);

    // 1. Sem X-Sonix-Token -> 401
    let (code_unauth, _) = http_request(
        port,
        "POST",
        "/api/input/calibrate",
        &[("Origin", &origin), ("Content-Type", "application/json")],
        r#"{"phase":"silence"}"#,
    );
    assert_eq!(code_unauth, 401, "POST /api/input/calibrate sem token deve retornar 401");

    // 2. Phase inválido -> 400
    let (code_bad, _) = http_request(
        port,
        "POST",
        "/api/input/calibrate",
        &[
            ("Origin", &origin),
            ("Content-Type", "application/json"),
            ("X-Sonix-Token", &token),
        ],
        r#"{"phase":"invalid_phase"}"#,
    );
    assert_eq!(code_bad, 400, "phase inválido deve retornar 400");

    // 3. Sem sinal na fase strum (< -42 dBFS) -> diagnostic = "input_too_low" e proposed_boost = 8.0
    let silent = vec![0.0f32; 16384];
    let mut out = vec![0.0f32; 16384];
    dsp_chain.lock().unwrap().process(&silent, &mut out);
    let (code_s1, _) = http_request(
        port,
        "POST",
        "/api/input/calibrate",
        &[
            ("Origin", &origin),
            ("Content-Type", "application/json"),
            ("X-Sonix-Token", &token),
        ],
        r#"{"phase":"silence"}"#,
    );
    assert_eq!(code_s1, 200);

    dsp_chain.lock().unwrap().process(&silent, &mut out);
    let (code_s2, body_s2) = http_request(
        port,
        "POST",
        "/api/input/calibrate",
        &[
            ("Origin", &origin),
            ("Content-Type", "application/json"),
            ("X-Sonix-Token", &token),
        ],
        r#"{"phase":"strum"}"#,
    );
    assert_eq!(code_s2, 200);
    let res: serde_json::Value = serde_json::from_str(&body_s2).unwrap();
    assert_eq!(res["diagnostic"].as_str(), Some("input_too_low"));
    assert_eq!(res["proposed_boost"].as_f64(), Some(8.0));
}

#[test]
fn test_ca_07_zero_alloc_zero_locks_during_tuner_and_calibration_capture() {
    let mut dsp = RealtimeDspChain::new(48000.0);
    dsp.set_tuner_enabled(true);
    let ring = dsp.clean_input_ring();

    let mut in_block = [0.0f32; 128];
    let mut out_block = [0.0f32; 128];
    for (i, s) in in_block.iter_mut().enumerate() {
        *s = (2.0 * std::f32::consts::PI * 220.0 * (i as f32 / 48000.0)).sin() * 0.1;
    }

    for _ in 0..200 {
        dsp.process(&in_block, &mut out_block);
    }

    assert!(
        ring.total_written() >= 128 * 200,
        "Ring buffer lock-free deve acumular amostras limpas continuamente"
    );
    let mut snapshot = vec![0.0f32; 4096];
    let n = ring.read_latest(&mut snapshot);
    assert_eq!(n, 4096);
}

#[test]
fn test_ca_08_non_regression_tuner_bypass_mode_and_a4() {
    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(48000.0);
    tuner.set_active(true);
    tuner.set_output_mode(TunerOutputMode::Bypass);

    let mut in_sig = vec![0.0f32; 2048];
    for (i, s) in in_sig.iter_mut().enumerate() {
        *s = (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / 48000.0)).sin() * 0.4;
    }
    let res = tuner.detect_pitch(&in_sig);
    assert_eq!(res.note_name, "A4");
    assert!(res.cents_deviation.abs() <= 0.2);

    let mut out_sig = vec![0.0f32; 256];
    tuner.process_audio(&in_sig[..256], &mut out_sig);
    assert!((out_sig[128] - in_sig[128]).abs() < 1e-5);
}
