//! Testes de contrato da auditoria 2026-11-20 (Fase 5 — Efeitos).

use ampforge_dsp::amp::{Ac30VibTremMode, AmpInputJack, AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::fx::delay::{DelayModel, DelayProcessor};
use ampforge_dsp::fx::drive::{DECISAO_SONIX_LM308_GBW, DriveModel, DriveProcessor};
use ampforge_dsp::fx::modulation::{
    DECISAO_SONIX_LDR_ATTACK_MS, DECISAO_SONIX_LDR_RELEASE_MS, DECISAO_SONIX_ROTARY_DRUM_FAST_HZ,
    DECISAO_SONIX_ROTARY_DRUM_SLOW_HZ, DECISAO_SONIX_ROTARY_HORN_FAST_HZ,
    DECISAO_SONIX_ROTARY_HORN_SLOW_HZ, ModModel, ModulationProcessor,
};
use ampforge_dsp::regua::{cents, ganho_db};
use ampforge_dsp::unidades::VOLTS_POR_FS;

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

type Cx = (f64, f64);

#[inline]
fn cadd(a: Cx, b: Cx) -> Cx {
    (a.0 + b.0, a.1 + b.1)
}

#[inline]
fn cmul(a: Cx, b: Cx) -> Cx {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

#[inline]
fn cdiv(a: Cx, b: Cx) -> Cx {
    let d = b.0 * b.0 + b.1 * b.1;
    ((a.0 * b.0 + a.1 * b.1) / d, (a.1 * b.0 - a.0 * b.1) / d)
}

#[inline]
fn cpar(a: Cx, b: Cx) -> Cx {
    cdiv(cmul(a, b), cadd(a, b))
}

#[inline]
fn cabs(a: Cx) -> f64 {
    a.0.hypot(a.1)
}

fn ts808_stage_transfer(d: f64, freq_hz: f64) -> Cx {
    let w = 2.0 * std::f64::consts::PI * freq_hz;
    let rf = 51_000.0 + d * 500_000.0;
    let zf = cpar((rf, 0.0), (0.0, -1.0 / (w * 51e-12)));
    let zg = (4_700.0, -1.0 / (w * 0.047e-6));
    cadd((1.0, 0.0), cdiv(zf, zg))
}

fn thd_pct_dft(samples: &[f32], sr: f32, fund_hz: f64) -> f64 {
    let n = samples.len();
    let mut harms = [0.0f64; 7];
    for (idx, h_out) in harms.iter_mut().enumerate() {
        let f = fund_hz * (idx + 1) as f64;
        let omega = 2.0 * std::f64::consts::PI * f / (sr as f64);
        let mut re = 0.0f64;
        let mut im = 0.0f64;
        for (i, &s) in samples.iter().enumerate() {
            let ang = omega * i as f64;
            re += (s as f64) * ang.cos();
            im -= (s as f64) * ang.sin();
        }
        *h_out = re.hypot(im) / n as f64;
    }
    let h1 = harms[0].max(1e-12);
    let h_sq: f64 = harms[1..].iter().map(|h| h * h).sum();
    (h_sq.sqrt() / h1) * 100.0
}

#[test]
fn aud_29_ts808_ganho_do_esquema() {
    let sr = 48_000.0f32;
    let len = 48_000usize;
    let amp = 1e-4f32; // -80 dBFS
    let level_unity = (10.0f32 / 1.5f32.sqrt()).clamp(0.0, 10.0); // 8.164966 -> out_level_taper = 1.0

    for &d in &[0.0f64, 0.5, 1.0] {
        for &freq in &[100.0f64, 1_000.0, 3_000.0] {
            let inp: Vec<f32> = (0..len)
                .map(|i| amp * (2.0 * std::f64::consts::PI * freq * i as f64 / sr as f64).sin() as f32)
                .collect();
            let mut drv = DriveProcessor::new(DriveModel::GreenScreamer808);
            drv.set_sample_rate(sr);
            drv.set_gain((d * 10.0) as f32);
            drv.set_tone(5.0);
            drv.set_level(level_unity);

            let mut out = vec![0.0f32; len];
            drv.process(&inp, &mut out);

            let g_stage = ts808_stage_transfer(d, freq);
            let w = 2.0 * std::f64::consts::PI * freq;
            let h_lpf = cdiv((1.0, 0.0), (1.0, w * 1_000.0 * 0.22e-6));
            let expected_db = 20.0 * cabs(cmul(g_stage, h_lpf)).log10();
            let measured_db = ganho_db(&inp, &out, sr, freq as f32);

            assert!(
                (measured_db - expected_db).abs() <= 1.0,
                "AUD-29 TS808 D={d} f={freq}Hz: esperado {expected_db:.2} dB (±1 dB), obtido {measured_db:.2} dB"
            );
        }
    }
}

#[test]
fn aud_29_rat_ganho_do_esquema() {
    let sr = 48_000.0f32;
    let len = 48_000usize;
    let amp = 1e-4f32; // -80 dBFS
    let level_unity = (10.0f32 / 1.5f32.sqrt()).clamp(0.0, 10.0);
    let gbw_hz = DECISAO_SONIX_LM308_GBW as f64;

    for &d in &[0.0f64, 0.5, 1.0] {
        for &freq in &[100.0f64, 1_000.0, 3_000.0] {
            let inp: Vec<f32> = (0..len)
                .map(|i| amp * (2.0 * std::f64::consts::PI * freq * i as f64 / sr as f64).sin() as f32)
                .collect();
            let mut drv = DriveProcessor::new(DriveModel::BlackRodentRat);
            drv.set_sample_rate(sr);
            drv.set_gain((d * 10.0) as f32);
            drv.set_tone(0.0); // Filter aberto (~32,15 kHz)
            drv.set_level(level_unity);

            let mut out = vec![0.0f32; len];
            drv.process(&inp, &mut out);

            let taper = if d <= 0.0 {
                0.0
            } else {
                d.powf(3.321928094887362)
            };
            let rd = 100_000.0 * taper;
            let w = 2.0 * std::f64::consts::PI * freq;
            let z1 = (47.0, -1.0 / (w * 2.2e-6));
            let z2 = (560.0, -1.0 / (w * 4.7e-6));
            let zg = cpar(z1, z2);
            let g_ideal = cadd((1.0, 0.0), cdiv((rd, 0.0), zg));
            let a_ol = (0.0, -gbw_hz / freq);
            let g_cl = cdiv(g_ideal, cadd((1.0, 0.0), cdiv(g_ideal, a_ol)));
            let expected_db = 20.0 * cabs(g_cl).log10();
            let measured_db = ganho_db(&inp, &out, sr, freq as f32);

            assert!(
                (measured_db - expected_db).abs() <= 1.0,
                "AUD-29 RAT D={d} f={freq}Hz: esperado {expected_db:.2} dB (±1 dB), obtido {measured_db:.2} dB"
            );
        }
    }
}

#[test]
fn aud_29_ds1_simetrico() {
    let sr = 48_000.0f32;
    let n = 9_600usize;
    let fund = 1_000.0f64;
    let inp: Vec<f32> = (0..n)
        .map(|i| 0.25 * (2.0 * std::f64::consts::PI * fund * i as f64 / sr as f64).sin() as f32)
        .collect();

    let mut drv = DriveProcessor::new(DriveModel::OrangeBossDs);
    drv.set_sample_rate(sr);
    drv.set_gain(10.0);
    drv.set_tone(5.0);
    drv.set_level(5.0);

    let mut out = vec![0.0f32; n];
    drv.process(&inp, &mut out);

    let seg = &out[4_800..]; // 100 ciclos exatos
    let mag_at = |h: usize| -> f64 {
        let f = fund * h as f64;
        let omega = 2.0 * std::f64::consts::PI * f / sr as f64;
        let mut re = 0.0f64;
        let mut im = 0.0f64;
        for (i, &s) in seg.iter().enumerate() {
            let ang = omega * i as f64;
            re += (s as f64) * ang.cos();
            im -= (s as f64) * ang.sin();
        }
        re.hypot(im) / seg.len() as f64
    };

    let h1 = mag_at(1).max(1e-12);
    for h_even in [2usize, 4, 6] {
        let he_dbc = 20.0 * (mag_at(h_even).max(1e-12) / h1).log10();
        assert!(
            he_dbc <= -40.0,
            "AUD-29 DS-1 harmônico par H{h_even} deveria ser <= -40 dBc (clip simétrico), obtido {he_dbc:.2} dBc"
        );
    }
}

#[test]
fn aud_29_limiar_em_volts() {
    let sr = 48_000.0f32;
    let n = 9_600usize;
    let fund = 1_000.0f64;
    let level_unity = (10.0f32 / 1.5f32.sqrt()).clamp(0.0, 10.0);
    let target_amp = 0.6 / cabs(ts808_stage_transfer(0.0, fund)) / (VOLTS_POR_FS as f64);

    let mut db_rel = -12.0f64;
    let mut found_amp: Option<f64> = None;
    while db_rel <= 12.0 {
        let amp = target_amp * 10.0f64.powf(db_rel / 20.0);
        let inp: Vec<f32> = (0..n)
            .map(|i| (amp * (2.0 * std::f64::consts::PI * fund * i as f64 / sr as f64).sin()) as f32)
            .collect();
        let mut drv = DriveProcessor::new(DriveModel::GreenScreamer808);
        drv.set_sample_rate(sr);
        drv.set_gain(0.0);
        drv.set_tone(5.0);
        drv.set_level(level_unity);
        let mut out = vec![0.0f32; n];
        drv.process(&inp, &mut out);

        let thd = thd_pct_dft(&out[4_800..], sr, fund);
        if thd >= 1.0 {
            found_amp = Some(amp);
            break;
        }
        db_rel += 0.25;
    }

    let measured_amp = found_amp.expect("AUD-29: TS808 não atingiu THD = 1% na varredura");
    let err_db = 20.0 * (measured_amp / target_amp).log10();
    assert!(
        err_db.abs() <= 2.0,
        "AUD-29: início do clip (THD=1%) em D=0 esperado a ±2 dB de {target_amp:.5} FS, medido {measured_amp:.5} FS ({err_db:+.2} dB)"
    );
}

fn measure_am_rate_hz(buf: &[f32], sr: f32) -> f32 {
    let win = (0.015 * sr) as usize; // 15 ms RMS window
    let step = (0.002 * sr) as usize; // 2 ms hop
    let mut env = Vec::new();
    let mut i = 0;
    while i + win <= buf.len() {
        env.push(rms(&buf[i..i + win]));
        i += step;
    }
    let min_e = env.iter().copied().fold(f32::MAX, f32::min);
    let max_e = env.iter().copied().fold(0.0f32, f32::max);
    let mid = 0.5 * (min_e + max_e);
    let mut crossings = Vec::new();
    for k in 1..env.len() {
        if env[k - 1] < mid && env[k] >= mid {
            let frac = (mid - env[k - 1]) / (env[k] - env[k - 1]).max(1e-9);
            crossings.push((k as f32 - 1.0 + frac) * (step as f32 / sr));
        }
    }
    assert!(
        crossings.len() >= 2,
        "AUD-30: cruzamentos insuficientes para medir taxa AM ({})",
        crossings.len()
    );
    let mut periods = Vec::new();
    for w in crossings.windows(2) {
        periods.push(w[1] - w[0]);
    }
    let avg_period = periods.iter().sum::<f32>() / periods.len() as f32;
    1.0 / avg_period
}

fn measure_freq_zero_crossings_hz(buf: &[f32], sr: f32, win_samples: usize) -> Vec<f64> {
    let mut freqs = Vec::new();
    let mut start = 0usize;
    while start + win_samples <= buf.len() {
        let seg = &buf[start..start + win_samples];
        let mut times = Vec::new();
        for k in 1..seg.len() {
            if seg[k - 1] <= 0.0 && seg[k] > 0.0 {
                let frac = -seg[k - 1] as f64 / (seg[k] as f64 - seg[k - 1] as f64).max(1e-12);
                times.push((k as f64 - 1.0 + frac) / sr as f64);
            }
        }
        if times.len() >= 3 {
            let total_t = times[times.len() - 1] - times[0];
            let cycles = (times.len() - 1) as f64;
            freqs.push(cycles / total_t);
        }
        start += win_samples / 2;
    }
    freqs
}

#[test]
fn aud_30_tzf_cancela_no_cruzamento() {
    let sr = 48_000.0f32;
    let total = (sr * 11.0) as usize; // cruza 1,0 ms em t = 10 s (LFO 0,05 Hz)
    let mut seed: u64 = 0x1234_5678_9ABC_DEF0;
    let mut noise = Vec::with_capacity(total);
    for _ in 0..total {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let u = (((seed >> 33) as f32) / (u32::MAX as f32)) * 2.0 - 1.0;
        noise.push(u * 0.4);
    }

    let mut m = ModulationProcessor::new(ModModel::ThroughZeroFlanger);
    m.set_sample_rate(sr);
    m.set_rate_hz(0.05);
    m.set_depth(10.0);
    m.set_mix_pct(50.0);
    m.set_feedback(0.0);
    m.set_tzf_inverted_polarity(true);

    let mut out = vec![0.0f32; total];
    m.process(&noise, &mut out);

    let win = (0.002 * sr) as usize; // 2 ms = 96 amostras
    let mut min_ratio_db = f32::INFINITY;
    for s in (4_800..total - win).step_by(24) {
        let r_in = rms(&noise[s..s + win]).max(1e-9);
        let r_out = rms(&out[s..s + win]).max(1e-12);
        let db = 20.0 * (r_out / r_in).log10();
        if db < min_ratio_db {
            min_ratio_db = db;
        }
    }
    assert!(
        min_ratio_db <= -30.0,
        "AUD-30 TZF: janela de 2 ms com menor RMS deveria cancelar <= -30 dB, obtido {min_ratio_db:.2} dB"
    );
}

#[test]
fn aud_30_rotary_dois_rotores() {
    let sr = 48_000.0f32;
    let len = (sr * 6.0) as usize;
    let sine_200: Vec<f32> = (0..len)
        .map(|i| 0.4 * (2.0 * std::f32::consts::PI * 200.0 * i as f32 / sr).sin())
        .collect();
    let sine_3k: Vec<f32> = (0..len)
        .map(|i| 0.4 * (2.0 * std::f32::consts::PI * 3000.0 * i as f32 / sr).sin())
        .collect();

    // Velocidade lenta
    let mut r_drum = ModulationProcessor::new(ModModel::RotarySpeaker);
    r_drum.set_sample_rate(sr);
    r_drum.set_depth(10.0);
    r_drum.set_mix_pct(100.0);
    r_drum.set_rotary_fast(false);
    let mut out_200 = vec![0.0f32; len];
    r_drum.process(&sine_200, &mut out_200);
    let am_200_slow = measure_am_rate_hz(&out_200[24_000..], sr);
    let err_drum_slow = ((am_200_slow - DECISAO_SONIX_ROTARY_DRUM_SLOW_HZ)
        / DECISAO_SONIX_ROTARY_DRUM_SLOW_HZ)
        .abs();
    assert!(
        err_drum_slow <= 0.05,
        "AUD-30 Rotary: taxa AM em 200 Hz ({am_200_slow:.3} Hz) deve igualar tambor lento ({DECISAO_SONIX_ROTARY_DRUM_SLOW_HZ:.3} Hz ± 5%)"
    );

    let mut r_horn = ModulationProcessor::new(ModModel::RotarySpeaker);
    r_horn.set_sample_rate(sr);
    r_horn.set_depth(10.0);
    r_horn.set_mix_pct(100.0);
    r_horn.set_rotary_fast(false);
    let mut out_3k = vec![0.0f32; len];
    r_horn.process(&sine_3k, &mut out_3k);
    let am_3k_slow = measure_am_rate_hz(&out_3k[24_000..], sr);
    let err_horn_slow = ((am_3k_slow - DECISAO_SONIX_ROTARY_HORN_SLOW_HZ)
        / DECISAO_SONIX_ROTARY_HORN_SLOW_HZ)
        .abs();
    assert!(
        err_horn_slow <= 0.05,
        "AUD-30 Rotary: taxa AM em 3 kHz ({am_3k_slow:.3} Hz) deve igualar corneta lenta ({DECISAO_SONIX_ROTARY_HORN_SLOW_HZ:.3} Hz ± 5%)"
    );

    // Transição lenta -> rápida: corneta atinge 90% da nova velocidade antes do tambor
    let mut r_trans = ModulationProcessor::new(ModModel::RotarySpeaker);
    r_trans.set_sample_rate(sr);
    r_trans.set_depth(10.0);
    r_trans.set_mix_pct(100.0);
    r_trans.set_rotary_fast(false);
    r_trans.set_rotary_fast(true);

    let horn_target_90 = DECISAO_SONIX_ROTARY_HORN_SLOW_HZ
        + 0.90 * (DECISAO_SONIX_ROTARY_HORN_FAST_HZ - DECISAO_SONIX_ROTARY_HORN_SLOW_HZ);
    let drum_target_90 = DECISAO_SONIX_ROTARY_DRUM_SLOW_HZ
        + 0.90 * (DECISAO_SONIX_ROTARY_DRUM_FAST_HZ - DECISAO_SONIX_ROTARY_DRUM_SLOW_HZ);

    let step_blk = 480usize; // 10 ms
    let inp_blk = vec![0.1f32; step_blk];
    let mut out_blk = vec![0.0f32; step_blk];
    let mut t_horn_90: Option<f32> = None;
    let mut t_drum_90: Option<f32> = None;
    for b in 0..1200 {
        r_trans.process(&inp_blk, &mut out_blk);
        let t_s = (b + 1) as f32 * (step_blk as f32 / sr);
        if t_horn_90.is_none() && r_trans.rotary_horn_hz() >= horn_target_90 {
            t_horn_90 = Some(t_s);
        }
        if t_drum_90.is_none() && r_trans.rotary_drum_hz() >= drum_target_90 {
            t_drum_90 = Some(t_s);
        }
    }
    let th = t_horn_90.expect("AUD-30 Rotary: corneta não atingiu 90% da velocidade rápida");
    let td = t_drum_90.expect("AUD-30 Rotary: tambor não atingiu 90% da velocidade rápida");
    assert!(
        th < td,
        "AUD-30 Rotary: corneta ({th:.2}s) deveria atingir 90% antes do tambor ({td:.2}s)"
    );
}

#[test]
fn aud_30_optico_assimetrico() {
    let _ = (DECISAO_SONIX_LDR_ATTACK_MS, DECISAO_SONIX_LDR_RELEASE_MS);
    let sr = 48_000.0f32;
    let len = (sr * 3.0) as usize;
    let inp: Vec<f32> = (0..len)
        .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / sr).sin())
        .collect();

    let mut m = ModulationProcessor::new(ModModel::OpticalBiasTremolo);
    m.set_sample_rate(sr);
    m.set_rate_hz(5.0);
    m.set_depth(10.0);
    m.set_mix_pct(100.0);

    let mut out = vec![0.0f32; len];
    m.process(&inp, &mut out);

    // Envoltória nos últimos 2 ciclos (2.6s..3.0s) em janelas de 2 ms
    let step = (0.002 * sr) as usize;
    let tail = &out[len - (0.4 * sr) as usize..];
    let env: Vec<f32> = tail.chunks(step).map(rms).collect();
    let e_min = env.iter().copied().fold(f32::MAX, f32::min);
    let e_max = env.iter().copied().fold(0.0f32, f32::max);
    let lo = e_min + 0.10 * (e_max - e_min);
    let hi = e_min + 0.90 * (e_max - e_min);

    let mut rise_steps = 0usize;
    let mut fall_steps = 0usize;
    for &e in &env {
        if e > lo && e < hi {
            // Distinguir subida vs descida pela tendência local
        }
    }
    for k in 1..env.len() {
        if env[k] > lo && env[k] < hi {
            if env[k] >= env[k - 1] {
                rise_steps += 1;
            } else {
                fall_steps += 1;
            }
        }
    }
    let ratio = (rise_steps.max(fall_steps) as f32) / (rise_steps.min(fall_steps).max(1) as f32);
    assert!(
        ratio >= 2.0,
        "AUD-30 Óptico: subida ({rise_steps}x4ms) e descida ({fall_steps}x4ms) devem diferir por fator >= 2 (ratio={ratio:.2})"
    );
}

#[test]
fn aud_30_ac30_vibtrem() {
    let sr = 48_000.0f32;
    let len = (sr * 1.5) as usize;
    let amp_in = 10.0f32.powf(-24.0 / 20.0);
    let inp: Vec<f32> = (0..len)
        .map(|i| amp_in * (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / sr).sin())
        .collect();

    // 1. Canal Brilliant com depth máximo -> saída idêntica à de depth 0
    let mut ac_b0 = WhiteBoxAmpProcessor::new(AmpModelId::UkAc30TopBoost);
    ac_b0.set_sample_rate(sr);
    ac_b0.set_input_jack(AmpInputJack::BrilliantHigh);
    ac_b0.panel_mut().set_vibrato_speed(5.0);
    ac_b0.panel_mut().set_vibrato_depth(0.0);
    let mut out_b0 = vec![0.0f32; len];
    ac_b0.process(&inp, &mut out_b0);

    let mut ac_b10 = WhiteBoxAmpProcessor::new(AmpModelId::UkAc30TopBoost);
    ac_b10.set_sample_rate(sr);
    ac_b10.set_input_jack(AmpInputJack::BrilliantHigh);
    ac_b10.panel_mut().set_vibrato_speed(5.0);
    ac_b10.panel_mut().set_vibrato_depth(10.0);
    let mut out_b10 = vec![0.0f32; len];
    ac_b10.process(&inp, &mut out_b10);

    let max_diff_brill = out_b0
        .iter()
        .zip(out_b10.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        max_diff_brill <= 1e-6,
        "AUD-30 AC30: canal Brilliant não deve modular com vibrato_depth=10 (max_diff={max_diff_brill})"
    );

    // 2. Canal Vib-Trem em modo Vibrato: desvio >= 5 cents, envoltória <= 0,5 dB
    let mut ac_vib = WhiteBoxAmpProcessor::new(AmpModelId::UkAc30TopBoost);
    ac_vib.set_sample_rate(sr);
    ac_vib.set_input_jack(AmpInputJack::Vibrato);
    ac_vib.panel_mut().set_ac30_vib_trem_mode(Ac30VibTremMode::Vibrato);
    ac_vib.panel_mut().set_vibrato_speed(5.0);
    ac_vib.panel_mut().set_vibrato_depth(10.0);
    let mut out_vib = vec![0.0f32; len];
    ac_vib.process(&inp, &mut out_vib);

    let freqs_vib = measure_freq_zero_crossings_hz(&out_vib[12_000..], sr, 1024);
    let f_min = freqs_vib.iter().copied().fold(f64::MAX, f64::min);
    let f_max = freqs_vib.iter().copied().fold(0.0f64, f64::max);
    let dev_vib_cents = cents(f_max, f_min) * 0.5;
    let envs_vib: Vec<f32> = out_vib[12_000..].chunks(960).map(rms).collect();
    let env_db_vib = 20.0
        * (envs_vib.iter().copied().fold(0.0f32, f32::max).max(1e-9)
            / envs_vib.iter().copied().fold(f32::MAX, f32::min).max(1e-9))
        .log10();
    assert!(
        dev_vib_cents >= 5.0 && env_db_vib <= 0.5,
        "AUD-30 AC30 Vibrato: esperado desvio >= 5 cents (foi {dev_vib_cents:.2}) e envoltória <= 0.5 dB (foi {env_db_vib:.2} dB)"
    );

    // 3. Canal Vib-Trem em modo Tremolo: AM >= 6 dB, desvio <= 0,5 cent
    let mut ac_trem = WhiteBoxAmpProcessor::new(AmpModelId::UkAc30TopBoost);
    ac_trem.set_sample_rate(sr);
    ac_trem.set_input_jack(AmpInputJack::Vibrato);
    ac_trem.panel_mut().set_ac30_vib_trem_mode(Ac30VibTremMode::Tremolo);
    ac_trem.panel_mut().set_vibrato_speed(5.0);
    ac_trem.panel_mut().set_vibrato_depth(10.0);
    let mut out_trem = vec![0.0f32; len];
    ac_trem.process(&inp, &mut out_trem);

    let envs_trem: Vec<f32> = out_trem[12_000..].chunks(960).map(rms).collect();
    let am_db_trem = 20.0
        * (envs_trem.iter().copied().fold(0.0f32, f32::max).max(1e-9)
            / envs_trem.iter().copied().fold(f32::MAX, f32::min).max(1e-9))
        .log10();
    let freqs_trem = measure_freq_zero_crossings_hz(&out_trem[12_000..], sr, 1024);
    let ft_min = freqs_trem.iter().copied().fold(f64::MAX, f64::min);
    let ft_max = freqs_trem.iter().copied().fold(0.0f64, f64::max);
    let dev_trem_cents = cents(ft_max, ft_min) * 0.5;
    assert!(
        am_db_trem >= 6.0 && dev_trem_cents <= 0.5,
        "AUD-30 AC30 Tremolo: esperado AM >= 6 dB (foi {am_db_trem:.2} dB) e desvio <= 0.5 cent (foi {dev_trem_cents:.2})"
    );
}
