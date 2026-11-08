//! crates/ampforge-dsp/tests/contract_007_mod_pitch_delay_reverb.rs
//! Testes de Contrato Automatizados para Spec 007: Modulação, Pitch, Delay e Reverb
//! Cobre Critérios de Aceite CA-1 a CA-9

use ampforge_dsp::fx::{
    DelayModel, DelayProcessor, ModModel, ModulationProcessor, NoteDivision, PitchKey, PitchModel,
    PitchProcessor, ReverbModel, ReverbProcessor, ScaleType,
};

#[test]
fn test_ca_01_mod_7_models_and_bpm_tempo_sync_frequency() {
    let mut proc = ModulationProcessor::new(ModModel::AnalogBbdChorus);
    proc.set_sample_rate(48000.0);

    let models = [
        ModModel::AnalogBbdChorus,
        ModModel::StereoTriChorus,
        ModModel::ThroughZeroFlanger,
        ModModel::Orange90Phaser,
        ModModel::OpticalBiasTremolo,
        ModModel::PhotocellUnivibe,
        ModModel::RotarySpeaker,
    ];

    for m in models {
        proc.set_model(m);
        assert_eq!(proc.model(), m);
    }

    // BPM Sync at 120 BPM
    proc.set_tempo_sync(true);
    proc.set_tempo_bpm(120.0);

    // 1/4 note at 120 BPM should be exactly 2.0 Hz
    proc.set_note_division(NoteDivision::Quarter);
    assert!((proc.calculated_lfo_freq_hz() - 2.0).abs() < 0.001);

    // 1/8D (dotted eighth) at 120 BPM: period is 0.75 * 0.5s = 0.375s -> freq = 1 / 0.375 = 2.6667 Hz
    proc.set_note_division(NoteDivision::EighthDotted);
    assert!((proc.calculated_lfo_freq_hz() - 2.6667).abs() < 0.01);
}

#[test]
fn test_ca_02_polyphonic_drop_capo_transposition_ratio() {
    let mut pitch = PitchProcessor::new(PitchModel::PolyphonicDropCapo);
    pitch.set_semitones(-2.0); // Drop 2 semitones (e.g. E standard to D standard)

    let expected_ratio = 2.0f32.powf(-2.0 / 12.0);
    assert!(
        (pitch.transposition_ratio() - expected_ratio).abs() < 0.0001,
        "Drop -2 st ratio must be ~0.890899 (got {})",
        pitch.transposition_ratio()
    );
}

#[test]
fn test_ca_03_intelligent_dual_harmonizer_c_major_diatonic_thirds() {
    let mut pitch = PitchProcessor::new(PitchModel::IntelligentDualHarmonizer);
    pitch.set_key(PitchKey::C);
    pitch.set_scale(ScaleType::Major);

    // C4 (261.63 Hz) in C major: diatonic 3rd is E4 (+4 semitones, 329.63 Hz)
    let harm_c4 = pitch.calculate_harmonic_frequency(261.63);
    assert!(
        (harm_c4 - 329.63).abs() < 1.0,
        "Harmonizer for C4 in C major must synthesize E4 (~329.63 Hz), got {}",
        harm_c4
    );

    // D4 (293.66 Hz) in C major: diatonic 3rd is F4 (+3 semitones, 349.23 Hz)
    let harm_d4 = pitch.calculate_harmonic_frequency(293.66);
    assert!(
        (harm_d4 - 349.23).abs() < 1.0,
        "Harmonizer for D4 in C major must synthesize F4 (~349.23 Hz), got {}",
        harm_d4
    );
}

#[test]
fn test_ca_04_tape_echo_self_oscillation_soft_saturation() {
    let mut delay = DelayProcessor::new(DelayModel::MultiHeadTapeEcho);
    delay.set_sample_rate(48000.0);
    delay.set_delay_time_ms(50.0);
    delay.set_feedback_pct(120.0); // Self-oscillation > 100%

    let n = 2048;
    let mut in_sig = vec![0.0f32; n];
    in_sig[0] = 0.9; // Initial impulse to trigger loop

    let mut out_sig = vec![0.0f32; n];
    delay.process(&in_sig, &mut out_sig);

    for (i, &s) in out_sig.iter().enumerate() {
        assert!(
            !s.is_nan() && !s.is_infinite(),
            "Output at sample {} became NaN or Inf in self-oscillation",
            i
        );
        assert!(
            s.abs() <= 1.05,
            "Magnetic saturation must limit peak level under 0 dBFS (got {} at sample {})",
            s, i
        );
    }
}

#[test]
fn test_ca_05_delay_ducking_attenuates_tail_during_input_presence() {
    let mut delay = DelayProcessor::new(DelayModel::PristineDigitalDelay);
    delay.set_sample_rate(48000.0);
    delay.set_delay_time_ms(10.0);
    delay.set_feedback_pct(50.0);
    delay.set_ducking_enabled(true);
    delay.set_ducking_threshold_db(-30.0);
    delay.set_ducking_attenuation_db(-12.0);

    let n = 512;
    // Strong input signal above threshold (-30 dBFS)
    let loud_input = vec![0.5f32; n];
    let mut out_ducked = vec![0.0f32; n];
    delay.process(&loud_input, &mut out_ducked);

    let duck_gr = delay.ducking_gain_reduction_db();
    assert!(
        duck_gr <= -6.0,
        "Ducking must attenuate repeats during loud input (got {} dB)",
        duck_gr
    );
}

#[test]
fn test_ca_06_reverb_models_spring_drip_and_shimmer_octave() {
    let mut spring = ReverbProcessor::new(ReverbModel::TubeSpringReverb);
    spring.set_spring_tension(8.0);
    assert_eq!(spring.spring_tension(), 8.0);

    let mut shimmer = ReverbProcessor::new(ReverbModel::ShimmerCloudAmbient);
    shimmer.set_sample_rate(48000.0);

    let n = 1024;
    let in_sig: Vec<f32> = (0..n)
        .map(|i| 0.3 * (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / 48000.0)).sin())
        .collect();
    let mut out_sig = vec![0.0f32; n];
    shimmer.process(&in_sig, &mut out_sig);

    assert!(shimmer.has_shimmer_octave_generation());
}

#[test]
fn test_ca_07_spillover_preserves_decaying_tail_when_bypassed() {
    let mut delay = DelayProcessor::new(DelayModel::AnalogBbdDelay);
    delay.set_sample_rate(48000.0);
    delay.set_delay_time_ms(20.0);
    delay.set_feedback_pct(60.0);
    delay.set_spillover_enabled(true);

    let n = 512;
    let mut in_sig = vec![0.0f32; n];
    in_sig[0] = 0.8; // Burst
    let mut out1 = vec![0.0f32; n];
    delay.process(&in_sig, &mut out1);

    // Now bypass the block with spillover enabled
    delay.set_bypassed(true);
    let silent_in = vec![0.0f32; n];
    let mut out2 = vec![0.0f32; n];
    delay.process(&silent_in, &mut out2);

    // Tail should still ring out during spillover!
    let tail_energy: f32 = out2.iter().map(|s| s * s).sum();
    assert!(
        tail_energy > 0.001,
        "Spillover must keep the existing tail sounding after bypass"
    );
}

#[test]
fn test_ca_08_error_handling_and_safe_clamping_bpm_and_delay_time() {
    let mut delay = DelayProcessor::new(DelayModel::PristineDigitalDelay);
    delay.set_tempo_bpm(0.0);
    assert_eq!(delay.tempo_bpm(), 20.0); // clamped to min 20

    delay.set_tempo_bpm(500.0);
    assert_eq!(delay.tempo_bpm(), 300.0); // clamped to max 300

    delay.set_delay_time_ms(9999.0);
    assert_eq!(delay.delay_time_ms(), 4000.0); // clamped to max buffer 4000ms
}

#[test]
fn test_ca_09_non_regression_zero_mix_dry_through_identity() {
    let mut mod_proc = ModulationProcessor::new(ModModel::AnalogBbdChorus);
    mod_proc.set_sample_rate(48000.0);
    mod_proc.set_mix_pct(0.0); // 100% dry

    let n = 256;
    let in_sig = vec![0.42f32; n];
    let mut out_sig = vec![0.0f32; n];
    mod_proc.process(&in_sig, &mut out_sig);

    for i in 0..n {
        assert_eq!(out_sig[i], in_sig[i]);
    }
}
