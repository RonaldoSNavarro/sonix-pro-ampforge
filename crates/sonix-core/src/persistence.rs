//! crates/sonix-core/src/persistence.rs
//! Persistência 100% Offline-First, Schema sonixpreset-v1.json e Pacotes .sforgepkg

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PresetSchemaValidationError {
    #[error("JSON malformado: {0}")]
    MalformedJson(String),
    #[error("Versão de schema incompatível: {0} (esperado 1.0.0)")]
    IncompatibleSchemaVersion(String),
    #[error("Campo obrigatório ausente: {0}")]
    MissingField(String),
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PackageIntegrityError {
    #[error("Divergência de soma de verificação SHA-256 para '{file}': esperado {expected}, calculado {actual}")]
    ChecksumMismatch {
        file: String,
        expected: String,
        actual: String,
    },
    #[error("Tentativa de travessia de caminho (path traversal) detectada em '{0}'")]
    PathTraversalDetected(String),
    #[error("Arquivo de pacote corrompido ou ilegível: {0}")]
    CorruptedArchive(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresetMetadata {
    pub id: String,
    pub name: String,
    pub author: String,
    pub category: String,
    #[serde(rename = "tempoBpm")]
    pub tempo_bpm: f32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "footswitchMode")]
    pub footswitch_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneDto {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockDto {
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    #[serde(rename = "type")]
    pub block_type: String,
    pub row: usize,
    pub col: usize,
    pub oversampling: String,
    #[serde(rename = "activeChannelByScene")]
    pub active_channel_by_scene: [usize; 8],
    #[serde(rename = "bypassByScene")]
    pub bypass_by_scene: [bool; 8],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GridDto {
    pub rows: usize,
    pub cols: usize,
    pub blocks: Vec<BlockDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModifierDto {
    pub id: String,
    pub source: String,
    #[serde(rename = "targetBlockId")]
    pub target_block_id: String,
    #[serde(rename = "targetParam")]
    pub target_param: String,
    #[serde(rename = "minVal")]
    pub min_val: f32,
    #[serde(rename = "maxVal")]
    pub max_val: f32,
    #[serde(rename = "curveSlope")]
    pub curve_slope: f32,
    #[serde(rename = "dampingMs")]
    pub damping_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SonixPresetDoc {
    #[serde(rename = "$schema")]
    pub schema: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    pub metadata: PresetMetadata,
    #[serde(rename = "activeSceneIndex")]
    pub active_scene_index: usize,
    pub scenes: Vec<SceneDto>,
    pub grid: GridDto,
    pub modifiers: Vec<ModifierDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<HashMap<String, HashMap<String, HashMap<String, serde_json::Value>>>>,
}

impl SonixPresetDoc {
    pub fn to_json_str(&self) -> Result<String, PresetSchemaValidationError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PresetSchemaValidationError::MalformedJson(e.to_string()))
    }

    pub fn default_boot_params() -> HashMap<String, serde_json::Value> {
        let mut m = HashMap::new();
        m.insert("gate.enabled".to_string(), serde_json::json!(true));
        m.insert("gate.threshold_db".to_string(), serde_json::json!(-60.0));
        m.insert("drive.enabled".to_string(), serde_json::json!(false));
        m.insert("drive.drive".to_string(), serde_json::json!(5.0));
        m.insert("drive.tone".to_string(), serde_json::json!(5.0));
        m.insert("drive.level".to_string(), serde_json::json!(5.0));
        m.insert("split.enabled".to_string(), serde_json::json!(false));
        m.insert("split.mode".to_string(), serde_json::json!("ab"));
        m.insert("split.balance".to_string(), serde_json::json!(0.5));
        m.insert("split.crossover_hz".to_string(), serde_json::json!(600.0));
        m.insert("merge.level_a_db".to_string(), serde_json::json!(0.0));
        m.insert("merge.level_b_db".to_string(), serde_json::json!(0.0));
        m.insert("merge.phase_invert_b".to_string(), serde_json::json!(false));
        m.insert("amp_1.gain".to_string(), serde_json::json!(5.0));
        m.insert("amp_1.bass".to_string(), serde_json::json!(5.0));
        m.insert("amp_1.middle".to_string(), serde_json::json!(5.0));
        m.insert("amp_1.treble".to_string(), serde_json::json!(5.0));
        m.insert("amp_1.presence".to_string(), serde_json::json!(5.0));
        m.insert("amp_1.master".to_string(), serde_json::json!(7.0));
        m.insert("amp_1.model".to_string(), serde_json::json!("us_twin_blackface_65"));
        m.insert("cab_1.enabled".to_string(), serde_json::json!(true));
        m.insert("cab_1.ir_enabled".to_string(), serde_json::json!(false));
        m.insert("cab_1.mic1.position".to_string(), serde_json::json!(0.2));
        m.insert("cab_1.mic1.distance_cm".to_string(), serde_json::json!(2.0));
        m.insert("comp_2.enabled".to_string(), serde_json::json!(false));
        m.insert("amp_2.enabled".to_string(), serde_json::json!(false));
        m.insert("cab_2.enabled".to_string(), serde_json::json!(false));
        m.insert("mod_2.enabled".to_string(), serde_json::json!(false));
        m.insert("eq_1.enabled".to_string(), serde_json::json!(false));
        m.insert("dly_1.enabled".to_string(), serde_json::json!(false));
        m.insert("rev_1.enabled".to_string(), serde_json::json!(false));
        m.insert("scene.active".to_string(), serde_json::json!("A"));
        m
    }

    pub fn from_json_str(s: &str) -> Result<Self, PresetSchemaValidationError> {
        let mut doc: Self = serde_json::from_str(s)
            .map_err(|e| PresetSchemaValidationError::MalformedJson(e.to_string()))?;

        if doc.schema_version != "1.0.0" && doc.schema_version != "1.1.0" {
            return Err(PresetSchemaValidationError::IncompatibleSchemaVersion(
                doc.schema_version,
            ));
        }

        if doc.schema_version == "1.0.0" {
            if doc.params.is_none() {
                doc.params = Some(Self::default_boot_params());
            }
            doc.schema_version = "1.1.0".to_string();
        }

        Ok(doc)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BundlePayload {
    manifest: HashMap<String, String>,
    preset_json: String,
    files: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct SforgePackage {
    pub preset: SonixPresetDoc,
    irs: HashMap<String, Vec<u8>>,
}

impl SforgePackage {
    pub fn new(preset: SonixPresetDoc) -> Self {
        Self {
            preset,
            irs: HashMap::new(),
        }
    }

    pub fn add_ir(&mut self, relative_path: &str, bytes: Vec<u8>) -> Result<(), PackageIntegrityError> {
        if relative_path.contains("..") || relative_path.starts_with('/') || relative_path.starts_with('\\') {
            return Err(PackageIntegrityError::PathTraversalDetected(
                relative_path.to_string(),
            ));
        }
        self.irs.insert(relative_path.to_string(), bytes);
        Ok(())
    }

    pub fn get_ir(&self, relative_path: &str) -> Option<&Vec<u8>> {
        self.irs.get(relative_path)
    }

    pub fn generate_manifest(&self) -> HashMap<String, String> {
        let mut manifest = HashMap::new();

        let preset_json = self.preset.to_json_str().unwrap_or_default();
        manifest.insert("preset.json".to_string(), Self::hash_bytes(preset_json.as_bytes()));

        for (path, bytes) in &self.irs {
            manifest.insert(path.clone(), Self::hash_bytes(bytes));
        }

        manifest
    }

    fn hash_bytes(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    pub fn export_bundle(&self) -> Result<Vec<u8>, PackageIntegrityError> {
        let manifest = self.generate_manifest();
        let preset_json = self.preset.to_json_str().map_err(|e| {
            PackageIntegrityError::CorruptedArchive(e.to_string())
        })?;

        let payload = BundlePayload {
            manifest,
            preset_json,
            files: self.irs.clone(),
        };

        serde_json::to_vec(&payload)
            .map_err(|e| PackageIntegrityError::CorruptedArchive(e.to_string()))
    }

    pub fn import_bundle(bytes: &[u8]) -> Result<Self, PackageIntegrityError> {
        let payload: BundlePayload = serde_json::from_slice(bytes)
            .map_err(|e| PackageIntegrityError::CorruptedArchive(e.to_string()))?;

        // 1. Verify preset.json hash (mandatory in manifest)
        let actual_preset_hash = Self::hash_bytes(payload.preset_json.as_bytes());
        let expected_preset_hash = payload.manifest.get("preset.json").ok_or_else(|| {
            PackageIntegrityError::CorruptedArchive("preset.json ausente no manifesto".to_string())
        })?;
        if &actual_preset_hash != expected_preset_hash {
            return Err(PackageIntegrityError::ChecksumMismatch {
                file: "preset.json".to_string(),
                expected: expected_preset_hash.clone(),
                actual: actual_preset_hash,
            });
        }

        let preset = SonixPresetDoc::from_json_str(&payload.preset_json)
            .map_err(|e| PackageIntegrityError::CorruptedArchive(e.to_string()))?;

        // 2. Verify all files
        let mut irs = HashMap::new();
        for (rel_path, file_bytes) in payload.files {
            if rel_path.contains("..") || rel_path.starts_with('/') || rel_path.starts_with('\\') {
                return Err(PackageIntegrityError::PathTraversalDetected(rel_path));
            }

            let actual_hash = Self::hash_bytes(&file_bytes);
            let expected_hash = payload.manifest.get(&rel_path).ok_or_else(|| {
                PackageIntegrityError::CorruptedArchive(format!(
                    "Arquivo '{}' não encontrado no manifesto",
                    rel_path
                ))
            })?;
            if &actual_hash != expected_hash {
                return Err(PackageIntegrityError::ChecksumMismatch {
                    file: rel_path,
                    expected: expected_hash.clone(),
                    actual: actual_hash,
                });
            }

            irs.insert(rel_path, file_bytes);
        }

        Ok(Self { preset, irs })
    }
}
