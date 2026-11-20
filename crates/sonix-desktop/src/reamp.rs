//! Reamping com medição de atraso por loopback e registro de metadados da sessão (Spec 049 CA-39).

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReampSessionMetadata {
    pub measured_delay_samples: usize,
    pub backend: String,
    pub driver: String,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub di_source: String,
}

/// Mede o atraso de ida e volta em amostras por correlação cruzada entre `probe` enviado
/// e `recorded` retornado pelo loopback.
pub fn measure_loopback_delay_samples(probe: &[f32], recorded: &[f32]) -> usize {
    if probe.is_empty() || recorded.len() < probe.len() {
        return 0;
    }
    let max_lag = recorded.len() - probe.len();
    let mut best_lag = 0usize;
    let mut best_corr = f64::NEG_INFINITY;

    for lag in 0..=max_lag {
        let mut dot = 0.0f64;
        for (i, &p) in probe.iter().enumerate() {
            dot += (p as f64) * (recorded[lag + i] as f64);
        }
        if dot > best_corr {
            best_corr = dot;
            best_lag = lag;
        }
    }
    best_lag
}

/// Alinha a gravação de retorno compensando `delay_samples` medido por loopback (erro <= 1 amostra).
pub const MAX_REAMP_SAMPLES: usize = 480_000;

pub fn align_reamp_recording(recorded: &[f32], delay_samples: usize, target_len: usize) -> Vec<f32> {
    let safe_len = target_len.min(MAX_REAMP_SAMPLES);
    let mut out = Vec::new();
    if safe_len == 0 || out.try_reserve_exact(safe_len).is_err() {
        return out;
    }
    out.resize(safe_len, 0.0);
    if delay_samples >= recorded.len() {
        return out;
    }
    let avail = (recorded.len() - delay_samples).min(safe_len);
    out[..avail].copy_from_slice(&recorded[delay_samples..delay_samples + avail]);
    out
}

pub fn save_reamp_session_metadata(path: &Path, meta: &ReampSessionMetadata) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Falha ao criar diretório de sessão reamp: {}", e))?;
    }
    let json = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("Falha ao serializar metadados de reamp: {}", e))?;
    std::fs::write(path, json)
        .map_err(|e| format!("Falha ao gravar metadados de reamp: {}", e))
}
