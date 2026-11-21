//! Testes de auditoria AUD-19 — Formato .sforgepkg v2 (base64), compatibilidade v1 e limites de importação.

use serde_json::json;
use sha2::{Digest, Sha256};
use sonix_core::persistence::{
    BlockDto, GridDto, MAX_PACKAGE_BYTES, MAX_PACKAGE_RESOURCES, MAX_RESOURCE_BYTES,
    PackageIntegrityError, PresetMetadata, SceneDto, SforgePackage, SonixPresetDoc,
};
use std::collections::HashMap;

fn make_preset(id: &str, name: &str, gain: f32) -> SonixPresetDoc {
    let mut params = SonixPresetDoc::default_boot_params();
    params.insert("amp_1.gain".to_string(), json!(gain));
    SonixPresetDoc {
        schema: "https://sonixpro.local/schemas/sonixpreset-v1.json".to_string(),
        schema_version: "1.1.0".to_string(),
        metadata: PresetMetadata {
            id: id.to_string(),
            name: name.to_string(),
            author: "Audit".to_string(),
            category: "Custom".to_string(),
            tempo_bpm: 120.0,
            created_at: "2026-11-21T00:00:00Z".to_string(),
            footswitch_mode: "scenes".to_string(),
        },
        active_scene_index: 0,
        scenes: vec![SceneDto {
            index: 0,
            id: "A".to_string(),
            name: "Main".to_string(),
            color: "#ff9f4d".to_string(),
        }],
        grid: GridDto {
            rows: 2,
            cols: 11,
            blocks: vec![BlockDto {
                instance_id: "amp_1".to_string(),
                block_type: "amp".to_string(),
                row: 0,
                col: 5,
                oversampling: "4x".to_string(),
                active_channel_by_scene: [0; 8],
                bypass_by_scene: [false; 8],
            }],
        },
        modifiers: vec![],
        params: Some(params),
        channels: Some(HashMap::new()),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

#[test]
fn aud_19_ida_e_volta() {
    let p1 = make_preset("aud19_a", "Preset A", 4.5);
    let p2 = make_preset("aud19_b", "Preset B", 8.5);
    let mut pkg = SforgePackage::new_multi(vec![p1.clone(), p2.clone()]).expect("new_multi deve aceitar 2 presets");

    let ir1: Vec<u8> = (0..1025).map(|i| ((i * 37) % 256) as u8).collect();
    let ir2: Vec<u8> = vec![0x52, 0x49, 0x46, 0x46, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE];
    pkg.add_ir("irs/custom_a.wav", ir1.clone()).unwrap();
    pkg.add_ir("irs/custom_b.wav", ir2.clone()).unwrap();

    let exported = pkg.export_bundle().expect("export_bundle v2 deve ter sucesso");
    let raw_json: serde_json::Value = serde_json::from_slice(&exported).expect("pacote é JSON v2");
    assert_eq!(raw_json["version"], 2, "export_bundle deve emitir formato v2");
    assert!(
        raw_json.get("files_b64").and_then(|v| v.as_object()).is_some(),
        "v2 deve serializar recursos em files_b64 (base64)"
    );
    assert!(
        raw_json.get("files").is_none(),
        "v2 não deve serializar array JSON de inteiros"
    );

    let imported = SforgePackage::import_bundle(&exported).expect("import_bundle v2 deve ler ida e volta");
    let presets = imported.all_presets();
    assert_eq!(presets.len(), 2);
    assert_eq!(presets[0], &p1);
    assert_eq!(presets[1], &p2);
    assert_eq!(imported.get_ir("irs/custom_a.wav").unwrap(), &ir1);
    assert_eq!(imported.get_ir("irs/custom_b.wav").unwrap(), &ir2);
}

#[test]
fn aud_19_v1_continua_lendo() {
    let p1 = make_preset("aud19_v1", "Legacy V1", 6.0);
    let preset_json = p1.to_json_str().unwrap();
    let ir_bytes = vec![10u8, 20, 30, 40, 50, 60, 70];
    let payload = json!({
        "manifest": {
            "preset.json": sha256_hex(preset_json.as_bytes()),
            "irs/legacy_v1.wav": sha256_hex(&ir_bytes),
        },
        "preset_json": preset_json,
        "files": {
            "irs/legacy_v1.wav": ir_bytes,
        }
    });
    let raw = serde_json::to_vec(&payload).unwrap();
    let imported = SforgePackage::import_bundle(&raw).expect("leitor deve continuar aceitando formato v1");
    assert_eq!(imported.preset, p1);
    assert_eq!(imported.get_ir("irs/legacy_v1.wav").unwrap(), &vec![10u8, 20, 30, 40, 50, 60, 70]);
}

#[test]
fn aud_19_limites() {
    assert_eq!(MAX_PACKAGE_BYTES, 64 * 1024 * 1024);
    assert_eq!(MAX_RESOURCE_BYTES, 20 * 1024 * 1024);
    assert_eq!(MAX_PACKAGE_RESOURCES, 256);

    // 1) Arquivo de 65 MiB -> erro
    let oversize_archive = vec![b' '; 65 * 1024 * 1024];
    let err_archive = SforgePackage::import_bundle(&oversize_archive);
    assert!(
        matches!(err_archive, Err(PackageIntegrityError::LimitExceeded(_))),
        "arquivo 65 MiB deve retornar LimitExceeded, veio {:?}",
        err_archive
    );

    // 2) Recurso individual de 21 MiB -> erro
    let p = make_preset("aud19_lim", "Limit", 5.0);
    let mut pkg_big_ir = SforgePackage::new(p.clone());
    let big_ir = vec![0x5Au8; 21 * 1024 * 1024];
    pkg_big_ir.add_ir("irs/big_21mib.wav", big_ir).unwrap();
    let exported_big = pkg_big_ir.export_bundle().unwrap();
    assert!(exported_big.len() < MAX_PACKAGE_BYTES);
    let err_res = SforgePackage::import_bundle(&exported_big);
    assert!(
        matches!(err_res, Err(PackageIntegrityError::LimitExceeded(_))),
        "recurso 21 MiB deve retornar LimitExceeded, veio {:?}",
        err_res
    );

    // 3) Mais de 256 recursos -> erro
    let mut pkg_many = SforgePackage::new(p);
    for i in 0..257 {
        pkg_many.add_ir(&format!("irs/r_{i:03}.wav"), vec![i as u8]).unwrap();
    }
    let exported_many = pkg_many.export_bundle().unwrap();
    let err_count = SforgePackage::import_bundle(&exported_many);
    assert!(
        matches!(err_count, Err(PackageIntegrityError::LimitExceeded(_))),
        "257 recursos devem retornar LimitExceeded, veio {:?}",
        err_count
    );
}
