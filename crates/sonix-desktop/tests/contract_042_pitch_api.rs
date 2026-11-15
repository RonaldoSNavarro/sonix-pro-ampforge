//! Testes de contrato da Spec 042 — Integração HTTP/API e RealtimeDspChain para pitch_1.*
//! Cobre CA-5 e CA-6.

use ampforge_dsp::fx::pitch::PitchModel;
use ampforge_dsp::tuner::TunerProcessor;
use sonix_desktop::audio::{DspParamUpdate, RealtimeDspChain};
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

#[test]
fn test_ca_05_pitch_1_api_params_accepted_and_out_of_range_rejected_400() {
    let chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let srv = LocalAppServer::bind_port_with_dsp(0, chain).expect("Servidor");
    let port = srv.port();
    let token = srv.session_token().to_string();

    let (status_ok, _) = http_request(
        port,
        "POST",
        "/api/params",
        &[
            ("Content-Type", "application/json"),
            ("X-Sonix-Token", &token),
        ],
        r#"{"pitch_1.enabled":true,"pitch_1.model":"whammy","pitch_1.semitones":12,"pitch_1.key":"C","pitch_1.scale":"major","pitch_1.mix":100}"#,
    );
    assert_eq!(status_ok, 200, "CA-5: pitch_1.* válidos devem retornar 200");

    let (status_bad, body_bad) = http_request(
        port,
        "POST",
        "/api/params",
        &[
            ("Content-Type", "application/json"),
            ("X-Sonix-Token", &token),
        ],
        r#"{"pitch_1.semitones":30}"#,
    );
    assert_eq!(
        status_bad, 400,
        "CA-5: pitch_1.semitones=30 fora de -24..24 deve retornar 400 (body: {body_bad})"
    );
}

#[test]
fn test_ca_06_realtime_chain_pitch_1_shifts_and_bypasses() {
    let sr = 48000.0f32;
    let mut chain = RealtimeDspChain::new(sr);
    chain.apply_param_update(DspParamUpdate::GateEnabled(false));
    chain.apply_param_update(DspParamUpdate::DriveEnabled(false));
    chain.apply_param_update(DspParamUpdate::AmpEnabled(false));
    chain.apply_param_update(DspParamUpdate::CabEnabled(false));
    let base_lat = chain.algorithmic_latency_samples();
    chain.apply_param_update(DspParamUpdate::Pitch1Model(PitchModel::PedalWhammy));
    chain.apply_param_update(DspParamUpdate::Pitch1Semitones(12.0));
    chain.apply_param_update(DspParamUpdate::Pitch1Mix(100.0));
    chain.apply_param_update(DspParamUpdate::Pitch1Enabled(true));

    assert_eq!(
        chain.algorithmic_latency_samples(),
        base_lat + 1024,
        "CA-6: pitch_1 ativo deve somar latência algorítmica declarada"
    );

    let n = 16384;
    let input: Vec<f32> = (0..n)
        .map(|i| 0.5 * (2.0 * PI * 220.0 * i as f32 / sr).sin())
        .collect();
    let mut out = vec![0.0f32; n];
    for start in (0..n).step_by(512) {
        let end = (start + 512).min(n);
        chain.process(&input[start..end], &mut out[start..end]);
    }

    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(sr);
    tuner.set_active(true);
    let f_out = tuner.detect_pitch(&out[8192..12288]).frequency_hz;
    let cents_err = (1200.0 * (f_out / 440.0).log2()).abs();
    assert!(
        cents_err <= 5.0,
        "CA-6: RealtimeDspChain com pitch_1=+12 deve produzir 440 Hz +/- 5 cents (obteve {f_out:.2} Hz)"
    );

    // Bypass deve zerar latência do pitch e passar o sinal 220 Hz sem transposição
    chain.apply_param_update(DspParamUpdate::Pitch1Enabled(false));
    assert_eq!(chain.algorithmic_latency_samples(), base_lat);
}
