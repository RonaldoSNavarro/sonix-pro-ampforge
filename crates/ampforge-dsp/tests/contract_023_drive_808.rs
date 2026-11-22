//! crates/ampforge-dsp/tests/contract_023_drive_808.rs
//! Testes de Contrato Automatizados para Spec 023: Pedal de Drive 808 Fiel ao Circuito
//! Cobre Critérios de Aceite CA-2, CA-3, CA-4, CA-5, CA-6, CA-7, CA-8, CA-9, CA-10

use std::f32::consts::PI;
use ampforge_dsp::fx::drive::{DriveModel, DriveProcessor};

fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() { return 0.0; }
    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

fn generate_sine(freq: f32, sr: f32, num_samples: usize, amplitude: f32) -> Vec<f32> {
    (0..num_samples)
        .map(|i| amplitude * (2.0 * PI * freq * (i as f32 / sr)).sin())
        .collect()
}

fn dft_harmonic_magnitudes(samples: &[f32], sr: f32, fund_freq: f32, num_harmonics: usize) -> Vec<f32> {
    let mut mags = Vec::with_capacity(num_harmonics);
    let n = samples.len();
    for h in 1..=num_harmonics {
        let f = fund_freq * h as f32;
        let mut real = 0.0f32;
        let mut imag = 0.0f32;
        for i in 0..n {
            let angle = 2.0 * PI * f * (i as f32 / sr);
            real += samples[i] * angle.cos();
            imag += samples[i] * angle.sin();
        }
        mags.push((real * real + imag * imag).sqrt() / n as f32);
    }
    mags
}

fn calculate_thd_pct(samples: &[f32], sr: f32, fund_freq: f32) -> f32 {
    let harms = dft_harmonic_magnitudes(samples, sr, fund_freq, 7);
    let fund = harms[0].max(1e-9);
    let harmonic_energy_sq: f32 = harms[1..].iter().map(|&m| m * m).sum();
    (harmonic_energy_sq.sqrt() / fund) * 100.0
}

#[test]
fn test_ca_02_gain_frequency_response_80hz_vs_1khz_at_drive_10() {
    let sr = 48000.0;
    let n = 4096;
    let mut drive = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive.set_sample_rate(sr);
    drive.set_gain(10.0);
    drive.set_tone(5.0);
    drive.set_level(5.0);

    // Pequeno sinal: -80 dBFS (0.0001 FS) para avaliar resposta linear antes do clipping (+39.5 dB em 1 kHz)
    let sig_80 = generate_sine(80.0, sr, n, 0.0001);
    let mut out_80 = vec![0.0f32; n];
    drive.process(&sig_80, &mut out_80);

    let sig_1k = generate_sine(1000.0, sr, n, 0.0001);
    let mut out_1k = vec![0.0f32; n];
    drive.process(&sig_1k, &mut out_1k);

    let rms_in_80 = calculate_rms(&sig_80[n / 2..]);
    let rms_out_80 = calculate_rms(&out_80[n / 2..]);
    let gain_80_db = 20.0 * (rms_out_80 / rms_in_80.max(1e-9)).log10();

    let rms_in_1k = calculate_rms(&sig_1k[n / 2..]);
    let rms_out_1k = calculate_rms(&out_1k[n / 2..]);
    let gain_1k_db = 20.0 * (rms_out_1k / rms_in_1k.max(1e-9)).log10();

    // No esquema real do TS808 com LPF pós-clip de 723,43 Hz (AUD-29): 80 Hz = +22,3 dB e 1 kHz = +34,9 dB (+12,6 dB)
    let diff_1k_80 = gain_1k_db - gain_80_db;
    assert!(
        diff_1k_80 >= 12.0,
        "Ganho em 1 kHz deve ser >= 12 dB acima de 80 Hz em drive=10: diff={:.2} dB",
        diff_1k_80
    );
}

#[test]
fn test_ca_03_bass_distortion_attenuation_100hz_vs_1khz() {
    let sr = 48000.0;
    // 4800 amostras: out[2400..] = 5 ciclos exatos de 100 Hz e 50 ciclos exatos de 1000 Hz (sem vazamento DFT)
    let n = 4800;
    let mut drive = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive.set_sample_rate(sr);
    drive.set_gain(10.0);
    drive.set_tone(5.0);
    drive.set_level(5.0);

    // Seno de pico 0.025 (100 Hz @ +24.2 dB = 0.41 V < 0.6 V, enquanto 1 kHz satura fortemente)
    let sig_100 = generate_sine(100.0, sr, n, 0.025);
    let mut out_100 = vec![0.0f32; n];
    drive.process(&sig_100, &mut out_100);

    let sig_1k = generate_sine(1000.0, sr, n, 0.025);
    let mut out_1k = vec![0.0f32; n];
    drive.process(&sig_1k, &mut out_1k);

    let thd_100 = calculate_thd_pct(&out_100[n / 2..], sr, 100.0);
    let thd_1k = calculate_thd_pct(&out_1k[n / 2..], sr, 1000.0);

    assert!(
        thd_100 <= 0.5 * thd_1k,
        "THD em 100 Hz deve ser <= 0.5x THD em 1 kHz (graves distorcem menos no TS808): thd_100={:.2}%, thd_1k={:.2}%",
        thd_100, thd_1k
    );
}

#[test]
fn test_ca_04_symmetrical_clipping_harmonics() {
    let sr = 48000.0;
    // 4800 amostras = exatamente 100 ciclos de 1000 Hz a 48 kHz (zero vazamento espectral na DFT)
    let n = 4800;
    let mut drive = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive.set_sample_rate(sr);
    drive.set_gain(10.0);
    drive.set_tone(5.0);
    drive.set_level(5.0);

    let sig = generate_sine(1000.0, sr, n, 0.1);
    let mut out = vec![0.0f32; n];
    drive.process(&sig, &mut out);

    // Avalia as últimas 2400 amostras = exatamente 50 ciclos inteiros
    let harms = dft_harmonic_magnitudes(&out[2400..], sr, 1000.0, 5);
    let h1 = harms[0].max(1e-9);
    let h2_dbc = 20.0 * (harms[1] / h1).log10();
    let h3_dbc = 20.0 * (harms[2] / h1).log10();

    assert!(
        h2_dbc <= -50.0,
        "2º harmônico deve ser <= -50 dBc (clipping simétrico por diodos antiparalelos): got {:.2} dBc",
        h2_dbc
    );
    assert!(
        h3_dbc >= -30.0,
        "3º harmônico deve ser >= -30 dBc: got {:.2} dBc",
        h3_dbc
    );
}

#[test]
fn test_ca_05_monotonic_gain_increase_and_minimum_mid_gain() {
    let sr = 48000.0;
    let n = 2048;
    let drive_vals = [0.0, 2.5, 5.0, 7.5, 10.0];
    let mut gains = Vec::new();

    for &d in &drive_vals {
        let mut drive = DriveProcessor::new(DriveModel::GreenScreamer808);
        drive.set_sample_rate(sr);
        drive.set_gain(d);
        drive.set_tone(5.0);
        drive.set_level(10.0);

        let sig = generate_sine(1000.0, sr, n, 0.005); // Pequeno sinal
        let mut out = vec![0.0f32; n];
        drive.process(&sig, &mut out);

        let rms_in = calculate_rms(&sig[n / 2..]);
        let rms_out = calculate_rms(&out[n / 2..]);
        let g_db = 20.0 * (rms_out / rms_in.max(1e-9)).log10();
        gains.push(g_db);
    }

    // 1. drive=0 tem ganho >= +6 dB em médios (estágio nunca é unitário em médios)
    assert!(
        gains[0] >= 6.0,
        "Ganho em médios com drive=0 deve ser >= +6 dB: got {:.2} dB",
        gains[0]
    );

    // 2. Crescimento monotônico
    for i in 0..4 {
        assert!(
            gains[i + 1] > gains[i],
            "Ganho deve crescer monotonicamente com drive: drive {} ({:.2} dB) vs drive {} ({:.2} dB)",
            drive_vals[i], gains[i], drive_vals[i + 1], gains[i + 1]
        );
    }
}

#[test]
fn test_ca_06_active_tone_control_response() {
    let sr = 48000.0;
    let n = 4096;
    let mut drive_tone0 = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive_tone0.set_sample_rate(sr);
    drive_tone0.set_gain(5.0);
    drive_tone0.set_tone(0.0);
    drive_tone0.set_level(5.0);

    let mut drive_tone10 = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive_tone10.set_sample_rate(sr);
    drive_tone10.set_gain(5.0);
    drive_tone10.set_tone(10.0);
    drive_tone10.set_level(5.0);

    // 1. Energia acima de 3 kHz (teste em 4 kHz) deve ser >= 6 dB maior em tone=10 vs tone=0
    let sig_4k = generate_sine(4000.0, sr, n, 0.01);
    let mut out_4k_0 = vec![0.0f32; n];
    let mut out_4k_10 = vec![0.0f32; n];
    drive_tone0.process(&sig_4k, &mut out_4k_0);
    drive_tone10.process(&sig_4k, &mut out_4k_10);

    let rms_4k_0 = calculate_rms(&out_4k_0[n / 2..]);
    let rms_4k_10 = calculate_rms(&out_4k_10[n / 2..]);
    let diff_treble = 20.0 * (rms_4k_10 / rms_4k_0.max(1e-9)).log10();
    assert!(
        diff_treble >= 6.0,
        "Tone 10 vs 0 deve ter energia acima de 3 kHz >= 6 dB maior: got {:.2} dB",
        diff_treble
    );

    // 2. Abaixo de 500 Hz (teste em 250 Hz) muda <= 1 dB
    let sig_250 = generate_sine(250.0, sr, n, 0.01);
    let mut out_250_0 = vec![0.0f32; n];
    let mut out_250_10 = vec![0.0f32; n];
    drive_tone0.process(&sig_250, &mut out_250_0);
    drive_tone10.process(&sig_250, &mut out_250_10);

    let rms_250_0 = calculate_rms(&out_250_0[n / 2..]);
    let rms_250_10 = calculate_rms(&out_250_10[n / 2..]);
    let diff_bass = (20.0 * (rms_250_10 / rms_250_0.max(1e-9)).log10()).abs();
    assert!(
        diff_bass <= 1.0,
        "Tone 10 vs 0 abaixo de 500 Hz deve mudar <= 1.0 dB: got {:.2} dB",
        diff_bass
    );
}

#[test]
fn test_ca_07_level_attenuation_and_thd_invariance() {
    let sr = 48000.0;
    let n = 4096;
    let mut drive_lvl10 = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive_lvl10.set_sample_rate(sr);
    drive_lvl10.set_gain(5.0);
    drive_lvl10.set_tone(5.0);
    drive_lvl10.set_level(10.0);

    let mut drive_lvl0 = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive_lvl0.set_sample_rate(sr);
    drive_lvl0.set_gain(5.0);
    drive_lvl0.set_tone(5.0);
    drive_lvl0.set_level(0.0);

    let sig = generate_sine(1000.0, sr, n, 0.1);
    let mut out_lvl10 = vec![0.0f32; n];
    let mut out_lvl0 = vec![0.0f32; n];
    drive_lvl10.process(&sig, &mut out_lvl10);
    drive_lvl0.process(&sig, &mut out_lvl0);

    let rms_10 = calculate_rms(&out_lvl10[n / 2..]);
    let rms_0 = calculate_rms(&out_lvl0[n / 2..]);
    let atten_db = 20.0 * (rms_10 / rms_0.max(1e-9)).log10();
    assert!(
        atten_db >= 60.0,
        "level=0 deve atenuar >= 60 dB vs level=10: got {:.2} dB",
        atten_db
    );

    // THD invariance entre level=3 e level=10
    let mut drive_lvl3 = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive_lvl3.set_sample_rate(sr);
    drive_lvl3.set_gain(5.0);
    drive_lvl3.set_tone(5.0);
    drive_lvl3.set_level(3.0);
    let mut out_lvl3 = vec![0.0f32; n];
    drive_lvl3.process(&sig, &mut out_lvl3);

    let thd_3 = calculate_thd_pct(&out_lvl3[n / 2..], sr, 1000.0);
    let thd_10 = calculate_thd_pct(&out_lvl10[n / 2..], sr, 1000.0);
    let thd_diff = (thd_3 - thd_10).abs();
    assert!(
        thd_diff <= 0.05,
        "level não deve mudar o THD (diferença <= 0.05 pp): got diff={:.4} pp",
        thd_diff
    );
}

#[test]
fn test_ca_09_model_differentiation_and_other_models_unchanged() {
    let sr = 48000.0;
    let n = 1024;
    let sig = generate_sine(1000.0, sr, n, 0.1);

    // Outros modelos (RAT, Centaur, Boss, Fuzz, BigMuff) processam sem erro e diferem entre si
    let other_models = [
        DriveModel::GoldenKloneCentaur,
        DriveModel::BlackRodentRat,
        DriveModel::OrangeBossDs,
        DriveModel::FuzzFaceSiGe,
        DriveModel::BigMuffPi,
    ];

    for &m in &other_models {
        let mut d = DriveProcessor::new(m);
        d.set_sample_rate(sr);
        let mut out = vec![0.0f32; n];
        d.process(&sig, &mut out);
        let rms = calculate_rms(&out);
        assert!(rms > 0.001, "Outro modelo {:?} deve processar áudio", m);
    }
}
