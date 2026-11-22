//! Testes de contrato da auditoria 2026-11-20 (Fase 5 — Efeitos).

use ampforge_dsp::fx::delay::{DelayModel, DelayProcessor};
use ampforge_dsp::regua::cents;

fn rms(buf: &[f32]) -> f32 {
    if buf.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = buf.iter().map(|&x| (x as f64) * (x as f64)).sum();
    (sum_sq / buf.len() as f64).sqrt() as f32
}

#[test]
fn aud_28_cabecas_por_modo() {
    let sr = 48_000.0f32;
    let delay_ms = 100.0f32;
    let d1 = (delay_ms * 0.001 * sr).round() as usize; // 4800
    let d2 = ((delay_ms * 0.001 * sr) * 1.92).round() as usize; // 9216
    let d3 = ((delay_ms * 0.001 * sr) * 2.83).round() as usize; // 13584
    let len = 24_000usize;

    let mut impulse = vec![0.0f32; len];
    impulse[0] = 1.0;

    // Tabela de referência RE-201 (OS §AUD-28):
    // 1..4 só eco; 5..7 cabeça única + mola; 8..10 pares + mola; 11 todas + mola.
    let expected: [(u8, (bool, bool, bool), bool); 11] = [
        (1, (true, false, false), false),
        (2, (false, true, false), false),
        (3, (false, false, true), false),
        (4, (false, true, true), false),
        (5, (true, false, false), true),
        (6, (false, true, false), true),
        (7, (false, false, true), true),
        (8, (true, true, false), true),
        (9, (false, true, true), true),
        (10, (true, false, true), true),
        (11, (true, true, true), true),
    ];

    let mut seen_without_spring: Vec<(bool, bool, bool)> = Vec::new();
    let mut seen_with_spring: Vec<(bool, bool, bool)> = Vec::new();

    for &(mode, (exp_h1, exp_h2, exp_h3), has_spring) in &expected {
        let mut d = DelayProcessor::new(DelayModel::MultiHeadTapeEcho);
        d.set_sample_rate(sr);
        d.set_delay_time_ms(delay_ms);
        d.set_feedback_pct(0.0);
        d.set_mix_pct(100.0);
        d.set_reverb_volume(0.0);
        d.set_re201_mode(mode);

        let mut out = vec![0.0f32; len];
        d.process(&impulse, &mut out);

        let peak_in_pm1 = |center: usize| -> f32 {
            out[center.saturating_sub(1)..=(center + 1).min(len - 1)]
                .iter()
                .map(|x| x.abs())
                .fold(0.0f32, f32::max)
        };
        let peak_in_pm6 = |center: usize| -> f32 {
            out[center.saturating_sub(6)..=(center + 6).min(len - 1)]
                .iter()
                .map(|x| x.abs())
                .fold(0.0f32, f32::max)
        };

        let n_active = [exp_h1, exp_h2, exp_h3].iter().filter(|&&b| b).count() as f32;
        let expected_w = 1.0 / n_active.sqrt();

        for (name, exp_on, center) in [("H1", exp_h1, d1), ("H2", exp_h2, d2), ("H3", exp_h3, d3)] {
            let p_pm1 = peak_in_pm1(center);
            let p_pm6 = peak_in_pm6(center);
            if exp_on {
                assert!(
                    p_pm1 > expected_w * 0.6 && (p_pm6 - p_pm1).abs() < 1e-4,
                    "AUD-28 modo {mode} {name}: esperado pico em {center} ± 1 amostra com ganho ~{expected_w:.3}, obtido pm1={p_pm1:.4}, pm6={p_pm6:.4}"
                );
            } else {
                let db = if p_pm6 > 0.0 {
                    20.0 * p_pm6.log10()
                } else {
                    -160.0
                };
                assert!(
                    db <= -60.0,
                    "AUD-28 modo {mode} {name}: cabeça inativa deveria ficar <= -60 dB, obtido {db:.2} dB (pico={p_pm6:.6})"
                );
            }
        }

        let obs = (
            peak_in_pm1(d1) > 0.1,
            peak_in_pm1(d2) > 0.1,
            peak_in_pm1(d3) > 0.1,
        );
        if has_spring {
            assert!(
                !seen_with_spring.contains(&obs),
                "AUD-28 modo {mode}: conjunto de cabeças com mola duplicado {obs:?}"
            );
            seen_with_spring.push(obs);
        } else {
            assert!(
                !seen_without_spring.contains(&obs),
                "AUD-28 modo {mode}: conjunto de cabeças sem mola duplicado {obs:?}"
            );
            seen_without_spring.push(obs);
        }
    }
}

#[test]
fn aud_28_mola_so_de_5_a_12() {
    let sr = 48_000.0f32;
    let len = 24_000usize;
    let mut input = vec![0.0f32; len];
    for i in 0..480 {
        input[i] = 0.5 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sr).sin();
    }

    for mode in 1u8..=12 {
        let run_with_rev = |rev_vol: f32| -> Vec<f32> {
            let mut d = DelayProcessor::new(DelayModel::MultiHeadTapeEcho);
            d.set_sample_rate(sr);
            d.set_delay_time_ms(120.0);
            d.set_feedback_pct(25.0);
            d.set_mix_pct(50.0);
            d.set_re201_mode(mode);
            d.set_reverb_volume(rev_vol);
            let mut out = vec![0.0f32; len];
            d.process(&input, &mut out);
            out
        };

        let out_0 = run_with_rev(0.0);
        let out_10 = run_with_rev(10.0);
        let max_diff = out_0
            .iter()
            .zip(out_10.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);

        if mode <= 4 {
            assert!(
                max_diff <= 1e-6,
                "AUD-28 modo {mode}: modos 1..4 não devem ter mola (max_diff={max_diff})"
            );
        } else {
            assert!(
                max_diff > 1e-3,
                "AUD-28 modo {mode}: modos 5..12 devem diferir com reverb_volume 0 vs 10 (max_diff={max_diff})"
            );
        }
    }
}

#[test]
fn aud_28_mola_tem_cauda() {
    let sr = 48_000.0f32;
    let len = 48_000usize; // 1.0 s
    let mut impulse = vec![0.0f32; len];
    impulse[0] = 1.0;

    for mode in [5u8, 12u8] {
        let mut d = DelayProcessor::new(DelayModel::MultiHeadTapeEcho);
        d.set_sample_rate(sr);
        d.set_delay_time_ms(120.0);
        d.set_feedback_pct(0.0);
        d.set_mix_pct(100.0);
        d.set_re201_mode(mode);
        d.set_reverb_volume(10.0);

        let mut out = vec![0.0f32; len];
        d.process(&impulse, &mut out);

        let peak = out.iter().map(|x| x.abs()).fold(0.0f32, f32::max).max(1e-9);
        let tail_rms = rms(&out[24_000..48_000]);
        let tail_db = 20.0 * (tail_rms.max(1e-12) / peak).log10();
        assert!(
            tail_db >= -60.0,
            "AUD-28 modo {mode}: energia da cauda da mola entre 0.5 e 1.0 s deveria ser >= -60 dB do pico, obtido {tail_db:.2} dB"
        );
    }
}

#[test]
fn aud_28_wow_audivel() {
    let sr = 48_000.0f32;
    let f0 = 1000.0f64;
    let total = 96_000usize; // 2.0 s
    let input: Vec<f32> = (0..total)
        .map(|i| 0.5 * (2.0 * std::f64::consts::PI * f0 * i as f64 / sr as f64).sin() as f32)
        .collect();

    for mode in [1u8, 2u8] {
        let mut d = DelayProcessor::new(DelayModel::MultiHeadTapeEcho);
        d.set_sample_rate(sr);
        d.set_delay_time_ms(120.0);
        d.set_feedback_pct(0.0);
        d.set_mix_pct(100.0);
        d.set_reverb_volume(0.0);
        d.set_re201_mode(mode);

        let mut out = vec![0.0f32; total];
        d.process(&input, &mut out);

        // Demodulação em quadratura em torno de f0 = 1 kHz com média de 1 ciclo (48 amostras = anula 2*f0)
        let cycle = 48usize; // sr / f0 exacto
        let mut i_comp = vec![0.0f64; total];
        let mut q_comp = vec![0.0f64; total];
        for n in 0..total {
            let phase = 2.0 * std::f64::consts::PI * f0 * n as f64 / sr as f64;
            i_comp[n] = (out[n] as f64) * phase.cos();
            q_comp[n] = -(out[n] as f64) * phase.sin();
        }

        let mut phase_bb = vec![0.0f64; total];
        let mut sum_i = 0.0f64;
        let mut sum_q = 0.0f64;
        for n in 0..total {
            sum_i += i_comp[n];
            sum_q += q_comp[n];
            if n >= cycle {
                sum_i -= i_comp[n - cycle];
                sum_q -= q_comp[n - cycle];
            }
            phase_bb[n] = sum_q.atan2(sum_i);
        }

        // Janela estável após atraso + acomodação do detector (0.3 s .. 1.9 s)
        let mut peak_cents = 0.0f64;
        let step = cycle;
        for n in (14_400 + step)..91_200 {
            let mut dp = phase_bb[n] - phase_bb[n - step];
            while dp > std::f64::consts::PI {
                dp -= 2.0 * std::f64::consts::PI;
            }
            while dp < -std::f64::consts::PI {
                dp += 2.0 * std::f64::consts::PI;
            }
            let delta_f = (dp / step as f64) * (sr as f64) / (2.0 * std::f64::consts::PI);
            let dev_cents = cents(f0 + delta_f, f0).abs();
            if dev_cents > peak_cents {
                peak_cents = dev_cents;
            }
        }

        assert!(
            (peak_cents - 3.0).abs() <= 0.3,
            "AUD-28 modo {mode}: desvio de pico esperado de 3.0 ± 0.3 cents a 1 kHz, obtido {peak_cents:.3} cents"
        );
    }
}
