//! SONIX PRO // AMPFORGE™ - VST3 & CLAP DAW Plugin Core
//! Compartilha o mesmo núcleo DSP (`RealtimeDspChain`) e tabela de parâmetros do standalone.

use std::num::NonZeroU32;
use std::sync::Arc;

use nih_plug::prelude::*;
use serde::{Deserialize, Serialize};
use sonix_core::persistence::{
    BlockDto, GridDto, PresetMetadata, SceneDto, SonixPresetDoc,
};
use sonix_desktop::audio::{CompiledModifierSource, RealtimeDspChain};
use sonix_desktop::midi::DesktopMidiHost;

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
struct PluginStateExtras {
    preset_name: String,
    tempo_bpm: f32,
    host_tempo_sync: bool,
    dly_1_tempo_sync: bool,
    amp_1_gain: f32,
    dly_1_enabled: bool,
}

pub struct SonixPluginProcessor {
    format: PluginFormat,
    descriptor: PluginDescriptor,
    chain: RealtimeDspChain,
    sample_rate: f32,
    preset_name: String,
    tempo_bpm: f32,
    host_tempo_sync: bool,
    dly_1_tempo_sync: bool,
    amp_1_gain: f32,
    dly_1_enabled: bool,
    doc: SonixPresetDoc,
}

impl SonixPluginProcessor {
    pub fn new(format: PluginFormat) -> Self {
        let mut scenes = Vec::with_capacity(8);
        for idx in 0..8usize {
            scenes.push(SceneDto {
                index: idx,
                id: format!("scene-{}", idx + 1),
                name: format!("Cena {}", idx + 1),
                color: "#d49b35".to_string(),
            });
        }
        let doc = SonixPresetDoc {
            schema: "https://sonixpro.local/schemas/sonixpreset-v1.json".to_string(),
            schema_version: "1.0.0".to_string(),
            metadata: PresetMetadata {
                id: "sonix-plugin-default".to_string(),
                name: "Default Stadium Rig".to_string(),
                author: "SonixPro Audio".to_string(),
                category: "Amp Rig".to_string(),
                tempo_bpm: 120.0,
                created_at: "2026-11-15T00:00:00Z".to_string(),
                footswitch_mode: "SCENE".to_string(),
            },
            active_scene_index: 0,
            scenes,
            grid: GridDto {
                rows: 4,
                cols: 12,
                blocks: Vec::<BlockDto>::new(),
            },
            modifiers: Vec::new(),
            params: Some(SonixPresetDoc::default_boot_params()),
            channels: None,
        };
        let mut chain = RealtimeDspChain::new(48000.0);
        chain.prepare(48000.0, 256);
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
            chain,
            sample_rate: 48000.0,
            preset_name: "Default Stadium Rig".to_string(),
            tempo_bpm: 120.0,
            host_tempo_sync: false,
            dly_1_tempo_sync: false,
            amp_1_gain: 5.0,
            dly_1_enabled: false,
            doc,
        }
    }

    pub fn prepare(&mut self, sample_rate: f32, block_size: usize) {
        self.sample_rate = sample_rate.max(8000.0);
        self.chain.prepare(self.sample_rate, block_size);
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
        let name = name.into();
        self.preset_name = name.clone();
        self.doc.metadata.name = name;
    }

    pub fn tempo_bpm(&self) -> f32 {
        self.tempo_bpm
    }

    pub fn set_tempo_bpm(&mut self, bpm: f32) {
        let clamped = bpm.clamp(20.0, 300.0);
        self.tempo_bpm = clamped;
        self.doc.metadata.tempo_bpm = clamped;
        self.chain.set_dly_1_tempo_bpm(clamped);
    }

    pub fn set_host_tempo_sync(&mut self, sync: bool) {
        self.host_tempo_sync = sync;
    }

    pub fn on_transport_update(&mut self, bpm: f32, _is_playing: bool) {
        if self.host_tempo_sync && bpm.is_finite() && bpm > 0.0 {
            self.set_tempo_bpm(bpm);
        }
    }

    pub fn delay_target_ms(&self) -> f32 {
        60_000.0 / self.tempo_bpm.max(20.0)
    }

    pub fn set_param(&mut self, key: &str, value: f32) -> Result<(), String> {
        match key {
            "amp_1.gain" => {
                let v = value.clamp(0.0, 10.0);
                self.amp_1_gain = v;
                self.chain.set_amp_gain(v);
                Ok(())
            }
            "amp_1.bass" => {
                self.chain.set_amp_bass(value.clamp(0.0, 10.0));
                Ok(())
            }
            "amp_1.middle" => {
                self.chain.set_amp_middle(value.clamp(0.0, 10.0));
                Ok(())
            }
            "amp_1.treble" => {
                self.chain.set_amp_treble(value.clamp(0.0, 10.0));
                Ok(())
            }
            "amp_1.master" => {
                self.chain.set_amp_master(value.clamp(0.0, 10.0));
                Ok(())
            }
            "amp_1.presence" => {
                self.chain.set_amp_presence(value.clamp(0.0, 10.0));
                Ok(())
            }
            "amp_1.depth" => {
                self.chain.set_amp_depth(value.clamp(0.0, 10.0));
                Ok(())
            }
            "gate.enabled" => {
                self.chain.set_gate_enabled(value >= 0.5);
                Ok(())
            }
            "gate.threshold_db" => {
                self.chain.set_gate_threshold_db(value.clamp(-90.0, 0.0));
                Ok(())
            }
            "dly_1.enabled" => {
                let en = value >= 0.5;
                self.dly_1_enabled = en;
                self.chain.set_dly_1_enabled(en);
                Ok(())
            }
            "dly_1.tempo_sync" => {
                self.dly_1_tempo_sync = value >= 0.5;
                Ok(())
            }
            "dly_1.intensity" => {
                self.chain.set_dly_1_intensity(value.clamp(0.0, 10.0));
                Ok(())
            }
            "dly_1.echo_volume" => {
                self.chain.set_dly_1_echo_volume(value.clamp(0.0, 10.0));
                Ok(())
            }
            "dly_1.spillover" => {
                self.chain.set_dly_1_spillover(value >= 0.5);
                Ok(())
            }
            "cab_1.enabled" => {
                self.chain.set_cab_enabled(value >= 0.5);
                Ok(())
            }
            _ => Err(format!("Parâmetro desconhecido no plugin: {}", key)),
        }
    }

    pub fn get_param(&self, key: &str) -> Option<f32> {
        match key {
            "amp_1.gain" => Some(self.amp_1_gain),
            "dly_1.enabled" => Some(if self.dly_1_enabled { 1.0 } else { 0.0 }),
            "dly_1.tempo_sync" => Some(if self.dly_1_tempo_sync { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Setter zero-alloc para automação em tempo real dentro do callback de áudio (CA-21).
    pub fn set_param_rt_normalized(&mut self, key: &str, norm: f32) {
        let n = norm.clamp(0.0, 1.0);
        if key == "amp_1.gain" {
            let v = n * 10.0;
            self.amp_1_gain = v;
            self.chain.set_amp_gain(v);
        }
    }

    pub fn save_state(&self) -> Result<Vec<u8>, String> {
        let mut doc = self.doc.clone();
        doc.metadata.name = self.preset_name.clone();
        doc.metadata.tempo_bpm = self.tempo_bpm;
        let mut val = serde_json::to_value(&doc).map_err(|e| e.to_string())?;
        if let Some(obj) = val.as_object_mut() {
            obj.insert(
                "__plugin_params".to_string(),
                serde_json::json!({
                    "amp_1.gain": self.amp_1_gain,
                    "dly_1.enabled": self.dly_1_enabled,
                    "dly_1.tempo_sync": self.dly_1_tempo_sync,
                    "host_tempo_sync": self.host_tempo_sync,
                }),
            );
        }
        serde_json::to_vec(&val).map_err(|e| e.to_string())
    }

    pub fn restore_state(&mut self, state: &[u8]) -> Result<(), String> {
        if let Ok(doc) = serde_json::from_slice::<SonixPresetDoc>(state) {
            self.preset_name = doc.metadata.name.clone();
            self.set_tempo_bpm(doc.metadata.tempo_bpm);
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(state) {
                if let Some(params) = val.get("__plugin_params") {
                    if let Some(g) = params.get("amp_1.gain").and_then(|v| v.as_f64()) {
                        let _ = self.set_param("amp_1.gain", g as f32);
                    }
                    if let Some(d) = params.get("dly_1.enabled").and_then(|v| v.as_bool()) {
                        let _ = self.set_param("dly_1.enabled", if d { 1.0 } else { 0.0 });
                    }
                    if let Some(ts) = params.get("dly_1.tempo_sync").and_then(|v| v.as_bool()) {
                        self.dly_1_tempo_sync = ts;
                    }
                    if let Some(hs) = params.get("host_tempo_sync").and_then(|v| v.as_bool()) {
                        self.host_tempo_sync = hs;
                    }
                }
            }
            self.doc = doc;
            return Ok(());
        }
        let chunk: PluginStateExtras = serde_json::from_slice(state).map_err(|e| e.to_string())?;
        self.preset_name = chunk.preset_name;
        self.set_tempo_bpm(chunk.tempo_bpm);
        self.host_tempo_sync = chunk.host_tempo_sync;
        Ok(())
    }

    pub fn load_sonixpreset_doc(&mut self, doc: &SonixPresetDoc) -> Result<(), String> {
        self.doc = doc.clone();
        self.preset_name = doc.metadata.name.clone();
        self.set_tempo_bpm(doc.metadata.tempo_bpm);
        Ok(())
    }

    pub fn process(&mut self, in_l: &[f32], in_r: &[f32], out_l: &mut [f32], out_r: &mut [f32]) {
        assert_eq!(in_l.len(), out_l.len());
        assert_eq!(in_r.len(), out_r.len());
        self.chain.process(in_l, out_l);
        out_r.copy_from_slice(out_l);
    }
}

/// Parâmetros automatizáveis do plugin (nih-plug) alinhados com a tabela única da Spec 014.
#[derive(Params)]
pub struct SonixPluginParams {
    #[id = "amp_1.gain"]
    pub amp_gain: FloatParam,
    #[id = "amp_1.bass"]
    pub amp_bass: FloatParam,
    #[id = "amp_1.middle"]
    pub amp_middle: FloatParam,
    #[id = "amp_1.treble"]
    pub amp_treble: FloatParam,
    #[id = "amp_1.master"]
    pub amp_master: FloatParam,
    #[id = "amp_1.presence"]
    pub amp_presence: FloatParam,
    #[id = "amp_1.depth"]
    pub amp_depth: FloatParam,
    #[id = "gate.threshold_db"]
    pub gate_threshold_db: FloatParam,
    #[id = "dly_1.intensity"]
    pub dly_intensity: FloatParam,
    #[id = "rev_1.mix"]
    pub rev_mix: FloatParam,
    #[id = "cab_1.enabled"]
    pub cab_enabled: BoolParam,
    #[id = "host_tempo_sync"]
    pub host_tempo_sync: BoolParam,
}

impl Default for SonixPluginParams {
    fn default() -> Self {
        Self {
            amp_gain: FloatParam::new("Amp Gain", 5.0, FloatRange::Linear { min: 0.0, max: 10.0 }),
            amp_bass: FloatParam::new("Amp Bass", 5.0, FloatRange::Linear { min: 0.0, max: 10.0 }),
            amp_middle: FloatParam::new("Amp Middle", 5.0, FloatRange::Linear { min: 0.0, max: 10.0 }),
            amp_treble: FloatParam::new("Amp Treble", 5.0, FloatRange::Linear { min: 0.0, max: 10.0 }),
            amp_master: FloatParam::new("Amp Master", 5.0, FloatRange::Linear { min: 0.0, max: 10.0 }),
            amp_presence: FloatParam::new("Amp Presence", 5.0, FloatRange::Linear { min: 0.0, max: 10.0 }),
            amp_depth: FloatParam::new("Amp Depth", 5.0, FloatRange::Linear { min: 0.0, max: 10.0 }),
            gate_threshold_db: FloatParam::new(
                "Gate Threshold",
                -50.0,
                FloatRange::Linear { min: -90.0, max: 0.0 },
            )
            .with_unit(" dB"),
            dly_intensity: FloatParam::new("Delay Intensity", 3.5, FloatRange::Linear { min: 0.0, max: 10.0 }),
            rev_mix: FloatParam::new("Reverb Mix", 0.25, FloatRange::Linear { min: 0.0, max: 1.0 }),
            cab_enabled: BoolParam::new("Cab Enabled", true),
            host_tempo_sync: BoolParam::new("Host Tempo Sync", false),
        }
    }
}

pub struct SonixProPlugin {
    params: Arc<SonixPluginParams>,
    chain: RealtimeDspChain,
    in_scratch: Vec<f32>,
    out_scratch: Vec<f32>,
}

impl Default for SonixProPlugin {
    fn default() -> Self {
        let mut chain = RealtimeDspChain::new(48000.0);
        chain.prepare(48000.0, 512);
        Self {
            params: Arc::new(SonixPluginParams::default()),
            chain,
            in_scratch: vec![0.0; 4096],
            out_scratch: vec![0.0; 4096],
        }
    }
}

impl SonixProPlugin {
    pub fn supports_vst3_and_clap() -> bool {
        true
    }

    pub fn audio_channels() -> (usize, usize) {
        (2, 2)
    }

    pub fn accepts_midi_input() -> bool {
        true
    }

    pub fn has_http_server() -> bool {
        false
    }

    pub fn uses_embedded_webview_ipc() -> bool {
        true
    }

    pub fn registered_param_ids() -> Vec<String> {
        [
            "amp_1.gain",
            "amp_1.bass",
            "amp_1.middle",
            "amp_1.treble",
            "amp_1.master",
            "amp_1.presence",
            "amp_1.depth",
            "amp_1.model",
            "amp_1.bright",
            "amp_1.channel",
            "gate.enabled",
            "gate.threshold_db",
            "dly_1.enabled",
            "dly_1.intensity",
            "dly_1.echo_volume",
            "dly_1.tempo_sync",
            "dly_1.spillover",
            "rev_1.enabled",
            "rev_1.decay_s",
            "rev_1.mix",
            "cab_1.enabled",
            "cab_1.ir_enabled",
            "mod_2.enabled",
            "pitch_1.enabled",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .collect()
    }
}

impl Plugin for SonixProPlugin {
    const NAME: &'static str = "SONIX PRO // AMPFORGE™";
    const VENDOR: &'static str = "SonixPro Audio";
    const URL: &'static str = "https://sonixpro.local";
    const EMAIL: &'static str = "support@sonixpro.local";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let max_buf = (buffer_config.max_buffer_size as usize).max(512);
        if self.in_scratch.len() < max_buf {
            self.in_scratch.resize(max_buf, 0.0);
            self.out_scratch.resize(max_buf, 0.0);
        }
        self.chain.prepare(buffer_config.sample_rate, max_buf);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if self.params.host_tempo_sync.value() {
            if let Some(bpm) = context.transport().tempo {
                self.chain.set_dly_1_tempo_bpm(bpm as f32);
            }
        }
        while let Some(event) = context.next_event() {
            if let NoteEvent::MidiCC { cc, value, .. } = event {
                self.chain
                    .set_modifier_source_value(CompiledModifierSource::MidiCc(cc), value);
                let _ = DesktopMidiHost::default_midi_modifiers();
            }
        }

        self.chain.set_amp_gain(self.params.amp_gain.smoothed.next());
        self.chain.set_amp_bass(self.params.amp_bass.smoothed.next());
        self.chain.set_amp_middle(self.params.amp_middle.smoothed.next());
        self.chain.set_amp_treble(self.params.amp_treble.smoothed.next());
        self.chain.set_amp_master(self.params.amp_master.smoothed.next());
        self.chain.set_amp_presence(self.params.amp_presence.smoothed.next());
        self.chain.set_amp_depth(self.params.amp_depth.smoothed.next());
        self.chain
            .set_gate_threshold_db(self.params.gate_threshold_db.smoothed.next());
        self.chain
            .set_dly_1_intensity(self.params.dly_intensity.smoothed.next());
        self.chain.set_cab_enabled(self.params.cab_enabled.value());

        let samples = buffer.samples().min(self.in_scratch.len());
        let slices = buffer.as_slice();
        if slices.len() >= 2 {
            self.in_scratch[..samples].copy_from_slice(&slices[0][..samples]);
            self.chain
                .process(&self.in_scratch[..samples], &mut self.out_scratch[..samples]);
            slices[0][..samples].copy_from_slice(&self.out_scratch[..samples]);
            slices[1][..samples].copy_from_slice(&self.out_scratch[..samples]);
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for SonixProPlugin {
    const CLAP_ID: &'static str = "com.sonixpro.ampforge";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("SONIX PRO // AMPFORGE™ Guitar Rig");
    const CLAP_MANUAL_URL: Option<&'static str> = Some("https://sonixpro.local");
    const CLAP_SUPPORT_URL: Option<&'static str> = Some("https://sonixpro.local");
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Distortion,
        ClapFeature::Filter,
    ];
}

impl Vst3Plugin for SonixProPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"SonixProAmpForge";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Distortion,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(SonixProPlugin);
nih_export_vst3!(SonixProPlugin);
