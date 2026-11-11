//! crates/sonix-core/tests/contract_010_persistence_offline.rs
//! Testes de Contrato Automatizados para Spec 010: Persistência Offline, Schema sonixpreset-v1.json e Bundles .sforgepkg
//! Cobre Critérios de Aceite CA-1 a CA-8

use sonix_core::persistence::{
    BlockDto, GridDto, PackageIntegrityError, PresetMetadata, PresetSchemaValidationError,
    SceneDto, SforgePackage, SonixPresetDoc,
};

#[test]
fn test_ca_01_json_preset_schema_v1_top_level_fields_and_grid_4x12() {
    let preset = create_valid_test_preset();
    let json_str = preset.to_json_str().unwrap();

    assert!(json_str.contains("\"$schema\": \"https://sonixpro.local/schemas/sonixpreset-v1.json\""));
    assert!(json_str.contains("\"schemaVersion\": \"1.0.0\""));
    assert!(json_str.contains("\"rows\": 4"));
    assert!(json_str.contains("\"cols\": 12"));
    assert!(json_str.contains("\"activeSceneIndex\": 0"));
}

#[test]
fn test_ca_02_lossless_round_trip_export_and_import() {
    let original = create_valid_test_preset();
    let json_str = original.to_json_str().unwrap();

    let restored = SonixPresetDoc::from_json_str(&json_str).unwrap();

    assert_eq!(original.metadata.id, restored.metadata.id);
    assert_eq!(original.metadata.name, restored.metadata.name);
    assert_eq!(original.metadata.tempo_bpm, restored.metadata.tempo_bpm);
    assert_eq!(original.scenes.len(), restored.scenes.len());
    assert_eq!(original.grid.blocks.len(), restored.grid.blocks.len());
    assert_eq!(original.modifiers.len(), restored.modifiers.len());
}

#[test]
fn test_ca_03_sforgepkg_export_manifest_sha256_and_relative_ir_paths() {
    let preset = create_valid_test_preset();
    let mut pkg = SforgePackage::new(preset);

    let mock_wav_bytes = vec![0x52, 0x49, 0x46, 0x46, 0x24, 0x00, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45];
    pkg.add_ir("irs/cab_greenback.wav", mock_wav_bytes.clone()).unwrap();

    let manifest = pkg.generate_manifest();
    assert!(manifest.contains_key("preset.json"));
    assert!(manifest.contains_key("irs/cab_greenback.wav"));

    // SHA-256 string must be 64 hexadecimal characters
    let wav_hash = manifest.get("irs/cab_greenback.wav").unwrap();
    assert_eq!(wav_hash.len(), 64);
}

#[test]
fn test_ca_04_sforgepkg_import_verifies_sha256_and_loads_completely() {
    let preset = create_valid_test_preset();
    let mut original_pkg = SforgePackage::new(preset);
    let mock_wav = vec![1, 2, 3, 4, 5, 6, 7, 8];
    original_pkg.add_ir("irs/test_ir.wav", mock_wav.clone()).unwrap();

    let exported_bytes = original_pkg.export_bundle().unwrap();

    // Import bundle
    let imported_pkg = SforgePackage::import_bundle(&exported_bytes).unwrap();
    assert_eq!(imported_pkg.preset.metadata.name, "Master Arena Rig");
    assert_eq!(imported_pkg.get_ir("irs/test_ir.wav").unwrap(), &mock_wav);
}

#[test]
fn test_ca_05_zero_external_cdn_requests_in_design_system_html() {
    let html_content = std::fs::read_to_string("design-system/index.html")
        .or_else(|_| std::fs::read_to_string("../../design-system/index.html"))
        .expect("Deve ler index.html localmente");

    // Must have zero references to external Google Fonts or remote CDNs
    assert!(
        !html_content.contains("fonts.googleapis.com"),
        "index.html não pode carregar fonts.googleapis.com (deve ser 100% offline)"
    );
    assert!(
        !html_content.contains("fonts.gstatic.com"),
        "index.html não pode carregar fonts.gstatic.com"
    );
    assert!(
        !html_content.contains("cdnjs.cloudflare.com"),
        "index.html não pode carregar CDNs externos"
    );
}

#[test]
fn test_ca_06_malformed_json_or_missing_schema_rejected() {
    let malformed_json = "{ invalid json ";
    assert!(matches!(
        SonixPresetDoc::from_json_str(malformed_json),
        Err(PresetSchemaValidationError::MalformedJson(_))
    ));

    let incompatible_version_json = r#"{
        "$schema": "https://sonixpro.local/schemas/sonixpreset-v1.json",
        "schemaVersion": "9.9.9",
        "metadata": {
            "id": "123", "name": "test", "author": "dev", "category": "rock",
            "tempoBpm": 120.0, "createdAt": "2026-11-08T00:00:00Z", "footswitchMode": "HYBRID_STOMP_SCENE"
        },
        "activeSceneIndex": 0,
        "scenes": [],
        "grid": { "rows": 4, "cols": 12, "blocks": [] },
        "modifiers": []
    }"#;

    assert!(matches!(
        SonixPresetDoc::from_json_str(incompatible_version_json),
        Err(PresetSchemaValidationError::IncompatibleSchemaVersion(_))
    ));
}

#[test]
fn test_ca_07_tampered_package_or_path_traversal_rejected_with_package_integrity_error() {
    let preset = create_valid_test_preset();
    let mut pkg = SforgePackage::new(preset);
    pkg.add_ir("irs/clean.wav", vec![10, 20, 30]).unwrap();
    let mut bundle_bytes = pkg.export_bundle().unwrap();

    // 1. Tamper with bytes inside bundle
    let last_idx = bundle_bytes.len() - 1;
    bundle_bytes[last_idx] ^= 0xFF; // Invert bit

    let import_res = SforgePackage::import_bundle(&bundle_bytes);
    assert!(matches!(import_res, Err(PackageIntegrityError::ChecksumMismatch { .. }) | Err(PackageIntegrityError::CorruptedArchive(_))));

    // 2. Reject path traversal
    let mut path_traversal_pkg = SforgePackage::new(create_valid_test_preset());
    let err = path_traversal_pkg.add_ir("../../../etc/passwd", vec![1, 2, 3]);
    assert!(matches!(err, Err(PackageIntegrityError::PathTraversalDetected(_))));
}

#[test]
fn test_ca_08_non_regression_export_is_non_blocking() {
    let preset = create_valid_test_preset();
    let start = std::time::Instant::now();
    let _json = preset.to_json_str().unwrap();
    let duration = start.elapsed();

    // Export should serialize in under 10 milliseconds
    assert!(
        duration.as_millis() < 10,
        "Export serialization must complete in under 10ms (took {:?})",
        duration
    );
}

fn create_valid_test_preset() -> SonixPresetDoc {
    SonixPresetDoc {
        schema: "https://sonixpro.local/schemas/sonixpreset-v1.json".to_string(),
        schema_version: "1.0.0".to_string(),
        metadata: PresetMetadata {
            id: "e4b9b940-bf7d-4e94-811c-c76b5d697b6a".to_string(),
            name: "Master Arena Rig".to_string(),
            author: "The Unforgiven".to_string(),
            category: "Stadium Rock".to_string(),
            tempo_bpm: 128.0,
            created_at: "2026-11-08T18:00:00Z".to_string(),
            footswitch_mode: "HYBRID_STOMP_SCENE".to_string(),
        },
        active_scene_index: 0,
        scenes: (0..8)
            .map(|i| SceneDto {
                index: i,
                id: format!("scene_{}", i),
                name: format!("Scene {}", (b'A' + i as u8) as char),
                color: "#7C4DFF".to_string(),
            })
            .collect(),
        grid: GridDto {
            rows: 4,
            cols: 12,
            blocks: vec![
                BlockDto {
                    instance_id: "amp_1".to_string(),
                    block_type: "AMP".to_string(),
                    row: 0,
                    col: 2,
                    oversampling: "4x".to_string(),
                    active_channel_by_scene: [0; 8],
                    bypass_by_scene: [false; 8],
                },
            ],
        },
        modifiers: vec![],
        params: None,
        channels: None,
    }
}
