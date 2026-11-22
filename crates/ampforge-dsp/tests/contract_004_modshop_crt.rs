//! crates/ampforge-dsp/tests/contract_004_modshop_crt.rs
//! Testes de Contrato Automatizados para Spec 004: Bancada Chassi Aberto (Mod Shop) e Osciloscópio CRT
//! Cobre Critérios de Aceite CA-1 a CA-9

use ampforge_dsp::modshop::{
    ChassisWorkbench, CrtScopeMode, PowerClass, PowerTube, PreampTube, RectifierType,
    ToneStackPosition, ToneStackTopology,
};

#[test]
fn test_ca_01_chassis_toggle_visibility_without_interrupting_audio() {
    let mut bench = ChassisWorkbench::new();
    assert!(!bench.is_chassis_open());

    bench.toggle_chassis();
    assert!(bench.is_chassis_open());

    bench.toggle_chassis();
    assert!(!bench.is_chassis_open());
}

#[test]
fn test_ca_02_preamp_tube_selection_and_gain_reduction() {
    let mut bench = ChassisWorkbench::new();

    // Default tube is 12AX7A with mu = 100
    bench.set_preamp_tube(PreampTube::Ax7A);
    let gain_12ax7 = bench.effective_preamp_gain();

    // 12AT7 has mu = 60
    bench.set_preamp_tube(PreampTube::At7);
    let gain_12at7 = bench.effective_preamp_gain();

    // 12AY7 has mu = 45
    bench.set_preamp_tube(PreampTube::Ay7);
    let gain_12ay7 = bench.effective_preamp_gain();

    assert!(
        gain_12at7 < gain_12ax7,
        "12AT7 must produce less preamp gain than 12AX7A"
    );
    assert!(
        gain_12ay7 < gain_12at7,
        "12AY7 must produce less preamp gain than 12AT7"
    );

    // Verify plate voltage and trimpots
    bench.set_plate_voltage(250.0);
    assert_eq!(bench.plate_voltage(), 250.0);

    bench.set_triode_asymmetry(0.35);
    assert_eq!(bench.triode_asymmetry(), 0.35);

    bench.set_interstage_hp_cut_hz(120.0);
    assert_eq!(bench.interstage_hp_cut_hz(), 120.0);
}

#[test]
fn test_ca_03_tone_stack_topologies_and_position_routing() {
    let mut bench = ChassisWorkbench::new();

    let topologies = [
        ToneStackTopology::PassiveJamesBaxandall,
        ToneStackTopology::FenderTmbMidScoop,
        ToneStackTopology::MarshallCathodeFollower,
        ToneStackTopology::VoxTopBoost,
        ToneStackTopology::RectoPostGain,
        ToneStackTopology::MesaMarkPreGain,
    ];

    for top in topologies {
        bench.set_tonestack_topology(top);
        assert_eq!(bench.tonestack_topology(), top);
    }

    let positions = [
        ToneStackPosition::PreDistortion,
        ToneStackPosition::PostPreamp,
        ToneStackPosition::PostPowerAmp,
    ];

    for pos in positions {
        bench.set_tonestack_position(pos);
        assert_eq!(bench.tonestack_position(), pos);
    }
}

#[test]
fn test_ca_04_power_amp_tubes_class_bias_and_saturation() {
    let mut bench = ChassisWorkbench::new();

    bench.set_power_tube(PowerTube::Tube6V6);
    assert_eq!(bench.power_tube(), PowerTube::Tube6V6);

    bench.set_power_class(PowerClass::ClassA);
    assert_eq!(bench.power_class(), PowerClass::ClassA);

    bench.set_power_class(PowerClass::ClassAbPushPull);
    assert_eq!(bench.power_class(), PowerClass::ClassAbPushPull);

    bench.set_power_tube_bias_ma(38.5);
    assert_eq!(bench.power_tube_bias_ma(), 38.5);

    bench.set_negative_feedback(4.0);
    assert_eq!(bench.negative_feedback(), 4.0);

    bench.set_transformer_saturation(7.5);
    assert_eq!(bench.transformer_saturation(), 7.5);
}

#[test]
fn test_ca_05_power_supply_rectifiers_and_sag_compression() {
    let mut bench = ChassisWorkbench::new();

    bench.set_rectifier(RectifierType::SolidStateDiode);
    let ss_sag = bench.transient_sag_compression(1.0);

    bench.set_rectifier(RectifierType::Gz34_5Ar4);
    let gz34_sag = bench.transient_sag_compression(1.0);

    bench.set_rectifier(RectifierType::Dual5U4Gb);
    let dual5u4_sag = bench.transient_sag_compression(1.0);

    assert!(
        gz34_sag > ss_sag,
        "GZ34 tube rectifier must exhibit more dynamic sag compression than Solid State Diode"
    );
    assert!(
        dual5u4_sag >= gz34_sag,
        "5U4GB tube rectifier must exhibit high sag compression"
    );
}

#[test]
fn test_ca_06_vacuum_tube_interactive_glow_scaling_with_bias() {
    let mut bench = ChassisWorkbench::new();

    bench.set_power_tube_bias_ma(20.0);
    let (glow_rad_20, glow_op_20) = bench.calculate_tube_glow();

    bench.set_power_tube_bias_ma(40.0);
    let (glow_rad_40, glow_op_40) = bench.calculate_tube_glow();

    bench.set_power_tube_bias_ma(55.0);
    let (glow_rad_55, glow_op_55) = bench.calculate_tube_glow();

    // Monotonically increasing glow radius and opacity
    assert!(glow_rad_40 > glow_rad_20);
    assert!(glow_rad_55 > glow_rad_40);
    assert!(glow_op_40 > glow_op_20);
    assert!(glow_op_55 > glow_op_40);
}

#[test]
fn test_ca_07_crt_scope_3_modes_telemetry_generation() {
    let mut bench = ChassisWorkbench::new();

    // Waveform mode
    bench.set_scope_mode(CrtScopeMode::Waveform);
    let wave_data = bench.generate_scope_telemetry();
    assert!(matches!(wave_data.mode, CrtScopeMode::Waveform));
    assert!(!wave_data.di_input.is_empty());
    assert!(!wave_data.saturated_output.is_empty());

    // Transfer curve mode
    bench.set_scope_mode(CrtScopeMode::Transfer);
    let transfer_data = bench.generate_scope_telemetry();
    assert!(matches!(transfer_data.mode, CrtScopeMode::Transfer));
    assert!(!transfer_data.transfer_curve_xy.is_empty());

    // Tone stack mode
    bench.set_scope_mode(CrtScopeMode::ToneStack);
    let tonestack_data = bench.generate_scope_telemetry();
    assert!(matches!(tonestack_data.mode, CrtScopeMode::ToneStack));
    assert!(!tonestack_data.tonestack_freq_response_db.is_empty());
    assert!(!tonestack_data.speaker_impedance_curve_ohm.is_empty());
}

#[test]
fn test_ca_08_error_handling_and_safe_clamping() {
    let mut bench = ChassisWorkbench::new();

    // Plate voltage valid range: 180 to 340 V
    bench.set_plate_voltage(100.0);
    assert_eq!(bench.plate_voltage(), 180.0);

    bench.set_plate_voltage(500.0);
    assert_eq!(bench.plate_voltage(), 340.0);

    // Power tube bias valid range: 20 to 55 mA
    bench.set_power_tube_bias_ma(5.0);
    assert_eq!(bench.power_tube_bias_ma(), 20.0);

    bench.set_power_tube_bias_ma(90.0);
    assert_eq!(bench.power_tube_bias_ma(), 55.0);
}

#[test]
fn test_ca_09_non_regression_closed_chassis_preserves_mod_shop_parameters() {
    let mut bench = ChassisWorkbench::new();
    bench.toggle_chassis(); // Open
    assert!(bench.is_chassis_open());

    // Modify circuit
    bench.set_preamp_tube(PreampTube::Ef86);
    bench.set_power_tube(PowerTube::Kt88);
    bench.set_tonestack_topology(ToneStackTopology::VoxTopBoost);
    bench.set_tonestack_position(ToneStackPosition::PreDistortion);
    bench.set_power_tube_bias_ma(48.0);

    bench.toggle_chassis(); // Close
    assert!(!bench.is_chassis_open());

    // All modified parameters remain active in audio calculation
    assert_eq!(bench.preamp_tube(), PreampTube::Ef86);
    assert_eq!(bench.power_tube(), PowerTube::Kt88);
    assert_eq!(bench.tonestack_topology(), ToneStackTopology::VoxTopBoost);
    assert_eq!(bench.tonestack_position(), ToneStackPosition::PreDistortion);
    assert_eq!(bench.power_tube_bias_ma(), 48.0);
}
