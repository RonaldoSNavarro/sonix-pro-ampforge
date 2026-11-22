//! crates/ampforge-dsp/src/unidades.rs
//! Convenção única de escala física entre volts nos circuitos analógicos e amplitude Full Scale (AUD-29).

/// Fator de conversão entre Volts de pico na entrada do circuito e unidades Full Scale (`1.0 FS = 0 dBFS`).
/// Convenção adotada: `0 dBFS = 1 V` de pico na entrada do pedal.
pub const VOLTS_POR_FS: f32 = 1.0;

/// Converte uma tensão em Volts (pico) para unidades Full Scale normalizadas no motor DSP.
#[inline(always)]
pub fn volts_para_fs(volts: f32) -> f32 {
    volts / VOLTS_POR_FS
}

/// Converte uma amplitude Full Scale para Volts equivalentes de entrada/circuito.
#[inline(always)]
pub fn fs_para_volts(fs: f32) -> f32 {
    fs * VOLTS_POR_FS
}
