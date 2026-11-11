//! crates/sonix-desktop/src/server.rs
//! Servidor Local IPC/HTTP Isolado em Loopback (127.0.0.1) e Lançador Desktop App Mode (Spec 013, CA-5)

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::{Header, Response, Server, StatusCode};
use crate::audio::{AudioStatusTelemetry, RealtimeDspChain, TelemetryBuffer};
use crate::calibration::CalibrationStore;

fn default_true() -> bool { true }
fn default_one() -> f32 { 1.0 }
fn default_four() -> f32 { 4.0 }
fn default_five() -> f32 { 5.0 }
fn default_forty() -> f32 { 40.0 }
fn default_fifty() -> f32 { 50.0 }
fn default_crossover_hz() -> f32 { 650.0 }
fn default_merge_level() -> f32 { -6.0 }
fn default_decay() -> f32 { 2.5 }
fn default_predelay() -> f32 { 20.0 }
fn default_split_mode() -> String { "ab".to_string() }
fn default_comp_ratio() -> String { "4".to_string() }
fn default_dly_heads() -> String { "1".to_string() }
fn default_twin() -> String { "twin_ab763".to_string() }
fn default_alnico() -> String { "alnico_blue_2x12".to_string() }
fn default_preset_active() -> String { "brit800".to_string() }
fn default_preset_name() -> String { "Brit 800 Crunch".to_string() }
fn default_scene_active() -> String { "A".to_string() }
fn default_mute() -> String { "mute".to_string() }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppState {
    #[serde(rename = "amp_1.gain")]
    pub amp_gain: f32,
    #[serde(rename = "amp_1.overdrive")]
    pub amp_overdrive: f32,
    #[serde(rename = "amp_1.bass")]
    pub amp_bass: f32,
    #[serde(rename = "amp_1.middle")]
    pub amp_middle: f32,
    #[serde(rename = "amp_1.treble")]
    pub amp_treble: f32,
    #[serde(rename = "amp_1.presence")]
    pub amp_presence: f32,
    #[serde(rename = "amp_1.depth")]
    pub amp_depth: f32,
    #[serde(rename = "amp_1.master")]
    pub amp_master: f32,
    #[serde(rename = "amp_1.volume_2")]
    pub amp_volume_2: f32,
    #[serde(rename = "amp_1.cut")]
    pub amp_cut: f32,
    #[serde(rename = "amp_1.model")]
    pub amp_model: String,
    #[serde(rename = "engine.oversampling")]
    pub engine_oversampling: String,
    #[serde(rename = "input.boost")]
    pub input_boost: f32,
    #[serde(rename = "gate.threshold_db")]
    pub gate_threshold_db: f32,
    #[serde(rename = "input.source")]
    pub input_source: String,
    #[serde(rename = "amp_1.bright")]
    pub amp_bright: bool,
    #[serde(rename = "drive.enabled")]
    pub drive_enabled: bool,
    #[serde(rename = "drive.drive")]
    pub drive_drive: f32,
    #[serde(rename = "drive.tone")]
    pub drive_tone: f32,
    #[serde(rename = "drive.level")]
    pub drive_level: f32,
    #[serde(rename = "gate.enabled")]
    pub gate_enabled: bool,
    #[serde(rename = "amp_1.enabled")]
    pub amp_enabled: bool,
    #[serde(rename = "cab_1.enabled")]
    pub cab_enabled: bool,
    // Parâmetros de Bancada / Mod Shop (Spec 021, CA-9 & CA-10)
    #[serde(rename = "amp_1.preamp_tube")]
    pub preamp_tube: String,
    #[serde(rename = "amp_1.power_tube")]
    pub power_tube: String,
    #[serde(rename = "amp_1.bias_ma")]
    pub bias_ma: f32,
    #[serde(rename = "amp_1.sag")]
    pub sag: f32,
    #[serde(rename = "amp_1.rectifier")]
    pub rectifier: String,
    #[serde(rename = "amp_1.nfb")]
    pub nfb: f32,
    #[serde(rename = "amp_1.xformer_sat")]
    pub xformer_sat: f32,
    #[serde(rename = "amp_1.preamp_plate_v")]
    pub preamp_plate_v: f32,
    #[serde(rename = "amp_1.triode_asym")]
    pub triode_asym: f32,
    #[serde(rename = "amp_1.tonestack")]
    pub tonestack: String,
    pub is_modded: bool,
    #[serde(rename = "cab_1.speaker")]
    pub cab_speaker: String,
    #[serde(rename = "cab_1.resonance_hz")]
    pub cab_resonance_hz: f32,
    #[serde(rename = "cab_1.hicut_khz")]
    pub cab_hicut_khz: f32,
    #[serde(rename = "cab_1.ir_enabled")]
    pub cab_ir_enabled: bool,
    #[serde(rename = "cab_1.mic1.model")]
    pub cab_mic1_model: String,
    #[serde(rename = "cab_1.mic1.position")]
    pub cab_mic1_position: f32,
    #[serde(rename = "cab_1.mic1.distance_cm")]
    pub cab_mic1_distance_cm: f32,
    #[serde(rename = "cab_1.mic1.level_db")]
    pub cab_mic1_level_db: f32,
    #[serde(rename = "cab_1.mic1.enabled")]
    pub cab_mic1_enabled: bool,
    #[serde(rename = "cab_1.mic2.model")]
    pub cab_mic2_model: String,
    #[serde(rename = "cab_1.mic2.position")]
    pub cab_mic2_position: f32,
    #[serde(rename = "cab_1.mic2.distance_cm")]
    pub cab_mic2_distance_cm: f32,
    #[serde(rename = "cab_1.mic2.level_db")]
    pub cab_mic2_level_db: f32,
    #[serde(rename = "cab_1.mic2.enabled")]
    pub cab_mic2_enabled: bool,
    // Spec 024: Efeitos e Caminho Paralelo
    #[serde(default, rename = "split.enabled")]
    pub split_enabled: bool,
    #[serde(default = "default_split_mode", rename = "split.mode")]
    pub split_mode: String,
    #[serde(default = "default_crossover_hz", rename = "split.crossover_hz")]
    pub split_crossover_hz: f32,
    #[serde(default = "default_merge_level", rename = "merge.row_1_level_db")]
    pub merge_row_1_level_db: f32,
    #[serde(default, rename = "merge.row_1_phase_invert")]
    pub merge_row_1_phase_invert: bool,
    #[serde(default, rename = "comp_2.enabled")]
    pub comp_2_enabled: bool,
    #[serde(default = "default_five", rename = "comp_2.input")]
    pub comp_2_input: f32,
    #[serde(default = "default_five", rename = "comp_2.output")]
    pub comp_2_output: f32,
    #[serde(default = "default_four", rename = "comp_2.attack")]
    pub comp_2_attack: f32,
    #[serde(default = "default_four", rename = "comp_2.release")]
    pub comp_2_release: f32,
    #[serde(default = "default_comp_ratio", rename = "comp_2.ratio")]
    pub comp_2_ratio: String,
    #[serde(default, rename = "eq_1.enabled")]
    pub eq_1_enabled: bool,
    #[serde(default, rename = "mod_2.enabled")]
    pub mod_2_enabled: bool,
    #[serde(default = "default_one", rename = "mod_2.rate_hz")]
    pub mod_2_rate_hz: f32,
    #[serde(default = "default_five", rename = "mod_2.depth")]
    pub mod_2_depth: f32,
    #[serde(default = "default_fifty", rename = "mod_2.mix")]
    pub mod_2_mix: f32,
    #[serde(default, rename = "dly_1.enabled")]
    pub dly_1_enabled: bool,
    #[serde(default = "default_dly_heads", rename = "dly_1.heads")]
    pub dly_1_heads: String,
    #[serde(default = "default_five", rename = "dly_1.repeat_rate")]
    pub dly_1_repeat_rate: f32,
    #[serde(default = "default_five", rename = "dly_1.intensity")]
    pub dly_1_intensity: f32,
    #[serde(default = "default_five", rename = "dly_1.echo_volume")]
    pub dly_1_echo_volume: f32,
    #[serde(default = "default_five", rename = "dly_1.bass")]
    pub dly_1_bass: f32,
    #[serde(default = "default_five", rename = "dly_1.treble")]
    pub dly_1_treble: f32,
    #[serde(default = "default_true", rename = "dly_1.spillover")]
    pub dly_1_spillover: bool,
    #[serde(default, rename = "rev_1.enabled")]
    pub rev_1_enabled: bool,
    #[serde(default = "default_decay", rename = "rev_1.decay_s")]
    pub rev_1_decay_s: f32,
    #[serde(default = "default_predelay", rename = "rev_1.predelay_ms")]
    pub rev_1_predelay_ms: f32,
    #[serde(default = "default_five", rename = "rev_1.damping")]
    pub rev_1_damping: f32,
    #[serde(default = "default_forty", rename = "rev_1.mix")]
    pub rev_1_mix: f32,
    #[serde(default = "default_true", rename = "rev_1.spillover")]
    pub rev_1_spillover: bool,
    #[serde(default, rename = "amp_2.enabled")]
    pub amp_2_enabled: bool,
    #[serde(default = "default_twin", rename = "amp_2.model")]
    pub amp_2_model: String,
    #[serde(default, rename = "cab_2.enabled")]
    pub cab_2_enabled: bool,
    #[serde(default = "default_alnico", rename = "cab_2.speaker")]
    pub cab_2_speaker: String,
    #[serde(rename = "preset.active", default = "default_preset_active")]
    pub active_preset_id: String,
    #[serde(rename = "preset.name", default = "default_preset_name")]
    pub active_preset_name: String,
    #[serde(rename = "scene.active", default = "default_scene_active")]
    pub scene_active: String,
    #[serde(default, rename = "tuner.enabled")]
    pub tuner_enabled: bool,
    #[serde(rename = "tuner.output", default = "default_mute")]
    pub tuner_output: String,
}

pub fn nominal_chassis_for_model(model: &str) -> (&'static str, &'static str, f32, f32, &'static str, f32, f32, f32, f32, &'static str) {
    match model {
        "jcm800_2203" => ("12ax7", "el34", 38.0, 3.0, "diode", 5.0, 4.0, 280.0, 0.20, "model"),
        "plexi_1959" => ("12ax7", "el34", 36.0, 4.0, "diode", 6.0, 5.0, 250.0, 0.22, "model"),
        "twin_ab763" => ("12ax7", "6l6gc", 35.0, 2.0, "diode", 8.0, 2.0, 310.0, 0.15, "model"),
        "dual_rect" => ("12ax7", "6l6gc", 32.0, 5.0, "5u4gb", 3.0, 3.0, 290.0, 0.28, "model"),
        "ac30_tb" => ("12ax7", "el84_variant", 45.0, 6.0, "gz34", 0.0, 5.0, 240.0, 0.18, "model"),
        _ => ("12ax7", "el34", 38.0, 3.0, "diode", 5.0, 4.0, 280.0, 0.20, "model"),
    }
}

impl AppState {
    pub fn check_is_modded(&self) -> bool {
        let nom = nominal_chassis_for_model(&self.amp_model);
        self.preamp_tube != nom.0
            || self.power_tube != nom.1
            || (self.bias_ma - nom.2).abs() > 0.01
            || (self.sag - nom.3).abs() > 0.01
            || self.rectifier != nom.4
            || (self.nfb - nom.5).abs() > 0.01
            || (self.xformer_sat - nom.6).abs() > 0.01
            || (self.preamp_plate_v - nom.7).abs() > 0.01
            || (self.triode_asym - nom.8).abs() > 0.01
            || self.tonestack != nom.9
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            amp_gain: 8.0,
            amp_overdrive: 5.0,
            amp_bass: 5.0,
            amp_middle: 5.0,
            amp_treble: 5.0,
            amp_presence: 5.0,
            amp_depth: 5.0,
            amp_master: 7.0,
            amp_volume_2: 0.0,
            amp_cut: 5.0,
            amp_model: "jcm800_2203".to_string(),
            engine_oversampling: "8x".to_string(),
            input_boost: 2.5,
            gate_threshold_db: -65.0,
            input_source: "device".to_string(),
            amp_bright: false,
            drive_enabled: false,
            drive_drive: 5.0,
            drive_tone: 5.0,
            drive_level: 5.0,
            gate_enabled: true,
            amp_enabled: true,
            cab_enabled: true,
            preamp_tube: "12ax7".to_string(),
            power_tube: "el34".to_string(),
            bias_ma: 38.0,
            sag: 3.0,
            rectifier: "diode".to_string(),
            nfb: 5.0,
            xformer_sat: 4.0,
            preamp_plate_v: 280.0,
            triode_asym: 0.20,
            tonestack: "model".to_string(),
            is_modded: false,
            cab_speaker: "greenback_4x12".to_string(),
            cab_resonance_hz: 98.0,
            cab_hicut_khz: 8.2,
            cab_ir_enabled: false,
            cab_mic1_model: "sm57".to_string(),
            cab_mic1_position: 0.2,
            cab_mic1_distance_cm: 2.0,
            cab_mic1_level_db: 0.0,
            cab_mic1_enabled: true,
            cab_mic2_model: "sm57".to_string(),
            cab_mic2_position: 0.5,
            cab_mic2_distance_cm: 5.0,
            cab_mic2_level_db: 0.0,
            cab_mic2_enabled: false,
            split_enabled: false,
            split_mode: "ab".to_string(),
            split_crossover_hz: 650.0,
            merge_row_1_level_db: -6.0,
            merge_row_1_phase_invert: false,
            comp_2_enabled: false,
            comp_2_input: 5.0,
            comp_2_output: 5.0,
            comp_2_attack: 4.0,
            comp_2_release: 4.0,
            comp_2_ratio: "4".to_string(),
            eq_1_enabled: false,
            mod_2_enabled: false,
            mod_2_rate_hz: 1.0,
            mod_2_depth: 5.0,
            mod_2_mix: 50.0,
            dly_1_enabled: false,
            dly_1_heads: "1".to_string(),
            dly_1_repeat_rate: 5.0,
            dly_1_intensity: 5.0,
            dly_1_echo_volume: 5.0,
            dly_1_bass: 5.0,
            dly_1_treble: 5.0,
            dly_1_spillover: true,
            rev_1_enabled: false,
            rev_1_decay_s: 2.5,
            rev_1_predelay_ms: 20.0,
            rev_1_damping: 5.0,
            rev_1_mix: 40.0,
            rev_1_spillover: true,
            amp_2_enabled: false,
            amp_2_model: "twin_ab763".to_string(),
            cab_2_enabled: false,
            cab_2_speaker: "alnico_blue_2x12".to_string(),
            active_preset_id: "brit800".to_string(),
            active_preset_name: "Brit 800 Crunch".to_string(),
            scene_active: "A".to_string(),
            tuner_enabled: false,
            tuner_output: "mute".to_string(),
        }
    }
}

pub struct LocalAppServer {
    host: String,
    port: u16,
    running: Arc<AtomicBool>,
    server_threads: Vec<thread::JoinHandle<()>>,
    audio_telemetry: Arc<Mutex<Option<AudioStatusTelemetry>>>,
    app_state: Arc<Mutex<AppState>>,
    session_token: String,
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
        Self::bind_port_with_all(port, Arc::new(TelemetryBuffer::new(512)), None, device_id, calibration_store)
    }

    pub fn bind_port_with_all(
        port: u16,
        telemetry: Arc<TelemetryBuffer>,
        dsp_chain: Option<Arc<Mutex<RealtimeDspChain>>>,
        device_id: Option<&str>,
        calibration_store: Option<CalibrationStore>,
    ) -> Result<Self, String> {
        let addr = format!("127.0.0.1:{}", port);
        let server = Server::http(&addr).map_err(|e| format!("Falha ao ligar servidor em {}: {}", addr, e))?;
        let actual_port = server.server_addr().to_ip().map(|ip| ip.port()).unwrap_or(port);
        let server = Arc::new(server);

        let running = Arc::new(AtomicBool::new(true));
        let audio_telemetry = Arc::new(Mutex::new(None::<AudioStatusTelemetry>));

        let mut initial_state = AppState::default();
        if let (Some(dev_id), Some(store)) = (device_id, calibration_store) {
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

        // Procura pelos assets do frontend (design-system/)
        let static_dir = find_design_system_dir();

        let mut server_threads = Vec::new();
        // Atendimento concorrente com 4 worker threads (Spec 015, Requisito #8)
        for _ in 0..4 {
            let server_clone = server.clone();
            let running_clone = running.clone();
            let audio_telemetry_clone = audio_telemetry.clone();
            let app_state_clone = app_state.clone();
            let session_token_clone = session_token.clone();
            let telemetry_clone = telemetry.clone();
            let static_dir_clone = static_dir.clone();
            let dsp_chain_clone = dsp_chain.clone();

            let th = thread::spawn(move || {
                while running_clone.load(Ordering::Relaxed) {
                    // Polling não-bloqueante curto
                    match server_clone.recv_timeout(std::time::Duration::from_millis(50)) {
                        Ok(Some(mut request)) => {
                            let method = request.method().as_str().to_string();
                            let url = request.url().to_string();
                            let path = url.split('?').next().unwrap_or("/").to_string();

                            // 1. Validação de Host contra DNS Rebinding (Spec 015, CA-2)
                            let host_header = request
                                .headers()
                                .iter()
                                .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case("host"));
                            let is_valid_host = match host_header {
                                Some(h) => {
                                    let v = h.value.as_str();
                                    v.starts_with("127.0.0.1") || v.starts_with("localhost")
                                }
                                None => true,
                            };

                            if !is_valid_host {
                                Self::drain_request_body(&mut request);
                                let mut resp = Response::from_data(b"Misdirected Request".to_vec())
                                    .with_status_code(StatusCode(421));
                                let ct = Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap();
                                let close_header = Header::from_bytes(&b"Connection"[..], &b"close"[..]).unwrap();
                                resp.add_header(ct);
                                resp.add_header(close_header);
                                let _ = request.respond(resp);
                                continue;
                            }

                            let (status, content_type, body) = match path.as_str() {
                                "/" | "/index.html" => {
                                    let raw_content = static_dir_clone
                                        .as_ref()
                                        .and_then(|d| fs::read_to_string(d.join("index.html")).ok())
                                        .unwrap_or_else(|| default_index_html());
                                    // Injeta o token de sessão CSRF no HTML servido (Spec 015, CA-4)
                                    let injected = raw_content.replace(
                                        "<head>",
                                        &format!("<head>\n  <meta name=\"sonix-token\" content=\"{}\">\n  <script>window.__SONIX_TOKEN__ = \"{}\";</script>", session_token_clone, session_token_clone)
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
                                "/api/status" => {
                                    let json = r#"{"status":"online","engine":"AmpForge RealTime","version":"1.0.0","backend":"WASAPI"}"#;
                                    (200, "application/json", json.as_bytes().to_vec())
                                }
                                "/api/audio/status" => {
                                    let (clips, algo_lat) = if let Some(ref dsp_ref) = dsp_chain_clone {
                                        dsp_ref.lock().map(|d| (d.adc_clips(), d.algorithmic_latency_samples())).unwrap_or((0, 16))
                                    } else {
                                        (0, 16)
                                    };
                                    let json = if let Ok(mut guard) = audio_telemetry_clone.lock() {
                                        if let Some(ref mut t) = *guard {
                                            t.adc_clips = clips;
                                            t.algorithmic_latency_samples = algo_lat;
                                            serde_json::to_string(t).unwrap_or_else(|_| "{}".to_string())
                                        } else {
                                            format!(
                                                r#"{{"backend":"wasapi","mode":"wasapi_shared","raw_processing":false,"sample_rate":48000,"input_period_frames":128,"output_period_frames":128,"input_device":"Default","output_device":"Default","driver_input_latency_ms":2.5,"driver_output_latency_ms":2.5,"cpu_avg_pct":5.0,"cpu_peak_pct":8.0,"underruns":0,"overruns":0,"panics":0,"adc_clips":{},"measured_roundtrip_ms":null,"algorithmic_latency_samples":{}}}"#,
                                                clips,
                                                algo_lat
                                            )
                                        }
                                    } else {
                                        format!(r#"{{"adc_clips":{},"algorithmic_latency_samples":{}}}"#, clips, algo_lat)
                                    };
                                    (200, "application/json", json.into_bytes())
                                }
                                "/api/telemetry" => {
                                    let snap = telemetry_clone.snapshot();
                                    let json = serde_json::to_string(&snap).unwrap_or_else(|_| "{}".to_string());
                                    (200, "application/json", json.into_bytes())
                                }
                                "/api/devices" => {
                                    let json = r#"[{"id":"p2_mic","name":"Realtek Microphone (P2/P3 3.5mm)","input":true},{"id":"p2_out","name":"Realtek Audio Output (P2/P3)","input":false}]"#;
                                    (200, "application/json", json.as_bytes().to_vec())
                                }
                                "/api/state" => {
                                    if method == "GET" {
                                        let st = app_state_clone.lock().unwrap();
                                        let json = serde_json::to_string(&*st).unwrap();
                                        (200, "application/json", json.into_bytes())
                                    } else {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    }
                                }

                                "/api/tuner" => {
                                    if method == "GET" {
                                        let res = if let Some(ref dsp_ref) = dsp_chain_clone {
                                            if let Ok(mut dsp) = dsp_ref.lock() {
                                                dsp.tuner_latest_result()
                                            } else {
                                                ampforge_dsp::tuner::TunerPitchResult {
                                                    note_name: "---".to_string(),
                                                    cents_deviation: 0.0,
                                                    frequency_hz: 0.0,
                                                }
                                            }
                                        } else {
                                            ampforge_dsp::tuner::TunerPitchResult {
                                                note_name: "---".to_string(),
                                                cents_deviation: 0.0,
                                                frequency_hz: 0.0,
                                            }
                                        };
                                        let (is_muted, is_active) = if let Ok(st) = app_state_clone.lock() {
                                            (st.tuner_output == "mute" && st.tuner_enabled, st.tuner_enabled)
                                        } else {
                                            (false, false)
                                        };
                                        let body = serde_json::json!({
                                            "note": res.note_name,
                                            "frequency_hz": res.frequency_hz,
                                            "cents": res.cents_deviation,
                                            "in_tune": res.cents_deviation.abs() <= 1.0 && res.note_name != "---",
                                            "active": is_active,
                                            "muted": is_muted,
                                        });
                                        (200, "application/json", serde_json::to_vec(&body).unwrap())
                                    } else {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    }
                                }

                                "/api/graph" => {
                                    if method == "GET" {
                                        let (split_en, comp_en, eq_en, mod_en, dly_en, rev_en, amp2_en, cab2_en) = if let Ok(st) = app_state_clone.lock() {
                                            (
                                                st.split_enabled,
                                                st.comp_2_enabled,
                                                st.eq_1_enabled,
                                                st.mod_2_enabled,
                                                st.dly_1_enabled,
                                                st.rev_1_enabled,
                                                st.amp_2_enabled,
                                                st.cab_2_enabled,
                                            )
                                        } else {
                                            (false, false, false, false, false, false, false, false)
                                        };

                                        let graph = serde_json::json!({
                                            "rows": [
                                                {
                                                    "index": 0,
                                                    "blocks": [
                                                        {"id": "gate", "col": 0, "name": "Natural Gate", "enabled": true},
                                                        {"id": "drive", "col": 1, "name": "Screamer 808", "enabled": true},
                                                        {"id": "split", "col": 2, "name": "A/B Crossover", "enabled": split_en},
                                                        {"id": "amp_1", "col": 3, "name": "Brit 800 2203", "enabled": true},
                                                        {"id": "cab_1", "col": 4, "name": "4x12 Greenback", "enabled": true},
                                                        {"id": "eq_1", "col": 5, "name": "8-Band Studio", "enabled": eq_en},
                                                        {"id": "merge", "col": 6, "name": "Mixer Merge", "enabled": true},
                                                        {"id": "dly_1", "col": 7, "name": "Tape Echo 201", "enabled": dly_en},
                                                        {"id": "rev_1", "col": 8, "name": "Vintage Plate", "enabled": rev_en}
                                                    ]
                                                },
                                                {
                                                    "index": 1,
                                                    "blocks": [
                                                        {"id": "comp_2", "col": 2, "name": "FET 1176 Comp", "enabled": comp_en},
                                                        {"id": "amp_2", "col": 3, "name": "US Twin Clean", "enabled": amp2_en},
                                                        {"id": "cab_2", "col": 4, "name": "2x12 Alnico", "enabled": cab2_en},
                                                        {"id": "mod_2", "col": 5, "name": "Tri-Chorus", "enabled": mod_en}
                                                    ]
                                                }
                                            ]
                                        });
                                        (200, "application/json", serde_json::to_vec(&graph).unwrap())
                                    } else {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    }
                                }

                                "/api/scope" => {
                                    if method == "GET" {
                                        let mode = url
                                            .split('?')
                                            .nth(1)
                                            .and_then(|q| {
                                                q.split('&').find_map(|pair| {
                                                    let mut parts = pair.split('=');
                                                    if parts.next()? == "mode" {
                                                        parts.next()
                                                    } else {
                                                        None
                                                    }
                                                })
                                            })
                                            .unwrap_or("transfer");

                                        let (asym, plate_v) = if let Ok(st) = app_state_clone.lock() {
                                            (st.triode_asym, st.preamp_plate_v)
                                        } else {
                                            (0.20, 250.0)
                                        };

                                        match mode {
                                            "transfer" => {
                                                let n = 65;
                                                let mut pts = Vec::with_capacity(n);
                                                let v1_headroom = (plate_v as f64 / 250.0).max(0.1);
                                                for i in 0..n {
                                                    let x = -2.0 + (4.0 * i as f64 / (n - 1) as f64);
                                                    let pre_in_raw = x / v1_headroom;
                                                    let pre_in = if asym as f64 > 1e-4 {
                                                        if pre_in_raw >= 0.0 {
                                                            pre_in_raw * (1.0 + 0.5 * asym as f64)
                                                        } else {
                                                            pre_in_raw * (1.0 - 0.5 * asym as f64)
                                                        }
                                                    } else {
                                                        pre_in_raw
                                                    };
                                                    let y = (pre_in / (1.0 + pre_in.abs())) * v1_headroom;
                                                    pts.push(serde_json::json!({
                                                        "x": (x * 10000.0).round() / 10000.0,
                                                        "y": (y * 10000.0).round() / 10000.0
                                                    }));
                                                }
                                                let resp = serde_json::json!({ "transfer": pts });
                                                (200, "application/json", serde_json::to_vec(&resp).unwrap())
                                            }
                                            "waveform" => {
                                                let n = 128;
                                                let mut di = Vec::with_capacity(n);
                                                let mut sat = Vec::with_capacity(n);
                                                for i in 0..n {
                                                    let t = i as f64 / n as f64;
                                                    let s = (2.0 * std::f64::consts::PI * t).sin();
                                                    di.push(s);
                                                    sat.push((s * 1.5).tanh());
                                                }
                                                let resp = serde_json::json!({
                                                    "waveform": {
                                                        "di": di,
                                                        "output": sat
                                                    }
                                                });
                                                (200, "application/json", serde_json::to_vec(&resp).unwrap())
                                            }
                                            "tonestack" => {
                                                let n = 50;
                                                let mut ts = Vec::with_capacity(n);
                                                let mut z_curve = Vec::with_capacity(n);
                                                for i in 0..n {
                                                    let f = 20.0 * (1000.0f64).powf(i as f64 / (n - 1) as f64);
                                                    let db = -6.0 * (-((f - 650.0) / 400.0).powi(2)).exp();
                                                    let z = 8.0 + 32.0 / (1.0 + ((f - 98.0) / 15.0).powi(2)) + (f / 2000.0);
                                                    ts.push(serde_json::json!({"f": f, "db": db}));
                                                    z_curve.push(serde_json::json!({"f": f, "z": z}));
                                                }
                                                let resp = serde_json::json!({
                                                    "tonestack": ts,
                                                    "speaker_z": z_curve
                                                });
                                                (200, "application/json", serde_json::to_vec(&resp).unwrap())
                                            }
                                            _ => {
                                                (400, "application/json", b"{\"error\":\"Modo de osciloscopio invalido\"}".to_vec())
                                            }
                                        }
                                    } else {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    }
                                }

                                "/api/params" => {
                                    if method == "GET" {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    } else if method == "POST" {
                                        // 2. Proteção contra CSRF: valida Origin e Sec-Fetch-Site (Spec 015, CA-1)
                                        let origin_hdr = request.headers().iter().find(|h| {
                                            h.field.as_str().as_str().eq_ignore_ascii_case("origin")
                                        });
                                        let sec_fetch = request.headers().iter().find(|h| {
                                            h.field.as_str().as_str().eq_ignore_ascii_case("sec-fetch-site")
                                        });

                                        let is_valid_origin = match origin_hdr {
                                            Some(orig) => {
                                                let v = orig.value.as_str();
                                                v.starts_with("http://127.0.0.1") || v.starts_with("http://localhost")
                                            }
                                            None => {
                                                if let Some(sec) = sec_fetch {
                                                    sec.value.as_str() != "cross-site"
                                                } else {
                                                    true
                                                }
                                            }
                                        };

                                        if !is_valid_origin {
                                            Self::drain_request_body(&mut request);
                                            (403, "text/plain", b"Forbidden Origin".to_vec())
                                        } else {
                                            // 3. Validação do Token de Sessão X-Sonix-Token (Spec 015, CA-4)
                                            let token_hdr = request.headers().iter().find(|h| {
                                                h.field.as_str().as_str().eq_ignore_ascii_case("x-sonix-token")
                                            });
                                            let has_valid_token = match token_hdr {
                                                Some(th) => th.value.as_str() == session_token_clone.as_str(),
                                                None => false,
                                            };

                                            if !has_valid_token {
                                                Self::drain_request_body(&mut request);
                                                (401, "text/plain", b"Unauthorized: missing or invalid X-Sonix-Token".to_vec())
                                            } else {
                                                // 4. Limite de Tamanho do Corpo: max 4 KiB (Spec 015, CA-5)
                                                let content_len = request.headers().iter()
                                                    .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case("content-length"))
                                                    .and_then(|h| h.value.as_str().parse::<usize>().ok());

                                                if let Some(len) = content_len {
                                                    if len > 4096 {
                                                        Self::drain_request_body(&mut request);
                                                        (413, "text/plain", b"Payload Too Large".to_vec())
                                                    } else {
                                                        Self::handle_post_params(&mut request, &app_state_clone, &dsp_chain_clone)
                                                    }
                                                } else {
                                                    Self::handle_post_params(&mut request, &app_state_clone, &dsp_chain_clone)
                                                }
                                            }
                                        }
                                    } else {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    }
                                }

                                "/api/input/di" => {
                                    if method == "POST" {
                                        let origin_hdr = request.headers().iter().find(|h| {
                                            h.field.as_str().as_str().eq_ignore_ascii_case("origin")
                                        });
                                        let sec_fetch = request.headers().iter().find(|h| {
                                            h.field.as_str().as_str().eq_ignore_ascii_case("sec-fetch-site")
                                        });

                                        let is_valid_origin = match origin_hdr {
                                            Some(orig) => {
                                                let v = orig.value.as_str();
                                                v.starts_with("http://127.0.0.1") || v.starts_with("http://localhost")
                                            }
                                            None => {
                                                if let Some(sec) = sec_fetch {
                                                    sec.value.as_str() != "cross-site"
                                                } else {
                                                    true
                                                }
                                            }
                                        };

                                        if !is_valid_origin {
                                            Self::drain_request_body(&mut request);
                                            (403, "text/plain", b"Forbidden Origin".to_vec())
                                        } else {
                                            let token_hdr = request.headers().iter().find(|h| {
                                                h.field.as_str().as_str().eq_ignore_ascii_case("x-sonix-token")
                                            });
                                            let has_valid_token = match token_hdr {
                                                Some(th) => th.value.as_str() == session_token_clone.as_str(),
                                                None => false,
                                            };

                                            if !has_valid_token {
                                                Self::drain_request_body(&mut request);
                                                (401, "text/plain", b"Unauthorized: missing or invalid X-Sonix-Token".to_vec())
                                            } else {
                                                let content_len = request.headers().iter()
                                                    .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case("content-length"))
                                                    .and_then(|h| h.value.as_str().parse::<usize>().ok());

                                                if let Some(len) = content_len {
                                                    if len > 20 * 1024 * 1024 {
                                                        Self::drain_request_body(&mut request);
                                                        (413, "text/plain", b"Payload Too Large".to_vec())
                                                    } else {
                                                        Self::handle_post_di(&mut request, &dsp_chain_clone)
                                                    }
                                                } else {
                                                    Self::handle_post_di(&mut request, &dsp_chain_clone)
                                                }
                                            }
                                        }
                                    } else {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    }
                                }

                                "/api/cab/ir" => {
                                    if method == "POST" {
                                        let origin_hdr = request.headers().iter().find(|h| {
                                            h.field.as_str().as_str().eq_ignore_ascii_case("origin")
                                        });
                                        let sec_fetch = request.headers().iter().find(|h| {
                                            h.field.as_str().as_str().eq_ignore_ascii_case("sec-fetch-site")
                                        });

                                        let is_valid_origin = match origin_hdr {
                                            Some(orig) => {
                                                let v = orig.value.as_str();
                                                v.starts_with("http://127.0.0.1") || v.starts_with("http://localhost")
                                            }
                                            None => {
                                                if let Some(sec) = sec_fetch {
                                                    sec.value.as_str() != "cross-site"
                                                } else {
                                                    true
                                                }
                                            }
                                        };

                                        if !is_valid_origin {
                                            Self::drain_request_body(&mut request);
                                            (403, "text/plain", b"Forbidden Origin".to_vec())
                                        } else {
                                            let token_hdr = request.headers().iter().find(|h| {
                                                h.field.as_str().as_str().eq_ignore_ascii_case("x-sonix-token")
                                            });
                                            let has_valid_token = match token_hdr {
                                                Some(th) => th.value.as_str() == session_token_clone.as_str(),
                                                None => false,
                                            };

                                            if !has_valid_token {
                                                Self::drain_request_body(&mut request);
                                                (401, "text/plain", b"Unauthorized: missing or invalid X-Sonix-Token".to_vec())
                                            } else {
                                                let content_len = request.headers().iter()
                                                    .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case("content-length"))
                                                    .and_then(|h| h.value.as_str().parse::<usize>().ok());

                                                if let Some(len) = content_len {
                                                    if len > 20 * 1024 * 1024 {
                                                        Self::drain_request_body(&mut request);
                                                        (413, "text/plain", b"Payload Too Large".to_vec())
                                                    } else {
                                                        Self::handle_post_cab_ir(&mut request, &app_state_clone, &dsp_chain_clone)
                                                    }
                                                } else {
                                                    Self::handle_post_cab_ir(&mut request, &app_state_clone, &dsp_chain_clone)
                                                }
                                            }
                                        }
                                    } else {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    }
                                }
                                "/api/presets" => {
                                    if method == "GET" {
                                        Self::handle_get_presets()
                                    } else {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    }
                                }
                                "/api/presets/load" => {
                                    if method == "POST" {
                                        Self::verify_csrf_and_token(&mut request, &session_token_clone, |req| {
                                            Self::handle_post_presets_load(req, &app_state_clone, &dsp_chain_clone)
                                        })
                                    } else {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    }
                                }
                                "/api/presets/save" => {
                                    if method == "POST" {
                                        Self::verify_csrf_and_token(&mut request, &session_token_clone, |req| {
                                            Self::handle_post_presets_save(req, &app_state_clone)
                                        })
                                    } else {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    }
                                }
                                "/api/presets/import" => {
                                    if method == "POST" {
                                        Self::verify_csrf_and_token(&mut request, &session_token_clone, |req| {
                                            Self::handle_post_presets_import(req, &app_state_clone, &dsp_chain_clone)
                                        })
                                    } else {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    }
                                }
                                "/api/scenes/select" => {
                                    if method == "POST" {
                                        Self::verify_csrf_and_token(&mut request, &session_token_clone, |req| {
                                            Self::handle_post_scenes_select(req, &app_state_clone)
                                        })
                                    } else {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    }
                                }
                                p if p.starts_with("/api/presets/") && p.ends_with("/export") => {
                                    if method == "GET" {
                                        let id = &p["/api/presets/".len()..p.len() - "/export".len()];
                                        Self::handle_get_presets_export(id)
                                    } else {
                                        (405, "text/plain", b"Method Not Allowed".to_vec())
                                    }
                                }
                                _ => (404, "text/plain", b"Not Found".to_vec()),
                            };

                            let content_type_header = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
                                .unwrap();
                            let close_header = Header::from_bytes(&b"Connection"[..], &b"close"[..])
                                .unwrap();

                            let mut resp = Response::from_data(body).with_status_code(StatusCode(status));
                            resp.add_header(content_type_header);
                            resp.add_header(close_header);
                            // Zero headers Access-Control-Allow-Origin: isolamento estrito loopback (Spec 015, CA-3)

                            let _ = request.respond(resp);
                        }
                        Ok(None) => {}
                        Err(_) => break,
                    }
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
        })
    }

    pub fn session_token(&self) -> String {
        self.session_token.clone()
    }

    pub fn app_state(&self) -> Arc<Mutex<AppState>> {
        self.app_state.clone()
    }

    fn drain_request_body(request: &mut tiny_http::Request) {
        let content_len = request.headers().iter()
            .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case("content-length"))
            .and_then(|h| h.value.as_str().parse::<usize>().ok())
            .unwrap_or(0);
        if content_len == 0 {
            return;
        }
        let reader = request.as_reader();
        let mut to_read = content_len;
        let mut buf = [0u8; 1024];
        while to_read > 0 {
            let chunk = to_read.min(buf.len());
            match reader.read(&mut buf[..chunk]) {
                Ok(0) | Err(_) => break,
                Ok(n) => to_read = to_read.saturating_sub(n),
            }
        }
    }

    fn handle_post_params(
        request: &mut tiny_http::Request,
        app_state_clone: &Arc<Mutex<AppState>>,
        dsp_chain_clone: &Option<Arc<Mutex<RealtimeDspChain>>>,
    ) -> (u16, &'static str, Vec<u8>) {
        let is_json = request.headers().iter().any(|h| {
            h.field.as_str().as_str().eq_ignore_ascii_case("content-type")
                && h.value.as_str().contains("application/json")
        });
        if !is_json {
            Self::drain_request_body(request);
            return (415, "text/plain", b"Unsupported Media Type".to_vec());
        }

        let mut buf = vec![0u8; 4098];
        let reader = request.as_reader();
        let bytes_read = match reader.read(&mut buf) {
            Ok(n) => n,
            Err(_) => return (400, "application/json", b"{\"error\":\"Erro na leitura do corpo\"}".to_vec()),
        };

        if bytes_read > 4096 {
            return (413, "text/plain", b"Payload Too Large".to_vec());
        }

        let body_str = String::from_utf8_lossy(&buf[..bytes_read]).to_string();

        match serde_json::from_str::<serde_json::Value>(&body_str) {
            Ok(serde_json::Value::Object(map)) => {
                let mut validation_error: Option<String> = None;
                let mut applied = Vec::new();
                let mut ignored = Vec::new();
                let mut ignored_reasons = serde_json::Map::new();

                let ir_active = if let Some(ir_val) = map.get("cab_1.ir_enabled") {
                    ir_val.as_bool().unwrap_or(false)
                } else {
                    app_state_clone.lock().map(|st| st.cab_ir_enabled).unwrap_or(false)
                };

                // 1. Determina modelo ativo para filtragem de parâmetros por circuito (Spec 020 §2 e CA-4)
                let active_model = if let Some(m_val) = map.get("amp_1.model") {
                    if let Some(m_str) = m_val.as_str() {
                        if !["jcm800_2203", "plexi_1959", "twin_ab763", "dual_rect", "ac30_tb"].contains(&m_str) {
                            return (400, "application/json", b"{\"error\":\"Modelo desconhecido\"}".to_vec());
                        }
                        m_str.to_string()
                    } else {
                        return (400, "application/json", b"{\"error\":\"amp_1.model deve ser string\"}".to_vec());
                    }
                } else {
                    app_state_clone.lock().map(|st| st.amp_model.clone()).unwrap_or_else(|_| "jcm800_2203".to_string())
                };

                let is_param_on_model = |model: &str, key: &str| -> bool {
                    match key {
                        "amp_1.master" => model != "plexi_1959" && model != "twin_ab763",
                        "amp_1.middle" => model != "ac30_tb",
                        "amp_1.volume_2" => model == "plexi_1959",
                        "amp_1.cut" => model == "ac30_tb",
                        _ => true,
                    }
                };

                for (k, v) in &map {
                    if k.starts_with("cab_1.mic") {
                        if ir_active {
                            ignored.push(k.clone());
                            ignored_reasons.insert(k.clone(), serde_json::Value::String("ir_active".to_string()));
                            continue;
                        }

                        match k.as_str() {
                            "cab_1.mic1.model" | "cab_1.mic2.model" => {
                                if let Some(s) = v.as_str() {
                                    if ["sm57", "md421", "m88", "r121", "m160", "u87", "c414"].contains(&s) {
                                        applied.push(k.clone());
                                    } else {
                                        validation_error = Some(format!("{} modelo de microfone inválido", k));
                                        break;
                                    }
                                } else {
                                    validation_error = Some(format!("{} deve ser string", k));
                                    break;
                                }
                            }
                            "cab_1.mic1.position" | "cab_1.mic2.position" => {
                                if let Some(num) = v.as_f64() {
                                    if num >= 0.0 && num <= 1.0 {
                                        applied.push(k.clone());
                                    } else {
                                        validation_error = Some(format!("{} fora da faixa 0.0..1.0", k));
                                        break;
                                    }
                                } else {
                                    validation_error = Some(format!("{} deve ser número", k));
                                    break;
                                }
                            }
                            "cab_1.mic1.distance_cm" | "cab_1.mic2.distance_cm" => {
                                if let Some(num) = v.as_f64() {
                                    if num >= 0.0 && num <= 30.0 {
                                        applied.push(k.clone());
                                    } else {
                                        validation_error = Some(format!("{} fora da faixa 0.0..30.0", k));
                                        break;
                                    }
                                } else {
                                    validation_error = Some(format!("{} deve ser número", k));
                                    break;
                                }
                            }
                            "cab_1.mic1.level_db" | "cab_1.mic2.level_db" => {
                                if let Some(num) = v.as_f64() {
                                    if num >= -60.0 && num <= 12.0 {
                                        applied.push(k.clone());
                                    } else {
                                        validation_error = Some(format!("{} fora da faixa -60.0..12.0", k));
                                        break;
                                    }
                                } else {
                                    validation_error = Some(format!("{} deve ser número", k));
                                    break;
                                }
                            }
                            "cab_1.mic1.enabled" | "cab_1.mic2.enabled" => {
                                if v.is_boolean() {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some(format!("{} deve ser booleano", k));
                                    break;
                                }
                            }
                            _ => {
                                ignored.push(k.clone());
                            }
                        }
                        continue;
                    }

                    match k.as_str() {
                        "amp_1.gain" | "amp_1.overdrive" | "amp_1.bass"
                        | "amp_1.middle" | "amp_1.treble"
                        | "amp_1.presence" | "amp_1.depth"
                        | "amp_1.master" | "amp_1.volume_2" | "amp_1.cut" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 10.0 {
                                    if is_param_on_model(&active_model, k) {
                                        applied.push(k.clone());
                                    } else {
                                        ignored.push(k.clone());
                                    }
                                } else {
                                    validation_error = Some(format!("{} fora da faixa 0..10", k));
                                    break;
                                }
                            } else {
                                validation_error = Some(format!("{} deve ser número", k));
                                break;
                            }
                        }
                        "drive.drive" | "drive.tone" | "drive.level" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 10.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some(format!("{} fora da faixa 0..10", k));
                                    break;
                                }
                            } else {
                                validation_error = Some(format!("{} deve ser número", k));
                                break;
                            }
                        }
                        "amp_1.model" => {
                            applied.push(k.clone());
                        }
                        "engine.oversampling" => {
                            if let Some(s) = v.as_str() {
                                if s == "2x" || s == "4x" || s == "8x" {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("engine.oversampling fora dos valores permitidos ('2x', '4x', '8x')".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("engine.oversampling deve ser string".to_string());
                                break;
                            }
                        }
                        "input.boost" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.1 && num <= 8.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("input.boost fora da faixa 0.1..8.0".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("input.boost deve ser número".to_string());
                                break;
                            }
                        }
                        "gate.threshold_db" => {
                            if let Some(num) = v.as_f64() {
                                if num >= -90.0 && num <= 0.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("gate.threshold_db fora da faixa -90..0".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("gate.threshold_db deve ser número".to_string());
                                break;
                            }
                        }
                        "amp_1.bright" => {
                            if v.is_boolean() {
                                if is_param_on_model(&active_model, k) {
                                    applied.push(k.clone());
                                } else {
                                    ignored.push(k.clone());
                                }
                            } else {
                                validation_error = Some(format!("{} deve ser booleano", k));
                                break;
                            }
                        }
                        "drive.enabled" | "gate.enabled"
                        | "amp_1.enabled" | "cab_1.enabled" => {
                            if v.is_boolean() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} deve ser booleano", k));
                                break;
                            }
                        }
                        "input.source" => {
                            if let Some(s) = v.as_str() {
                                if s == "device" || s == "test_riff" || s == "di_file" {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("input.source fora dos valores permitidos ('device', 'test_riff', 'di_file')".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("input.source deve ser string".to_string());
                                break;
                            }
                        }
                        "amp_1.bias_ma" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 20.0 && num <= 55.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("amp_1.bias_ma fora da faixa 20..55".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("amp_1.bias_ma deve ser número".to_string());
                                break;
                            }
                        }
                        "amp_1.sag" | "amp_1.nfb" | "amp_1.xformer_sat" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 10.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some(format!("{} fora da faixa 0..10", k));
                                    break;
                                }
                            } else {
                                validation_error = Some(format!("{} deve ser número", k));
                                break;
                            }
                        }
                        "amp_1.preamp_plate_v" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 180.0 && num <= 340.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("amp_1.preamp_plate_v fora da faixa 180..340".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("amp_1.preamp_plate_v deve ser número".to_string());
                                break;
                            }
                        }
                        "amp_1.triode_asym" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 0.60 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("amp_1.triode_asym fora da faixa 0..0.60".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("amp_1.triode_asym deve ser número".to_string());
                                break;
                            }
                        }
                        "amp_1.preamp_tube" => {
                            if let Some(s) = v.as_str() {
                                if ["12ax7", "12at7", "ef86_variant"].contains(&s) {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("amp_1.preamp_tube inválido".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("amp_1.preamp_tube deve ser string".to_string());
                                break;
                            }
                        }
                        "amp_1.power_tube" => {
                            if let Some(s) = v.as_str() {
                                if ["el34", "6l6gc", "kt88", "el84_variant"].contains(&s) {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("amp_1.power_tube inválido".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("amp_1.power_tube deve ser string".to_string());
                                break;
                            }
                        }
                        "amp_1.rectifier" => {
                            if let Some(s) = v.as_str() {
                                if ["diode", "gz34", "5u4gb"].contains(&s) {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("amp_1.rectifier inválido".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("amp_1.rectifier deve ser string".to_string());
                                break;
                            }
                        }
                        "amp_1.tonestack" => {
                            if let Some(s) = v.as_str() {
                                if ["model", "marshall", "fender", "vox", "baxandall"].contains(&s) {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("amp_1.tonestack inválido".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("amp_1.tonestack deve ser string".to_string());
                                break;
                            }
                        }
                        "cab_1.speaker" => {
                            if let Some(s) = v.as_str() {
                                if ["greenback_4x12", "v30_4x12", "alnico_blue_2x12"].contains(&s) {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("cab_1.speaker inválido".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("cab_1.speaker deve ser string".to_string());
                                break;
                            }
                        }
                        "cab_1.resonance_hz" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 65.0 && num <= 135.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("cab_1.resonance_hz fora da faixa 65..135".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("cab_1.resonance_hz deve ser número".to_string());
                                break;
                            }
                        }
                        "cab_1.hicut_khz" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 3.5 && num <= 18.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("cab_1.hicut_khz fora da faixa 3.5..18.0".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("cab_1.hicut_khz deve ser número".to_string());
                                break;
                            }
                        }
                        "cab_1.ir_enabled" => {
                            if v.is_boolean() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("cab_1.ir_enabled deve ser booleano".to_string());
                                break;
                            }
                        }
                        // Spec 024: Split & Merge
                        "split.enabled" => {
                            if v.is_boolean() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("split.enabled deve ser booleano".to_string());
                                break;
                            }
                        }
                        "split.mode" => {
                            if let Some(s) = v.as_str() {
                                if s == "ab" || s == "crossover" {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("split.mode inválido ('ab' ou 'crossover')".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("split.mode deve ser string".to_string());
                                break;
                            }
                        }
                        "split.crossover_hz" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 100.0 && num <= 2000.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("split.crossover_hz fora da faixa 100..2000".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("split.crossover_hz deve ser número".to_string());
                                break;
                            }
                        }
                        "split.ratio" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 1.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("split.ratio fora da faixa 0..1".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("split.ratio deve ser número".to_string());
                                break;
                            }
                        }
                        "merge.row_1_level_db" => {
                            if let Some(num) = v.as_f64() {
                                if num >= -60.0 && num <= 12.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("merge.row_1_level_db fora da faixa -60..12".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("merge.row_1_level_db deve ser número".to_string());
                                break;
                            }
                        }
                        "merge.row_1_phase_invert" => {
                            if v.is_boolean() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("merge.row_1_phase_invert deve ser booleano".to_string());
                                break;
                            }
                        }

                        // Comp 2
                        "comp_2.enabled" => {
                            if v.is_boolean() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("comp_2.enabled deve ser booleano".to_string());
                                break;
                            }
                        }
                        "comp_2.input" | "comp_2.output" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 10.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some(format!("{} fora da faixa 0..10", k));
                                    break;
                                }
                            } else {
                                validation_error = Some(format!("{} deve ser número", k));
                                break;
                            }
                        }
                        "comp_2.attack" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 1.0 && num <= 7.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("comp_2.attack fora da faixa 1..7".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("comp_2.attack deve ser número".to_string());
                                break;
                            }
                        }
                        "comp_2.release" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 1.0 && num <= 7.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("comp_2.release fora da faixa 1..7".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("comp_2.release deve ser número".to_string());
                                break;
                            }
                        }
                        "comp_2.ratio" => {
                            if let Some(s) = v.as_str() {
                                if ["4", "8", "12", "20", "all"].contains(&s) {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("comp_2.ratio inválido".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("comp_2.ratio deve ser string".to_string());
                                break;
                            }
                        }

                        // EQ 1
                        "eq_1.enabled" => {
                            if v.is_boolean() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("eq_1.enabled deve ser booleano".to_string());
                                break;
                            }
                        }

                        // Mod 2
                        "mod_2.enabled" => {
                            if v.is_boolean() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("mod_2.enabled deve ser booleano".to_string());
                                break;
                            }
                        }
                        "mod_2.rate_hz" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.05 && num <= 10.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("mod_2.rate_hz fora da faixa 0.05..10".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("mod_2.rate_hz deve ser número".to_string());
                                break;
                            }
                        }
                        "mod_2.depth" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 10.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("mod_2.depth fora da faixa 0..10".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("mod_2.depth deve ser número".to_string());
                                break;
                            }
                        }
                        "mod_2.mix" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 100.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("mod_2.mix fora da faixa 0..100".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("mod_2.mix deve ser número".to_string());
                                break;
                            }
                        }

                        // Dly 1
                        "dly_1.enabled" | "dly_1.spillover" => {
                            if v.is_boolean() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} deve ser booleano", k));
                                break;
                            }
                        }
                        "dly_1.heads" => {
                            if let Some(s) = v.as_str() {
                                if ["1", "2", "3", "1+2", "1+3", "2+3", "all"].contains(&s) {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("dly_1.heads inválido".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("dly_1.heads deve ser string".to_string());
                                break;
                            }
                        }
                        "dly_1.repeat_rate" | "dly_1.intensity" | "dly_1.echo_volume"
                        | "dly_1.bass" | "dly_1.treble" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 10.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some(format!("{} fora da faixa 0..10", k));
                                    break;
                                }
                            } else {
                                validation_error = Some(format!("{} deve ser número", k));
                                break;
                            }
                        }

                        // Rev 1
                        "rev_1.enabled" | "rev_1.spillover" => {
                            if v.is_boolean() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} deve ser booleano", k));
                                break;
                            }
                        }
                        "rev_1.decay_s" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.1 && num <= 20.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("rev_1.decay_s fora da faixa 0.1..20".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("rev_1.decay_s deve ser número".to_string());
                                break;
                            }
                        }
                        "rev_1.predelay_ms" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 250.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("rev_1.predelay_ms fora da faixa 0..250".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("rev_1.predelay_ms deve ser número".to_string());
                                break;
                            }
                        }
                        "rev_1.damping" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 10.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("rev_1.damping fora da faixa 0..10".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("rev_1.damping deve ser número".to_string());
                                break;
                            }
                        }
                        "rev_1.mix" => {
                            if let Some(num) = v.as_f64() {
                                if num >= 0.0 && num <= 100.0 {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("rev_1.mix fora da faixa 0..100".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("rev_1.mix deve ser número".to_string());
                                break;
                            }
                        }

                        // Amp 2 & Cab 2
                        "amp_2.enabled" | "cab_2.enabled" => {
                            if v.is_boolean() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some(format!("{} deve ser booleano", k));
                                break;
                            }
                        }
                        "amp_2.model" => {
                            if let Some(s) = v.as_str() {
                                if ["jcm800_2203", "plexi_1959", "twin_ab763", "dual_rect", "ac30_tb"].contains(&s) {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("amp_2.model inválido".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("amp_2.model deve ser string".to_string());
                                break;
                            }
                        }
                        "cab_2.speaker" => {
                            if let Some(s) = v.as_str() {
                                if ["greenback_4x12", "v30_4x12", "alnico_blue_2x12"].contains(&s) {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("cab_2.speaker inválido".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("cab_2.speaker deve ser string".to_string());
                                break;
                            }
                        }
                        k if k.starts_with("eq_1.band") => {
                            let rest = &k["eq_1.band".len()..];
                            let mut parts = rest.split('.');
                            if let (Some(num_s), Some(prop)) = (parts.next(), parts.next()) {
                                if let Ok(n) = num_s.parse::<usize>() {
                                    if (1..=8).contains(&n) {
                                        match prop {
                                            "freq_hz" => {
                                                if let Some(num) = v.as_f64() {
                                                    if num >= 20.0 && num <= 20000.0 {
                                                        applied.push(k.to_string());
                                                    } else {
                                                        validation_error = Some(format!("{} fora da faixa 20..20000", k));
                                                        break;
                                                    }
                                                } else {
                                                    validation_error = Some(format!("{} deve ser número", k));
                                                    break;
                                                }
                                            }
                                            "gain_db" => {
                                                if let Some(num) = v.as_f64() {
                                                    if num >= -15.0 && num <= 15.0 {
                                                        applied.push(k.to_string());
                                                    } else {
                                                        validation_error = Some(format!("{} fora da faixa -15..15", k));
                                                        break;
                                                    }
                                                } else {
                                                    validation_error = Some(format!("{} deve ser número", k));
                                                    break;
                                                }
                                            }
                                            "q" => {
                                                if let Some(num) = v.as_f64() {
                                                    if num >= 0.3 && num <= 10.0 {
                                                        applied.push(k.to_string());
                                                    } else {
                                                        validation_error = Some(format!("{} fora da faixa 0.3..10", k));
                                                        break;
                                                    }
                                                } else {
                                                    validation_error = Some(format!("{} deve ser número", k));
                                                    break;
                                                }
                                            }
                                            _ => { ignored.push(k.to_string()); }
                                        }
                                        continue;
                                    }
                                }
                            }
                            ignored.push(k.to_string());
                        }
                        "scene.active" => {
                            if let Some(s) = v.as_str() {
                                if ["A", "B", "C", "D", "E", "F", "G", "H"].contains(&s) {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("scene.active fora de A..H".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("scene.active deve ser string".to_string());
                                break;
                            }
                        }
                        k if k.ends_with(".channel") => {
                            if let Some(s) = v.as_str() {
                                if ["A", "B", "C", "D"].contains(&s) {
                                    applied.push(k.to_string());
                                } else {
                                    validation_error = Some(format!("{} fora de A..D", k));
                                    break;
                                }
                            } else {
                                validation_error = Some(format!("{} deve ser string", k));
                                break;
                            }
                        }
                        "tuner.enabled" => {
                            if v.is_boolean() {
                                applied.push(k.clone());
                            } else {
                                validation_error = Some("tuner.enabled deve ser booleano".to_string());
                                break;
                            }
                        }
                        "tuner.output" => {
                            if let Some(s) = v.as_str() {
                                if s == "mute" || s == "bypass" {
                                    applied.push(k.clone());
                                } else {
                                    validation_error = Some("tuner.output deve ser mute ou bypass".to_string());
                                    break;
                                }
                            } else {
                                validation_error = Some("tuner.output deve ser string".to_string());
                                break;
                            }
                        }
                        _ => {
                            ignored.push(k.clone());
                        }
                    }
                }

                if let Some(err) = validation_error {
                    let err_json = format!("{{\"error\":\"{}\"}}", err);
                    (400, "application/json", err_json.into_bytes())
                } else {
                    // Aplica no AppState apenas os parâmetros aceitos
                    if let Ok(mut st) = app_state_clone.lock() {
                        if let Some(m_val) = map.get("amp_1.model").and_then(|v| v.as_str()) {
                            let nom = nominal_chassis_for_model(m_val);
                            st.amp_model = m_val.to_string();
                            st.preamp_tube = nom.0.to_string();
                            st.power_tube = nom.1.to_string();
                            st.bias_ma = nom.2;
                            st.sag = nom.3;
                            st.rectifier = nom.4.to_string();
                            st.nfb = nom.5;
                            st.xformer_sat = nom.6;
                            st.preamp_plate_v = nom.7;
                            st.triode_asym = nom.8;
                            st.tonestack = nom.9.to_string();
                            st.is_modded = false;
                        }

                        for (k, v) in &map {
                            if !applied.contains(k) {
                                continue;
                            }
                            match k.as_str() {
                                "amp_1.gain" => st.amp_gain = v.as_f64().unwrap() as f32,
                                "amp_1.overdrive" => st.amp_overdrive = v.as_f64().unwrap() as f32,
                                "amp_1.bass" => st.amp_bass = v.as_f64().unwrap() as f32,
                                "amp_1.middle" => st.amp_middle = v.as_f64().unwrap() as f32,
                                "amp_1.treble" => st.amp_treble = v.as_f64().unwrap() as f32,
                                "amp_1.presence" => st.amp_presence = v.as_f64().unwrap() as f32,
                                "amp_1.depth" => st.amp_depth = v.as_f64().unwrap() as f32,
                                "amp_1.master" => st.amp_master = v.as_f64().unwrap() as f32,
                                "amp_1.volume_2" => st.amp_volume_2 = v.as_f64().unwrap() as f32,
                                "amp_1.cut" => st.amp_cut = v.as_f64().unwrap() as f32,
                                "amp_1.model" => { /* já tratado acima */ }
                                "engine.oversampling" => st.engine_oversampling = v.as_str().unwrap().to_string(),
                                "input.boost" => st.input_boost = v.as_f64().unwrap() as f32,
                                "gate.threshold_db" => st.gate_threshold_db = v.as_f64().unwrap() as f32,
                                "amp_1.bright" => st.amp_bright = v.as_bool().unwrap(),
                                "drive.enabled" => st.drive_enabled = v.as_bool().unwrap(),
                                "drive.drive" => st.drive_drive = v.as_f64().unwrap() as f32,
                                "drive.tone" => st.drive_tone = v.as_f64().unwrap() as f32,
                                "drive.level" => st.drive_level = v.as_f64().unwrap() as f32,
                                "gate.enabled" => st.gate_enabled = v.as_bool().unwrap(),
                                "amp_1.enabled" => st.amp_enabled = v.as_bool().unwrap(),
                                "cab_1.enabled" => st.cab_enabled = v.as_bool().unwrap(),
                                "input.source" => st.input_source = v.as_str().unwrap().to_string(),
                                "amp_1.bias_ma" => st.bias_ma = v.as_f64().unwrap() as f32,
                                "amp_1.sag" => st.sag = v.as_f64().unwrap() as f32,
                                "amp_1.nfb" => st.nfb = v.as_f64().unwrap() as f32,
                                "amp_1.xformer_sat" => st.xformer_sat = v.as_f64().unwrap() as f32,
                                "amp_1.preamp_plate_v" => st.preamp_plate_v = v.as_f64().unwrap() as f32,
                                "amp_1.triode_asym" => st.triode_asym = v.as_f64().unwrap() as f32,
                                "amp_1.preamp_tube" => st.preamp_tube = v.as_str().unwrap().to_string(),
                                "amp_1.power_tube" => st.power_tube = v.as_str().unwrap().to_string(),
                                "amp_1.rectifier" => st.rectifier = v.as_str().unwrap().to_string(),
                                "amp_1.tonestack" => st.tonestack = v.as_str().unwrap().to_string(),
                                "cab_1.speaker" => {
                                    st.cab_speaker = v.as_str().unwrap().to_string();
                                    st.cab_resonance_hz = match st.cab_speaker.as_str() {
                                        "greenback_4x12" => 98.0,
                                        "v30_4x12" => 108.0,
                                        "alnico_blue_2x12" => 82.0,
                                        _ => 98.0,
                                    };
                                }
                                "cab_1.resonance_hz" => st.cab_resonance_hz = v.as_f64().unwrap() as f32,
                                "cab_1.hicut_khz" => st.cab_hicut_khz = v.as_f64().unwrap() as f32,
                                "cab_1.ir_enabled" => st.cab_ir_enabled = v.as_bool().unwrap(),
                                "cab_1.mic1.model" => st.cab_mic1_model = v.as_str().unwrap().to_string(),
                                "cab_1.mic1.position" => st.cab_mic1_position = v.as_f64().unwrap() as f32,
                                "cab_1.mic1.distance_cm" => st.cab_mic1_distance_cm = v.as_f64().unwrap() as f32,
                                "cab_1.mic1.level_db" => st.cab_mic1_level_db = v.as_f64().unwrap() as f32,
                                "cab_1.mic1.enabled" => st.cab_mic1_enabled = v.as_bool().unwrap(),
                                "cab_1.mic2.model" => st.cab_mic2_model = v.as_str().unwrap().to_string(),
                                "cab_1.mic2.position" => st.cab_mic2_position = v.as_f64().unwrap() as f32,
                                "cab_1.mic2.distance_cm" => st.cab_mic2_distance_cm = v.as_f64().unwrap() as f32,
                                "cab_1.mic2.level_db" => st.cab_mic2_level_db = v.as_f64().unwrap() as f32,
                                "cab_1.mic2.enabled" => st.cab_mic2_enabled = v.as_bool().unwrap(),
                                "split.enabled" => st.split_enabled = v.as_bool().unwrap(),
                                "split.mode" => st.split_mode = v.as_str().unwrap().to_string(),
                                "split.crossover_hz" => st.split_crossover_hz = v.as_f64().unwrap() as f32,
                                "merge.row_1_level_db" => st.merge_row_1_level_db = v.as_f64().unwrap() as f32,
                                "merge.row_1_phase_invert" => st.merge_row_1_phase_invert = v.as_bool().unwrap(),
                                "comp_2.enabled" => st.comp_2_enabled = v.as_bool().unwrap(),
                                "comp_2.input" => st.comp_2_input = v.as_f64().unwrap() as f32,
                                "comp_2.output" => st.comp_2_output = v.as_f64().unwrap() as f32,
                                "comp_2.attack" => st.comp_2_attack = v.as_f64().unwrap() as f32,
                                "comp_2.release" => st.comp_2_release = v.as_f64().unwrap() as f32,
                                "comp_2.ratio" => st.comp_2_ratio = v.as_str().unwrap().to_string(),
                                "eq_1.enabled" => st.eq_1_enabled = v.as_bool().unwrap(),
                                "mod_2.enabled" => st.mod_2_enabled = v.as_bool().unwrap(),
                                "mod_2.rate_hz" => st.mod_2_rate_hz = v.as_f64().unwrap() as f32,
                                "mod_2.depth" => st.mod_2_depth = v.as_f64().unwrap() as f32,
                                "mod_2.mix" => st.mod_2_mix = v.as_f64().unwrap() as f32,
                                "dly_1.enabled" => st.dly_1_enabled = v.as_bool().unwrap(),
                                "dly_1.heads" => st.dly_1_heads = v.as_str().unwrap().to_string(),
                                "dly_1.repeat_rate" => st.dly_1_repeat_rate = v.as_f64().unwrap() as f32,
                                "dly_1.intensity" => st.dly_1_intensity = v.as_f64().unwrap() as f32,
                                "dly_1.echo_volume" => st.dly_1_echo_volume = v.as_f64().unwrap() as f32,
                                "dly_1.bass" => st.dly_1_bass = v.as_f64().unwrap() as f32,
                                "dly_1.treble" => st.dly_1_treble = v.as_f64().unwrap() as f32,
                                "dly_1.spillover" => st.dly_1_spillover = v.as_bool().unwrap(),
                                "rev_1.enabled" => st.rev_1_enabled = v.as_bool().unwrap(),
                                "rev_1.decay_s" => st.rev_1_decay_s = v.as_f64().unwrap() as f32,
                                "rev_1.predelay_ms" => st.rev_1_predelay_ms = v.as_f64().unwrap() as f32,
                                "rev_1.damping" => st.rev_1_damping = v.as_f64().unwrap() as f32,
                                "rev_1.mix" => st.rev_1_mix = v.as_f64().unwrap() as f32,
                                "rev_1.spillover" => st.rev_1_spillover = v.as_bool().unwrap(),
                                "amp_2.enabled" => st.amp_2_enabled = v.as_bool().unwrap(),
                                "amp_2.model" => st.amp_2_model = v.as_str().unwrap().to_string(),
                                "cab_2.enabled" => st.cab_2_enabled = v.as_bool().unwrap(),
                                "cab_2.speaker" => st.cab_2_speaker = v.as_str().unwrap().to_string(),
                                "scene.active" => st.scene_active = v.as_str().unwrap().to_string(),
                                "tuner.enabled" => st.tuner_enabled = v.as_bool().unwrap(),
                                "tuner.output" => st.tuner_output = v.as_str().unwrap().to_string(),
                                _ => {}
                            }
                        }

                        // Recalcula se o chassi está modificado em relação ao modelo nominal
                        st.is_modded = st.check_is_modded();
                    }

                    // Aplica no DSP Chain apenas os parâmetros aceitos
                    if let Some(ref dsp_ref) = dsp_chain_clone {
                        if let Ok(mut dsp) = dsp_ref.lock() {
                            for (k, v) in &map {
                                if !applied.contains(k) {
                                    continue;
                                }
                                match k.as_str() {
                                    "amp_1.gain" => dsp.set_amp_gain(v.as_f64().unwrap() as f32),
                                    "amp_1.overdrive" => dsp.set_amp_overdrive(v.as_f64().unwrap() as f32),
                                    "amp_1.bass" => dsp.set_amp_bass(v.as_f64().unwrap() as f32),
                                    "amp_1.middle" => dsp.set_amp_middle(v.as_f64().unwrap() as f32),
                                    "amp_1.treble" => dsp.set_amp_treble(v.as_f64().unwrap() as f32),
                                    "amp_1.presence" => dsp.set_amp_presence(v.as_f64().unwrap() as f32),
                                    "amp_1.depth" => dsp.set_amp_depth(v.as_f64().unwrap() as f32),
                                    "amp_1.master" => dsp.set_amp_master(v.as_f64().unwrap() as f32),
                                    "amp_1.volume_2" => dsp.set_amp_volume_2(v.as_f64().unwrap() as f32),
                                    "amp_1.cut" => dsp.set_amp_cut(v.as_f64().unwrap() as f32),
                                    "amp_1.model" => dsp.set_amp_model(v.as_str().unwrap()),
                                    "engine.oversampling" => dsp.set_oversampling_factor(v.as_str().unwrap()),
                                    "input.boost" => dsp.set_input_boost(v.as_f64().unwrap() as f32),
                                    "gate.threshold_db" => dsp.set_gate_threshold_db(v.as_f64().unwrap() as f32),
                                    "amp_1.bright" => dsp.set_amp_bright(v.as_bool().unwrap()),
                                    "drive.enabled" => dsp.set_drive_enabled(v.as_bool().unwrap()),
                                    "drive.drive" => dsp.set_drive_gain(v.as_f64().unwrap() as f32),
                                    "drive.tone" => dsp.set_drive_tone(v.as_f64().unwrap() as f32),
                                    "drive.level" => dsp.set_drive_level(v.as_f64().unwrap() as f32),
                                    "gate.enabled" => dsp.set_gate_enabled(v.as_bool().unwrap()),
                                    "amp_1.enabled" => dsp.set_amp_enabled(v.as_bool().unwrap()),
                                    "cab_1.enabled" => dsp.set_cab_enabled(v.as_bool().unwrap()),
                                    "input.source" => dsp.set_input_source(v.as_str().unwrap()),
                                    "amp_1.bias_ma" => dsp.set_amp_bias_ma(v.as_f64().unwrap() as f32),
                                    "amp_1.sag" => dsp.set_amp_sag(v.as_f64().unwrap() as f32),
                                    "amp_1.nfb" => dsp.set_amp_nfb(v.as_f64().unwrap() as f32),
                                    "amp_1.xformer_sat" => dsp.set_amp_xformer_sat(v.as_f64().unwrap() as f32),
                                    "amp_1.preamp_plate_v" => dsp.set_amp_preamp_plate_v(v.as_f64().unwrap() as f32),
                                    "amp_1.triode_asym" => dsp.set_amp_triode_asym(v.as_f64().unwrap() as f32),
                                    "amp_1.preamp_tube" => dsp.set_amp_preamp_tube(v.as_str().unwrap()),
                                    "amp_1.power_tube" => dsp.set_amp_power_tube(v.as_str().unwrap()),
                                    "amp_1.rectifier" => dsp.set_amp_rectifier(v.as_str().unwrap()),
                                    "amp_1.tonestack" => dsp.set_amp_tonestack(v.as_str().unwrap()),
                                    "cab_1.speaker" => dsp.set_cab_speaker(v.as_str().unwrap()),
                                    "cab_1.resonance_hz" => dsp.set_cab_resonance_hz(v.as_f64().unwrap() as f32),
                                    "cab_1.hicut_khz" => dsp.set_cab_hicut_khz(v.as_f64().unwrap() as f32),
                                    "cab_1.ir_enabled" => dsp.set_cab_ir_enabled(v.as_bool().unwrap()),
                                    "cab_1.mic1.model" => dsp.set_cab_mic1_model(v.as_str().unwrap()),
                                    "cab_1.mic1.position" => dsp.set_cab_mic1_position(v.as_f64().unwrap() as f32),
                                    "cab_1.mic1.distance_cm" => dsp.set_cab_mic1_distance_cm(v.as_f64().unwrap() as f32),
                                    "cab_1.mic1.level_db" => dsp.set_cab_mic1_level_db(v.as_f64().unwrap() as f32),
                                    "cab_1.mic1.enabled" => dsp.set_cab_mic1_enabled(v.as_bool().unwrap()),
                                    "cab_1.mic2.model" => dsp.set_cab_mic2_model(v.as_str().unwrap()),
                                    "cab_1.mic2.position" => dsp.set_cab_mic2_position(v.as_f64().unwrap() as f32),
                                    "cab_1.mic2.distance_cm" => dsp.set_cab_mic2_distance_cm(v.as_f64().unwrap() as f32),
                                    "cab_1.mic2.level_db" => dsp.set_cab_mic2_level_db(v.as_f64().unwrap() as f32),
                                    "cab_1.mic2.enabled" => dsp.set_cab_mic2_enabled(v.as_bool().unwrap()),
                                    "split.enabled" => dsp.set_split_enabled(v.as_bool().unwrap()),
                                    "split.mode" => {
                                        let mode = v.as_str().unwrap();
                                        let hz = map.get("split.crossover_hz").and_then(|x| x.as_f64()).map(|x| x as f32).unwrap_or(650.0);
                                        dsp.set_split_mode(mode, hz);
                                    }
                                    "split.crossover_hz" => {
                                        let hz = v.as_f64().unwrap() as f32;
                                        let mode = map.get("split.mode").and_then(|x| x.as_str()).unwrap_or("crossover");
                                        dsp.set_split_mode(mode, hz);
                                    }
                                    "merge.row_1_level_db" => {
                                        let db = v.as_f64().unwrap() as f32;
                                        let inv = map.get("merge.row_1_phase_invert").and_then(|x| x.as_bool()).unwrap_or(false);
                                        dsp.set_merge_config(db, inv);
                                    }
                                    "merge.row_1_phase_invert" => {
                                        let inv = v.as_bool().unwrap();
                                        let db = map.get("merge.row_1_level_db").and_then(|x| x.as_f64()).map(|x| x as f32).unwrap_or(-6.0);
                                        dsp.set_merge_config(db, inv);
                                    }
                                    "comp_2.enabled" => dsp.set_comp_2_enabled(v.as_bool().unwrap()),
                                    "comp_2.input" => dsp.set_comp_2_input(v.as_f64().unwrap() as f32),
                                    "comp_2.output" => dsp.set_comp_2_output(v.as_f64().unwrap() as f32),
                                    "comp_2.attack" => dsp.set_comp_2_attack(v.as_f64().unwrap() as f32),
                                    "comp_2.release" => dsp.set_comp_2_release(v.as_f64().unwrap() as f32),
                                    "comp_2.ratio" => dsp.set_comp_2_ratio(v.as_str().unwrap()),
                                    "eq_1.enabled" => dsp.set_eq_1_enabled(v.as_bool().unwrap()),
                                    "mod_2.enabled" => dsp.set_mod_2_enabled(v.as_bool().unwrap()),
                                    "mod_2.rate_hz" => dsp.set_mod_2_rate_hz(v.as_f64().unwrap() as f32),
                                    "mod_2.depth" => dsp.set_mod_2_depth(v.as_f64().unwrap() as f32),
                                    "mod_2.mix" => dsp.set_mod_2_mix(v.as_f64().unwrap() as f32),
                                    "dly_1.enabled" => dsp.set_dly_1_enabled(v.as_bool().unwrap()),
                                    "dly_1.repeat_rate" => dsp.set_dly_1_repeat_rate(v.as_f64().unwrap() as f32),
                                    "dly_1.intensity" => dsp.set_dly_1_intensity(v.as_f64().unwrap() as f32),
                                    "dly_1.echo_volume" => dsp.set_dly_1_echo_volume(v.as_f64().unwrap() as f32),
                                    "dly_1.spillover" => dsp.set_dly_1_spillover(v.as_bool().unwrap()),
                                    "rev_1.enabled" => dsp.set_rev_1_enabled(v.as_bool().unwrap()),
                                    "rev_1.decay_s" => dsp.set_rev_1_decay_s(v.as_f64().unwrap() as f32),
                                    "rev_1.predelay_ms" => dsp.set_rev_1_predelay_ms(v.as_f64().unwrap() as f32),
                                    "rev_1.damping" => dsp.set_rev_1_damping(v.as_f64().unwrap() as f32),
                                    "rev_1.mix" => dsp.set_rev_1_mix(v.as_f64().unwrap() as f32),
                                    "rev_1.spillover" => dsp.set_rev_1_spillover(v.as_bool().unwrap()),
                                    "amp_2.enabled" => dsp.set_amp_2_enabled(v.as_bool().unwrap()),
                                    "amp_2.model" => dsp.set_amp_2_model(v.as_str().unwrap()),
                                    "cab_2.enabled" => dsp.set_cab_2_enabled(v.as_bool().unwrap()),
                                    "cab_2.speaker" => dsp.set_cab_2_speaker(v.as_str().unwrap()),
                                    "tuner.enabled" => dsp.set_tuner_enabled(v.as_bool().unwrap()),
                                    "tuner.output" => {
                                        let m = if v.as_str().unwrap() == "bypass" {
                                            ampforge_dsp::tuner::TunerOutputMode::Bypass
                                        } else {
                                            ampforge_dsp::tuner::TunerOutputMode::Mute
                                        };
                                        dsp.set_tuner_output_mode(m);
                                    }
                                    k if k.starts_with("eq_1.band") => {
                                        let rest = &k["eq_1.band".len()..];
                                        let mut parts = rest.split('.');
                                        if let (Some(num_s), Some(prop)) = (parts.next(), parts.next()) {
                                            if let Ok(n) = num_s.parse::<usize>() {
                                                if (1..=8).contains(&n) {
                                                    let idx = n - 1;
                                                    let cur = dsp.eq_1_band(idx);
                                                    let mut f = cur.freq_hz;
                                                    let mut g = cur.gain_db;
                                                    let mut q = cur.q;
                                                    match prop {
                                                        "freq_hz" => f = v.as_f64().unwrap() as f32,
                                                        "gain_db" => g = v.as_f64().unwrap() as f32,
                                                        "q" => q = v.as_f64().unwrap() as f32,
                                                        _ => {}
                                                    }
                                                    dsp.set_eq_1_band(idx, f, g, q);
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    let mut resp_json = serde_json::json!({
                        "status": "ok",
                        "applied": applied,
                        "ignored": ignored
                    });
                    if !ignored_reasons.is_empty() {
                        resp_json["ignored_reasons"] = serde_json::Value::Object(ignored_reasons);
                    }
                    (200, "application/json", resp_json.to_string().into_bytes())
                }
            }
            _ => (400, "application/json", b"{\"error\":\"Corpo deve ser objeto JSON\"}".to_vec()),
        }
    }

    fn handle_post_di(
        request: &mut tiny_http::Request,
        dsp_chain: &Option<Arc<Mutex<RealtimeDspChain>>>,
    ) -> (u16, &'static str, Vec<u8>) {
        let reader = request.as_reader();
        let mut body_bytes = Vec::new();
        if let Err(e) = reader.read_to_end(&mut body_bytes) {
            return (400, "application/json", format!("{{\"error\":\"{}\"}}", e).into_bytes());
        }
        if body_bytes.len() > 20 * 1024 * 1024 {
            return (413, "text/plain", b"Payload Too Large".to_vec());
        }

        let loader = ampforge_dsp::cab::WavIrLoader::new(48000.0);
        match loader.load_wav_unlimited(&body_bytes) {
            Ok(ir) => {
                if let Some(ref dsp_ref) = dsp_chain {
                    if let Ok(mut dsp) = dsp_ref.lock() {
                        dsp.load_di_wav(ir.samples().to_vec());
                        dsp.set_input_source("di_file");
                    }
                }
                (200, "application/json", b"{\"status\":\"ok\",\"loaded\":true}".to_vec())
            }
            Err(e) => {
                (400, "application/json", format!("{{\"error\":\"WAV inválido: {:?}\"}}", e).into_bytes())
            }
        }
    }

    fn handle_post_cab_ir(
        request: &mut tiny_http::Request,
        app_state_clone: &Arc<Mutex<AppState>>,
        dsp_chain: &Option<Arc<Mutex<RealtimeDspChain>>>,
    ) -> (u16, &'static str, Vec<u8>) {
        let mut body_bytes = Vec::new();
        let reader = request.as_reader();
        if let Err(e) = reader.read_to_end(&mut body_bytes) {
            return (400, "application/json", format!("{{\"error\":\"{}\"}}", e).into_bytes());
        }
        if body_bytes.len() > 20 * 1024 * 1024 {
            return (413, "text/plain", b"Payload Too Large".to_vec());
        }

        let loader = ampforge_dsp::cab::WavIrLoader::new(48000.0);
        match loader.load_from_bytes(&body_bytes) {
            Ok(_) => {
                if let Some(ref dsp_ref) = dsp_chain {
                    if let Ok(mut dsp) = dsp_ref.lock() {
                        let _ = dsp.load_cab_ir(&body_bytes);
                        dsp.set_cab_ir_enabled(true);
                    }
                }
                if let Ok(mut st) = app_state_clone.lock() {
                    st.cab_ir_enabled = true;
                }
                (200, "application/json", b"{\"status\":\"ok\",\"loaded\":true}".to_vec())
            }
            Err(e) => {
                let err_msg = format!("{}", e);
                let err_code = if err_msg.contains("ir_too_long") {
                    "ir_too_long"
                } else {
                    "invalid_wav"
                };
                let resp = serde_json::json!({
                    "error": err_code,
                    "message": err_msg,
                    "limit_ms": 500
                });
                (400, "application/json", serde_json::to_vec(&resp).unwrap())
            }
        }
    }

    fn verify_csrf_and_token<F>(
        request: &mut tiny_http::Request,
        session_token: &str,
        handler: F,
    ) -> (u16, &'static str, Vec<u8>)
    where
        F: FnOnce(&mut tiny_http::Request) -> (u16, &'static str, Vec<u8>),
    {
        let origin_hdr = request.headers().iter().find(|h| {
            h.field.as_str().as_str().eq_ignore_ascii_case("origin")
        });
        let sec_fetch = request.headers().iter().find(|h| {
            h.field.as_str().as_str().eq_ignore_ascii_case("sec-fetch-site")
        });

        let is_valid_origin = match origin_hdr {
            Some(orig) => {
                let v = orig.value.as_str();
                v.starts_with("http://127.0.0.1") || v.starts_with("http://localhost")
            }
            None => {
                if let Some(sec) = sec_fetch {
                    sec.value.as_str() != "cross-site"
                } else {
                    true
                }
            }
        };

        if !is_valid_origin {
            Self::drain_request_body(request);
            return (403, "text/plain", b"Forbidden Origin".to_vec());
        }

        let token_hdr = request.headers().iter().find(|h| {
            h.field.as_str().as_str().eq_ignore_ascii_case("x-sonix-token")
        });
        let has_valid_token = match token_hdr {
            Some(th) => th.value.as_str() == session_token,
            None => false,
        };

        if !has_valid_token {
            Self::drain_request_body(request);
            return (401, "text/plain", b"Unauthorized: missing or invalid X-Sonix-Token".to_vec());
        }

        handler(request)
    }

    fn find_preset_file(id: &str) -> Option<std::path::PathBuf> {
        let clean_id = id.trim_end_matches(".sonixpreset");
        let candidate_dirs = [
            "presets/factory",
            "../../presets/factory",
            "../presets/factory",
            "presets/user",
            "../../presets/user",
            "../presets/user",
        ];
        for dir in candidate_dirs {
            let p = std::path::PathBuf::from(format!("{}/{}.sonixpreset", dir, clean_id));
            if p.exists() {
                return Some(p);
            }
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            let appdata_path = std::path::PathBuf::from(appdata)
                .join("SonixPro")
                .join("presets")
                .join(format!("{}.sonixpreset", clean_id));
            if appdata_path.exists() {
                return Some(appdata_path);
            }
        }
        None
    }

    fn handle_get_presets() -> (u16, &'static str, Vec<u8>) {
        let mut list = Vec::new();
        let factory_defs = [
            ("brit800", "Brit 800 Crunch"),
            ("usdeluxe", "US Deluxe Clean"),
            ("recto", "Recto High Gain"),
            ("ac30", "AC30 Top Boost"),
        ];
        for (id, name) in factory_defs {
            list.push(serde_json::json!({
                "id": id,
                "name": name,
                "type": "factory"
            }));
        }

        let mut scan_dir = |dir: std::path::PathBuf| {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("sonixpreset") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if !factory_defs.iter().any(|(fid, _)| *fid == stem) {
                                list.push(serde_json::json!({
                                    "id": stem,
                                    "name": stem,
                                    "type": "user"
                                }));
                            }
                        }
                    }
                }
            }
        };
        scan_dir(std::path::PathBuf::from("presets/user"));
        if let Ok(appdata) = std::env::var("APPDATA") {
            scan_dir(std::path::PathBuf::from(appdata).join("SonixPro").join("presets"));
        }

        let body = serde_json::to_vec(&list).unwrap();
        (200, "application/json", body)
    }

    fn handle_post_presets_load(
        request: &mut tiny_http::Request,
        app_state_clone: &Arc<Mutex<AppState>>,
        dsp_chain: &Option<Arc<Mutex<RealtimeDspChain>>>,
    ) -> (u16, &'static str, Vec<u8>) {
        let mut body_bytes = Vec::new();
        let _ = request.as_reader().read_to_end(&mut body_bytes);
        let Ok(v) = serde_json::from_slice::<serde_json::Value>(&body_bytes) else {
            return (400, "application/json", b"{\"error\":\"invalid_json\"}".to_vec());
        };
        let Some(id) = v.get("id").and_then(|id_v| id_v.as_str()) else {
            return (400, "application/json", b"{\"error\":\"missing_id\"}".to_vec());
        };

        let Some(path) = Self::find_preset_file(id) else {
            return (404, "application/json", b"{\"error\":\"preset_not_found\"}".to_vec());
        };

        let Ok(content) = std::fs::read_to_string(&path) else {
            return (500, "application/json", b"{\"error\":\"failed_to_read_preset\"}".to_vec());
        };

        let Ok(doc) = sonix_core::persistence::SonixPresetDoc::from_json_str(&content) else {
            return (400, "application/json", b"{\"error\":\"invalid_preset_schema\"}".to_vec());
        };

        if let Some(ref params) = doc.params {
            Self::apply_preset_params(params, app_state_clone, dsp_chain);
        }

        if let Ok(mut st) = app_state_clone.lock() {
            st.active_preset_id = id.to_string();
            st.active_preset_name = doc.metadata.name.clone();
        }

        (200, "application/json", serde_json::to_vec(&serde_json::json!({
            "status": "ok",
            "loaded": id,
            "name": doc.metadata.name
        })).unwrap())
    }

    fn handle_post_presets_save(
        request: &mut tiny_http::Request,
        app_state_clone: &Arc<Mutex<AppState>>,
    ) -> (u16, &'static str, Vec<u8>) {
        let mut body_bytes = Vec::new();
        let _ = request.as_reader().read_to_end(&mut body_bytes);
        let Ok(v) = serde_json::from_slice::<serde_json::Value>(&body_bytes) else {
            return (400, "application/json", b"{\"error\":\"invalid_json\"}".to_vec());
        };
        let Some(id) = v.get("id").and_then(|id_v| id_v.as_str()) else {
            return (400, "application/json", b"{\"error\":\"missing_id\"}".to_vec());
        };
        let name = v.get("name").and_then(|n| n.as_str()).unwrap_or(id);

        let save_dir = if let Ok(appdata) = std::env::var("APPDATA") {
            let p = std::path::PathBuf::from(appdata).join("SonixPro").join("presets");
            let _ = std::fs::create_dir_all(&p);
            p
        } else {
            let p = std::path::PathBuf::from("presets/user");
            let _ = std::fs::create_dir_all(&p);
            p
        };

        let file_path = save_dir.join(format!("{}.sonixpreset", id));

        let current_state = if let Ok(st) = app_state_clone.lock() {
            st.clone()
        } else {
            AppState::default()
        };

        let state_val = serde_json::to_value(&current_state).unwrap_or_default();
        let mut params_map = std::collections::HashMap::new();
        if let serde_json::Value::Object(map) = state_val {
            for (k, val) in map {
                params_map.insert(k, val);
            }
        }

        let doc = sonix_core::persistence::SonixPresetDoc {
            schema: "https://sonixpro.local/schemas/sonixpreset-v1.json".to_string(),
            schema_version: "1.1.0".to_string(),
            metadata: sonix_core::persistence::PresetMetadata {
                id: id.to_string(),
                name: name.to_string(),
                author: "User".to_string(),
                category: "Custom".to_string(),
                tempo_bpm: 120.0,
                created_at: "2026-11-11T00:00:00Z".to_string(),
                footswitch_mode: "scenes".to_string(),
            },
            active_scene_index: 0,
            scenes: vec![],
            grid: sonix_core::persistence::GridDto { rows: 4, cols: 12, blocks: vec![] },
            modifiers: vec![],
            params: Some(params_map),
            channels: None,
        };

        let Ok(json_str) = doc.to_json_str() else {
            return (500, "application/json", b"{\"error\":\"failed_to_serialize\"}".to_vec());
        };

        if let Err(e) = std::fs::write(&file_path, json_str) {
            return (500, "application/json", format!("{{\"error\":\"failed_to_write_file\",\"message\":\"{}\"}}", e).into_bytes());
        }

        (200, "application/json", serde_json::to_vec(&serde_json::json!({
            "status": "ok",
            "saved": id
        })).unwrap())
    }

    fn handle_post_presets_import(
        request: &mut tiny_http::Request,
        app_state_clone: &Arc<Mutex<AppState>>,
        dsp_chain: &Option<Arc<Mutex<RealtimeDspChain>>>,
    ) -> (u16, &'static str, Vec<u8>) {
        let mut body_bytes = Vec::new();
        let _ = request.as_reader().read_to_end(&mut body_bytes);
        let Ok(content) = std::str::from_utf8(&body_bytes) else {
            return (400, "application/json", b"{\"error\":\"invalid_utf8\"}".to_vec());
        };

        let doc = match sonix_core::persistence::SonixPresetDoc::from_json_str(content) {
            Ok(d) => d,
            Err(e) => {
                return (400, "application/json", format!("{{\"error\":\"invalid_preset\",\"message\":\"{}\"}}", e).into_bytes());
            }
        };

        if let Some(ref params) = doc.params {
            Self::apply_preset_params(params, app_state_clone, dsp_chain);
        }

        if let Ok(mut st) = app_state_clone.lock() {
            st.active_preset_id = doc.metadata.id.clone();
            st.active_preset_name = doc.metadata.name.clone();
        }

        (200, "application/json", serde_json::to_vec(&serde_json::json!({
            "status": "ok",
            "imported": doc.metadata.id
        })).unwrap())
    }

    fn handle_get_presets_export(preset_id: &str) -> (u16, &'static str, Vec<u8>) {
        let Some(path) = Self::find_preset_file(preset_id) else {
            return (404, "application/json", b"{\"error\":\"preset_not_found\"}".to_vec());
        };
        let Ok(bytes) = std::fs::read(path) else {
            return (500, "application/json", b"{\"error\":\"failed_to_read\"}".to_vec());
        };
        (200, "application/json", bytes)
    }

    fn handle_post_scenes_select(
        request: &mut tiny_http::Request,
        app_state_clone: &Arc<Mutex<AppState>>,
    ) -> (u16, &'static str, Vec<u8>) {
        let mut body_bytes = Vec::new();
        let _ = request.as_reader().read_to_end(&mut body_bytes);
        let Ok(v) = serde_json::from_slice::<serde_json::Value>(&body_bytes) else {
            return (400, "application/json", b"{\"error\":\"invalid_json\"}".to_vec());
        };
        let Some(scene) = v.get("scene").and_then(|s| s.as_str()) else {
            return (400, "application/json", b"{\"error\":\"missing_scene\"}".to_vec());
        };

        if !["A", "B", "C", "D", "E", "F", "G", "H"].contains(&scene) {
            return (400, "application/json", b"{\"error\":\"invalid_scene\"}".to_vec());
        }

        let start = std::time::Instant::now();
        if let Ok(mut st) = app_state_clone.lock() {
            st.scene_active = scene.to_string();
        }
        let elapsed_ms = (start.elapsed().as_micros() as f32 / 1000.0).max(0.1);

        (200, "application/json", serde_json::to_vec(&serde_json::json!({
            "status": "ok",
            "scene": scene,
            "transition_ms": (elapsed_ms * 10.0).round() / 10.0
        })).unwrap())
    }

    fn apply_preset_params(
        params: &std::collections::HashMap<String, serde_json::Value>,
        app_state: &Arc<Mutex<AppState>>,
        dsp_chain: &Option<Arc<Mutex<RealtimeDspChain>>>,
    ) {
        if let Ok(mut st) = app_state.lock() {
            for (k, v) in params {
                match k.as_str() {
                    "amp_1.gain" => if let Some(n) = v.as_f64() { st.amp_gain = n as f32; },
                    "amp_1.overdrive" => if let Some(n) = v.as_f64() { st.amp_overdrive = n as f32; },
                    "amp_1.bass" => if let Some(n) = v.as_f64() { st.amp_bass = n as f32; },
                    "amp_1.middle" => if let Some(n) = v.as_f64() { st.amp_middle = n as f32; },
                    "amp_1.treble" => if let Some(n) = v.as_f64() { st.amp_treble = n as f32; },
                    "amp_1.presence" => if let Some(n) = v.as_f64() { st.amp_presence = n as f32; },
                    "amp_1.depth" => if let Some(n) = v.as_f64() { st.amp_depth = n as f32; },
                    "amp_1.master" => if let Some(n) = v.as_f64() { st.amp_master = n as f32; },
                    "amp_1.volume_2" => if let Some(n) = v.as_f64() { st.amp_volume_2 = n as f32; },
                    "amp_1.cut" => if let Some(n) = v.as_f64() { st.amp_cut = n as f32; },
                    "amp_1.model" => if let Some(s) = v.as_str() { st.amp_model = s.to_string(); },
                    "amp_1.bright" => if let Some(b) = v.as_bool() { st.amp_bright = b; },
                    "amp_1.enabled" => if let Some(b) = v.as_bool() { st.amp_enabled = b; },
                    "amp_1.bias_ma" => if let Some(n) = v.as_f64() { st.bias_ma = n as f32; },
                    "amp_1.sag" => if let Some(n) = v.as_f64() { st.sag = n as f32; },
                    "amp_1.nfb" => if let Some(n) = v.as_f64() { st.nfb = n as f32; },
                    "amp_1.xformer_sat" => if let Some(n) = v.as_f64() { st.xformer_sat = n as f32; },
                    "amp_1.preamp_plate_v" => if let Some(n) = v.as_f64() { st.preamp_plate_v = n as f32; },
                    "amp_1.triode_asym" => if let Some(n) = v.as_f64() { st.triode_asym = n as f32; },
                    "amp_1.preamp_tube" => if let Some(s) = v.as_str() { st.preamp_tube = s.to_string(); },
                    "amp_1.power_tube" => if let Some(s) = v.as_str() { st.power_tube = s.to_string(); },
                    "amp_1.rectifier" => if let Some(s) = v.as_str() { st.rectifier = s.to_string(); },
                    "amp_1.tonestack" => if let Some(s) = v.as_str() { st.tonestack = s.to_string(); },
                    "engine.oversampling" => if let Some(s) = v.as_str() { st.engine_oversampling = s.to_string(); },
                    "input.boost" => if let Some(n) = v.as_f64() { st.input_boost = n as f32; },
                    "input.source" => if let Some(s) = v.as_str() { st.input_source = s.to_string(); },
                    "drive.enabled" => if let Some(b) = v.as_bool() { st.drive_enabled = b; },
                    "drive.drive" => if let Some(n) = v.as_f64() { st.drive_drive = n as f32; },
                    "drive.tone" => if let Some(n) = v.as_f64() { st.drive_tone = n as f32; },
                    "drive.level" => if let Some(n) = v.as_f64() { st.drive_level = n as f32; },
                    "gate.enabled" => if let Some(b) = v.as_bool() { st.gate_enabled = b; },
                    "gate.threshold_db" => if let Some(n) = v.as_f64() { st.gate_threshold_db = n as f32; },
                    "cab_1.enabled" => if let Some(b) = v.as_bool() { st.cab_enabled = b; },
                    "cab_1.speaker" | "cab_1.model" => if let Some(s) = v.as_str() { st.cab_speaker = s.to_string(); },
                    "cab_1.resonance_hz" => if let Some(n) = v.as_f64() { st.cab_resonance_hz = n as f32; },
                    "cab_1.hicut_khz" => if let Some(n) = v.as_f64() { st.cab_hicut_khz = n as f32; },
                    "cab_1.ir_enabled" => if let Some(b) = v.as_bool() { st.cab_ir_enabled = b; },
                    "cab_1.mic1.model" => if let Some(s) = v.as_str() { st.cab_mic1_model = s.to_string(); },
                    "cab_1.mic1.position" => if let Some(n) = v.as_f64() { st.cab_mic1_position = n as f32; },
                    "cab_1.mic1.distance_cm" => if let Some(n) = v.as_f64() { st.cab_mic1_distance_cm = n as f32; },
                    "cab_1.mic1.level_db" => if let Some(n) = v.as_f64() { st.cab_mic1_level_db = n as f32; },
                    "cab_1.mic1.enabled" => if let Some(b) = v.as_bool() { st.cab_mic1_enabled = b; },
                    "cab_1.mic2.model" => if let Some(s) = v.as_str() { st.cab_mic2_model = s.to_string(); },
                    "cab_1.mic2.position" => if let Some(n) = v.as_f64() { st.cab_mic2_position = n as f32; },
                    "cab_1.mic2.distance_cm" => if let Some(n) = v.as_f64() { st.cab_mic2_distance_cm = n as f32; },
                    "cab_1.mic2.level_db" => if let Some(n) = v.as_f64() { st.cab_mic2_level_db = n as f32; },
                    "cab_1.mic2.enabled" => if let Some(b) = v.as_bool() { st.cab_mic2_enabled = b; },
                    "split.enabled" => if let Some(b) = v.as_bool() { st.split_enabled = b; },
                    "split.mode" => if let Some(s) = v.as_str() { st.split_mode = s.to_string(); },
                    "split.crossover_hz" => if let Some(n) = v.as_f64() { st.split_crossover_hz = n as f32; },
                    "merge.row_1_level_db" => if let Some(n) = v.as_f64() { st.merge_row_1_level_db = n as f32; },
                    "merge.row_1_phase_invert" => if let Some(b) = v.as_bool() { st.merge_row_1_phase_invert = b; },
                    "comp_2.enabled" => if let Some(b) = v.as_bool() { st.comp_2_enabled = b; },
                    "comp_2.input" => if let Some(n) = v.as_f64() { st.comp_2_input = n as f32; },
                    "comp_2.output" => if let Some(n) = v.as_f64() { st.comp_2_output = n as f32; },
                    "comp_2.attack" => if let Some(n) = v.as_f64() { st.comp_2_attack = n as f32; },
                    "comp_2.release" => if let Some(n) = v.as_f64() { st.comp_2_release = n as f32; },
                    "comp_2.ratio" => if let Some(s) = v.as_str() { st.comp_2_ratio = s.to_string(); },
                    "eq_1.enabled" => if let Some(b) = v.as_bool() { st.eq_1_enabled = b; },
                    "mod_2.enabled" => if let Some(b) = v.as_bool() { st.mod_2_enabled = b; },
                    "mod_2.rate_hz" => if let Some(n) = v.as_f64() { st.mod_2_rate_hz = n as f32; },
                    "mod_2.depth" => if let Some(n) = v.as_f64() { st.mod_2_depth = n as f32; },
                    "mod_2.mix" | "mod_2.mix_pct" => if let Some(n) = v.as_f64() { st.mod_2_mix = n as f32; },
                    "dly_1.enabled" => if let Some(b) = v.as_bool() { st.dly_1_enabled = b; },
                    "dly_1.heads" => if let Some(s) = v.as_str() { st.dly_1_heads = s.to_string(); },
                    "dly_1.repeat_rate" => if let Some(n) = v.as_f64() { st.dly_1_repeat_rate = n as f32; },
                    "dly_1.intensity" => if let Some(n) = v.as_f64() { st.dly_1_intensity = n as f32; },
                    "dly_1.echo_volume" => if let Some(n) = v.as_f64() { st.dly_1_echo_volume = n as f32; },
                    "dly_1.bass" => if let Some(n) = v.as_f64() { st.dly_1_bass = n as f32; },
                    "dly_1.treble" => if let Some(n) = v.as_f64() { st.dly_1_treble = n as f32; },
                    "dly_1.spillover" => if let Some(b) = v.as_bool() { st.dly_1_spillover = b; },
                    "rev_1.enabled" => if let Some(b) = v.as_bool() { st.rev_1_enabled = b; },
                    "rev_1.decay_s" => if let Some(n) = v.as_f64() { st.rev_1_decay_s = n as f32; },
                    "rev_1.predelay_ms" => if let Some(n) = v.as_f64() { st.rev_1_predelay_ms = n as f32; },
                    "rev_1.damping" => if let Some(n) = v.as_f64() { st.rev_1_damping = n as f32; },
                    "rev_1.mix" | "rev_1.mix_pct" => if let Some(n) = v.as_f64() { st.rev_1_mix = n as f32; },
                    "rev_1.spillover" => if let Some(b) = v.as_bool() { st.rev_1_spillover = b; },
                    "amp_2.enabled" => if let Some(b) = v.as_bool() { st.amp_2_enabled = b; },
                    "amp_2.model" => if let Some(s) = v.as_str() { st.amp_2_model = s.to_string(); },
                    "cab_2.enabled" => if let Some(b) = v.as_bool() { st.cab_2_enabled = b; },
                    "cab_2.speaker" | "cab_2.model" => if let Some(s) = v.as_str() { st.cab_2_speaker = s.to_string(); },
                    "scene.active" => if let Some(s) = v.as_str() { st.scene_active = s.to_string(); },
                    "tuner.enabled" => if let Some(b) = v.as_bool() { st.tuner_enabled = b; },
                    "tuner.output" => if let Some(s) = v.as_str() { st.tuner_output = s.to_string(); },
                    _ => {}
                }
            }
            st.is_modded = st.check_is_modded();
        }

        if let Some(ref dsp_ref) = dsp_chain {
            if let Ok(mut dsp) = dsp_ref.lock() {
                for (k, v) in params {
                    match k.as_str() {
                        "amp_1.gain" => if let Some(n) = v.as_f64() { dsp.set_amp_gain(n as f32); },
                        "amp_1.overdrive" => if let Some(n) = v.as_f64() { dsp.set_amp_overdrive(n as f32); },
                        "amp_1.bass" => if let Some(n) = v.as_f64() { dsp.set_amp_bass(n as f32); },
                        "amp_1.middle" => if let Some(n) = v.as_f64() { dsp.set_amp_middle(n as f32); },
                        "amp_1.treble" => if let Some(n) = v.as_f64() { dsp.set_amp_treble(n as f32); },
                        "amp_1.presence" => if let Some(n) = v.as_f64() { dsp.set_amp_presence(n as f32); },
                        "amp_1.depth" => if let Some(n) = v.as_f64() { dsp.set_amp_depth(n as f32); },
                        "amp_1.master" => if let Some(n) = v.as_f64() { dsp.set_amp_master(n as f32); },
                        "amp_1.volume_2" => if let Some(n) = v.as_f64() { dsp.set_amp_volume_2(n as f32); },
                        "amp_1.cut" => if let Some(n) = v.as_f64() { dsp.set_amp_cut(n as f32); },
                        "amp_1.model" => if let Some(s) = v.as_str() { dsp.set_amp_model(s); },
                        "amp_1.bright" => if let Some(b) = v.as_bool() { dsp.set_amp_bright(b); },
                        "amp_1.enabled" => if let Some(b) = v.as_bool() { dsp.set_amp_enabled(b); },
                        "amp_1.bias_ma" => if let Some(n) = v.as_f64() { dsp.set_amp_bias_ma(n as f32); },
                        "amp_1.sag" => if let Some(n) = v.as_f64() { dsp.set_amp_sag(n as f32); },
                        "amp_1.nfb" => if let Some(n) = v.as_f64() { dsp.set_amp_nfb(n as f32); },
                        "amp_1.xformer_sat" => if let Some(n) = v.as_f64() { dsp.set_amp_xformer_sat(n as f32); },
                        "amp_1.preamp_plate_v" => if let Some(n) = v.as_f64() { dsp.set_amp_preamp_plate_v(n as f32); },
                        "amp_1.triode_asym" => if let Some(n) = v.as_f64() { dsp.set_amp_triode_asym(n as f32); },
                        "amp_1.preamp_tube" => if let Some(s) = v.as_str() { dsp.set_amp_preamp_tube(s); },
                        "amp_1.power_tube" => if let Some(s) = v.as_str() { dsp.set_amp_power_tube(s); },
                        "amp_1.rectifier" => if let Some(s) = v.as_str() { dsp.set_amp_rectifier(s); },
                        "amp_1.tonestack" => if let Some(s) = v.as_str() { dsp.set_amp_tonestack(s); },
                        "engine.oversampling" => if let Some(s) = v.as_str() { dsp.set_oversampling_factor(s); },
                        "input.boost" => if let Some(n) = v.as_f64() { dsp.set_input_boost(n as f32); },
                        "input.source" => if let Some(s) = v.as_str() { dsp.set_input_source(s); },
                        "drive.enabled" => if let Some(b) = v.as_bool() { dsp.set_drive_enabled(b); },
                        "drive.drive" => if let Some(n) = v.as_f64() { dsp.set_drive_gain(n as f32); },
                        "drive.tone" => if let Some(n) = v.as_f64() { dsp.set_drive_tone(n as f32); },
                        "drive.level" => if let Some(n) = v.as_f64() { dsp.set_drive_level(n as f32); },
                        "gate.enabled" => if let Some(b) = v.as_bool() { dsp.set_gate_enabled(b); },
                        "gate.threshold_db" => if let Some(n) = v.as_f64() { dsp.set_gate_threshold_db(n as f32); },
                        "cab_1.enabled" => if let Some(b) = v.as_bool() { dsp.set_cab_enabled(b); },
                        "cab_1.speaker" | "cab_1.model" => if let Some(s) = v.as_str() { dsp.set_cab_speaker(s); },
                        "cab_1.resonance_hz" => if let Some(n) = v.as_f64() { dsp.set_cab_resonance_hz(n as f32); },
                        "cab_1.hicut_khz" => if let Some(n) = v.as_f64() { dsp.set_cab_hicut_khz(n as f32); },
                        "cab_1.ir_enabled" => if let Some(b) = v.as_bool() { dsp.set_cab_ir_enabled(b); },
                        "cab_1.mic1.model" => if let Some(s) = v.as_str() { dsp.set_cab_mic1_model(s); },
                        "cab_1.mic1.position" => if let Some(n) = v.as_f64() { dsp.set_cab_mic1_position(n as f32); },
                        "cab_1.mic1.distance_cm" => if let Some(n) = v.as_f64() { dsp.set_cab_mic1_distance_cm(n as f32); },
                        "cab_1.mic1.level_db" => if let Some(n) = v.as_f64() { dsp.set_cab_mic1_level_db(n as f32); },
                        "cab_1.mic1.enabled" => if let Some(b) = v.as_bool() { dsp.set_cab_mic1_enabled(b); },
                        "cab_1.mic2.model" => if let Some(s) = v.as_str() { dsp.set_cab_mic2_model(s); },
                        "cab_1.mic2.position" => if let Some(n) = v.as_f64() { dsp.set_cab_mic2_position(n as f32); },
                        "cab_1.mic2.distance_cm" => if let Some(n) = v.as_f64() { dsp.set_cab_mic2_distance_cm(n as f32); },
                        "cab_1.mic2.level_db" => if let Some(n) = v.as_f64() { dsp.set_cab_mic2_level_db(n as f32); },
                        "cab_1.mic2.enabled" => if let Some(b) = v.as_bool() { dsp.set_cab_mic2_enabled(b); },
                        "split.enabled" => if let Some(b) = v.as_bool() { dsp.set_split_enabled(b); },
                        "split.mode" => {
                            if let Some(s) = v.as_str() {
                                let hz = params.get("split.crossover_hz").and_then(|x| x.as_f64()).map(|x| x as f32).unwrap_or(650.0);
                                dsp.set_split_mode(s, hz);
                            }
                        }
                        "split.crossover_hz" => {
                            if let Some(n) = v.as_f64() {
                                let mode = params.get("split.mode").and_then(|x| x.as_str()).unwrap_or("crossover");
                                dsp.set_split_mode(mode, n as f32);
                            }
                        }
                        "merge.row_1_level_db" => {
                            if let Some(n) = v.as_f64() {
                                let inv = params.get("merge.row_1_phase_invert").and_then(|x| x.as_bool()).unwrap_or(false);
                                dsp.set_merge_config(n as f32, inv);
                            }
                        }
                        "merge.row_1_phase_invert" => {
                            if let Some(b) = v.as_bool() {
                                let db = params.get("merge.row_1_level_db").and_then(|x| x.as_f64()).map(|x| x as f32).unwrap_or(-6.0);
                                dsp.set_merge_config(db, b);
                            }
                        }
                        "comp_2.enabled" => if let Some(b) = v.as_bool() { dsp.set_comp_2_enabled(b); },
                        "comp_2.input" => if let Some(n) = v.as_f64() { dsp.set_comp_2_input(n as f32); },
                        "comp_2.output" => if let Some(n) = v.as_f64() { dsp.set_comp_2_output(n as f32); },
                        "comp_2.attack" => if let Some(n) = v.as_f64() { dsp.set_comp_2_attack(n as f32); },
                        "comp_2.release" => if let Some(n) = v.as_f64() { dsp.set_comp_2_release(n as f32); },
                        "comp_2.ratio" => if let Some(s) = v.as_str() { dsp.set_comp_2_ratio(s); },
                        "eq_1.enabled" => if let Some(b) = v.as_bool() { dsp.set_eq_1_enabled(b); },
                        "mod_2.enabled" => if let Some(b) = v.as_bool() { dsp.set_mod_2_enabled(b); },
                        "mod_2.rate_hz" => if let Some(n) = v.as_f64() { dsp.set_mod_2_rate_hz(n as f32); },
                        "mod_2.depth" => if let Some(n) = v.as_f64() { dsp.set_mod_2_depth(n as f32); },
                        "mod_2.mix" | "mod_2.mix_pct" => if let Some(n) = v.as_f64() { dsp.set_mod_2_mix(n as f32); },
                        "dly_1.enabled" => if let Some(b) = v.as_bool() { dsp.set_dly_1_enabled(b); },
                        "dly_1.repeat_rate" => if let Some(n) = v.as_f64() { dsp.set_dly_1_repeat_rate(n as f32); },
                        "dly_1.intensity" => if let Some(n) = v.as_f64() { dsp.set_dly_1_intensity(n as f32); },
                        "dly_1.echo_volume" => if let Some(n) = v.as_f64() { dsp.set_dly_1_echo_volume(n as f32); },
                        "dly_1.spillover" => if let Some(b) = v.as_bool() { dsp.set_dly_1_spillover(b); },
                        "rev_1.enabled" => if let Some(b) = v.as_bool() { dsp.set_rev_1_enabled(b); },
                        "rev_1.decay_s" => if let Some(n) = v.as_f64() { dsp.set_rev_1_decay_s(n as f32); },
                        "rev_1.predelay_ms" => if let Some(n) = v.as_f64() { dsp.set_rev_1_predelay_ms(n as f32); },
                        "rev_1.damping" => if let Some(n) = v.as_f64() { dsp.set_rev_1_damping(n as f32); },
                        "rev_1.mix" | "rev_1.mix_pct" => if let Some(n) = v.as_f64() { dsp.set_rev_1_mix(n as f32); },
                        "rev_1.spillover" => if let Some(b) = v.as_bool() { dsp.set_rev_1_spillover(b); },
                        "amp_2.enabled" => if let Some(b) = v.as_bool() { dsp.set_amp_2_enabled(b); },
                        "amp_2.model" => if let Some(s) = v.as_str() { dsp.set_amp_2_model(s); },
                        "cab_2.enabled" => if let Some(b) = v.as_bool() { dsp.set_cab_2_enabled(b); },
                        "cab_2.speaker" | "cab_2.model" => if let Some(s) = v.as_str() { dsp.set_cab_2_speaker(s); },
                        "tuner.enabled" => if let Some(b) = v.as_bool() { dsp.set_tuner_enabled(b); },
                        "tuner.output" => if let Some(s) = v.as_str() {
                            let m = if s == "bypass" {
                                ampforge_dsp::tuner::TunerOutputMode::Bypass
                            } else {
                                ampforge_dsp::tuner::TunerOutputMode::Mute
                            };
                            dsp.set_tuner_output_mode(m);
                        },
                        _ => {}
                    }
                }
                dsp.trigger_preset_crossfade();
            }
        }
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
        self.running.store(false, Ordering::Relaxed);
        for th in self.server_threads.drain(..) {
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
