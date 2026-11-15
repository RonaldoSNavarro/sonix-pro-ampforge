//! Testes de contrato API/Desktop da Spec 038 — CA-7 (WAV estéreo channels_used) e CA-9 (WAV truncado 400)

use sonix_desktop::audio::RealtimeDspChain;
use sonix_desktop::server::LocalAppServer;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

fn http_raw(port: u16, method: &str, path: &str, token: &str, body: &[u8]) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let head = format!(
        "{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Sonix-Token: {}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        method,
        path,
        port,
        token,
        body.len()
    );
    stream.write_all(head.as_bytes()).unwrap();
    stream.write_all(body).unwrap();
    let mut resp = String::new();
    stream.read_to_string(&mut resp).unwrap();
    let code = resp
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|c| c.parse::<u16>().ok())
        .unwrap_or(0);
    let resp_body = resp.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    (code, resp_body)
}

fn make_stereo_wav(sr: u32, frames: usize) -> Vec<u8> {
    let data_len = (frames * 2 * 2) as u32;
    let riff_len = 36 + data_len;
    let mut b = Vec::with_capacity(44 + data_len as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&riff_len.to_le_bytes());
    b.extend_from_slice(b"WAVE");
    b.extend_from_slice(b"fmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes()); // stereo
    b.extend_from_slice(&sr.to_le_bytes());
    b.extend_from_slice(&(sr * 4).to_le_bytes());
    b.extend_from_slice(&4u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for i in 0..frames {
        let l = if i == 0 { 16384i16 } else { 0i16 };
        let r = if i == 10 { 16384i16 } else { 0i16 };
        b.extend_from_slice(&l.to_le_bytes());
        b.extend_from_slice(&r.to_le_bytes());
    }
    b
}

#[test]
fn ca7_and_ca9_stereo_wav_returns_channels_used_and_truncated_returns_400() {
    let chain = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    let server = LocalAppServer::bind_port_with_dsp(0, chain).unwrap();
    let port = server.port();
    let token = server.session_token();

    let stereo = make_stereo_wav(48000, 256);
    let (code, body) = http_raw(port, "POST", "/api/cab/ir", &token, &stereo);
    assert_eq!(code, 200, "Upload estéreo falhou: {}", body);
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["channels_used"], 1, "CA-7: channels_used deve ser informado");
    assert_eq!(v["source_channels"], 2, "CA-7: source_channels deve ser 2");

    // CA-9: WAV truncado devolve 400
    let (code_bad, _) = http_raw(port, "POST", "/api/cab/ir", &token, &stereo[..28]);
    assert_eq!(code_bad, 400, "CA-9: WAV truncado deve devolver 400");
}
