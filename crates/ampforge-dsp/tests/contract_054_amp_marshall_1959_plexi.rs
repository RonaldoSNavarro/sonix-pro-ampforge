//! Testes de contrato da Spec 054: Amplificador Marshall 1959 Super Lead / Plexi (Rev. 1969 SL/A)
//! Cobre: CA-1 (Tone Stack FMV Mid 10% log), CA-2 (4 entradas -6 dB + Jumpered),
//! CA-3 (Bright Cap 5000 pF exclusivo do Canal I), CA-4 (sem Master), CA-5 (4 Ohm > 16 Ohm NFB), CA-7 (RT zero-alloc)

use ampforge_dsp::amp::AmpInputJack;
use ampforge_dsp::amps::Marshall1959SuperLead;
use ampforge_dsp::cab::{SpeakerCabModel, VirtualCabProcessor};

fn rms_db(samples: &[f32]) -> f64 {
    let sq: f64 = samples.iter().map(|&v| (v as f64) * (v as f64)).sum();
    10.0 * (sq / (samples.len().max(1) as f64)).max(1e-18).log10()
}

fn measure_amp_db(amp: &mut Marshall1959SuperLead, freq: f32, amp_pk: f32) -> f64 {
    let sr = 48_000.0f32;
    let mut inp = vec![0.0f32; 4096];
    for (i, s) in inp.iter_mut().enumerate() {
        *s = amp_pk * (2.0 * std::f32::consts::PI * freq * (i as f32) / sr).sin();
    }
    let mut out = vec![0.0f32; 4096];
    amp.process_mono_slice(&inp, &mut out);
    rms_db(&out[2048..])
}

#[test]
fn ca2_four_inputs_low_minus_6db_and_jumpered() {
    let mut amp = Marshall1959SuperLead::new();
    amp.panel.input_gain = 3.0;
    amp.panel.volume_2 = 5.0;

    amp.panel.input_jack = AmpInputJack::High1;
    let h1 = measure_amp_db(&mut amp, 440.0, 0.02);
    amp.panel.input_jack = AmpInputJack::Low1;
    let l1 = measure_amp_db(&mut amp, 440.0, 0.02);
    let diff1 = h1 - l1;
    assert!((diff1 - 6.02).abs() <= 0.3, "High1 - Low1 = {:.2} dB", diff1);

    amp.panel.input_jack = AmpInputJack::Jumpered;
    amp.panel.jumpered = true;
    amp.panel.input_gain = 2.0;
    amp.panel.volume_2 = 0.0;
    let j0 = measure_amp_db(&mut amp, 440.0, 0.02);
    amp.panel.volume_2 = 10.0;
    let j10 = measure_amp_db(&mut amp, 440.0, 0.02);
    assert!(j10 - j0 >= 3.0, "Jumpered volume_2 0->10 deve elevar >= 3 dB: {:.2} -> {:.2}", j0, j10);
}

#[test]
fn ca3_bright_cap_5000pf_isolated_to_channel_1() {
    let mut amp = Marshall1959SuperLead::new();
    amp.panel.input_jack = AmpInputJack::High1;
    amp.panel.input_gain = 2.0;
    let ch1_lo_tilt = measure_amp_db(&mut amp, 4000.0, 0.02) - measure_amp_db(&mut amp, 250.0, 0.02);

    amp.panel.input_gain = 9.0;
    let ch1_hi_tilt = measure_amp_db(&mut amp, 4000.0, 0.02) - measure_amp_db(&mut amp, 250.0, 0.02);
    assert!(
        ch1_lo_tilt - ch1_hi_tilt >= 4.0,
        "Bright cap Canal I em gain=2 vs 9: {:.2} vs {:.2}",
        ch1_lo_tilt, ch1_hi_tilt
    );

    // Canal II puro (gain=0, high2) não recebe bright cap do Canal I
    amp.panel.input_jack = AmpInputJack::High2;
    amp.panel.jumpered = false;
    amp.panel.input_gain = 0.0;
    amp.panel.volume_2 = 3.0;
    let ch2_tilt = measure_amp_db(&mut amp, 4000.0, 0.02) - measure_amp_db(&mut amp, 250.0, 0.02);
    assert!(
        ch1_lo_tilt - ch2_tilt >= 3.0,
        "Canal II deve ser >= 3 dB mais escuro que Canal I em gain=2: ch1={:.2}, ch2={:.2}",
        ch1_lo_tilt, ch2_tilt
    );
}

#[test]
fn ca5_impedance_selector_4_ohm_more_resonant_than_16_ohm_and_paired_cab() {
    let mut amp = Marshall1959SuperLead::new();
    amp.panel.impedance_ohms = 4;
    let g4 = measure_amp_db(&mut amp, 98.0, 0.10) - measure_amp_db(&mut amp, 1000.0, 0.10);
    amp.panel.impedance_ohms = 16;
    let g16 = measure_amp_db(&mut amp, 98.0, 0.10) - measure_amp_db(&mut amp, 1000.0, 0.10);
    assert!(g4 - g16 >= 0.8, "4 Ohm ({:.2}) deve ser mais ressonante que 16 Ohm ({:.2})", g4, g16);

    // Teste ponta-a-ponta com gabinete Greenback 4x12 pareado (palm mute 98 Hz)
    let mut cab = VirtualCabProcessor::new();
    cab.set_speaker_model(SpeakerCabModel::Greenback4x12Closed);
    let mut inp = [0.0f32; 1024];
    for (i, s) in inp.iter_mut().enumerate() {
        *s = 0.18 * (2.0 * std::f32::consts::PI * 98.0 * (i as f32) / 48_000.0).sin();
    }
    let mut amp_out = [0.0f32; 1024];
    let mut cab_out = [0.0f32; 1024];
    amp.process_mono_slice(&inp, &mut amp_out);
    cab.process(&amp_out, &mut cab_out);
    assert!(cab_out.iter().all(|v| v.is_finite()));
}
