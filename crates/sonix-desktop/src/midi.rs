//! crates/sonix-desktop/src/midi.rs
//! Host MIDI Nativo Desktop (Portas USB / MIDI Din e Roteamento Spec 014)

use crate::api::{dispatch, ApiContext};
use ampforge_dsp::queue::{DspCommand, SpscCommandQueue};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiMessageType {
    ControlChange { controller: u8, value: u8 },
    ProgramChange { program: u8 },
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

#[cfg(target_os = "windows")]
mod winmm_midi {
    #[repr(C)]
    struct MidiInCapsW {
        w_mid: u16,
        w_pid: u16,
        v_driver_version: u32,
        sz_pname: [u16; 32],
        dw_support: u32,
    }

    #[link(name = "winmm")]
    extern "system" {
        fn midiInGetNumDevs() -> u32;
        fn midiInGetDevCapsW(u_device_id: usize, pmic: *mut MidiInCapsW, cbmic: u32) -> u32;
    }

    pub fn list_ports() -> Vec<String> {
        let count = unsafe { midiInGetNumDevs() };
        let mut ports = Vec::new();
        for idx in 0..count {
            let mut caps = MidiInCapsW {
                w_mid: 0,
                w_pid: 0,
                v_driver_version: 0,
                sz_pname: [0u16; 32],
                dw_support: 0,
            };
            let res = unsafe {
                midiInGetDevCapsW(
                    idx as usize,
                    &mut caps as *mut MidiInCapsW,
                    std::mem::size_of::<MidiInCapsW>() as u32,
                )
            };
            if res == 0 {
                let end = caps
                    .sz_pname
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(caps.sz_pname.len());
                let name = String::from_utf16_lossy(&caps.sz_pname[..end])
                    .trim()
                    .to_string();
                if !name.is_empty() {
                    ports.push(name);
                }
            }
        }
        ports
    }
}

impl MidiPortEnumerator for OsMidiEnumerator {
    fn enumerate_input_ports(&self) -> Vec<String> {
        #[cfg(target_os = "windows")]
        {
            return winmm_midi::list_ports();
        }
        #[cfg(not(target_os = "windows"))]
        {
            Vec::new()
        }
    }
}

pub struct DesktopMidiHost {
    connected_port: Option<String>,
    enumerator: Arc<dyn MidiPortEnumerator>,
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
        }
    }

    pub fn with_enumerator(enumerator: Arc<dyn MidiPortEnumerator>) -> Self {
        Self {
            connected_port: None,
            enumerator,
        }
    }

    pub fn enumerate_input_ports(&self) -> Vec<String> {
        self.enumerator.enumerate_input_ports()
    }

    pub fn connect_port(&mut self, port_name: &str) -> Result<(), String> {
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

    pub fn process_midi_cc_to_params(
        &self,
        bytes: &[u8],
        ctx: &ApiContext,
        command_queue: &mut SpscCommandQueue,
    ) -> Result<MidiMessageType, String> {
        let msg = self.process_raw_midi_message(bytes, command_queue)?;
        if let MidiMessageType::ControlChange { controller, value } = msg {
            if let Some((key, val)) = Self::map_cc_to_param(controller, value) {
                let patch = serde_json::json!({ key: val });
                let body = serde_json::to_vec(&patch).map_err(|e| e.to_string())?;
                let (code, _, resp) = dispatch(ctx, "/api/params", "POST", &body);
                if code >= 400 {
                    return Err(format!(
                        "Falha ao rotear MIDI CC {} via Spec 014 ({}): {}",
                        controller,
                        code,
                        String::from_utf8_lossy(&resp)
                    ));
                }
            }
        }
        Ok(msg)
    }

    pub fn process_raw_midi_message(
        &self,
        bytes: &[u8],
        command_queue: &mut SpscCommandQueue,
    ) -> Result<MidiMessageType, String> {
        if bytes.is_empty() {
            return Err("Mensagem MIDI vazia".to_string());
        }

        let status = bytes[0] & 0xF0;
        match status {
            0xB0 if bytes.len() >= 3 => {
                let controller = bytes[1];
                let value = bytes[2];
                // Forward lock-free to DSP command queue
                let _ = command_queue.push(DspCommand::SetParameter {
                    block_id: 0,
                    param_id: controller as usize,
                    value: value as f32 / 127.0,
                });
                Ok(MidiMessageType::ControlChange { controller, value })
            }
            0xC0 if bytes.len() >= 2 => {
                let program = bytes[1];
                let _ = command_queue.push(DspCommand::SelectChannel {
                    block_id: 0,
                    channel_index: program as usize,
                });
                Ok(MidiMessageType::ProgramChange { program })
            }
            _ => Ok(MidiMessageType::Other),
        }
    }
}
