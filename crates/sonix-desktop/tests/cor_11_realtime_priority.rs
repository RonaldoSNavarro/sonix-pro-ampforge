//! COR-11 — o thread de áudio do produto roda com prioridade de tempo real.
//!
//! O `cpal` 0.18.2 só promove o thread de áudio do WASAPI
//! (`audio_thread_priority::promote_current_thread_to_real_time`, com
//! fallback `THREAD_PRIORITY_TIME_CRITICAL`) quando a feature `realtime`
//! está ligada. Este teste reprova se a feature `host-audio` não a ligar ou
//! se `audio_thread_priority` não estiver resolvido no `Cargo.lock`.

use std::path::PathBuf;

#[test]
fn cor_11_host_audio_ativa_realtime() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let toml = std::fs::read_to_string(manifest_dir.join("Cargo.toml")).expect("ler Cargo.toml");
    let host_audio = toml
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("host-audio") && l.contains('='))
        .expect("feature host-audio ausente em crates/sonix-desktop/Cargo.toml");
    assert!(
        host_audio.contains("\"dep:cpal\"") && host_audio.contains("\"cpal/realtime\""),
        "host-audio precisa ser [\"dep:cpal\", \"cpal/realtime\"]; encontrado: {host_audio}"
    );

    let lock = std::fs::read_to_string(manifest_dir.join("../../Cargo.lock")).expect("ler Cargo.lock");
    assert!(
        lock.lines().any(|l| l.trim() == "name = \"audio_thread_priority\""),
        "audio_thread_priority não está no Cargo.lock"
    );
}
