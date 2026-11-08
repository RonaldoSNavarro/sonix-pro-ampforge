//! Structured error definitions for AmpForge™ DSP Engine
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum InvalidAudioConfigError {
    #[error("Taxa de amostragem não suportada: {0} Hz. Suportadas: 44100, 48000, 88200, 96000")]
    UnsupportedSampleRate(u32),

    #[error("Tamanho de buffer inválido: {0} amostras. Suportados: potências de 2 entre 32 e 512")]
    InvalidBufferSize(usize),
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum DspEngineError {
    #[error("Configuração de áudio inválida: {0}")]
    Config(#[from] InvalidAudioConfigError),

    #[error("Orçamento de CPU excedido ({usage_pct:.1}% >= 88.0%). Inserção do bloco recusada para evitar dropout (xrun).")]
    CpuOverload { usage_pct: f32 },

    #[error("Comutação de topologia inválida para o grafo ID: {0}")]
    InvalidTopologyId(usize),
}
