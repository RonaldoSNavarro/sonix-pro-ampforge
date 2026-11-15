use ampforge_dsp::looper::{LooperProcessor, LooperState};
use ampforge_dsp::tuner::{TunerMode, TunerProcessor, TunerTemperament};
use std::f32::consts::PI;

#[test]
fn test_ca_01_polyphonic_tuner_detects_detuned_g3_in_chord() {
    let sr = 48000.0f32;
    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(sr);
    tuner.set_active(true);
    tuner.set_mode(TunerMode::Polyphonic);

    // E2 A2 D3 G3(+20 cents) B3 E4 chord
    let g3_detuned = 196.00 * 2.0f32.powf(20.0 / 1200.0);
    let freqs = [82.41f32, 110.00, 146.83, g3_detuned, 246.94, 329.63];
    let n = 4096;
    let mut buf = vec![0.0f32; n];
    for i in 0..n {
        let t = i as f32 / sr;
        let mut sum = 0.0f32;
        for &f in &freqs {
            sum += (2.0 * PI * f * t).sin();
        }
        buf[i] = sum / freqs.len() as f32;
    }

    let _ = tuner.detect_pitch(&buf);
    let statuses = tuner.detect_polyphonic_status();
    assert_eq!(statuses.len(), 6);
    assert!(
        (statuses[3].cents_deviation - 20.0).abs() <= 3.0,
        "CA-1: Corda 4 (G3) deve relatar +20 +/- 3 cents, obteve {:.2}",
        statuses[3].cents_deviation
    );
    for (idx, s) in statuses.iter().enumerate() {
        if idx != 3 {
            assert!(
                s.cents_deviation.abs() <= 3.0,
                "CA-1: Corda {} ({}) deveria estar em |cents| <= 3, obteve {:.2}",
                s.string_index,
                s.note,
                s.cents_deviation
            );
        }
    }
}

#[test]
fn test_ca_02_sweetened_temperament_shifts_pitch_by_table() {
    let sr = 48000.0f32;
    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(sr);
    tuner.set_active(true);

    // 196.00 Hz (G3)
    let buf: Vec<f32> = (0..4096)
        .map(|i| (2.0 * PI * 196.00 * i as f32 / sr).sin())
        .collect();

    tuner.set_temperament(TunerTemperament::EqualTemperament);
    let eq_res = tuner.detect_pitch(&buf);

    tuner.set_temperament(TunerTemperament::SweetenedGuitar);
    let sw_res = tuner.detect_pitch(&buf);

    assert!(
        eq_res.cents_deviation.abs() <= 0.5,
        "CA-2: Equal temperament em G3 puro deve ser ~0 cents, obteve {:.2}",
        eq_res.cents_deviation
    );
    assert!(
        (sw_res.cents_deviation - eq_res.cents_deviation).abs() >= 1.5,
        "CA-2: SweetenedGuitar precisa deslocar G3 conforme tabela documentada (eq={:.2}, sw={:.2})",
        eq_res.cents_deviation,
        sw_res.cents_deviation
    );
}

#[test]
fn test_ca_03_tuner_measured_accuracy_at_110hz_plus_half_cent() {
    let sr = 48000.0f32;
    let mut tuner = TunerProcessor::new();
    tuner.set_sample_rate(sr);
    tuner.set_active(true);

    let f_test = 110.0 * 2.0f32.powf(0.5 / 1200.0);
    let buf: Vec<f32> = (0..4096)
        .map(|i| (2.0 * PI * f_test * i as f32 / sr).sin())
        .collect();

    let res = tuner.detect_pitch(&buf);
    let err = (res.cents_deviation - 0.5).abs();
    assert!(
        err <= 0.2,
        "CA-3: Erro medido em 110 Hz +0.5 cent foi {err:.3} cents (esperado <= 0.2 cent)"
    );
}

#[test]
fn test_ca_04_looper_half_speed_overdub_averages_not_doubles() {
    let mut looper = LooperProcessor::new();
    looper.prepare(48000.0, 2);

    // Record 100 samples of 0.0 as base
    looper.record();
    let base_in = vec![0.0f32; 100];
    let mut out = vec![0.0f32; 100];
    looper.process(&base_in, &mut out);
    looper.play();

    // Half-speed overdub of constant 0.1 for 1 full pass (200 input samples = 100 loop samples at 0.5x)
    looper.set_half_speed(true);
    looper.overdub();
    let od_in = vec![0.1f32; 200];
    let mut od_out = vec![0.0f32; 200];
    looper.process(&od_in, &mut od_out);

    // Now play back at 1x speed with 0.0 input and verify stored overdub is 0.1, NOT 0.2
    looper.set_half_speed(false);
    looper.play();
    let zeros = vec![0.0f32; 100];
    let mut check_out = vec![0.0f32; 100];
    looper.process(&zeros, &mut check_out);

    for (idx, &v) in check_out.iter().enumerate() {
        assert!(
            (v - 0.1).abs() <= 1e-4,
            "CA-4: Posição {idx} após overdub 0.5x tinha {v:.5} (esperado 0.1, não 0.2)"
        );
    }
}

#[test]
fn test_ca_05_looper_repeated_overdub_preserves_initial_undo() {
    let mut looper = LooperProcessor::new();
    looper.prepare(48000.0, 2);

    // Record base 0.2
    looper.record();
    let base = vec![0.2f32; 64];
    let mut out = vec![0.0f32; 64];
    looper.process(&base, &mut out);
    looper.play();

    // Overdub session (pass 1)
    looper.overdub();
    let layer1 = vec![0.1f32; 64];
    looper.process(&layer1, &mut out);

    // Press overdub again while in Overdub
    looper.overdub();
    let layer2 = vec![0.1f32; 64];
    looper.process(&layer2, &mut out);

    // Undo restores base (0.2) without losing pre-overdub snapshot
    looper.undo();
    let zeros = vec![0.0f32; 64];
    looper.process(&zeros, &mut out);
    assert!(
        (out[10] - 0.2).abs() <= 1e-5,
        "CA-5: Undo após overdub repetido deve restaurar base (0.2), obteve {}",
        out[10]
    );
}

#[test]
fn test_ca_06_looper_prepare_96khz_has_60s_capacity() {
    let mut looper = LooperProcessor::new();
    looper.prepare(96000.0, 60);
    looper.record();

    // Feed 50 seconds at 96 kHz -> must still be in Record (not auto-switched to Play at 30s!)
    let block = vec![0.01f32; 96000];
    let mut out = vec![0.0f32; 96000];
    for _ in 0..50 {
        looper.process(&block, &mut out);
    }
    assert_eq!(
        looper.state(),
        LooperState::Record,
        "CA-6: A 96 kHz após 50s o looper ainda deve estar em Record (capacidade = 60s)"
    );

    // Feed 11 more seconds -> exceeds 60s -> auto switches to Play
    for _ in 0..11 {
        looper.process(&block, &mut out);
    }
    assert_eq!(
        looper.state(),
        LooperState::Play,
        "CA-6: Ao atingir 60s a 96 kHz, looper deve passar automaticamente para Play"
    );
}
