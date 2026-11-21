//! Testes de auditoria 2026-11-20 — Parâmetros por modelo na API e presets (AUD-08)

use std::f32::consts::PI;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{LockFreeParamQueue, RealtimeDspChain, TelemetryBuffer};
use sonix_desktop::calibration::CalibrationStore;

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(prefix: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{}_{}_{}", prefix, std::process::id(), nanos));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn make_ctx_with_data_dir(data_dir: PathBuf) -> (ApiContext, Arc<Mutex<RealtimeDspChain>>) {
    let mut chain = RealtimeDspChain::new(48_000.0);
    chain.prepare(48_000.0, 256);
    let dsp = Arc::new(Mutex::new(chain));
    let mut ctx = ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(512)),
        Some(dsp.clone()),
        Some(Arc::new(LockFreeParamQueue::new())),
        None,
        Some(CalibrationStore::new()),
    );
    ctx.set_data_dir(data_dir);
    (ctx, dsp)
}

fn post_params(ctx: &ApiContext, body: &str) -> (u16, String) {
    let (status, _, bytes) = dispatch(ctx, "/api/params", "POST", body.as_bytes());
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

#[test]
fn aud_08_dispatch_rejeita_controle_alheio_ao_modelo() {
    let dir = TempDirGuard::new("sonix_aud08_dispatch");
    let (ctx, _) = make_ctx_with_data_dir(dir.path.clone());

    // Twin Reverb (AB763): sem presence, sem master
    let (st, resp) = post_params(&ctx, r#"{"amp_1.model":"twin_ab763"}"#);
    assert_eq!(st, 200, "troca para twin_ab763 falhou: {}", resp);

    let (st_pres, resp_pres) = post_params(&ctx, r#"{"amp_1.presence":7.0}"#);
    assert_eq!(
        st_pres, 400,
        "twin_ab763 deve rejeitar amp_1.presence com 400, retornou {} ({})",
        st_pres, resp_pres
    );
    assert!(
        resp_pres.contains("E_VALIDATION")
            && resp_pres.contains("amp_1.presence")
            && resp_pres.contains("twin_ab763"),
        "resposta 400 deve citar E_VALIDATION, amp_1.presence e twin_ab763: {}",
        resp_pres
    );

    let (st_mast, resp_mast) = post_params(&ctx, r#"{"amp_1.master":7.0}"#);
    assert_eq!(
        st_mast, 400,
        "twin_ab763 deve rejeitar amp_1.master com 400, retornou {} ({})",
        st_mast, resp_mast
    );
    assert!(
        resp_mast.contains("E_VALIDATION")
            && resp_mast.contains("amp_1.master")
            && resp_mast.contains("twin_ab763"),
        "resposta 400 deve citar E_VALIDATION, amp_1.master e twin_ab763: {}",
        resp_mast
    );

    // Plexi 1959: sem master, mas aceita presence
    assert_eq!(post_params(&ctx, r#"{"amp_1.model":"plexi_1959"}"#).0, 200);
    assert_eq!(
        post_params(&ctx, r#"{"amp_1.master":3.0}"#).0,
        400,
        "plexi_1959 deve rejeitar amp_1.master com 400"
    );
    assert_eq!(
        post_params(&ctx, r#"{"amp_1.presence":6.0}"#).0,
        200,
        "plexi_1959 deve aceitar amp_1.presence"
    );

    // AC30 Top Boost: sem presence, sem master, sem middle
    assert_eq!(post_params(&ctx, r#"{"amp_1.model":"ac30_tb"}"#).0, 200);
    for key in ["amp_1.presence", "amp_1.master", "amp_1.middle"] {
        let body = format!(r#"{{"{}":6.0}}"#, key);
        let (code, body_resp) = post_params(&ctx, &body);
        assert_eq!(
            code, 400,
            "ac30_tb deve rejeitar {} com 400, retornou {} ({})",
            key, code, body_resp
        );
    }

    // JCM800 & Dual Rectifier aceitam presence e master
    for model in ["jcm800_2203", "dual_rect"] {
        let sw = format!(r#"{{"amp_1.model":"{}"}}"#, model);
        assert_eq!(post_params(&ctx, &sw).0, 200);
        assert_eq!(
            post_params(&ctx, r#"{"amp_1.presence":6.0,"amp_1.master":6.5}"#).0,
            200,
            "{} deve aceitar presence e master",
            model
        );
    }

    // overdrive e depth removidos do contrato (não estão em GET /api/state)
    let (_, _, st_bytes) = dispatch(&ctx, "/api/state", "GET", b"");
    let st_json: serde_json::Value = serde_json::from_slice(&st_bytes).unwrap();
    assert!(
        st_json.get("amp_1.overdrive").is_none(),
        "GET /api/state não pode expor amp_1.overdrive"
    );
    assert!(
        st_json.get("amp_1.depth").is_none(),
        "GET /api/state não pode expor amp_1.depth"
    );
}

#[test]
fn aud_08_preset_descarta_chave_inexistente() {
    let dir_dirty = TempDirGuard::new("sonix_aud08_dirty");
    let dir_clean = TempDirGuard::new("sonix_aud08_clean");
    let (ctx_dirty, dsp_dirty) = make_ctx_with_data_dir(dir_dirty.path.clone());
    let (ctx_clean, dsp_clean) = make_ctx_with_data_dir(dir_clean.path.clone());

    let preset_dirty = serde_json::json!({
        "$schema": "https://sonixpro.local/schemas/sonixpreset-v1.json",
        "schemaVersion": "1.1.0",
        "metadata": {
            "id": "twin_dirty_aud08",
            "name": "Twin Legacy Dirty",
            "author": "Audit",
            "category": "Clean",
            "tempoBpm": 120.0,
            "createdAt": "2026-11-20T00:00:00Z",
            "footswitchMode": "scenes"
        },
        "activeSceneIndex": 0,
        "scenes": [
            { "index": 0, "id": "scene_0", "name": "Scene A", "color": "#ffc400" }
        ],
        "grid": { "rows": 4, "cols": 12, "blocks": [] },
        "modifiers": [],
        "params": {
            "gate.enabled": false,
            "drive.enabled": false,
            "cab_1.enabled": false,
            "amp_1.model": "twin_ab763",
            "amp_1.gain": 4.0,
            "amp_1.bass": 5.0,
            "amp_1.middle": 5.0,
            "amp_1.treble": 5.5,
            "amp_1.presence": 9.5,
            "amp_1.master": 9.0,
            "amp_1.overdrive": 8.0,
            "amp_1.depth": 8.0
        }
    });

    let preset_clean = serde_json::json!({
        "$schema": "https://sonixpro.local/schemas/sonixpreset-v1.json",
        "schemaVersion": "1.1.0",
        "metadata": {
            "id": "twin_clean_aud08",
            "name": "Twin Clean Strict",
            "author": "Audit",
            "category": "Clean",
            "tempoBpm": 120.0,
            "createdAt": "2026-11-20T00:00:00Z",
            "footswitchMode": "scenes"
        },
        "activeSceneIndex": 0,
        "scenes": [
            { "index": 0, "id": "scene_0", "name": "Scene A", "color": "#ffc400" }
        ],
        "grid": { "rows": 4, "cols": 12, "blocks": [] },
        "modifiers": [],
        "params": {
            "gate.enabled": false,
            "drive.enabled": false,
            "cab_1.enabled": false,
            "amp_1.model": "twin_ab763",
            "amp_1.gain": 4.0,
            "amp_1.bass": 5.0,
            "amp_1.middle": 5.0,
            "amp_1.treble": 5.5
        }
    });

    let (st_imp_dirty, _, imp_dirty_bytes) = dispatch(
        &ctx_dirty,
        "/api/presets/import",
        "POST",
        preset_dirty.to_string().as_bytes(),
    );
    assert_eq!(st_imp_dirty, 200);
    let imp_dirty_json: serde_json::Value = serde_json::from_slice(&imp_dirty_bytes).unwrap();
    let ignored = imp_dirty_json["ignored_keys"]
        .as_array()
        .expect("resposta de /api/presets/import deve incluir array ignored_keys");
    for expected in [
        "amp_1.depth",
        "amp_1.master",
        "amp_1.overdrive",
        "amp_1.presence",
    ] {
        assert!(
            ignored.iter().any(|v| v == expected),
            "ignored_keys deveria conter '{}' (recebido {:?})",
            expected,
            ignored
        );
    }

    let (st_imp_clean, _, imp_clean_bytes) = dispatch(
        &ctx_clean,
        "/api/presets/import",
        "POST",
        preset_clean.to_string().as_bytes(),
    );
    assert_eq!(st_imp_clean, 200);
    let imp_clean_json: serde_json::Value = serde_json::from_slice(&imp_clean_bytes).unwrap();
    assert_eq!(
        imp_clean_json["ignored_keys"].as_array().map(|a| a.len()),
        Some(0)
    );

    // Áudio processado nas duas cadeias deve ser idêntico (<= -120 dB)
    let input: Vec<f32> = (0..4800)
        .map(|i| {
            let t = i as f32 / 48_000.0;
            0.25 * (2.0 * PI * 1000.0 * t).sin() + 0.15 * (2.0 * PI * 4000.0 * t).sin()
        })
        .collect();
    let mut out_dirty = vec![0.0f32; input.len()];
    let mut out_clean = vec![0.0f32; input.len()];
    {
        let mut d = dsp_dirty.lock().unwrap();
        d.drain_param_queue();
        d.process(&input, &mut out_dirty);
    }
    {
        let mut c = dsp_clean.lock().unwrap();
        c.drain_param_queue();
        c.process(&input, &mut out_clean);
    }

    let mut sum_sq_ref = 0.0f64;
    let mut sum_sq_diff = 0.0f64;
    for (&a, &b) in out_clean[2400..].iter().zip(out_dirty[2400..].iter()) {
        sum_sq_ref += (a as f64) * (a as f64);
        let df = (b - a) as f64;
        sum_sq_diff += df * df;
    }
    let rms_ref = (sum_sq_ref / 2400.0).sqrt();
    let rms_diff = (sum_sq_diff / 2400.0).sqrt();
    let diff_db = if rms_diff == 0.0 {
        -240.0
    } else {
        20.0 * (rms_diff / rms_ref.max(1e-12)).max(1e-12).log10()
    };
    assert!(
        diff_db <= -120.0,
        "preset com chaves inexistentes alterou o áudio em {:.2} dB (> -120 dB)",
        diff_db
    );

    // Presets de fábrica: nome corrigido para "US Twin Clean", id "ustwin" + alias "usdeluxe", zero ignored_keys
    let (_, _, list_bytes) = dispatch(&ctx_clean, "/api/presets", "GET", b"");
    let list_str = String::from_utf8_lossy(&list_bytes);
    assert!(
        list_str.contains("US Twin Clean") && !list_str.contains("US Deluxe Clean"),
        "lista de presets deve trazer 'US Twin Clean' sem 'US Deluxe Clean': {}",
        list_str
    );

    for preset_id in ["brit800", "ustwin", "usdeluxe", "recto", "ac30"] {
        let body = format!(r#"{{"id":"{}"}}"#, preset_id);
        let (code, _, resp_bytes) = dispatch(&ctx_clean, "/api/presets/load", "POST", body.as_bytes());
        assert_eq!(code, 200, "load preset '{}' falhou", preset_id);
        let v: serde_json::Value = serde_json::from_slice(&resp_bytes).unwrap();
        let ig = v["ignored_keys"]
            .as_array()
            .unwrap_or_else(|| panic!("preset '{}' não retornou ignored_keys: {}", preset_id, v));
        assert!(
            ig.is_empty(),
            "preset de fábrica '{}' não deveria ter chaves ignoradas: {:?}",
            preset_id,
            ig
        );
    }
}

#[test]
fn aud_09_toda_chave_ida_e_volta() {
    use sonix_desktop::params::{ParamDefault, ParamKind, PARAMS};
    use std::collections::HashSet;

    assert!(
        !PARAMS.is_empty(),
        "AUD-09: tabela única sonix_desktop::params::PARAMS não pode estar vazia"
    );

    let mut seen_keys = HashSet::new();
    for desc in PARAMS {
        assert!(
            seen_keys.insert(desc.key),
            "AUD-09: chave duplicada em PARAMS: {}",
            desc.key
        );
        for &alias in desc.aliases {
            assert!(
                seen_keys.insert(alias),
                "AUD-09: alias duplicado em PARAMS: {} -> {}",
                alias,
                desc.key
            );
        }

        let dir = TempDirGuard::new("sonix_aud09_roundtrip");
        let (ctx, _) = make_ctx_with_data_dir(dir.path.clone());
        let pq = ctx.param_queue.as_ref().expect("param_queue deve existir");
        while pq.pop().is_some() {}

        if let Some(allowed) = desc.models {
            assert!(!allowed.is_empty());
            let (m_code, m_body) = post_params(
                &ctx,
                &serde_json::json!({ "amp_1.model": allowed[0].as_str() }).to_string(),
            );
            assert_eq!(
                m_code, 200,
                "AUD-09: falha ao selecionar modelo {:?} para {}: {}",
                allowed[0], desc.key, m_body
            );
            while pq.pop().is_some() {}
        }

        let test_val: serde_json::Value = match (desc.kind, desc.default) {
            (ParamKind::Bool, ParamDefault::Bool(def)) => serde_json::json!(!def),
            (ParamKind::Number { min, max, step }, ParamDefault::Number(def)) => {
                let candidate = if (def - min).abs() > step.max(1e-3) {
                    min
                } else {
                    max
                };
                serde_json::json!(candidate)
            }
            (ParamKind::Enum(opts), ParamDefault::Enum(def)) => {
                let chosen = opts
                    .iter()
                    .copied()
                    .find(|&o| o != def)
                    .unwrap_or(opts[0]);
                serde_json::json!(chosen)
            }
            _ => panic!("AUD-09: kind/default inconsistentes para {}", desc.key),
        };

        let payload = serde_json::json!({ (desc.key): test_val });
        let (code, body) = post_params(&ctx, &payload.to_string());
        assert_eq!(
            code, 200,
            "AUD-09: POST /api/params falhou para {}={}: {}",
            desc.key, test_val, body
        );
        let resp_json: serde_json::Value = serde_json::from_str(&body).unwrap();
        let applied = resp_json["applied"]
            .as_array()
            .expect("resposta deve conter array applied");
        assert!(
            applied.iter().any(|v| v == desc.key),
            "AUD-09: chave '{}' deveria constar em applied ({:?})",
            desc.key,
            resp_json
        );

        let mut drained = Vec::new();
        while let Some(u) = pq.pop() {
            drained.push(u);
        }
        assert_eq!(
            drained.len(),
            1,
            "AUD-09: chave '{}' deve emitir exatamente 1 DspParamUpdate (emitiu {}: {:?})",
            desc.key,
            drained.len(),
            drained
        );

        let (st_code, _, st_bytes) = dispatch(&ctx, "/api/state", "GET", b"");
        assert_eq!(st_code, 200);
        let st_json: serde_json::Value = serde_json::from_slice(&st_bytes).unwrap();
        let actual = st_json
            .get(desc.key)
            .unwrap_or_else(|| panic!("AUD-09: chave '{}' ausente em GET /api/state", desc.key));

        match (&test_val, actual) {
            (serde_json::Value::Number(e), serde_json::Value::Number(a)) => {
                let ef = e.as_f64().unwrap();
                let af = a.as_f64().unwrap();
                assert!(
                    (ef - af).abs() < 1e-4,
                    "AUD-09: round-trip {} esperado {}, obtido {}",
                    desc.key,
                    ef,
                    af
                );
            }
            _ => {
                assert_eq!(
                    actual, &test_val,
                    "AUD-09: round-trip {} esperado {}, obtido {}",
                    desc.key, test_val, actual
                );
            }
        }
    }
}

#[test]
fn aud_09_ts_gerado_em_dia() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ts_path = manifest_dir
        .join("..")
        .join("..")
        .join("apps")
        .join("sonix-app")
        .join("ui")
        .join("src")
        .join("params.generated.ts");

    assert!(
        ts_path.exists(),
        "AUD-09: arquivo gerado '{}' não existe; rode `cargo run -p sonix-desktop --example gerar_params_ts`",
        ts_path.display()
    );

    let disk = std::fs::read_to_string(&ts_path)
        .expect("falha ao ler params.generated.ts")
        .replace("\r\n", "\n");
    let generated = sonix_desktop::params::render_params_ts().replace("\r\n", "\n");
    assert!(
        !generated.trim().is_empty(),
        "AUD-09: render_params_ts() não pode retornar vazio"
    );
    assert_eq!(
        disk, generated,
        "AUD-09: params.generated.ts está desatualizado em relação a PARAMS"
    );
}

#[test]
fn aud_09_lint_sem_literais_de_params_em_api_rs() {
    use sonix_desktop::params::PARAMS;

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let api_src = std::fs::read_to_string(manifest_dir.join("src").join("api.rs"))
        .expect("falha ao ler api.rs");
    let audio_src = std::fs::read_to_string(manifest_dir.join("src").join("audio.rs"))
        .expect("falha ao ler audio.rs");

    assert!(
        !PARAMS.is_empty(),
        "AUD-09 lint: PARAMS vazia"
    );
    assert!(
        !api_src.contains("json_kv_to_dsp_updates") && !api_src.contains("validate_preset_param_kv"),
        "AUD-09 lint: matches paralelos antigos ainda existem em api.rs"
    );
    assert!(
        !audio_src.contains("set_comp_2_input") && !audio_src.contains("0.8f32 * (0.02f32 / 0.8f32)"),
        "AUD-09 lint: conversão duplicada do 1176 ainda existe em audio.rs"
    );
    assert!(
        !api_src.contains("\"eq_1.band\""),
        "AUD-09 lint: prefixo literal \"eq_1.band\" proibido em api.rs"
    );

    for desc in PARAMS {
        let needle = format!("\"{}\"", desc.key);
        assert!(
            !api_src.contains(&needle),
            "AUD-09 lint: literal {} encontrado em api.rs",
            needle
        );
        for &alias in desc.aliases {
            let alias_needle = format!("\"{}\"", alias);
            assert!(
                !api_src.contains(&alias_needle),
                "AUD-09 lint: alias literal {} encontrado em api.rs",
                alias_needle
            );
        }
    }
}

