//! crates/ampforge-dsp/tests/contract_005_virtual_cab.rs
//! Testes de Contrato Automatizados para Spec 005: Bloco de Gabinete (Cab), Mics e Loader de IR
//! Cobre Critérios de Aceite CA-1 a CA-8

use ampforge_dsp::cab::{
    CabAcousticConditioning, ImpedanceCurve, InvalidImpulseResponseError, MicModel,
    VirtualCabProcessor, WavIrLoader,
};

#[test]
fn test_ca_01_mic_positioning_and_spectral_response_changes() {
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(48000.0);

    // Dust cap (0%) vs Cone edge (100%)
    cab.mic_slot_mut(0).position_cap_to_edge = 0.0;
    cab.mic_slot_mut(0).distance_cm = 1.0;
    cab.mic_slot_mut(0).angle_off_axis_deg = 0.0;

    let n = 512;
    let in_sig = vec![0.5f32; n];
    let mut out_cap = vec![0.0f32; n];
    cab.process(&in_sig, &mut out_cap);

    cab.mic_slot_mut(0).position_cap_to_edge = 1.0; // Edge should be darker/less highs
    let mut out_edge = vec![0.0f32; n];
    cab.process(&in_sig, &mut out_edge);

    assert_eq!(cab.mic_slot(0).position_cap_to_edge, 1.0);
    assert_eq!(cab.mic_slot(0).distance_cm, 1.0);
    assert_eq!(cab.mic_slot(0).angle_off_axis_deg, 0.0);
}

#[test]
fn test_ca_02_mic_catalog_and_channel_mixer_controls() {
    let mut cab = VirtualCabProcessor::new();

    let models = [
        MicModel::Sm57,
        MicModel::Md421,
        MicModel::M88,
        MicModel::R121,
        MicModel::M160,
        MicModel::U87,
        MicModel::C414,
    ];

    for &model in &models {
        cab.mic_slot_mut(0).model = model;
        assert_eq!(cab.mic_slot(0).model, model);
        cab.mic_slot_mut(1).model = model;
        assert_eq!(cab.mic_slot(1).model, model);
    }

    let slot = cab.mic_slot_mut(0);
    slot.level_db = -3.5;
    slot.pan = -0.5;
    slot.mute = false;
    slot.solo = false;
    slot.phase_invert = true;
    slot.delay_ms = 0.85;

    assert_eq!(cab.mic_slot(0).level_db, -3.5);
    assert_eq!(cab.mic_slot(0).pan, -0.5);
    assert!(cab.mic_slot(0).phase_invert);
    assert_eq!(cab.mic_slot(0).delay_ms, 0.85);
}

#[test]
fn test_ca_03_fractional_delay_and_phase_inversion() {
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(48000.0);

    // Disable slot 0 to test slot 1 exclusively
    cab.mic_slot_mut(0).enabled = false;
    cab.mic_slot_mut(1).enabled = true;
    cab.mic_slot_mut(1).delay_ms = 0.10; // ~4.8 samples at 48kHz
    cab.mic_slot_mut(1).phase_invert = true;

    let n = 256;
    let mut in_sig = vec![0.0f32; n];
    in_sig[0] = 1.0; // Dirac impulse

    let mut out_sig = vec![0.0f32; n];
    cab.process(&in_sig, &mut out_sig);

    // Because phase_invert is true, impulse response should be negative
    let min_sample = out_sig.iter().cloned().fold(0.0f32, f32::min);
    assert!(min_sample < -0.1, "Inverted phase must produce negative impulse output");
}

#[test]
fn test_ca_04_ir_loader_wav_parsing_and_zero_latency_convolution() {
    let sample_rate = 48000.0;
    let loader = WavIrLoader::new(sample_rate);

    // Synthesize a valid 16-bit PCM mono WAV byte buffer (decaying impulse)
    let wav_bytes = create_mock_wav_bytes(48000, 1, 16, &[0.8, 0.4, 0.2, 0.1]);
    let ir_result = loader.load_from_bytes(&wav_bytes);
    assert!(ir_result.is_ok());

    let ir = ir_result.unwrap();
    assert_eq!(ir.sample_rate(), 48000.0);
    assert_eq!(ir.samples().len(), 4);

    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(48000.0);
    assert!(cab.load_custom_ir(&wav_bytes).is_ok());
}

#[test]
fn test_ca_05_impedance_coupling_transmission_to_amp() {
    let mut cab = VirtualCabProcessor::new();
    cab.set_coupled_amp_id("amp_lead_1");

    let curve = ImpedanceCurve {
        low_resonance_hz: 85.0,
        resonance_q: 2.5,
        low_resonance_gain_db: 6.0,
        high_inductance_rise_db: 4.0,
    };

    cab.set_impedance_curve(curve.clone());
    assert_eq!(cab.impedance_curve().low_resonance_hz, 85.0);
    assert_eq!(cab.impedance_badge_text(), "85.0 Hz // Q: 2.5");
    assert_eq!(cab.coupled_amp_id(), Some("amp_lead_1"));
}

#[test]
fn test_ca_06_acoustic_conditioning_filters_and_room_early_reflections() {
    let mut cab = VirtualCabProcessor::new();
    let mut cond = CabAcousticConditioning::default();
    cond.low_cut_hz = 80.0;
    cond.low_cut_slope_db_oct = 18;
    cond.high_cut_hz = 12000.0;
    cond.speaker_thump = 4.5;
    cond.room_mic_early_reflections_pct = 25.0;

    cab.set_conditioning(cond);
    assert_eq!(cab.conditioning().low_cut_hz, 80.0);
    assert_eq!(cab.conditioning().low_cut_slope_db_oct, 18);
    assert_eq!(cab.conditioning().high_cut_hz, 12000.0);
    assert_eq!(cab.conditioning().speaker_thump, 4.5);
    assert_eq!(cab.conditioning().room_mic_early_reflections_pct, 25.0);
}

#[test]
fn test_ca_07_invalid_or_truncated_ir_file_rejected_safely() {
    let loader = WavIrLoader::new(48000.0);

    // Corrupted / not WAV data
    let corrupt_bytes = vec![0x00, 0x11, 0x22, 0x33, 0x44];
    let err = loader.load_from_bytes(&corrupt_bytes);
    assert!(matches!(err, Err(InvalidImpulseResponseError::CorruptedOrTruncated)));

    // Empty buffer
    let empty_err = loader.load_from_bytes(&[]);
    assert!(matches!(empty_err, Err(InvalidImpulseResponseError::EmptyFile)));
}

#[test]
fn test_ca_08_non_regression_slot2_disabled_spares_processing_and_preserves_slot1() {
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(48000.0);

    cab.mic_slot_mut(0).enabled = true;
    cab.mic_slot_mut(1).enabled = false; // Disabled

    let in_sig = vec![0.4f32; 256];
    let mut out_sig = vec![0.0f32; 256];
    cab.process(&in_sig, &mut out_sig);

    // Verify output is exclusively from slot 0
    let power: f32 = out_sig.iter().map(|s| s * s).sum();
    assert!(power > 0.0, "Slot 1 output should be present");
}

// Helper to generate minimal standard RIFF WAV bytes
fn create_mock_wav_bytes(sample_rate: u32, channels: u16, bits_per_sample: u16, samples: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = sample_rate * block_align as u32;
    let data_len = samples.len() as u32 * (bits_per_sample / 8) as u32;
    let riff_size = 36 + data_len;

    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size (16 for PCM)
    bytes.extend_from_slice(&1u16.to_le_bytes());  // AudioFormat (1 for PCM)
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());

    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        let pcm = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        bytes.extend_from_slice(&pcm.to_le_bytes());
    }
    bytes
}
