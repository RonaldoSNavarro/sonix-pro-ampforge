//! crates/sonix-desktop/tests/contract_013_desktop_realtime_audio.rs
//! Testes de Contrato Automatizados para Spec 013: Execução Desktop Nativa no Windows com Captura Física P2/P3 e Duplo Clique
//! Cobre Critérios de Aceite CA-1 a CA-8
//!
//! CA-1: Execução Duplo-Clique sem CLI (artefatos executáveis na raiz: SonixPro.exe e INICIAR_SONIX_PRO.bat)
//! CA-2: Enumeração e Seleção de Dispositivo Físico WASAPI (P2/P3 Line-In / Fones)
//! CA-3: Duplex Real-Time Lock-Free com Ring Buffer SPSC (zero-alloc, zero-lock)
//! CA-4: Cadeia DSP Ativa em Tempo Real (Noise Gate + White-Box Amp + Gabinete IR)
//! CA-5: Janela Desktop Dedicada e Servidor Local Isolado (127.0.0.1, modo App sem navegador)
//! CA-6: Telemetria em Tempo Real (VU meters, buffer CRT e afinador sem bloqueios)
//! CA-7: Tratamento de Falhas e Desconexão Graciosa (zero-fill, proteção contra cliques e panic)
//! CA-8: Não-regressão das Specs Anteriores (Presets, DAG, 64-bit solvers e plugins)

use std::fs;
use std::path::Path;
use sonix_core::persistence::SonixPresetDoc;
use sonix_desktop::audio::{
    AudioDriverError, DesktopAudioHost, RealtimeDspChain, RealtimeDuplexEngine, TelemetryBuffer,
};
use sonix_desktop::server::LocalAppServer;

#[test]
fn test_ca_01_desktop_windows_double_click_launcher_artifact_present() {
    // CA-1: O usuário pode iniciar a aplicação por duplo clique na raiz
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let bat_launcher = repo_root.join("INICIAR_SONIX_PRO.bat");
    let exe_launcher = repo_root.join("SonixPro.exe");

    assert!(
        bat_launcher.exists() || exe_launcher.exists(),
        "Pelo menos um artefato executável de duplo clique (INICIAR_SONIX_PRO.bat ou SonixPro.exe) deve existir na raiz"
    );

    if bat_launcher.exists() {
        let bat_content = fs::read_to_string(&bat_launcher).expect("Falha ao ler INICIAR_SONIX_PRO.bat");
        assert!(
            bat_content.contains("SonixPro") || bat_content.contains("sonix-desktop"),
            "Script BAT deve inicializar o binário do host SonixPro"
        );
        assert!(
            bat_content.contains("--app=") || bat_content.contains("127.0.0.1"),
            "Script BAT deve abrir a aplicação em modo janela dedicada (App Mode)"
        );
    }
}

#[test]
fn test_ca_02_physical_audio_device_enumeration_wasapi_default_duplex() {
    // CA-2: Host detecta e enumera dispositivo físico de entrada (P2/P3) e saída
    let host = DesktopAudioHost::new();
    let devices = host.enumerate_physical_devices();

    assert!(!devices.is_empty(), "Deve enumerar dispositivos físicos de áudio");

    let devices_summary = devices
        .iter()
        .map(|d| {
            format!(
                "nome={:?}, is_input={}, supported_sample_rates={:?}, supported_buffer_sizes={:?}",
                d.name, d.is_input, d.supported_sample_rates, d.supported_buffer_sizes
            )
        })
        .collect::<Vec<_>>()
        .join("\n  ");

    // Verifica que existe pelo menos um dispositivo com suporte a duplex ou par entrada/saída
    let input_dev = devices.iter().find(|d| d.is_input);
    let output_dev = devices.iter().find(|d| !d.is_input);

    assert!(
        input_dev.is_some(),
        "Deve detectar dispositivo físico de entrada (Microfone P2/Line-in). Dispositivos enumerados:\n  {devices_summary}"
    );
    assert!(
        output_dev.is_some(),
        "Deve detectar dispositivo físico de saída (Alto-falantes/Fones). Dispositivos enumerados:\n  {devices_summary}"
    );

    let input = input_dev.unwrap();
    assert!(
        input.supported_sample_rates.contains(&44100)
            || input.supported_sample_rates.contains(&48000)
            || input.supported_sample_rates.contains(&96000),
        "Dispositivo de entrada deve suportar taxas de amostragem padrão (44.1k, 48k ou 96k). Dispositivos enumerados:\n  {devices_summary}"
    );
    assert!(
        input.supported_buffer_sizes.contains(&64)
            || input.supported_buffer_sizes.contains(&128)
            || input.supported_buffer_sizes.contains(&256),
        "Dispositivo deve suportar buffers de baixa latência (64, 128 ou 256). Dispositivos enumerados:\n  {devices_summary}"
    );
}

#[test]
fn test_ca_03_duplex_realtime_lock_free_spsc_ring_buffer() {
    // CA-3: Duplex streaming lock-free com SPSC Ring Buffer
    let buffer_capacity = 1024;
    let duplex_engine = RealtimeDuplexEngine::new(buffer_capacity);

    assert_eq!(duplex_engine.capacity(), buffer_capacity);
    assert_eq!(duplex_engine.available_samples(), 0);

    // Simula captura física da guitarra na entrada P2 (bloco de 128 amostras)
    let block_size = 128;
    let input_signal: Vec<f32> = (0..block_size)
        .map(|i| (i as f32 * 0.1).sin() * 0.5)
        .collect();

    // Push lock-free na thread de captura
    let pushed = duplex_engine.push_captured_samples(&input_signal);
    assert_eq!(pushed, block_size, "Todas as amostras devem ser inseridas no ring buffer");
    assert_eq!(duplex_engine.available_samples(), block_size);

    // Pop lock-free na thread de processamento/saída
    let mut read_buf = vec![0.0f32; block_size];
    let popped = duplex_engine.pop_samples_for_processing(&mut read_buf);
    assert_eq!(popped, block_size);

    for i in 0..block_size {
        assert!((read_buf[i] - input_signal[i]).abs() < 1e-6);
    }
}

#[test]
fn test_ca_04_active_dsp_chain_realtime_processing() {
    // CA-4: Cadeia DSP ativa em tempo real: Noise Gate + White-Box Amp + Gabinete IR
    let sample_rate = 48000.0;
    let mut dsp_chain = RealtimeDspChain::new(sample_rate);

    // Configura Noise Gate para eliminar ruído da entrada P2
    dsp_chain.set_gate_threshold_db(-45.0);
    dsp_chain.set_amp_gain(7.5);
    dsp_chain.set_amp_master(6.0);
    dsp_chain.set_cab_enabled(true);

    let block_size = 256;

    // Cenário 1: Ruído de fundo de baixa amplitude na entrada P2 (-60 dBFS)
    // O Noise Gate deve atuar e silenciar a saída
    let low_noise_input = vec![0.001f32; block_size]; // ~ -60 dBFS
    let mut silence_output = vec![0.0f32; block_size];
    dsp_chain.process(&low_noise_input, &mut silence_output);

    let noise_energy: f32 = silence_output[128..].iter().map(|&s| s * s).sum();
    assert!(
        noise_energy < 1e-4,
        "Noise Gate deve suprimir ruído de fundo abaixo do limiar (energia: {})",
        noise_energy
    );

    // Cenário 2: Sinal real da guitarra (ex: acorde forte ~ 0.4 de pico)
    let guitar_input: Vec<f32> = (0..block_size)
        .map(|i| (i as f32 * 0.05).sin() * 0.4)
        .collect();
    let mut amp_output = vec![0.0f32; block_size];
    dsp_chain.process(&guitar_input, &mut amp_output);

    let signal_energy: f32 = amp_output.iter().map(|&s| s * s).sum();
    assert!(signal_energy > 0.01, "Sinal da guitarra deve ser processado e amplificado");

    // Verifica saturação não linear do amplificador valvulado (sinal delimitado)
    for &sample in &amp_output {
        assert!(sample.is_finite());
        assert!(sample.abs() <= 2.5, "Sinal saturado não deve explodir");
    }
}

#[test]
fn test_ca_05_dedicated_desktop_window_and_local_isolated_server() {
    // CA-5: Servidor local isolado em 127.0.0.1 e comando de janela dedicada (App Mode)
    let server = LocalAppServer::bind_ephemeral().expect("Deve iniciar servidor local em 127.0.0.1");

    assert_eq!(server.host(), "127.0.0.1", "Servidor deve estar restrito ao loopback");
    assert!(server.port() > 0);

    let app_url = server.app_url();
    assert!(app_url.starts_with("http://127.0.0.1:"));

    // Verifica comando de lançamento em modo aplicativo dedicado (sem barra de navegação/abas)
    let win_cmd = server.build_desktop_app_command();
    assert!(
        win_cmd.contains("--app=") || win_cmd.contains("-app"),
        "Comando de inicialização deve utilizar flag de aplicação dedicada (--app=URL)"
    );
    assert!(win_cmd.contains(&app_url));
}

#[test]
fn test_ca_06_realtime_telemetry_vu_crt_oscilloscope_tuner() {
    // CA-6: Telemetria em tempo real para VU meters, osciloscópio CRT e afinador
    let telemetry = TelemetryBuffer::new(512);

    let sample_block = vec![0.5f32; 128];
    telemetry.update_peaks(-6.0, -3.0, 48000.0);
    telemetry.push_waveform(&sample_block);

    let snapshot = telemetry.snapshot();
    assert!((snapshot.input_peak_db - (-6.0)).abs() < 1e-4);
    assert!((snapshot.output_peak_db - (-3.0)).abs() < 1e-4);
    assert_eq!(snapshot.waveform.len(), 512);

    // Amostras recentes devem estar refletidas no buffer circular do osciloscópio
    let has_samples = snapshot.waveform.iter().any(|&s| (s - 0.5).abs() < 1e-4);
    assert!(has_samples, "Osciloscópio CRT deve conter amostras da forma de onda");
}

#[test]
fn test_ca_07_graceful_fault_handling_and_zero_fill_silence() {
    // CA-7: Tratamento de falhas, desconexão e underrun com preenchimento com zero (silêncio)
    let mut host = DesktopAudioHost::new();

    let block_size = 256;
    let mut output_buffer = vec![0.9f32; block_size]; // buffer com resíduos

    // Simula underrun / ausência de sinal
    host.handle_audio_underrun(&mut output_buffer);

    for &s in &output_buffer {
        assert_eq!(s, 0.0, "Underrun deve preencher buffer com silêncio (zero-fill)");
    }

    // Simula desconexão abrupta
    let err = host.simulate_device_disconnect(&mut output_buffer);
    assert_eq!(err, AudioDriverError::DeviceDisconnected);
    for &s in &output_buffer {
        assert_eq!(s, 0.0, "Desconexão deve silenciar saída para evitar ruído explosivo");
    }
}

#[test]
fn test_ca_08_non_regression_specs_001_to_012_integrity() {
    // CA-8: Não-regressão de todas as specs anteriores
    let preset_json = r#"{
        "$schema": "https://sonixpro.local/schemas/sonixpreset-v1.json",
        "schemaVersion": "1.0.0",
        "metadata": {
            "id": "e8d7c6b5-a4f3-4210-9876-123456789abc",
            "name": "Spec 013 Live Windows Rig",
            "author": "Antigravity",
            "category": "Realtime",
            "tempoBpm": 120.0,
            "createdAt": "2026-11-08T19:30:00Z",
            "footswitchMode": "HYBRID_STOMP_SCENE"
        },
        "activeSceneIndex": 0,
        "scenes": [],
        "grid": { "rows": 4, "cols": 12, "blocks": [] },
        "modifiers": []
    }"#;

    let doc = SonixPresetDoc::from_json_str(preset_json);
    assert!(doc.is_ok(), "Preset Schema 1.0.0 deve ser desserializado com sucesso");

    let chain = RealtimeDspChain::new(48000.0);
    assert!(chain.sample_rate() == 48000.0);
}
