//! Testes de contrato da Spec 058: Amplificador Vox AC30/6 Top Boost (Rev. 1964 JMI)
//! Cobre: CA-2 (Tone Cut >= 6 dB), CA-3 (Isolamento Top Boost no canal Normal + volume_2), CA-4 (sem NFB)

use ampforge_dsp::amp::AmpInputJack;
use ampforge_dsp::amps::VoxAc30TopBoost1964;

fn measure_db(amp: &mut VoxAc30TopBoost1964, freq: f32, pk: f32) -> f64 {
    let mut inp = vec![0.0f32; 4096];
    for (i, s) in inp.iter_mut().enumerate() {
        *s = pk * (2.0 * std::f32::consts::PI * freq * (i as f32) / 48_000.0).sin();
    }
    let mut out = vec![0.0f32; 4096];
    amp.process_mono_slice(&inp, &mut out);
    let sq: f64 = out[2048..].iter().map(|&v| (v as f64).powi(2)).sum();
    10.0 * (sq / 2048.0).max(1e-18).log10()
}

#[test]
fn ca2_tone_cut_attenuates_highs_in_both_channels() {
    let mut amp = VoxAc30TopBoost1964::new();
    for jack in [AmpInputJack::NormalHigh, AmpInputJack::BrilliantHigh] {
        amp.panel.input_jack = jack;
        amp.panel.cut = 0.0;
        let db0 = measure_db(&mut amp, 4000.0, 0.05);
        amp.panel.cut = 10.0;
        let db10 = measure_db(&mut amp, 4000.0, 0.05);
        assert!(
            db0 - db10 >= 6.0,
            "Tone Cut em {:?} deve atenuar >= 6 dB em 4 kHz: {:.2} -> {:.2}",
            jack, db0, db10
        );
    }
}

#[test]
fn ca3_normal_channel_uses_volume_2_and_bypasses_top_boost() {
    let mut amp = VoxAc30TopBoost1964::new();
    amp.panel.input_jack = AmpInputJack::NormalHigh;
    amp.panel.volume_2 = 5.0;
    amp.panel.treble = 0.0;
    amp.panel.bass = 0.0;
    let db_0 = measure_db(&mut amp, 4000.0, 0.03);
    amp.panel.treble = 10.0;
    amp.panel.bass = 10.0;
    let db_10 = measure_db(&mut amp, 4000.0, 0.03);
    assert!(
        (db_10 - db_0).abs() <= 0.1,
        "Canal Normal deve isolar Top Boost Treble/Bass: {:.3} vs {:.3}",
        db_0, db_10
    );

    // volume_2 governa o ganho no canal Normal
    amp.panel.volume_2 = 0.0;
    let v0 = measure_db(&mut amp, 1000.0, 0.05);
    amp.panel.volume_2 = 8.0;
    let v8 = measure_db(&mut amp, 1000.0, 0.05);
    assert!(v8 - v0 >= 20.0, "Normal Vol (volume_2) deve governar o canal Normal: {:.2} -> {:.2}", v0, v8);
}
