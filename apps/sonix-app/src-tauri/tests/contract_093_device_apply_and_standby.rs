//! Spec 093 — CA-3: Standby seguro no boot frio + saída automática ao aplicar dispositivo

use sonix_app_lib::build_runtime_for_startup_test_in_dir;
use sonix_app_lib::commands::apply_device_selection_to_runtime;
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

#[test]
fn test_ca_03_apply_device_updates_active_device_id_and_exits_standby() {
    let dir = TempDirGuard::new("sonix_093_ca3");
    let runtime = build_runtime_for_startup_test_in_dir(dir.path.clone(), Some("default"));

    // Boot frio sem calibração deve continuar iniciando em Standby (Spec 050)
    {
        let st = runtime.api_ctx.app_state.lock().expect("state lock");
        assert!(st.amp_standby, "Boot frio sem calibração deve iniciar em Standby");
    }

    // Ao aplicar um dispositivo de entrada/saída, active_device_id atualiza e Standby desliga
    apply_device_selection_to_runtime(
        &runtime,
        "line-in-pc-01",
        "pc_line_in",
        "left",
    );

    let active_dev = runtime
        .api_ctx
        .active_device_id
        .lock()
        .expect("active_device_id lock")
        .clone();
    assert_eq!(active_dev.as_deref(), Some("line-in-pc-01"));

    let st = runtime.api_ctx.app_state.lock().expect("state lock");
    assert_eq!(st.input_mode, "pc_line_in");
    assert_eq!(st.input_channel, "left");
    assert!(
        !st.amp_standby,
        "Aplicar dispositivo de áudio deve sair automaticamente de Standby"
    );
    assert!(
        !runtime.api_ctx.telemetry.is_output_muted(),
        "Telemetria não pode permanecer mutada após aplicar dispositivo"
    );
}
