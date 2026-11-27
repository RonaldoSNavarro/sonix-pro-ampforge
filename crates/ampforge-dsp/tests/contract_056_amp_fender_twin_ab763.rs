//! Testes de contrato da Spec 056: Amplificador Fender Twin Reverb AB763
//! Cobre: CA-2 (Clean Headroom THD < 1.5%), CA-3 (Isolamento Normal vs Vibrato), CA-4 (Bright 120 pF)

use ampforge_dsp::amp::AmpInputJack;
use ampforge_dsp::amps::FenderTwinReverbAb763;
use ampforge_dsp::AudioProcessor;

fn compute_thd(samples: &[f32], freq: f64, sr: f64) -> f64 {
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
    let mut h_sq = 0.0;
    for k in 2..=5 {
        h_sq += mag(b1 * k).powi(2);
    }
    h_sq.sqrt() / h1
}

#[test]
fn ca2_crystal_clean_headroom_at_nominal_guitar_level() {
    let sr = 48_000.0;
    let mut amp = FenderTwinReverbAb763::new();
    amp.panel.input_gain = 5.0;
    amp.panel.bass = 5.0;
    amp.panel.middle = 5.0;
    amp.panel.treble = 5.0;
    amp.panel.reverb = 0.0;
    amp.panel.vibrato_depth = 0.0;

    let mut inp = vec![0.0f32; 4096];
    for (i, s) in inp.iter_mut().enumerate() {
        *s = 0.06 * (2.0 * std::f32::consts::PI * 1000.0 * (i as f32) / (sr as f32)).sin();
    }
    let mut out = vec![0.0f32; 4096];
    amp.process_mono_slice(&inp, &mut out);
    let thd = compute_thd(&out[2048..], 1000.0, sr);
    assert!(thd < 0.015, "Twin AB763 clean headroom THD={:.4} (esperado < 0.015)", thd);
}

#[test]
fn ca3_and_ca4_normal_isolation_and_bright_switch() {
    let mut amp = FenderTwinReverbAb763::new();
    amp.panel.input_jack = AmpInputJack::Normal;
    amp.panel.reverb = 0.0;
    amp.panel.vibrato_depth = 0.0;
    let inp = [0.1f32; 1024];
    let mut out_a = [0.0f32; 1024];
    amp.process_mono_slice(&inp, &mut out_a);

    amp.reset();
    amp.panel.reverb = 10.0;
    amp.panel.vibrato_depth = 10.0;
    let mut out_b = [0.0f32; 1024];
    amp.process_mono_slice(&inp, &mut out_b);

    let max_diff = out_a.iter().zip(out_b.iter()).map(|(a, b)| (a - b).abs()).fold(0.0f32, f32::max);
    assert!(max_diff <= 1e-6, "Normal channel must isolate reverb/tremolo: diff={}", max_diff);

    // Bright switch +120 pF
    let measure_tilt = |a: &mut FenderTwinReverbAb763| -> f64 {
        let mut s200 = vec![0.0f32; 4096];
        let mut s5k = vec![0.0f32; 4096];
        for i in 0..4096 {
            s200[i] = 0.02 * (2.0 * std::f32::consts::PI * 200.0 * (i as f32) / 48_000.0).sin();
            s5k[i] = 0.02 * (2.0 * std::f32::consts::PI * 5000.0 * (i as f32) / 48_000.0).sin();
        }
        let mut o200 = vec![0.0f32; 4096];
        let mut o5k = vec![0.0f32; 4096];
        a.process_mono_slice(&s200, &mut o200);
        a.process_mono_slice(&s5k, &mut o5k);
        let p200: f64 = o200[2048..].iter().map(|v| (*v as f64).powi(2)).sum();
        let p5k: f64 = o5k[2048..].iter().map(|v| (*v as f64).powi(2)).sum();
        10.0 * (p5k / p200.max(1e-18)).log10()
    };

    amp.panel.input_gain = 2.0;
    amp.panel.bright_switch = false;
    let t_off = measure_tilt(&mut amp);
    amp.panel.bright_switch = true;
    let t_on = measure_tilt(&mut amp);
    assert!(t_on - t_off >= 3.0, "Bright 120 pF deve elevar agudos >= 3 dB: {:.2} -> {:.2}", t_off, t_on);
}
