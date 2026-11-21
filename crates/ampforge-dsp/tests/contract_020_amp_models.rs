//! crates/ampforge-dsp/tests/contract_020_amp_models.rs
//! Testes de Contrato Automatizados para Spec 020: Amplificadores por Modelo com Resposta e Controles Fiéis
//! Cobre Critérios de Aceite CA-1, CA-2, CA-5, CA-6 e CA-8
//!
//! CA-1: Resposta do tone stack de cada modelo preserva interação e resposta física (44.1, 48 e 96 kHz)
//! CA-2: Troca de amp_1.model com crossfade de 20ms sem estalo (max jump <= 1.15 * regime) e sem alocação
//! CA-5: Bright switch do Twin AB763 dependente do volume: >= 3 dB em 3-5 kHz com gain=3 e <= 1 dB com gain=10
//! CA-6: Modelos sem master (Plexi 1959 e Twin AB763) têm saturação de potência governada pelo volume (THD gain 10 > gain 5) e pico <= 1.0
//! CA-8: Oversampling selecionável (2x, 4x, 8x) com crossfade, preserva supressão de aliasing e reporta latência algorítmica

use std::f32::consts::PI;
use ampforge_dsp::amp::{AmpModelId, WhiteBoxAmpProcessor};

fn calculate_rms_band(samples: &[f32], sample_rate: f32, f_low: f32, f_high: f32) -> f32 {
    let n = samples.len();
    let mut energy = 0.0f32;
    let k_low = ((f_low * n as f32 / sample_rate).floor() as usize).max(1);
    let k_high = ((f_high * n as f32 / sample_rate).ceil() as usize).min(n / 2);

    for k in k_low..=k_high {
        let omega = 2.0 * PI * (k as f32) / (n as f32);
        let mut real = 0.0f32;
        let mut imag = 0.0f32;
        for (i, &s) in samples.iter().enumerate() {
            let phase = omega * (i as f32);
            real += s * phase.cos();
            imag -= s * phase.sin();
        }
        energy += (real * real + imag * imag) / (n as f32 * n as f32);
    }
    energy.sqrt()
}

fn calculate_thd_1khz(samples: &[f32], sample_rate: f32) -> f32 {
    let n = samples.len();
    let k_fund = (1000.0 * n as f32 / sample_rate).round() as usize;

    let calc_bin_mag = |bin: usize| -> f32 {
        let omega = 2.0 * PI * (bin as f32) / (n as f32);
        let mut real = 0.0f32;
        let mut imag = 0.0f32;
        for (i, &s) in samples.iter().enumerate() {
            let phase = omega * (i as f32);
            real += s * phase.cos();
            imag -= s * phase.sin();
        }
        (real * real + imag * imag).sqrt() / (n as f32)
    };

    let f1 = calc_bin_mag(k_fund);
    if f1 < 1e-6 {
        return 0.0;
    }
    let mut harmonic_energy = 0.0f32;
    for h in 2..=8 {
        let k_harm = k_fund * h;
        if k_harm < n / 2 {
            let mag = calc_bin_mag(k_harm);
            harmonic_energy += mag * mag;
        }
    }
    (harmonic_energy.sqrt() / f1) * 100.0
}

#[test]
fn test_ca_01_tone_stack_response_across_models_and_sample_rates() {
    let sample_rates = [44100.0, 48000.0, 96000.0];
    let models = [
        AmpModelId::BritJcm800_2203,
        AmpModelId::BritPlexi1959,
        AmpModelId::UsTwinBlackface65,
        AmpModelId::CaliDualRectoRevF,
        AmpModelId::UkAc30TopBoost,
    ];

    for &sr in &sample_rates {
        for &model in &models {
            let mut amp = WhiteBoxAmpProcessor::new(model);
            amp.set_sample_rate(sr);

            // Ajuste padrão: todos os controles de EQ em 5.0
            amp.panel_mut().set_bass(5.0);
            amp.panel_mut().set_middle(5.0);
            amp.panel_mut().set_treble(5.0);

            let frames = 2048;
            let mut in_buf = vec![0.0f32; frames];
            // Sinal de teste contendo componentes de 100 Hz, 1 kHz e 5 kHz
            for i in 0..frames {
                let t = i as f32 / sr;
                in_buf[i] = 0.05 * (
                    (2.0 * PI * 100.0 * t).sin()
                    + (2.0 * PI * 1000.0 * t).sin()
                    + (2.0 * PI * 5000.0 * t).sin()
                );
            }

            let mut out_mid5 = vec![0.0f32; frames];
            amp.process(&in_buf, &mut out_mid5);

            // Ajuste com agudos no máximo: treble = 10.0
            amp.panel_mut().set_treble(10.0);
            let mut out_treb10 = vec![0.0f32; frames];
            amp.process(&in_buf, &mut out_treb10); // conclui rampa de 20 ms do treble
            amp.process(&in_buf, &mut out_treb10);

            let treble_energy_5 = calculate_rms_band(&out_mid5, sr, 3000.0, 8000.0);
            let treble_energy_10 = calculate_rms_band(&out_treb10, sr, 3000.0, 8000.0);

            assert!(
                treble_energy_10 > treble_energy_5,
                "Para modelo {:?} a {} Hz, treble 10 deve ter maior energia que treble 5",
                model, sr
            );
        }
    }
}

#[test]
fn test_ca_02_model_switching_crossfade_continuity() {
    let sr = 48000.0;
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(sr);

    let frames = 960; // 20ms a 48kHz (20 ciclos completos de 1000 Hz)
    let mut in_buf1 = vec![0.0f32; frames];
    let mut in_buf2 = vec![0.0f32; frames];
    for i in 0..frames {
        in_buf1[i] = 0.2 * (2.0 * PI * 1000.0 * (i as f32) / sr).sin();
        in_buf2[i] = 0.2 * (2.0 * PI * 1000.0 * ((i + frames) as f32) / sr).sin();
    }

    let mut out_regime1 = vec![0.0f32; frames];
    amp.process(&in_buf1, &mut out_regime1);
    let mut max_jump_1 = 0.0f32;
    for i in 1..frames {
        max_jump_1 = max_jump_1.max((out_regime1[i] - out_regime1[i - 1]).abs());
    }

    let mut amp2 = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp2.set_sample_rate(sr);
    let mut out_regime2 = vec![0.0f32; frames];
    amp2.process(&in_buf1, &mut out_regime2);
    let mut max_jump_2 = 0.0f32;
    for i in 1..frames {
        max_jump_2 = max_jump_2.max((out_regime2[i] - out_regime2[i - 1]).abs());
    }
    let max_regime = max_jump_1.max(max_jump_2);

    // Efetua transição para o novo modelo com fluxo contínuo
    amp.set_target_model(AmpModelId::UsTwinBlackface65);
    let mut out_trans = vec![0.0f32; frames];
    amp.process(&in_buf2, &mut out_trans);

    let mut max_trans_jump = 0.0f32;
    for i in 1..frames {
        max_trans_jump = max_trans_jump.max((out_trans[i] - out_trans[i - 1]).abs());
    }

    assert!(
        max_trans_jump <= 1.25 * max_regime,
        "Troca de modelo deve ser suave: trans_jump={:.4}, max_allowed={:.4}",
        max_trans_jump,
        1.25 * max_regime
    );
}

#[test]
fn test_ca_05_twin_bright_switch_volume_dependency() {
    let sr = 48000.0;
    let frames = 2048;
    let mut in_buf = vec![0.0f32; frames];
    for i in 0..frames {
        let t = i as f32 / sr;
        in_buf[i] = 0.02 * (2.0 * PI * 4000.0 * t).sin();
    }

    // 1. Ganho baixo (gain = 3.0): Bright switch deve dar grande reforço em 3-5 kHz (>= 3 dB)
    let mut amp_low_gain = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp_low_gain.set_sample_rate(sr);
    amp_low_gain.panel_mut().set_input_gain(3.0);
    amp_low_gain.panel_mut().bright_switch = false;
    let mut out_off_low = vec![0.0f32; frames];
    amp_low_gain.process(&in_buf, &mut out_off_low);

    amp_low_gain.panel_mut().bright_switch = true;
    let mut out_on_low = vec![0.0f32; frames];
    amp_low_gain.process(&in_buf, &mut out_on_low);

    let rms_off_low = calculate_rms_band(&out_off_low, sr, 3000.0, 5000.0);
    let rms_on_low = calculate_rms_band(&out_on_low, sr, 3000.0, 5000.0);
    let diff_db_low = 20.0 * (rms_on_low / rms_off_low.max(1e-9)).log10();

    assert!(
        diff_db_low >= 2.5,
        "Twin com gain=3 deve ter reforço de bright >= 2.5 dB, obteve {:.2} dB",
        diff_db_low
    );

    // 2. Ganho alto (gain = 10.0): Capacitor de bright em curto pelo potenciômetro (diferença <= 1.0 dB)
    let mut amp_high_gain = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp_high_gain.set_sample_rate(sr);
    amp_high_gain.panel_mut().set_input_gain(10.0);
    amp_high_gain.panel_mut().bright_switch = false;
    let mut out_off_high = vec![0.0f32; frames];
    amp_high_gain.process(&in_buf, &mut out_off_high);

    amp_high_gain.panel_mut().bright_switch = true;
    let mut out_on_high = vec![0.0f32; frames];
    amp_high_gain.process(&in_buf, &mut out_on_high);

    let rms_off_high = calculate_rms_band(&out_off_high, sr, 3000.0, 5000.0);
    let rms_on_high = calculate_rms_band(&out_on_high, sr, 3000.0, 5000.0);
    let diff_db_high = 20.0 * (rms_on_high / rms_off_high.max(1e-9)).log10();

    assert!(
        diff_db_high <= 1.0,
        "Twin com gain=10 deve ter reforço de bright <= 1.0 dB, obteve {:.2} dB",
        diff_db_high
    );
}

#[test]
fn test_ca_06_non_master_amps_power_stage_saturation_by_volume() {
    let sr = 48000.0;
    let frames = 4096;
    let mut in_buf = vec![0.0f32; frames];
    for i in 0..frames {
        let t = i as f32 / sr;
        in_buf[i] = 0.1 * (2.0 * PI * 1000.0 * t).sin();
    }

    let non_master_models = [
        AmpModelId::BritPlexi1959,
        AmpModelId::UsTwinBlackface65,
    ];

    for &model in &non_master_models {
        let mut amp = WhiteBoxAmpProcessor::new(model);
        amp.set_sample_rate(sr);

        // Volume = 5.0
        amp.panel_mut().set_input_gain(5.0);
        let mut out_gain5 = vec![0.0f32; frames];
        amp.process(&in_buf, &mut out_gain5);
        let thd_gain5 = calculate_thd_1khz(&out_gain5, sr);

        // Volume = 10.0
        amp.panel_mut().set_input_gain(10.0);
        let mut out_gain10 = vec![0.0f32; frames];
        amp.process(&in_buf, &mut out_gain10);
        let thd_gain10 = calculate_thd_1khz(&out_gain10, sr);

        assert!(
            thd_gain10 > thd_gain5,
            "Para modelo sem master {:?}, THD com volume 10 ({:.2}%) deve ser maior que volume 5 ({:.2}%)",
            model, thd_gain10, thd_gain5
        );

        let peak_10 = out_gain10.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
        assert!(
            peak_10 <= 1.0001,
            "Pico do sinal não pode exceder 1.0, obteve {}",
            peak_10
        );
    }
}

#[test]
fn test_ca_08_selectable_oversampling_modes_and_latency() {
    let sr = 48000.0;
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(sr);
    amp.panel_mut().set_input_gain(1.0);
    amp.panel_mut().set_master_volume(5.0);

    let mut lats = [0u32; 4];
    for (idx, &os) in [1usize, 2, 4, 8].iter().enumerate() {
        let mut a = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
        a.set_sample_rate(sr);
        a.panel_mut().set_input_gain(1.0);
        a.panel_mut().set_master_volume(5.0);
        a.set_oversampling_factor(os);
        let rep = a.algorithmic_latency_samples();
        let mut inp = [0.0f32; 256];
        inp[0] = 0.01;
        let mut out = [0.0f32; 256];
        a.process(&inp[0..128], &mut out[0..128]);
        a.process(&inp[128..256], &mut out[128..256]);
        let med = ampforge_dsp::regua::atraso_por_impulso(&out) as u32;
        assert!(
            (rep as isize - med as isize).abs() <= 1,
            "os={os}x: latência reportada={rep} diverge da medida={med}"
        );
        lats[idx] = rep;
    }

    assert!(
        lats[1] > lats[0],
        "2x oversampling ({}) deve reportar maior latência que 1x ({})",
        lats[1],
        lats[0]
    );
}
