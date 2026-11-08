//! SONIX PRO Core - Presets, Serialization, Schema and Modifiers Engine

pub mod preset;
pub mod modifiers;
pub mod footswitch;

pub use preset::{BlockChannel, BlockChannelState, BlockNode, CoreError, GaplessCrossfader, Preset, Scene, Setlist};
pub use modifiers::{ModifierBinding, ModifierMatrix, ModifierSource};
pub use footswitch::{FootswitchArray, FootswitchMode, FootswitchUnit};
