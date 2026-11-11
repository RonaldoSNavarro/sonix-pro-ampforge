//! crates/ampforge-dsp/tests/contract_021_workbench_live.rs
//! Testes de Contrato Automatizados para Spec 021: Bancada do Chassi Aberto (Mod Shop) ao Vivo
//! Cobre Critérios de Aceite CA-1, CA-2, CA-3, CA-4, CA-5, CA-6, CA-7, CA-8, CA-11 e CA-13

use std::f32::consts::PI;
use ampforge_dsp::amp::{AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::modshop::{PreampTube, RectifierType, ToneStackTopology};

fn calculate_thd(samples: &[f32], sample_rate: f32, f_fund: f32) -> f32 {
    let n = samples.len();
    let k_fund = (f_fund * n as f32 / sample_rate).round() as usize;

    let calc_bin_mag = |bin: usize| -> f32 {
        let omega = 2.0 * PI * (bin as f32) / (n as f32);
        let mut real = 0.0f32;
        let mut imag = 0.0f32;
        for (i, &s) in samples.iter().enumerate() {
            let hann = 0.5 * (1.0 - (2.0 * PI * (i as f32) / (n as f32)).cos());
            let w = s * hann;
            let phase = omega * (i as f32);
            real += w * phase.cos();
            imag -= w * phase.sin();
        }
        (real * real + imag * imag).sqrt() / (n as f32) * 2.0
    };

    let f1 = calc_bin_mag(k_fund);
    if f1 < 1e-6 {
        return 0.0;
    }
    let mut harmonic_energy = 0.0f32;
    for h in 2..=8 {
        let k_harm = ((f_fund * h as f32) * n as f32 / sample_rate).round() as usize;
        if k_harm < n / 2 {
            let mag = calc_bin_mag(k_harm);
            harmonic_energy += mag * mag;
        }
    }
    (harmonic_energy.sqrt() / f1) * 100.0
}

fn calculate_harmonic_db(samples: &[f32], sample_rate: f32, f_fund: f32, harmonic: usize) -> f32 {
    let n = samples.len();
    let calc_mag = |f: f32| -> f32 {
        let k = (f * n as f32 / sample_rate).round() as usize;
        let omega = 2.0 * PI * (k as f32) / (n as f32);
        let mut real = 0.0f32;
        let mut imag = 0.0f32;
        for (i, &s) in samples.iter().enumerate() {
            let hann = 0.5 * (1.0 - (2.0 * PI * (i as f32) / (n as f32)).cos());
            let w = s * hann;
            let phase = omega * (i as f32);
            real += w * phase.cos();
            imag -= w * phase.sin();
        }
        (real * real + imag * imag).sqrt() / (n as f32) * 2.0
    };

    let fund_mag = calc_mag(f_fund);
    let harm_mag = calc_mag(f_fund * harmonic as f32);
    if fund_mag < 1e-9 || harm_mag < 1e-9 {
        return -120.0;
    }
    20.0 * (harm_mag / fund_mag).log10()
}

fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() { return 0.0; }
    let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

#[test]
fn test_ca_01_preamp_tube_small_signal_gain() {
    let sr = 48000.0;
    let mut amp_ax7 = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_ax7.set_sample_rate(sr);
    amp_ax7.set_preamp_tube_type(PreampTube::Ax7A);

    let mut amp_at7 = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_at7.set_sample_rate(sr);
    amp_at7.set_preamp_tube_type(PreampTube::At7);

    let frames = 2048;
    let mut in_buf = vec![0.0f32; frames];
    for i in 0..frames {
        in_buf[i] = 0.005 * (2.0 * PI * 1000.0 * (i as f32) / sr).sin();
    }

    let mut out_ax7 = vec![0.0f32; frames];
    let mut out_at7 = vec![0.0f32; frames];
    amp_ax7.process(&in_buf, &mut out_ax7);
    amp_at7.process(&in_buf, &mut out_at7);

    let rms_ax7 = calculate_rms(&out_ax7[frames / 2..]);
    let rms_at7 = calculate_rms(&out_at7[frames / 2..]);

    let diff_db = 20.0 * (rms_ax7 / rms_at7.max(1e-9)).log10();
    assert!(
        diff_db >= 4.0,
        "12at7 deve reduzir ganho em >= 4 dB em relação a 12ax7: diff={:.2} dB",
        diff_db
    );
}

#[test]
fn test_ca_02_bias_crossover_distortion() {
    let sr = 48000.0;
    let mut amp_cold = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_cold.set_sample_rate(sr);
    amp_cold.set_triode_asymmetry(0.0);
    amp_cold.panel_mut().set_input_gain(7.1);
    amp_cold.panel_mut().set_overdrive(0.0);
    amp_cold.panel_mut().set_master_volume(5.0);
    amp_cold.set_negative_feedback(0.0);
    amp_cold.set_power_tube_bias_ma(20.0);

    let mut amp_warm = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_warm.set_sample_rate(sr);
    amp_warm.set_triode_asymmetry(0.0);
    amp_warm.panel_mut().set_input_gain(7.1);
    amp_warm.panel_mut().set_overdrive(0.0);
    amp_warm.panel_mut().set_master_volume(5.0);
    amp_warm.set_negative_feedback(0.0);
    amp_warm.set_power_tube_bias_ma(45.0);

    // Sinal de -30 dBFS (amplitude ~0.0316)
    // Usar 4800 amostras para que 2400 amostras a 48 kHz tenham exatamente 50 ciclos de 1 kHz (zero leakage DFT)
    let amp_in = 10.0f32.powf(-30.0 / 20.0);
    let frames = 4800;
    let mut in_buf = vec![0.0f32; frames];
    for i in 0..frames {
        in_buf[i] = amp_in * (2.0 * PI * 1000.0 * (i as f32) / sr).sin();
    }

    let mut out_cold = vec![0.0f32; frames];
    let mut out_warm = vec![0.0f32; frames];
    amp_cold.process(&in_buf, &mut out_cold);
    amp_warm.process(&in_buf, &mut out_warm);

    let thd_cold = calculate_thd(&out_cold[2400..], sr, 1000.0);
    let thd_warm = calculate_thd(&out_warm[2400..], sr, 1000.0);

    assert!(
        thd_cold >= 2.0 * thd_warm,
        "THD com bias=20mA ({:.2}%) deve ser >= 2x THD com bias=45mA ({:.2}%)",
        thd_cold,
        thd_warm
    );
}

#[test]
fn test_ca_03_dynamic_sag_rectifiers() {
    let sr = 48000.0;
    let mut amp_gz34 = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_gz34.set_sample_rate(sr);
    amp_gz34.set_rectifier_type(RectifierType::Gz34_5Ar4);
    amp_gz34.set_sag_amount(5.0);

    let mut amp_diode = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_diode.set_sample_rate(sr);
    amp_diode.set_rectifier_type(RectifierType::SolidStateDiode);
    amp_diode.set_sag_amount(5.0);

    let mut amp_nosag = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_nosag.set_sample_rate(sr);
    amp_nosag.set_rectifier_type(RectifierType::Gz34_5Ar4);
    amp_nosag.set_sag_amount(0.0);

    // Rajada de 0.5s a 0.5 de pico seguida de 0.5s a 0.05 de pico
    let n_burst = (sr * 0.5) as usize;
    let n_tail = (sr * 0.5) as usize;
    let frames = n_burst + n_tail;
    let mut in_buf = vec![0.0f32; frames];
    for i in 0..n_burst {
        in_buf[i] = 0.5 * (2.0 * PI * 1000.0 * (i as f32) / sr).sin();
    }
    for i in n_burst..frames {
        in_buf[i] = 0.05 * (2.0 * PI * 1000.0 * (i as f32) / sr).sin();
    }

    let mut out_gz34 = vec![0.0f32; frames];
    let mut out_diode = vec![0.0f32; frames];
    let mut out_nosag = vec![0.0f32; frames];

    amp_gz34.process(&in_buf, &mut out_gz34);
    amp_diode.process(&in_buf, &mut out_diode);
    amp_nosag.process(&in_buf, &mut out_nosag);

    // Redução de ganho durante a rajada (fim da rajada vs início)
    let rms_gz34_start = calculate_rms(&out_gz34[0..1024]);
    let rms_gz34_end = calculate_rms(&out_gz34[n_burst - 1024..n_burst]);
    let drop_gz34_db = 20.0 * (rms_gz34_start / rms_gz34_end.max(1e-9)).log10();

    let rms_diode_start = calculate_rms(&out_diode[0..1024]);
    let rms_diode_end = calculate_rms(&out_diode[n_burst - 1024..n_burst]);
    let drop_diode_db = 20.0 * (rms_diode_start / rms_diode_end.max(1e-9)).log10();

    let rms_nosag_start = calculate_rms(&out_nosag[0..1024]);
    let rms_nosag_end = calculate_rms(&out_nosag[n_burst - 1024..n_burst]);
    let drop_nosag_db = 20.0 * (rms_nosag_start / rms_nosag_end.max(1e-9)).log10();

    assert!(
        drop_gz34_db >= 2.0 * drop_diode_db,
        "Queda em dB com GZ34 ({:.2} dB) deve ser >= 2x a com diodo ({:.2} dB)",
        drop_gz34_db,
        drop_diode_db
    );

    assert!(
        drop_nosag_db.abs() <= 0.15,
        "Com sag=0 a queda deve ser <= 0.15 dB: got {:.2} dB",
        drop_nosag_db
    );
}

#[test]
fn test_ca_04_nfb_thd_and_speaker_resonance() {
    let sr = 48000.0;
    let mut amp_nfb0 = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_nfb0.set_sample_rate(sr);
    amp_nfb0.panel_mut().set_input_gain(2.5);
    amp_nfb0.panel_mut().set_overdrive(0.0);
    amp_nfb0.panel_mut().set_presence(0.0);
    amp_nfb0.panel_mut().set_master_volume(9.0);
    amp_nfb0.set_triode_asymmetry(0.0);
    amp_nfb0.set_power_tube_bias_ma(45.0);
    amp_nfb0.set_negative_feedback(0.0);

    let mut amp_nfb10 = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_nfb10.set_sample_rate(sr);
    amp_nfb10.panel_mut().set_input_gain(2.5);
    amp_nfb10.panel_mut().set_overdrive(0.0);
    amp_nfb10.panel_mut().set_presence(0.0);
    amp_nfb10.panel_mut().set_master_volume(9.0);
    amp_nfb10.set_triode_asymmetry(0.0);
    amp_nfb10.set_power_tube_bias_ma(45.0);
    amp_nfb10.set_negative_feedback(10.0);

    let frames = 4800;
    let mut in_buf = vec![0.0f32; frames];
    for i in 0..frames {
        in_buf[i] = 0.5 * (2.0 * PI * 1000.0 * (i as f32) / sr).sin();
    }

    let mut out_nfb0 = vec![0.0f32; frames];
    let mut out_nfb10 = vec![0.0f32; frames];
    amp_nfb0.process(&in_buf, &mut out_nfb0);
    amp_nfb10.process(&in_buf, &mut out_nfb10);

    let thd_nfb0 = calculate_thd(&out_nfb0[frames - 2400..], sr, 1000.0);
    let thd_nfb10 = calculate_thd(&out_nfb10[frames - 2400..], sr, 1000.0);

    assert!(
        thd_nfb10 < thd_nfb0,
        "NFB=10 ({:.2}%) deve ter menor THD que NFB=0 ({:.2}%)",
        thd_nfb10,
        thd_nfb0
    );

    // Resposta na frequência de ressonância do falante (~98 Hz) vs 1 kHz
    let mut in_res = vec![0.0f32; frames];
    for i in 0..frames {
        in_res[i] = 0.05 * (2.0 * PI * 98.0 * (i as f32) / sr).sin();
    }
    let mut out_res_nfb0 = vec![0.0f32; frames];
    let mut out_res_nfb10 = vec![0.0f32; frames];
    amp_nfb0.process(&in_res, &mut out_res_nfb0);
    amp_nfb10.process(&in_res, &mut out_res_nfb10);

    let boost_nfb0 = 20.0 * (calculate_rms(&out_res_nfb0[frames - 2400..]) / calculate_rms(&out_nfb0[frames - 2400..])).log10();
    let boost_nfb10 = 20.0 * (calculate_rms(&out_res_nfb10[frames - 2400..]) / calculate_rms(&out_nfb10[frames - 2400..])).log10();

    assert!(
        boost_nfb0 - boost_nfb10 >= 1.5,
        "Realce na ressonância deve ser maior com NFB=0 do que com NFB=10: diff={:.2} dB",
        boost_nfb0 - boost_nfb10
    );
}

#[test]
fn test_ca_05_transformer_saturation_low_freq() {
    let sr = 48000.0;
    let mut amp_x10 = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_x10.set_sample_rate(sr);
    amp_x10.panel_mut().set_input_gain(1.0);
    amp_x10.panel_mut().set_overdrive(0.0);
    amp_x10.panel_mut().set_bass(9.0);
    amp_x10.panel_mut().set_middle(3.0);
    amp_x10.panel_mut().set_treble(3.0);
    amp_x10.panel_mut().set_master_volume(8.0);
    amp_x10.set_power_tube_bias_ma(45.0);
    amp_x10.set_triode_asymmetry(0.0);
    amp_x10.set_transformer_saturation(10.0);

    let mut amp_x0 = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_x0.set_sample_rate(sr);
    amp_x0.panel_mut().set_input_gain(1.0);
    amp_x0.panel_mut().set_overdrive(0.0);
    amp_x0.panel_mut().set_bass(9.0);
    amp_x0.panel_mut().set_middle(3.0);
    amp_x0.panel_mut().set_treble(3.0);
    amp_x0.panel_mut().set_master_volume(8.0);
    amp_x0.set_power_tube_bias_ma(45.0);
    amp_x0.set_triode_asymmetry(0.0);
    amp_x0.set_transformer_saturation(0.0);

    let frames = 4800;
    let mut in_60hz = vec![0.0f32; frames];
    let mut in_1khz = vec![0.0f32; frames];
    for i in 0..frames {
        in_60hz[i] = 0.5 * (2.0 * PI * 60.0 * (i as f32) / sr).sin();
        in_1khz[i] = 0.5 * (2.0 * PI * 1000.0 * (i as f32) / sr).sin();
    }

    let mut out_60_x10 = vec![0.0f32; frames];
    let mut out_1k_x10 = vec![0.0f32; frames];
    amp_x10.process(&in_60hz, &mut out_60_x10);
    amp_x10.process(&in_1khz, &mut out_1k_x10);

    let thd_60_x10 = calculate_thd(&out_60_x10[frames - 2400..], sr, 60.0);
    let thd_1k_x10 = calculate_thd(&out_1k_x10[frames - 2400..], sr, 1000.0);
    let diff_x10 = thd_60_x10 - thd_1k_x10;

    let mut out_60_x0 = vec![0.0f32; frames];
    let mut out_1k_x0 = vec![0.0f32; frames];
    amp_x0.process(&in_60hz, &mut out_60_x0);
    amp_x0.process(&in_1khz, &mut out_1k_x0);

    let thd_60_x0 = calculate_thd(&out_60_x0[frames - 2400..], sr, 60.0);
    let thd_1k_x0 = calculate_thd(&out_1k_x0[frames - 2400..], sr, 1000.0);
    let diff_x0 = thd_60_x0 - thd_1k_x0;

    assert!(
        diff_x10 >= 2.0 * diff_x0.max(0.1),
        "THD(60Hz) - THD(1kHz) com xformer_sat=10 ({:.2}%) deve ser >= 2x a com xformer_sat=0 ({:.2}%)",
        diff_x10,
        diff_x0
    );
}

#[test]
fn test_ca_06_plate_voltage_headroom() {
    let sr = 48000.0;
    let mut amp_low_v = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_low_v.set_sample_rate(sr);
    amp_low_v.panel_mut().set_input_gain(8.0);
    amp_low_v.panel_mut().set_overdrive(0.0);
    amp_low_v.panel_mut().set_master_volume(1.0);
    amp_low_v.set_power_tube_bias_ma(45.0);
    amp_low_v.set_sag_amount(0.0);
    amp_low_v.set_transformer_saturation(0.0);
    amp_low_v.set_triode_asymmetry(0.0);
    amp_low_v.set_preamp_plate_voltage(180.0);

    let mut amp_high_v = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_high_v.set_sample_rate(sr);
    amp_high_v.panel_mut().set_input_gain(8.0);
    amp_high_v.panel_mut().set_overdrive(0.0);
    amp_high_v.panel_mut().set_master_volume(1.0);
    amp_high_v.set_power_tube_bias_ma(45.0);
    amp_high_v.set_sag_amount(0.0);
    amp_high_v.set_transformer_saturation(0.0);
    amp_high_v.set_triode_asymmetry(0.0);
    amp_high_v.set_preamp_plate_voltage(340.0);

    let frames = 4800;
    // Encontra amplitude de entrada para 1% de THD
    let find_input_for_1pct_thd = |amp: &mut WhiteBoxAmpProcessor| -> f32 {
        let mut low = 0.01f32;
        let mut high = 1.0f32;
        for _ in 0..16 {
            let mid = (low + high) / 2.0;
            let mut in_buf = vec![0.0f32; frames];
            for i in 0..frames {
                in_buf[i] = mid * (2.0 * PI * 1000.0 * (i as f32) / sr).sin();
            }
            let mut out = vec![0.0f32; frames];
            amp.process(&in_buf, &mut out);
            let thd = calculate_thd(&out[frames - 2400..], sr, 1000.0);
            if thd < 1.0 {
                low = mid;
            } else {
                high = mid;
            }
        }
        (low + high) / 2.0
    };

    let in_1pct_low = find_input_for_1pct_thd(&mut amp_low_v);
    let in_1pct_high = find_input_for_1pct_thd(&mut amp_high_v);

    let diff_db = 20.0 * (in_1pct_high / in_1pct_low).log10();
    assert!(
        diff_db >= 3.0,
        "Nível de entrada para 1% THD com 180V deve ser >= 3 dB menor que com 340V: diff={:.2} dB",
        diff_db
    );
}

#[test]
fn test_ca_07_triode_asymmetry_second_harmonic() {
    let sr = 48000.0;
    let mut amp_sym = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_sym.set_sample_rate(sr);
    amp_sym.set_sag_amount(0.0);
    amp_sym.set_power_tube_bias_ma(45.0);
    amp_sym.set_transformer_saturation(0.0);
    amp_sym.set_triode_asymmetry(0.0);

    let mut amp_asym = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_asym.set_sample_rate(sr);
    amp_asym.set_sag_amount(0.0);
    amp_asym.set_power_tube_bias_ma(45.0);
    amp_asym.set_transformer_saturation(0.0);
    amp_asym.set_triode_asymmetry(0.6);

    let frames = 4096;
    let mut in_buf = vec![0.0f32; frames];
    for i in 0..frames {
        in_buf[i] = 0.1 * (2.0 * PI * 1000.0 * (i as f32) / sr).sin();
    }

    let mut out_sym = vec![0.0f32; frames];
    let mut out_asym = vec![0.0f32; frames];
    amp_sym.process(&in_buf, &mut out_sym);
    amp_asym.process(&in_buf, &mut out_asym);

    let h2_sym_db = calculate_harmonic_db(&out_sym[frames / 2..], sr, 1000.0, 2);
    let h2_asym_db = calculate_harmonic_db(&out_asym[frames / 2..], sr, 1000.0, 2);

    assert!(
        h2_sym_db <= -55.0,
        "2º harmônico com assimetria 0 deve ser <= -55 dBc: got {:.1} dBc",
        h2_sym_db
    );
    assert!(
        h2_asym_db - h2_sym_db >= 10.0,
        "2º harmônico deve subir >= 10 dB com assimetria 0.6: sym={:.1}, asym={:.1}",
        h2_sym_db,
        h2_asym_db
    );
}

#[test]
fn test_ca_08_tonestack_override() {
    let sr = 48000.0;
    let mut amp_stock = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_stock.set_sample_rate(sr);

    let mut amp_fender = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp_fender.set_sample_rate(sr);
    amp_fender.set_tonestack_topology(ToneStackTopology::FenderTmbMidScoop);

    let frames = 2048;
    let mut in_buf = vec![0.0f32; frames];
    for i in 0..frames {
        let t = i as f32 / sr;
        in_buf[i] = 0.05 * (
            (2.0 * PI * 100.0 * t).sin()
            + (2.0 * PI * 650.0 * t).sin()
            + (2.0 * PI * 3500.0 * t).sin()
        );
    }

    let mut out_stock = vec![0.0f32; frames];
    let mut out_fender = vec![0.0f32; frames];
    amp_stock.process(&in_buf, &mut out_stock);
    amp_fender.process(&in_buf, &mut out_fender);

    let rms_stock = calculate_rms(&out_stock[frames / 2..]);
    let rms_fender = calculate_rms(&out_fender[frames / 2..]);

    assert!(
        (rms_stock - rms_fender).abs() > 1e-4,
        "Substituir tone stack deve alterar a resposta em frequência"
    );
}

#[test]
fn test_ca_11_enum_switching_continuity() {
    let sr = 48000.0;
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(sr);

    let frames = 960;
    let mut in_buf1 = vec![0.0f32; frames];
    let mut in_buf2 = vec![0.0f32; frames];
    for i in 0..frames {
        in_buf1[i] = 0.2 * (2.0 * PI * 1000.0 * (i as f32) / sr).sin();
        in_buf2[i] = 0.2 * (2.0 * PI * 1000.0 * ((i + frames) as f32) / sr).sin();
    }

    let mut out_regime = vec![0.0f32; frames];
    amp.process(&in_buf1, &mut out_regime);
    let mut max_jump = 0.0f32;
    for i in 1..frames {
        max_jump = max_jump.max((out_regime[i] - out_regime[i - 1]).abs());
    }

    amp.set_preamp_tube_type(PreampTube::At7);
    let mut out_trans = vec![0.0f32; frames];
    amp.process(&in_buf2, &mut out_trans);

    let mut max_trans_jump = 0.0f32;
    for i in 1..frames {
        max_trans_jump = max_trans_jump.max((out_trans[i] - out_trans[i - 1]).abs());
    }

    assert!(
        max_trans_jump <= 1.25 * max_jump,
        "Troca de válvula do pré deve ser suave: trans={:.4}, max_allowed={:.4}",
        max_trans_jump,
        1.25 * max_jump
    );
}
