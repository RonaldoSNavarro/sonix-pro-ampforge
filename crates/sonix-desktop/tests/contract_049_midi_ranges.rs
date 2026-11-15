//! crates/sonix-desktop/tests/contract_049_midi_ranges.rs
//! Testes de Contrato para Spec 049 (Parte A):
//! - CA-11: Todo controlador MIDI mapeado e valor 0..127 passa na validação de dispatch; CC14 0 -> -90 dB, 127 -> 0 dB.

use std::sync::{Arc, Mutex};

use sonix_desktop::api::{dispatch, ApiContext, AppState};
use sonix_desktop::audio::{RealtimeDspChain, TelemetryBuffer};
use sonix_desktop::midi::DesktopMidiHost;

#[test]
fn ca11_all_mapped_midi_ccs_and_values_0_to_127_pass_dispatch_validation() {
    let chain = RealtimeDspChain::new(48000.0);
    let param_queue = chain.param_queue();
    let ctx = ApiContext::new(
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(AppState::default())),
        Arc::new(TelemetryBuffer::new(1024)),
        Some(Arc::new(Mutex::new(chain))),
        Some(param_queue),
        None,
        None,
    );

    for cc in 0u8..=127u8 {
        for val in 0u8..=127u8 {
            if let Some((key, mapped_val)) = DesktopMidiHost::map_cc_to_param(cc, val) {
                let payload = serde_json::json!({ key: mapped_val });
                let (code, _, body) = dispatch(
                    &ctx,
                    "/api/params",
                    "POST",
                    &serde_json::to_vec(&payload).unwrap(),
                );
                assert_eq!(
                    code,
                    200,
                    "CA-11: CC {} val {} -> {}={} falhou na validação: {}",
                    cc,
                    val,
                    key,
                    mapped_val,
                    String::from_utf8_lossy(&body)
                );
            }
        }
    }

    let (k0, v0) = DesktopMidiHost::map_cc_to_param(14, 0).unwrap();
    assert_eq!(k0, "gate.threshold_db");
    assert!((v0.as_f64().unwrap() - (-90.0)).abs() < 1e-4, "CA-11: CC14 val 0 deu {:?}", v0);

    let (k127, v127) = DesktopMidiHost::map_cc_to_param(14, 127).unwrap();
    assert_eq!(k127, "gate.threshold_db");
    assert!((v127.as_f64().unwrap() - 0.0).abs() < 1e-4, "CA-11: CC14 val 127 deu {:?}", v127);
}
