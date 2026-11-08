//! SONIX PRO // AMPFORGE™ Desktop Host Standalone (Tauri v2 + Rust)

pub mod audio;
pub mod midi;

pub use audio::{AudioBackendType, AudioDriverError, DesktopAudioDevice, DesktopAudioHost};
pub use midi::{DesktopMidiHost, MidiMessageType};
