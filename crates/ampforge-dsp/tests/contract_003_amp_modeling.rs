//! crates/ampforge-dsp/tests/contract_003_amp_modeling.rs
//! Testes de Contrato Automatizados para Spec 003: Modelagem White-Box de Amps Valvulados
//! Cobre Critérios de Aceite CA-1 a CA-8

use ampforge_dsp::amp::{
    AmpChannelVoicing, AmpFrontPanel, AmpModelCatalog, AmpModelId, AmpPanelTheme,
    WhiteBoxAmpProcessor,
};

#[test]
fn test_ca_01_front_panel_exposes_11_controls_and_clamps() {
    let mut panel = AmpFrontPanel::default();

    // Verify initial defaults
    assert_eq!(panel.input_gain, 5.0);
    assert_eq!(panel.bass, 5.0);
    assert_eq!(panel.middle, 5.0);
    assert_eq!(panel.treble, 5.0);
    assert_eq!(panel.presence, 5.0);
    assert_eq!(panel.master_volume, 5.0);
    assert_eq!(panel.output_level_db, 0.0);
    assert!(!panel.bright_switch);
    assert_eq!(panel.channel_toggle, AmpChannelVoicing::Norm);

    // Test clamping behavior
    panel.set_input_gain(15.0);
    assert_eq!(panel.input_gain, 10.0);
    panel.set_input_gain(-2.0);
    assert_eq!(panel.input_gain, 0.0);

    panel.set_output_level_db(25.0);
    assert_eq!(panel.output_level_db, 12.0);
    panel.set_output_level_db(-50.0);
    assert_eq!(panel.output_level_db, -24.0);
}

#[test]
fn test_ca_02_amp_model_selection_updates_topology_and_theme() {
    let catalog = AmpModelCatalog::new();

    let models = [
        (AmpModelId::BritJcm800_2203, "MARSHALL™ / BRIT 800", AmpPanelTheme::BrushedBrass),
        (AmpModelId::BritPlexi1959, "PLEXI 1959 SUPER LEAD", AmpPanelTheme::BrushedBrass),
        (AmpModelId::UsTwinBlackface65, "US '65 TWIN REVERB", AmpPanelTheme::BrushedChrome),
        (AmpModelId::CaliDualRectoRevF, "CALI RECTIFIER REV-F", AmpPanelTheme::BlackoutMetal),
        (AmpModelId::UkAc30TopBoost, "UK AC30 TOP BOOST", AmpPanelTheme::BrushedBrass),
    ];

    for (model_id, expected_brand, expected_theme) in models {
        let descriptor = catalog.get_descriptor(model_id);
        assert_eq!(descriptor.brand_text, expected_brand);
        assert_eq!(descriptor.panel_theme, expected_theme);
    }
}

#[test]
fn test_ca_03_bright_switch_behavior_dependent_on_input_gain() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(48000.0);

    let sample_rate = 48000.0;
    let n = 1024;
    let freq_4k = 4000.0;

    // Test signal: 4 kHz sine wave
    let mut test_sig = vec![0.0f32; n];
    for i in 0..n {
        test_sig[i] = 0.1 * (2.0 * std::f32::consts::PI * freq_4k * (i as f32 / sample_rate)).sin();
    }

    // 1. With inputGain = 2.0 (<= 3.0)
    amp.panel_mut().input_gain = 2.0;
    amp.panel_mut().master_volume = 2.0;

    amp.panel_mut().bright_switch = false;
    let mut out_off = vec![0.0f32; n];
    amp.process(&test_sig, &mut out_off);
    let rms_off: f32 = (out_off[256..].iter().map(|s| s * s).sum::<f32>() / (n - 256) as f32).sqrt();

    amp.panel_mut().bright_switch = true;
    let mut out_on = vec![0.0f32; n];
    amp.process(&test_sig, &mut out_on);
    let rms_on: f32 = (out_on[256..].iter().map(|s| s * s).sum::<f32>() / (n - 256) as f32).sqrt();

    let boost_db = 20.0 * (rms_on / rms_off.max(1e-6)).log10();
    assert!(
        boost_db >= 3.5,
        "Bright switch at input_gain=2.0 should give >= +3.5 dB boost at 4kHz, got {} dB",
        boost_db
    );

    // 2. With inputGain = 10.0 (maximum)
    amp.panel_mut().input_gain = 10.0;

    amp.panel_mut().bright_switch = false;
    amp.process(&test_sig, &mut out_off);
    let rms_off_10: f32 = (out_off[256..].iter().map(|s| s * s).sum::<f32>() / (n - 256) as f32).sqrt();

    amp.panel_mut().bright_switch = true;
    amp.process(&test_sig, &mut out_on);
    let rms_on_10: f32 = (out_on[256..].iter().map(|s| s * s).sum::<f32>() / (n - 256) as f32).sqrt();

    let diff_10_db = (20.0 * (rms_on_10 / rms_off_10.max(1e-6)).log10()).abs();
    assert!(
        diff_10_db < 0.2,
        "Bright switch difference at input_gain=10.0 should be < 0.2 dB, got {} dB",
        diff_10_db
    );
}

#[test]
fn test_ca_04_master_volume_increases_thd_vs_output_level_linear_scaling() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(48000.0);
    amp.panel_mut().input_gain = 3.0;

    let n = 2048;
    let freq = 1000.0;
    let mut test_sig = vec![0.0f32; n];
    for i in 0..n {
        test_sig[i] = 0.5 * (2.0 * std::f32::consts::PI * freq * (i as f32 / 48000.0)).sin();
    }

    // A. masterVolume = 2.0
    amp.panel_mut().master_volume = 2.0;
    amp.panel_mut().output_level_db = 0.0;
    let mut out_low_master = vec![0.0f32; n];
    amp.process(&test_sig, &mut out_low_master);
    let thd_low = amp.compute_thd(&out_low_master[512..]);

    // B. masterVolume = 9.0 (power amp overdrive / saturation)
    amp.panel_mut().master_volume = 9.0;
    let mut out_high_master = vec![0.0f32; n];
    amp.process(&test_sig, &mut out_high_master); // conclui rampa de 20 ms
    amp.process(&test_sig, &mut out_high_master);
    let thd_high = amp.compute_thd(&out_high_master[512..]);

    assert!(
        thd_high > thd_low * 1.5,
        "Master volume=9.0 must produce significantly higher THD than master=2.0 (low: {}, high: {})",
        thd_low, thd_high
    );

    // C. Change output_level_db from -12.0 to +6.0 (THD remains constant within 0.01%)
    amp.panel_mut().master_volume = 5.0;
    amp.panel_mut().output_level_db = -12.0;
    let mut out_out1 = vec![0.0f32; n];
    amp.process(&test_sig, &mut out_out1); // conclui rampa de 20 ms do master_volume
    amp.process(&test_sig, &mut out_out1);
    let thd_out1 = amp.compute_thd(&out_out1[512..]);

    amp.panel_mut().output_level_db = 6.0;
    let mut out_out2 = vec![0.0f32; n];
    amp.process(&test_sig, &mut out_out2);
    let thd_out2 = amp.compute_thd(&out_out2[512..]);

    let thd_diff = (thd_out1 - thd_out2).abs();
    assert!(
        thd_diff < 0.0001,
        "Varying outputLevel must not affect THD (diff was {}, expected < 0.0001)",
        thd_diff
    );
}

#[test]
fn test_ca_05_presence_and_depth_in_negative_feedback_loop() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.set_sample_rate(48000.0);

    let n = 1024;
    let sig_5khz: Vec<f32> = (0..n)
        .map(|i| 0.2 * (2.0 * std::f32::consts::PI * 5000.0 * (i as f32 / 48000.0)).sin())
        .collect();

    // Test Presence (5 kHz treble boost in NFB loop)
    amp.panel_mut().presence = 0.0;
    let mut out_pres_0 = vec![0.0f32; n];
    amp.process(&sig_5khz, &mut out_pres_0);
    let rms_p0: f32 = out_pres_0[256..].iter().map(|s| s * s).sum();

    amp.panel_mut().presence = 10.0;
    let mut out_pres_10 = vec![0.0f32; n];
    amp.process(&sig_5khz, &mut out_pres_10);
    let rms_p10: f32 = out_pres_10[256..].iter().map(|s| s * s).sum();

    assert!(rms_p10 > rms_p0 * 1.2, "Presence=10 must boost high frequencies (>2kHz)");
}

#[test]
fn test_ca_06_channel_toggle_voicing_reconfiguration() {
    let n = 512;
    let in_sig: Vec<f32> = (0..n)
        .map(|i| 0.2 * (2.0 * std::f32::consts::PI * 440.0 * (i as f32) / 48000.0).sin())
        .collect();

    let mut amp_norm = WhiteBoxAmpProcessor::new(AmpModelId::CaliDualRectoRevF);
    amp_norm.set_sample_rate(48000.0);
    amp_norm.panel_mut().channel_toggle = AmpChannelVoicing::Norm;
    let mut out_norm = vec![0.0f32; n];
    amp_norm.process(&in_sig, &mut out_norm);

    let mut amp_lead = WhiteBoxAmpProcessor::new(AmpModelId::CaliDualRectoRevF);
    amp_lead.set_sample_rate(48000.0);
    amp_lead.panel_mut().channel_toggle = AmpChannelVoicing::HighLead;
    let mut out_lead = vec![0.0f32; n];
    amp_lead.process(&in_sig, &mut out_lead);

    let sum_norm: f32 = out_norm[128..].iter().map(|s| s.abs()).sum();
    let sum_lead: f32 = out_lead[128..].iter().map(|s| s.abs()).sum();

    assert!(sum_lead > sum_norm, "HighLead channel should exhibit higher gain/saturation");
}

#[test]
fn test_ca_07_invalid_model_id_and_out_of_bounds_handling() {
    let catalog = AmpModelCatalog::new();
    assert!(catalog.find_model_by_name("invalid_model_xyz").is_err());

    let mut panel = AmpFrontPanel::default();
    panel.set_bass(999.0);
    assert_eq!(panel.bass, 10.0);
    panel.set_bass(f32::NAN);
    assert_eq!(panel.bass, 5.0); // defaults back safely on NaN
}

#[test]
fn test_ca_08_non_regression_front_panel_preserves_chassis_mod_settings() {
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
    amp.mod_settings_mut().b_plus_voltage = 450.0;
    amp.mod_settings_mut().cathode_bias_resistor_ohm = 1800.0;

    // Tweak front panel knobs
    amp.panel_mut().input_gain = 8.5;
    amp.panel_mut().master_volume = 7.0;

    // Mod shop settings remain intact
    assert_eq!(amp.mod_settings().b_plus_voltage, 450.0);
    assert_eq!(amp.mod_settings().cathode_bias_resistor_ohm, 1800.0);
}
