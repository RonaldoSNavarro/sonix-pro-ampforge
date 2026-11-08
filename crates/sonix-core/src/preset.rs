//! crates/sonix-core/src/preset.rs
//! Hierarquia Setlist/Preset, 8 Cenas Gapless, 4 Canais A/B/C/D por Bloco

use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    #[error("Índice de cena inválido: {0} (máx 7)")]
    InvalidSceneIndex(usize),
    #[error("Canal de bloco inválido: {0} (válidos: A, B, C, D)")]
    InvalidChannel(char),
    #[error("Bloco '{0}' não encontrado")]
    BlockNotFound(String),
    #[error("Vínculo de modifier inválido: {0}")]
    InvalidBinding(String),
    #[error("Erro de E/S ou formato: {0}")]
    Io(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockChannel {
    A,
    B,
    C,
    D,
}

impl BlockChannel {
    pub fn as_char(&self) -> char {
        match self {
            Self::A => 'A',
            Self::B => 'B',
            Self::C => 'C',
            Self::D => 'D',
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockChannelState {
    pub model_id: String,
    pub params: HashMap<String, f32>,
}

impl BlockChannelState {
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            params: HashMap::new(),
        }
    }

    pub fn set_param(&mut self, name: impl Into<String>, val: f32) {
        self.params.insert(name.into(), val);
    }

    pub fn get_param(&self, name: &str) -> Option<f32> {
        self.params.get(name).copied()
    }
}

#[derive(Debug, Clone)]
pub struct BlockNode {
    pub id: String,
    pub block_type: String,
    pub channels: HashMap<BlockChannel, BlockChannelState>,
    pub active_channel_by_scene: [BlockChannel; 8],
    pub bypass_by_scene: [bool; 8],
}

impl BlockNode {
    pub fn new(id: impl Into<String>, block_type: impl Into<String>) -> Self {
        let mut channels = HashMap::new();
        channels.insert(BlockChannel::A, BlockChannelState::new("default"));
        channels.insert(BlockChannel::B, BlockChannelState::new("default"));
        channels.insert(BlockChannel::C, BlockChannelState::new("default"));
        channels.insert(BlockChannel::D, BlockChannelState::new("default"));

        Self {
            id: id.into(),
            block_type: block_type.into(),
            channels,
            active_channel_by_scene: [BlockChannel::A; 8],
            bypass_by_scene: [false; 8],
        }
    }

    pub fn set_channel_state(&mut self, ch: BlockChannel, state: BlockChannelState) {
        self.channels.insert(ch, state);
    }

    pub fn channel_state(&self, ch: BlockChannel) -> &BlockChannelState {
        self.channels.get(&ch).expect("Canal sempre presente")
    }

    pub fn set_active_channel_for_scene(&mut self, scene_idx: usize, ch: BlockChannel) {
        if scene_idx < 8 {
            self.active_channel_by_scene[scene_idx] = ch;
        }
    }

    pub fn active_channel_for_scene(&self, scene_idx: usize) -> BlockChannel {
        if scene_idx < 8 {
            self.active_channel_by_scene[scene_idx]
        } else {
            BlockChannel::A
        }
    }

    pub fn count_active_channels_in_scene(&self, _scene_idx: usize) -> usize {
        1 // Exactly 1 active channel per block, 3 remain idle
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    pub index: usize,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub tempo_bpm: f32,
    scenes: [Scene; 8],
    pub blocks: HashMap<String, BlockNode>,
}

impl Preset {
    pub fn new(id: impl Into<String>, name: impl Into<String>, tempo_bpm: f32) -> Self {
        let scene_names = [
            "Scene A", "Scene B", "Scene C", "Scene D",
            "Scene E", "Scene F", "Scene G", "Scene H",
        ];
        let scenes = std::array::from_fn(|i| Scene {
            index: i,
            name: scene_names[i].to_string(),
        });

        Self {
            id: id.into(),
            name: name.into(),
            tempo_bpm,
            scenes,
            blocks: HashMap::new(),
        }
    }

    pub fn scenes(&self) -> &[Scene; 8] {
        &self.scenes
    }

    pub fn scene(&self, idx: usize) -> Result<&Scene, CoreError> {
        if idx < 8 {
            Ok(&self.scenes[idx])
        } else {
            Err(CoreError::InvalidSceneIndex(idx))
        }
    }
}

pub struct Setlist {
    pub name: String,
    presets: Vec<Preset>,
}

impl Setlist {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            presets: Vec::new(),
        }
    }

    pub fn add_preset(&mut self, preset: Preset) {
        self.presets.push(preset);
    }

    pub fn presets(&self) -> &[Preset] {
        &self.presets
    }
}

pub struct GaplessCrossfader {
    sample_rate: f32,
    crossfade_samples: usize,
}

impl GaplessCrossfader {
    pub fn new(sample_rate: f32, crossfade_ms: f32) -> Self {
        let samples = (crossfade_ms * 0.001 * sample_rate).round() as usize;
        Self {
            sample_rate,
            crossfade_samples: samples.max(1),
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn crossfade_samples(&self) -> usize {
        self.crossfade_samples
    }

    pub fn crossfade(&self, old_buf: &[f32], new_buf: &[f32], out_buf: &mut [f32]) {
        let n = out_buf.len().min(old_buf.len()).min(new_buf.len());
        for i in 0..n {
            let t = i as f32 / n as f32; // 0.0 to 1.0
            let angle = t * (std::f32::consts::PI * 0.5);
            let w_out = angle.cos().powi(2);
            let w_in = angle.sin().powi(2);
            out_buf[i] = old_buf[i] * w_out + new_buf[i] * w_in;
        }
    }
}
