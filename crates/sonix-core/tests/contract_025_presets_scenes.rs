//! Testes de contrato da Spec 025 — Presets, Cenas e Canais de Bloco Reais
//! Cobre CA-3, CA-4, CA-5, CA-8, CA-11, CA-12.

use sonix_core::persistence::{
    PresetMetadata, PresetSchemaValidationError, SceneDto, SforgePackage, SonixPresetDoc,
};
use sonix_core::preset::{BlockChannel, BlockChannelState, BlockNode, Preset};
use std::collections::HashMap;

fn create_base_test_preset_doc(schema_ver: &str) -> SonixPresetDoc {
    SonixPresetDoc {
        schema: "https://sonixpro.local/schemas/sonixpreset-v1.json".to_string(),
        schema_version: schema_ver.to_string(),
        metadata: PresetMetadata {
            id: "test-preset-025".to_string(),
            name: "Test Preset 025".to_string(),
            author: "Tester".to_string(),
            category: "Rock".to_string(),
            tempo_bpm: 120.0,
            created_at: "2026-11-11T00:00:00Z".to_string(),
            footswitch_mode: "scenes".to_string(),
        },
        active_scene_index: 0,
        scenes: (0..8)
            .map(|i| SceneDto {
                index: i,
                id: format!("scene_{}", i),
                name: format!("Scene {}", (b'A' + i as u8) as char),
                color: "#ffc400".to_string(),
            })
            .collect(),
        grid: sonix_core::persistence::GridDto {
            rows: 4,
            cols: 12,
            blocks: vec![],
        },
        modifiers: vec![],
        params: None,
        channels: None,
    }
}

#[test]
fn test_ca_04_schema_1_1_0_export_import_and_1_0_0_migration() {
    // 1. Exporta e reimporta documento schema 1.1.0 com parâmetros completos
    let mut doc_v1_1 = create_base_test_preset_doc("1.1.0");
    let mut params = HashMap::new();
    params.insert("amp_1.gain".to_string(), serde_json::json!(7.5));
    params.insert("drive.enabled".to_string(), serde_json::json!(true));
    params.insert("scene.active".to_string(), serde_json::json!("A"));
    doc_v1_1.params = Some(params);

    let json_v1_1 = doc_v1_1.to_json_str().expect("Serialização 1.1.0 deve ter sucesso");
    assert!(json_v1_1.contains("\"schemaVersion\": \"1.1.0\""));
    assert!(json_v1_1.contains("\"amp_1.gain\": 7.5"));

    let restored_v1_1 = SonixPresetDoc::from_json_str(&json_v1_1)
        .expect("Importação de documento 1.1.0 deve ter sucesso");
    assert_eq!(restored_v1_1.schema_version, "1.1.0");
    assert_eq!(
        restored_v1_1.params.as_ref().unwrap().get("amp_1.gain").unwrap(),
        &serde_json::json!(7.5)
    );

    // 2. Migração de documento schema 1.0.0 (sem params) -> preenche parâmetros com boot defaults
    let doc_v1_0 = create_base_test_preset_doc("1.0.0");
    let json_v1_0 = doc_v1_0.to_json_str().expect("Serialização 1.0.0");
    assert!(json_v1_0.contains("\"schemaVersion\": \"1.0.0\""));

    let migrated = SonixPresetDoc::from_json_str(&json_v1_0)
        .expect("Migração de 1.0.0 deve ter sucesso");
    assert!(
        migrated.params.is_some(),
        "Documento 1.0.0 migrado deve receber parâmetros de boot"
    );
    let p = migrated.params.unwrap();
    assert!(p.contains_key("amp_1.gain"), "Deve conter amp_1.gain default");
    assert!(p.contains_key("drive.enabled"), "Deve conter drive.enabled default");

    // 3. SchemaVersion desconhecido retorna IncompatibleSchemaVersion
    let bad_version_json = json_v1_1.replace("\"1.1.0\"", "\"2.0.0\"");
    let err_ver = SonixPresetDoc::from_json_str(&bad_version_json);
    assert!(matches!(
        err_ver,
        Err(PresetSchemaValidationError::IncompatibleSchemaVersion(_))
    ));

    // 4. JSON malformado retorna MalformedJson
    let bad_json = "{ invalid json ";
    let err_malformed = SonixPresetDoc::from_json_str(bad_json);
    assert!(matches!(
        err_malformed,
        Err(PresetSchemaValidationError::MalformedJson(_))
    ));
}

#[test]
fn test_ca_05_sforgepkg_export_and_import_with_ir_hash() {
    let mut doc = create_base_test_preset_doc("1.1.0");
    let mut params = HashMap::new();
    params.insert("cab_1.ir_enabled".to_string(), serde_json::json!(true));
    params.insert(
        "cab_1.ir_hash".to_string(),
        serde_json::json!("a1b2c3d4e5f6"),
    );
    doc.params = Some(params);

    let mut pkg = SforgePackage::new(doc);
    let dummy_wav = vec![0u8; 1024];
    pkg.add_ir("irs/cab_custom.wav", dummy_wav.clone())
        .expect("Adição de IR");

    let bundle_bytes = pkg.export_bundle().expect("Exportação de pacote");
    assert!(!bundle_bytes.is_empty());

    let imported_pkg = SforgePackage::import_bundle(&bundle_bytes)
        .expect("Importação de pacote com IR");
    assert_eq!(
        imported_pkg.preset.params.as_ref().unwrap().get("cab_1.ir_enabled").unwrap(),
        &serde_json::json!(true)
    );
    let ir_file = imported_pkg.get_ir("irs/cab_custom.wav");
    assert!(ir_file.is_some(), "IR deve estar presente no pacote importado");
    assert_eq!(ir_file.unwrap().len(), 1024);
}

#[test]
fn test_ca_08_block_channels_a_b_c_d_independence_and_scene_association() {
    let mut amp_block = BlockNode::new("amp_1", "amp");

    // Configura canal A com gain = 3.0
    let mut ch_a = BlockChannelState::new("jcm800_2203");
    ch_a.params.insert("gain".to_string(), 3.0);
    amp_block.set_channel_state(BlockChannel::A, ch_a);

    // Configura canal D com gain = 9.0
    let mut ch_d = BlockChannelState::new("jcm800_2203");
    ch_d.params.insert("gain".to_string(), 9.0);
    amp_block.set_channel_state(BlockChannel::D, ch_d);

    // Associa Canal A à Cena 0 (Scene A) e Canal D à Cena 3 (Scene D)
    amp_block.set_active_channel_for_scene(0, BlockChannel::A);
    amp_block.set_active_channel_for_scene(3, BlockChannel::D);

    assert_eq!(amp_block.active_channel_for_scene(0), BlockChannel::A);
    assert_eq!(amp_block.channel_state(BlockChannel::A).params.get("gain"), Some(&3.0));

    assert_eq!(amp_block.active_channel_for_scene(3), BlockChannel::D);
    assert_eq!(amp_block.channel_state(BlockChannel::D).params.get("gain"), Some(&9.0));

    // Voltar à Cena 0 restaura Canal A com gain = 3.0
    let current_ch_scene_0 = amp_block.active_channel_for_scene(0);
    assert_eq!(current_ch_scene_0, BlockChannel::A);
    assert_eq!(
        amp_block.channel_state(current_ch_scene_0).params.get("gain"),
        Some(&3.0)
    );
}

#[test]
fn test_ca_11_scene_and_channel_validation_rules() {
    let preset = Preset::new("preset_1", "Preset 1", 120.0);
    // 8 cenas válidas (0..7)
    for i in 0..8 {
        assert!(preset.scene(i).is_ok());
    }
    // Cena 8 é inválida -> Erro
    assert!(preset.scene(8).is_err());

    // Block channels válidos: A, B, C, D
    assert_eq!(BlockChannel::A.as_char(), 'A');
    assert_eq!(BlockChannel::B.as_char(), 'B');
    assert_eq!(BlockChannel::C.as_char(), 'C');
    assert_eq!(BlockChannel::D.as_char(), 'D');
}
