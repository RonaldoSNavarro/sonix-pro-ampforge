use ampforge_dsp::fx::delay::{DelayModel, DelayProcessor};
use ampforge_dsp::fx::modulation::{ModModel, ModulationProcessor};
use ampforge_dsp::fx::reverb::{ReverbModel, ReverbProcessor};
use std::f32::consts::PI;

fn goertzel_mag(samples: &[f32], freq_hz: f32, sample_rate: f32) -> f32 {
    let n = samples.len();
    let omega = 2.0 * PI * freq_hz / sample_rate;
    let coeff = 2.0 * omega.cos();
    let mut s_prev = 0.0f64;
    let mut s_prev2 = 0.0f64;
    let coeff64 = coeff as f64;
    for &x in samples {
        let s = (x as f64) + coeff64 * s_prev - s_prev2;
        s_prev2 = s_prev;
        s_prev = s;
    }
    let power = s_prev2 * s_prev2 + s_prev * s_prev - coeff64 * s_prev * s_prev2;
    (power.max(0.0).sqrt() / (n as f64 * 0.5)) as f32
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples.iter().map(|&x| (x as f64) * (x as f64)).sum();
    (sum / samples.len() as f64).sqrt() as f32
}

#[test]
fn test_ca_02_optical_bias_tremolo_pure_am_no_delay() {
    let sr = 48000.0;
    let mut m = ModulationProcessor::new(ModModel::OpticalBiasTremolo);
    m.set_sample_rate(sr);
    m.set_rate_hz(4.0);
    m.set_depth(10.0);
    m.set_mix_pct(100.0);

    let n = 48000; // 1.0 s
    let input: Vec<f32> = (0..n)
        .map(|i| (2.0 * PI * 1000.0 * i as f32 / sr).sin())
        .collect();
    let mut out = vec![0.0f32; n];
    m.process(&input, &mut out);

    // Pure AM of 1000 Hz at 4 Hz only has energy at 996, 1000, 1004 Hz.
    // Sidebands outside 1000 +/- 10 Hz (e.g. 950, 980, 1020, 1050, 2000 Hz) must be < -40 dB vs 1000 Hz peak.
    let window = &out[12000..36000];
    let peak_1k = goertzel_mag(window, 1000.0, sr).max(1e-9);
    for &probe in &[950.0, 980.0, 1020.0, 1050.0, 2000.0] {
        let sb = goertzel_mag(window, probe, sr);
        let db = 20.0 * (sb.max(1e-12) / peak_1k).log10();
        assert!(
            db < -40.0,
            "CA-2: OpticalBiasTremolo sideband at {probe} Hz was {db:.1} dB (expected < -40 dB; pitch/delay modulation leaked!)"
        );
    }

    // Envelope must actually modulate at ~4 Hz (high peak-to-trough ratio)
    let mut min_env = f32::MAX;
    let mut max_env = 0.0f32;
    for chunk in window.chunks(600) {
        let r = rms(chunk);
        min_env = min_env.min(r);
        max_env = max_env.max(r);
    }
    assert!(
        max_env > min_env * 2.5,
        "CA-2: OpticalBiasTremolo envelope barely modulated: max={max_env:.4}, min={min_env:.4}"
    );
}

#[test]
fn test_ca_03_through_zero_flanger_comb_peaks_and_feedback() {
    let sr = 48000.0;
    let measure_comb_contrast = |fb: f32| -> (f32, f32) {
        let mut m = ModulationProcessor::new(ModModel::ThroughZeroFlanger);
        m.set_sample_rate(sr);
        m.set_rate_hz(0.1);
        m.set_depth(6.0);
        m.set_mix_pct(50.0);
        m.set_feedback(fb);
        m.set_lfo_frozen(true);

        // Feed unit impulse and measure 4096-point frequency response
        let mut imp = vec![0.0f32; 4096];
        imp[0] = 1.0;
        let mut out = vec![0.0f32; 4096];
        m.process(&imp, &mut out);

        let mut min_db = f32::INFINITY;
        let mut max_db = f32::NEG_INFINITY;
        for k in 10..400 {
            let freq = k as f32 * 25.0; // 250 Hz .. 10 kHz
            let mag = goertzel_mag(&out, freq, sr).max(1e-7);
            let db = 20.0 * mag.log10();
            min_db = min_db.min(db);
            max_db = max_db.max(db);
        }
        (max_db, max_db - min_db)
    };

    let (peak_70, contrast_70) = measure_comb_contrast(0.70);
    let (peak_0, _) = measure_comb_contrast(0.0);

    assert!(
        contrast_70 >= 10.0,
        "CA-3: ThroughZeroFlanger at 70% feedback needs >= 10 dB peak-to-valley comb contrast, got {contrast_70:.2} dB"
    );
    assert!(
        peak_70 - peak_0 >= 6.0,
        "CA-3: ThroughZeroFlanger peak must drop >= 6 dB when feedback goes 70% -> 0% (70%={peak_70:.2} dB, 0%={peak_0:.2} dB)"
    );
}

#[test]
fn test_ca_04_orange90_phaser_notches_and_photocell_univibe() {
    let sr = 48000.0;
    let mut phaser = ModulationProcessor::new(ModModel::Orange90Phaser);
    phaser.set_sample_rate(sr);
    phaser.set_rate_hz(0.5);
    phaser.set_depth(7.0);
    phaser.set_mix_pct(50.0);
    phaser.set_feedback(0.0);
    phaser.set_lfo_frozen(true);

    let mut imp = vec![0.0f32; 4096];
    imp[0] = 1.0;
    let mut out = vec![0.0f32; 4096];
    phaser.process(&imp, &mut out);

    // Log-spaced frequency scan from 40 Hz to 18 kHz for deep notches (< -10 dB relative to passband)
    let mut mags_db = Vec::new();
    for i in 0..240 {
        let f = 40.0 * (18000.0f32 / 40.0).powf(i as f32 / 239.0);
        let mag = goertzel_mag(&out, f, sr).max(1e-7);
        mags_db.push((f, 20.0 * mag.log10()));
    }
    let max_db = mags_db.iter().map(|x| x.1).fold(f32::NEG_INFINITY, f32::max);
    let mut notch_count = 0;
    for w in mags_db.windows(3) {
        if w[1].1 < w[0].1 && w[1].1 < w[2].1 && (max_db - w[1].1) > 10.0 {
            notch_count += 1;
        }
    }
    assert_eq!(
        notch_count, 2,
        "CA-4: 4-stage Orange90Phaser must produce exactly N/2 = 2 notches, found {notch_count}"
    );

    // PhotocellUnivibe vs AnalogBbdChorus vs StereoTriChorus vs RotarySpeaker must differ measurably
    let test_sig: Vec<f32> = (0..12000)
        .map(|i| (2.0 * PI * 330.0 * i as f32 / sr).sin())
        .collect();
    let run_model = |model: ModModel| -> Vec<f32> {
        let mut p = ModulationProcessor::new(model);
        p.set_sample_rate(sr);
        p.set_rate_hz(2.0);
        p.set_depth(8.0);
        p.set_mix_pct(70.0);
        let mut o = vec![0.0f32; test_sig.len()];
        p.process(&test_sig, &mut o);
        o
    };
    let univibe = run_model(ModModel::PhotocellUnivibe);
    let bbd = run_model(ModModel::AnalogBbdChorus);
    let tri = run_model(ModModel::StereoTriChorus);
    let rotary = run_model(ModModel::RotarySpeaker);

    let diff_rms = |a: &[f32], b: &[f32]| -> f32 {
        let d: Vec<f32> = a.iter().zip(b).map(|(x, y)| x - y).collect();
        rms(&d)
    };
    assert!(diff_rms(&univibe, &bbd) > 0.05, "CA-4: Univibe and BBD chorus must differ");
    assert!(diff_rms(&bbd, &tri) > 0.03, "CA-4: BBD chorus and Tri-Chorus must differ");
    assert!(diff_rms(&rotary, &bbd) > 0.05, "CA-4: RotarySpeaker and BBD chorus must differ");
}

#[test]
fn test_ca_05_analog_bbd_vs_pristine_digital_delay_5khz_rolloff() {
    let sr = 48000.0;
    let measure_5khz_echo_ratio = |model: DelayModel| -> f32 {
        let mut d = DelayProcessor::new(model);
        d.set_sample_rate(sr);
        d.set_delay_time_ms(50.0); // 2400 samples
        d.set_feedback_pct(50.0);
        d.set_mix_pct(100.0); // pure wet to measure echo cleanly

        let total = 9600;
        let mut input = vec![0.0f32; total];
        for i in 0..480 {
            input[i] = 0.5 * (2.0 * PI * 5000.0 * i as f32 / sr).sin();
        }
        let mut out = vec![0.0f32; total];
        d.process(&input, &mut out);

        // First echo is at [2400..2880], second echo (after feedback) at [4800..5280]
        let echo1_rms = rms(&out[2400..2880]);
        let echo2_rms = rms(&out[4800..5280]);
        20.0 * (echo2_rms.max(1e-9) / echo1_rms.max(1e-9)).log10()
    };

    // With feedback = 50% (-6.02 dB):
    // PristineDigitalDelay should repeat at ~ -6.02 dB (+/- 0.3 dB)
    // AnalogBbdDelay should attenuate 5 kHz by >= 3 dB extra (so <= -9.02 dB)
    let digital_db = measure_5khz_echo_ratio(DelayModel::PristineDigitalDelay);
    let bbd_db = measure_5khz_echo_ratio(DelayModel::AnalogBbdDelay);

    assert!(
        (digital_db - (-6.02)).abs() <= 0.4,
        "CA-5: PristineDigitalDelay 5kHz feedback ratio was {digital_db:.2} dB (expected -6.02 +/- 0.4 dB)"
    );
    assert!(
        bbd_db <= digital_db - 3.0,
        "CA-5: AnalogBbdDelay 5kHz feedback must attenuate >= 3 dB more than digital (bbd={bbd_db:.2} dB, digital={digital_db:.2} dB)"
    );
}

#[test]
fn test_ca_06_reverse_delay_and_dual_parallel_delay() {
    let sr = 48000.0;
    // ReverseDelay: feed a rising ramp (0 -> 1) over 100 ms, wet echo should have falling slope
    let mut rev = DelayProcessor::new(DelayModel::ReverseDelay);
    rev.set_sample_rate(sr);
    rev.set_delay_time_ms(100.0); // 4800 samples
    rev.set_feedback_pct(0.0);
    rev.set_mix_pct(100.0);

    let grain = 4800;
    let mut ramp = vec![0.0f32; grain * 3];
    for i in 0..grain {
        ramp[i] = i as f32 / grain as f32;
    }
    let mut rev_out = vec![0.0f32; grain * 3];
    rev.process(&ramp, &mut rev_out);

    // In the reversed grain output [grain .. 2*grain], first quarter should be louder than last quarter
    let first_q = rms(&rev_out[grain + 200..grain + 1200]);
    let last_q = rms(&rev_out[grain + 3600..grain + 4600]);
    assert!(
        first_q > last_q * 2.0,
        "CA-6: ReverseDelay did not reverse rising ramp: first_q={first_q:.4}, last_q={last_q:.4}"
    );

    // DualParallelDelay: impulse -> two distinct taps (at 0.75*T and 1.0*T)
    let mut dual = DelayProcessor::new(DelayModel::DualParallelDelay);
    dual.set_sample_rate(sr);
    dual.set_delay_time_ms(100.0); // T = 4800 samples, 0.75*T = 3600 samples
    dual.set_feedback_pct(0.0);
    dual.set_mix_pct(100.0);

    let mut imp = vec![0.0f32; 6000];
    imp[0] = 1.0;
    let mut dual_out = vec![0.0f32; 6000];
    dual.process(&imp, &mut dual_out);

    let tap1 = dual_out[3550..3650].iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    let tap2 = dual_out[4750..4850].iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    assert!(
        tap1 > 0.2 && tap2 > 0.2,
        "CA-6: DualParallelDelay must emit two distinct taps at 0.75*T and T (tap1={tap1:.4}, tap2={tap2:.4})"
    );
}

#[test]
fn test_ca_07_delay_time_glide_no_discontinuity() {
    let sr = 48000.0;
    let mut d = DelayProcessor::new(DelayModel::MultiHeadTapeEcho);
    d.set_sample_rate(sr);
    d.set_delay_time_ms(300.0);
    d.set_feedback_pct(30.0);
    d.set_mix_pct(50.0);

    let block = 24000;
    let total = block * 2;
    let input: Vec<f32> = (0..total)
        .map(|i| (2.0 * PI * 440.0 * i as f32 / sr).sin())
        .collect();

    let mut out1 = vec![0.0f32; block];
    d.process(&input[..block], &mut out1);
    let max_deriv_before = out1[12000..]
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0f32, f32::max);

    d.set_delay_time_ms(301.136);
    let mut out2 = vec![0.0f32; block];
    d.process(&input[block..], &mut out2);
    let boundary_jump = (out2[0] - out1[block - 1]).abs();
    let max_deriv_after = out2
        .windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(boundary_jump, f32::max);

    assert!(
        max_deriv_after <= max_deriv_before * 1.5,
        "CA-7: Delay time change caused discontinuity: before={max_deriv_before:.5}, after={max_deriv_after:.5}"
    );
}

#[test]
fn test_ca_08_delay_ducking_smooth_gain_envelope() {
    let sr = 48000.0;
    let mut d = DelayProcessor::new(DelayModel::PristineDigitalDelay);
    d.set_sample_rate(sr);
    d.set_delay_time_ms(40.0);
    d.set_feedback_pct(40.0);
    d.set_mix_pct(100.0);
    d.set_ducking_enabled(true);
    d.set_ducking_threshold_db(-20.0);
    d.set_ducking_attenuation_db(-18.0);

    // Prime delay buffer with 0.08 (below -20 dB = 0.1), then hit with 0.8 above threshold
    let n = 12000;
    let mut input = vec![0.0f32; n];
    for i in 0..4800 {
        input[i] = 0.08 * (2.0 * PI * 440.0 * i as f32 / sr).sin();
    }
    for i in 4800..9600 {
        input[i] = 0.8 * (2.0 * PI * 440.0 * i as f32 / sr).sin();
    }

    let mut out = vec![0.0f32; n];
    d.process(&input, &mut out);

    // Smooth gain envelope must not jump more than 0.5 dB per sample on the delayed 440Hz carrier
    let mut max_step = 0.0f32;
    for w in out[4800..6000].windows(2) {
        max_step = max_step.max((w[1] - w[0]).abs());
    }
    assert!(
        max_step < 0.008,
        "CA-8: Ducking envelope stepped abruptly (max sample delta = {max_step:.5})"
    );
}

#[test]
fn test_ca_09_reverb_rt60_accuracy_across_sample_rates() {
    for &sr in &[44100.0f32, 48000.0, 96000.0] {
        let target_rt60 = 1.5f32;
        let mut r = ReverbProcessor::new(ReverbModel::StudioVintagePlate);
        r.set_sample_rate(sr);
        r.set_decay_time_s(target_rt60);
        r.set_predelay_ms(0.0);
        r.set_damping(0.0); // minimal HF damping for clean RT60 check
        r.set_mix_pct(100.0);

        let len = (sr * 2.0) as usize;
        let mut imp = vec![0.0f32; len];
        imp[0] = 1.0;
        let mut out = vec![0.0f32; len];
        r.process(&imp, &mut out);

        // Schroeder backward integration (EDC)
        let mut edc = vec![0.0f64; len];
        let mut acc = 0.0f64;
        for i in (0..len).rev() {
            let s = out[i] as f64;
            acc += s * s;
            edc[i] = acc;
        }
        let total_e = edc[0].max(1e-18);
        let t_minus5 = edc.iter().position(|&e| 10.0 * (e / total_e).log10() <= -5.0).unwrap_or(0);
        let t_minus25 = edc.iter().position(|&e| 10.0 * (e / total_e).log10() <= -25.0).unwrap_or(len - 1);
        assert!(t_minus25 < len - 1, "CA-9: EDC did not reach -25 dB within buffer at {sr} Hz");
        let measured_rt60 = ((t_minus25 - t_minus5) as f32 / sr) * 3.0;
        let rel_err = (measured_rt60 - target_rt60).abs() / target_rt60;
        assert!(
            rel_err <= 0.15,
            "CA-9: At {sr} Hz, measured RT60={measured_rt60:.3}s vs target={target_rt60:.3}s (err={:.1}%)",
            rel_err * 100.0
        );
    }
}

#[test]
fn test_ca_10_reverb_plate_echo_density_and_spring_tension_dispersion() {
    let sr = 48000.0;
    let mut plate = ReverbProcessor::new(ReverbModel::StudioVintagePlate);
    plate.set_sample_rate(sr);
    plate.set_decay_time_s(2.0);
    plate.set_predelay_ms(0.0);
    plate.set_mix_pct(100.0);

    let mut imp = vec![0.0f32; 9600];
    imp[0] = 1.0;
    let mut out = vec![0.0f32; 9600];
    plate.process(&imp, &mut out);

    // Count significant local peaks in [4800..6000] (100..125 ms)
    let seg = &out[4800..6000];
    let peak_max = seg.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    let thresh = peak_max * 0.05;
    let peaks = seg
        .windows(3)
        .filter(|w| w[1].abs() > w[0].abs() && w[1].abs() > w[2].abs() && w[1].abs() > thresh)
        .count();
    assert!(
        peaks >= 40,
        "CA-10: StudioVintagePlate echo density @ 100ms too low ({peaks} peaks in 25ms, expected >= 40 from diffusers+FDN)"
    );

    // TubeSpringReverb: spring_tension 0 vs 10 must alter tail energy centroid / group delay >= 20%
    let measure_centroid_ms = |tension: f32| -> f32 {
        let mut sp = ReverbProcessor::new(ReverbModel::TubeSpringReverb);
        sp.set_sample_rate(sr);
        sp.set_decay_time_s(1.5);
        sp.set_predelay_ms(0.0);
        sp.set_spring_tension(tension);
        sp.set_mix_pct(100.0);
        let mut o = vec![0.0f32; 48000];
        let mut i_buf = vec![0.0f32; 48000];
        i_buf[0] = 1.0;
        sp.process(&i_buf, &mut o);
        let mut num = 0.0f64;
        let mut den = 0.0f64;
        for (idx, &v) in o.iter().enumerate() {
            let e = (v as f64) * (v as f64);
            num += (idx as f64) * e;
            den += e;
        }
        ((num / den.max(1e-18)) / sr as f64 * 1000.0) as f32
    };
    let c0 = measure_centroid_ms(0.0);
    let c10 = measure_centroid_ms(10.0);
    let rel_diff = (c10 - c0).abs() / c0.max(1e-6);
    assert!(
        rel_diff >= 0.20,
        "CA-10: TubeSpringReverb spring_tension 0->10 changed group delay centroid by only {:.1}% (c0={c0:.2}ms, c10={c10:.2}ms)",
        rel_diff * 100.0
    );
}

#[test]
fn test_ca_11_shimmer_cloud_octave_up_energy() {
    let sr = 48000.0;
    let measure_octave_ratio = |model: ReverbModel| -> f32 {
        let mut r = ReverbProcessor::new(model);
        r.set_sample_rate(sr);
        r.set_decay_time_s(2.5);
        r.set_predelay_ms(0.0);
        r.set_mix_pct(100.0);

        let n = 48000;
        let mut input = vec![0.0f32; n];
        for i in 0..12000 {
            input[i] = (2.0 * PI * 440.0 * i as f32 / sr).sin();
        }
        let mut out = vec![0.0f32; n];
        r.process(&input, &mut out);

        let tail = &out[14000..44000];
        let e440 = goertzel_mag(tail, 440.0, sr).max(1e-9);
        let e880 = goertzel_mag(tail, 880.0, sr);
        e880 / e440
    };

    let shimmer_ratio = measure_octave_ratio(ReverbModel::ShimmerCloudAmbient);
    let plate_ratio = measure_octave_ratio(ReverbModel::StudioVintagePlate);
    assert!(
        shimmer_ratio > 0.15 && shimmer_ratio > plate_ratio * 3.0,
        "CA-11: ShimmerCloudAmbient lacks +12 semitone energy (shimmer={shimmer_ratio:.4}, plate={plate_ratio:.4})"
    );
}
