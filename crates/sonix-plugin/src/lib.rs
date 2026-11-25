//! SONIX PRO // AMPFORGE™ - VST3 & CLAP DAW Plugin Core
//! Compartilha o mesmo núcleo DSP (`RealtimeDspChain`) e tabela de parâmetros do standalone.

#![allow(semicolon_in_expressions_from_non_local_macros)]

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
    stereo_input: bool,
    preset_name: String,
    tempo_bpm: f32,
    host_tempo_sync: bool,
    dly_1_tempo_sync: bool,
    amp_1_gain: f32,
    dly_1_enabled: bool,
    doc: SonixPresetDoc,
    in_scratch: Vec<f32>,
    out_scratch: Vec<f32>,
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
            stereo_input: true,
            preset_name: "Default Stadium Rig".to_string(),
            tempo_bpm: 120.0,
            host_tempo_sync: false,
            dly_1_tempo_sync: false,
            amp_1_gain: 5.0,
            dly_1_enabled: false,
            doc,
            in_scratch: vec![0.0; 4096],
            out_scratch: vec![0.0; 4096],
        }
    }

    pub fn prepare(&mut self, sample_rate: f32, block_size: usize) {
        self.sample_rate = sample_rate.max(8000.0);
        let scratch_len = block_size.max(512);
        if self.in_scratch.len() < scratch_len {
            self.in_scratch.resize(scratch_len, 0.0);
            self.out_scratch.resize(scratch_len, 0.0);
        }
        self.chain.prepare(self.sample_rate, block_size);
    }

    pub fn compile_default_modifiers(&mut self) {
        let raw = DesktopMidiHost::default_midi_modifiers();
        let dtos: Vec<sonix_desktop::params::ModifierBindingDto> = raw
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        let table = sonix_desktop::api::compile_modifiers_table(&dtos);
        self.chain.set_compiled_modifiers(table);
    }

    pub fn set_stereo_input(&mut self, stereo: bool) {
        self.stereo_input = stereo;
    }

    pub fn latency_samples(&self) -> u32 {
        self.chain.algorithmic_latency_samples()
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

    #[inline]
    fn apply_plugin_params(
        &mut self,
        params: &PluginParamValues,
        midi_ccs: &[(u8, f32)],
        transport_bpm: Option<f64>,
    ) {
        if params.host_tempo_sync {
            if let Some(bpm) = transport_bpm {
                if bpm.is_finite() && bpm > 0.0 {
                    self.chain.set_dly_1_tempo_bpm(bpm as f32);
                }
            }
        }
        for &(cc, value) in midi_ccs {
            self.chain
                .set_modifier_source_value(CompiledModifierSource::MidiCc(cc), value);
        }

        self.chain.set_amp_gain(params.amp_gain);
        self.chain.set_amp_bass(params.amp_bass);
        self.chain.set_amp_middle(params.amp_middle);
        self.chain.set_amp_treble(params.amp_treble);
        self.chain.set_amp_master(params.amp_master);
        self.chain.set_amp_presence(params.amp_presence);
        self.chain.set_gate_threshold_db(params.gate_threshold_db);
        self.chain.set_dly_1_enabled(params.dly_intensity > 0.001);
        self.chain.set_dly_1_intensity(params.dly_intensity);
        self.chain.set_rev_1_enabled(params.rev_mix > 0.001);
        self.chain.set_rev_1_mix(params.rev_mix * 100.0);
        self.chain.set_cab_enabled(params.cab_enabled);
    }

    pub fn process_block(
        &mut self,
        in_l: &[f32],
        in_r: Option<&[f32]>,
        out_l: &mut [f32],
        out_r: &mut [f32],
        params: &PluginParamValues,
        midi_ccs: &[(u8, f32)],
        transport_bpm: Option<f64>,
    ) {
        let n = in_l
            .len()
            .min(out_l.len())
            .min(out_r.len())
            .min(self.in_scratch.len());
        if n == 0 {
            return;
        }
        let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.apply_plugin_params(params, midi_ccs, transport_bpm);
            if self.stereo_input {
                if let Some(r) = in_r {
                    let rn = r.len().min(n);
                    for i in 0..rn {
                        self.in_scratch[i] = 0.5 * (in_l[i] + r[i]);
                    }
                    if rn < n {
                        self.in_scratch[rn..n].copy_from_slice(&in_l[rn..n]);
                    }
                } else {
                    self.in_scratch[..n].copy_from_slice(&in_l[..n]);
                }
            } else {
                self.in_scratch[..n].copy_from_slice(&in_l[..n]);
            }
            self.chain
                .process(&self.in_scratch[..n], &mut self.out_scratch[..n]);
            out_l[..n].copy_from_slice(&self.out_scratch[..n]);
            out_r[..n].copy_from_slice(&self.out_scratch[..n]);
        }));
        if ok.is_err() {
            out_l[..n].fill(0.0);
            out_r[..n].fill(0.0);
        }
    }

    pub fn process_block_in_place(
        &mut self,
        ch0: &mut [f32],
        mut ch1: Option<&mut [f32]>,
        params: &PluginParamValues,
        midi_ccs: &[(u8, f32)],
        transport_bpm: Option<f64>,
    ) {
        let n = ch0.len().min(self.in_scratch.len());
        if n == 0 {
            return;
        }
        let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.apply_plugin_params(params, midi_ccs, transport_bpm);
            if self.stereo_input {
                if let Some(r) = ch1.as_deref() {
                    let rn = r.len().min(n);
                    for i in 0..rn {
                        self.in_scratch[i] = 0.5 * (ch0[i] + r[i]);
                    }
                    if rn < n {
                        self.in_scratch[rn..n].copy_from_slice(&ch0[rn..n]);
                    }
                } else {
                    self.in_scratch[..n].copy_from_slice(&ch0[..n]);
                }
            } else {
                self.in_scratch[..n].copy_from_slice(&ch0[..n]);
            }
            self.chain
                .process(&self.in_scratch[..n], &mut self.out_scratch[..n]);
            ch0[..n].copy_from_slice(&self.out_scratch[..n]);
            if let Some(r) = ch1.as_deref_mut() {
                let rn = r.len().min(n);
                r[..rn].copy_from_slice(&self.out_scratch[..rn]);
            }
        }));
        if ok.is_err() {
            ch0[..n].fill(0.0);
            if let Some(r) = ch1 {
                let rn = r.len().min(n);
                r[..rn].fill(0.0);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PluginParamValues {
    pub amp_gain: f32,
    pub amp_bass: f32,
    pub amp_middle: f32,
    pub amp_treble: f32,
    pub amp_master: f32,
    pub amp_presence: f32,
    pub gate_threshold_db: f32,
    pub dly_intensity: f32,
    pub rev_mix: f32,
    pub cab_enabled: bool,
    pub host_tempo_sync: bool,
}

impl Default for PluginParamValues {
    fn default() -> Self {
        Self {
            amp_gain: 5.0,
            amp_bass: 5.0,
            amp_middle: 5.0,
            amp_treble: 5.0,
            amp_master: 5.0,
            amp_presence: 5.0,
            gate_threshold_db: -50.0,
            dly_intensity: 3.5,
            rev_mix: 0.25,
            cab_enabled: true,
            host_tempo_sync: false,
        }
    }
}

impl From<&SonixPluginParams> for PluginParamValues {
    fn from(p: &SonixPluginParams) -> Self {
        Self {
            amp_gain: p.amp_gain.value(),
            amp_bass: p.amp_bass.value(),
            amp_middle: p.amp_middle.value(),
            amp_treble: p.amp_treble.value(),
            amp_master: p.amp_master.value(),
            amp_presence: p.amp_presence.value(),
            gate_threshold_db: p.gate_threshold_db.value(),
            dly_intensity: p.dly_intensity.value(),
            rev_mix: p.rev_mix.value(),
            cab_enabled: p.cab_enabled.value(),
            host_tempo_sync: p.host_tempo_sync.value(),
        }
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
    processor: SonixPluginProcessor,
}

impl Default for SonixProPlugin {
    fn default() -> Self {
        let mut processor = SonixPluginProcessor::new(PluginFormat::Vst3);
        processor.prepare(48000.0, 512);
        Self {
            params: Arc::new(SonixPluginParams::default()),
            processor,
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

    pub fn registered_param_ids() -> Vec<String> {
        [
            "amp_1.gain",
            "amp_1.bass",
            "amp_1.middle",
            "amp_1.treble",
            "amp_1.master",
            "amp_1.presence",
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

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(2),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames::const_default(),
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        let max_buf = (buffer_config.max_buffer_size as usize).max(512);
        self.processor.prepare(buffer_config.sample_rate, max_buf);
        self.processor.compile_default_modifiers();
        let stereo_in = audio_io_layout
            .main_input_channels
            .map(|c| c.get() >= 2)
            .unwrap_or(false);
        self.processor.set_stereo_input(stereo_in);
        context.set_latency_samples(self.processor.latency_samples());
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut midi_ccs = [(0u8, 0.0_f32); 256];
        let mut midi_count = 0usize;
        while let Some(event) = context.next_event() {
            if let NoteEvent::MidiCC { cc, value, .. } = event {
                if midi_count < midi_ccs.len() {
                    midi_ccs[midi_count] = (cc, value);
                    midi_count += 1;
                }
            }
        }

        let param_vals = PluginParamValues::from(self.params.as_ref());
        let transport_bpm = context.transport().tempo;
        let slices = buffer.as_slice();
        if slices.len() >= 2 {
            let (head, tail) = slices.split_at_mut(1);
            self.processor.process_block_in_place(
                head[0],
                Some(tail[0]),
                &param_vals,
                &midi_ccs[..midi_count],
                transport_bpm,
            );
        } else if slices.len() == 1 {
            self.processor.process_block_in_place(
                slices[0],
                None,
                &param_vals,
                &midi_ccs[..midi_count],
                transport_bpm,
            );
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
