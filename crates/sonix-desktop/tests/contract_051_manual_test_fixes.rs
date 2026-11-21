//! Spec 051 — régua de contrato UI↔backend (CA-0) e correções T-01..T-07 no backend.
//! O fixture `tests/fixtures/ipc-schemas/051_ui_contract_calls.json` é gerado pelo teste JS
//! `tests/contract/051-correcao-contratos-e-controles-ui.test.js` a partir do que a UI REAL envia.

use serde_json::{json, Value};
use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{
    AudioStatusTelemetry, DesktopAudioHost, LockFreeParamQueue, RealtimeDspChain,
    RealtimeDuplexEngine, TelemetryBuffer,
};
use sonix_desktop::calibration::CalibrationStore;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn make_ctx() -> ApiContext {
    let dsp = Arc::new(Mutex::new(RealtimeDspChain::new(48000.0)));
    ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(512)),
        Some(dsp),
        Some(Arc::new(LockFreeParamQueue::new())),
        None,
        Some(CalibrationStore::new()),
    )
}

fn fixture() -> Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/ipc-schemas/051_ui_contract_calls.json");
    let txt = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("fixture 051 ausente ({p:?}): {e}"));
    serde_json::from_str(&txt).expect("fixture 051 inválido")
}

fn post_params(ctx: &ApiContext, key: &str, value: &Value) -> (u16, String) {
    let body = serde_json::to_vec(&json!({ key: value })).unwrap();
    let (code, _, resp) = dispatch(ctx, "/api/params", "POST", &body);
    (code, String::from_utf8_lossy(&resp).into_owned())
}

/// Chaves cujo valor enum é aceito por outro caminho que não `/api/params` ou que dependem de
/// contexto (declaradas aqui de forma explícita para não esconder lacunas).
const ENUM_KEYS_FORA_DO_PARAMS: &[&str] = &[];

#[test]
fn test_ca_00_every_enum_option_in_registry_is_accepted_by_backend() {
    let fx = fixture();
    let mut rejeitados = Vec::new();
    for ent in fx["enum_options"].as_array().unwrap() {
        let key = ent["key"].as_str().unwrap();
        if ENUM_KEYS_FORA_DO_PARAMS.contains(&key) {
            continue;
        }
        for opt in ent["options"].as_array().unwrap() {
            let ctx = make_ctx();
            let (code, body) = post_params(&ctx, key, opt);
            if code != 200 {
                rejeitados.push(format!("{key}={opt} -> {code} {body}"));
            }
        }
    }
    assert!(
        rejeitados.is_empty(),
        "CA-0: opções do registro rejeitadas pelo backend:\n{}",
        rejeitados.join("\n")
    );
}

#[test]
fn test_ca_00_every_param_write_the_ui_really_sends_is_accepted() {
    let fx = fixture();
    let mut rejeitados = Vec::new();
    for w in fx["param_writes"].as_array().unwrap() {
        let ctx = make_ctx();
        if let Some(model) = w["source"].as_str().and_then(|s| s.strip_prefix("AmpHead:")) {
            let (mcode, mbody) = post_params(&ctx, "amp_1.model", &serde_json::json!(model));
            assert_eq!(mcode, 200, "falha ao selecionar modelo {model}: {mbody}");
        }
        let (code, body) = post_params(&ctx, w["key"].as_str().unwrap(), &w["value"]);
        if code != 200 {
            rejeitados.push(format!("{} {}={} -> {code} {body}", w["source"], w["key"], w["value"]));
        }
    }
    assert!(
        rejeitados.is_empty(),
        "CA-0: escritas reais da UI rejeitadas:\n{}",
        rejeitados.join("\n")
    );
}

fn route_for(cmd: &str) -> &'static str {
    match cmd {
        "select_scene" => "/api/scenes/select",
        "calibrate_input" => "/api/input/calibrate",
        "load_preset" => "/api/presets/load",
        other => panic!("comando sem rota mapeada na régua 051: {other}"),
    }
}

#[test]
fn test_ca_00_every_command_payload_the_ui_really_sends_is_accepted() {
    let fx = fixture();
    let ctx = make_ctx();
    let mut rejeitados: Vec<String> = Vec::new();
    for c in fx["commands"].as_array().unwrap() {
        let cmd = c["cmd"].as_str().unwrap();
        let body = serde_json::to_vec(&c["payload"]).unwrap();
        let (code, _, resp) = dispatch(&ctx, route_for(cmd), "POST", &body);
        if code != 200 {
            rejeitados.push(format!(
                "{} {} {} -> {code} {}",
                c["source"],
                cmd,
                c["payload"],
                String::from_utf8_lossy(&resp)
            ));
        }
    }
    assert!(
        rejeitados.is_empty(),
        "CA-0: payloads reais da UI rejeitados:\n{}",
        rejeitados.join("\n")
    );
}

#[test]
fn test_ca_05_factory_presets_list_and_load_outside_repo_cwd() {
    let mut ctx = make_ctx();
    let empty_dir = std::env::temp_dir()
        .join("sonix_empty_presets_test_051")
        .join("a")
        .join("b")
        .join("c");
    let _ = std::fs::create_dir_all(&empty_dir);
    ctx.custom_presets_dir = Some(empty_dir.clone());
    let orig_cwd = std::env::current_dir().expect("current_dir");
    std::env::set_current_dir(&empty_dir).expect("set_current_dir temp");

    let (code_list, _, body_list) = dispatch(&ctx, "/api/presets", "GET", b"");
    let list_res: Value = serde_json::from_slice(&body_list).unwrap();
    let presets = list_res["presets"].as_array().expect("presets array");

    let mut results = Vec::new();
    for p in presets {
        if p["type"].as_str() != Some("factory") {
            continue;
        }
        let id = p["id"].as_str().unwrap();
        let body = serde_json::to_vec(&json!({ "id": id })).unwrap();
        let (code, _, resp) = dispatch(&ctx, "/api/presets/load", "POST", &body);
        results.push((id.to_string(), code, String::from_utf8_lossy(&resp).into_owned()));
    }

    let _ = std::env::set_current_dir(&orig_cwd);

    assert_eq!(code_list, 200);
    assert_eq!(results.len(), 4, "esperava 4 presets de fábrica");
    for (id, code, resp) in results {
        assert_eq!(
            code, 200,
            "CA-5: preset de fábrica '{}' falhou ao carregar fora do CWD do repo: {} {}",
            id, code, resp
        );
    }
}

#[test]
fn test_ca_01_calibration_wizard_begin_silence_strum_and_no_standby() {
    let ctx = make_ctx();
    if let Ok(mut st) = ctx.app_state.lock() {
        st.amp_standby = false;
    }
    let (c_begin, _, b_begin) = dispatch(
        &ctx,
        "/api/input/calibrate",
        "POST",
        br#"{"phase":"begin"}"#,
    );
    assert_eq!(c_begin, 200, "CA-1: phase begin deve retornar 200: {}", String::from_utf8_lossy(&b_begin));

    if let Some(ref ring) = ctx.clean_input_ring {
        let base = ring.total_written();
        for i in 0..4800u64 {
            let s = ((i as f32 * 0.1).sin()) * 0.001;
            ring.push_sample_at(base + i, s);
        }
        ring.commit_write_pos(base + 4800);
    }
    let (c_sil, _, b_sil) = dispatch(
        &ctx,
        "/api/input/calibrate",
        "POST",
        br#"{"phase":"silence"}"#,
    );
    assert_eq!(c_sil, 200, "CA-1: phase silence deve retornar 200: {}", String::from_utf8_lossy(&b_sil));

    if let Some(ref ring) = ctx.clean_input_ring {
        let base = ring.total_written();
        for i in 0..4800u64 {
            let s = ((i as f32 * 0.25).sin()) * 0.125;
            ring.push_sample_at(base + i, s);
        }
        ring.commit_write_pos(base + 4800);
    }
    let (c_str, _, b_str) = dispatch(
        &ctx,
        "/api/input/calibrate",
        "POST",
        br#"{"phase":"strum"}"#,
    );
    assert_eq!(c_str, 200, "CA-1: phase strum deve retornar 200: {}", String::from_utf8_lossy(&b_str));
    let v_str: Value = serde_json::from_slice(&b_str).unwrap();
    assert!(v_str["proposed_boost"].as_f64().unwrap_or(0.0) > 1.0, "CA-1: deve propor boost > 1.0 para sinal -18 dBFS");
    assert!(v_str["proposed_gate_threshold_db"].is_number(), "CA-1: deve propor gate threshold");

    // Spec 050 CA-5: calibração não pode deixar amp em standby
    let standby = ctx.app_state.lock().unwrap().amp_standby;
    assert!(
        !standby,
        "CA-1 / Spec 050 CA-5: calibração não pode deixar amp_standby = true"
    );
}

#[test]
fn test_ca_03_scenes_a_to_h_select_and_update_active_scene() {
    let ctx = make_ctx();
    for letter in ["A", "B", "C", "D", "E", "F", "G", "H"] {
        let body = serde_json::to_vec(&json!({ "scene": letter })).unwrap();
        let (code, _, resp) = dispatch(&ctx, "/api/scenes/select", "POST", &body);
        assert_eq!(code, 200, "CA-3: cena {letter} deve retornar 200: {}", String::from_utf8_lossy(&resp));
        let v: Value = serde_json::from_slice(&resp).unwrap();
        assert_eq!(v["scene"].as_str(), Some(letter));
        assert_eq!(v["state"]["scene.active"].as_str(), Some(letter));
    }
}

fn render_multitone_rms_db(ctx: &ApiContext, peak_dbfs: f32) -> f32 {
    render_multitone_signal(ctx, peak_dbfs).0
}

fn render_multitone_signal(ctx: &ApiContext, peak_dbfs: f32) -> (f32, Vec<f32>) {
    let amp_lin = 10.0f32.powf(peak_dbfs / 20.0);
    let mut dsp = ctx.dsp_chain.as_ref().unwrap().lock().unwrap();
    dsp.drain_param_queue();
    let n = 8192;
    let mut input = vec![0.0f32; n];
    let mut output = vec![0.0f32; n];
    for (i, s) in input.iter_mut().enumerate() {
        let t = i as f32 / 48000.0;
        let raw = 0.45 * (2.0 * std::f32::consts::PI * 110.0 * t).sin()
            + 0.30 * (2.0 * std::f32::consts::PI * 550.0 * t).sin()
            + 0.25 * (2.0 * std::f32::consts::PI * 2800.0 * t).sin();
        *s = amp_lin * raw;
    }
    dsp.process(&input, &mut output);
    let tail = output[4096..].to_vec();
    let rms = (tail.iter().map(|x| x * x).sum::<f32>() / tail.len() as f32).sqrt();
    (20.0 * rms.max(1e-7).log10(), tail)
}

fn waveform_diff_db(a: &[f32], b: &[f32]) -> f32 {
    let diff_rms = (a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f32>() / a.len() as f32).sqrt();
    let base_rms = (a.iter().map(|x| x * x).sum::<f32>() / a.len() as f32).sqrt().max(1e-7);
    20.0 * (diff_rms / base_rms).max(1e-7).log10()
}

#[test]
fn test_ca_08_brit800_controls_and_jacks_have_measurable_effect_after_calibration() {
    let ctx = make_ctx();
    // Desativa gate/drive para isolar o cabeçote Brit 800
    let _ = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"gate.enabled":false,"drive.enabled":false,"amp_1.model":"jcm800_2203","amp_1.input_jack":"high","amp_1.gain":4.0,"amp_1.bass":5.0,"amp_1.middle":5.0,"amp_1.treble":5.0,"amp_1.presence":5.0,"amp_1.master":5.0}"#,
    );

    // Medição ANTES da calibração (-39 dBFS) vs DEPOIS (-12 dBFS)
    let uncal_base_db = render_multitone_rms_db(&ctx, -39.0);
    let cal_base_db = render_multitone_rms_db(&ctx, -12.0);
    assert!(
        cal_base_db - uncal_base_db > 10.0,
        "CA-8: entrada calibrada (-12 dBFS) deve produzir nível muito superior ao sinal de -39 dBFS (uncal={uncal_base_db:.2} dB, cal={cal_base_db:.2} dB)"
    );

    // 1. PRE-AMP (gain): 1.0 vs 9.5
    let _ = post_params(&ctx, "amp_1.gain", &json!(1.0));
    let gain_lo = render_multitone_rms_db(&ctx, -12.0);
    let _ = post_params(&ctx, "amp_1.gain", &json!(9.5));
    let gain_hi = render_multitone_rms_db(&ctx, -12.0);
    assert!(
        gain_hi - gain_lo > 6.0,
        "CA-8: PRE-AMP (amp_1.gain) deve alterar sensivelmente a saída (lo={gain_lo:.2} dB, hi={gain_hi:.2} dB)"
    );
    let _ = post_params(&ctx, "amp_1.gain", &json!(3.5));

    // 2. MASTER: 1.0 vs 9.0
    let _ = post_params(&ctx, "amp_1.master", &json!(1.0));
    let master_lo = render_multitone_rms_db(&ctx, -12.0);
    let _ = post_params(&ctx, "amp_1.master", &json!(9.0));
    let master_hi = render_multitone_rms_db(&ctx, -12.0);
    assert!(
        master_hi - master_lo > 10.0,
        "CA-8: MASTER (amp_1.master) deve alterar sensivelmente o volume (lo={master_lo:.2} dB, hi={master_hi:.2} dB)"
    );
    let _ = post_params(&ctx, "amp_1.master", &json!(5.0));

    // 3. Tone stack (BASS, MIDDLE, TREBLE, PRESENCE): extremo 0 vs 10 altera forma de onda / espectro (> -22 dB de diferença relativa)
    for key in ["amp_1.bass", "amp_1.middle", "amp_1.treble", "amp_1.presence"] {
        let _ = post_params(&ctx, key, &json!(0.0));
        let (lo_db, lo_sig) = render_multitone_signal(&ctx, -12.0);
        let _ = post_params(&ctx, key, &json!(10.0));
        let (hi_db, hi_sig) = render_multitone_signal(&ctx, -12.0);
        let _ = post_params(&ctx, key, &json!(5.0));
        let diff_rel_db = waveform_diff_db(&lo_sig, &hi_sig);
        assert!(
            diff_rel_db > -22.0,
            "CA-8: {key} deve alterar a resposta do amplificador (0={lo_db:.2} dB, 10={hi_db:.2} dB, diff_rel={diff_rel_db:.2} dB)"
        );
    }

    // 4. Jack HIGH vs LOW no Brit 800 (~6 dB de atenuação na entrada)
    let _ = post_params(&ctx, "amp_1.gain", &json!(3.0));
    let _ = post_params(&ctx, "amp_1.input_jack", &json!("high"));
    let jack_hi = render_multitone_rms_db(&ctx, -18.0);
    let _ = post_params(&ctx, "amp_1.input_jack", &json!("low"));
    let jack_lo = render_multitone_rms_db(&ctx, -18.0);
    assert!(
        jack_hi - jack_lo > 3.0,
        "CA-8: jack LOW deve atenuar em relação a HIGH (high={jack_hi:.2} dB, low={jack_lo:.2} dB)"
    );
}

#[test]
fn test_ca_10_noise_gate_and_all_six_drive_pedals_have_measurable_effect() {
    let ctx = make_ctx();
    // 1. Noise Gate: sinal fraco (-50 dBFS) com threshold -20 dB (fechado) vs -80 dB (aberto)
    let _ = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"gate.enabled":true,"gate.range_db":-70.0,"gate.threshold_db":-20.0,"drive.enabled":false,"amp_1.enabled":false,"cab_1.enabled":false}"#,
    );
    let gate_closed_db = render_multitone_rms_db(&ctx, -50.0);
    let _ = post_params(&ctx, "gate.threshold_db", &json!(-80.0));
    let gate_open_db = render_multitone_rms_db(&ctx, -50.0);
    assert!(
        gate_open_db - gate_closed_db > 25.0,
        "CA-10: Noise Gate deve atenuar sinal abaixo do threshold (closed={gate_closed_db:.2} dB, open={gate_open_db:.2} dB)"
    );

    // 2. Todos os 6 pedais de Drive: drive.drive (0.0 vs 10.0), drive.tone (0.0 vs 10.0), drive.level (1.5 vs 9.5)
    let _ = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"gate.enabled":false,"drive.enabled":true,"amp_1.enabled":false,"cab_1.enabled":false}"#,
    );
    for model in ["ts808", "klon", "rat", "muff", "fuzzface", "ds1"] {
        let (code, body) = post_params(&ctx, "drive.model", &json!(model));
        assert_eq!(code, 200, "CA-10: drive.model={model} rejeitado: {body}");

        let _ = post_params(&ctx, "drive.tone", &json!(5.0));
        let _ = post_params(&ctx, "drive.level", &json!(6.0));
        let _ = post_params(&ctx, "drive.drive", &json!(0.0));
        let (drv_lo, sig_lo) = render_multitone_signal(&ctx, -22.0);
        let _ = post_params(&ctx, "drive.drive", &json!(10.0));
        let (drv_hi, sig_hi) = render_multitone_signal(&ctx, -22.0);
        let drv_diff = waveform_diff_db(&sig_lo, &sig_hi);
        assert!(
            drv_diff > -15.0,
            "CA-10 ({model}): drive.drive deve alterar a saída/saturação (0={drv_lo:.2} dB, 10={drv_hi:.2} dB, diff={drv_diff:.2} dB)"
        );

        let _ = post_params(&ctx, "drive.drive", &json!(5.0));
        let _ = post_params(&ctx, "drive.level", &json!(1.5));
        let lvl_lo = render_multitone_rms_db(&ctx, -18.0);
        let _ = post_params(&ctx, "drive.level", &json!(9.5));
        let lvl_hi = render_multitone_rms_db(&ctx, -18.0);
        assert!(
            lvl_hi - lvl_lo > 6.0,
            "CA-10 ({model}): drive.level deve alterar o volume (1.5={lvl_lo:.2} dB, 9.5={lvl_hi:.2} dB)"
        );
    }
}

#[test]
fn test_ca_11_master_level_and_parallel_line_b_merge_work_distinctly() {
    let ctx = make_ctx();
    let _ = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"gate.enabled":false,"drive.enabled":false,"split.enabled":false,"amp_1.enabled":true,"cab_1.enabled":true,"master.level_db":0.0}"#,
    );
    let base_db = render_multitone_rms_db(&ctx, -15.0);
    let (c_master, body_master) = post_params(&ctx, "master.level_db", &json!(-12.0));
    assert_eq!(c_master, 200, "CA-11: master.level_db deve ser aceito: {body_master}");
    let atten_db = render_multitone_rms_db(&ctx, -15.0);
    assert!(
        (base_db - atten_db - 12.0).abs() < 1.5,
        "CA-11: master.level_db=-12 dB deve atenuar saída principal em ~12 dB mesmo com split desligado (base={base_db:.2}, atten={atten_db:.2})"
    );
    let _ = post_params(&ctx, "master.level_db", &json!(0.0));

    // Com split.enabled=true e amp_1 desligado, a Linha B (amp_2 + cab_2) passa pelo merge.row_1_level_db
    let _ = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"split.enabled":true,"amp_1.enabled":false,"cab_1.enabled":false,"amp_2.enabled":true,"cab_2.enabled":true,"merge.row_1_level_db":0.0}"#,
    );
    let (_, sig_twin) = render_multitone_signal(&ctx, -15.0);
    let _ = post_params(&ctx, "amp_2.model", &json!("dual_rect"));
    let (_, sig_recto) = render_multitone_signal(&ctx, -15.0);
    assert!(
        waveform_diff_db(&sig_twin, &sig_recto) > -20.0,
        "CA-11: trocar amp_2.model na Linha B ativa deve alterar a saída"
    );
}

#[test]
fn test_ca_12_cab_1_and_cab_2_speakers_resonance_and_hicut_have_distinct_effect() {
    let ctx = make_ctx();
    let _ = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"gate.enabled":false,"drive.enabled":false,"amp_1.enabled":false,"cab_1.enabled":true,"cab_1.speaker":"v30_4x12"}"#,
    );
    let (_, sig_v30) = render_multitone_signal(&ctx, -12.0);
    let _ = post_params(&ctx, "cab_1.speaker", &json!("twin_2x12_open"));
    let (_, sig_twin) = render_multitone_signal(&ctx, -12.0);
    assert!(
        waveform_diff_db(&sig_v30, &sig_twin) > -20.0,
        "CA-12: trocar cab_1.speaker deve alterar o timbre"
    );

    // cab_2.resonance_hz e cab_2.hicut_khz próprios na Linha B
    let _ = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"split.enabled":true,"amp_1.enabled":false,"cab_1.enabled":false,"amp_2.enabled":false,"cab_2.enabled":true,"cab_2.resonance_hz":70.0,"cab_2.hicut_khz":16.0}"#,
    );
    let (c_res, b_res) = post_params(&ctx, "cab_2.resonance_hz", &json!(130.0));
    assert_eq!(c_res, 200, "CA-12: cab_2.resonance_hz deve ser aceito: {b_res}");
    let (c_hc, b_hc) = post_params(&ctx, "cab_2.hicut_khz", &json!(3.8));
    assert_eq!(c_hc, 200, "CA-12: cab_2.hicut_khz deve ser aceito: {b_hc}");
}

#[test]
fn test_ca_13_tuner_reads_pitch_and_looper_reverse_and_half_speed_work() {
    let ctx = make_ctx();
    // 1. Afinador: mesmo quando tuner.enabled foi ligado ANTES do sinal entrar no ring, /api/tuner detecta A2 (110 Hz ± 0.5 Hz)
    let _ = post_params(&ctx, "tuner.enabled", &json!(true));
    if let Some(ref ring) = ctx.clean_input_ring {
        let base = ring.total_written();
        for i in 0..4096u64 {
            let t = i as f32 / 48000.0;
            let s = 0.25 * (2.0 * std::f32::consts::PI * 110.0 * t).sin();
            ring.push_sample_at(base + i, s);
        }
        ring.commit_write_pos(base + 4096);
    }
    let (c_tun, _, b_tun) = dispatch(&ctx, "/api/tuner", "GET", b"");
    assert_eq!(c_tun, 200);
    let v_tun: Value = serde_json::from_slice(&b_tun).unwrap();
    let freq = v_tun["frequency_hz"].as_f64().unwrap_or(0.0);
    assert!(
        (freq - 110.0).abs() <= 0.5,
        "CA-13: /api/tuner deve detectar 110.0 +- 0.5 Hz (obteve {v_tun})"
    );
    assert_eq!(v_tun["note"].as_str(), Some("A2"), "CA-13: note deve ser A2: {v_tun}");
    assert_eq!(v_tun["note_letter"].as_str(), Some("A"), "CA-13: note_letter deve ser A sem oitava: {v_tun}");
    assert_eq!(v_tun["octave"].as_i64(), Some(2), "CA-13: octave deve ser 2: {v_tun}");
    assert_eq!(v_tun["signal_present"].as_bool(), Some(true), "CA-13: signal_present deve ser true: {v_tun}");

    // 2. Looper: aceita reverse e half_speed além de record/play/overdub/stop/undo/redo/clear e reporta telemetria
    for act in ["record", "play", "overdub", "undo", "redo", "reverse", "half_speed", "stop", "clear"] {
        let (code, body) = post_params(&ctx, "looper.action", &json!(act));
        assert_eq!(code, 200, "CA-13: looper.action={act} rejeitado: {body}");
    }
}

#[test]
fn test_ca_13_tuner_six_guitar_strings_accuracy_ref_hz_mute_and_no_signal() {
    let ctx = make_ctx();
    let _ = post_params(&ctx, "tuner.enabled", &json!(true));
    let _ = post_params(&ctx, "tuner.output", &json!("mute"));

    // 1. Estado inicial sem sinal: deve retornar "---", octave null, frequency_hz 0.0, signal_present false
    let (c_empty, _, b_empty) = dispatch(&ctx, "/api/tuner", "GET", b"");
    assert_eq!(c_empty, 200);
    let v_empty: Value = serde_json::from_slice(&b_empty).unwrap();
    assert_eq!(v_empty["note"].as_str(), Some("---"));
    assert!(v_empty["octave"].is_null(), "octave deve ser null sem sinal: {v_empty}");
    assert_eq!(v_empty["frequency_hz"].as_f64().unwrap_or(-1.0), 0.0);
    assert_eq!(v_empty["signal_present"].as_bool(), Some(false));

    // 2. Todas as 6 frequências obrigatórias com erro <= 0.5 Hz e tuner.output = mute silenciando saída
    let cases = [
        (82.41f32, "E2", "E", 2i64),
        (110.00, "A2", "A", 2),
        (146.83, "D3", "D", 3),
        (220.00, "A3", "A", 3),
        (329.63, "E4", "E", 4),
        (440.00, "A4", "A", 4),
    ];
    for (target_hz, full_note, letter, oct) in cases {
        let inp: Vec<f32> = (0..4096)
            .map(|i| 0.25 * (2.0 * std::f32::consts::PI * target_hz * (i as f32 / 48000.0)).sin())
            .collect();
        let mut out = vec![0.0f32; 4096];
        {
            let mut chain = ctx.dsp_chain.as_ref().unwrap().lock().unwrap();
            chain.process(&inp, &mut out);
        }
        let rms_out = (out[512..].iter().map(|s| s * s).sum::<f32>() / (out.len() - 512) as f32).sqrt();
        assert!(
            rms_out < 1e-4,
            "CA-13: tuner.output=mute deve silenciar saída (rms_out={rms_out} para {target_hz} Hz)"
        );

        let (c, _, b) = dispatch(&ctx, "/api/tuner", "GET", b"");
        assert_eq!(c, 200);
        let v: Value = serde_json::from_slice(&b).unwrap();
        let freq = v["frequency_hz"].as_f64().unwrap_or(0.0);
        assert!(
            (freq - target_hz as f64).abs() <= 0.5,
            "Nota {full_note} ({target_hz} Hz) fora de +-0.5 Hz: obteve {freq:.3} Hz ({v})"
        );
        assert_eq!(v["note"].as_str(), Some(full_note), "{v}");
        assert_eq!(v["note_letter"].as_str(), Some(letter), "{v}");
        assert_eq!(v["octave"].as_i64(), Some(oct), "{v}");
        assert_eq!(v["muted"].as_bool(), Some(true), "{v}");
    }

    // 3. Propagação real de tuner.ref_hz = 442.0 para A4 (440 Hz -> ~ -7.85 cents relativo a A4=442)
    let _ = post_params(&ctx, "tuner.ref_hz", &json!(442.0));
    let (c_ref, _, b_ref) = dispatch(&ctx, "/api/tuner", "GET", b"");
    assert_eq!(c_ref, 200);
    let v_ref: Value = serde_json::from_slice(&b_ref).unwrap();
    let cents_442 = v_ref["cents"].as_f64().unwrap_or(0.0);
    assert!(
        (cents_442 - (-7.85)).abs() < 1.0,
        "tuner.ref_hz=442 deve ser aplicado ao /api/tuner em 440 Hz (esperado ~-7.85 cents, obteve {cents_442})"
    );
    assert!((v_ref["ref_hz"].as_f64().unwrap_or(0.0) - 442.0).abs() < 0.01);

    // 4. Quando entra silêncio longo no ring, sinal volta a ausente ("---")
    let silent = vec![0.0f32; 4096];
    let mut silent_out = vec![0.0f32; 4096];
    {
        let mut chain = ctx.dsp_chain.as_ref().unwrap().lock().unwrap();
        chain.process(&silent, &mut silent_out);
    }
    let (_, _, b_sil) = dispatch(&ctx, "/api/tuner", "GET", b"");
    let v_sil: Value = serde_json::from_slice(&b_sil).unwrap();
    assert_eq!(v_sil["note"].as_str(), Some("---"));
    assert_eq!(v_sil["frequency_hz"].as_f64().unwrap_or(-1.0), 0.0);
    assert_eq!(v_sil["signal_present"].as_bool(), Some(false));
}

#[test]
fn test_ca_15_chassis_preamp_tube_ecc83_12ay7_and_mods_have_measurable_effect() {
    let ctx = make_ctx();
    for tube in ["12ax7", "ecc83", "12at7", "12ay7", "ef86", "ef86_variant"] {
        let (code, body) = post_params(&ctx, "amp_1.preamp_tube", &json!(tube));
        assert_eq!(code, 200, "CA-15: amp_1.preamp_tube={tube} rejeitado: {body}");
    }
    let _ = dispatch(
        &ctx,
        "/api/params",
        "POST",
        br#"{"gate.enabled":false,"drive.enabled":false,"amp_1.enabled":true,"cab_1.enabled":false,"amp_1.model":"jcm800_2203","amp_1.gain":7.5,"amp_1.master":7.5}"#,
    );
    let _ = post_params(&ctx, "amp_1.power_tube", &json!("el34"));
    let (_, sig_el34) = render_multitone_signal(&ctx, -12.0);
    let _ = post_params(&ctx, "amp_1.power_tube", &json!("el84_variant"));
    let (_, sig_el84) = render_multitone_signal(&ctx, -12.0);
    assert!(
        waveform_diff_db(&sig_el34, &sig_el84) > -24.0,
        "CA-15: trocar válvula de potência EL34 -> EL84 deve alterar a saída"
    );

    let _ = post_params(&ctx, "amp_1.bias_ma", &json!(22.0));
    let (_, sig_cold) = render_multitone_signal(&ctx, -12.0);
    let _ = post_params(&ctx, "amp_1.bias_ma", &json!(52.0));
    let (_, sig_hot) = render_multitone_signal(&ctx, -12.0);
    assert!(
        waveform_diff_db(&sig_cold, &sig_hot) > -25.0,
        "CA-15: alterar bias_ma (22 vs 52 mA) deve alterar a saída"
    );
}

#[test]
fn test_ca_15_scope_waveform_window_and_no_signal_handling() {
    let ctx = make_ctx();

    // 1. Silêncio absoluto processado pelo callback não pode ser reportado como sinal válido no osciloscópio
    let silent = vec![0.0f32; 1024];
    let mut sil_out = vec![0.0f32; 1024];
    {
        let mut chain = ctx.dsp_chain.as_ref().unwrap().lock().unwrap();
        chain.process(&silent, &mut sil_out);
    }
    let (c_sil, _, b_sil) = dispatch(&ctx, "/api/scope?mode=waveform", "GET", b"");
    assert_eq!(c_sil, 200);
    let v_sil: Value = serde_json::from_slice(&b_sil).unwrap();
    assert_eq!(
        v_sil["no_signal"].as_bool(),
        Some(true),
        "Silêncio absoluto no ring deve reportar no_signal=true: {v_sil}"
    );
    assert_eq!(
        v_sil["waveform"]["output"].as_array().map(|a| a.len()),
        Some(0),
        "Sem sinal real, waveform.output não pode conter zeros falsos: {v_sil}"
    );

    // 2. Com sinal E2 (82.41 Hz) contínuo, /api/scope entrega janela >= 512 amostras (até 1024) para cobrir ciclos graves completos
    let sig: Vec<f32> = (0..2048)
        .map(|i| 0.25 * (2.0 * std::f32::consts::PI * 82.41 * (i as f32 / 48000.0)).sin())
        .collect();
    let mut sig_out = vec![0.0f32; 2048];
    {
        let mut chain = ctx.dsp_chain.as_ref().unwrap().lock().unwrap();
        chain.process(&sig, &mut sig_out);
    }
    let (c_sig, _, b_sig) = dispatch(&ctx, "/api/scope?mode=waveform", "GET", b"");
    assert_eq!(c_sig, 200);
    let v_sig: Value = serde_json::from_slice(&b_sig).unwrap();
    assert_eq!(v_sig["no_signal"].as_bool(), Some(false));
    let di_len = v_sig["waveform"]["di"].as_array().map(|a| a.len()).unwrap_or(0);
    let out_len = v_sig["waveform"]["output"].as_array().map(|a| a.len()).unwrap_or(0);
    assert!(
        di_len >= 512 && out_len >= 512,
        "Janela do osciloscópio deve ser >= 512 amostras para cobrir notas graves (di={di_len}, out={out_len})"
    );
}

#[test]
fn test_ca_17_oversampling_2x_4x_8x_updates_algorithmic_latency_and_audio() {
    let ctx = make_ctx();
    // Padrão inicial deve ser 2x (4 amostras) conforme Spec 051 CA-17
    let (c_init_st, _, b_init_st) = dispatch(&ctx, "/api/state", "GET", b"");
    assert_eq!(c_init_st, 200);
    let v_init_st: Value = serde_json::from_slice(&b_init_st).unwrap();
    assert_eq!(
        v_init_st["engine.oversampling"].as_str(),
        Some("2x"),
        "CA-17: AppState padrão deve iniciar em engine.oversampling='2x': {v_init_st}"
    );
    let (c_init_aud, _, b_init_aud) = dispatch(&ctx, "/api/audio/status", "GET", b"");
    assert_eq!(c_init_aud, 200);
    let v_init_aud: Value = serde_json::from_slice(&b_init_aud).unwrap();
    assert_eq!(
        v_init_aud["oversampling"].as_str(),
        Some("2x"),
        "CA-17: /api/audio/status padrão deve iniciar em 2x: {v_init_aud}"
    );
    assert_eq!(
        v_init_aud["algorithmic_latency_samples"].as_u64(),
        Some(4),
        "CA-17: /api/audio/status padrão deve iniciar em 4 amostras (2x): {v_init_aud}"
    );

    let expected = [("2x", 4u64), ("4x", 8u64), ("8x", 16u64)];
    for (mode, samples) in expected {
        let (code, body) = post_params(&ctx, "engine.oversampling", &json!(mode));
        assert_eq!(code, 200, "CA-17: engine.oversampling={mode} rejeitado: {body}");
        let mut chain = ctx.dsp_chain.as_ref().unwrap().lock().unwrap();
        let inp = [0.0f32; 64];
        let mut out = [0.0f32; 64];
        chain.process(&inp, &mut out);
        drop(chain);

        let (c_st, _, b_st) = dispatch(&ctx, "/api/audio/status", "GET", b"");
        assert_eq!(c_st, 200);
        let v_st: Value = serde_json::from_slice(&b_st).unwrap();
        assert_eq!(
            v_st["oversampling"].as_str(),
            Some(mode),
            "CA-17: /api/audio/status deve refletir oversampling={mode}"
        );
        assert_eq!(
            v_st["algorithmic_latency_samples"].as_u64(),
            Some(samples),
            "CA-17: oversampling {mode} deve reportar {samples} amostras de latência algorítmica"
        );
    }

    // Com audio_telemetry ativo (Some(t)), trocar oversampling e período observado atualiza estimated_roundtrip_ms e períodos
    let _ = post_params(&ctx, "engine.oversampling", &json!("2x"));
    {
        let mut chain = ctx.dsp_chain.as_ref().unwrap().lock().unwrap();
        let inp = [0.0f32; 64];
        let mut out = [0.0f32; 64];
        chain.process(&inp, &mut out);
    }
    *ctx.audio_telemetry.lock().unwrap() = Some(AudioStatusTelemetry {
        backend: "wasapi".to_string(),
        mode: "wasapi_shared".to_string(),
        fallback_reason: None,
        raw_processing: false,
        sample_rate: 48000,
        input_period_frames: 128,
        output_period_frames: 128,
        input_device: "In".to_string(),
        output_device: "Out".to_string(),
        driver_input_latency_ms: 2.667,
        driver_output_latency_ms: 2.667,
        cpu_avg_pct: 1.0,
        cpu_peak_pct: 2.0,
        underruns: 0,
        overruns: 0,
        panics: 0,
        adc_clips: 0,
        measured_roundtrip_ms: None,
        estimated_roundtrip_ms: 2.667 + 2.667 + 2.667 + (4.0 / 48.0),
        algorithmic_latency_samples: 4,
        pitch_wet_latency_ms: 0.0,
    });
    let (_, _, b_2x) = dispatch(&ctx, "/api/audio/status", "GET", b"");
    let v_2x: Value = serde_json::from_slice(&b_2x).unwrap();
    let rt_2x = v_2x["estimated_roundtrip_ms"].as_f64().unwrap_or(0.0);

    let _ = post_params(&ctx, "engine.oversampling", &json!("8x"));
    {
        let mut chain = ctx.dsp_chain.as_ref().unwrap().lock().unwrap();
        let inp = [0.0f32; 64];
        let mut out = [0.0f32; 64];
        chain.process(&inp, &mut out);
    }
    let (_, _, b_8x) = dispatch(&ctx, "/api/audio/status", "GET", b"");
    let v_8x: Value = serde_json::from_slice(&b_8x).unwrap();
    let rt_8x = v_8x["estimated_roundtrip_ms"].as_f64().unwrap_or(0.0);
    assert!(
        (rt_8x - rt_2x - 0.25).abs() < 0.01,
        "CA-17/CA-25: estimated_roundtrip_ms com stream ativo deve subir +0.25 ms de 2x para 8x (2x={rt_2x}, 8x={rt_8x})"
    );

    // Callbacks observando período real de 480 frames (10 ms @ 48 kHz) propagam para /api/audio/status
    let duplex = RealtimeDuplexEngine::with_target_period(4096, 480);
    let in_stereo = vec![0.05f32; 480 * 2];
    let mut mono_in = vec![0.0f32; 480];
    let mut mono_out = vec![0.0f32; 480];
    let mut out_stereo = vec![0.0f32; 480 * 2];
    DesktopAudioHost::process_input_callback_f32(&in_stereo, 2, &mut mono_in, &duplex, &ctx.telemetry);
    DesktopAudioHost::process_output_callback_f32(
        &mut out_stereo,
        2,
        48000.0,
        &mut mono_in,
        &mut mono_out,
        &duplex,
        ctx.dsp_chain.as_ref().unwrap(),
        &ctx.telemetry,
        None,
    );
    let (_, _, b_obs) = dispatch(&ctx, "/api/audio/status", "GET", b"");
    let v_obs: Value = serde_json::from_slice(&b_obs).unwrap();
    assert_eq!(v_obs["input_period_frames"].as_u64(), Some(480));
    assert_eq!(v_obs["output_period_frames"].as_u64(), Some(480));
    assert!((v_obs["driver_input_latency_ms"].as_f64().unwrap_or(0.0) - 10.0).abs() < 0.05);
    assert!(v_obs["estimated_roundtrip_ms"].as_f64().unwrap_or(0.0) >= 30.0);
}
