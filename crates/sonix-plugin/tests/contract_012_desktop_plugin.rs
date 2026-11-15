//! crates/sonix-desktop/tests/contract_012_desktop_plugin.rs
//! Testes de Contrato Automatizados para Spec 012: Host Desktop Standalone e Plugins VST3/CLAP
//! Cobre Critérios de Aceite CA-1 a CA-8

use ampforge_dsp::queue::SpscCommandQueue;
use sonix_core::persistence::SonixPresetDoc;
use sonix_desktop::audio::{
    AudioBackendType, AudioDeviceEnumerator, AudioDriverError, DesktopAudioDevice,
    DesktopAudioHost,
};
use sonix_desktop::midi::{DesktopMidiHost, MidiMessageType, MidiPortEnumerator};
use sonix_plugin::{PluginFormat, SonixPluginProcessor};
use std::sync::Arc;

struct TestAudioEnumerator;
impl AudioDeviceEnumerator for TestAudioEnumerator {
    fn enumerate_devices(&self) -> Vec<DesktopAudioDevice> {
        vec![DesktopAudioDevice {
            id: "test_dev".to_string(),
            name: "Studio Test Audio Device".to_string(),
            backend: AudioBackendType::Wasapi,
            is_input: true,
            supported_sample_rates: vec![44100, 48000, 96000],
            supported_buffer_sizes: vec![64, 128, 256],
        }]
    }
}

struct TestMidiEnumerator;
impl MidiPortEnumerator for TestMidiEnumerator {
    fn enumerate_input_ports(&self) -> Vec<String> {
        vec!["Test MIDI Port".to_string()]
    }
}

#[test]
fn test_ca_01_desktop_audio_host_device_enumeration_and_stream_init() {
    let mut host = DesktopAudioHost::with_enumerator(Arc::new(TestAudioEnumerator));
    let devices = host.enumerate_devices();

    assert!(!devices.is_empty(), "Audio host must enumerate at least 1 audio device");

    let default_dev = &devices[0];
    assert!(
        default_dev.supported_sample_rates.contains(&44100)
            || default_dev.supported_sample_rates.contains(&48000)
            || default_dev.supported_sample_rates.contains(&96000),
        "Device must support 44.1k, 48k or 96k"
    );
    assert!(default_dev.supported_buffer_sizes.contains(&128) || default_dev.supported_buffer_sizes.contains(&256));

    let stream_res = host.start_stream(default_dev, 48000, 128);
    assert!(stream_res.is_ok(), "Must successfully start audio stream");
    assert!(host.is_stream_active());
}

#[test]
fn test_ca_02_desktop_midi_input_enumeration_and_cc_queue_forwarding() {
    let mut midi_host = DesktopMidiHost::with_enumerator(Arc::new(TestMidiEnumerator));
    let ports = midi_host.enumerate_input_ports();
    assert!(!ports.is_empty(), "MIDI host must enumerate input ports");

    let connect_res = midi_host.connect_port(&ports[0]);
    assert!(connect_res.is_ok());

    let (mut command_producer, command_consumer) = SpscCommandQueue::new();

    // Send MIDI CC 7 (Volume) val 100
    let midi_cc_msg = [0xB0, 0x07, 0x64];
    let parsed = midi_host.process_raw_midi_message(&midi_cc_msg, &mut command_producer);
    assert!(matches!(parsed, Ok(MidiMessageType::ControlChange { controller: 7, value: 100 })));

    // Verify command was pushed to queue lock-free
    assert!(command_consumer.pop().is_some());
}

#[test]
fn test_ca_03_vst3_and_clap_plugin_descriptors_and_64bit_processing() {
    let formats = [PluginFormat::Vst3, PluginFormat::Clap];

    for fmt in formats {
        let mut plugin = SonixPluginProcessor::new(fmt);
        let desc = plugin.descriptor();

        assert_eq!(desc.name, "SONIX PRO // AMPFORGE™");
        assert_eq!(desc.vendor, "SonixPro Audio");
        assert_eq!(desc.version, "1.0.0");
        assert!(desc.is_64bit, "Plugin must be 64-bit");
        assert_eq!(desc.audio_in_channels, 2);
        assert_eq!(desc.audio_out_channels, 2);

        // Process a stereo block
        let n = 256;
        let in_l = vec![0.5f32; n];
        let in_r = vec![0.5f32; n];
        let mut out_l = vec![0.0f32; n];
        let mut out_r = vec![0.0f32; n];

        plugin.process(&in_l, &in_r, &mut out_l, &mut out_r);
        assert_eq!(out_l.len(), n);
        assert_eq!(out_r.len(), n);
    }
}

#[test]
fn test_ca_04_daw_state_save_and_restore_lossless_matching_sonixpreset() {
    let mut plugin = SonixPluginProcessor::new(PluginFormat::Vst3);
    plugin.set_preset_name("Plugin Lead Rig");
    plugin.set_tempo_bpm(135.0);

    // Save DAW state
    let state_bytes = plugin.save_state().unwrap();
    assert!(!state_bytes.is_empty());

    // Restore state in new instance
    let mut restored_plugin = SonixPluginProcessor::new(PluginFormat::Vst3);
    let restore_res = restored_plugin.restore_state(&state_bytes);
    assert!(restore_res.is_ok());

    assert_eq!(restored_plugin.preset_name(), "Plugin Lead Rig");
    assert_eq!(restored_plugin.tempo_bpm(), 135.0);
}

#[test]
fn test_ca_05_daw_transport_tempo_sync() {
    let mut plugin = SonixPluginProcessor::new(PluginFormat::Clap);
    plugin.set_host_tempo_sync(true);

    // DAW changes tempo from 120.0 to 140.0 BPM
    plugin.on_transport_update(140.0, true);
    assert_eq!(plugin.tempo_bpm(), 140.0);
}

#[test]
fn test_ca_06_desktop_and_plugin_native_webview_ipc_zero_network_server() {
    let host = DesktopAudioHost::new();
    assert!(host.is_local_ipc_only(), "Must use local IPC without opening network HTTP server");
}

#[test]
fn test_ca_07_audio_device_disconnect_safe_handling_without_crash() {
    let mut host = DesktopAudioHost::with_enumerator(Arc::new(TestAudioEnumerator));
    let dev = &host.enumerate_devices()[0];
    host.start_stream(dev, 48000, 128).unwrap();

    let n = 128;
    let mut out_buffer = vec![0.7f32; n];

    // Simulate abrupt device disconnect
    let err = host.simulate_device_disconnect(&mut out_buffer);
    assert!(matches!(err, AudioDriverError::DeviceDisconnected));

    // Must zero output buffer cleanly without explosive loud noise
    for i in 0..n {
        assert_eq!(out_buffer[i], 0.0);
    }
    assert!(!host.is_stream_active());
}

#[test]
fn test_ca_08_non_regression_preset_interoperability_between_standalone_and_plugins() {
    let standalone_json = r#"{
        "$schema": "https://sonixpro.local/schemas/sonixpreset-v1.json",
        "schemaVersion": "1.0.0",
        "metadata": {
            "id": "c1f88eb2-6902-4217-bf41-6fbce3aa93d0",
            "name": "Shared Cross-Platform Rig",
            "author": "The Unforgiven",
            "category": "High Gain",
            "tempoBpm": 124.0,
            "createdAt": "2026-11-08T18:30:00Z",
            "footswitchMode": "HYBRID_STOMP_SCENE"
        },
        "activeSceneIndex": 0,
        "scenes": [],
        "grid": { "rows": 4, "cols": 12, "blocks": [] },
        "modifiers": []
    }"#;

    // Load into Standalone
    let doc_standalone = SonixPresetDoc::from_json_str(standalone_json).unwrap();

    // Load into VST3 Plugin
    let mut vst3_plugin = SonixPluginProcessor::new(PluginFormat::Vst3);
    assert!(vst3_plugin.load_sonixpreset_doc(&doc_standalone).is_ok());
    assert_eq!(vst3_plugin.preset_name(), "Shared Cross-Platform Rig");

    // Load into CLAP Plugin
    let mut clap_plugin = SonixPluginProcessor::new(PluginFormat::Clap);
    assert!(clap_plugin.load_sonixpreset_doc(&doc_standalone).is_ok());
    assert_eq!(clap_plugin.preset_name(), "Shared Cross-Platform Rig");
}
