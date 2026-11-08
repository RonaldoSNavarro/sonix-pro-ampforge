//! crates/sonix-desktop/src/midi.rs
//! Host MIDI Nativo Desktop (Portas USB / MIDI Din e SPSC Queue Forwarding)

use ampforge_dsp::queue::{DspCommand, SpscCommandQueue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiMessageType {
    ControlChange { controller: u8, value: u8 },
    ProgramChange { program: u8 },
    Other,
}

pub struct DesktopMidiHost {
    connected_port: Option<String>,
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
        }
    }

    pub fn enumerate_input_ports(&self) -> Vec<String> {
        vec![
            "USB Floorboard MIDI In".to_string(),
            "Expression Pedal Interface".to_string(),
        ]
    }

    pub fn connect_port(&mut self, port_name: &str) -> Result<(), String> {
        self.connected_port = Some(port_name.to_string());
        Ok(())
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
