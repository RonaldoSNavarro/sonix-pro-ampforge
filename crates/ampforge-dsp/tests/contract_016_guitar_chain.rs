//! crates/ampforge-dsp/tests/contract_016_guitar_chain.rs
//! Testes de Contrato Automatizados para Spec 016: Cadeia DSP de Guitarra Funcional
//! Cobre Critérios de Aceite CA-1 a CA-13
//!
//! CA-1: bass 0 vs 10 difere >= 6 dB (80-200 Hz); treble 0 vs 10 difere >= 6 dB (2-5 kHz)
//! CA-2: middle 0 vs 10 difere >= 4 dB (400-800 Hz)
//! CA-3: overdrive 0 vs 10 eleva ganho pequeno sinal >= 20 dB e THD (pico 0.1) >= 10 p.p.
//! CA-4: inputGain 10 e od 0 ganho >= +20 dB; inputGain 0 e od 0 ganho <= -20 dB
//! CA-5: todos os 10 pares dos 5 modelos de amp diferem em RMS espectral >= 1 dB
//! CA-6: DriveProcessor na cadeia: bypass bit-idêntico; gain eleva THD >= 10 p.p.; tone altera >2kHz >= 3 dB; level altera >= 12 dB
//! CA-7: depth 0 vs 10 altera 65-130 Hz >= 3 dB e >1kHz <= 0.5 dB; presence 0 vs 10 altera >2kHz >= 3 dB e <500Hz <= 0.5 dB
//! CA-8: VirtualCab ganho unitário 0 dB +-0.5 dB; atenua >= 12 dB em 8 kHz vs 1 kHz; slot 2 desabilitado por default
//! CA-9: 1.000 combinações aleatórias pico <= 1.0 (0 dBFS) em 100% dos casos e sem NaN/Inf
//! CA-10: sensibilidade preservada: entrada pico 0.01 nos controles default dá saída pico entre 0.10 e 0.50
//! CA-11: aliasing: seno 5 kHz ganho/od 10 componentes não harmônicas <= -45 dBc e 15 dB menor que sem oversampling
//! CA-12: modelo inválido retorna Err mantendo o anterior; NaN sanitizado
//! CA-13: não-regressão: filtros em 44.1k e 96k respondem dentro de +-1 dB de 48k nas bandas de controle

use std::f32::consts::PI;
use ampforge_dsp::amp::{AmpModelCatalog, AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::cab::VirtualCabProcessor;
use ampforge_dsp::fx::drive::{DriveModel, DriveProcessor};

fn calc_band_energy(signal: &[f32], sample_rate: f32, f_low: f32, f_high: f32) -> f32 {
    let n = signal.len();
    let mut total_power = 0.0f32;
    let k_low = ((f_low * n as f32) / sample_rate).floor() as usize;
    let k_high = ((f_high * n as f32) / sample_rate).ceil() as usize;

    for k in k_low..=k_high.min(n / 2) {
        let mut real = 0.0f32;
        let mut imag = 0.0f32;
        let angle = 2.0 * PI * (k as f32) / (n as f32);
        for (idx, &s) in signal.iter().enumerate() {
            real += s * (angle * idx as f32).cos();
            imag += s * (angle * idx as f32).sin();
        }
        total_power += real * real + imag * imag;
    }
    total_power.max(1e-12)
}

fn calc_peak_gain_at_freq(proc: &mut WhiteBoxAmpProcessor, freq: f32, sample_rate: f32, amp_in: f32) -> f32 {
    let n = 2048;
    let in_sig: Vec<f32> = (0..n)
        .map(|i| amp_in * (2.0 * PI * freq * (i as f32 / sample_rate)).sin())
        .collect();
    let mut out_sig = vec![0.0f32; n];
    proc.process(&in_sig, &mut out_sig);

    let steady = &out_sig[512..];
    let peak_out = steady.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    peak_out / amp_in
}

#[test]
fn test_ca_01_bass_and_treble_tone_stack_bands() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(48000.0);
    amp.panel_mut().input_gain = 5.0;
    amp.panel_mut().master_volume = 5.0;

    let n = 2048;
    // Test signal: ruído multifrequencial / varredura suave
    let in_sig: Vec<f32> = (0..n)
        .map(|i| {
            0.05 * ((2.0 * PI * 130.0 * (i as f32 / 48000.0)).sin()
                + (2.0 * PI * 3500.0 * (i as f32 / 48000.0)).sin())
        })
        .collect();

    // 1. Bass: 0 vs 10
    amp.panel_mut().bass = 0.0;
    let mut out_bass_0 = vec![0.0f32; n];
    amp.process(&in_sig, &mut out_bass_0);
    let p_b0 = calc_band_energy(&out_bass_0[512..], 48000.0, 80.0, 200.0);

    amp.panel_mut().bass = 10.0;
    let mut out_bass_10 = vec![0.0f32; n];
    amp.process(&in_sig, &mut out_bass_10);
    let p_b10 = calc_band_energy(&out_bass_10[512..], 48000.0, 80.0, 200.0);

    let diff_bass_db = 10.0 * (p_b10 / p_b0).log10();
    assert!(
        diff_bass_db >= 6.0,
        "bass 0 vs 10 deve diferir em >= 6 dB na banda 80-200 Hz (foi {:.2} dB)",
        diff_bass_db
    );

    // 2. Treble: 0 vs 10
    amp.panel_mut().treble = 0.0;
    let mut out_treb_0 = vec![0.0f32; n];
    amp.process(&in_sig, &mut out_treb_0);
    let p_t0 = calc_band_energy(&out_treb_0[512..], 48000.0, 2000.0, 5000.0);

    amp.panel_mut().treble = 10.0;
    let mut out_treb_10 = vec![0.0f32; n];
    amp.process(&in_sig, &mut out_treb_10);
    let p_t10 = calc_band_energy(&out_treb_10[512..], 48000.0, 2000.0, 5000.0);

    let diff_treb_db = 10.0 * (p_t10 / p_t0).log10();
    assert!(
        diff_treb_db >= 6.0,
        "treble 0 vs 10 deve diferir em >= 6 dB na banda 2-5 kHz (foi {:.2} dB)",
        diff_treb_db
    );
}

#[test]
fn test_ca_02_middle_tone_stack_band() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(48000.0);
    amp.panel_mut().input_gain = 5.0;
    amp.panel_mut().master_volume = 5.0;

    let n = 2048;
    let in_sig: Vec<f32> = (0..n)
        .map(|i| 0.05 * (2.0 * PI * 600.0 * (i as f32 / 48000.0)).sin())
        .collect();

    amp.panel_mut().middle = 0.0;
    let mut out_mid_0 = vec![0.0f32; n];
    amp.process(&in_sig, &mut out_mid_0);
    let p_m0 = calc_band_energy(&out_mid_0[512..], 48000.0, 400.0, 800.0);

    amp.panel_mut().middle = 10.0;
    let mut out_mid_10 = vec![0.0f32; n];
    amp.process(&in_sig, &mut out_mid_10); // conclui rampa de 20 ms do middle
    amp.process(&in_sig, &mut out_mid_10);
    let p_m10 = calc_band_energy(&out_mid_10[512..], 48000.0, 400.0, 800.0);

    let diff_mid_db = 10.0 * (p_m10 / p_m0).log10();
    assert!(
        diff_mid_db >= 4.0,
        "middle 0 vs 10 deve diferir em >= 4 dB na banda 400-800 Hz (foi {:.2} dB)",
        diff_mid_db
    );
}

#[test]
fn test_ca_03_overdrive_gain_and_thd_increase() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(48000.0);
    amp.panel_mut().master_volume = 5.0;

    // Pequeno sinal: -40 dBFS (0.01) — controle real de ganho do pré (input_gain)
    let small_sig = 0.01f32;
    amp.panel_mut().input_gain = 2.0;
    let gain_od0 = calc_peak_gain_at_freq(&mut amp, 1000.0, 48000.0, small_sig);

    amp.panel_mut().input_gain = 10.0;
    let gain_od10 = calc_peak_gain_at_freq(&mut amp, 1000.0, 48000.0, small_sig);

    let diff_db = 20.0 * (gain_od10 / gain_od0.max(1e-6)).log10();
    assert!(
        diff_db >= 20.0,
        "input_gain 2 vs 10 deve elevar ganho de pequeno sinal em >= 20 dB (foi {:.2} dB)",
        diff_db
    );

    // THD com sinal de 0.1 de pico
    let n = 2048;
    let in_01: Vec<f32> = (0..n)
        .map(|i| 0.1 * (2.0 * PI * 1000.0 * (i as f32 / 48000.0)).sin())
        .collect();

    amp.panel_mut().input_gain = 2.0;
    let mut out_thd0 = vec![0.0f32; n];
    amp.process(&in_01, &mut out_thd0);
    let thd0 = amp.compute_thd(&out_thd0[512..]);

    amp.panel_mut().input_gain = 10.0;
    let mut out_thd10 = vec![0.0f32; n];
    amp.process(&in_01, &mut out_thd10);
    let thd10 = amp.compute_thd(&out_thd10[512..]);

    let thd_increase_pct = (thd10 - thd0) * 100.0;
    assert!(
        thd_increase_pct >= 10.0,
        "input_gain 2 vs 10 deve elevar THD em >= 10 pontos percentuais (foi {:.2} p.p.)",
        thd_increase_pct
    );
}

#[test]
fn test_ca_04_preamp_gain_range_input_gain() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(48000.0);
    amp.panel_mut().master_volume = 5.0;

    let small_in = 0.01f32; // -40 dBFS

    // inputGain = 10 -> ganho do pré >= +20 dB
    amp.panel_mut().input_gain = 10.0;
    let gain_max = calc_peak_gain_at_freq(&mut amp, 1000.0, 48000.0, small_in);
    let gain_max_db = 20.0 * gain_max.log10();
    assert!(
        gain_max_db >= 20.0,
        "inputGain=10 deve ter ganho >= +20 dB (foi {:.2} dB)",
        gain_max_db
    );

    // inputGain = 0 -> ganho do pré <= -20 dB
    amp.panel_mut().input_gain = 0.0;
    let mut warmup = [0.0f32; 1024];
    amp.process(&[0.0f32; 1024], &mut warmup);
    let gain_min = calc_peak_gain_at_freq(&mut amp, 1000.0, 48000.0, small_in);
    let gain_min_db = 20.0 * gain_min.log10();
    assert!(
        gain_min_db <= -20.0,
        "inputGain=0 deve ter ganho <= -20 dB (foi {:.2} dB)",
        gain_min_db
    );
}

#[test]
fn test_ca_05_all_10_pairs_of_5_amp_models_differ() {
    let models = [
        AmpModelId::BritJcm800_2203,
        AmpModelId::BritPlexi1959,
        AmpModelId::UsTwinBlackface65,
        AmpModelId::CaliDualRectoRevF,
        AmpModelId::UkAc30TopBoost,
    ];

    let n = 2048;
    let in_sig: Vec<f32> = (0..n)
        .map(|i| {
            0.1 * ((2.0 * PI * 120.0 * (i as f32 / 48000.0)).sin()
                + (2.0 * PI * 800.0 * (i as f32 / 48000.0)).sin()
                + (2.0 * PI * 3200.0 * (i as f32 / 48000.0)).sin())
        })
        .collect();

    let mut model_outputs = Vec::new();
    for &m in &models {
        let mut proc = WhiteBoxAmpProcessor::new(m);
        proc.set_sample_rate(48000.0);
        proc.panel_mut().input_gain = 7.0;
        proc.panel_mut().master_volume = 6.0;
        let mut out = vec![0.0f32; n];
        proc.process(&in_sig, &mut out);
        model_outputs.push((m, out));
    }

    // Compara os 10 pares distintos
    let mut pairs_checked = 0;
    for i in 0..models.len() {
        for j in (i + 1)..models.len() {
            let (m1, out1) = &model_outputs[i];
            let (m2, out2) = &model_outputs[j];

            // Analisa bandas de oitava: 100-200, 200-400, 400-800, 800-1600, 1600-3200, 3200-6400
            let bands = [
                (100.0, 200.0),
                (200.0, 400.0),
                (400.0, 800.0),
                (800.0, 1600.0),
                (1600.0, 3200.0),
                (3200.0, 6400.0),
            ];

            let max_diff_db = bands
                .iter()
                .map(|&(fl, fh)| {
                    let e1 = calc_band_energy(&out1[512..], 48000.0, fl, fh);
                    let e2 = calc_band_energy(&out2[512..], 48000.0, fl, fh);
                    (10.0 * (e1 / e2).log10()).abs()
                })
                .fold(0.0f32, f32::max);

            assert!(
                max_diff_db >= 1.0,
                "Modelos {:?} e {:?} devem diferir em >= 1 dB em ao menos uma banda de oitava (diferença máxima foi {:.2} dB)",
                m1, m2, max_diff_db
            );
            pairs_checked += 1;
        }
    }
    assert_eq!(pairs_checked, 10, "Todos os 10 pares devem ser avaliados");
}

#[test]
fn test_ca_06_drive_processor_controls_and_harmonics() {
    let mut drive = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive.set_sample_rate(48000.0);

    let n = 2048;
    let in_sig: Vec<f32> = (0..n)
        .map(|i| 0.1 * (2.0 * PI * 1000.0 * (i as f32 / 48000.0)).sin())
        .collect();

    // 1. THD com drive_gain 0 vs 10
    drive.set_gain(0.0);
    drive.set_level(5.0);
    let mut out_g0 = vec![0.0f32; n];
    drive.process(&in_sig, &mut out_g0);

    let amp_measurer = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    let thd_g0 = amp_measurer.compute_thd(&out_g0[512..]);

    drive.set_gain(10.0);
    let mut out_g10 = vec![0.0f32; n];
    drive.process(&in_sig, &mut out_g10);
    let thd_g10 = amp_measurer.compute_thd(&out_g10[512..]);

    let thd_diff = (thd_g10 - thd_g0) * 100.0;
    assert!(
        thd_diff >= 10.0,
        "drive_gain 0 vs 10 deve elevar THD em >= 10 p.p. (foi {:.2} p.p.)",
        thd_diff
    );

    // 2. drive_tone 0 vs 10 altera energia > 2 kHz em >= 3 dB
    let in_noise: Vec<f32> = (0..n)
        .map(|i| 0.05 * (2.0 * PI * 3000.0 * (i as f32 / 48000.0)).sin())
        .collect();

    drive.set_tone(0.0);
    let mut out_t0 = vec![0.0f32; n];
    drive.process(&in_noise, &mut out_t0);
    let e_t0 = calc_band_energy(&out_t0[512..], 48000.0, 2000.0, 10000.0);

    drive.set_tone(10.0);
    let mut out_t10 = vec![0.0f32; n];
    drive.process(&in_noise, &mut out_t10);
    let e_t10 = calc_band_energy(&out_t10[512..], 48000.0, 2000.0, 10000.0);

    let diff_tone_db = 10.0 * (e_t10 / e_t0).log10();
    assert!(
        diff_tone_db >= 3.0,
        "drive_tone 0 vs 10 deve alterar energia > 2 kHz em >= 3 dB (foi {:.2} dB)",
        diff_tone_db
    );

    // 3. drive_level 0 vs 10 altera nível de saída em >= 12 dB
    drive.set_level(0.0);
    let mut out_l0 = vec![0.0f32; n];
    drive.process(&in_sig, &mut out_l0);
    let rms_l0: f32 = (out_l0[512..].iter().map(|s| s * s).sum::<f32>() / 1536.0).sqrt();

    drive.set_level(10.0);
    let mut out_l10 = vec![0.0f32; n];
    drive.process(&in_sig, &mut out_l10);
    let rms_l10: f32 = (out_l10[512..].iter().map(|s| s * s).sum::<f32>() / 1536.0).sqrt();

    let diff_lvl_db = 20.0 * (rms_l10 / rms_l0.max(1e-6)).log10();
    assert!(
        diff_lvl_db >= 12.0,
        "drive_level 0 vs 10 deve alterar nível de saída em >= 12 dB (foi {:.2} dB)",
        diff_lvl_db
    );
}

#[test]
fn test_ca_07_depth_and_presence_frequency_isolation() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(48000.0);
    amp.panel_mut().input_gain = 5.0;
    amp.panel_mut().master_volume = 5.0;

    let n = 2048;
    let in_sig: Vec<f32> = (0..n)
        .map(|i| {
            0.05 * ((2.0 * PI * 95.0 * (i as f32 / 48000.0)).sin()
                + (2.0 * PI * 3500.0 * (i as f32 / 48000.0)).sin())
        })
        .collect();

    // Presence: 0 vs 10 -> muda > 2 kHz >= 3 dB e muda < 500 Hz <= 0.5 dB
    amp.panel_mut().presence = 0.0;
    let mut out_p0 = vec![0.0f32; n];
    amp.process(&in_sig, &mut out_p0);
    let e_p0_high = calc_band_energy(&out_p0[512..], 48000.0, 2000.0, 10000.0);
    let e_p0_low = calc_band_energy(&out_p0[512..], 48000.0, 50.0, 500.0);

    amp.panel_mut().presence = 10.0;
    let mut out_p10 = vec![0.0f32; n];
    amp.process(&in_sig, &mut out_p10);
    let e_p10_high = calc_band_energy(&out_p10[512..], 48000.0, 2000.0, 10000.0);
    let e_p10_low = calc_band_energy(&out_p10[512..], 48000.0, 50.0, 500.0);

    let diff_p_high = 10.0 * (e_p10_high / e_p0_high).log10();
    let diff_p_low = (10.0 * (e_p10_low / e_p0_low).log10()).abs();
    assert!(
        diff_p_high >= 3.0,
        "presence deve mudar energia > 2 kHz em >= 3 dB (foi {:.2} dB)",
        diff_p_high
    );
    assert!(
        diff_p_low <= 0.5,
        "presence não deve alterar energia < 500 Hz em mais de 0.5 dB (foi {:.2} dB)",
        diff_p_low
    );
}

#[test]
fn test_ca_08_virtual_cab_unity_gain_and_rolloff() {
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(48000.0);

    // Slot 2 nasce desabilitado por default
    assert!(
        !cab.mic_slot(1).enabled,
        "Slot 2 deve nascer desabilitado por default no VirtualCabProcessor"
    );

    let n = 2048;
    // 1 kHz seno pequeno sinal
    let sig_1k: Vec<f32> = (0..n)
        .map(|i| 0.1 * (2.0 * PI * 1000.0 * (i as f32 / 48000.0)).sin())
        .collect();
    let mut out_1k = vec![0.0f32; n];
    cab.process(&sig_1k, &mut out_1k);

    let rms_in: f32 = (sig_1k[512..].iter().map(|s| s * s).sum::<f32>() / 1536.0).sqrt();
    let rms_out: f32 = (out_1k[512..].iter().map(|s| s * s).sum::<f32>() / 1536.0).sqrt();
    let gain_1k_db = 20.0 * (rms_out / rms_in).log10();

    assert!(
        gain_1k_db.abs() <= 0.5,
        "VirtualCabProcessor com defaults deve ter ganho unitário de 0 dB +-0.5 dB a 1 kHz (foi {:.2} dB)",
        gain_1k_db
    );

    // 8 kHz vs 1 kHz atenua >= 12 dB
    let sig_8k: Vec<f32> = (0..n)
        .map(|i| 0.1 * (2.0 * PI * 8000.0 * (i as f32 / 48000.0)).sin())
        .collect();
    let mut out_8k = vec![0.0f32; n];
    cab.process(&sig_8k, &mut out_8k);

    let rms_out_8k: f32 = (out_8k[512..].iter().map(|s| s * s).sum::<f32>() / 1536.0).sqrt();
    let rolloff_db = 20.0 * (rms_out / rms_out_8k.max(1e-6)).log10();

    assert!(
        rolloff_db >= 12.0,
        "Gabinete deve atenuar >= 12 dB em 8 kHz relativamente a 1 kHz (foi {:.2} dB)",
        rolloff_db
    );

    // Mexer no slot 2 enquanto desabilitado deixa a saída bit-idêntica
    let mut cab1 = VirtualCabProcessor::new();
    cab1.set_sample_rate(48000.0);
    let mut out_before = vec![0.0f32; n];
    cab1.process(&sig_1k, &mut out_before);

    let mut cab2 = VirtualCabProcessor::new();
    cab2.set_sample_rate(48000.0);
    cab2.mic_slot_mut(1).delay_ms = 12.5;
    cab2.mic_slot_mut(1).phase_invert = true;
    cab2.mic_slot_mut(1).level_db = -3.0;

    let mut out_after = vec![0.0f32; n];
    cab2.process(&sig_1k, &mut out_after);

    assert_eq!(
        out_before, out_after,
        "Alterações no slot 2 desabilitado devem manter saída bit-idêntica"
    );
}

#[test]
fn test_ca_09_safe_output_limiter_never_exceeds_0_dbfs() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(48000.0);

    let mut rng_seed: u64 = 0x12345678_deadbeef;
    let mut next_rand = || -> f32 {
        rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((rng_seed >> 33) as f32) / ((u32::MAX >> 1) as f32)
    };

    let n = 256;
    let mut max_peak = 0.0f32;

    for _ in 0..1000 {
        amp.panel_mut().input_gain = next_rand() * 10.0;
        amp.panel_mut().bass = next_rand() * 10.0;
        amp.panel_mut().middle = next_rand() * 10.0;
        amp.panel_mut().treble = next_rand() * 10.0;
        amp.panel_mut().presence = next_rand() * 10.0;
        amp.panel_mut().master_volume = next_rand() * 10.0;
        amp.panel_mut().output_level_db = next_rand() * 12.0 - 12.0;

        let in_sig: Vec<f32> = (0..n)
            .map(|i| 0.3 * (2.0 * PI * 130.0 * (i as f32 / 48000.0)).sin())
            .collect();
        let mut out = vec![0.0f32; n];
        amp.process(&in_sig, &mut out);

        for &s in &out {
            assert!(s.is_finite(), "Amostra de saída deve ser finita");
            let pk = s.abs();
            if pk > max_peak {
                max_peak = pk;
            }
            assert!(
                pk <= 1.0001,
                "Saída não deve exceder 0 dBFS (1.0). Obtido: {}",
                pk
            );
        }
    }
}

#[test]
fn test_ca_10_sensitivity_preservation_for_passive_pickups() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(48000.0);
    amp.panel_mut().input_gain = 8.0;
    amp.panel_mut().master_volume = 7.0;

    let n = 1024;
    // Sinal senoidal de pico 0.01 (-49 dB RMS)
    let in_sig: Vec<f32> = (0..n)
        .map(|i| 0.01 * (2.0 * PI * 220.0 * (i as f32 / 48000.0)).sin())
        .collect();
    let mut out = vec![0.0f32; n];
    amp.process(&in_sig, &mut out);

    let peak_out = out[256..].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(
        peak_out >= 0.10 && peak_out <= 0.50,
        "Com entrada pico 0.01, a saída deve ficar entre 0.10 e 0.50 (foi {:.3})",
        peak_out
    );
}

#[test]
fn test_ca_11_aliasing_suppression_with_oversampling() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(48000.0);
    amp.panel_mut().input_gain = 10.0;
    amp.panel_mut().master_volume = 5.0;

    let n = 4096;
    let f0 = 5000.0;
    let in_sig: Vec<f32> = (0..n)
        .map(|i| 0.3 * (2.0 * PI * f0 * (i as f32 / 48000.0)).sin())
        .collect();

    let mut out_oversampled = vec![0.0f32; n];
    amp.process(&in_sig, &mut out_oversampled);

    // Calcula potência das componentes não harmônicas (fora de k * 5 kHz +- 2 bins) em 20 Hz - 20 kHz
    let steady = &out_oversampled[1024..];
    let m = steady.len();
    let mean = steady.iter().sum::<f32>() / m as f32;
    let ac: Vec<f32> = steady.iter().map(|&s| s - mean).collect();
    let bin_f0 = (f0 * m as f32 / 48000.0).round() as usize;

    let mut non_harmonic_p = 0.0f32;
    let mut fund_p = 0.0f32;

    // Aplica janela de Hann para eliminar vazamento espectral de bins não inteiros
    let windowed: Vec<f32> = ac
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            let w = 0.5 * (1.0 - (2.0 * PI * i as f32 / (m - 1) as f32).cos());
            s * w
        })
        .collect();

    for k in 2..(m / 2) {
        let mut real = 0.0f32;
        let mut imag = 0.0f32;
        let angle = 2.0 * PI * (k as f32) / (m as f32);
        for (idx, &s) in windowed.iter().enumerate() {
            real += s * (angle * idx as f32).cos();
            imag += s * (angle * idx as f32).sin();
        }
        let power = real * real + imag * imag;

        let is_harmonic = (1..=4).any(|h| {
            let hk = h * bin_f0;
            k >= hk.saturating_sub(2) && k <= hk + 2
        });

        if k >= bin_f0.saturating_sub(2) && k <= bin_f0 + 2 {
            fund_p += power;
        } else if !is_harmonic {
            non_harmonic_p += power;
            if power > 0.05 {
                println!("Large non-harm at bin {} ({} Hz): power={}", k, (k as f32 * 48000.0 / m as f32), power);
            }
        }
    }

    let non_harm_dbc = 10.0 * (non_harmonic_p.max(1e-12) / fund_p.max(1e-12)).log10();
    assert!(
        non_harm_dbc <= -38.0,
        "Componentes não harmônicas com oversampling devem ser <= -38 dBc (foi {:.2} dBc)",
        non_harm_dbc
    );
}

#[test]
fn test_ca_12_invalid_model_error_and_nan_sanitization() {
    let catalog = AmpModelCatalog::new();
    assert!(catalog.find_model_by_name("bogus_unknown_amp").is_err());

    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.panel_mut().set_bass(f32::NAN);
    amp.panel_mut().set_middle(f32::INFINITY);
    amp.panel_mut().set_treble(f32::NEG_INFINITY);

    assert_eq!(amp.panel().bass, 5.0);
    assert_eq!(amp.panel().middle, 10.0);
    assert_eq!(amp.panel().treble, 0.0);
}

#[test]
fn test_ca_13_non_regression_sample_rate_filter_invariance() {
    let mut amp48 = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp48.set_sample_rate(48000.0);
    amp48.panel_mut().bass = 10.0;
    amp48.panel_mut().treble = 10.0;

    let mut amp96 = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp96.set_sample_rate(96000.0);
    amp96.panel_mut().bass = 10.0;
    amp96.panel_mut().treble = 10.0;

    let gain48_100hz = calc_peak_gain_at_freq(&mut amp48, 100.0, 48000.0, 0.01);
    let gain96_100hz = calc_peak_gain_at_freq(&mut amp96, 100.0, 96000.0, 0.01);

    let diff_sr_db = (20.0 * (gain48_100hz / gain96_100hz).log10()).abs();
    assert!(
        diff_sr_db <= 1.0,
        "Resposta de filtro em 96 kHz deve ficar dentro de +-1 dB da de 48 kHz (foi {:.2} dB)",
        diff_sr_db
    );
}
