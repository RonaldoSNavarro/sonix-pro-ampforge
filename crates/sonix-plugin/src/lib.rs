//! SONIX PRO // AMPFORGE™ - VST3 & CLAP DAW Plugin Core

use serde::{Deserialize, Serialize};
use sonix_core::persistence::SonixPresetDoc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginFormat {
    Vst3,
    Clap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginDescriptor {
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub is_64bit: bool,
    pub audio_in_channels: usize,
    pub audio_out_channels: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginStateChunk {
    preset_name: String,
    tempo_bpm: f32,
    host_tempo_sync: bool,
}

pub struct SonixPluginProcessor {
    format: PluginFormat,
    descriptor: PluginDescriptor,
    preset_name: String,
    tempo_bpm: f32,
    host_tempo_sync: bool,
}

impl SonixPluginProcessor {
    pub fn new(format: PluginFormat) -> Self {
        Self {
            format,
            descriptor: PluginDescriptor {
                name: "SONIX PRO // AMPFORGE™".to_string(),
                vendor: "SonixPro Audio".to_string(),
                version: "1.0.0".to_string(),
                is_64bit: true,
                audio_in_channels: 2,
                audio_out_channels: 2,
            },
            preset_name: "Default Stadium Rig".to_string(),
            tempo_bpm: 120.0,
            host_tempo_sync: false,
        }
    }

    pub fn format(&self) -> PluginFormat {
        self.format
    }

    pub fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    pub fn preset_name(&self) -> &str {
        &self.preset_name
    }

    pub fn set_preset_name(&mut self, name: impl Into<String>) {
        self.preset_name = name.into();
    }

    pub fn tempo_bpm(&self) -> f32 {
        self.tempo_bpm
    }

    pub fn set_tempo_bpm(&mut self, bpm: f32) {
        self.tempo_bpm = bpm;
    }

    pub fn set_host_tempo_sync(&mut self, sync: bool) {
        self.host_tempo_sync = sync;
    }

    pub fn on_transport_update(&mut self, bpm: f32, _is_playing: bool) {
        if self.host_tempo_sync {
            self.tempo_bpm = bpm;
        }
    }

    pub fn save_state(&self) -> Result<Vec<u8>, String> {
        let chunk = PluginStateChunk {
            preset_name: self.preset_name.clone(),
            tempo_bpm: self.tempo_bpm,
            host_tempo_sync: self.host_tempo_sync,
        };
        serde_json::to_vec(&chunk).map_err(|e| e.to_string())
    }

    pub fn restore_state(&mut self, state: &[u8]) -> Result<(), String> {
        let chunk: PluginStateChunk = serde_json::from_slice(state).map_err(|e| e.to_string())?;
        self.preset_name = chunk.preset_name;
        self.tempo_bpm = chunk.tempo_bpm;
        self.host_tempo_sync = chunk.host_tempo_sync;
        Ok(())
    }

    pub fn load_sonixpreset_doc(&mut self, doc: &SonixPresetDoc) -> Result<(), String> {
        self.preset_name = doc.metadata.name.clone();
        self.tempo_bpm = doc.metadata.tempo_bpm;
        Ok(())
    }

    pub fn process(&mut self, in_l: &[f32], in_r: &[f32], out_l: &mut [f32], out_r: &mut [f32]) {
        assert_eq!(in_l.len(), out_l.len());
        assert_eq!(in_r.len(), out_r.len());

        out_l.copy_from_slice(in_l);
        out_r.copy_from_slice(in_r);
    }
}
