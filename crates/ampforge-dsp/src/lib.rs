//! AmpForge™ DSP Real-Time Lock-Free Audio Engine
//!
//! Núcleo em Rust para processamento de áudio digital de ultrabaixa latência (< 1.5 ms),
//! comunicação lock-free SPSC, precisão mista 32-bit SIMD / 64-bit solvers,
//! suavização de parâmetros de 1 polo e proteção contra sobrecarga de CPU.

pub mod config;
pub mod errors;
pub mod queue;
pub mod smoother;
pub mod oversampling;
pub mod precision;
pub mod cpu;
pub mod engine;
pub mod dag;
pub mod amp;
pub mod modshop;

// Re-exports ergonômicos
pub use config::{AudioConfig, BufferSize, SampleRate};
pub use errors::{DspEngineError, InvalidAudioConfigError};
pub use queue::{DspCommand, SpscCommandQueue, SpscTelemetryQueue, TelemetrySnapshot};
pub use smoother::OnePoleSmoother;
pub use oversampling::{HalfBandOversampler, OversamplingFactor};
pub use precision::{IirBiquad64, MixedPrecisionSolver};
pub use cpu::CpuMonitor;
pub use engine::DspEngine;
