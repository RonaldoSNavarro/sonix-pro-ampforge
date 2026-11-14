//! crates/ampforge-dsp/src/fx/mod.rs
//! Módulo de Efeitos de Áudio: Dinâmica, Drive, Equalização, Modulação, Pitch, Delay e Reverb

pub mod dynamics;
pub mod drive;
pub mod eq;
pub mod modulation;
pub mod pitch;
pub mod delay;
pub mod reverb;

pub use dynamics::{
    CompressorProcessor, CompressorTopology, GateState, NoiseGateMode, NoiseGateProcessor,
};
pub use drive::{ClippingDiode, DriveModel, DriveProcessor};
pub use eq::{EqMode, EqProcessor, ParametricBand};
pub use modulation::{ModModel, ModulationProcessor, NoteDivision};
pub use pitch::{PitchKey, PitchModel, PitchProcessor, ScaleType};
pub use delay::{DelayModel, DelayProcessor};
pub use reverb::{ReverbModel, ReverbProcessor};
