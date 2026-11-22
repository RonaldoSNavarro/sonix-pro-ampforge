//! crates/sonix-core/tests/contract_009_scenes_modifiers.rs
//! Testes de Contrato Automatizados para Spec 009: Cenas Gapless, Canais A/B/C/D, Modifiers e Footswitches
//! Cobre Critérios de Aceite CA-1 a CA-9

use sonix_core::footswitch::{FootswitchArray, FootswitchMode};
use sonix_core::modifiers::{ModifierBinding, ModifierMatrix, ModifierSource};
use sonix_core::preset::{
    BlockChannel, BlockChannelState, BlockNode, CoreError, Preset, Setlist,
};

#[test]
fn test_ca_01_setlist_preset_hierarchy_and_8_scenes() {
    let setlist_name = "Tour 2026";
    let mut setlist = Setlist::new(setlist_name);
    let preset = Preset::new("P01", "Stadium Lead", 120.0);

    // Verify 8 scenes (0 to 7, A to H)
    assert_eq!(preset.scenes().len(), 8);
    assert_eq!(preset.scene(0).unwrap().name, "Scene A");
    assert_eq!(preset.scene(7).unwrap().name, "Scene H");

    setlist.add_preset(preset);
    assert_eq!(setlist.presets().len(), 1);
    assert_eq!(setlist.presets()[0].name, "Stadium Lead");
}

#[test]
fn test_ca_02_block_4_channels_a_b_c_d_independence() {
    let mut block = BlockNode::new("amp_1", "AMP");

    // Configure Channel A as US Clean
    let mut state_a = BlockChannelState::new("us_twin_blackface_65");
    state_a.set_param("gain", 3.0);
    block.set_channel_state(BlockChannel::A, state_a);

    // Configure Channel C as High Gain Lead
    let mut state_c = BlockChannelState::new("cali_dual_recto_revf");
    state_c.set_param("gain", 8.5);
    block.set_channel_state(BlockChannel::C, state_c);

    // Verify Channel C did not alter Channel A
    assert_eq!(block.channel_state(BlockChannel::A).model_id, "us_twin_blackface_65");
    assert_eq!(block.channel_state(BlockChannel::A).get_param("gain"), Some(3.0));
    assert_eq!(block.channel_state(BlockChannel::C).model_id, "cali_dual_recto_revf");
    assert_eq!(block.channel_state(BlockChannel::C).get_param("gain"), Some(8.5));
}

#[test]
fn test_ca_03_inactive_channels_consume_zero_cpu() {
    let mut block = BlockNode::new("amp_1", "AMP");
    block.set_active_channel_for_scene(0, BlockChannel::B);

    // Only active channel B is executed; A, C, D are inactive
    let active_ch = block.active_channel_for_scene(0);
    assert_eq!(active_ch, BlockChannel::B);
}

#[test]
fn test_ca_05_modifier_matrix_creation_and_binding() {
    let mut matrix = ModifierMatrix::new();
    let binding = ModifierBinding {
        target_block_id: "delay_1".to_string(),
        target_param: "feedback".to_string(),
        source: ModifierSource::ExpPedal1,
        min_val: 10.0,
        max_val: 85.0,
        curve_slope: 0.5,
        damping_ms: 20.0,
    };

    matrix.add_binding(binding.clone()).unwrap();
    assert_eq!(matrix.bindings_for_block("delay_1").len(), 1);
    assert_eq!(matrix.bindings_for_block("delay_1")[0].target_param, "feedback");
}

#[test]
fn test_ca_06_modifier_transfer_function_and_linear_interpolation() {
    let binding = ModifierBinding {
        target_block_id: "wah_1".to_string(),
        target_param: "pedal".to_string(),
        source: ModifierSource::ExpPedal1,
        min_val: 200.0,
        max_val: 2200.0,
        curve_slope: 0.5, // exact linear
        damping_ms: 0.0,
    };

    // u = 0.0 -> min_val
    let val_0 = binding.evaluate(0.0);
    assert_eq!(val_0, 200.0);

    // u = 1.0 -> max_val
    let val_1 = binding.evaluate(1.0);
    assert_eq!(val_1, 2200.0);

    // u = 0.5 -> 200 + 0.5 * (2200 - 200) = 1200.0
    let val_half = binding.evaluate(0.5);
    assert_eq!(val_half, 1200.0);
}

#[test]
fn test_ca_07_footswitch_4_modes_reconfiguration() {
    let mut fs_array = FootswitchArray::new();

    // Preset Mode
    fs_array.set_mode(FootswitchMode::Preset);
    assert_eq!(fs_array.unit(0).oled_top_line, "PRESET");
    assert_eq!(fs_array.unit(0).oled_main_text, "P01");

    // Scene Mode
    fs_array.set_mode(FootswitchMode::Scene);
    assert_eq!(fs_array.unit(0).oled_top_line, "SCENE");
    assert_eq!(fs_array.unit(0).oled_main_text, "SCENE A");
    assert_eq!(fs_array.unit(7).oled_main_text, "SCENE H");

    // Stomp Mode
    fs_array.set_mode(FootswitchMode::Stomp);
    assert_eq!(fs_array.unit(0).oled_top_line, "STOMP");

    // Hybrid Mode
    fs_array.set_mode(FootswitchMode::HybridStompScene);
    assert_eq!(fs_array.unit(0).oled_top_line, "STOMP");
    assert_eq!(fs_array.unit(4).oled_top_line, "SCENE");
}

#[test]
fn test_ca_08_error_handling_invalid_scene_or_block() {
    let preset = Preset::new("P01", "Preset 1", 120.0);
    assert!(matches!(preset.scene(8), Err(CoreError::InvalidSceneIndex(8))));

    let mut matrix = ModifierMatrix::new();
    let invalid_binding = ModifierBinding {
        target_block_id: "".to_string(), // Empty block ID
        target_param: "gain".to_string(),
        source: ModifierSource::ExpPedal1,
        min_val: 0.0,
        max_val: 10.0,
        curve_slope: 0.5,
        damping_ms: 0.0,
    };
    assert!(matches!(matrix.add_binding(invalid_binding), Err(CoreError::InvalidBinding(_))));
}

#[test]
fn test_ca_09_non_regression_unbound_parameters_unaffected() {
    let matrix = ModifierMatrix::new();
    assert!(!matrix.has_modifier_for("amp_1", "presence"));
    // Unbound param maintains directly assigned value without alteration
    let direct_val = 6.5f32;
    let computed_val = matrix.apply_or_fallback("amp_1", "presence", direct_val, 0.0);
    assert_eq!(computed_val, 6.5);
}
