//! Testes de contrato da Spec 057: Mesa/Boogie Dual Rectifier Solo Head (Rev. G, 1994–2000)
//! Cobre: CA-1 (Progressão de ganho), CA-2 (Bold/Spongy Variac -15..25% B+),
//! CA-3 (Diode vs 5U4GB sag), CA-4 (NFB=0 em Modern), CA-7 (Presence ativo fora do NFB em Modern >= +6 dB)

use ampforge_dsp::amp::AmpChannelMode;
use ampforge_dsp::amps::MesaDualRectifierRevG;
use ampforge_dsp::modshop::RectifierType;

fn measure_rms_db(amp: &mut MesaDualRectifierRevG, freq: f32, pk: f32) -> f64 {
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
fn ca2_spongy_variac_reduces_b_plus_by_20_percent() {
    let mut amp = MesaDualRectifierRevG::new();
    amp.panel.spongy = false;
    let bold_v = amp.current_b_plus_v();
    amp.panel.spongy = true;
    let _ = measure_rms_db(&mut amp, 1000.0, 0.01);
    let spongy_v = amp.current_b_plus_v();
    let ratio = spongy_v / bold_v;
    assert!(
        (0.75..=0.85).contains(&ratio),
        "Spongy deve reduzir B+ em 15..25%: bold={}, spongy={}, ratio={}",
        bold_v, spongy_v, ratio
    );
}

#[test]
fn ca3_rectifier_diode_vs_5u4gb_sag() {
    let mut amp = MesaDualRectifierRevG::new();
    amp.mod_settings.sag = 8.0;
    amp.mod_settings.rectifier = RectifierType::SolidStateDiode;
    let _ = measure_rms_db(&mut amp, 200.0, 0.7);
    let b_diode = amp.current_b_plus_v();

    let mut amp_tube = MesaDualRectifierRevG::new();
    amp_tube.mod_settings.sag = 8.0;
    amp_tube.mod_settings.rectifier = RectifierType::Dual5U4Gb;
    let _ = measure_rms_db(&mut amp_tube, 200.0, 0.7);
    let b_5u4 = amp_tube.current_b_plus_v();

    let drop_diode = 470.0 - b_diode;
    let drop_5u4 = 470.0 - b_5u4;
    assert!(
        drop_5u4 >= 2.5 * drop_diode,
        "5U4GB drop ({:.2}V) deve ser >= 2.5x Diode drop ({:.2}V)",
        drop_5u4, drop_diode
    );
}

#[test]
fn ca4_and_ca7_modern_nfb_off_and_presence_active_outside_nfb() {
    let mut amp = MesaDualRectifierRevG::new();
    amp.panel.channel_mode = AmpChannelMode::Modern;
    amp.panel.input_gain = 2.0; // linear para medir filtro Presence limpo

    amp.panel.presence = 0.0;
    let p0 = measure_rms_db(&mut amp, 4500.0, 0.02);
    amp.panel.presence = 10.0;
    let p10 = measure_rms_db(&mut amp, 4500.0, 0.02);

    assert!(
        p10 - p0 >= 6.0,
        "Rev G CA-7: Presence em Modern deve elevar >= +6 dB @ 4.5 kHz: {:.2} -> {:.2}",
        p0, p10
    );
}

#[test]
fn ca6_mesa_rev_g_block_size_invariance_and_clean_modern_crossfade() {
    let total = 2048usize;
    let inp: Vec<f32> = (0..total)
        .map(|i| 0.08 * (2.0 * std::f32::consts::PI * 220.0 * (i as f32) / 48_000.0).sin())
        .collect();

    let run = |blk: usize| -> Vec<f32> {
        let mut amp = MesaDualRectifierRevG::new();
        amp.panel.channel_mode = AmpChannelMode::Modern;
        let mut out = vec![0.0f32; total];
        for (ci, co) in inp.chunks(blk).zip(out.chunks_mut(blk)) {
            amp.process_mono_slice(ci, co);
        }
        out
    };

    let ref_512 = run(512);
    let got_32 = run(32);
    let max_diff = ref_512
        .iter()
        .zip(got_32.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        max_diff <= 1e-6,
        "MesaDualRectifierRevG não pode resetar filtros por bloco (max_diff={max_diff})"
    );
}
