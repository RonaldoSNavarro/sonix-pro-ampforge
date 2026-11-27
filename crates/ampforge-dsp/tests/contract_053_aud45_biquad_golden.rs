// AUD-45: Teste dourado bit a bit para unificação do BiquadFilter em ampforge_dsp::biquad
use ampforge_dsp::amp::{AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::cab::{SpeakerCabModel, VirtualCabProcessor};
use ampforge_dsp::dag::{SplitMode, SplitNode};
use ampforge_dsp::fx::delay::{DelayModel, DelayProcessor};
use ampforge_dsp::fx::drive::{DriveModel, DriveProcessor};
use ampforge_dsp::fx::dynamics::{NoiseGateMode, NoiseGateProcessor};
use ampforge_dsp::fx::modulation::{ModModel, ModulationProcessor};

fn fnv1a_bits(samples: &[f32]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &s in samples {
        for b in s.to_bits().to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
    }
    h
}

#[test]
fn aud_45_biquad_unificado_bit_a_bit_identico() {
    let sr = 48_000.0_f32;
    let n = 1024usize;
    let inp: Vec<f32> = (0..n)
        .map(|i| 0.25 * (2.0 * std::f32::consts::PI * 220.0 * (i as f32) / sr).sin())
        .collect();

    // 1. amp
    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp.set_sample_rate(sr);
    amp.set_oversampling_factor(4);
    let mut out_amp = vec![0.0_f32; n];
    amp.process(&inp, &mut out_amp);

    // 2. cab
    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);
    cab.set_speaker_model(SpeakerCabModel::V304x12Closed);
    let mut out_cab = vec![0.0_f32; n];
    cab.process(&inp, &mut out_cab);

    // 3. dag (SplitNode Crossover)
    let mut split = SplitNode::new(SplitMode::Crossover { freq_hz: 600.0 });
    split.set_sample_rate(sr);
    let mut out_dag_a = vec![0.0_f32; n];
    let mut out_dag_b = vec![0.0_f32; n];
    split.process(&inp, &mut out_dag_a, &mut out_dag_b);

    // 4. fx/delay
    let mut delay = DelayProcessor::new(DelayModel::AnalogBbdDelay);
    delay.set_sample_rate(sr);
    delay.set_delay_time_ms(120.0);
    delay.set_mix_pct(50.0);
    let mut out_delay = vec![0.0_f32; n];
    delay.process(&inp, &mut out_delay);

    // 5. fx/drive
    let mut drive = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive.set_sample_rate(sr);
    drive.set_gain(6.0);
    let mut out_drive = vec![0.0_f32; n];
    drive.process(&inp, &mut out_drive);

    // 6. fx/dynamics
    let mut gate = NoiseGateProcessor::new(NoiseGateMode::IntelligentNaturalGate);
    gate.set_sample_rate(sr);
    let mut out_dyn = vec![0.0_f32; n];
    gate.process(&inp, None, &mut out_dyn);

    // 7. fx/modulation
    let mut modu = ModulationProcessor::new(ModModel::RotarySpeaker);
    modu.set_sample_rate(sr);
    let mut out_mod = vec![0.0_f32; n];
    modu.process(&inp, &mut out_mod);

    let hashes = (
        fnv1a_bits(&out_amp),
        fnv1a_bits(&out_cab),
        fnv1a_bits(&out_dag_a) ^ fnv1a_bits(&out_dag_b),
        fnv1a_bits(&out_delay),
        fnv1a_bits(&out_drive),
        fnv1a_bits(&out_dyn),
        fnv1a_bits(&out_mod),
    );
    assert_eq!(
        hashes,
        (
            0x6fa1e028c5e66052, // HEAD AmpCore baseline preservado bit a bit
            718727116713510475, // Baseline Spec 080 (VirtualCabProcessor: low_cut, SM57, proximidade)
            0xb0c27ae8c5aa09c5,
            0xd356337bf70c0fdb,
            0xb6200fb55eb81ffe,
            0xbb6143011ee61534,
            0xc8e64e315b795ef4,
        ),
        "AUD-45: saída após unificação do BiquadFilter deve permanecer bit a bit idêntica"
    );
}
