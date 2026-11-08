//! crates/ampforge-dsp/src/fx/mod.rs
//! Módulo de Efeitos de Áudio: Dinâmica, Drive, Equalização e Modulação

pub mod dynamics;
pub mod drive;
pub mod eq;

pub use dynamics::{CompressorProcessor, CompressorTopology, NoiseGateMode, NoiseGateProcessor};
pub use drive::{ClippingDiode, DriveModel, DriveProcessor};
pub use eq::{EqMode, EqProcessor, ParametricBand};
