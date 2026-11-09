//! crates/sonix-desktop/src/server.rs
//! Servidor Local IPC/HTTP Isolado em Loopback (127.0.0.1) e Lançador Desktop App Mode (Spec 013, CA-5)

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::{Header, Response, Server, StatusCode};
use crate::audio::{RealtimeDspChain, TelemetryBuffer};

pub struct LocalAppServer {
    host: String,
    port: u16,
    running: Arc<AtomicBool>,
    server_thread: Option<thread::JoinHandle<()>>,
}

impl LocalAppServer {
    pub fn bind_ephemeral() -> Result<Self, String> {
        Self::bind_port(0)
    }

    pub fn bind_port(port: u16) -> Result<Self, String> {
        Self::bind_port_with_telemetry(port, Arc::new(TelemetryBuffer::new(512)), None)
    }

    pub fn bind_port_with_telemetry(
        port: u16,
        telemetry: Arc<TelemetryBuffer>,
        dsp_chain: Option<Arc<Mutex<RealtimeDspChain>>>,
    ) -> Result<Self, String> {
        let addr = format!("127.0.0.1:{}", port);
        let server = Server::http(&addr).map_err(|e| format!("Falha ao ligar servidor em {}: {}", addr, e))?;
        let actual_port = server.server_addr().to_ip().map(|ip| ip.port()).unwrap_or(port);

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        // Procura pelos assets do frontend (design-system/)
        let static_dir = find_design_system_dir();
        let dsp_chain_clone = dsp_chain;

        let server_thread = thread::spawn(move || {
            while running_clone.load(Ordering::Relaxed) {
                // Polling não-bloqueante curto
                match server.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(Some(request)) => {
                        let url = request.url().to_string();
                        let path = url.split('?').next().unwrap_or("/");

                        let (status, content_type, body) = match path {
                            "/" | "/index.html" => {
                                let content = static_dir
                                    .as_ref()
                                    .and_then(|d| fs::read_to_string(d.join("index.html")).ok())
                                    .unwrap_or_else(|| default_index_html());
                                (200, "text/html; charset=utf-8", content.into_bytes())
                            }
                            "/styles.css" => {
                                let content = static_dir
                                    .as_ref()
                                    .and_then(|d| fs::read_to_string(d.join("styles.css")).ok())
                                    .unwrap_or_default();
                                (200, "text/css; charset=utf-8", content.into_bytes())
                            }
                            "/app.js" => {
                                let content = static_dir
                                    .as_ref()
                                    .and_then(|d| fs::read_to_string(d.join("app.js")).ok())
                                    .unwrap_or_default();
                                (200, "application/javascript; charset=utf-8", content.into_bytes())
                            }
                            "/api/status" => {
                                let json = r#"{"status":"online","engine":"AmpForge RealTime","version":"1.0.0","backend":"WASAPI"}"#;
                                (200, "application/json", json.as_bytes().to_vec())
                            }
                            "/api/telemetry" => {
                                let snap = telemetry.snapshot();
                                let json = serde_json::to_string(&snap).unwrap_or_else(|_| "{}".to_string());
                                (200, "application/json", json.into_bytes())
                            }
                            "/api/devices" => {
                                let json = r#"[{"id":"p2_mic","name":"Realtek Microphone (P2/P3 3.5mm)","input":true},{"id":"p2_out","name":"Realtek Audio Output (P2/P3)","input":false}]"#;
                                (200, "application/json", json.as_bytes().to_vec())
                            }
                            "/api/params" => {
                                if let Some(ref dsp_ref) = dsp_chain_clone {
                                    if let Some(query) = url.split('?').nth(1) {
                                        if let Ok(mut dsp) = dsp_ref.lock() {
                                            for param in query.split('&') {
                                                let mut parts = param.split('=');
                                                if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                                                    if let Ok(val) = v.parse::<f32>() {
                                                        match k {
                                                            "gain" => dsp.set_amp_gain(val),
                                                            "master" => dsp.set_amp_master(val),
                                                            "gate" => dsp.set_gate_threshold_db(val),
                                                            "boost" => dsp.set_input_boost(val),
                                                            "cab" => dsp.set_cab_enabled(val > 0.5),
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                (200, "application/json", b"{\"status\":\"ok\"}".to_vec())
                            }
                            _ => (404, "text/plain", b"Not Found".to_vec()),
                        };

                        let content_type_header = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
                            .unwrap();
                        let cors_header = Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..])
                            .unwrap();

                        let mut resp = Response::from_data(body).with_status_code(StatusCode(status));
                        resp.add_header(content_type_header);
                        resp.add_header(cors_header);

                        let _ = request.respond(resp);
                    }
                    Ok(None) => {}
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            host: "127.0.0.1".to_string(),
            port: actual_port,
            running,
            server_thread: Some(server_thread),
        })
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
        self.running.store(false, Ordering::Relaxed);
        if let Some(th) = self.server_thread.take() {
            let _ = th.join();
        }
    }
}

fn find_design_system_dir() -> Option<PathBuf> {
    // Tenta encontrar o diretório design-system navegando a partir do diretório atual ou CARGO_MANIFEST_DIR
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
        let p = Path::new(&manifest).parent().and_then(|p| p.parent()).map(|p| p.join("design-system"));
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
