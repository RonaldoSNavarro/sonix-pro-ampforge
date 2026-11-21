//! Testes de auditoria 2026-11-20 — AUD-18: Persistência atômica de calibração por dispositivo

use serde_json::Value;
use sonix_app_lib::build_runtime_with_data_dir;
use sonix_desktop::api::dispatch;
use std::f32::consts::PI;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn feed_clean_sine(runtime: &sonix_app_lib::AppRuntime, amp: f32, frames: usize) {
    let dsp_arc = runtime.api_ctx.dsp_chain.as_ref().expect("dsp_chain");
    let mut dsp = dsp_arc.lock().expect("dsp lock");
    let mut inp = vec![0.0_f32; 256];
    let mut out = vec![0.0_f32; 256];
    let mut fed = 0usize;
    while fed < frames {
        for (i, s) in inp.iter_mut().enumerate() {
            let idx = fed + i;
            *s = amp * (2.0 * PI * 196.0 * (idx as f32) / 48_000.0).sin();
        }
        dsp.process(&inp, &mut out);
        fed += 256;
    }
}

fn calibrate_through_api(runtime: &sonix_app_lib::AppRuntime) -> (f32, f32) {
    let (s0, _, b0) = dispatch(
        &runtime.api_ctx,
        "/api/input/calibrate",
        "POST",
        br#"{"phase":"begin"}"#,
    );
    assert_eq!(s0, 200, "begin falhou: {}", String::from_utf8_lossy(&b0));

    feed_clean_sine(runtime, 0.0004, 2048);
    let (s1, _, b1) = dispatch(
        &runtime.api_ctx,
        "/api/input/calibrate",
        "POST",
        br#"{"phase":"silence"}"#,
    );
    assert_eq!(s1, 200, "silence falhou: {}", String::from_utf8_lossy(&b1));

    feed_clean_sine(runtime, 0.08, 4096);
    let (s2, _, b2) = dispatch(
        &runtime.api_ctx,
        "/api/input/calibrate",
        "POST",
        br#"{"phase":"strum"}"#,
    );
    assert_eq!(s2, 200, "strum falhou: {}", String::from_utf8_lossy(&b2));
    let v: Value = serde_json::from_slice(&b2).unwrap();
    let boost = v["proposed_boost"].as_f64().expect("proposed_boost") as f32;
    let gate = v["proposed_gate_threshold_db"]
        .as_f64()
        .expect("proposed_gate_threshold_db") as f32;
    (boost, gate)
}

#[test]
fn aud_18_calibracao_sobrevive_ao_reinicio() {
    let guard = TempDirGuard::new("sonix_aud_18_persist");
    let dev = "USB_Audio_Interface_X";

    let (saved_boost, saved_gate) = {
        let (rt, _) = build_runtime_with_data_dir(guard.path.clone(), Some(dev), false);
        assert!(
            rt.api_ctx.app_state.lock().unwrap().amp_standby,
            "sem arquivo prévio, deve iniciar em standby"
        );
        calibrate_through_api(&rt)
    };

    let json_path = guard.path.join("calibration.json");
    assert!(json_path.exists(), "AUD-18: calibration.json deve existir no disco");
    let parsed: Value = serde_json::from_str(&std::fs::read_to_string(&json_path).unwrap()).unwrap();
    assert_eq!(parsed["versao"].as_u64(), Some(1));
    assert!(parsed["dispositivos"][dev]["salvo_em"].as_str().is_some());

    // Novo boot com mesma pasta e mesmo dispositivo
    let (rt2, _) = build_runtime_with_data_dir(guard.path.clone(), Some(dev), false);
    let st2 = rt2.api_ctx.app_state.lock().unwrap();
    assert!(
        !st2.amp_standby,
        "AUD-18: após calibração e reinício no mesmo dispositivo, não pode abrir em standby"
    );
    assert!(
        (st2.input_boost - saved_boost).abs() < 1e-4,
        "AUD-18: input_boost após reinício ({}) difere do gravado ({saved_boost})",
        st2.input_boost
    );
    assert!(
        (st2.gate_threshold_db - saved_gate).abs() < 1e-4,
        "AUD-18: gate_threshold_db após reinício ({}) difere do gravado ({saved_gate})",
        st2.gate_threshold_db
    );
}

#[test]
fn aud_18_chave_de_gravacao_igual_a_de_leitura() {
    let guard = TempDirGuard::new("sonix_aud_18_key");

    let (saved_boost, saved_gate) = {
        let (rt_x, _) = build_runtime_with_data_dir(guard.path.clone(), Some("Device_X"), false);
        calibrate_through_api(&rt_x)
    };

    // Boot com X encontra
    {
        let (rt_x2, _) = build_runtime_with_data_dir(guard.path.clone(), Some("Device_X"), false);
        let st = rt_x2.api_ctx.app_state.lock().unwrap();
        assert!(!st.amp_standby, "AUD-18: boot com Device_X deve restaurar calibração");
        assert!((st.input_boost - saved_boost).abs() < 1e-4);
        assert!((st.gate_threshold_db - saved_gate).abs() < 1e-4);
    }

    // Boot com Y entra em standby
    {
        let (rt_y, _) = build_runtime_with_data_dir(guard.path.clone(), Some("Device_Y"), false);
        let st = rt_y.api_ctx.app_state.lock().unwrap();
        assert!(
            st.amp_standby,
            "AUD-18: boot com Device_Y (não calibrado) deve entrar em standby"
        );
    }
}

#[test]
fn aud_18_arquivo_corrompido() {
    let guard = TempDirGuard::new("sonix_aud_18_corrupt");
    let json_path = guard.path.join("calibration.json");
    std::fs::write(&json_path, b"{corrompido json invalido").unwrap();

    let (rt, _) = build_runtime_with_data_dir(guard.path.clone(), Some("Device_X"), false);
    assert!(
        rt.api_ctx.app_state.lock().unwrap().amp_standby,
        "AUD-18: JSON inválido deve fazer boot entrar em standby sem pânico"
    );
    let corrupt_path = guard.path.join("calibration.json.corrompido");
    assert!(
        corrupt_path.exists(),
        "AUD-18: calibration.json inválido deve ser renomeado para .corrompido"
    );
    assert!(
        !json_path.exists(),
        "AUD-18: arquivo original inválido não deve permanecer como calibration.json"
    );
}
