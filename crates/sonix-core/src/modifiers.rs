//! crates/sonix-core/src/modifiers.rs
//! Matriz Universal de Modifiers (Expression, MIDI CC, LFO, Envelope Follower)

use crate::preset::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierSource {
    ExpPedal1,
    ExpPedal2,
    MidiCc(u8),
    InternalLfo1,
    InternalLfo2,
    EnvelopeFollower,
    SceneController,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModifierBinding {
    pub target_block_id: String,
    pub target_param: String,
    pub source: ModifierSource,
    pub min_val: f32,
    pub max_val: f32,
    pub curve_slope: f32, // 0.0 to 1.0 (0.5 is linear)
    pub damping_ms: f32,
}

impl ModifierBinding {
    pub fn evaluate(&self, normalized_input: f32) -> f32 {
        let u = normalized_input.clamp(0.0, 1.0);
        let curved_u = if (self.curve_slope - 0.5).abs() < 1e-4 {
            u
        } else {
            let exponent = ((1.0 - self.curve_slope) / self.curve_slope.max(0.01)).clamp(0.1, 10.0);
            u.powf(exponent)
        };
        self.min_val + curved_u * (self.max_val - self.min_val)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ModifierMatrix {
    bindings: Vec<ModifierBinding>,
}

impl ModifierMatrix {
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub fn add_binding(&mut self, binding: ModifierBinding) -> Result<(), CoreError> {
        if binding.target_block_id.trim().is_empty() {
            return Err(CoreError::InvalidBinding(
                "ID de bloco alvo vazio".to_string(),
            ));
        }
        self.bindings.push(binding);
        Ok(())
    }

    pub fn bindings_for_block(&self, block_id: &str) -> Vec<&ModifierBinding> {
        self.bindings
            .iter()
            .filter(|b| b.target_block_id == block_id)
            .collect()
    }

    pub fn has_modifier_for(&self, block_id: &str, param: &str) -> bool {
        self.bindings
            .iter()
            .any(|b| b.target_block_id == block_id && b.target_param == param)
    }

    pub fn apply_or_fallback(
        &self,
        block_id: &str,
        param: &str,
        direct_val: f32,
        normalized_source: f32,
    ) -> f32 {
        if let Some(binding) = self
            .bindings
            .iter()
            .find(|b| b.target_block_id == block_id && b.target_param == param)
        {
            binding.evaluate(normalized_source)
        } else {
            direct_val
        }
    }
}
