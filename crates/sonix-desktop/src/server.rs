//! crates/sonix-desktop/src/server.rs
//! Servidor Local IPC/HTTP Isolado em Loopback (127.0.0.1) e Lançador Desktop App Mode (Spec 013, CA-5)
//! Compilado apenas quando a feature `dev-http` está habilitada (Spec 033, Fase 1).

use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::api::{self, ApiContext};
pub use crate::api::{nominal_chassis_for_model, AppState};
use crate::audio::{
    AudioStatusTelemetry, LockFreeParamQueue, RealtimeDspChain, TelemetryBuffer,
};
use crate::calibration::CalibrationStore;

struct RawRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl RawRequest {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Payload Too Large",
        415 => "Unsupported Media Type",
        421 => "Misdirected Request",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

fn read_http_request(stream: &mut TcpStream) -> Option<RawRequest> {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(1500)));
    let mut buf = Vec::with_capacity(2048);
    let mut tmp = [0u8; 2048];
    let header_end;

    loop {
        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            header_end = pos;
            break;
        }
        if buf.len() > 16 * 1024 {
            return None;
        }
        match stream.read(&mut tmp) {
            Ok(0) | Err(_) => return None,
            Ok(n) => buf.extend_from_slice(&tmp[..n]),
        }
    }

    let header_bytes = &buf[..header_end];
    let header_str = std::str::from_utf8(header_bytes).ok()?;
    let mut lines = header_str.split("\r\n");
    let request_line = lines.next()?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let url = parts.next()?.to_string();

    let mut headers = Vec::new();
    let mut content_length: usize = 0;
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_ascii_lowercase();
            let val = v.trim().to_string();
            if key == "content-length" {
                content_length = val.parse::<usize>().unwrap_or(0);
            }
            headers.push((key, val));
        }
    }

    let body_start = header_end + 4;
    let mut body = Vec::new();
    let path = url.split('?').next().unwrap_or("/");
    let max_read = if path == "/api/params" { 4097 } else { 20 * 1024 * 1024 + 1 };
    let target_len = content_length.min(max_read);

    if buf.len() > body_start {
        let available = &buf[body_start..];
        let take = available.len().min(target_len.max(available.len()).min(max_read));
        body.extend_from_slice(&available[..take]);
    }

    while body.len() < target_len {
        let to_read = (target_len - body.len()).min(tmp.len());
        match stream.read(&mut tmp[..to_read]) {
            Ok(0) | Err(_) => break,
            Ok(n) => body.extend_from_slice(&tmp[..n]),
        }
    }

    Some(RawRequest {
        method,
        url,
        headers,
        body,
    })
}

pub struct LocalAppServer {
    host: String,
    port: u16,
    running: Arc<AtomicBool>,
    server_threads: Vec<thread::JoinHandle<()>>,
    audio_telemetry: Arc<Mutex<Option<AudioStatusTelemetry>>>,
    app_state: Arc<Mutex<AppState>>,
    session_token: String,
    param_queue: Option<Arc<LockFreeParamQueue>>,
}

fn generate_secure_token() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let ptr = Box::into_raw(Box::new(12345)) as usize;
    let seed1 = now ^ ((ptr as u128) << 32) ^ 0x517cc1b727220a95;
    let seed2 = (now.rotate_left(17)) ^ (ptr as u128) ^ 0xdeadbeefcafebabe;
    format!(
        "{:016x}{:016x}",
        (seed1 & 0xFFFFFFFFFFFFFFFF) as u64,
        (seed2 & 0xFFFFFFFFFFFFFFFF) as u64
    )
}

impl LocalAppServer {
    pub fn bind_ephemeral() -> Result<Self, String> {
        Self::bind_port(0)
    }

    pub fn bind_port(port: u16) -> Result<Self, String> {
        Self::bind_port_with_telemetry(port, Arc::new(TelemetryBuffer::new(512)), None)
    }

    pub fn bind_port_with_dsp(
        port: u16,
        dsp_chain: Arc<Mutex<RealtimeDspChain>>,
    ) -> Result<Self, String> {
        Self::bind_port_with_telemetry(port, Arc::new(TelemetryBuffer::new(512)), Some(dsp_chain))
    }

    pub fn bind_port_with_telemetry(
        port: u16,
        telemetry: Arc<TelemetryBuffer>,
        dsp_chain: Option<Arc<Mutex<RealtimeDspChain>>>,
    ) -> Result<Self, String> {
        Self::bind_port_with_all(port, telemetry, dsp_chain, None, None)
    }

    pub fn bind_port_with_device(
        port: u16,
        device_id: Option<&str>,
        calibration_store: Option<CalibrationStore>,
    ) -> Result<Self, String> {
        Self::bind_port_with_all(
            port,
            Arc::new(TelemetryBuffer::new(512)),
            None,
            device_id,
            calibration_store,
        )
    }

    pub fn bind_port_with_all(
        port: u16,
        telemetry: Arc<TelemetryBuffer>,
        dsp_chain: Option<Arc<Mutex<RealtimeDspChain>>>,
        device_id: Option<&str>,
        calibration_store: Option<CalibrationStore>,
    ) -> Result<Self, String> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr)
            .map_err(|e| format!("Falha ao ligar servidor em {}: {}", addr, e))?;
        let actual_port = listener.local_addr().map(|ip| ip.port()).unwrap_or(port);

        let running = Arc::new(AtomicBool::new(true));
        let audio_telemetry = Arc::new(Mutex::new(None::<AudioStatusTelemetry>));
        let param_queue = dsp_chain
            .as_ref()
            .and_then(|d| d.lock().ok().map(|chain| chain.param_queue()));

        let mut initial_state = AppState::default();
        if let (Some(dev_id), Some(ref store)) = (device_id, calibration_store.as_ref()) {
            if let Some((boost, gate)) = store.get(dev_id) {
                initial_state.input_boost = boost;
                initial_state.gate_threshold_db = gate;
                if let Some(ref dsp_ref) = dsp_chain {
                    if let Ok(mut dsp) = dsp_ref.lock() {
                        dsp.set_input_boost(boost);
                        dsp.set_gate_threshold_db(gate);
                    }
                }
            }
        }

        let app_state = Arc::new(Mutex::new(initial_state));
        let session_token = generate_secure_token();
        let static_dir = find_design_system_dir();

        let api_ctx = ApiContext::new(
            audio_telemetry.clone(),
            app_state.clone(),
            telemetry,
            dsp_chain,
            param_queue.clone(),
            device_id,
            calibration_store,
        );

        let mut server_threads = Vec::new();
        for _ in 0..4 {
            let listener_clone = listener
                .try_clone()
                .map_err(|e| format!("Falha ao clonar listener TCP: {}", e))?;
            let running_clone = running.clone();
            let session_token_clone = session_token.clone();
            let static_dir_clone = static_dir.clone();
            let api_ctx_clone = api_ctx.clone();

            let th = thread::spawn(move || {
                while running_clone.load(Ordering::SeqCst) {
                    let mut stream = match listener_clone.accept() {
                        Ok((s, _)) => s,
                        Err(_) => break,
                    };
                    if !running_clone.load(Ordering::SeqCst) {
                        break;
                    }
                    let _ = stream.set_nodelay(true);

                    let Some(request) = read_http_request(&mut stream) else {
                        continue;
                    };

                    let method = request.method.as_str();
                    let url = request.url.as_str();
                    let path = url.split('?').next().unwrap_or("/");

                    // 1. Validação de Host contra DNS Rebinding (Spec 015, CA-2)
                    let host_header = request.header("host");
                    let is_valid_host = match host_header {
                        Some(v) => v.starts_with("127.0.0.1") || v.starts_with("localhost"),
                        None => true,
                    };

                    let (status, content_type, body) = if !is_valid_host {
                        (421, "text/plain", b"Misdirected Request".to_vec())
                    } else {
                        match path {
                            "/" | "/index.html" => {
                                let raw_content = static_dir_clone
                                    .as_ref()
                                    .and_then(|d| fs::read_to_string(d.join("index.html")).ok())
                                    .unwrap_or_else(default_index_html);
                                let injected = raw_content.replace(
                                    "<head>",
                                    &format!(
                                        "<head>\n  <meta name=\"sonix-token\" content=\"{}\">\n  <script>window.__SONIX_TOKEN__ = \"{}\";</script>",
                                        session_token_clone, session_token_clone
                                    ),
                                );
                                (200, "text/html; charset=utf-8", injected.into_bytes())
                            }
                            "/styles.css" => {
                                let content = static_dir_clone
                                    .as_ref()
                                    .and_then(|d| fs::read_to_string(d.join("styles.css")).ok())
                                    .unwrap_or_default();
                                (200, "text/css; charset=utf-8", content.into_bytes())
                            }
                            "/app.js" => {
                                let content = static_dir_clone
                                    .as_ref()
                                    .and_then(|d| fs::read_to_string(d.join("app.js")).ok())
                                    .unwrap_or_default();
                                (200, "application/javascript; charset=utf-8", content.into_bytes())
                            }
                            "/api/params" if method == "POST" => {
                                Self::verify_csrf_and_token(&request, &session_token_clone, |req| {
                                    let is_json = req
                                        .header("content-type")
                                        .map(|v| v.contains("application/json"))
                                        .unwrap_or(false);
                                    if !is_json {
                                        return (415, "text/plain", b"Unsupported Media Type".to_vec());
                                    }
                                    let content_len = req
                                        .header("content-length")
                                        .and_then(|v| v.parse::<usize>().ok());
                                    if content_len.map(|l| l > 4096).unwrap_or(false) || req.body.len() > 4096 {
                                        (413, "text/plain", b"Payload Too Large".to_vec())
                                    } else {
                                        api::dispatch(&api_ctx_clone, url, method, &req.body)
                                    }
                                })
                            }
                            "/api/input/di" | "/api/cab/ir" if method == "POST" => {
                                Self::verify_csrf_and_token(&request, &session_token_clone, |req| {
                                    let content_len = req
                                        .header("content-length")
                                        .and_then(|v| v.parse::<usize>().ok());
                                    if content_len.map(|l| l > 20 * 1024 * 1024).unwrap_or(false)
                                        || req.body.len() > 20 * 1024 * 1024
                                    {
                                        (413, "text/plain", b"Payload Too Large".to_vec())
                                    } else {
                                        api::dispatch(&api_ctx_clone, url, method, &req.body)
                                    }
                                })
                            }
                            "/api/presets/load"
                            | "/api/presets/save"
                            | "/api/presets/import"
                            | "/api/scenes/select"
                            | "/api/input/calibrate"
                                if method == "POST" =>
                            {
                                Self::verify_csrf_and_token(&request, &session_token_clone, |req| {
                                    api::dispatch(&api_ctx_clone, url, method, &req.body)
                                })
                            }
                            _ => api::dispatch(&api_ctx_clone, url, method, &request.body),
                        }
                    };

                    let header_block = format!(
                        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        status,
                        reason_phrase(status),
                        content_type,
                        body.len()
                    );
                    let mut frame = Vec::with_capacity(header_block.len() + body.len());
                    frame.extend_from_slice(header_block.as_bytes());
                    frame.extend_from_slice(&body);
                    let _ = stream.write_all(&frame);
                    let _ = stream.flush();
                    let _ = stream.shutdown(Shutdown::Write);
                }
            });
            server_threads.push(th);
        }

        Ok(Self {
            host: "127.0.0.1".to_string(),
            port: actual_port,
            running,
            server_threads,
            audio_telemetry,
            app_state,
            session_token,
            param_queue,
        })
    }

    pub fn session_token(&self) -> String {
        self.session_token.clone()
    }

    pub fn app_state(&self) -> Arc<Mutex<AppState>> {
        self.app_state.clone()
    }

    pub fn param_queue(&self) -> Option<Arc<LockFreeParamQueue>> {
        self.param_queue.clone()
    }

    fn verify_csrf_and_token<F>(
        request: &RawRequest,
        session_token: &str,
        handler: F,
    ) -> (u16, &'static str, Vec<u8>)
    where
        F: FnOnce(&RawRequest) -> (u16, &'static str, Vec<u8>),
    {
        let origin_hdr = request.header("origin");
        let sec_fetch = request.header("sec-fetch-site");

        let is_valid_origin = match origin_hdr {
            Some(v) => v.starts_with("http://127.0.0.1") || v.starts_with("http://localhost"),
            None => sec_fetch.map(|s| s != "cross-site").unwrap_or(true),
        };

        if !is_valid_origin {
            return (403, "text/plain", b"Forbidden Origin".to_vec());
        }

        let has_valid_token = request
            .header("x-sonix-token")
            .map(|th| th == session_token)
            .unwrap_or(false);

        if !has_valid_token {
            return (401, "text/plain", b"Unauthorized: missing or invalid X-Sonix-Token".to_vec());
        }

        handler(request)
    }

    pub fn set_audio_telemetry(&self, telemetry: AudioStatusTelemetry) {
        if let Ok(mut guard) = self.audio_telemetry.lock() {
            *guard = Some(telemetry);
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn app_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Retorna o comando de inicialização em modo de janela dedicada de aplicativo nativa (Spec 013, CA-5)
    pub fn build_desktop_app_command(&self) -> String {
        format!("msedge --app={} --window-size=1280,820", self.app_url())
    }
}

impl Drop for LocalAppServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        for _ in 0..self.server_threads.len() {
            let _ = TcpStream::connect(("127.0.0.1", self.port));
        }
        for th in self.server_threads.drain(..) {
            let _ = th.join();
        }
    }
}

fn find_design_system_dir() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("design-system"),
        PathBuf::from("../design-system"),
        PathBuf::from("../../design-system"),
    ];

    for c in &candidates {
        if c.join("index.html").exists() {
            return Some(c.clone());
        }
    }

    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = Path::new(&manifest)
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("design-system"));
        if let Some(ref p) = p {
            if p.join("index.html").exists() {
                return Some(p.clone());
            }
        }
    }

    None
}

fn default_index_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>SONIX PRO // AMPFORGE™</title></head>
<body style="background:#121214;color:#eee;font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;">
    <h1>🎛️ SONIX PRO // AMPFORGE™ Desktop Host</h1>
</body>
</html>"#
        .to_string()
}
