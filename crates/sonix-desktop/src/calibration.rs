//! crates/sonix-desktop/src/calibration.rs
//! Assistente de Calibração de Entrada, Diagnósticos de Hardware e Síntese de Teste Karplus-Strong
//! Spec 019: Entrada de Guitarra na Placa Onboard: Calibração, Clipping e Sinal de Teste

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticReason {
    Ok,
    InputTooHot,
    InputTooLow,
    Hum50Hz,
    Hum60Hz,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputCalibrationResult {
    pub proposed_boost: f32,
    pub proposed_gate_threshold_db: f32,
    pub diagnostic: DiagnosticReason,
    pub noise_floor_rms_db: f32,
    pub raw_peak_db: f32,
}

/// Mede a energia espectral em uma frequência específica através de DFT de 1 bin
fn discrete_frequency_energy(samples: &[f32], freq: f32, sample_rate: f32) -> f32 {
    if samples.is_empty() || sample_rate <= 0.0 {
        return 0.0;
    }
    let n_len = samples.len();
    let omega = 2.0 * std::f32::consts::PI * freq / sample_rate;
    let mut real = 0.0f32;
    let mut imag = 0.0f32;

    for (n, &s) in samples.iter().enumerate() {
        let phase = omega * n as f32;
        real += s * phase.cos();
        imag -= s * phase.sin();
    }

    (real * real + imag * imag) / (n_len as f32 * n_len as f32)
}

/// Calibra ganho de entrada e threshold de gate sobre amostras de silêncio e palhetada
pub fn calibrate_input(noise_samples: &[f32], strum_samples: &[f32], sample_rate: f32) -> InputCalibrationResult {
    // 1. Análise de Silêncio (Piso de ruído RMS e detecção de Hum)
    let noise_rms = if noise_samples.is_empty() {
        0.0003162 // -70 dBFS default fallback
    } else {
        (noise_samples.iter().map(|s| s * s).sum::<f32>() / noise_samples.len() as f32)
            .sqrt()
            .max(1e-9)
    };
    let noise_rms_db = 20.0 * noise_rms.log10();

    let e50 = discrete_frequency_energy(noise_samples, 50.0, sample_rate)
        + discrete_frequency_energy(noise_samples, 150.0, sample_rate);
    let e60 = discrete_frequency_energy(noise_samples, 60.0, sample_rate)
        + discrete_frequency_energy(noise_samples, 180.0, sample_rate);

    let mut detected_hum = DiagnosticReason::Ok;
    let broadband_bin_expect = (noise_rms * noise_rms) / noise_samples.len().max(1) as f32;
    if e60 > 16.0 * broadband_bin_expect && e60 > 2.0 * e50 && e60 > 1e-9 {
        detected_hum = DiagnosticReason::Hum60Hz;
    } else if e50 > 16.0 * broadband_bin_expect && e50 > 2.0 * e60 && e50 > 1e-9 {
        detected_hum = DiagnosticReason::Hum50Hz;
    }

    // 2. Análise de Palhetada (Pico bruto e headroom)
    let raw_peak = if strum_samples.is_empty() {
        0.0316228 // -30 dBFS default
    } else {
        strum_samples.iter().fold(0.0f32, |acc, &s| acc.max(s.abs())).max(1e-9)
    };
    let raw_peak_db = 20.0 * raw_peak.log10();

    // Alvo: pico após boost em -12 dBFS (10^(-12/20) = 0.25118864)
    let target_peak = 0.25118864f32;
    let ideal_boost = target_peak / raw_peak;

    let (proposed_boost, diagnostic) = if raw_peak_db >= -1.0 {
        (1.0f32.min(ideal_boost), DiagnosticReason::InputTooHot)
    } else if raw_peak_db < -42.0 {
        (8.0f32, DiagnosticReason::InputTooLow)
    } else {
        (ideal_boost.clamp(0.1, 8.0), detected_hum)
    };

    // Threshold de gate: ruído RMS após boost + 6 dB
    let noise_after_boost_db = noise_rms_db + 20.0 * proposed_boost.log10();
    let proposed_gate = (noise_after_boost_db + 6.0).clamp(-90.0, -10.0);

    InputCalibrationResult {
        proposed_boost,
        proposed_gate_threshold_db: proposed_gate,
        diagnostic,
        noise_floor_rms_db: noise_rms_db,
        raw_peak_db,
    }
}

/// Síntese determinística de riff de guitarra por algoritmo Karplus-Strong
/// Retorna buffer com pico calibrado exatamente em -12 dBFS ± 0.5 dB
pub fn generate_karplus_strong_riff(sample_rate: f32, seed: u64) -> Vec<f32> {
    let sr = sample_rate.max(8000.0);
    // Notas do riff clássico (E2, G2, A2, Bb2, A2, G2)
    let notes = [
        (82.41f32, 0.4f32),  // E2
        (98.00f32, 0.4f32),  // G2
        (110.00f32, 0.5f32), // A2
        (116.54f32, 0.3f32), // Bb2
        (110.00f32, 0.4f32), // A2
        (98.00f32, 0.6f32),  // G2
    ];

    let total_duration: f32 = notes.iter().map(|(_, d)| *d).sum();
    let total_samples = (total_duration * sr).round() as usize;
    let mut output = Vec::with_capacity(total_samples);

    let mut lcg = seed;
    let rho = 0.996f32; // Coeficiente de amortecimento da corda

    for &(freq, dur) in &notes {
        let n_period = (sr / freq).round().max(4.0) as usize;
        let note_samples = (dur * sr).round() as usize;

        // Inicializa linha de retardo com ruído determinístico
        let mut delay_line = vec![0.0f32; n_period];
        for s in delay_line.iter_mut() {
            lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
            let u = (lcg >> 33) as f32 / (1u32 << 31) as f32 - 1.0;
            *s = u;
        }

        let mut read_idx = 0usize;
        let mut prev_val = 0.0f32;

        for _ in 0..note_samples {
            let next_idx = (read_idx + 1) % n_period;
            let current_val = delay_line[read_idx];
            let next_val = delay_line[next_idx];
            // Filtro de média móvel Karplus-Strong
            let filtered = 0.5 * (current_val + next_val) * rho;
            delay_line[read_idx] = filtered;

            output.push(prev_val);
            prev_val = filtered;

            read_idx = next_idx;
        }
    }

    // Normaliza para pico de -12 dBFS (0.25118864)
    let max_peak = output.iter().fold(0.0f32, |acc, &s| acc.max(s.abs())).max(1e-6);
    let scale = 0.25118864 / max_peak;
    for s in output.iter_mut() {
        *s *= scale;
    }

    output
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CalibrationEntryDisk {
    boost: f32,
    gate_threshold_db: f32,
    salvo_em: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CalibrationFileDisk {
    versao: u32,
    dispositivos: HashMap<String, CalibrationEntryDisk>,
}

/// Diretório padrão de dados persistentes do SONIX PRO (`%APPDATA%\SonixPro`).
pub fn default_sonix_data_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("LOCALAPPDATA"))
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("SonixPro")
}

/// Armazenamento persistente em disco de perfis calibrados por dispositivo de áudio (`<dir>/calibration.json`).
#[derive(Debug, Clone)]
pub struct CalibrationStore {
    dir: PathBuf,
    file_path: PathBuf,
    profiles: Arc<Mutex<HashMap<String, CalibrationEntryDisk>>>,
}

impl CalibrationStore {
    pub fn open(dir: impl AsRef<Path>) -> Self {
        let dir_path = dir.as_ref().to_path_buf();
        let file_path = dir_path.join("calibration.json");
        let mut profiles = HashMap::new();

        if file_path.exists() {
            let parsed_ok = std::fs::read_to_string(&file_path)
                .ok()
                .and_then(|txt| serde_json::from_str::<CalibrationFileDisk>(&txt).ok())
                .filter(|doc| doc.versao == 1);

            if let Some(doc) = parsed_ok {
                profiles = doc.dispositivos;
            } else {
                let corrupt_path = dir_path.join("calibration.json.corrompido");
                let _ = std::fs::remove_file(&corrupt_path);
                let _ = std::fs::rename(&file_path, &corrupt_path);
            }
        }

        Self {
            dir: dir_path,
            file_path,
            profiles: Arc::new(Mutex::new(profiles)),
        }
    }

    pub fn save(&self, device_id: &str, boost: f32, gate_threshold_db: f32) {
        if let Ok(mut map) = self.profiles.lock() {
            map.insert(
                device_id.to_string(),
                CalibrationEntryDisk {
                    boost,
                    gate_threshold_db,
                    salvo_em: crate::api::format_utc_iso8601_now(),
                },
            );
            let doc = CalibrationFileDisk {
                versao: 1,
                dispositivos: map.clone(),
            };
            if let Ok(json) = serde_json::to_string_pretty(&doc) {
                let _ = std::fs::create_dir_all(&self.dir);
                let tmp_path = self.dir.join(format!("calibration.json.{}.tmp", std::process::id()));
                if std::fs::write(&tmp_path, json.as_bytes()).is_ok() {
                    if std::fs::rename(&tmp_path, &self.file_path).is_err() {
                        let _ = std::fs::remove_file(&tmp_path);
                    }
                }
            }
        }
    }

    pub fn get(&self, device_id: &str) -> Option<(f32, f32)> {
        if let Ok(map) = self.profiles.lock() {
            map.get(device_id)
                .map(|entry| (entry.boost, entry.gate_threshold_db))
        } else {
            None
        }
    }
}
