//! crates/ampforge-dsp/tests/contract_008_tuner_looper.rs
//! Testes de Contrato Automatizados para Spec 008: Afinador Estroboscópico/Polifônico e Looper
//! Cobre Critérios de Aceite CA-1 a CA-9

use ampforge_dsp::looper::{LooperPosition, LooperProcessor, LooperState};
use ampforge_dsp::tuner::{
    TunerMode, TunerOutputMode, TunerProcessor, TunerTemperament,
};

#[test]
fn test_ca_01_tuner_toggle_and_long_press_activation() {
    let mut tuner = TunerProcessor::new();
    assert!(!tuner.is_active());

    // Toggle by click
    tuner.toggle();
    assert!(tuner.is_active());

    tuner.toggle();
    assert!(!tuner.is_active());

    // Long press activation (hold >= 500 ms)
    tuner.on_footswitch_event(true, 550);
    assert!(tuner.is_active());

    // Short press does not activate if long-press required
    tuner.set_active(false);
    tuner.on_footswitch_event(true, 150);
    assert!(!tuner.is_active());
}

#[test]
fn test_ca_02_strobe_pitch_detection_accuracy_within_0_1_cent() {
    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(48000.0);
    tuner.set_a4_reference_hz(440.0);

    let n = 4096;
    let sr = 48000.0;

    // 1. Exact 440.0 Hz (A4, 0.0 cents)
    let sig_440: Vec<f32> = (0..n)
        .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / sr)).sin())
        .collect();
    let res_440 = tuner.detect_pitch(&sig_440);
    assert_eq!(res_440.note_name, "A4");
    assert!(
        res_440.cents_deviation.abs() <= 0.1,
        "Error for 440Hz must be within +-0.1 cent, got {}",
        res_440.cents_deviation
    );

    // 2. 440.254 Hz (+1.0 cent deviation: 440 * 2^(1/1200) = 440.254)
    let f_plus_1cent = 440.0 * 2.0f32.powf(1.0 / 1200.0);
    let sig_plus_1: Vec<f32> = (0..n)
        .map(|i| 0.5 * (2.0 * std::f32::consts::PI * f_plus_1cent * (i as f32 / sr)).sin())
        .collect();
    let res_plus_1 = tuner.detect_pitch(&sig_plus_1);
    assert_eq!(res_plus_1.note_name, "A4");
    assert!(
        (res_plus_1.cents_deviation - 1.0).abs() <= 0.1,
        "Error for +1.0 cent must be within +-0.1 cent (got {})",
        res_plus_1.cents_deviation
    );
}

#[test]
fn test_ca_03_tuner_a4_calibration_polyphonic_mode_and_temperament() {
    let mut tuner = TunerProcessor::new();
    tuner.set_a4_reference_hz(442.0);
    assert_eq!(tuner.a4_reference_hz(), 442.0);

    tuner.set_mode(TunerMode::Polyphonic);
    assert_eq!(tuner.mode(), TunerMode::Polyphonic);

    tuner.set_temperament(TunerTemperament::SweetenedGuitar);
    assert_eq!(tuner.temperament(), TunerTemperament::SweetenedGuitar);

    // Verify 6-string polyphonic detection struct
    let poly_status = tuner.detect_polyphonic_status();
    assert_eq!(poly_status.len(), 6);
    assert_eq!(poly_status[0].note, "E2");
    assert_eq!(poly_status[5].note, "E4");
}

#[test]
fn test_ca_04_tuner_mute_vs_bypass_audio_routing() {
    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(48000.0);
    tuner.set_active(true);

    let n = 512;
    let in_sig = vec![0.5f32; n];
    let mut out_sig = vec![0.0f32; n];

    // Mute mode (default)
    tuner.set_output_mode(TunerOutputMode::Mute);
    tuner.process_audio(&in_sig, &mut out_sig);
    let energy_muted: f32 = out_sig[256..].iter().map(|s| s * s).sum();
    assert!(energy_muted < 1e-6, "Tuner in Mute mode must silence audio output");

    // Bypass mode
    tuner.set_output_mode(TunerOutputMode::Bypass);
    tuner.process_audio(&in_sig, &mut out_sig);
    let energy_bypass: f32 = out_sig[256..].iter().map(|s| s * s).sum();
    assert!(energy_bypass > 0.0, "Tuner in Bypass mode must let audio sound");
}

#[test]
fn test_ca_05_looper_state_machine_undo_redo_and_progress_pct() {
    let mut looper = LooperProcessor::new();
    looper.set_sample_rate(48000.0);
    assert_eq!(looper.state(), LooperState::Idle);

    // 1. RECORD base layer
    looper.record();
    assert_eq!(looper.state(), LooperState::Record);
    let block = vec![0.5f32; 256];
    let mut dummy_out = vec![0.0f32; 256];
    looper.process(&block, &mut dummy_out);

    // 2. PLAY loop
    looper.play();
    assert_eq!(looper.state(), LooperState::Play);
    assert_eq!(looper.loop_length_samples(), 256);

    // 3. OVERDUB
    looper.overdub();
    assert_eq!(looper.state(), LooperState::Overdub);
    let overdub_block = vec![0.2f32; 256];
    looper.process(&overdub_block, &mut dummy_out);

    // 4. Return to PLAY
    looper.play();
    assert_eq!(looper.state(), LooperState::Play);

    // Progress percentage
    let prog = looper.progress_pct();
    assert!((0.0..=100.0).contains(&prog));

    // 5. UNDO removes overdub
    assert!(looper.can_undo());
    looper.undo();
    assert!(!looper.can_undo());
    assert!(looper.can_redo());

    // 6. REDO restores overdub
    looper.redo();
    assert!(looper.can_undo());

    // 7. STOP
    looper.stop();
    assert_eq!(looper.state(), LooperState::Stop);
}

#[test]
fn test_ca_06_looper_reverse_and_half_speed() {
    let mut looper = LooperProcessor::new();
    looper.set_sample_rate(48000.0);

    // Record an ascending ramp
    looper.record();
    let ramp: Vec<f32> = (0..100).map(|i| i as f32 / 100.0).collect();
    let mut out = vec![0.0f32; 100];
    looper.process(&ramp, &mut out);
    looper.play();

    // Enable Reverse
    looper.set_reverse(true);
    assert!(looper.is_reverse());

    // Enable Half-Speed
    looper.set_half_speed(true);
    assert!(looper.is_half_speed());
}

#[test]
fn test_ca_07_looper_pre_chain_vs_post_chain_positioning() {
    let mut looper = LooperProcessor::new();
    looper.set_position(LooperPosition::PreChain);
    assert_eq!(looper.position(), LooperPosition::PreChain);

    looper.set_position(LooperPosition::PostChain);
    assert_eq!(looper.position(), LooperPosition::PostChain);
}

#[test]
fn test_ca_08_error_handling_and_auto_record_buffer_limit_switch() {
    let mut tuner = TunerProcessor::new();
    tuner.set_a4_reference_hz(400.0);
    assert_eq!(tuner.a4_reference_hz(), 430.0); // clamped to min
    tuner.set_a4_reference_hz(600.0);
    assert_eq!(tuner.a4_reference_hz(), 450.0); // clamped to max

    // Looper auto-switch to Play when max buffer reached
    let mut looper = LooperProcessor::new_with_max_capacity(512);
    looper.record();
    let big_block = vec![0.1f32; 600]; // larger than 512
    let mut out = vec![0.0f32; 600];
    looper.process(&big_block, &mut out);

    // Must automatically switch from Record to Play without panic
    assert_eq!(looper.state(), LooperState::Play);
    assert_eq!(looper.loop_length_samples(), 512);
}

#[test]
fn test_ca_09_non_regression_idle_tuner_and_looper_transparency() {
    let tuner = TunerProcessor::new();
    let mut looper = LooperProcessor::new();

    assert!(!tuner.is_active());
    assert_eq!(looper.state(), LooperState::Idle);

    let n = 256;
    let in_sig = vec![0.75f32; n];
    let mut out_sig = vec![0.0f32; n];
    looper.process(&in_sig, &mut out_sig);

    // In Idle, looper passes dry input untouched (0 latency, identical amplitude)
    for i in 0..n {
        assert_eq!(out_sig[i], in_sig[i]);
    }
}
