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

#[test]
fn aud_03_post_sem_content_length_411() {
    let server = LocalAppServer::bind_port(0).unwrap();
    let port = server.port();
    let token = server.session_token();

    let raw = format!(
        "POST /api/params HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nOrigin: http://127.0.0.1:{}\r\nX-Sonix-Token: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n",
        port, port, token
    );
    let (status, body) = send_raw(port, &raw);
    assert_eq!(
        status, 411,
        "AUD-03: POST sem Content-Length deve retornar 411 Length Required, retornou {} ({})",
        status, body
    );
}

#[test]
fn aud_04_dispatch_nunca_entra_em_panico() {
    use sonix_desktop::api::{dispatch, ApiContext, AppState};
    use sonix_desktop::audio::{LockFreeParamQueue, RealtimeDspChain, TelemetryBuffer};
    use std::sync::{Arc, Mutex};

    let mut chain = RealtimeDspChain::new(48000.0);
    chain.prepare(48000.0, 256);
    let dsp = Arc::new(Mutex::new(chain));
    let pq = Arc::new(LockFreeParamQueue::new());
    let ctx = ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(128)),
        Some(dsp),
        Some(pq),
        None,
        None,
    );

    let keys = [
        "amp_1.bass",
        "amp_1.bias_ma",
        "amp_1.bright",
        "amp_1.channel_mode",
        "amp_1.cut",
        "amp_1.depth",
        "amp_1.enabled",
        "amp_1.gain",
        "amp_1.impedance_ohms",
        "amp_1.input_jack",
        "amp_1.is_modded",
        "amp_1.jumpered",
        "amp_1.master",
        "amp_1.middle",
        "amp_1.model",
        "amp_1.nfb",
        "amp_1.overdrive",
        "amp_1.power",
        "amp_1.power_tube",
        "amp_1.preamp_plate_v",
        "amp_1.preamp_tube",
        "amp_1.presence",
        "amp_1.rectifier",
        "amp_1.reverb",
        "amp_1.sag",
        "amp_1.spongy",
        "amp_1.standby",
        "amp_1.tonestack",
        "amp_1.treble",
        "amp_1.triode_asym",
        "amp_1.vibrato_depth",
        "amp_1.vibrato_speed",
        "amp_1.volume_2",
        "amp_1.xformer_sat",
        "amp_2.bass",
        "amp_2.enabled",
        "amp_2.gain",
        "amp_2.master",
        "amp_2.middle",
        "amp_2.model",
        "amp_2.treble",
        "cab_1.enabled",
        "cab_1.hicut_khz",
        "cab_1.ir_enabled",
        "cab_1.mic",
        "cab_1.mic1.angle_deg",
        "cab_1.mic1.distance_cm",
        "cab_1.mic1.enabled",
        "cab_1.mic1.level_db",
        "cab_1.mic1.model",
        "cab_1.mic1.phase_invert",
        "cab_1.mic1.position",
        "cab_1.mic2.angle_deg",
        "cab_1.mic2.distance_cm",
        "cab_1.mic2.enabled",
        "cab_1.mic2.level_db",
        "cab_1.mic2.model",
        "cab_1.mic2.phase_invert",
        "cab_1.mic2.position",
        "cab_1.model",
        "cab_1.resonance_hz",
        "cab_1.speaker",
        "cab_2.enabled",
        "cab_2.hicut_khz",
        "cab_2.model",
        "cab_2.resonance_hz",
        "cab_2.speaker",
        "comp_2.attack",
        "comp_2.enabled",
        "comp_2.input",
        "comp_2.output",
        "comp_2.ratio",
        "comp_2.release",
        "dly_1.bass",
        "dly_1.echo_volume",
        "dly_1.enabled",
        "dly_1.feedback",
        "dly_1.heads",
        "dly_1.intensity",
        "dly_1.mix_pct",
        "dly_1.mode",
        "dly_1.repeat_rate",
        "dly_1.reverb_volume",
        "dly_1.spillover",
        "dly_1.time_ms",
        "dly_1.treble",
        "drive.bias",
        "drive.drive",
        "drive.enabled",
        "drive.level",
        "drive.model",
        "drive.tone",
        "engine.oversampling",
        "eq_1.band",
        "eq_1.bands_db",
        "eq_1.enabled",
        "eq_1.gain_db",
        "eq_1.input_gain_db",
        "eq_1.mode",
        "eq_1.output_level_db",
        "eq_1.volume_db",
        "footswitch.mode",
        "gate.attack_ms",
        "gate.enabled",
        "gate.hold_ms",
        "gate.hysteresis_db",
        "gate.key_hpf_hz",
        "gate.key_lpf_hz",
        "gate.mode",
        "gate.range_db",
        "gate.release_ms",
        "gate.threshold_db",
        "input.boost",
        "input.mode",
        "input.source",
        "looper.action",
        "looper.half_speed",
        "looper.position",
        "looper.reverse",
        "master.level_db",
        "merge.level_a_db",
        "merge.level_b_db",
        "merge.phase_invert_b",
        "merge.row_1_level_db",
        "merge.row_1_phase_invert",
        "mod_2.depth",
        "mod_2.enabled",
        "mod_2.mix",
        "mod_2.mix_pct",
        "mod_2.model",
        "mod_2.rate_hz",
        "palco.exp_1",
        "pitch_1.enabled",
        "pitch_1.key",
        "pitch_1.mix",
        "pitch_1.mix_pct",
        "pitch_1.model",
        "pitch_1.scale",
        "pitch_1.semitones",
        "preset.active",
        "preset.name",
        "rev_1.damping",
        "rev_1.decay_s",
        "rev_1.enabled",
        "rev_1.mix",
        "rev_1.mix_pct",
        "rev_1.predelay_ms",
        "rev_1.spillover",
        "scene.active",
        "split.balance",
        "split.crossover_hz",
        "split.enabled",
        "split.mode",
        "split.ratio",
        "tuner.enabled",
        "tuner.output",
        "tuner.ref_hz",
        "wah_1.enabled",
        "wah_1.mode",
        "wah_1.position",
        "wah_1.q",
    ];

    let adversarial_raw = [
        "true",
        "false",
        "0",
        "-1",
        "1e308",
        "-1e308",
        "\"x\"",
        "\"\"",
        "null",
        "[]",
        "{}",
        "\"NaN\"",
    ];

    for key in keys {
        for raw_val in adversarial_raw {
            let body = format!("{{\"{}\":{}}}", key, raw_val);
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                dispatch(&ctx, "/api/params", "POST", body.as_bytes())
            }));
            let (status, _, resp_bytes) = res.unwrap_or_else(|_| {
                panic!("AUD-04: dispatch entrou em pânico para {}", body);
            });
            assert!(
                (200..300).contains(&status) || (400..500).contains(&status),
                "AUD-04: status inválido {} para {}: {}",
                status,
                body,
                String::from_utf8_lossy(&resp_bytes)
            );
        }
    }
}

#[test]
fn aud_04_api_sem_unwrap() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().unwrap().parent().unwrap();
    let targets = [
        manifest_dir.join("src/api.rs"),
        root.join("apps/sonix-app/src-tauri/src/commands.rs"),
    ];

    let mut violations = Vec::new();
    for path in targets {
        let content = std::fs::read_to_string(&path).expect("ler fonte para lint");
        let prod_section = content.split("#[cfg(test)]").next().unwrap_or(&content);
        for (idx, line) in prod_section.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if line.contains(".unwrap(") || line.contains(".expect(") {
                violations.push(format!("{}:{}: {}", path.display(), idx + 1, trimmed));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "AUD-04: encontrados {} unwrap/expect em código não-teste:\n{}",
        violations.len(),
        violations.join("\n")
    );
}
