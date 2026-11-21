//! crates/ampforge-dsp/tests/contract_002_matriz_dag.rs
//! Testes de Contrato Automatizados para Spec 002: Matriz de Roteamento de Sinal 4x12 (DAG)
//! Cobre Critérios de Aceite CA-1 a CA-10

use ampforge_dsp::dag::{
    BlockInstance, BlockType, DagCycleError, DagGraph, GridMatrix4x12, MixerChannelConfig,
    MixerNode, RowInput, RowOutput, SlotCell, SlotId, SplitMode, SplitNode,
};

#[test]
fn test_ca_01_grid_4x12_addressable_48_slots() {
    let mut grid = GridMatrix4x12::new();
    assert_eq!(grid.rows(), 4);
    assert_eq!(grid.cols(), 12);
    assert_eq!(grid.total_slots(), 48);

    // Initial state: all cells are empty
    for r in 0..4 {
        for c in 0..12 {
            let slot = SlotId::new(r, c).unwrap();
            assert!(matches!(grid.get_slot(slot), SlotCell::Empty));
        }
    }

    // Set block at (0, 0)
    let slot_0_0 = SlotId::new(0, 0).unwrap();
    grid.set_slot(slot_0_0, SlotCell::Block(BlockInstance::new("gate", BlockType::Gate)));
    assert!(matches!(grid.get_slot(slot_0_0), SlotCell::Block(_)));

    // Replace with SHUNT at (0, 0)
    grid.set_slot(slot_0_0, SlotCell::Shunt);
    assert!(matches!(grid.get_slot(slot_0_0), SlotCell::Shunt));

    // Clear / remove
    grid.set_slot(slot_0_0, SlotCell::Empty);
    assert!(matches!(grid.get_slot(slot_0_0), SlotCell::Empty));

    // Out of bounds validation
    assert!(SlotId::new(4, 0).is_err());
    assert!(SlotId::new(0, 12).is_err());
}

#[test]
fn test_ca_02_split_node_modes_instantiation() {
    let split_bal = SplitNode::new(SplitMode::BalanceAb { ratio: 0.5 });
    assert!(matches!(split_bal.mode(), SplitMode::BalanceAb { .. }));

    let split_xo = SplitNode::new(SplitMode::Crossover { freq_hz: 800.0 });
    assert!(matches!(split_xo.mode(), SplitMode::Crossover { freq_hz } if *freq_hz == 800.0));

    let split_dyn = SplitNode::new(SplitMode::DynamicSplit {
        threshold_db: -20.0,
        attack_ms: 10.0,
        release_ms: 50.0,
    });
    assert!(matches!(split_dyn.mode(), SplitMode::DynamicSplit { .. }));
}

#[test]
fn test_ca_03_split_crossover_flat_sum_reconstruction() {
    let mut split = SplitNode::new(SplitMode::Crossover { freq_hz: 1000.0 });
    let sample_rate = 48000.0;
    split.set_sample_rate(sample_rate);

    let num_samples = 4096;
    let mut in_buffer = vec![0.0f32; num_samples];
    // Generate test signal with combined low (200Hz) and high (5000Hz) frequencies
    for i in 0..num_samples {
        let t = i as f32 / sample_rate;
        in_buffer[i] = 0.5 * (2.0 * std::f32::consts::PI * 200.0 * t).sin()
                     + 0.5 * (2.0 * std::f32::consts::PI * 5000.0 * t).sin();
    }

    let mut out_low = vec![0.0f32; num_samples];
    let mut out_high = vec![0.0f32; num_samples];

    split.process(&in_buffer, &mut out_low, &mut out_high);
    split.process(&in_buffer, &mut out_low, &mut out_high);

    // LR4 soma em filtro passa-tudo (magnitude plana 0 dB +- 0.1 dB; fase invertida em fc)
    let sum: Vec<f32> = out_low.iter().zip(out_high.iter()).map(|(l, h)| l + h).collect();
    let rms_in = (in_buffer[2048..].iter().map(|s| s * s).sum::<f32>() / 2048.0).sqrt();
    let rms_sum = (sum[2048..].iter().map(|s| s * s).sum::<f32>() / 2048.0).sqrt();
    let db_diff = 20.0 * (rms_sum / rms_in.max(1e-9)).log10();
    assert!(
        db_diff.abs() <= 0.1,
        "LR4 reconstruction RMS error: db_diff={}",
        db_diff
    );
}

#[test]
fn test_ca_04_split_dynamic_envelope_routing() {
    let mut split = SplitNode::new(SplitMode::DynamicSplit {
        threshold_db: -20.0,
        attack_ms: 1.0,
        release_ms: 10.0,
    });
    split.set_sample_rate(48000.0);

    // Soft signal (-40 dBFS, below -20 dBFS threshold) -> goes to Branch A
    let soft_len = 512;
    let soft_amp = 0.01; // ~ -40 dBFS
    let soft_in = vec![soft_amp; soft_len];
    let mut out_a = vec![0.0f32; soft_len];
    let mut out_b = vec![0.0f32; soft_len];

    split.process(&soft_in, &mut out_a, &mut out_b);
    let energy_a_soft: f32 = out_a[100..].iter().map(|s| s * s).sum();
    let energy_b_soft: f32 = out_b[100..].iter().map(|s| s * s).sum();
    assert!(energy_a_soft > energy_b_soft * 10.0, "Soft signal should prioritize branch A");

    // Hard transient signal (0 dBFS, above -20 dBFS threshold) -> goes to Branch B
    let hard_amp = 1.0;
    let hard_in = vec![hard_amp; soft_len];
    split.process(&hard_in, &mut out_a, &mut out_b);
    let energy_a_hard: f32 = out_a[200..].iter().map(|s| s * s).sum();
    let energy_b_hard: f32 = out_b[200..].iter().map(|s| s * s).sum();
    assert!(energy_b_hard > energy_a_hard, "Hard signal should route to branch B");
}

#[test]
fn test_ca_05_mixer_node_controls_gain_pan_phase_master() {
    let mut mixer = MixerNode::new();
    mixer.configure_channel(0, MixerChannelConfig {
        level_db: -6.0,
        pan: -0.5,
        phase_invert: false,
    });
    mixer.configure_channel(1, MixerChannelConfig {
        level_db: 3.0,
        pan: 0.5,
        phase_invert: true,
    });
    mixer.set_master_level_db(-1.0);

    assert_eq!(mixer.channel_config(0).level_db, -6.0);
    assert_eq!(mixer.channel_config(0).pan, -0.5);
    assert!(!mixer.channel_config(0).phase_invert);
    assert_eq!(mixer.channel_config(1).level_db, 3.0);
    assert!(mixer.channel_config(1).phase_invert);
    assert_eq!(mixer.master_level_db(), -1.0);
}

#[test]
fn test_ca_06_mixer_complete_phase_cancellation_under_minus_90dbfs() {
    let mut mixer = MixerNode::new();
    // Ch 0: 0 dB, Pan 0.0, Phase 0 deg
    mixer.configure_channel(0, MixerChannelConfig {
        level_db: 0.0,
        pan: 0.0,
        phase_invert: false,
    });
    // Ch 1: 0 dB, Pan 0.0, Phase 180 deg (inverted)
    mixer.configure_channel(1, MixerChannelConfig {
        level_db: 0.0,
        pan: 0.0,
        phase_invert: true,
    });
    mixer.set_master_level_db(0.0);

    let n = 256;
    let in_ch0 = vec![0.707f32; n];
    let in_ch1 = vec![0.707f32; n];
    let inputs = vec![in_ch0.as_slice(), in_ch1.as_slice()];

    let mut out_l = vec![0.0f32; n];
    let mut out_r = vec![0.0f32; n];

    mixer.process(&inputs, &mut out_l, &mut out_r);

    for i in 0..n {
        let peak = out_l[i].abs().max(out_r[i].abs());
        let peak_db = if peak > 1e-9 { 20.0 * peak.log10() } else { -120.0 };
        assert!(
            peak_db < -90.0,
            "Expected cancellation < -90 dBFS, got peak_db={}", peak_db
        );
    }
}

#[test]
fn test_ca_07_independent_io_routing_per_row() {
    let mut graph = DagGraph::new();
    graph.set_row_input(0, RowInput::InstIn1);
    graph.set_row_input(1, RowInput::AuxIn2);
    graph.set_row_input(2, RowInput::FxReturnLr);
    graph.set_row_input(3, RowInput::UsbReamp34);

    graph.set_row_output(0, RowOutput::MainXlrLr);
    graph.set_row_output(1, RowOutput::AmpOutP10NoCab);
    graph.set_row_output(2, RowOutput::FxSend);
    graph.set_row_output(3, RowOutput::UsbRecOut);

    assert_eq!(graph.row_input(0), RowInput::InstIn1);
    assert_eq!(graph.row_input(1), RowInput::AuxIn2);
    assert_eq!(graph.row_input(2), RowInput::FxReturnLr);
    assert_eq!(graph.row_input(3), RowInput::UsbReamp34);

    assert_eq!(graph.row_output(0), RowOutput::MainXlrLr);
    assert_eq!(graph.row_output(1), RowOutput::AmpOutP10NoCab);
    assert_eq!(graph.row_output(2), RowOutput::FxSend);
    assert_eq!(graph.row_output(3), RowOutput::UsbRecOut);
}

#[test]
fn test_ca_08_amp_out_p10_no_cab_bypasses_cab_convolution() {
    let mut graph = DagGraph::new();
    graph.set_row_output(1, RowOutput::AmpOutP10NoCab);

    // Place an amp and a cab on row 1
    let amp_slot = SlotId::new(1, 4).unwrap();
    let cab_slot = SlotId::new(1, 5).unwrap();
    graph.set_slot(amp_slot, SlotCell::Block(BlockInstance::new("plexi", BlockType::Amp)));
    graph.set_slot(cab_slot, SlotCell::Block(BlockInstance::new("cab_4x12", BlockType::Cab)));

    // Verify tap for AmpOutP10NoCab flags CAB block as bypassed for this row's output
    assert!(graph.is_cab_bypassed_for_output(1));
}

#[test]
fn test_ca_09_dag_cycle_prevention_rejects_feedback_loop() {
    let mut graph = DagGraph::new();
    let slot_a = SlotId::new(0, 5).unwrap();
    let slot_b = SlotId::new(1, 7).unwrap();
    let slot_c = SlotId::new(0, 3).unwrap(); // Backward: col 3 <= col 7

    // Forward edge (0,5) -> (1,7) is valid
    assert!(graph.add_connection(slot_a, slot_b).is_ok());

    // Backward edge (1,7) -> (0,3) creates a feedback loop (col 3 <= col 7)
    let err = graph.add_connection(slot_b, slot_c);
    assert!(matches!(err, Err(DagCycleError::FeedbackCycleDetected { .. })));

    // Self loop (1,7) -> (1,7) is also rejected
    let self_err = graph.add_connection(slot_b, slot_b);
    assert!(matches!(self_err, Err(DagCycleError::FeedbackCycleDetected { .. })));
}

#[test]
fn test_ca_10_non_regression_parallel_line_state_isolation() {
    let mut graph = DagGraph::new();
    let slot_row0 = SlotId::new(0, 2).unwrap();
    let slot_row1 = SlotId::new(1, 2).unwrap();

    let block0 = BlockInstance::new("delay0", BlockType::Delay);
    let block1 = BlockInstance::new("delay1", BlockType::Delay);

    graph.set_slot(slot_row0, SlotCell::Block(block0));
    graph.set_slot(slot_row1, SlotCell::Block(block1));

    // Verify initial states
    assert!(!graph.is_slot_bypassed(slot_row0));
    assert!(!graph.is_slot_bypassed(slot_row1));

    // Toggle bypass on row 0
    graph.toggle_bypass(slot_row0);

    // Row 0 should be bypassed, Row 1 must remain completely unaffected
    assert!(graph.is_slot_bypassed(slot_row0));
    assert!(!graph.is_slot_bypassed(slot_row1));

    // Change parameter on row 0 block
    graph.set_parameter(slot_row0, "mix", 0.75).unwrap();
    assert_eq!(graph.get_parameter(slot_row0, "mix").unwrap(), 0.75);
    // Row 1 block parameter remains unchanged
    assert_ne!(graph.get_parameter(slot_row1, "mix"), Some(0.75));
}
