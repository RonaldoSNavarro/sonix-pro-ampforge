//! Testes de contrato da auditoria 2026-11-20 (Fase 6 — Afinador).

use ampforge_dsp::tuner::{TunerMode, TunerProcessor, TunerTemperament};
use std::f64::consts::PI;

fn gen_signal(freq_hz: f64, sr: f32, len: usize, harmonics: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; len];
    for i in 0..len {
        let t = (i as f64) / (sr as f64);
        let mut s = 0.0f64;
        for h in 1..=harmonics {
            let f = freq_hz * (h as f64);
            if f < (sr as f64) * 0.45 {
                s += (2.0 * PI * f * t).sin() / (h as f64);
            }
        }
        out[i] = (s * 0.35) as f32;
    }
    out
}

#[test]
fn aud_33_detecta_de_41_a_1400() {
    let freqs: [(&str, f64); 13] = [
        ("E1", 41.20344),
        ("F#1", 46.24930),
        ("A1", 55.00000),
        ("B1", 61.73541),
        ("E2", 82.40689),
        ("A2", 110.00000),
        ("D3", 146.83238),
        ("G3", 195.99772),
        ("B3", 246.94165),
        ("E4", 329.62756),
        ("E5", 659.25511),
        ("E6", 1318.51023),
        ("1400Hz", 1400.0),
    ];

    for &sr in &[44_100.0f32, 48_000.0, 96_000.0] {
        let len = 8192usize;
        for &(name, f_true) in &freqs {
            for &harmonics in &[1usize, 8] {
                let sig = gen_signal(f_true, sr, len, harmonics);
                let mut tuner = TunerProcessor::new();
                tuner.set_sample_rate(sr);
                tuner.set_active(true);
                tuner.set_mode(TunerMode::Strobe);
                tuner.set_temperament(TunerTemperament::EqualTemperament);

                let res = tuner.detect_pitch(&sig);
                assert!(
                    res.note_name != "---" && res.frequency_hz > 0.0,
                    "AUD-33: {name} ({f_true} Hz, {harmonics} harm, sr={sr}) não foi detectado"
                );
                let err_cents = (1200.0 * ((res.frequency_hz as f64) / f_true).log2()).abs();
                assert!(
                    err_cents <= 0.1,
                    "AUD-33: {name} ({f_true} Hz, {harmonics} harm, sr={sr}) erro={err_cents:.4} cents (> 0.1 cent), medido={:.4} Hz",
                    res.frequency_hz
                );
            }
        }
    }
}

#[test]
fn aud_33_abaixo_de_38_nao_detecta() {
    let sr = 48_000.0f32;
    let sig = gen_signal(30.0, sr, 8192, 1);
    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(sr);
    tuner.set_active(true);
    let res = tuner.detect_pitch(&sig);
    assert_eq!(
        res.note_name, "---",
        "AUD-33: 30 Hz deve retornar '---', retornou {} ({:.2} Hz)",
        res.note_name, res.frequency_hz
    );
}

fn gen_poly_chord(sr: f32, len: usize, detune_cents: [f64; 6]) -> Vec<f32> {
    let base_freqs = [82.40689f64, 110.0, 146.83238, 195.99772, 246.94165, 329.62756];
    let mut out = vec![0.0f32; len];
    // Fases pseudo-aleatórias determinísticas por corda e harmônico
    for s in 0..6 {
        let f0 = base_freqs[s] * 2.0f64.powf(detune_cents[s] / 1200.0);
        for h in 1..=8 {
            let f = f0 * (h as f64);
            let phase = ((s * 17 + h * 31) as f64) * 0.73;
            let amp = 1.0 / (h as f64) / 6.0;
            for i in 0..len {
                let t = (i as f64) / (sr as f64);
                out[i] += (amp * (2.0 * PI * f * t + phase).sin()) as f32;
            }
        }
    }
    out
}

#[test]
fn aud_34_poli_com_harmonicos() {
    let sr = 48_000.0f32;
    let len = 16_384usize;

    // 1. Todas afinadas
    let sig_tuned = gen_poly_chord(sr, len, [0.0; 6]);
    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(sr);
    tuner.set_active(true);
    tuner.set_mode(TunerMode::Polyphonic);
    tuner.set_temperament(TunerTemperament::EqualTemperament);
    let _ = tuner.detect_pitch(&sig_tuned);
    let st = tuner.detect_polyphonic_status();
    for s in &st {
        assert!(
            s.cents_deviation.abs() <= 3.0,
            "AUD-34 (todas afinadas): corda {} ({}) teve {:.2} cents (esperado <= +-3 cents)",
            s.string_index,
            s.note,
            s.cents_deviation
        );
    }

    // 2. Cada corda desafinada independentemente em +10 e -10 cents
    for target in 0..6 {
        for &delta in &[10.0f64, -10.0f64] {
            let mut det = [0.0f64; 6];
            det[target] = delta;
            let sig = gen_poly_chord(sr, len, det);
            let _ = tuner.detect_pitch(&sig);
            let statuses = tuner.detect_polyphonic_status();
            for (idx, s) in statuses.iter().enumerate() {
                let expected = det[idx] as f32;
                let err = (s.cents_deviation - expected).abs();
                assert!(
                    err <= 3.0,
                    "AUD-34 (corda {} desafinada {:+.0}c): corda {} ({}) mediu {:+.2}c (esperado {:+.0}c +-3c, erro={:.2}c)",
                    target + 1,
                    delta,
                    s.string_index,
                    s.note,
                    s.cents_deviation,
                    expected,
                    err
                );
            }
        }
    }
}
