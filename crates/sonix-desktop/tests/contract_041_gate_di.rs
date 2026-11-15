use sonix_desktop::audio::{DspParamUpdate, RealtimeDspChain};
use ampforge_dsp::fx::dynamics::NoiseGateMode;
use std::f32::consts::PI;

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    (sum_sq / samples.len() as f64).sqrt() as f32
}

#[test]
fn test_ca_04_realtime_chain_natural_gate_uses_clean_di_sidechain() {
    let mut chain = RealtimeDspChain::new(48000.0);
    // DI at -43 dBFS RMS (3 dB below -40 dB threshold), boosted +12 dB before block input (-31 dBFS > -40 dB)
    let di_amp = 10.0f32.powf(-43.0 / 20.0) * std::f32::consts::SQRT_2;
    chain.apply_param_update(DspParamUpdate::AmpEnabled(false));
    chain.apply_param_update(DspParamUpdate::CabEnabled(false));
    chain.apply_param_update(DspParamUpdate::DriveEnabled(false));
    chain.apply_param_update(DspParamUpdate::Eq1Enabled(false));
    chain.apply_param_update(DspParamUpdate::Comp2Enabled(false));
    chain.apply_param_update(DspParamUpdate::Mod2Enabled(false));
    chain.apply_param_update(DspParamUpdate::Dly1Enabled(false));
    chain.apply_param_update(DspParamUpdate::Rev1Enabled(false));
    chain.apply_param_update(DspParamUpdate::GateEnabled(true));
    chain.apply_param_update(DspParamUpdate::GateThreshold(-40.0));
    chain.apply_param_update(DspParamUpdate::GateRange(-60.0));
    chain.apply_param_update(DspParamUpdate::GateHold(0.0));
    chain.apply_param_update(DspParamUpdate::GateHysteresis(0.0));
    chain.apply_param_update(DspParamUpdate::InputBoost(12.0));

    let block = 256;
    let mut inp = vec![0.0f32; block];
    let mut out_l = vec![0.0f32; block];

    // 1. IntelligentNaturalGate: DI (-43 dB) is below -40 dB -> gate stays CLOSED even though boosted input is -31 dB
    chain.apply_param_update(DspParamUpdate::GateMode(NoiseGateMode::IntelligentNaturalGate));
    for b in 0..40 {
        for i in 0..block {
            let idx = (b * block + i) as f32;
            inp[i] = di_amp * (2.0 * PI * 440.0 * idx / 48000.0).sin();
        }
        chain.process(&inp, &mut out_l);
    }
    let natural_rms = rms(&out_l);

    // 2. ClassicExpanderGate: reads boosted block input (-31 dB > -40 dB) -> gate OPENS
    chain.apply_param_update(DspParamUpdate::GateMode(NoiseGateMode::ClassicExpanderGate));
    for b in 40..80 {
        for i in 0..block {
            let idx = (b * block + i) as f32;
            inp[i] = di_amp * (2.0 * PI * 440.0 * idx / 48000.0).sin();
        }
        chain.process(&inp, &mut out_l);
    }
    let classic_rms = rms(&out_l);

    assert!(
        classic_rms > natural_rms * 10.0,
        "Natural gate must stay closed on -43 dB DI while Classic opens on boosted -31 dB input (natural={:.6}, classic={:.6})",
        natural_rms,
        classic_rms
    );
}
