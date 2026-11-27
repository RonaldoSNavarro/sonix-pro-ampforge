//! Testes de contrato da Spec 055: Amplificador Marshall JCM800 2203 (Rev. 1981)
//! Cobre: CA-2 (Cold Clipper V1b H2), CA-3 (High vs Low), CA-4 (Pre-Amp vs Master Volume)

use ampforge_dsp::amp::AmpInputJack;
use ampforge_dsp::amps::MarshallJcm800_2203;

fn dft_harmonics(samples: &[f32], freq: f64, sr: f64) -> (f64, f64, f64) {
    let n = samples.len();
    let mag = |bin: usize| -> f64 {
        let mut re = 0.0;
        let mut im = 0.0;
        for (i, &x) in samples.iter().enumerate() {
            let ang = 2.0 * std::f64::consts::PI * (bin as f64) * (i as f64) / (n as f64);
            re += (x as f64) * ang.cos();
            im -= (x as f64) * ang.sin();
        }
        (re * re + im * im).sqrt()
    };
    let b1 = (freq * (n as f64) / sr).round() as usize;
    let h1 = mag(b1).max(1e-12);
    let h2 = mag(b1 * 2);
    let mut h_sq = 0.0;
    for k in 2..=5 {
        h_sq += mag(b1 * k).powi(2);
    }
    (20.0 * (h2 / h1).max(1e-12).log10(), h_sq.sqrt() / h1, h1)
}

#[test]
fn ca2_and_ca3_cold_clipper_h2_and_high_vs_low_input() {
    let sr = 48_000.0;
    let mut amp = MarshallJcm800_2203::new();
    amp.panel.input_jack = AmpInputJack::High;
    amp.panel.input_gain = 8.0;
    amp.panel.master_volume = 5.0;

    let mut inp = vec![0.0f32; 4096];
    for (i, s) in inp.iter_mut().enumerate() {
        *s = 0.2512 * (2.0 * std::f32::consts::PI * 1000.0 * (i as f32) / (sr as f32)).sin();
    }
    let mut out_hi = vec![0.0f32; 4096];
    amp.process_mono_slice(&inp, &mut out_hi);
    let (h2_rel_db, thd_hi, _) = dft_harmonics(&out_hi[2048..], 1000.0, sr);
    assert!(h2_rel_db >= -24.0, "Cold Clipper H2/H1 = {:.2} dB (esperado >= -24 dB)", h2_rel_db);

    amp.panel.input_jack = AmpInputJack::Low;
    let mut out_lo = vec![0.0f32; 4096];
    amp.process_mono_slice(&inp, &mut out_lo);
    let (_, thd_lo, _) = dft_harmonics(&out_lo[2048..], 1000.0, sr);
    assert!(
        thd_lo <= thd_hi * 0.70,
        "Entrada LOW deve reduzir THD >= 30%: hi={:.4}, lo={:.4}",
        thd_hi, thd_lo
    );
}

#[test]
fn ca4_master_volume_silences_at_zero_and_drives_power_amp() {
    let mut amp = MarshallJcm800_2203::new();
    amp.panel.input_gain = 4.0;
    amp.panel.master_volume = 0.0;
    let mut inp = [0.0f32; 512];
    for (i, s) in inp.iter_mut().enumerate() {
        *s = 0.25 * (2.0 * std::f32::consts::PI * 1000.0 * (i as f32) / 48_000.0).sin();
    }
    let mut out0 = [0.0f32; 512];
    amp.process_mono_slice(&inp, &mut out0);
    let max0 = out0.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
    assert!(max0 < 1e-6, "Master=0.0 deve silenciar, max={}", max0);

    amp.panel.master_volume = 9.0;
    let mut out9 = [0.0f32; 512];
    amp.process_mono_slice(&inp, &mut out9);
    let max9 = out9.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
    assert!(max9 > 0.1);
}
