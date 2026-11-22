//! SONIX PRO Core - Presets, Serialization, Schema and Modifiers Engine

pub mod preset;
pub mod modifiers;
pub mod footswitch;
pub mod persistence;

pub use preset::{BlockChannel, BlockChannelState, BlockNode, CoreError, Preset, Scene, Setlist};
pub use modifiers::{ModifierBinding, ModifierMatrix, ModifierSource};
pub use footswitch::{FootswitchArray, FootswitchMode, FootswitchUnit};
pub use persistence::{
    BlockDto, GridDto, ModifierDto, PackageIntegrityError, PresetMetadata,
    PresetSchemaValidationError, SceneDto, SforgePackage, SonixPresetDoc,
};
