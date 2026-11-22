use ampforge_dsp::amp::{AmpChannelMode, AmpInputJack, AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::modshop::ToneStackTopology;
use ampforge_dsp::tonestack::{
    analog_tonestack_magnitude_db, analog_tonestack_magnitude_db_for_components,
    resolved_fmv_components, PassiveToneStack,
};
use std::f64::consts::PI;

fn measure_sine_gain_db(amp: &mut WhiteBoxAmpProcessor, freq_hz: f64, amp_peak: f32, sr: f64) -> f64 {
    amp.set_sample_rate(sr as f32);
    let n = (sr as usize).max(8192);
    let mut input = vec![0.0f32; n];
    for (i, x) in input.iter_mut().enumerate() {
        *x = amp_peak * (2.0 * PI * freq_hz * (i as f64) / sr).sin() as f32;
    }
    let mut output = vec![0.0f32; n];
    amp.process(&input, &mut output);

    let settle = n / 2;
    let meas = &output[settle..];
    let mut re = 0.0f64;
    let mut im = 0.0f64;
    for (idx, &y) in meas.iter().enumerate() {
        let ph = 2.0 * PI * freq_hz * ((settle + idx) as f64) / sr;
        re += (y as f64) * ph.cos();
        im += (y as f64) * ph.sin();
    }
    let out_peak = 2.0 * (re * re + im * im).sqrt() / (meas.len() as f64);
    20.0 * (out_peak / (amp_peak as f64).max(1e-12)).max(1e-12).log10()
}

fn measure_stack_gain_db(
    ts: &mut PassiveToneStack,
    topo: ToneStackTopology,
    b: f32,
    m: f32,
    t: f32,
    freq_hz: f64,
    sr: f64,
) -> f64 {
    ts.reset();
    ts.update(topo, b, m, t, sr);
    let n = 48_000usize;
    let settle = 24_000usize;
    let mut re = 0.0f64;
    let mut im = 0.0f64;
    for i in 0..n {
        let ph = 2.0 * PI * freq_hz * (i as f64) / sr;
        let y = ts.process(ph.sin());
        if i >= settle {
            re += y * ph.cos();
            im += y * ph.sin();
        }
    }
    let mag = 2.0 * (re * re + im * im).sqrt() / ((n - settle) as f64);
    20.0 * mag.max(1e-12).log10()
}

#[test]
fn aud_23a_twin_sem_scoop_extra() {
    let sr = 48_000.0;
    let amp_peak = 0.003162f32; // -50 dBFS (regime linear)

    let gain_amp = |freq: f64| {
        let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
        amp.panel_mut().set_input_gain(4.0);
        amp.panel_mut().set_bass(5.0);
        amp.panel_mut().set_middle(5.0);
        amp.panel_mut().set_treble(5.0);
        amp.panel_mut().set_master_volume(5.0);
        measure_sine_gain_db(&mut amp, freq, amp_peak, sr)
    };

    let amp_200 = gain_amp(200.0);
    let amp_450 = gain_amp(450.0);
    let amp_1000 = gain_amp(1000.0);
    let amp_notch = amp_450 - 0.5 * (amp_200 + amp_1000);

    let mut ts = PassiveToneStack::new();
    let ts_200 = measure_stack_gain_db(&mut ts, ToneStackTopology::FenderTmbMidScoop, 5.0, 5.0, 5.0, 200.0, 96_000.0);
    let ts_450 = measure_stack_gain_db(&mut ts, ToneStackTopology::FenderTmbMidScoop, 5.0, 5.0, 5.0, 450.0, 96_000.0);
    let ts_1000 = measure_stack_gain_db(&mut ts, ToneStackTopology::FenderTmbMidScoop, 5.0, 5.0, 5.0, 1000.0, 96_000.0);
    let ts_notch = ts_450 - 0.5 * (ts_200 + ts_1000);

    let diff = (amp_notch - ts_notch).abs();
    assert!(
        diff <= 0.5,
        "AUD-23a Twin scoop duplo: amp_450_rel={:.3} dB vs stack_isolado={:.3} dB (diff={:.3} dB > 0.5)",
        amp_notch, ts_notch, diff
    );
}

#[derive(Clone, Copy)]
struct Cpx(f64, f64);
impl Cpx {
    fn add(self, o: Self) -> Self { Self(self.0 + o.0, self.1 + o.1) }
    fn sub(self, o: Self) -> Self { Self(self.0 - o.0, self.1 - o.1) }
    fn mul(self, o: Self) -> Self { Self(self.0 * o.0 - self.1 * o.1, self.0 * o.1 + self.1 * o.0) }
    fn div(self, o: Self) -> Self {
        let d = o.0 * o.0 + o.1 * o.1;
        Self((self.0 * o.0 + self.1 * o.1) / d, (self.1 * o.0 - self.0 * o.1) / d)
    }
    fn mag(self) -> f64 { self.0.hypot(self.1) }
}

/// Análise nodal independente KCL 5x5 para a rede Vox AC30/6 Top Boost (OS/010):
/// Nós: 0=V1(após C1), 1=Vout, 2=Vbm(após RT2 e C2), 3=Vw(cursor Bass + C3 + R2//RB2), 4=Vs(após R1)
fn independent_vox_tb_mna_db(
    r1: f64, r2: f64, rt: f64, rb: f64, rl: f64,
    c1: f64, c2: f64, c3: f64,
    t_pos: f64, b_pos: f64,
    freq_hz: f64,
) -> f64 {
    let w = 2.0 * PI * freq_hz;
    let z = Cpx(0.0, 0.0);
    let mut m = [[z; 5]; 5];
    let mut rhs = [z; 5];

    let rt1 = ((1.0 - t_pos) * rt).max(1e-2);
    let rt2 = (t_pos * rt).max(1e-2);
    let rb1 = (b_pos * rb).max(1e-2);
    let rb2 = ((1.0 - b_pos) * rb).max(1e-2);

    let stamp_y = |mat: &mut [[Cpx; 5]; 5], i: Option<usize>, j: Option<usize>, y: Cpx| {
        if let Some(ii) = i { mat[ii][ii] = mat[ii][ii].add(y); }
        if let Some(jj) = j { mat[jj][jj] = mat[jj][jj].add(y); }
        if let (Some(ii), Some(jj)) = (i, j) {
            mat[ii][jj] = mat[ii][jj].sub(y);
            mat[jj][ii] = mat[jj][ii].sub(y);
        }
    };

    // Vin = 1.0 conectado a V1 via C1 e a Vs via R1
    m[0][0] = m[0][0].add(Cpx(0.0, w * c1));
    rhs[0] = Cpx(0.0, w * c1);
    m[4][4] = m[4][4].add(Cpx(1.0 / r1, 0.0));
    rhs[4] = Cpx(1.0 / r1, 0.0);

    stamp_y(&mut m, Some(0), Some(1), Cpx(1.0 / rt1, 0.0));
    if rl.is_finite() && rl > 0.0 {
        stamp_y(&mut m, Some(1), None, Cpx(1.0 / rl, 0.0));
    }
    stamp_y(&mut m, Some(1), Some(2), Cpx(1.0 / rt2, 0.0));
    stamp_y(&mut m, Some(4), Some(2), Cpx(0.0, w * c2));
    stamp_y(&mut m, Some(4), Some(3), Cpx(0.0, w * c3));
    stamp_y(&mut m, Some(2), Some(3), Cpx(1.0 / rb1, 0.0));
    stamp_y(&mut m, Some(3), None, Cpx(1.0 / rb2 + 1.0 / r2, 0.0));

    for p in 0..5 {
        let piv = m[p][p];
        for j in p..5 { m[p][j] = m[p][j].div(piv); }
        rhs[p] = rhs[p].div(piv);
        for i in 0..5 {
            if i != p {
                let fac = m[i][p];
                for j in p..5 { m[i][j] = m[i][j].sub(fac.mul(m[p][j])); }
                rhs[i] = rhs[i].sub(fac.mul(rhs[p]));
            }
        }
    }
    20.0 * rhs[1].mag().max(1e-12).log10()
}

#[test]
fn aud_23b_ac30_stack() {
    let base = resolved_fmv_components(ToneStackTopology::VoxTopBoost, 5.0, 5.0, 5.0);
    // OS/010: R1=100k slope, R2=10k tail, RT=1M, RB=1M, C1=50pF, C2=22nF, C3=22nF
    assert!(
        (base.r_treble - 1_000_000.0).abs() < 1.0,
        "AUD-23b OS/010 exige R_treble = 1M, obteve {}",
        base.r_treble
    );

    let mut freqs = Vec::new();
    for k in 0..=32 {
        let f = 20.0 * 2.0f64.powf((k as f64) / 3.0);
        if f <= 20_000.5 { freqs.push(f); }
    }

    let knobs = [0.0f32, 2.5, 5.0, 7.5, 10.0];
    let mut max_err = 0.0f64;
    for &b in &knobs {
        for &t in &knobs {
            let comps = resolved_fmv_components(ToneStackTopology::VoxTopBoost, b, 0.0, t);
            for &f in &freqs {
                let m0 = analog_tonestack_magnitude_db(ToneStackTopology::VoxTopBoost, b, 0.0, t, f);
                let m10 = analog_tonestack_magnitude_db(ToneStackTopology::VoxTopBoost, b, 10.0, t, f);
                assert!(
                    (m0 - m10).abs() < 1e-9,
                    "AUD-23b AC30 não possui Middle, mas m=0 vs m=10 diferiu em {} dB",
                    (m0 - m10).abs()
                );

                let ref_db = independent_vox_tb_mna_db(
                    comps.r_slope,
                    comps.r_mid,
                    comps.r_treble,
                    comps.r_bass,
                    comps.r_load,
                    comps.c1,
                    comps.c2,
                    comps.c3,
                    comps.t_pos,
                    comps.b_pos,
                    f,
                );
                let err = (m0 - ref_db).abs();
                max_err = max_err.max(err);
                assert!(
                    err <= 0.05,
                    "AUD-23b Vox Top Boost (B={}, T={}) @ {:.1} Hz: prod={:.4} vs ref={:.4} (erro={:.4} dB > 0.05)",
                    b, t, f, m0, ref_db, err
                );
            }
        }
    }
    // Verifica que filtro digital a 48 kHz acompanha a mesma curva Top Boost (<= 0.10 dB até 5 kHz)
    let mut ts = PassiveToneStack::new();
    let comps_mid = resolved_fmv_components(ToneStackTopology::VoxTopBoost, 7.5, 0.0, 2.5);
    for &f in &[50.0f64, 250.0, 1000.0, 4000.0] {
        let dig = measure_stack_gain_db(&mut ts, ToneStackTopology::VoxTopBoost, 7.5, 0.0, 2.5, f, 48_000.0);
        let ana = independent_vox_tb_mna_db(
            comps_mid.r_slope, comps_mid.r_mid, comps_mid.r_treble, comps_mid.r_bass, comps_mid.r_load,
            comps_mid.c1, comps_mid.c2, comps_mid.c3, comps_mid.t_pos, comps_mid.b_pos, f,
        );
        assert!(
            (dig - ana).abs() <= 0.1,
            "AUD-23b Vox digital 48 kHz @ {:.1} Hz: dig={:.4} vs ref={:.4}",
            f, dig, ana
        );
    }
    // Verifica que AC30 difere estruturalmente da matriz FMV (para B=10, Middle=5 FMV vs Top Boost > 1 dB)
    let fmv_db = analog_tonestack_magnitude_db_for_components(base, 1000.0);
    let tb_db = analog_tonestack_magnitude_db(ToneStackTopology::VoxTopBoost, 10.0, 5.0, 5.0, 1000.0);
    assert!(
        (fmv_db - tb_db).abs() > 0.5,
        "AUD-23b Top Boost deve divergir do FMV (fmv={:.2}, tb={:.2})",
        fmv_db, tb_db
    );
    assert!(max_err <= 0.05);
}

#[test]
fn aud_23c_recto_stack_pos_ganho() {
    let topo = ToneStackTopology::RectoPostGain;
    assert_eq!(topo.as_str(), "recto");

    let sr = 48_000.0;
    let measure_tilt = |amp_peak: f32, bass: f32, treble: f32| -> [f64; 3] {
        let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::CaliDualRectoRevF);
        amp.set_channel_mode(AmpChannelMode::Modern);
        amp.panel_mut().set_input_gain(8.5);
        amp.panel_mut().set_master_volume(1.0);
        amp.panel_mut().set_bass(bass);
        amp.panel_mut().set_middle(5.0);
        amp.panel_mut().set_treble(treble);
        [
            measure_sine_gain_db(&mut amp, 150.0, amp_peak, sr),
            measure_sine_gain_db(&mut amp, 1000.0, amp_peak, sr),
            measure_sine_gain_db(&mut amp, 4000.0, amp_peak, sr),
        ]
    };

    // Comprova que em sinal grande (0.25 pico, 1 kHz, master=1.0) o pré-amp está realmente saturando
    let mut amp_sat = WhiteBoxAmpProcessor::new(AmpModelId::CaliDualRectoRevF);
    amp_sat.set_sample_rate(sr as f32);
    amp_sat.set_channel_mode(AmpChannelMode::Modern);
    amp_sat.panel_mut().set_input_gain(8.5);
    amp_sat.panel_mut().set_master_volume(1.0);
    let n = sr as usize;
    let in_sat: Vec<f32> = (0..n)
        .map(|i| 0.25 * (2.0 * PI * 1000.0 * (i as f64) / sr).sin() as f32)
        .collect();
    let mut out_sat = vec![0.0f32; n];
    amp_sat.process(&in_sat, &mut out_sat);
    let thd = amp_sat.compute_thd(&out_sat[24000..]);
    assert!(
        thd > 0.15,
        "AUD-23c pré-amp deve saturar em sinal grande antes do tone stack (THD={:.3} <= 0.15)",
        thd
    );

    let small_bright = measure_tilt(0.001, 2.0, 8.0);
    let small_dark = measure_tilt(0.001, 8.0, 2.0);
    let large_bright = measure_tilt(0.25, 2.0, 8.0);
    let large_dark = measure_tilt(0.25, 8.0, 2.0);

    for idx in 0..3 {
        let delta_small = small_bright[idx] - small_dark[idx];
        let delta_large = large_bright[idx] - large_dark[idx];
        assert!(
            (delta_small - delta_large).abs() <= 1.0,
            "AUD-23c Recto stack pós-ganho banda {}: delta_small={:.2} dB vs delta_large={:.2} dB",
            idx, delta_small, delta_large
        );
    }
}

#[test]
fn aud_23d_presence_age_em_modern() {
    let sr = 48_000.0;
    let run_modern = |presence: f32, freq: f64| {
        let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::CaliDualRectoRevF);
        amp.set_channel_mode(AmpChannelMode::Modern);
        amp.panel_mut().set_input_gain(4.0);
        amp.panel_mut().set_master_volume(3.0);
        amp.panel_mut().set_presence(presence);
        measure_sine_gain_db(&mut amp, freq, 0.003162, sr)
    };

    let g0 = run_modern(0.0, 4000.0);
    let g10 = run_modern(10.0, 4000.0);
    let boost = g10 - g0;
    assert!(
        boost >= 6.0,
        "AUD-23d Presence em Modern (0->10 @ 4 kHz) deveria subir >= 6 dB, subiu {:.2} dB",
        boost
    );
}

#[test]
fn aud_24_low_jcm800_pula_estagio1() {
    let sr = 48_000.0;
    let measure = |jack: AmpInputJack| -> (f64, f64) {
        let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
        amp.panel_mut().set_input_gain(5.0);
        amp.panel_mut().set_master_volume(5.0);
        amp.set_input_jack(jack);
        let stage1 = amp.ganho_estagio1_pequeno_sinal();
        let gain_db = measure_sine_gain_db(&mut amp, 1000.0, 1e-4, sr);
        (stage1, gain_db)
    };

    let (stage1, g_high) = measure(AmpInputJack::High);
    let (_, g_low) = measure(AmpInputJack::Low);
    assert!(
        stage1 > 20.0,
        "AUD-24 ganho_estagio1_pequeno_sinal do JCM800 deve ser > 20 (foi {:.2})",
        stage1
    );
    let expected_diff_db = 20.0 * stage1.log10() + 6.0206;
    let actual_diff_db = g_high - g_low;
    assert!(
        (actual_diff_db - expected_diff_db).abs() <= 1.0,
        "AUD-24 JCM800 HIGH-LOW @ 1 kHz -80 dBFS: obtido={:.2} dB vs esperado={:.2} dB (stage1={:.2})",
        actual_diff_db, expected_diff_db, stage1
    );

    // Em sinal alto, LOW pula V1a e produz THD muito menor que HIGH
    let measure_thd = |jack: AmpInputJack| -> f32 {
        let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
        amp.set_sample_rate(sr as f32);
        amp.panel_mut().set_input_gain(8.0);
        amp.panel_mut().set_master_volume(5.0);
        amp.set_input_jack(jack);
        let n = sr as usize;
        let input: Vec<f32> = (0..n)
            .map(|i| 0.25 * (2.0 * PI * 1000.0 * (i as f64) / sr).sin() as f32)
            .collect();
        let mut out = vec![0.0f32; n];
        amp.process(&input, &mut out);
        amp.compute_thd(&out[24000..])
    };
    let thd_high = measure_thd(AmpInputJack::High);
    let thd_low = measure_thd(AmpInputJack::Low);
    assert!(
        thd_low < 0.5 * thd_high,
        "AUD-24 JCM800 LOW em sinal alto deve ter THD < 0.5*HIGH (low={:.4}, high={:.4})",
        thd_low, thd_high
    );
}

#[test]
fn aud_24_low_plexi_6db() {
    let sr = 48_000.0;
    let measure = |jack: AmpInputJack| {
        let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritPlexi1959);
        amp.panel_mut().set_input_gain(3.0);
        amp.panel_mut().set_volume_2(5.0);
        amp.set_input_jack(jack);
        measure_sine_gain_db(&mut amp, 1000.0, 1e-4, sr)
    };

    let diff_ch1 = measure(AmpInputJack::High1) - measure(AmpInputJack::Low1);
    let diff_ch2 = measure(AmpInputJack::High2) - measure(AmpInputJack::Low2);
    assert!(
        (diff_ch1 - 6.0).abs() <= 1.0,
        "AUD-24 Plexi High1 - Low1 deve ser 6.0 +- 1.0 dB, foi {:.2} dB",
        diff_ch1
    );
    assert!(
        (diff_ch2 - 6.0).abs() <= 1.0,
        "AUD-24 Plexi High2 - Low2 deve ser 6.0 +- 1.0 dB, foi {:.2} dB",
        diff_ch2
    );
}
