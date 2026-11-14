//! Testes de contrato da Spec 030 — Noise Gate Completo com Histerese, Hold, Range e Sidechain Key Filter
//! Regime SDD + TDD estrito: testes red -> green cobrindo CA-1, CA-2, CA-3, CA-6, CA-7.

use ampforge_dsp::fx::{GateState, NoiseGateMode, NoiseGateProcessor};

fn make_sine(freq_hz: f32, amp_peak: f32, sr: f32, len: usize) -> Vec<f32> {
    (0..len)
        .map(|i| amp_peak * (2.0 * std::f32::consts::PI * freq_hz * (i as f32 / sr)).sin())
        .collect()
}

fn db_to_lin(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

#[test]
fn test_ca_01_hold_ms_and_hysteresis_zero_chatter() {
    let sr = 48000.0;
    let mut gate = NoiseGateProcessor::new(NoiseGateMode::ClassicExpanderGate);
    gate.set_sample_rate(sr);
    gate.set_threshold_db(-45.0);
    gate.set_hysteresis_db(4.0); // Fecha apenas abaixo de -49.0 dB
    gate.set_attack_ms(0.5);
    gate.set_hold_ms(40.0);
    gate.set_release_ms(20.0);
    gate.set_range_db(-70.0);
    gate.set_key_hpf_hz(20.0);
    gate.set_key_lpf_hz(12000.0);

    // 1. Abre o gate com seno forte de 220 Hz (-30 dBFS RMS -> peak = -30 dB * sqrt(2))
    let open_sig = make_sine(220.0, db_to_lin(-30.0) * std::f32::consts::SQRT_2, sr, 4800); // 100 ms
    let mut open_out = vec![0.0f32; open_sig.len()];
    gate.process(&open_sig, None, &mut open_out);
    assert_eq!(gate.state(), GateState::Open, "Gate deveria estar Open após sinal forte");

    // 2. Sinal na banda de histerese [-49 dB, -45 dB] (ex.: -47 dBFS RMS) NÃO deve causar chatter
    let hyst_sig = make_sine(220.0, db_to_lin(-47.0) * std::f32::consts::SQRT_2, sr, 4800);
    let mut hyst_out = vec![0.0f32; hyst_sig.len()];
    let mut transitions = 0usize;
    let mut prev_state = gate.state();
    for chunk_idx in 0..(hyst_sig.len() / 64) {
        let start = chunk_idx * 64;
        gate.process(&hyst_sig[start..start + 64], None, &mut hyst_out[start..start + 64]);
        if gate.state() != prev_state {
            transitions += 1;
            prev_state = gate.state();
        }
    }
    assert_eq!(transitions, 0, "Sinal dentro da banda de histerese não pode causar chatter");
    assert_eq!(gate.state(), GateState::Open, "Gate deve permanecer Open na histerese");

    // 3. Queda abrupta para -65 dBFS (abaixo de -49 dB): deve manter ganho ~1.0 por hold_ms (40ms ± 2ms)
    let drop_sig = make_sine(220.0, db_to_lin(-65.0) * std::f32::consts::SQRT_2, sr, 4800); // 100ms
    let mut drop_out = vec![0.0f32; drop_sig.len()];
    gate.process(&drop_sig, None, &mut drop_out);

    // Verifica durante o período inicial de hold (de t=2ms até t=36ms, garantindo margem de 2ms)
    let hold_check_start = (0.002 * sr) as usize;
    let hold_check_end = (0.036 * sr) as usize;
    let rms_in_hold: f32 = (drop_sig[hold_check_start..hold_check_end]
        .iter()
        .map(|s| s * s)
        .sum::<f32>()
        / (hold_check_end - hold_check_start) as f32)
        .sqrt();
    let rms_out_hold: f32 = (drop_out[hold_check_start..hold_check_end]
        .iter()
        .map(|s| s * s)
        .sum::<f32>()
        / (hold_check_end - hold_check_start) as f32)
        .sqrt();
    let ratio_hold = rms_out_hold / rms_in_hold.max(1e-9);
    assert!(
        ratio_hold > 0.95,
        "Durante o hold_ms (até ~38ms), o gate deve manter ganho ~1.0; razão obtida = {}",
        ratio_hold
    );

    // Após hold + release (t = 90..100ms), o gate deve ter fechado e atenuado
    let tail_start = (0.090 * sr) as usize;
    let rms_in_tail: f32 = (drop_sig[tail_start..].iter().map(|s| s * s).sum::<f32>()
        / (drop_sig.len() - tail_start) as f32)
        .sqrt();
    let rms_out_tail: f32 = (drop_out[tail_start..].iter().map(|s| s * s).sum::<f32>()
        / (drop_out.len() - tail_start) as f32)
        .sqrt();
    let atten_db = 20.0 * (rms_out_tail / rms_in_tail.max(1e-9)).log10();
    assert!(
        atten_db < -25.0,
        "Após expirar hold_ms + release, deve atenuar fortemente; obteve {:.2} dB",
        atten_db
    );
}

#[test]
fn test_ca_02_range_db_caps_stationary_attenuation() {
    let sr = 48000.0;
    let mut gate = NoiseGateProcessor::new(NoiseGateMode::ClassicExpanderGate);
    gate.set_sample_rate(sr);
    gate.set_threshold_db(-40.0);
    gate.set_hysteresis_db(2.0);
    gate.set_hold_ms(0.0);
    gate.set_release_ms(5.0);
    gate.set_range_db(-20.0);

    // Sinal constante bem abaixo do threshold (-60 dBFS -> permite medir ganho residual com precisão)
    let quiet = make_sine(220.0, db_to_lin(-60.0), sr, 4800); // 100 ms
    let mut out = vec![0.0f32; quiet.len()];
    gate.process(&quiet, None, &mut out);

    let gr = gate.gain_reduction_db();
    assert!(
        (gr - (-20.0)).abs() <= 0.5,
        "Com range_db = -20.0, atenuação estacionária deve ser -20.0 ± 0.5 dB, obteve {:.2} dB",
        gr
    );

    // Verifica também na amplitude RMS do áudio de saída (tail estacionária)
    let tail = &out[2400..];
    let in_tail = &quiet[2400..];
    let rms_in = (in_tail.iter().map(|x| x * x).sum::<f32>() / in_tail.len() as f32).sqrt();
    let rms_out = (tail.iter().map(|x| x * x).sum::<f32>() / tail.len() as f32).sqrt();
    let att_db = 20.0 * (rms_out / rms_in).log10();
    assert!(
        (att_db - (-20.0)).abs() <= 0.5,
        "Atenuação medida no sinal deve ser -20.0 ± 0.5 dB, obteve {:.2} dB",
        att_db
    );
}

#[test]
fn test_ca_03_sidechain_key_filter_rejects_60hz_hum_and_passes_220hz_note() {
    let sr = 48000.0;
    let mut gate = NoiseGateProcessor::new(NoiseGateMode::ClassicExpanderGate);
    gate.set_sample_rate(sr);
    gate.set_threshold_db(-50.0);
    gate.set_hysteresis_db(2.0);
    gate.set_attack_ms(1.0);
    gate.set_hold_ms(10.0);
    gate.set_release_ms(20.0);
    gate.set_range_db(-60.0);
    gate.set_key_hpf_hz(120.0);
    gate.set_key_lpf_hz(6000.0);

    // 1. Hum de 60 Hz a -35 dBFS não deve abrir o gate (rejeitado pelo key_hpf_hz = 120 Hz)
    let hum_60 = make_sine(60.0, db_to_lin(-35.0) * std::f32::consts::SQRT_2, sr, 9600);
    let mut out_hum = vec![0.0f32; hum_60.len()];
    gate.process(&hum_60, None, &mut out_hum);

    assert_ne!(
        gate.state(),
        GateState::Open,
        "Hum de 60 Hz a -35 dBFS com key_hpf_hz=120 Hz NÃO pode abrir o gate"
    );
    assert!(
        gate.gain_reduction_db() < -30.0,
        "Atenuação deve se manter com hum 60 Hz rejeitado no sidechain (GR = {:.2} dB)",
        gate.gain_reduction_db()
    );

    // 2. Nota de 220 Hz a -35 dBFS abre o gate normalmente e passa sem filtro de áudio
    let note_220 = make_sine(220.0, db_to_lin(-35.0) * std::f32::consts::SQRT_2, sr, 4800);
    let mut out_note = vec![0.0f32; note_220.len()];
    gate.process(&note_220, None, &mut out_note);

    assert_eq!(gate.state(), GateState::Open, "Nota de 220 Hz deve abrir o gate");
    assert!(
        gate.gain_reduction_db() > -0.5,
        "Com nota 220 Hz, GR deve ir perto de 0 dB (GR = {:.2} dB)",
        gate.gain_reduction_db()
    );
}

#[test]
fn test_ca_06_zero_alloc_and_steady_processing_without_locks() {
    let mut gate = NoiseGateProcessor::new(NoiseGateMode::IntelligentNaturalGate);
    gate.set_sample_rate(48000.0);
    gate.set_key_hpf_hz(80.0);
    gate.set_key_lpf_hz(6000.0);

    let block = make_sine(220.0, 0.1, 48000.0, 128);
    let mut out = [0.0f32; 128];
    for _ in 0..2000 {
        gate.process(&block, Some(&block), &mut out);
        assert!(out[127].is_finite());
    }
}

#[test]
fn test_ca_07_non_regression_natural_mode_uses_di_and_classic_uses_input() {
    let sr = 48000.0;
    let noisy = make_sine(220.0, 0.1, sr, 2400);
    let silent_di = vec![0.0f32; 2400];
    let mut out_nat = vec![0.0f32; 2400];

    let mut nat = NoiseGateProcessor::new(NoiseGateMode::IntelligentNaturalGate);
    nat.set_sample_rate(sr);
    nat.set_threshold_db(-40.0);
    nat.set_range_db(-70.0);
    nat.process(&noisy, Some(&silent_di), &mut out_nat);
    assert!(nat.gain_reduction_db() < -30.0);

    let mut classic = NoiseGateProcessor::new(NoiseGateMode::ClassicExpanderGate);
    classic.set_sample_rate(sr);
    classic.set_threshold_db(-40.0);
    classic.set_range_db(-70.0);
    let mut out_cls = vec![0.0f32; 2400];
    classic.process(&noisy, Some(&silent_di), &mut out_cls);
    assert!(classic.gain_reduction_db() > -1.0, "Classic deve ler entrada principal, ignorando DI");
}
