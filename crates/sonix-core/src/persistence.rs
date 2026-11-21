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

pub const MAX_PACKAGE_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_RESOURCE_BYTES: usize = 20 * 1024 * 1024;
pub const MAX_PACKAGE_RESOURCES: usize = 256;

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
    #[error("Limite excedido na importação do pacote: {0}")]
    LimitExceeded(String),
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
        m.insert("merge.row_1_level_db".to_string(), serde_json::json!(-6.0));
        m.insert("merge.row_1_phase_invert".to_string(), serde_json::json!(false));
        m.insert("amp_1.gain".to_string(), serde_json::json!(5.0));
        m.insert("amp_1.bass".to_string(), serde_json::json!(5.0));
        m.insert("amp_1.middle".to_string(), serde_json::json!(5.0));
        m.insert("amp_1.treble".to_string(), serde_json::json!(5.0));
        m.insert("amp_1.presence".to_string(), serde_json::json!(5.0));
        m.insert("amp_1.master".to_string(), serde_json::json!(7.0));
        m.insert("amp_1.model".to_string(), serde_json::json!("us_twin_blackface_65"));
        m.insert("cab_1.enabled".to_string(), serde_json::json!(true));
        m.insert("cab_1.speaker".to_string(), serde_json::json!("alnico_blue_2x12"));
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
struct BundlePayloadV2 {
    version: u32,
    manifest: HashMap<String, String>,
    preset_json: String,
    #[serde(default)]
    extra_presets_json: Vec<String>,
    files_b64: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleEnvelope {
    #[serde(default)]
    version: Option<u32>,
    manifest: HashMap<String, String>,
    preset_json: String,
    #[serde(default)]
    extra_presets_json: Vec<String>,
    #[serde(default)]
    files: Option<HashMap<String, Vec<u8>>>,
    #[serde(default)]
    files_b64: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct SforgePackage {
    pub preset: SonixPresetDoc,
    extra_presets: Vec<SonixPresetDoc>,
    irs: HashMap<String, Vec<u8>>,
}

const B64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(((input.len() + 2) / 3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        out.push(B64_ALPHABET[b0 >> 2] as char);
        out.push(B64_ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            out.push(B64_ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(B64_ALPHABET[b2 & 0x3f] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn b64_val(b: u8) -> Option<u8> {
    match b {
        b'A'..=b'Z' => Some(b - b'A'),
        b'a'..=b'z' => Some(b - b'a' + 26),
        b'0'..=b'9' => Some(b - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn base64_decode(input: &str) -> Result<Vec<u8>, PackageIntegrityError> {
    let raw = input.as_bytes();
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    if raw.len() % 4 != 0 {
        return Err(PackageIntegrityError::CorruptedArchive(
            "Comprimento Base64 inválido".to_string(),
        ));
    }
    let pad = if raw.ends_with(b"==") {
        2
    } else if raw.ends_with(b"=") {
        1
    } else {
        0
    };
    let mut out = Vec::with_capacity((raw.len() / 4) * 3 - pad);
    let num_chunks = raw.len() / 4;
    for (ci, chunk) in raw.chunks_exact(4).enumerate() {
        let is_last = ci + 1 == num_chunks;
        let v0 = b64_val(chunk[0]).ok_or_else(|| {
            PackageIntegrityError::CorruptedArchive("Caractere Base64 inválido".to_string())
        })?;
        let v1 = b64_val(chunk[1]).ok_or_else(|| {
            PackageIntegrityError::CorruptedArchive("Caractere Base64 inválido".to_string())
        })?;
        out.push((v0 << 2) | (v1 >> 4));
        if chunk[2] == b'=' {
            if !is_last || chunk[3] != b'=' {
                return Err(PackageIntegrityError::CorruptedArchive(
                    "Padding Base64 inválido".to_string(),
                ));
            }
            break;
        }
        let v2 = b64_val(chunk[2]).ok_or_else(|| {
            PackageIntegrityError::CorruptedArchive("Caractere Base64 inválido".to_string())
        })?;
        out.push(((v1 & 0x0f) << 4) | (v2 >> 2));
        if chunk[3] == b'=' {
            if !is_last {
                return Err(PackageIntegrityError::CorruptedArchive(
                    "Padding Base64 inválido".to_string(),
                ));
            }
            break;
        }
        let v3 = b64_val(chunk[3]).ok_or_else(|| {
            PackageIntegrityError::CorruptedArchive("Caractere Base64 inválido".to_string())
        })?;
        out.push(((v2 & 0x03) << 6) | v3);
    }
    Ok(out)
}

fn validate_safe_relative_path(rel_path: &str) -> Result<(), PackageIntegrityError> {
    if rel_path.is_empty()
        || rel_path.contains('\0')
        || rel_path.starts_with('/')
        || rel_path.contains('\\')
        || rel_path.contains(':')
        || rel_path.contains("..")
    {
        return Err(PackageIntegrityError::PathTraversalDetected(
            rel_path.to_string(),
        ));
    }
    if rel_path
        .split('/')
        .any(|seg| seg.is_empty() || seg == "." || seg == "..")
    {
        return Err(PackageIntegrityError::PathTraversalDetected(
            rel_path.to_string(),
        ));
    }
    Ok(())
}

impl SforgePackage {
    pub fn new(preset: SonixPresetDoc) -> Self {
        Self {
            preset,
            extra_presets: Vec::new(),
            irs: HashMap::new(),
        }
    }

    pub fn new_multi(mut presets: Vec<SonixPresetDoc>) -> Result<Self, PackageIntegrityError> {
        if presets.is_empty() {
            return Err(PackageIntegrityError::CorruptedArchive(
                "Pacote .sforgepkg requer ao menos um preset".to_string(),
            ));
        }
        let primary = presets.remove(0);
        Ok(Self {
            preset: primary,
            extra_presets: presets,
            irs: HashMap::new(),
        })
    }

    pub fn add_preset(&mut self, preset: SonixPresetDoc) {
        self.extra_presets.push(preset);
    }

    pub fn all_presets(&self) -> Vec<&SonixPresetDoc> {
        let mut all = Vec::with_capacity(1 + self.extra_presets.len());
        all.push(&self.preset);
        for extra in &self.extra_presets {
            all.push(extra);
        }
        all
    }

    pub fn list_irs(&self) -> &HashMap<String, Vec<u8>> {
        &self.irs
    }

    pub fn add_ir(&mut self, relative_path: &str, bytes: Vec<u8>) -> Result<(), PackageIntegrityError> {
        validate_safe_relative_path(relative_path)?;
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

        for (idx, extra) in self.extra_presets.iter().enumerate() {
            let extra_json = extra.to_json_str().unwrap_or_default();
            manifest.insert(
                format!("presets/extra_{idx}.json"),
                Self::hash_bytes(extra_json.as_bytes()),
            );
        }

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
        let mut extra_presets_json = Vec::with_capacity(self.extra_presets.len());
        for extra in &self.extra_presets {
            extra_presets_json.push(
                extra
                    .to_json_str()
                    .map_err(|e| PackageIntegrityError::CorruptedArchive(e.to_string()))?,
            );
        }
        let mut files_b64 = HashMap::with_capacity(self.irs.len());
        for (path, bytes) in &self.irs {
            files_b64.insert(path.clone(), base64_encode(bytes));
        }

        let payload = BundlePayloadV2 {
            version: 2,
            manifest,
            preset_json,
            extra_presets_json,
            files_b64,
        };

        serde_json::to_vec(&payload)
            .map_err(|e| PackageIntegrityError::CorruptedArchive(e.to_string()))
    }

    pub fn import_bundle(bytes: &[u8]) -> Result<Self, PackageIntegrityError> {
        if bytes.len() > MAX_PACKAGE_BYTES {
            return Err(PackageIntegrityError::LimitExceeded(format!(
                "Tamanho do pacote ({} bytes) excede limite máximo de {} bytes",
                bytes.len(),
                MAX_PACKAGE_BYTES
            )));
        }

        let envelope: BundleEnvelope = serde_json::from_slice(bytes)
            .map_err(|e| PackageIntegrityError::CorruptedArchive(e.to_string()))?;

        // 1. Verify preset.json hash (mandatory in manifest)
        let actual_preset_hash = Self::hash_bytes(envelope.preset_json.as_bytes());
        let expected_preset_hash = envelope.manifest.get("preset.json").ok_or_else(|| {
            PackageIntegrityError::CorruptedArchive("preset.json ausente no manifesto".to_string())
        })?;
        if &actual_preset_hash != expected_preset_hash {
            return Err(PackageIntegrityError::ChecksumMismatch {
                file: "preset.json".to_string(),
                expected: expected_preset_hash.clone(),
                actual: actual_preset_hash,
            });
        }

        let preset = SonixPresetDoc::from_json_str(&envelope.preset_json)
            .map_err(|e| PackageIntegrityError::CorruptedArchive(e.to_string()))?;

        let mut extra_presets = Vec::with_capacity(envelope.extra_presets_json.len());
        for (idx, extra_json) in envelope.extra_presets_json.iter().enumerate() {
            let key = format!("presets/extra_{idx}.json");
            let actual_extra_hash = Self::hash_bytes(extra_json.as_bytes());
            let expected_extra_hash = envelope.manifest.get(&key).ok_or_else(|| {
                PackageIntegrityError::CorruptedArchive(format!(
                    "Entrada '{}' ausente no manifesto",
                    key
                ))
            })?;
            if &actual_extra_hash != expected_extra_hash {
                return Err(PackageIntegrityError::ChecksumMismatch {
                    file: key,
                    expected: expected_extra_hash.clone(),
                    actual: actual_extra_hash,
                });
            }
            let extra_doc = SonixPresetDoc::from_json_str(extra_json)
                .map_err(|e| PackageIntegrityError::CorruptedArchive(e.to_string()))?;
            extra_presets.push(extra_doc);
        }

        let raw_files: HashMap<String, Vec<u8>> = match envelope.version {
            Some(2) => {
                let b64_map = envelope.files_b64.ok_or_else(|| {
                    PackageIntegrityError::CorruptedArchive(
                        "Pacote v2 sem campo files_b64".to_string(),
                    )
                })?;
                if b64_map.len() > MAX_PACKAGE_RESOURCES {
                    return Err(PackageIntegrityError::LimitExceeded(format!(
                        "Quantidade de recursos ({}) excede limite máximo de {}",
                        b64_map.len(),
                        MAX_PACKAGE_RESOURCES
                    )));
                }
                let mut decoded = HashMap::with_capacity(b64_map.len());
                for (rel_path, encoded) in b64_map {
                    let bytes = base64_decode(&encoded)?;
                    if bytes.len() > MAX_RESOURCE_BYTES {
                        return Err(PackageIntegrityError::LimitExceeded(format!(
                            "Recurso '{}' ({} bytes) excede limite máximo de {} bytes",
                            rel_path,
                            bytes.len(),
                            MAX_RESOURCE_BYTES
                        )));
                    }
                    decoded.insert(rel_path, bytes);
                }
                decoded
            }
            None | Some(1) => {
                if let Some(b64_map) = envelope.files_b64 {
                    if b64_map.len() > MAX_PACKAGE_RESOURCES {
                        return Err(PackageIntegrityError::LimitExceeded(format!(
                            "Quantidade de recursos ({}) excede limite máximo de {}",
                            b64_map.len(),
                            MAX_PACKAGE_RESOURCES
                        )));
                    }
                    let mut decoded = HashMap::with_capacity(b64_map.len());
                    for (rel_path, encoded) in b64_map {
                        let bytes = base64_decode(&encoded)?;
                        if bytes.len() > MAX_RESOURCE_BYTES {
                            return Err(PackageIntegrityError::LimitExceeded(format!(
                                "Recurso '{}' ({} bytes) excede limite máximo de {} bytes",
                                rel_path,
                                bytes.len(),
                                MAX_RESOURCE_BYTES
                            )));
                        }
                        decoded.insert(rel_path, bytes);
                    }
                    decoded
                } else {
                    let v1_map = envelope.files.ok_or_else(|| {
                        PackageIntegrityError::CorruptedArchive(
                            "Pacote sem campo files ou files_b64".to_string(),
                        )
                    })?;
                    if v1_map.len() > MAX_PACKAGE_RESOURCES {
                        return Err(PackageIntegrityError::LimitExceeded(format!(
                            "Quantidade de recursos ({}) excede limite máximo de {}",
                            v1_map.len(),
                            MAX_PACKAGE_RESOURCES
                        )));
                    }
                    for (rel_path, file_bytes) in &v1_map {
                        if file_bytes.len() > MAX_RESOURCE_BYTES {
                            return Err(PackageIntegrityError::LimitExceeded(format!(
                                "Recurso '{}' ({} bytes) excede limite máximo de {} bytes",
                                rel_path,
                                file_bytes.len(),
                                MAX_RESOURCE_BYTES
                            )));
                        }
                    }
                    v1_map
                }
            }
            Some(other) => {
                return Err(PackageIntegrityError::CorruptedArchive(format!(
                    "Versão de pacote .sforgepkg não suportada: {other}"
                )));
            }
        };

        // 2. Verify all files
        let mut irs = HashMap::new();
        for (rel_path, file_bytes) in raw_files {
            validate_safe_relative_path(&rel_path)?;

            let actual_hash = Self::hash_bytes(&file_bytes);
            let expected_hash = envelope.manifest.get(&rel_path).ok_or_else(|| {
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

        // 3. Verify every manifest entry (except preset.json / presets/extra_*) has a corresponding payload file
        for manifest_key in envelope.manifest.keys() {
            let is_extra_preset = manifest_key
                .strip_prefix("presets/extra_")
                .and_then(|rest| rest.strip_suffix(".json"))
                .and_then(|idx_str| idx_str.parse::<usize>().ok())
                .map_or(false, |idx| idx < extra_presets.len());
            if manifest_key != "preset.json" && !is_extra_preset && !irs.contains_key(manifest_key) {
                return Err(PackageIntegrityError::CorruptedArchive(format!(
                    "Entrada '{}' listada no manifesto sem arquivo correspondente",
                    manifest_key
                )));
            }
        }

        Ok(Self {
            preset,
            extra_presets,
            irs,
        })
    }
}
