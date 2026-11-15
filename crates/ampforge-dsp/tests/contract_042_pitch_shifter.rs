use ampforge_dsp::fx::pitch::{PitchKey, PitchModel, PitchProcessor, ScaleType};
use ampforge_dsp::tuner::TunerProcessor;
use std::f32::consts::PI;

fn detect_freq(samples: &[f32], sr: f32) -> f32 {
    let mut t = TunerProcessor::new();
    t.set_sample_rate(sr);
    t.set_active(true);
    t.detect_pitch(samples).frequency_hz
}

fn cents_err(measured: f32, expected: f32) -> f32 {
    1200.0 * (measured / expected).log2()
}

#[test]
fn test_ca_01_pitch_plus_12_octave_up_220_to_440hz() {
    let sr = 48000.0f32;
    let mut p = PitchProcessor::new(PitchModel::PedalWhammy);
    p.prepare(sr);
    p.set_semitones(12.0);
    p.set_mix_pct(100.0);

    let n = 16384;
    let input: Vec<f32> = (0..n)
        .map(|i| 0.5 * (2.0 * PI * 220.0 * i as f32 / sr).sin())
        .collect();
    let mut out = vec![0.0f32; n];
    p.process(&input, &mut out);

    let f_out = detect_freq(&out[8192..12288], sr);
    let err = cents_err(f_out, 440.0).abs();
    assert!(
        err <= 5.0,
        "CA-1: +12 semitons de 220 Hz deve produzir 440 Hz +/- 5 cents (obteve {f_out:.2} Hz, err={err:.2} cents)"
    );
}

#[test]
fn test_ca_02_pitch_drop_minus_2_110_to_98hz() {
    let sr = 48000.0f32;
    let mut p = PitchProcessor::new(PitchModel::PolyphonicDropCapo);
    p.prepare(sr);
    p.set_semitones(-2.0);
    p.set_mix_pct(100.0);

    let n = 16384;
    let input: Vec<f32> = (0..n)
        .map(|i| 0.5 * (2.0 * PI * 110.0 * i as f32 / sr).sin())
        .collect();
    let mut out = vec![0.0f32; n];
    p.process(&input, &mut out);

    let expected = 110.0 * 2.0f32.powf(-2.0 / 12.0); // ~97.9989 Hz
    let f_out = detect_freq(&out[8192..12288], sr);
    let err = cents_err(f_out, expected).abs();
    assert!(
        err <= 5.0,
        "CA-2: Drop -2 semitons de 110 Hz deve produzir {expected:.2} Hz +/- 5 cents (obteve {f_out:.2} Hz, err={err:.2} cents)"
    );
}

#[test]
fn test_ca_03_diatonic_harmonizer_c_major_all_7_degrees() {
    let mut p = PitchProcessor::new(PitchModel::IntelligentDualHarmonizer);
    p.prepare(48000.0);
    p.set_key(PitchKey::C);
    p.set_scale(ScaleType::Major);
    p.set_mix_pct(100.0);

    // C4, D4, E4, F4, G4, A4, B4 -> E4, F4, G4, A4, B4, C5, D5
    let pairs = [
        (261.6256f32, 329.6276f32),
        (293.6648, 349.2282),
        (329.6276, 391.9954),
        (349.2282, 440.0000),
        (391.9954, 493.8833),
        (440.0000, 523.2511),
        (493.8833, 587.3295),
    ];

    for (in_f, exp_f) in pairs {
        let got = p.calculate_harmonic_frequency(in_f);
        let err = cents_err(got, exp_f).abs();
        assert!(
            err <= 10.0,
            "CA-3: Harmonizer C Maior {in_f:.2} Hz deveria dar {exp_f:.2} Hz +/- 10 cents, obteve {got:.2} Hz (err={err:.2})"
        );
    }
}

#[test]
fn test_ca_04_pitch_declared_latency_matches_measured_impulse() {
    let sr = 48000.0f32;
    let mut p = PitchProcessor::new(PitchModel::PedalWhammy);
    p.prepare(sr);
    p.set_semitones(0.0);
    p.set_mix_pct(100.0);

    let declared = p.algorithmic_latency_samples();
    assert!(declared > 0, "CA-4: Latência algorítmica ativa deve ser > 0");

    let n = declared + 512;
    let mut imp = vec![0.0f32; n];
    imp[0] = 1.0;
    let mut out = vec![0.0f32; n];
    p.process(&imp, &mut out);

    let peak_idx = out
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
        .unwrap()
        .0;
    assert!(
        (peak_idx as isize - declared as isize).abs() <= 1,
        "CA-4: Latência medida ({peak_idx}) diverge da declarada ({declared})"
    );
}

#[test]
fn test_ca_07_pitch_bypass_bit_exact() {
    let mut p = PitchProcessor::new(PitchModel::PedalWhammy);
    p.prepare(48000.0);
    p.set_semitones(12.0);
    p.set_bypassed(true);

    let input: Vec<f32> = (0..512).map(|i| (i as f32 * 0.13).sin()).collect();
    let mut out = vec![0.0f32; 512];
    p.process(&input, &mut out);
    assert_eq!(input, out, "CA-7: Em bypass a saída deve ser bit-a-bit idêntica à entrada");
}
