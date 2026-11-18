use ampforge_dsp::amp::{AmpChannelMode, AmpInputJack, AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::cab::VirtualCabProcessor;
use ampforge_dsp::fx::eq::{EqMode, EqProcessor};
use ampforge_dsp::modshop::{PowerTube, ToneStackTopology};
use ampforge_dsp::tonestack::{
    analog_tonestack_magnitude_db, resolved_fmv_components, PassiveToneStack,
};
use std::f64::consts::PI;

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum_sq / samples.len() as f64).sqrt() as f32
}

fn measure_sine_gain_db(amp: &mut WhiteBoxAmpProcessor, freq_hz: f32, sample_rate: f32) -> f64 {
    let n = (sample_rate as usize).max(4096);
    let mut input = vec![0.0f32; n];
    for (i, x) in input.iter_mut().enumerate() {
        *x = 0.02 * (2.0 * std::f32::consts::PI * freq_hz * (i as f32) / sample_rate).sin();
    }
    let mut output = vec![0.0f32; n];
    amp.process(&input, &mut output);
    let tail_in = rms(&input[n / 2..]);
    let tail_out = rms(&output[n / 2..]);
    20.0 * ((tail_out as f64) / (tail_in as f64).max(1e-12)).log10()
}

#[test]
fn test_ca_01_fender_ab763_components_and_curve_accuracy() {
    let comps = resolved_fmv_components(ToneStackTopology::FenderTmbMidScoop, 5.0, 5.0, 5.0);
    assert!(
        (comps.c2 - 100e-9).abs() < 1e-12,
        "Fender AB763 c2 must be 100e-9 (0.1 uF), got {}",
        comps.c2
    );
    assert!(
        (comps.c3 - 47e-9).abs() < 1e-12,
        "Fender AB763 c3 must be 47e-9 (0.047 uF), got {}",
        comps.c3
    );

    // Compare digital PassiveToneStack response at 384 kHz (8x OS) against theoretical AB763 curve
    let sr = 384_000.0;
    let mut ts = PassiveToneStack::new();
    for &freq in &[100.0f64, 400.0, 1000.0, 4000.0] {
        ts.reset();
        ts.update(ToneStackTopology::FenderTmbMidScoop, 5.0, 5.0, 5.0, sr);
        let theory_db =
            analog_tonestack_magnitude_db(ToneStackTopology::FenderTmbMidScoop, 5.0, 5.0, 5.0, freq);
        let n = 32_768usize;
        let mut out_sq = 0.0f64;
        let mut in_sq = 0.0f64;
        for i in 0..n {
            let x = (2.0 * PI * freq * (i as f64) / sr).sin();
            let y = ts.process(x);
            if i >= n / 2 {
                in_sq += x * x;
                out_sq += y * y;
            }
        }
        let meas_db = 10.0 * (out_sq / in_sq.max(1e-24)).max(1e-18).log10();
        assert!(
            (meas_db - theory_db).abs() < 1.0,
            "AB763 curve mismatch at {} Hz: measured {:.2} dB vs theory {:.2} dB",
            freq,
            meas_db,
            theory_db
        );
    }
}

#[test]
fn test_ca_02_antialiasing_decimation_passband_flat_and_stopband_attenuated() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp.set_sample_rate(48000.0);
    amp.set_oversampling_factor(8);

    let mut freqs = Vec::new();
    let mut f = 100.0;
    while f <= 18000.0 {
        freqs.push(f);
        f *= 1.35;
    }
    freqs.push(18000.0);

    let mut mags = Vec::new();
    let sr = 48000.0 * 8.0;
    for &freq in &freqs {
        // Measure aa_filters directly via public method
        let mag_db = amp.measure_aa_filter_magnitude_db(freq);
        mags.push((freq, mag_db));
    }

    let max_db = mags.iter().map(|m| m.1).fold(f64::NEG_INFINITY, f64::max);
    let min_db = mags.iter().map(|m| m.1).fold(f64::INFINITY, f64::min);
    let ripple_db = max_db - min_db;
    assert!(
        ripple_db <= 1.0,
        "AA passband ripple 100Hz..18kHz must be <= 1.0 dB, got {:.2} dB ({:?})",
        ripple_db,
        mags
    );

    let stop_24k = amp.measure_aa_filter_magnitude_db(24000.0);
    assert!(
        stop_24k <= -60.0,
        "AA stopband at fs_base/2 (24 kHz, sr={}) must attenuate >= 60 dB, got {:.2} dB",
        sr,
        stop_24k
    );
}

#[test]
fn test_ca_03_plexi_master_volume_and_low2_single_6db_pad() {
    // 1. Plexi master_volume should NOT alter power-stage THD
    let mut amp_m1 = WhiteBoxAmpProcessor::new(AmpModelId::BritPlexi1959);
    let mut amp_m9 = WhiteBoxAmpProcessor::new(AmpModelId::BritPlexi1959);
    amp_m1.panel_mut().set_input_gain(7.0);
    amp_m9.panel_mut().set_input_gain(7.0);
    amp_m1.panel_mut().set_master_volume(1.0);
    amp_m9.panel_mut().set_master_volume(9.0);

    let n = 4096;
    let mut input = vec![0.0f32; n];
    for (i, x) in input.iter_mut().enumerate() {
        *x = 0.25 * (2.0 * std::f32::consts::PI * 1000.0 * (i as f32) / 48000.0).sin();
    }
    let mut out1 = vec![0.0f32; n];
    let mut out9 = vec![0.0f32; n];
    amp_m1.process(&input, &mut out1);
    amp_m9.process(&input, &mut out9);
    let thd1 = amp_m1.compute_thd(&out1[2048..]);
    let thd9 = amp_m9.compute_thd(&out9[2048..]);
    assert!(
        (thd1 - thd9).abs() < 0.01,
        "BritPlexi1959 has no Master Volume; THD at master=1 ({:.4}) vs master=9 ({:.4}) should match",
        thd1,
        thd9
    );

    // 2. Plexi Low2 input pad must be -6 dB +/- 0.2 dB vs High2 (not -12 dB)
    let mut amp_hi2 = WhiteBoxAmpProcessor::new(AmpModelId::BritPlexi1959);
    let mut amp_lo2 = WhiteBoxAmpProcessor::new(AmpModelId::BritPlexi1959);
    amp_hi2.set_input_jack(AmpInputJack::High2);
    amp_lo2.set_input_jack(AmpInputJack::Low2);
    amp_hi2.panel_mut().set_volume_2(5.0);
    amp_lo2.panel_mut().set_volume_2(5.0);
    amp_hi2.panel_mut().set_input_gain(0.0);
    amp_lo2.panel_mut().set_input_gain(0.0);

    let gain_hi2 = measure_sine_gain_db(&mut amp_hi2, 440.0, 48000.0);
    let gain_lo2 = measure_sine_gain_db(&mut amp_lo2, 440.0, 48000.0);
    let diff = gain_lo2 - gain_hi2;
    assert!(
        (diff - (-6.0)).abs() <= 0.2,
        "Plexi Low2 vs High2 must be -6.0 dB +/- 0.2 dB, got {:.2} dB",
        diff
    );
}

#[test]
fn test_ca_04_sample_rate_invariant_coefficients() {
    // 1. Bright cap tilt invariance across 48 kHz and 96 kHz
    let mut amp_48 = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp_48.set_sample_rate(48000.0);
    amp_48.panel_mut().bright_switch = true;
    amp_48.panel_mut().set_input_gain(2.0);
    amp_48.panel_mut().set_reverb(0.0);

    let mut amp_96 = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp_96.set_sample_rate(96000.0);
    amp_96.panel_mut().bright_switch = true;
    amp_96.panel_mut().set_input_gain(2.0);
    amp_96.panel_mut().set_reverb(0.0);

    let g48_low = measure_sine_gain_db(&mut amp_48, 200.0, 48000.0);
    let g48_high = measure_sine_gain_db(&mut amp_48, 5000.0, 48000.0);
    let g96_low = measure_sine_gain_db(&mut amp_96, 200.0, 96000.0);
    let g96_high = measure_sine_gain_db(&mut amp_96, 5000.0, 96000.0);

    let tilt_48 = g48_high - g48_low;
    let tilt_96 = g96_high - g96_low;
    assert!(
        (tilt_48 - tilt_96).abs() < 0.8,
        "Bright tilt must be invariant across sample rates: 48k={:.2} dB vs 96k={:.2} dB",
        tilt_48,
        tilt_96
    );

    // 2. Spring reverb tail high-frequency damping invariance across 48 kHz and 96 kHz
    let mut rev_48 = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    rev_48.set_sample_rate(48000.0);
    rev_48.panel_mut().set_reverb(10.0);
    let mut in48 = vec![0.0f32; 24000]; // 500 ms
    for i in 0..480 {
        in48[i] = 0.2 * (2.0 * std::f32::consts::PI * 1000.0 * (i as f32) / 48000.0).sin();
    }
    let mut out48 = vec![0.0f32; 24000];
    rev_48.process(&in48, &mut out48);
    let tail48 = rms(&out48[12000..]);

    let mut rev_96 = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    rev_96.set_sample_rate(96000.0);
    rev_96.panel_mut().set_reverb(10.0);
    let mut in96 = vec![0.0f32; 48000]; // 500 ms
    for i in 0..960 {
        in96[i] = 0.2 * (2.0 * std::f32::consts::PI * 1000.0 * (i as f32) / 96000.0).sin();
    }
    let mut out96 = vec![0.0f32; 48000];
    rev_96.process(&in96, &mut out96);
    let tail96 = rms(&out96[24000..]);

    let ratio_db = 20.0 * ((tail96 as f64) / (tail48 as f64).max(1e-12)).log10();
    assert!(
        ratio_db.abs() < 3.0,
        "Spring reverb tail RMS over 250-500 ms must match across 48k/96k within 3 dB, got {:.2} dB",
        ratio_db
    );
}

#[test]
fn test_ca_06_set_target_model_crossfade_delays_mod_settings_and_avoids_clicks() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp.set_sample_rate(48000.0);
    amp.panel_mut().set_input_gain(5.0);

    // Warm up with steady tone
    let block = 128;
    let mut in_buf = vec![0.0f32; block];
    let mut out_buf = vec![0.0f32; block];
    for i in 0..block {
        in_buf[i] = 0.08 * (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / 48000.0).sin();
    }
    for _ in 0..10 {
        amp.process(&in_buf, &mut out_buf);
    }

    let initial_power_tube = amp.mod_settings().power_tube;
    assert_eq!(initial_power_tube, PowerTube::Tube6L6Gc);

    amp.set_target_model(AmpModelId::UkAc30TopBoost);
    // Immediately after set_target_model, mod_settings must NOT have jumped to EL84 yet
    assert_eq!(
        amp.mod_settings().power_tube,
        PowerTube::Tube6L6Gc,
        "mod_settings must not update before 20 ms crossfade finishes"
    );

    let mut max_step_jump = 0.0f32;
    let mut prev = out_buf[block - 1];
    // Process 12 blocks (~32 ms > 20 ms)
    for b in 0..12 {
        for i in 0..block {
            let idx = (b * block + i) as f32;
            in_buf[i] = 0.08 * (2.0 * std::f32::consts::PI * 440.0 * idx / 48000.0).sin();
        }
        amp.process(&in_buf, &mut out_buf);
        for &s in &out_buf {
            let jump = (s - prev).abs();
            if jump > max_step_jump {
                max_step_jump = jump;
            }
            prev = s;
        }
    }

    assert!(
        max_step_jump < 0.25,
        "Model crossfade must be glitch-free without step clicks, max_step_jump={:.4}",
        max_step_jump
    );
    assert_eq!(
        amp.mod_settings().power_tube,
        PowerTube::El84,
        "mod_settings must commit to target model (EL84) once crossfade finishes"
    );
}

#[test]
fn test_ca_07_voicing_fontes_doc_exists_for_all_models() {
    let doc = include_str!("../../../docs/amps/voicing-fontes.md");
    for needle in [
        "UsTwinBlackface65",
        "BritJcm800_2203",
        "BritPlexi1959",
        "CaliDualRectoRevF",
        "UkAc30TopBoost",
        "AB763",
    ] {
        assert!(doc.contains(needle), "docs/amps/voicing-fontes.md missing {}", needle);
    }
}

#[test]
fn test_ca_08_presence_and_depth_scale_with_effective_nfb() {
    // Dual Rectifier Modern mode disconnects global NFB (eff_nfb = 0)
    let mut rect_mod_0 = WhiteBoxAmpProcessor::new(AmpModelId::CaliDualRectoRevF);
    rect_mod_0.set_channel_mode(AmpChannelMode::Modern);
    rect_mod_0.panel_mut().set_presence(0.0);
    rect_mod_0.panel_mut().set_depth(0.0);

    let mut rect_mod_10 = WhiteBoxAmpProcessor::new(AmpModelId::CaliDualRectoRevF);
    rect_mod_10.set_channel_mode(AmpChannelMode::Modern);
    rect_mod_10.panel_mut().set_presence(10.0);
    rect_mod_10.panel_mut().set_depth(10.0);

    let g0_hi = measure_sine_gain_db(&mut rect_mod_0, 4200.0, 48000.0);
    let g10_hi = measure_sine_gain_db(&mut rect_mod_10, 4200.0, 48000.0);
    assert!(
        (g10_hi - g0_hi).abs() < 0.2,
        "With eff_nfb == 0 (Dual Rectifier Modern), Presence should have no effect: diff={:.2} dB",
        g10_hi - g0_hi
    );

    // Vintage mode has NFB connected -> Presence=10 boosts 4.2 kHz noticeably vs Presence=0
    let mut rect_vin_0 = WhiteBoxAmpProcessor::new(AmpModelId::CaliDualRectoRevF);
    rect_vin_0.set_channel_mode(AmpChannelMode::Vintage);
    rect_vin_0.set_negative_feedback(8.0);
    rect_vin_0.panel_mut().set_presence(0.0);

    let mut rect_vin_10 = WhiteBoxAmpProcessor::new(AmpModelId::CaliDualRectoRevF);
    rect_vin_10.set_channel_mode(AmpChannelMode::Vintage);
    rect_vin_10.set_negative_feedback(8.0);
    rect_vin_10.panel_mut().set_presence(10.0);

    let v0_hi = measure_sine_gain_db(&mut rect_vin_0, 4200.0, 48000.0);
    let v10_hi = measure_sine_gain_db(&mut rect_vin_10, 4200.0, 48000.0);
    assert!(
        (v10_hi - v0_hi) > 2.0,
        "With NFB connected, Presence=10 must boost 4.2 kHz > 2 dB, got {:.2} dB",
        v10_hi - v0_hi
    );
}

#[test]
fn test_ca_09_five_amp_models_distinct_tonal_footprint_and_transient_attack() {
    // 1. Schematic component verification: Marshall 1959/2203 uses 220k Treble and 22k Mid pots
    let m_comps = resolved_fmv_components(ToneStackTopology::MarshallCathodeFollower, 5.0, 5.0, 5.0);
    assert!(
        (m_comps.r_treble - 220_000.0).abs() < 1.0 && (m_comps.r_mid - 22_000.0).abs() < 1.0,
        "Marshall FMV deve usar R_treble=220k e R_mid=22k (obtido treb={}, mid={})",
        m_comps.r_treble,
        m_comps.r_mid
    );

    // 2. Cab default hi-cut (4.2 kHz for standalone 4x12 cab) supports calibrating to 8.2 kHz for open transient clarity
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(48000.0);
    let tone_6k: Vec<f32> = (0..4096)
        .map(|i| 0.1 * (2.0 * std::f32::consts::PI * 6000.0 * (i as f32 / 48000.0)).sin())
        .collect();
    let mut out_4200 = vec![0.0f32; 4096];
    cab.process(&tone_6k, &mut out_4200);
    let db_6k_at_4200 = 20.0 * (rms(&out_4200[1024..]) / 0.07071f32).max(1e-6).log10();

    cab.set_hicut_khz(8.2);
    assert!(
        (cab.hicut_khz() - 8.2).abs() < 0.1,
        "VirtualCabProcessor.set_hicut_khz(8.2) deve ajustar para 8.2 kHz, obteve {} kHz",
        cab.hicut_khz()
    );
    let mut out_8200 = vec![0.0f32; 4096];
    cab.process(&tone_6k, &mut out_8200);
    let db_6k_at_8200 = 20.0 * (rms(&out_8200[1024..]) / 0.07071f32).max(1e-6).log10();
    assert!(
        db_6k_at_8200 - db_6k_at_4200 > 4.0,
        "Ajustar hi-cut para 8.2 kHz deve recuperar > 4 dB em 6 kHz vs 4.2 kHz (diff={:.2} dB)",
        db_6k_at_8200 - db_6k_at_4200
    );

    // 3. Level-matched THD progression under default overdrive=5.0 at gain=5.0, 1 kHz 0.06 peak
    let sr = 48000.0f32;
    let n = 4800usize;
    let tone_1k: Vec<f32> = (0..n)
        .map(|i| 0.06 * (2.0 * std::f32::consts::PI * 1000.0 * (i as f32 / sr)).sin())
        .collect();

    let measure_model_thd = |model: AmpModelId, ch_mode: AmpChannelMode| -> f32 {
        let mut amp = WhiteBoxAmpProcessor::new(model);
        amp.set_sample_rate(sr);
        amp.set_channel_mode(ch_mode);
        amp.panel_mut().set_input_gain(5.0);
        amp.panel_mut().set_master_volume(5.0);
        let mut out = vec![0.0f32; n];
        amp.process(&tone_1k, &mut out);
        amp.compute_thd(&out[2400..])
    };

    let thd_twin = measure_model_thd(AmpModelId::UsTwinBlackface65, AmpChannelMode::Clean);
    let thd_ac30 = measure_model_thd(AmpModelId::UkAc30TopBoost, AmpChannelMode::Clean);
    let thd_plexi = measure_model_thd(AmpModelId::BritPlexi1959, AmpChannelMode::Clean);
    let thd_jcm = measure_model_thd(AmpModelId::BritJcm800_2203, AmpChannelMode::Vintage);
    let thd_rect = measure_model_thd(AmpModelId::CaliDualRectoRevF, AmpChannelMode::Modern);

    assert!(
        thd_twin < 0.08,
        "Twin Reverb AB763 em gain=5 (overdrive padrão) deve manter headroom limpo (THD < 8%, foi {:.2}%)",
        thd_twin * 100.0
    );
    assert!(
        thd_jcm > thd_twin * 2.0 && thd_rect > thd_jcm * 1.15,
        "Progression esperada THD Twin ({:.4}) << JCM800 ({:.4}) < Rectifier Modern ({:.4})",
        thd_twin, thd_jcm, thd_rect
    );
    assert!(
        thd_ac30 > thd_twin * 1.2 && thd_plexi > thd_twin * 1.2,
        "AC30 ({:.4}) e Plexi ({:.4}) devem saturar acima do Twin limpo ({:.4})",
        thd_ac30, thd_plexi, thd_twin
    );

    // 4. Band energy separation across all 5 amps using identical multi-harmonic guitar signal + transient burst
    let sig_len = 16384usize;
    let test_chord: Vec<f32> = (0..sig_len)
        .map(|i| {
            let t = i as f32 / sr;
            let attack = if i < 960 { 1.0 + 1.5 * (-t * 220.0).exp() } else { 1.0 };
            0.05
                * attack
                * (1.0 * (2.0 * std::f32::consts::PI * 110.0 * t).sin()
                    + 0.75 * (2.0 * std::f32::consts::PI * 330.0 * t).sin()
                    + 0.50 * (2.0 * std::f32::consts::PI * 880.0 * t).sin()
                    + 0.35 * (2.0 * std::f32::consts::PI * 2400.0 * t).sin()
                    + 0.25 * (2.0 * std::f32::consts::PI * 4200.0 * t).sin())
        })
        .collect();

    let band_profile = |model: AmpModelId, ch_mode: AmpChannelMode| -> [f64; 3] {
        let mut amp = WhiteBoxAmpProcessor::new(model);
        amp.set_sample_rate(sr);
        amp.set_channel_mode(ch_mode);
        amp.panel_mut().set_input_gain(6.0);
        amp.panel_mut().set_master_volume(5.5);
        let mut out = vec![0.0f32; sig_len];
        amp.process(&test_chord, &mut out);
        let steady = &out[4096..];
        let total_rms = (rms(steady) as f64).max(1e-9);
        let mut lp1 = 0.0f64;
        let mut lp2 = 0.0f64;
        let a_low = 1.0 - (-2.0 * std::f64::consts::PI * 250.0 / sr as f64).exp();
        let a_mid = 1.0 - (-2.0 * std::f64::consts::PI * 1800.0 / sr as f64).exp();
        let (mut e_low, mut e_mid, mut e_hi) = (0.0f64, 0.0f64, 0.0f64);
        for &s in steady {
            let x = (s as f64) / total_rms;
            lp1 += a_low * (x - lp1);
            lp2 += a_mid * (x - lp2);
            let low = lp1;
            let mid = lp2 - lp1;
            let hi = x - lp2;
            e_low += low * low;
            e_mid += mid * mid;
            e_hi += hi * hi;
        }
        let n_f = steady.len() as f64;
        [
            10.0 * (e_low / n_f).max(1e-12).log10(),
            10.0 * (e_mid / n_f).max(1e-12).log10(),
            10.0 * (e_hi / n_f).max(1e-12).log10(),
        ]
    };

    let profiles = [
        ("Twin", band_profile(AmpModelId::UsTwinBlackface65, AmpChannelMode::Clean)),
        ("Plexi", band_profile(AmpModelId::BritPlexi1959, AmpChannelMode::Clean)),
        ("JCM800", band_profile(AmpModelId::BritJcm800_2203, AmpChannelMode::Vintage)),
        ("AC30", band_profile(AmpModelId::UkAc30TopBoost, AmpChannelMode::Clean)),
        ("Recto", band_profile(AmpModelId::CaliDualRectoRevF, AmpChannelMode::Modern)),
    ];

    for i in 0..profiles.len() {
        for j in (i + 1)..profiles.len() {
            let max_band_diff = (0..3)
                .map(|b| (profiles[i].1[b] - profiles[j].1[b]).abs())
                .fold(0.0f64, f64::max);
            assert!(
                max_band_diff >= 2.0,
                "Amps {} ({:?}) e {} ({:?}) soam próximos demais após normalização (max_diff={:.2} dB < 2.0 dB)",
                profiles[i].0, profiles[i].1, profiles[j].0, profiles[j].1, max_band_diff
            );
        }
    }

    // 5. Transient pick attack preservation (crest factor >= 2.2 on pick onset vs sustain)
    let mut pick_sig = vec![0.0f32; 8192];
    for i in 0..8192 {
        let t = i as f32 / sr;
        let env = if i < 2400 {
            0.05
        } else {
            let dt = (i - 2400) as f32 / sr;
            0.05 + 0.30 * (-dt * 85.0).exp()
        };
        pick_sig[i] = env * (2.0 * std::f32::consts::PI * 220.0 * t).sin();
    }
    let mut amp_pick = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp_pick.set_sample_rate(sr);
    amp_pick.panel_mut().set_input_gain(5.5);
    amp_pick.panel_mut().set_master_volume(6.0);
    let mut out_pick = vec![0.0f32; 8192];
    amp_pick.process(&pick_sig, &mut out_pick);
    let sustain_rms = rms(&out_pick[1024..2300]).max(1e-6);
    let onset_peak = out_pick[2400..3000].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let crest = onset_peak / sustain_rms;
    assert!(
        crest >= 2.5,
        "Ataque transiente esmagado (crest={:.2} < 2.5, peak={:.4}, sustain_rms={:.4})",
        crest, onset_peak, sustain_rms
    );

    // 6. Fender AB763 pre-V1b tone stack placement: sweeping Bass with high Gain alters downstream clipping harmonics
    let measure_twin_bass_thd = |bass: f32| -> f32 {
        let mut t = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
        t.set_sample_rate(sr);
        t.panel_mut().set_input_gain(8.5);
        t.panel_mut().set_bass(bass);
        let low_note: Vec<f32> = (0..4800)
            .map(|i| 0.14 * (2.0 * std::f32::consts::PI * 100.0 * (i as f32 / sr)).sin())
            .collect();
        let mut out = vec![0.0f32; 4800];
        t.process(&low_note, &mut out);
        // measure 300 Hz (3rd harmonic) ratio
        let tail = &out[2400..];
        let mut r1 = 0.0f64;
        let mut i1 = 0.0f64;
        let mut r3 = 0.0f64;
        let mut i3 = 0.0f64;
        for (idx, &s) in tail.iter().enumerate() {
            let a1 = 2.0 * std::f64::consts::PI * 100.0 * idx as f64 / sr as f64;
            let a3 = 3.0 * a1;
            r1 += s as f64 * a1.cos();
            i1 -= s as f64 * a1.sin();
            r3 += s as f64 * a3.cos();
            i3 -= s as f64 * a3.sin();
        }
        ((r3 * r3 + i3 * i3).sqrt() / (r1 * r1 + i1 * i1).sqrt().max(1e-12)) as f32
    };
    let h3_b0 = measure_twin_bass_thd(0.0);
    let h3_b10 = measure_twin_bass_thd(10.0);
    assert!(
        (h3_b10 - h3_b0).abs() > 0.015,
        "Tone Stack AB763 pré-V1b deve alterar geração harmônica na V1b ao varrer Bass 0->10 (h3_b0={:.4}, h3_b10={:.4})",
        h3_b0, h3_b10
    );
}

#[test]
fn test_ca_10_mxr_10_band_eq_selective_q_reduces_adjacent_octave_overlap() {
    let sr = 48000.0f32;
    let mut eq = EqProcessor::new(EqMode::Graphic10BandPedal);
    eq.set_sample_rate(sr);
    // Boost 250 Hz (index 3) by +12 dB; leave 125 Hz (index 2) and 500 Hz (index 4) at 0 dB
    eq.set_graphic_band_db(3, 12.0);

    let gain_at = |eq_p: &mut EqProcessor, f_hz: f32| -> f32 {
        let n = 8192;
        let inp: Vec<f32> = (0..n)
            .map(|i| 0.1 * (2.0 * std::f32::consts::PI * f_hz * (i as f32 / sr)).sin())
            .collect();
        let mut out = vec![0.0f32; n];
        eq_p.process(&inp, &mut out);
        20.0 * (rms(&out[4096..]) / rms(&inp[4096..]).max(1e-9)).log10()
    };

    let g_250 = gain_at(&mut eq, 250.0);
    let g_500 = gain_at(&mut eq, 500.0);
    assert!(
        g_250 >= 11.0 && g_500 <= 3.5,
        "MXR 10-Band EQ deve manter +12 dB focado em 250 Hz ({:.2} dB) com vazamento limitado em 500 Hz ({:.2} dB <= 3.5 dB)",
        g_250, g_500
    );
}
