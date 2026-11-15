//! Testes de contrato da Spec 047 — Bancada e Osciloscópio com Dados do Motor
//! Cobre CA-1 a CA-7.

use ampforge_dsp::modshop::ToneStackTopology;
use ampforge_dsp::tonestack::analog_tonestack_magnitude_db;
use sonix_desktop::audio::RealtimeDspChain;
use sonix_desktop::server::LocalAppServer;
use std::f32::consts::PI;
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
    let body_idx = resp_str.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
    (status_code, resp_str[body_idx..].to_string())
}

fn post_params(port: u16, token: &str, payload: serde_json::Value) {
    let (status, body) = http_request(
        port,
        "POST",
        "/api/params",
        &[
            ("Content-Type", "application/json"),
            ("X-Sonix-Token", token),
        ],
        &payload.to_string(),
    );
    assert_eq!(status, 200, "POST /api/params falhou: {body}");
}

fn get_json(port: u16, path: &str) -> serde_json::Value {
    let (status, body) = http_request(port, "GET", path, &[], "");
    assert_eq!(status, 200, "GET {path} falhou: {body}");
    serde_json::from_str(&body).expect("JSON inválido")
}

fn pearson_corr(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let ma = a[..n].iter().sum::<f32>() / n as f32;
    let mb = b[..n].iter().sum::<f32>() / n as f32;
    let mut num = 0.0f64;
    let mut da = 0.0f64;
    let mut db = 0.0f64;
    for i in 0..n {
        let xa = a[i] as f64 - ma as f64;
        let xb = b[i] as f64 - mb as f64;
        num += xa * xb;
        da += xa * xa;
        db += xb * xb;
    }
    let den = (da * db).sqrt();
    if den < 1e-12 {
        0.0
    } else {
        (num / den) as f32
    }
}

#[test]
fn test_ca_01_scope_transfer_reflects_preamp_tube_and_dsp_curve() {
    let dsp = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let srv = LocalAppServer::bind_port_with_dsp(0, dsp).unwrap();
    let port = srv.port();
    let tok = srv.session_token().to_string();

    post_params(
        port,
        &tok,
        serde_json::json!({ "amp_1.preamp_tube": "12ax7" }),
    );
    let ax7 = get_json(port, "/api/scope?mode=transfer");
    let pts_ax7 = ax7["transfer"].as_array().expect("transfer array");
    assert!(pts_ax7.len() >= 21);

    post_params(
        port,
        &tok,
        serde_json::json!({ "amp_1.preamp_tube": "12at7" }),
    );
    let at7 = get_json(port, "/api/scope?mode=transfer");
    let pts_at7 = at7["transfer"].as_array().expect("transfer array");

    let max_diff = pts_ax7
        .iter()
        .zip(pts_at7.iter())
        .map(|(a, b)| {
            (a["y"].as_f64().unwrap() - b["y"].as_f64().unwrap()).abs()
        })
        .fold(0.0f64, f64::max);

    assert!(
        max_diff > 0.05,
        "CA-1: Mudar preamp_tube 12ax7 -> 12at7 precisa alterar a curva de transferência (max_diff={max_diff})"
    );
}

#[test]
fn test_ca_02_and_07_scope_waveform_real_audio_and_no_signal() {
    let dsp = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let srv = LocalAppServer::bind_port_with_dsp(0, dsp.clone()).unwrap();
    let port = srv.port();

    // CA-7: Sem áudio chegando, não pode retornar senoide sintética
    let empty_wf = get_json(port, "/api/scope?mode=waveform");
    let empty_di: Vec<f32> = empty_wf["waveform"]["di"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();
    let max_empty = empty_di.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    assert!(
        max_empty < 1e-5,
        "CA-7: Sem sinal no motor, CRT waveform deve ficar plano (max={max_empty}), nunca senoide sintética"
    );

    // CA-2: Processar riff real pelo motor e verificar correlação > 0.99
    let n = 512;
    let input: Vec<f32> = (0..n)
        .map(|i| 0.35 * (2.0 * PI * 220.0 * i as f32 / 48000.0).sin())
        .collect();
    let mut output = vec![0.0f32; n];
    {
        let mut chain = dsp.lock().unwrap();
        chain.process(&input, &mut output);
    }

    let live_wf = get_json(port, "/api/scope?mode=waveform");
    let wf_out: Vec<f32> = live_wf["waveform"]["output"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();
    let tail_start = n - wf_out.len();
    let corr = pearson_corr(&wf_out, &output[tail_start..]);
    assert!(
        corr > 0.99,
        "CA-2: CRT waveform output deve bater com saída real do motor (corr={corr:.4})"
    );
}

#[test]
fn test_ca_03_scope_tonestack_matches_analog_magnitude_db() {
    let dsp = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let srv = LocalAppServer::bind_port_with_dsp(0, dsp).unwrap();
    let port = srv.port();
    let tok = srv.session_token().to_string();

    post_params(
        port,
        &tok,
        serde_json::json!({
            "amp_1.model": "twin_ab763",
            "amp_1.bass": 0.0,
            "amp_1.middle": 5.0,
            "amp_1.treble": 5.0
        }),
    );
    let ts_low = get_json(port, "/api/scope?mode=tonestack");
    let pts_low = ts_low["tonestack"].as_array().unwrap();

    post_params(
        port,
        &tok,
        serde_json::json!({
            "amp_1.bass": 10.0
        }),
    );
    let ts_high = get_json(port, "/api/scope?mode=tonestack");
    let pts_high = ts_high["tonestack"].as_array().unwrap();

    // Encontrar ponto mais próximo de 100 Hz
    let idx_100 = pts_low
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let da = (a["f"].as_f64().unwrap() - 100.0).abs();
            let db = (b["f"].as_f64().unwrap() - 100.0).abs();
            da.partial_cmp(&db).unwrap()
        })
        .unwrap()
        .0;

    let db_low = pts_low[idx_100]["db"].as_f64().unwrap();
    let db_high = pts_high[idx_100]["db"].as_f64().unwrap();
    assert!(
        db_high - db_low >= 6.0,
        "CA-3: Bass 0->10 precisa subir >= 6 dB em ~100 Hz (subiu {:.2} dB)",
        db_high - db_low
    );

    // Verificar exatidão <= 0.01 dB contra analog_tonestack_magnitude_db
    for p in pts_high {
        let f = p["f"].as_f64().unwrap();
        let db = p["db"].as_f64().unwrap();
        let expected = analog_tonestack_magnitude_db(
            ToneStackTopology::FenderTmbMidScoop,
            10.0,
            5.0,
            5.0,
            f,
        );
        assert!(
            (db - expected).abs() <= 0.01,
            "CA-3: Ponto f={f:.1} Hz divergiu: api={db:.4} dB vs dsp={expected:.4} dB"
        );
    }
}

#[test]
fn test_ca_04_scope_speaker_z_changes_with_cab_speaker() {
    let dsp = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let srv = LocalAppServer::bind_port_with_dsp(0, dsp).unwrap();
    let port = srv.port();
    let tok = srv.session_token().to_string();

    let peak_freq = |speaker: &str| -> f64 {
        post_params(port, &tok, serde_json::json!({ "cab_1.speaker": speaker }));
        let resp = get_json(port, "/api/scope?mode=tonestack");
        let z_pts = resp["speaker_z"].as_array().unwrap();
        z_pts
            .iter()
            .filter(|p| p["f"].as_f64().unwrap() < 400.0)
            .max_by(|a, b| {
                a["z"]
                    .as_f64()
                    .unwrap()
                    .partial_cmp(&b["z"].as_f64().unwrap())
                    .unwrap()
            })
            .unwrap()["f"]
            .as_f64()
            .unwrap()
    };

    let f_v30 = peak_freq("v30_4x12");
    let f_alnico = peak_freq("alnico_blue_2x12");
    assert!(
        (f_v30 - f_alnico).abs() >= 15.0,
        "CA-4: Pico de impedância deve acompanhar default_impedance_curve (v30={f_v30:.1} Hz, alnico={f_alnico:.1} Hz)"
    );
}

#[test]
fn test_ca_06_telemetry_b_plus_v_tracks_amp_sag() {
    let dsp = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let srv = LocalAppServer::bind_port_with_dsp(0, dsp.clone()).unwrap();
    let port = srv.port();
    let tok = srv.session_token().to_string();

    post_params(
        port,
        &tok,
        serde_json::json!({
            "amp_1.preamp_plate_v": 280.0,
            "amp_1.sag": 10.0,
            "amp_1.gain": 9.0,
            "amp_1.master": 9.0
        }),
    );

    let idle_snap = get_json(port, "/api/telemetry");
    let b_plus_idle = idle_snap["b_plus_v"]
        .as_f64()
        .expect("CA-6: /api/telemetry deve expor b_plus_v");

    // Ataque forte contínuo (110 Hz) para provocar sag no B+
    let loud: Vec<f32> = (0..4096)
        .map(|i| 0.95 * (2.0 * PI * 110.0 * i as f32 / 48000.0).sin())
        .collect();
    let mut out = vec![0.0f32; 4096];
    {
        let mut chain = dsp.lock().unwrap();
        for _ in 0..8 {
            chain.process(&loud, &mut out);
        }
    }

    let sagged_snap = get_json(port, "/api/telemetry");
    let b_plus_sagged = sagged_snap["b_plus_v"].as_f64().unwrap();
    assert!(
        b_plus_sagged < b_plus_idle - 5.0,
        "CA-6: Sob ataque forte B+ deve cair com o sag (idle={b_plus_idle:.1}V, sagged={b_plus_sagged:.1}V)"
    );
}
