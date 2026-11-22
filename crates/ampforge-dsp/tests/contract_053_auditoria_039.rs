// AUD-39: Spec 053 CA-4 — Invariância ao tamanho de bloco (bit a bit idêntica vs 128)
use ampforge_dsp::amp::{AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::cab::VirtualCabProcessor;
use ampforge_dsp::fx::delay::{DelayModel, DelayProcessor};
use ampforge_dsp::fx::drive::{DriveModel, DriveProcessor};
use ampforge_dsp::fx::reverb::{ReverbModel, ReverbProcessor};

fn run_chain(heavy: bool, block_size: usize, input: &[f32]) -> Vec<f32> {
    let sr = 48_000.0;
    let mut drive = DriveProcessor::new(DriveModel::GreenScreamer808);
    drive.set_sample_rate(sr);
    drive.set_gain(5.5);

    let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
    amp.set_sample_rate(sr);
    amp.set_oversampling_factor(4);
    amp.panel_mut().set_input_gain(6.0);

    let mut cab = VirtualCabProcessor::new();
    cab.set_sample_rate(sr);

    let mut delay = DelayProcessor::new(DelayModel::PristineDigitalDelay);
    delay.set_sample_rate(sr);
    delay.set_delay_time_ms(250.0);
    delay.set_mix_pct(25.0);

    let mut reverb = ReverbProcessor::new(ReverbModel::StudioVintagePlate);
    reverb.set_sample_rate(sr);
    reverb.set_mix_pct(20.0);

    let mut out = vec![0.0_f32; input.len()];
    let mut tmp_a = vec![0.0_f32; block_size];
    let mut tmp_b = vec![0.0_f32; block_size];

    for (chunk_in, chunk_out) in input.chunks(block_size).zip(out.chunks_mut(block_size)) {
        let n = chunk_in.len();
        if heavy {
            drive.process(chunk_in, &mut tmp_a[..n]);
            amp.process(&tmp_a[..n], &mut tmp_b[..n]);
            cab.process(&tmp_b[..n], &mut tmp_a[..n]);
            delay.process(&tmp_a[..n], &mut tmp_b[..n]);
            reverb.process(&tmp_b[..n], chunk_out);
        } else {
            amp.process(chunk_in, chunk_out);
        }
    }
    out
}

#[test]
fn ca4_invariancia_ao_tamanho_de_bloco() {
    // AUD-39
    let sr = 48_000.0_f32;
    let total = 4800usize; // 100 ms
    let input: Vec<f32> = (0..total)
        .map(|i| 0.25 * (2.0 * std::f32::consts::PI * 220.0 * (i as f32) / sr).sin())
        .collect();

    for heavy in [false, true] {
        let ref_128 = run_chain(heavy, 128, &input);
        for blk in [32usize, 37, 64, 256, 480, 512, 1024] {
            let got = run_chain(heavy, blk, &input);
            assert_eq!(got.len(), ref_128.len());
            for i in 0..total {
                assert_eq!(
                    got[i].to_bits(),
                    ref_128[i].to_bits(),
                    "AUD-39 (heavy={heavy}): bloco {blk} divergiu de 128 no sample {i}: got={} ref={}",
                    got[i],
                    ref_128[i]
                );
            }
        }
    }
}
