use sonix_app_lib::build_runtime_for_startup_test_in_dir;
use sonix_desktop::calibration::CalibrationStore;
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

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|x| x * x).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

#[test]
fn test_ca_02_uncalibrated_startup_enters_safe_standby_silence() {
    let dir = TempDirGuard::new("sonix_050_ca2");
    let runtime = build_runtime_for_startup_test_in_dir(dir.path.clone(), Some("default"));

    let st = runtime.api_ctx.app_state.lock().expect("state lock");
    assert!(
        st.amp_standby,
        "sem calibração salva, app desktop deve iniciar em standby seguro para evitar chiado"
    );
    drop(st);

    let mut input = vec![0.0f32; 1024];
    for (i, s) in input.iter_mut().enumerate() {
        *s = 0.05 * ((i as f32) * 0.08).sin();
    }
    let mut output = vec![0.0f32; 1024];
    {
        let dsp_arc = runtime.api_ctx.dsp_chain.as_ref().expect("dsp_chain");
        let mut dsp = dsp_arc.lock().expect("dsp lock");
        dsp.process(&input, &mut output);
    }
    assert!(
        rms(&output[256..]) < 1e-5,
        "em standby inicial, saída deve ser silenciosa (rms={})",
        rms(&output[256..])
    );

    // Ao desativar standby, o sinal volta a ser processado
    {
        let dsp_arc = runtime.api_ctx.dsp_chain.as_ref().expect("dsp_chain");
        let mut dsp = dsp_arc.lock().expect("dsp lock");
        dsp.set_amp_standby(false);
        dsp.process(&input, &mut output);
    }
    assert!(
        rms(&output[256..]) > 1e-3,
        "ao sair de standby, sinal deve voltar a sair normalmente (rms={})",
        rms(&output[256..])
    );
}

#[test]
fn test_ca_04_calibrated_startup_keeps_amp_active() {
    let dir = TempDirGuard::new("sonix_050_ca4");
    {
        let store = CalibrationStore::open(&dir.path);
        store.save("default", 1.5, -55.0);
    }
    let runtime = build_runtime_for_startup_test_in_dir(dir.path.clone(), Some("default"));
    {
        let st = runtime.api_ctx.app_state.lock().expect("state lock");
        assert!(
            !st.amp_standby,
            "com calibração salva, amplificador não força standby"
        );
        assert!(
            (st.input_boost - 1.5).abs() < 1e-6,
            "S7: input_boost calibrado deve ser restaurado no AppState (got {})",
            st.input_boost
        );
        assert!(
            (st.gate_threshold_db - (-55.0)).abs() < 1e-6,
            "S7: gate_threshold_db calibrado deve ser restaurado no AppState (got {})",
            st.gate_threshold_db
        );
    }

    let mut input = vec![0.0f32; 1024];
    for (i, s) in input.iter_mut().enumerate() {
        *s = 0.02 * ((i as f32) * 0.08).sin();
    }
    let mut output_calibrated = vec![0.0f32; 1024];
    let mut output_unity = vec![0.0f32; 1024];
    {
        let dsp_arc = runtime.api_ctx.dsp_chain.as_ref().expect("dsp_chain");
        let mut dsp = dsp_arc.lock().expect("dsp lock");
        dsp.process(&input, &mut output_calibrated);
        // Reset para boost unitário e compara para provar que 1.5x chegou ao DSP no startup
        dsp.set_input_boost(1.0);
        dsp.process(&input, &mut output_unity);
    }
    let rms_cal = rms(&output_calibrated[256..]);
    let rms_uni = rms(&output_unity[256..]);
    assert!(
        rms_cal > rms_uni * 1.15,
        "S7: input_boost=1.5 calibrado deve chegar ao DSP no startup (cal={rms_cal}, unity={rms_uni})"
    );
}

#[test]
fn test_ca_05_calibration_disables_initial_standby_in_state_and_dsp() {
    let dir = TempDirGuard::new("sonix_050_ca5");
    let runtime = build_runtime_for_startup_test_in_dir(dir.path.clone(), Some("default"));
    assert!(
        runtime.api_ctx.app_state.lock().expect("state lock").amp_standby,
        "pré-condição: sem calibração, inicia em standby"
    );

    let mut ctx = runtime.api_ctx.clone();
    let (s1, _, _) = sonix_desktop::api::dispatch(
        &mut ctx,
        "/api/input/calibrate",
        "POST",
        br#"{"phase":"silence"}"#,
    );
    assert_eq!(s1, 200);
    let (s2, _, _) = sonix_desktop::api::dispatch(
        &mut ctx,
        "/api/input/calibrate",
        "POST",
        br#"{"phase":"strum"}"#,
    );
    assert_eq!(s2, 200);

    let st_after = ctx.app_state.lock().expect("state lock").amp_standby;
    assert!(
        !st_after,
        "CA-5 (S3): calibração concluída deve desativar amp_standby no AppState"
    );

    let mut input = vec![0.0f32; 1024];
    for (i, s) in input.iter_mut().enumerate() {
        *s = 0.05 * ((i as f32) * 0.08).sin();
    }
    let mut output = vec![0.0f32; 1024];
    {
        let dsp_arc = ctx.dsp_chain.as_ref().expect("dsp_chain");
        let mut dsp = dsp_arc.lock().expect("dsp lock");
        dsp.process(&input, &mut output);
    }
    assert!(
        rms(&output[256..]) > 1e-3,
        "CA-5 (S3): após calibração, DSP deve sair de standby e amplificar o sinal (rms={})",
        rms(&output[256..])
    );
}
