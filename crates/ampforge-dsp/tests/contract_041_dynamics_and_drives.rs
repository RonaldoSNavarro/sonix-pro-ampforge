use ampforge_dsp::fx::dynamics::{
    CompressorProcessor, CompressorTopology, NoiseGateMode, NoiseGateProcessor,
};
use ampforge_dsp::fx::drive::{DriveModel, DriveProcessor};
use std::f32::consts::PI;

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum_sq / samples.len() as f64).sqrt() as f32
}

fn db_from_rms(val: f32) -> f32 {
    20.0 * val.max(1e-7).log10()
}

#[test]
fn test_ca_01_all_buttons_in_monotonic_polarity_and_low_dc() {
    let mut comp = CompressorProcessor::new(CompressorTopology::StudioFet1176);
    comp.set_sample_rate(48000.0);
    comp.set_all_buttons_in(true);
    comp.set_threshold_db(-20.0);
    comp.set_attack_ms(0.1);
    comp.set_release_ms(100.0);

    // 1. Static transfer curve across -1.0..+1.0 (run each point from settled state or ramp)
    let mut prev_out = f32::NEG_INFINITY;
    for step in -100..=100 {
        let x = (step as f32) / 100.0;
        let mut c = CompressorProcessor::new(CompressorTopology::StudioFet1176);
        c.set_sample_rate(48000.0);
        c.set_all_buttons_in(true);
        c.set_threshold_db(-20.0);
        // Feed constant/steady sample to evaluate static transfer after envelope settles
        let inp = vec![x; 256];
        let mut out = vec![0.0f32; 256];
        c.process(&inp, &mut out);
        let y = out[255];
        assert!(
            y >= prev_out - 1e-5,
            "All-Buttons-In transfer curve must be monotonic non-decreasing: x={} gave y={} < prev={}",
            x,
            y,
            prev_out
        );
        if x.abs() > 1e-3 {
            assert_eq!(
                y.signum(),
                x.signum(),
                "All-Buttons-In must preserve polarity: x={} gave y={}",
                x,
                y
            );
        }
        prev_out = y;
    }

    // 2. DC component of a 1 kHz sine wave must be < -60 dBFS
    let n = 4800; // exact integer cycles at 1 kHz / 48 kHz (100 cycles)
    let sine: Vec<f32> = (0..n)
        .map(|i| 0.5 * (2.0 * PI * 1000.0 * (i as f32) / 48000.0).sin())
        .collect();
    let mut out = vec![0.0f32; n];
    comp.process(&sine, &mut out);
    let tail = &out[2400..];
    let dc = (tail.iter().map(|&s| s as f64).sum::<f64>() / tail.len() as f64).abs() as f32;
    let dc_db = db_from_rms(dc);
    assert!(
        dc_db < -60.0,
        "All-Buttons-In 1 kHz sine DC offset must be < -60 dBFS, got {:.2} dBFS",
        dc_db
    );
}

#[test]
fn test_ca_02_compressor_topologies_produce_distinct_envelopes() {
    let topologies = [
        CompressorTopology::PedalRossDyna,
        CompressorTopology::StudioFet1176,
        CompressorTopology::OpticalLa2a,
        CompressorTopology::VcaMasterBus,
    ];

    // Burst test signal: loud transient followed by moderate sustain
    let n = 4800;
    let mut burst = vec![0.0f32; n];
    for i in 0..n {
        let env = if i < 1200 { 0.8 } else { 0.15 };
        burst[i] = env * (2.0 * PI * 440.0 * (i as f32) / 48000.0).sin();
    }

    let mut outputs: Vec<Vec<f32>> = Vec::new();
    for &topo in &topologies {
        let mut c = CompressorProcessor::new(topo);
        c.set_sample_rate(48000.0);
        c.set_threshold_db(-20.0);
        c.set_ratio(4.0);
        c.set_attack_ms(5.0);
        c.set_release_ms(120.0);
        let mut out = vec![0.0f32; n];
        c.process(&burst, &mut out);
        outputs.push(out);
    }

    for i in 0..topologies.len() {
        for j in (i + 1)..topologies.len() {
            let diff: Vec<f32> = outputs[i]
                .iter()
                .zip(outputs[j].iter())
                .map(|(a, b)| a - b)
                .collect();
            let diff_rms = rms(&diff);
            let ref_rms = rms(&outputs[i]).max(1e-6);
            let rel_diff_db = 20.0 * (diff_rms / ref_rms).max(1e-6).log10();
            // Distinct envelopes should have meaningful difference (well above -20 dB relative diff,
            // or RMS level difference / envelope divergence)
            assert!(
                diff_rms > 0.015,
                "Topologies {:?} and {:?} must produce distinct processing (diff_rms={:.5}, rel={:.2} dB)",
                topologies[i],
                topologies[j],
                diff_rms,
                rel_diff_db
            );
        }
    }
}

#[test]
fn test_ca_03_noise_gate_expander_applies_ratio_accurately() {
    let mut gate = NoiseGateProcessor::new(NoiseGateMode::ClassicExpanderGate);
    gate.set_sample_rate(48000.0);
    gate.set_threshold_db(-40.0);
    gate.set_ratio(4.0);
    gate.set_range_db(-90.0);
    gate.set_hysteresis_db(0.0);
    gate.set_attack_ms(1.0);
    gate.set_hold_ms(0.0);
    gate.set_release_ms(20.0);

    let run_level = |g: &mut NoiseGateProcessor, in_db: f32| -> f32 {
        let amp = 10.0f32.powf(in_db / 20.0) * std::f32::consts::SQRT_2;
        let n = 12000;
        let inp: Vec<f32> = (0..n)
            .map(|i| amp * (2.0 * PI * 440.0 * (i as f32) / 48000.0).sin())
            .collect();
        let mut out = vec![0.0f32; n];
        g.process(&inp, None, &mut out);
        db_from_rms(rms(&out[8000..]))
    };

    let out_m50 = run_level(&mut gate, -50.0);
    assert!(
        (out_m50 - (-80.0)).abs() <= 2.0,
        "Expander 1:4 @ -40 dB with -50 dB input must output -80 +/- 2 dB, got {:.2} dB",
        out_m50
    );

    let out_m45 = run_level(&mut gate, -45.0);
    assert!(
        (out_m45 - (-60.0)).abs() <= 2.0,
        "Expander 1:4 @ -40 dB with -45 dB input must output -60 +/- 2 dB, got {:.2} dB",
        out_m45
    );
}

#[test]
fn test_ca_05_drives_oversampled_aliasing_below_minus_50_dbfs() {
    let models = [
        DriveModel::GreenScreamer808,
        DriveModel::GoldenKloneCentaur,
        DriveModel::BlackRodentRat,
        DriveModel::BigMuffPi,
        DriveModel::FuzzFaceSiGe,
        DriveModel::OrangeBossDs,
    ];
    let sr = 48000.0f32;
    let f0 = 5000.0f32;
    // -6 dBFS peak -> 0.501187
    let amp = 10.0f32.powf(-6.0 / 20.0);
    let n = 4800usize; // 500 cycles of 5 kHz, exact integer bins!

    for &model in &models {
        let mut drv = DriveProcessor::new(model);
        drv.set_sample_rate(sr);
        drv.set_gain(10.0);
        drv.set_tone(5.0);
        drv.set_level(7.0);

        let inp: Vec<f32> = (0..n)
            .map(|i| amp * (2.0 * PI * f0 * (i as f32) / sr).sin())
            .collect();
        let mut out = vec![0.0f32; n];
        drv.process(&inp, &mut out);

        // Analyze last 2400 samples (exact integer cycles of 5 kHz at 48 kHz: 2400 * 5000 / 48000 = 250)
        let seg = &out[2400..];
        let m = seg.len();
        // Fundamental is k=250; harmonics in [0..m/2] are 5k (k=250), 10k (k=500), 15k (k=750), 20k (k=1000).
        // Non-harmonic bins (aliasing!) include 1k, 2k, 3k, 4k, 6k, 7k, 8k, 9k, etc.
        let mut max_alias_db = -120.0f32;
        let mut max_alias_bin = 0usize;
        for bin in 1..(m / 2) {
            if bin % 250 == 0 {
                continue; // skip true harmonics (5, 10, 15, 20 kHz)
            }
            let w = 2.0 * std::f64::consts::PI * (bin as f64) / (m as f64);
            let mut re = 0.0f64;
            let mut im = 0.0f64;
            for (idx, &s) in seg.iter().enumerate() {
                let ang = w * (idx as f64);
                re += (s as f64) * ang.cos();
                im -= (s as f64) * ang.sin();
            }
            let mag = (2.0 * (re * re + im * im).sqrt() / (m as f64)) as f32;
            let db = db_from_rms(mag);
            if db > max_alias_db {
                max_alias_db = db;
                max_alias_bin = bin;
            }
        }

        assert!(
            max_alias_db < -50.0,
            "Drive {:?} aliasing spurs at 5 kHz / -6 dBFS must be < -50 dBFS, got {:.2} dBFS at bin {} ({:.1} Hz)",
            model,
            max_alias_db,
            max_alias_bin,
            (max_alias_bin as f32) * sr / (m as f32)
        );
    }
}

#[test]
fn test_ca_06_ts808_clean_path_present_at_drive_0_and_drive_10() {
    let sr = 48000.0f32;
    let n = 4800usize;
    let amp = 10.0f32.powf(-30.0 / 20.0) * std::f32::consts::SQRT_2; // -30 dBFS RMS
    let inp: Vec<f32> = (0..n)
        .map(|i| amp * (2.0 * PI * 200.0 * (i as f32) / sr).sin())
        .collect();

    // 1. Drive = 0, Tone = 5, Level = 10 (or unity output level)
    let mut ts0 = DriveProcessor::new(DriveModel::GreenScreamer808);
    ts0.set_sample_rate(sr);
    ts0.set_gain(0.0);
    ts0.set_tone(5.0);
    ts0.set_level(8.165); // out_level_taper = (8.165/10)^2 * 1.5 ~= 1.0
    let mut out0 = vec![0.0f32; n];
    ts0.process(&inp, &mut out0);

    let gain_db = 20.0 * (rms(&out0[2400..]) / rms(&inp[2400..])).log10();
    assert!(
        (8.0..=13.0).contains(&gain_db),
        "TS808 @ Drive=0, 200 Hz -30 dBFS must match real schematic gain (+10.1 dB in [8, +13] dB), got {:.2} dB",
        gain_db
    );

    // 2. Drive = 10: clean path still contributes -> correlation > 0.3
    let mut ts10 = DriveProcessor::new(DriveModel::GreenScreamer808);
    ts10.set_sample_rate(sr);
    ts10.set_gain(10.0);
    ts10.set_tone(5.0);
    ts10.set_level(8.165);
    let mut out10 = vec![0.0f32; n];
    ts10.process(&inp, &mut out10);

    let x_seg = &inp[2400..];
    let y_seg = &out10[2400..];
    let dot: f64 = x_seg.iter().zip(y_seg).map(|(&a, &b)| (a as f64) * (b as f64)).sum();
    let nx: f64 = x_seg.iter().map(|&a| (a as f64) * (a as f64)).sum::<f64>().sqrt();
    let ny: f64 = y_seg.iter().map(|&b| (b as f64) * (b as f64)).sum::<f64>().sqrt();
    let corr = dot / (nx * ny).max(1e-12);
    assert!(
        corr > 0.3,
        "TS808 @ Drive=10 must preserve clean path correlation > 0.3, got {:.3}",
        corr
    );
}

#[test]
fn test_ca_07_rat_filter_is_first_order_6db_per_octave() {
    let sr = 48000.0f32;
    let mut rat = DriveProcessor::new(DriveModel::BlackRodentRat);
    rat.set_sample_rate(sr);
    rat.set_gain(0.0);
    rat.set_tone(10.0); // ~600 Hz cutoff
    rat.set_level(8.165);

    let measure_rat = |r: &mut DriveProcessor, freq: f32| -> f32 {
        let n = 8192;
        let inp: Vec<f32> = (0..n)
            .map(|i| 0.005 * (2.0 * PI * freq * (i as f32) / sr).sin())
            .collect();
        let mut out = vec![0.0f32; n];
        r.process(&inp, &mut out);
        db_from_rms(rms(&out[4096..]))
    };

    let g_pass = measure_rat(&mut rat, 100.0);
    let g_fc = measure_rat(&mut rat, 600.0);
    let g_4fc = measure_rat(&mut rat, 2400.0); // two octaves above ~600 Hz
    let drop_4fc = g_pass - g_4fc;
    let drop_fc = g_pass - g_fc;
    assert!(
        (drop_4fc - 12.0).abs() <= 1.5,
        "RAT filter 2 octaves above cutoff (2400 Hz vs 100 Hz) must drop 12 +/- 1.5 dB (1st order), got {:.2} dB (fc drop={:.2} dB)",
        drop_4fc,
        drop_fc
    );
}
