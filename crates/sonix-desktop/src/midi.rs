//! crates/sonix-desktop/src/midi.rs
//! Host MIDI Nativo Desktop (Portas USB / MIDI Din e Roteamento Spec 014)

use crate::api::{dispatch, dispatch_midi_cc, ApiContext};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub const DECISAO_SONIX_CC_CENA: u8 = 69;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiMessageType {
    ControlChange { controller: u8, value: u8 },
    ProgramChange { program: u8 },
    Clock,
    Other,
}

pub trait MidiPortEnumerator: Send + Sync {
    fn enumerate_input_ports(&self) -> Vec<String>;
}

#[derive(Debug, Default)]
pub struct OsMidiEnumerator;

impl OsMidiEnumerator {
    pub fn new() -> Self {
        Self
    }
}

impl MidiPortEnumerator for OsMidiEnumerator {
    fn enumerate_input_ports(&self) -> Vec<String> {
        let Ok(midi_in) = midir::MidiInput::new("SonixProMidiEnum") else {
            return Vec::new();
        };
        midi_in
            .ports()
            .iter()
            .filter_map(|p| midi_in.port_name(p).ok())
            .filter(|name| !name.trim().is_empty())
            .collect()
    }
}

pub struct DesktopMidiHost {
    connected_port: Option<String>,
    enumerator: Arc<dyn MidiPortEnumerator>,
    active_connection: Option<midir::MidiInputConnection<()>>,
}

impl Default for DesktopMidiHost {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopMidiHost {
    pub fn new() -> Self {
        Self {
            connected_port: None,
            enumerator: Arc::new(OsMidiEnumerator::new()),
            active_connection: None,
        }
    }

    pub fn with_enumerator(enumerator: Arc<dyn MidiPortEnumerator>) -> Self {
        Self {
            connected_port: None,
            enumerator,
            active_connection: None,
        }
    }

    pub fn enumerate_input_ports(&self) -> Vec<String> {
        self.enumerator.enumerate_input_ports()
    }

    pub fn connected_port(&self) -> Option<&str> {
        self.connected_port.as_deref()
    }

    pub fn disconnect(&mut self) {
        self.active_connection = None;
        self.connected_port = None;
    }

    pub fn connect_port(&mut self, port_name: &str) -> Result<(), String> {
        self.connect_port_internal(port_name, None)
    }

    pub fn connect_port_with_router(
        &mut self,
        port_name: &str,
        router: Arc<Mutex<MidiRouter>>,
    ) -> Result<(), String> {
        self.connect_port_internal(port_name, Some(router))
    }

    fn connect_port_internal(
        &mut self,
        port_name: &str,
        router: Option<Arc<Mutex<MidiRouter>>>,
    ) -> Result<(), String> {
        self.disconnect();
        let midi_in = midir::MidiInput::new("SonixProMidiIn")
            .map_err(|e| format!("falha ao inicializar MIDI IN: {e}"))?;
        let target_port = midi_in
            .ports()
            .into_iter()
            .find(|p| midi_in.port_name(p).ok().as_deref() == Some(port_name))
            .ok_or_else(|| format!("porta MIDI não encontrada: {port_name}"))?;

        let conn = midi_in
            .connect(
                &target_port,
                "sonix-pro-midi-in",
                move |_timestamp, message, _| {
                    if let Some(ref r) = router {
                        if let Ok(mut guard) = r.lock() {
                            guard.on_message(message);
                        }
                    }
                },
                (),
            )
            .map_err(|e| format!("falha ao conectar porta MIDI {port_name}: {e}"))?;

        self.active_connection = Some(conn);
        self.connected_port = Some(port_name.to_string());
        Ok(())
    }

    /// Mapeamento MIDI padrão migrado para Modifiers editáveis (Spec 049, CA-33)
    pub fn default_midi_modifiers() -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "id": "midi_cc_1_amp_gain",
                "target_param": "amp_1.gain",
                "source": "MIDI_CC_1",
                "min_val": 0.0,
                "max_val": 10.0,
                "curve_slope": 0.5,
                "damping_ms": 0.0
            }),
            serde_json::json!({
                "id": "midi_cc_7_amp_master",
                "target_param": "amp_1.master",
                "source": "MIDI_CC_7",
                "min_val": 0.0,
                "max_val": 10.0,
                "curve_slope": 0.5,
                "damping_ms": 0.0
            }),
            serde_json::json!({
                "id": "midi_cc_11_drive",
                "target_param": "drive.drive",
                "source": "MIDI_CC_11",
                "min_val": 0.0,
                "max_val": 10.0,
                "curve_slope": 0.5,
                "damping_ms": 0.0
            }),
            serde_json::json!({
                "id": "midi_cc_14_gate_thresh",
                "target_param": "gate.threshold_db",
                "source": "MIDI_CC_14",
                "min_val": -90.0,
                "max_val": 0.0,
                "curve_slope": 0.5,
                "damping_ms": 0.0
            }),
        ]
    }

    pub fn map_cc_to_param(controller: u8, value: u8) -> Option<(&'static str, serde_json::Value)> {
        let norm = (value.min(127) as f32) / 127.0;
        match controller {
            1 => Some(("amp_1.gain", serde_json::json!(norm * 10.0))),
            7 => Some(("amp_1.master", serde_json::json!(norm * 10.0))),
            11 => Some(("drive.drive", serde_json::json!(norm * 10.0))),
            14 => Some(("gate.threshold_db", serde_json::json!(-90.0 + norm * 90.0))),
            80 => Some(("amp_1.enabled", serde_json::json!(value >= 64))),
            81 => Some(("drive.enabled", serde_json::json!(value >= 64))),
            _ => None,
        }
    }
}

pub struct MidiRouter {
    ctx: ApiContext,
    start_instant: Instant,
    clock_prev_us: Option<u64>,
    clock_intervals_us: [u64; 24],
    clock_fill: usize,
    clock_idx: usize,
    last_clock_send_us: Option<u64>,
    last_sent_bpm: Option<f32>,
}

impl MidiRouter {
    pub fn new(ctx: ApiContext) -> Self {
        Self {
            ctx,
            start_instant: Instant::now(),
            clock_prev_us: None,
            clock_intervals_us: [0; 24],
            clock_fill: 0,
            clock_idx: 0,
            last_clock_send_us: None,
            last_sent_bpm: None,
        }
    }

    pub fn on_message(&mut self, bytes: &[u8]) -> MidiMessageType {
        let now_us = self.start_instant.elapsed().as_micros() as u64;
        self.on_message_at(bytes, now_us)
    }

    pub fn on_message_at(&mut self, bytes: &[u8], now_us: u64) -> MidiMessageType {
        if bytes.is_empty() {
            return MidiMessageType::Other;
        }
        match bytes[0] {
            0xF8 => {
                if let Some(prev) = self.clock_prev_us {
                    let dt = now_us.saturating_sub(prev);
                    if dt > 100 && dt < 2_000_000 {
                        self.clock_intervals_us[self.clock_idx] = dt;
                        self.clock_idx = (self.clock_idx + 1) % 24;
                        if self.clock_fill < 24 {
                            self.clock_fill += 1;
                        }
                        if self.clock_fill == 24 {
                            let sum_us: u64 = self.clock_intervals_us.iter().sum();
                            let avg_dt_us = (sum_us as f64) / 24.0;
                            let bpm = (60_000_000.0 / (avg_dt_us * 24.0)).clamp(20.0, 300.0) as f32;
                            let rate_ok = self
                                .last_clock_send_us
                                .map(|last| now_us.saturating_sub(last) >= 100_000)
                                .unwrap_or(true);
                            let delta_ok = self
                                .last_sent_bpm
                                .map(|prev_bpm| (bpm - prev_bpm).abs() >= 0.5)
                                .unwrap_or(true);
                            if rate_ok && delta_ok {
                                let body = serde_json::json!({ "dly_1.tempo_bpm": bpm });
                                if let Ok(bytes) = serde_json::to_vec(&body) {
                                    let (code, _, _) = dispatch(&self.ctx, "/api/params", "POST", &bytes);
                                    if code < 400 {
                                        self.last_clock_send_us = Some(now_us);
                                        self.last_sent_bpm = Some(bpm);
                                    }
                                }
                            }
                        }
                    }
                }
                self.clock_prev_us = Some(now_us);
                MidiMessageType::Clock
            }
            0xFA | 0xFC => MidiMessageType::Other,
            _ => {
                let status = bytes[0] & 0xF0;
                match status {
                    0xB0 if bytes.len() >= 3 => {
                        let controller = bytes[1];
                        let value = bytes[2];
                        if controller == DECISAO_SONIX_CC_CENA && value <= 7 {
                            if let Ok(req) = serde_json::to_vec(&serde_json::json!({ "index": value })) {
                                let _ = dispatch(&self.ctx, "/api/scenes/select", "POST", &req);
                            }
                        }
                        let _ = dispatch_midi_cc(&self.ctx, controller, value);
                        MidiMessageType::ControlChange { controller, value }
                    }
                    0xC0 if bytes.len() >= 2 => {
                        let program = bytes[1];
                        let (_, _, sl_bytes) = dispatch(&self.ctx, "/api/setlist", "GET", b"");
                        if let Ok(sl_json) = serde_json::from_slice::<serde_json::Value>(&sl_bytes) {
                            if let Some(pid) = sl_json["active_setlist"]["presets"]
                                .as_array()
                                .and_then(|arr| arr.get(program as usize))
                                .and_then(|v| v.as_str())
                            {
                                if let Ok(req) = serde_json::to_vec(&serde_json::json!({ "id": pid })) {
                                    let _ = dispatch(&self.ctx, "/api/presets/load", "POST", &req);
                                }
                            }
                        }
                        MidiMessageType::ProgramChange { program }
                    }
                    _ => MidiMessageType::Other,
                }
            }
        }
    }
}
