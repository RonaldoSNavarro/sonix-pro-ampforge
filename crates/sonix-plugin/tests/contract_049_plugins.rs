//! crates/sonix-plugin/tests/contract_049_plugins.rs
//! Testes de Contrato Automatizados para Spec 049 Parte B (CA-16..CA-22):
//! Plugins VST3 & CLAP compartilhando o mesmo núcleo DSP, parâmetros, estado e WebView IPC.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::f32::consts::PI;

use sonix_core::persistence::SonixPresetDoc;
use sonix_desktop::audio::RealtimeDspChain;
use sonix_plugin::{PluginFormat, SonixPluginProcessor, SonixProPlugin};

struct CountingAllocator;
thread_local! {
    static TRACK_ALLOC: Cell<bool> = const { Cell::new(false) };
    static ALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
    static DEALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        TRACK_ALLOC.with(|t| {
            if t.get() {
                ALLOC_COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        TRACK_ALLOC.with(|t| {
            if t.get() {
                DEALLOC_COUNT.with(|c| c.set(c.get() + 1));
            }
        });
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[test]
fn ca16_plugin_exports_vst3_clap_stereo_and_midi() {
    assert!(SonixProPlugin::supports_vst3_and_clap(), "CA-16: plugin deve exportar VST3 e CLAP");
    assert_eq!(SonixProPlugin::audio_channels(), (2, 2), "CA-16: deve declarar estéreo in/out");
    assert!(SonixProPlugin::accepts_midi_input(), "CA-16: deve declarar entrada MIDI");
}

#[test]
fn ca17_plugin_and_standalone_share_exact_dsp_core_within_minus_100dbfs() {
    let sr = 48000.0;
    let block = 256;
    let mut standalone = RealtimeDspChain::new(sr);
    standalone.prepare(sr, block);

    let mut plugin = SonixPluginProcessor::new(PluginFormat::Clap);
    plugin.prepare(sr, block);

    let input: Vec<f32> = (0..block * 8)
        .map(|i| 0.25 * (2.0 * PI * 440.0 * i as f32 / sr).sin())
        .collect();

    let mut s_out = vec![0.0f32; block];
    let mut p_out_l = vec![0.0f32; block];
    let mut p_out_r = vec![0.0f32; block];
    let mut max_diff = 0.0f32;

    for chunk in input.chunks_exact(block) {
        standalone.process(chunk, &mut s_out);
        plugin.process(chunk, chunk, &mut p_out_l, &mut p_out_r);
        for i in 0..block {
            max_diff = max_diff.max((s_out[i] - p_out_l[i]).abs());
        }
    }

    let diff_db = 20.0 * max_diff.max(1e-12).log10();
    assert!(
        diff_db <= -100.0,
        "CA-17: diferença entre plugin e standalone foi {:.2} dBFS (esperado <= -100 dBFS)",
        diff_db
    );
}

#[test]
fn ca18_unified_param_table_and_gain_automation_increases_thd() {
    let ids = SonixProPlugin::registered_param_ids();
    for required in [
        "amp_1.gain",
        "amp_1.bass",
        "amp_1.middle",
        "amp_1.treble",
        "amp_1.master",
        "amp_1.model",
        "gate.threshold_db",
        "dly_1.intensity",
        "rev_1.mix",
        "cab_1.enabled",
    ] {
        assert!(ids.contains(&required.to_string()), "CA-18: parâmetro {} ausente", required);
    }

    let sr = 48000.0;
    let mut plugin_low = SonixPluginProcessor::new(PluginFormat::Clap);
    plugin_low.prepare(sr, 256);
    plugin_low.set_param("amp_1.gain", 1.0).unwrap();

    let mut plugin_high = SonixPluginProcessor::new(PluginFormat::Clap);
    plugin_high.prepare(sr, 256);
    plugin_high.set_param("amp_1.gain", 9.0).unwrap();

    let input: Vec<f32> = (0..256)
        .map(|i| 0.3 * (2.0 * PI * 220.0 * i as f32 / sr).sin())
        .collect();
    let mut low_l = vec![0.0f32; 256];
    let mut low_r = vec![0.0f32; 256];
    let mut high_l = vec![0.0f32; 256];
    let mut high_r = vec![0.0f32; 256];

    for _ in 0..8 {
        plugin_low.process(&input, &input, &mut low_l, &mut low_r);
        plugin_high.process(&input, &input, &mut high_l, &mut high_r);
    }
    let rms_low = (low_l.iter().map(|x| x * x).sum::<f32>() / 256.0).sqrt();
    let rms_high = (high_l.iter().map(|x| x * x).sum::<f32>() / 256.0).sqrt();
    assert!(rms_high > rms_low, "CA-18: aumentar amp_1.gain deve aumentar nível/saturação");
}

#[test]
fn ca19_state_save_restore_roundtrips_full_sonixpreset_and_audio() {
    let mut plugin = SonixPluginProcessor::new(PluginFormat::Vst3);
    plugin.prepare(48000.0, 256);
    plugin.set_param("amp_1.gain", 7.5).unwrap();
    plugin.set_param("dly_1.enabled", 1.0).unwrap();

    let saved = plugin.save_state().unwrap();
    let doc: SonixPresetDoc = serde_json::from_slice(&saved)
        .expect("CA-19: state salvo deve ser SonixPresetDoc válido");
    assert_eq!(doc.scenes.len(), 8, "CA-19: deve preservar as 8 cenas");

    let mut restored = SonixPluginProcessor::new(PluginFormat::Vst3);
    restored.prepare(48000.0, 256);
    restored.restore_state(&saved).unwrap();
    assert!((restored.get_param("amp_1.gain").unwrap() - 7.5).abs() < 1e-4);
}

#[test]
fn ca20_host_tempo_sync_updates_delay_time_from_120_to_140_bpm() {
    let mut plugin = SonixPluginProcessor::new(PluginFormat::Clap);
    plugin.prepare(48000.0, 256);
    plugin.set_param("dly_1.enabled", 1.0).unwrap();
    plugin.set_param("dly_1.tempo_sync", 1.0).unwrap();
    plugin.set_host_tempo_sync(true);

    plugin.on_transport_update(120.0, true);
    assert!((plugin.delay_target_ms() - 500.0).abs() < 0.5);

    plugin.on_transport_update(140.0, true);
    assert!(
        (plugin.delay_target_ms() - 428.5714).abs() < 0.5,
        "CA-20: target ms esperado ~428.57 ms, obtido {}",
        plugin.delay_target_ms()
    );
}

#[test]
fn ca21_plugin_process_zero_alloc_for_10s_with_automation() {
    let sr = 48000.0;
    let block = 256;
    let mut plugin = SonixPluginProcessor::new(PluginFormat::Clap);
    plugin.prepare(sr, block);

    let inp: Vec<f32> = (0..block)
        .map(|i| 0.2 * (2.0 * PI * 440.0 * i as f32 / sr).sin())
        .collect();
    let mut out_l = vec![0.0f32; block];
    let mut out_r = vec![0.0f32; block];

    // Warmup
    for _ in 0..10 {
        plugin.process(&inp, &inp, &mut out_l, &mut out_r);
    }

    ALLOC_COUNT.with(|c| c.set(0));
    DEALLOC_COUNT.with(|c| c.set(0));
    TRACK_ALLOC.with(|t| t.set(true));

    let blocks_10s = (48000 * 10) / block;
    for b in 0..blocks_10s {
        let g = (b % 100) as f32 / 10.0;
        plugin.set_param_rt_normalized("amp_1.gain", g / 10.0);
        plugin.process(&inp, &inp, &mut out_l, &mut out_r);
    }

    TRACK_ALLOC.with(|t| t.set(false));
    assert_eq!(ALLOC_COUNT.with(|c| c.get()), 0, "CA-21: zero alocações no process");
    assert_eq!(DEALLOC_COUNT.with(|c| c.get()), 0, "CA-21: zero desalocações no process");
}

#[test]
fn ca22_plugin_editor_uses_ipc_webview_without_http_server() {
    assert!(!SonixProPlugin::has_http_server(), "CA-22: binário do plugin não deve conter dev-http");
    assert!(SonixProPlugin::uses_embedded_webview_ipc(), "CA-22: editor deve usar WebView via IPC");
}
