//! Testes de contrato da Spec 045 — Cenas, Presets e Grafo Reais
//! Cobre CA-1 a CA-7.

use sonix_core::persistence::SonixPresetDoc;
use sonix_desktop::audio::RealtimeDspChain;
use sonix_desktop::server::LocalAppServer;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

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

fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

/// Calcula THD aproximado para uma senoide de frequência `f0` via DFT dos harmônicos 1..6.
fn compute_thd(samples: &[f32], sample_rate: f32, f0: f32) -> f32 {
    let n = samples.len();
    let dft_mag = |freq: f32| -> f32 {
        let mut re = 0.0f64;
        let mut im = 0.0f64;
        for (i, &x) in samples.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * (freq as f64) * (i as f64) / (sample_rate as f64);
            re += (x as f64) * angle.cos();
            im -= (x as f64) * angle.sin();
        }
        ((re * re + im * im).sqrt() / n as f64) as f32
    };
    let h1 = dft_mag(f0).max(1e-9);
    let mut harm_sq = 0.0f32;
    for k in 2..=6 {
        let hk = dft_mag(f0 * k as f32);
        harm_sq += hk * hk;
    }
    harm_sq.sqrt() / h1
}

fn remove_saved_preset(id: &str) {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = std::path::PathBuf::from(appdata)
            .join("SonixPro")
            .join("presets")
            .join(format!("{}.sonixpreset", id));
        let _ = std::fs::remove_file(p);
    }
    let p2 = std::path::PathBuf::from("presets/user").join(format!("{}.sonixpreset", id));
    let _ = std::fs::remove_file(p2);
}

#[test]
fn ca1_scene_switch_changes_output_thd() {
    let sr = 48000.0f32;
    let chain = Arc::new(Mutex::new(RealtimeDspChain::new(sr)));
    let server = LocalAppServer::bind_port_with_dsp(0, chain.clone()).expect("Falha ao ligar server");
    let port = server.port();
    let token = server.session_token();

    // Importa preset onde Cena A tem drive ativo e Cena B tem drive em bypass, amp limpo
    let preset_json = serde_json::json!({
        "$schema": "https://sonixpro.local/schemas/sonixpreset-v1.json",
        "schemaVersion": "1.1.0",
        "metadata": {
            "id": "ca1_scene_thd",
            "name": "CA1 Scene THD",
            "author": "Test",
            "category": "Test",
            "tempoBpm": 120.0,
            "createdAt": "2026-11-14T00:00:00Z",
            "footswitchMode": "scenes"
        },
        "activeSceneIndex": 0,
        "scenes": [
            { "index": 0, "id": "scene_0", "name": "Dirty", "color": "#ff0000" },
            { "index": 1, "id": "scene_1", "name": "Clean", "color": "#00ff00" }
        ],
        "grid": {
            "rows": 4,
            "cols": 12,
            "blocks": [
                {
                    "instanceId": "drive",
                    "type": "drive",
                    "row": 0,
                    "col": 1,
                    "oversampling": "1x",
                    "activeChannelByScene": [0, 0, 0, 0, 0, 0, 0, 0],
                    "bypassByScene": [false, true, false, false, false, false, false, false]
                }
            ]
        },
        "modifiers": [],
        "params": {
            "gate.enabled": false,
            "drive.enabled": true,
            "drive.drive": 9.5,
            "drive.level": 7.0,
            "amp_1.model": "twin_ab763",
            "amp_1.gain": 1.5,
            "amp_1.master": 6.0,
            "cab_1.enabled": false
        }
    });

    let (code, resp) = http_request(
        port,
        "POST",
        "/api/presets/import",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        &preset_json.to_string(),
    );
    assert_eq!(code, 200, "Import falhou: {}", resp);

    // Senoide 1 kHz contínua
    let block_len = 256usize;
    let mut in_buf = vec![0.0f32; block_len];
    let mut out_buf = vec![0.0f32; block_len];
    let mut phase = 0usize;
    let mut fill_sine = |buf: &mut [f32]| {
        for s in buf.iter_mut() {
            *s = 0.25 * (2.0 * std::f32::consts::PI * 1000.0 * (phase as f32 / sr)).sin();
            phase += 1;
        }
    };

    // Estabiliza Cena A
    for _ in 0..20 {
        fill_sine(&mut in_buf);
        chain.lock().unwrap().process(&in_buf, &mut out_buf);
    }
    let mut steady_a = vec![0.0f32; 480]; // 10 ciclos exatos de 1 kHz a 48 kHz
    let mut in_480 = vec![0.0f32; 480];
    fill_sine(&mut in_480);
    chain.lock().unwrap().process(&in_480, &mut steady_a);
    let thd_a = compute_thd(&steady_a, sr, 1000.0);
    assert!(thd_a > 0.05, "Cena A com drive deveria ter THD relevante, obteve {}", thd_a);

    // Seleciona Cena B (drive em bypass)
    let (code_sc, resp_sc) = http_request(
        port,
        "POST",
        "/api/scenes/select",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        r#"{"scene":"B"}"#,
    );
    assert_eq!(code_sc, 200, "scenes/select falhou: {}", resp_sc);

    // Aguarda 1 bloco + 20 ms (~1216 amostras = 5 blocos de 256)
    for _ in 0..5 {
        fill_sine(&mut in_buf);
        chain.lock().unwrap().process(&in_buf, &mut out_buf);
    }

    let mut steady_b = vec![0.0f32; 480];
    fill_sine(&mut in_480);
    chain.lock().unwrap().process(&in_480, &mut steady_b);
    let thd_b = compute_thd(&steady_b, sr, 1000.0);

    assert!(
        thd_b <= thd_a * 0.5,
        "CA-1: Cena B com drive em bypass deve derrubar THD em >= 50% (thd_a={}, thd_b={})",
        thd_a,
        thd_b
    );
}

#[test]
fn ca2_scene_and_preset_switch_have_no_silence_hole() {
    let sr = 48000.0f32;
    let chain = Arc::new(Mutex::new(RealtimeDspChain::new(sr)));
    let server = LocalAppServer::bind_port_with_dsp(0, chain.clone()).expect("Falha ao ligar server");
    let port = server.port();
    let token = server.session_token();

    let mut phase = 0usize;
    let mut next_block = |n: usize| -> Vec<f32> {
        let mut v = vec![0.0f32; n];
        for s in v.iter_mut() {
            *s = 0.25 * (2.0 * std::f32::consts::PI * 500.0 * (phase as f32 / sr)).sin();
            phase += 1;
        }
        v
    };

    // Carrega brit800 e estabiliza
    let (code, _) = http_request(
        port,
        "POST",
        "/api/presets/load",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        r#"{"id":"brit800"}"#,
    );
    assert_eq!(code, 200);

    let mut out = vec![0.0f32; 256];
    for _ in 0..20 {
        let inp = next_block(256);
        chain.lock().unwrap().process(&inp, &mut out);
    }
    let rms_before = compute_rms(&out);
    assert!(rms_before > 1e-3, "RMS antes da troca deve ser audível: {}", rms_before);

    // Troca para ac30 durante a senoide
    let (code2, _) = http_request(
        port,
        "POST",
        "/api/presets/load",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        r#"{"id":"ac30"}"#,
    );
    assert_eq!(code2, 200);

    // Captura 40 ms (1920 amostras = 40 janelas de 1 ms) durante e após a troca
    let mut captured = Vec::with_capacity(1920);
    for _ in 0..15 {
        let inp = next_block(128);
        let mut o = vec![0.0f32; 128];
        chain.lock().unwrap().process(&inp, &mut o);
        captured.extend_from_slice(&o);
    }
    // Mais 10 blocos para estabilizar e medir rms_after
    for _ in 0..10 {
        let inp = next_block(256);
        chain.lock().unwrap().process(&inp, &mut out);
    }
    let rms_after = compute_rms(&out);
    let min_ref = rms_before.min(rms_after);
    assert!(min_ref > 1e-3);

    // Verifica todas as janelas de 1 ms (48 amostras)
    for (idx, win) in captured.chunks_exact(48).enumerate() {
        let w_rms = compute_rms(win);
        assert!(
            w_rms >= 0.5 * min_ref,
            "CA-2: Buraco detectado na janela {} ms: rms={} < 50% de min_ref={} (before={}, after={})",
            idx,
            w_rms,
            min_ref,
            rms_before,
            rms_after
        );
    }
}

#[test]
fn ca3_save_load_preserves_scenes_channels_and_active_scene() {
    let preset_id = "test_ca3_custom_save_load";
    remove_saved_preset(preset_id);

    let chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_dsp(0, chain).unwrap();
    let port = server.port();
    let token = server.session_token();

    let custom_preset = serde_json::json!({
        "$schema": "https://sonixpro.local/schemas/sonixpreset-v1.json",
        "schemaVersion": "1.1.0",
        "metadata": {
            "id": preset_id,
            "name": "Custom CA3",
            "author": "Tester",
            "category": "Custom",
            "tempoBpm": 128.0,
            "createdAt": "2026-11-14T12:00:00Z",
            "footswitchMode": "scenes"
        },
        "activeSceneIndex": 2,
        "scenes": [
            { "index": 0, "id": "scene_0", "name": "Intro", "color": "#112233" },
            { "index": 1, "id": "scene_1", "name": "Verse", "color": "#223344" },
            { "index": 2, "id": "scene_2", "name": "Chorus", "color": "#334455" },
            { "index": 3, "id": "scene_3", "name": "Solo", "color": "#445566" },
            { "index": 4, "id": "scene_4", "name": "Bridge", "color": "#556677" },
            { "index": 5, "id": "scene_5", "name": "Out", "color": "#667788" },
            { "index": 6, "id": "scene_6", "name": "Clean", "color": "#778899" },
            { "index": 7, "id": "scene_7", "name": "Lead", "color": "#8899aa" }
        ],
        "grid": {
            "rows": 4,
            "cols": 12,
            "blocks": [
                {
                    "instanceId": "amp_1",
                    "type": "amp",
                    "row": 0,
                    "col": 3,
                    "oversampling": "4x",
                    "activeChannelByScene": [0, 1, 2, 3, 0, 1, 2, 3],
                    "bypassByScene": [false, false, false, false, false, false, false, false]
                }
            ]
        },
        "modifiers": [],
        "params": {
            "amp_1.model": "plexi_1959",
            "amp_1.gain": 7.25,
            "scene.active": "C"
        },
        "channels": {
            "amp_1": {
                "A": { "amp_1.gain": 3.0 },
                "B": { "amp_1.gain": 5.5 },
                "C": { "amp_1.gain": 7.25 },
                "D": { "amp_1.gain": 9.0 }
            }
        }
    });

    let (c_imp, r_imp) = http_request(
        port,
        "POST",
        "/api/presets/import",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        &custom_preset.to_string(),
    );
    assert_eq!(c_imp, 200, "import falhou: {}", r_imp);

    let (_, st_before_str) = http_request(port, "GET", "/api/state", &[("X-Sonix-Token", &token)], "");
    let st_before: serde_json::Value = serde_json::from_str(&st_before_str).unwrap();

    // Salva
    let (c_save, r_save) = http_request(
        port,
        "POST",
        "/api/presets/save",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        &format!(r#"{{"id":"{}","name":"Custom CA3"}}"#, preset_id),
    );
    assert_eq!(c_save, 200, "save falhou: {}", r_save);

    // Altera estado para outro preset/cena
    let _ = http_request(
        port,
        "POST",
        "/api/presets/load",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        r#"{"id":"usdeluxe"}"#,
    );

    // Carrega o preset salvo
    let (c_load, r_load) = http_request(
        port,
        "POST",
        "/api/presets/load",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        &format!(r#"{{"id":"{}"}}"#, preset_id),
    );
    assert_eq!(c_load, 200, "load falhou: {}", r_load);

    let (_, st_after_str) = http_request(port, "GET", "/api/state", &[("X-Sonix-Token", &token)], "");
    let st_after: serde_json::Value = serde_json::from_str(&st_after_str).unwrap();

    remove_saved_preset(preset_id);

    assert_eq!(st_after["scene.active"], "C");
    assert_eq!(st_before["scenes"][0]["name"], "Intro");
    assert_eq!(st_before["channels"]["amp_1"]["B"]["amp_1.gain"], 5.5);
    assert_eq!(st_before["grid"]["blocks"][0]["instanceId"], "amp_1");
    assert_eq!(st_after["scenes"], st_before["scenes"]);
    assert_eq!(st_after["channels"], st_before["channels"]);
    assert_eq!(st_after["grid"], st_before["grid"]);
    assert_eq!(st_after, st_before);
}

#[test]
fn ca4_saved_preset_created_at_is_within_5s_of_system_clock() {
    let preset_id = "test_ca4_timestamp";
    remove_saved_preset(preset_id);

    let chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_dsp(0, chain).unwrap();
    let port = server.port();
    let token = server.session_token();

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let (c_save, r_save) = http_request(
        port,
        "POST",
        "/api/presets/save",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        &format!(r#"{{"id":"{}","name":"TS Test"}}"#, preset_id),
    );
    assert_eq!(c_save, 200, "save falhou: {}", r_save);

    let (c_exp, r_exp) = http_request(
        port,
        "GET",
        &format!("/api/presets/{}/export", preset_id),
        &[("X-Sonix-Token", &token)],
        "",
    );
    remove_saved_preset(preset_id);
    assert_eq!(c_exp, 200);

    let doc: serde_json::Value = serde_json::from_str(&r_exp).unwrap();
    let created_at = doc["metadata"]["createdAt"].as_str().unwrap_or("");
    // Parse YYYY-MM-DDTHH:MM:SSZ
    let parsed_secs = parse_iso8601_utc(created_at)
        .unwrap_or_else(|| panic!("createdAt inválido: {}", created_at));

    assert!(
        (parsed_secs - now_secs).abs() <= 5,
        "CA-4: createdAt '{}' ({}) difere mais de 5s de now ({})",
        created_at,
        parsed_secs,
        now_secs
    );
}

fn parse_iso8601_utc(s: &str) -> Option<i64> {
    let s = s.strip_suffix('Z')?;
    let (date, time) = s.split_once('T')?;
    let mut dp = date.split('-');
    let y: i64 = dp.next()?.parse().ok()?;
    let m: i64 = dp.next()?.parse().ok()?;
    let d: i64 = dp.next()?.parse().ok()?;
    let mut tp = time.split(':');
    let hh: i64 = tp.next()?.parse().ok()?;
    let mm: i64 = tp.next()?.parse().ok()?;
    let ss: i64 = tp.next()?.split('.').next()?.parse().ok()?;

    // Howard Hinnant days_from_civil
    let y_adj = if m <= 2 { y - 1 } else { y };
    let era = if y_adj >= 0 { y_adj } else { y_adj - 399 } / 400;
    let yoe = y_adj - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some(days * 86400 + hh * 3600 + mm * 60 + ss)
}

#[test]
fn ca5_graph_reflects_live_models_and_bypass() {
    let chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_dsp(0, chain).unwrap();
    let port = server.port();
    let token = server.session_token();

    let (c_p, r_p) = http_request(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        r#"{"amp_1.model":"ac30_tb","drive.enabled":false}"#,
    );
    assert_eq!(c_p, 200, "params falhou: {}", r_p);

    let (c_g, r_g) = http_request(port, "GET", "/api/graph", &[("X-Sonix-Token", &token)], "");
    assert_eq!(c_g, 200);
    let graph: serde_json::Value = serde_json::from_str(&r_g).unwrap();
    let blocks = graph["rows"][0]["blocks"].as_array().unwrap();

    let amp_block = blocks.iter().find(|b| b["id"] == "amp_1").unwrap();
    let drive_block = blocks.iter().find(|b| b["id"] == "drive").unwrap();

    let amp_name = amp_block["name"].as_str().unwrap_or("");
    assert!(
        amp_name.to_ascii_lowercase().contains("ac30"),
        "CA-5: nome de amp_1 deveria refletir AC30, obteve '{}'",
        amp_name
    );
    assert_eq!(
        drive_block["enabled"].as_bool(),
        Some(false),
        "CA-5: drive.enabled deveria ser false no /api/graph"
    );
}

#[test]
fn ca6_invalid_key_in_preset_returns_400_and_leaves_state_unchanged() {
    let chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_dsp(0, chain).unwrap();
    let port = server.port();
    let token = server.session_token();

    let _ = http_request(
        port,
        "POST",
        "/api/params",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        r#"{"amp_1.gain":7.5}"#,
    );
    let (_, st_before) = http_request(port, "GET", "/api/state", &[("X-Sonix-Token", &token)], "");

    let bad_preset = serde_json::json!({
        "$schema": "https://sonixpro.local/schemas/sonixpreset-v1.json",
        "schemaVersion": "1.1.0",
        "metadata": {
            "id": "bad_preset",
            "name": "Bad Preset",
            "author": "Test",
            "category": "Test",
            "tempoBpm": 120.0,
            "createdAt": "2026-11-14T00:00:00Z",
            "footswitchMode": "scenes"
        },
        "activeSceneIndex": 0,
        "scenes": [],
        "grid": { "rows": 4, "cols": 12, "blocks": [] },
        "modifiers": [],
        "params": {
            "amp_1.gain": 2.0,
            "chave.inexistente_no_meio": 999,
            "drive.drive": 8.5
        }
    });

    let (code, resp) = http_request(
        port,
        "POST",
        "/api/presets/import",
        &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
        &bad_preset.to_string(),
    );
    assert_eq!(code, 400, "CA-6: chave inválida deveria retornar 400: {}", resp);

    let (_, st_after) = http_request(port, "GET", "/api/state", &[("X-Sonix-Token", &token)], "");
    assert_eq!(st_after, st_before, "CA-6: estado não pode mudar quando preset é inválido");
}

#[test]
fn ca7_schema_migration_and_factory_presets_still_work() {
    let v100 = r#"{
        "$schema": "https://sonixpro.local/schemas/sonixpreset-v1.json",
        "schemaVersion": "1.0.0",
        "metadata": {
            "id": "legacy_100",
            "name": "Legacy 1.0.0",
            "author": "User",
            "category": "Rock",
            "tempoBpm": 120.0,
            "createdAt": "2026-11-11T00:00:00Z",
            "footswitchMode": "scenes"
        },
        "activeSceneIndex": 0,
        "scenes": [],
        "grid": { "rows": 4, "cols": 12, "blocks": [] },
        "modifiers": []
    }"#;
    let migrated = SonixPresetDoc::from_json_str(v100).expect("Migração 1.0.0 -> 1.1.0 falhou");
    assert_eq!(migrated.schema_version, "1.1.0");

    let chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_dsp(0, chain).unwrap();
    let port = server.port();
    let token = server.session_token();

    for id in ["brit800", "usdeluxe", "recto", "ac30"] {
        let (code, resp) = http_request(
            port,
            "POST",
            "/api/presets/load",
            &[("Content-Type", "application/json"), ("X-Sonix-Token", &token)],
            &format!(r#"{{"id":"{}"}}"#, id),
        );
        assert_eq!(code, 200, "Preset de fábrica '{}' falhou: {}", id, resp);
    }
}
