//! Testes de contrato da Spec 046 CA-5 e CA-6 — Endurecimento de caminhos e manifesto em .sforgepkg

use serde_json::json;
use sonix_core::persistence::{
    GridDto, PackageIntegrityError, PresetMetadata, SforgePackage, SonixPresetDoc,
};
use sha2::{Digest, Sha256};

fn minimal_preset() -> SonixPresetDoc {
    SonixPresetDoc {
        schema: "https://sonixpro.local/schemas/sonixpreset-v1.json".to_string(),
        schema_version: "1.1.0".to_string(),
        metadata: PresetMetadata {
            id: "test-pkg".to_string(),
            name: "Test Pkg".to_string(),
            author: "Tester".to_string(),
            category: "Custom".to_string(),
            tempo_bpm: 120.0,
            created_at: "2026-11-14T00:00:00Z".to_string(),
            footswitch_mode: "scenes".to_string(),
        },
        active_scene_index: 0,
        scenes: vec![],
        grid: GridDto {
            rows: 4,
            cols: 12,
            blocks: vec![],
        },
        modifiers: vec![],
        params: None,
        channels: None,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[test]
fn test_ca_05_windows_absolute_and_unc_paths_rejected_in_add_ir_and_import() {
    let bad_paths = [
        r"C:\x.wav",
        "C:x.wav",
        r"\\server\x.wav",
        r"\\?\C:\x.wav",
        r"irs\..\evil.wav",
        "/abs/x.wav",
    ];

    let mut pkg = SforgePackage::new(minimal_preset());
    for bad in bad_paths {
        let res = pkg.add_ir(bad, vec![1, 2, 3, 4]);
        assert!(
            matches!(res, Err(PackageIntegrityError::PathTraversalDetected(_))),
            "CA-5: add_ir aceitou caminho perigoso '{}': {:?}",
            bad,
            res
        );
    }

    // Também injeta diretamente num payload JSON para testar import_bundle
    let preset_json = minimal_preset().to_json_str().unwrap();
    for bad in [r"C:\x.wav", "C:x.wav", r"\\server\x.wav", r"\\?\C:\x.wav"] {
        let ir_bytes = vec![1u8, 2, 3, 4];
        let payload = json!({
            "manifest": {
                "preset.json": sha256_hex(preset_json.as_bytes()),
                bad: sha256_hex(&ir_bytes),
            },
            "preset_json": preset_json,
            "files": {
                bad: ir_bytes,
            }
        });
        let raw = serde_json::to_vec(&payload).unwrap();
        let res = SforgePackage::import_bundle(&raw);
        assert!(
            matches!(res, Err(PackageIntegrityError::PathTraversalDetected(_))),
            "CA-5: import_bundle aceitou caminho perigoso '{}': {:?}",
            bad,
            res
        );
    }
}

#[test]
fn test_ca_06_manifest_missing_file_rejected_as_corrupted_archive() {
    let preset_json = minimal_preset().to_json_str().unwrap();
    let payload = json!({
        "manifest": {
            "preset.json": sha256_hex(preset_json.as_bytes()),
            "irs/a.wav": sha256_hex(b"phantom"),
        },
        "preset_json": preset_json,
        "files": {}
    });
    let raw = serde_json::to_vec(&payload).unwrap();
    let res = SforgePackage::import_bundle(&raw);
    assert!(
        matches!(res, Err(PackageIntegrityError::CorruptedArchive(_))),
        "CA-6: manifesto com arquivo ausente em files deveria retornar CorruptedArchive, veio {:?}",
        res
    );
}
