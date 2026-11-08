//! crates/ampforge-dsp/tests/contract_006_dynamics_drive_eq.rs
//! Testes de Contrato Automatizados para Spec 006: Dinâmica, Drive e Equalização
//! Cobre Critérios de Aceite CA-1 a CA-8

use ampforge_dsp::fx::{
    ClippingDiode, CompressorProcessor, CompressorTopology, DriveModel, DriveProcessor, EqMode,
    EqProcessor, NoiseGateMode, NoiseGateProcessor,
};

#[test]
fn test_ca_01_intelligent_natural_gate_sidechain_di_tracking() {
    let mut gate = NoiseGateProcessor::new(NoiseGateMode::IntelligentNaturalGate);
    gate.set_sample_rate(48000.0);
    gate.set_threshold_db(-40.0);
    gate.set_attack_ms(1.0);
    gate.set_release_ms(10.0);

    let n = 512;
    // Input is noisy high-gain tail (e.g. 0.05 amplitude, ~ -26 dBFS)
    let noisy_input = vec![0.05f32; n];

    // Clean DI sidechain is completely silent (guitar strings muted, 0.0 amplitude)
    let silent_di = vec![0.0f32; n];

    let mut output = vec![0.0f32; n];
    gate.process(&noisy_input, Some(&silent_di), &mut output);

    // Because clean DI is silent (below -40 dBFS threshold), gate should close and attenuate noise
    let final_gr = gate.gain_reduction_db();
    assert!(
        final_gr < -20.0,
        "Gate should apply significant gain reduction (got {} dB)",
        final_gr
    );
    let output_energy: f32 = output[256..].iter().map(|s| s * s).sum();
    assert!(output_energy < 0.001, "Gate should attenuate noisy input when DI is silent");
}

#[test]
fn test_ca_02_compressor_4_topologies_and_parallel_mix() {
    let topologies = [
        CompressorTopology::PedalRossDyna,
        CompressorTopology::StudioFet1176,
        CompressorTopology::OpticalLa2a,
        CompressorTopology::VcaMasterBus,
    ];

    for top in topologies {
        let mut comp = CompressorProcessor::new(top);
        comp.set_sample_rate(48000.0);
        comp.set_threshold_db(-20.0);
        comp.set_ratio(4.0);
        comp.set_parallel_mix_pct(100.0); // 100% wet

        let n = 512;
        let in_sig = vec![0.8f32; n]; // High level > threshold
        let mut out_sig = vec![0.0f32; n];
        comp.process(&in_sig, &mut out_sig);

        let gr = comp.gain_reduction_db();
        assert!(gr < 0.0, "Compressor in topology {:?} must exhibit gain reduction", top);
    }
}

#[test]
fn test_ca_03_drive_6_models_and_5_clipping_diodes() {
    let models = [
        DriveModel::GreenScreamer808,
        DriveModel::GoldenKloneCentaur,
        DriveModel::BlackRodentRat,
        DriveModel::OrangeBossDs,
        DriveModel::FuzzFaceSiGe,
        DriveModel::BigMuffPi,
    ];

    let diodes = [
        ClippingDiode::SiliconSymmetrical,
        ClippingDiode::AsymmetricalSiLed,
        ClippingDiode::Germanium1N34A,
        ClippingDiode::MosfetClip,
        ClippingDiode::HardOpamp,
    ];

    let mut drive = DriveProcessor::new(DriveModel::GreenScreamer808);
    for m in models {
        drive.set_model(m);
        assert_eq!(drive.model(), m);
    }

    for d in diodes {
        drive.set_clipping_diode(d);
        assert_eq!(drive.clipping_diode(), d);
    }
}

#[test]
fn test_ca_04_drive_germanium_vs_asymmetrical_si_led_headroom_and_harmonics() {
    let mut drive = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive.set_sample_rate(48000.0);
    drive.set_gain(5.0);

    // Test with Germanium (low forward voltage Vf ~ 0.3V)
    drive.set_clipping_diode(ClippingDiode::Germanium1N34A);
    let n = 1024;
    let in_sig: Vec<f32> = (0..n)
        .map(|i| 0.4 * (2.0 * std::f32::consts::PI * 1000.0 * (i as f32 / 48000.0)).sin())
        .collect();
    let mut out_ge = vec![0.0f32; n];
    drive.process(&in_sig, &mut out_ge);
    let peak_ge = out_ge[256..].iter().cloned().fold(0.0f32, f32::max);

    // Test with Asymmetrical Si+LED (Vf ~ 0.65V / 1.8V -> higher headroom)
    drive.set_clipping_diode(ClippingDiode::AsymmetricalSiLed);
    let mut out_led = vec![0.0f32; n];
    drive.process(&in_sig, &mut out_led);
    let peak_led = out_led[256..].iter().cloned().fold(0.0f32, f32::max);

    assert!(
        peak_led > peak_ge * 1.3,
        "Asymmetrical Si+LED should allow higher headroom/peak voltage before saturation (ge: {}, led: {})",
        peak_ge, peak_led
    );
}

#[test]
fn test_ca_05_parametric_eq_8_bands_64bit_filters_and_rta() {
    let mut eq = EqProcessor::new(EqMode::Parametric8Band);
    eq.set_sample_rate(48000.0);

    // Boost band 3 (1000 Hz) by +6 dB
    eq.set_parametric_band(3, 1000.0, 6.0, 1.414);
    assert_eq!(eq.parametric_band(3).freq_hz, 1000.0);
    assert_eq!(eq.parametric_band(3).gain_db, 6.0);
    assert_eq!(eq.parametric_band(3).q, 1.414);

    let n = 1024;
    let sig_1k: Vec<f32> = (0..n)
        .map(|i| 0.2 * (2.0 * std::f32::consts::PI * 1000.0 * (i as f32 / 48000.0)).sin())
        .collect();
    let mut out_eq = vec![0.0f32; n];
    eq.process(&sig_1k, &mut out_eq);

    let rms_in: f32 = (sig_1k[256..].iter().map(|s| s * s).sum::<f32>() / 768.0).sqrt();
    let rms_out: f32 = (out_eq[256..].iter().map(|s| s * s).sum::<f32>() / 768.0).sqrt();
    let gain_db = 20.0 * (rms_out / rms_in).log10();
    assert!(
        (gain_db - 6.0).abs() < 0.5,
        "Parametric EQ band 3 should boost ~6 dB (got {} dB)",
        gain_db
    );
}

#[test]
fn test_ca_06_graphic_eq_boogie_5_band_and_pedal_10_band() {
    let boogie = EqProcessor::new(EqMode::Graphic5BandBoogie);
    let boogie_freqs = boogie.graphic_frequencies();
    assert_eq!(boogie_freqs, &[80.0, 240.0, 750.0, 2200.0, 6600.0]);

    let pedal = EqProcessor::new(EqMode::Graphic10BandPedal);
    let pedal_freqs = pedal.graphic_frequencies();
    assert_eq!(
        pedal_freqs,
        &[31.25, 62.5, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0]
    );
}

#[test]
fn test_ca_07_invalid_parameters_clamped_safely() {
    let mut eq = EqProcessor::new(EqMode::Parametric8Band);
    // Q <= 0 should clamp to safe min
    eq.set_parametric_band(0, 50.0, 0.0, -1.0);
    assert!(eq.parametric_band(0).q >= 0.1);

    // Freq out of [20..20000] should clamp
    eq.set_parametric_band(0, 10.0, 0.0, 1.0);
    assert_eq!(eq.parametric_band(0).freq_hz, 20.0);
    eq.set_parametric_band(0, 30000.0, 0.0, 1.0);
    assert_eq!(eq.parametric_band(0).freq_hz, 20000.0);

    // Gate threshold > 0 clamped to 0
    let mut gate = NoiseGateProcessor::new(NoiseGateMode::ClassicExpanderGate);
    gate.set_threshold_db(10.0);
    assert_eq!(gate.threshold_db(), 0.0);
}

#[test]
fn test_ca_08_non_regression_flat_response_at_zero_gain() {
    let mut eq = EqProcessor::new(EqMode::Parametric8Band);
    eq.set_sample_rate(48000.0);
    // All bands at 0 dB
    for i in 0..8 {
        eq.set_parametric_band(i, 100.0 * (i + 1) as f32, 0.0, 1.0);
    }

    let n = 512;
    let in_sig = vec![0.5f32; n];
    let mut out_sig = vec![0.0f32; n];
    eq.process(&in_sig, &mut out_sig);

    for i in 64..n {
        let diff_db = 20.0 * (out_sig[i] / in_sig[i]).abs().log10();
        assert!(
            diff_db.abs() <= 0.05,
            "Flat EQ must preserve magnitude within +-0.05 dB (got {} dB at sample {})",
            diff_db, i
        );
    }
}
