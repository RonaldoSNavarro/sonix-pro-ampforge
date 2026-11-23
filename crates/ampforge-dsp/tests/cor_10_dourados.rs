//! COR-10 — dourados reais: saída atual de cada processador comparada com a
//! saída gravada em `9ab9bb8` (commit anterior ao COR-07) pelo exemplo
//! `gerar_dourados_cor10`.
//!
//! Critério: `max|y − y_dourado| ≤ 1e-6 · max|y_dourado|`.
//!
//! O gerador de sinal e os parâmetros abaixo são cópia literal dos do exemplo
//! `crates/ampforge-dsp/examples/gerar_dourados_cor10.rs`.

use std::path::PathBuf;

use ampforge_dsp::amp::{AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::cab::VirtualCabProcessor;
use ampforge_dsp::fx::{DriveModel, DriveProcessor};

const COR10_BLOCK: usize = 64;
const COR10_DRIVE_GAIN: f32 = 7.5;
const COR10_TOL_REL: f32 = 1e-6;

fn cor10_signal(sr: u32) -> Vec<f32> {
    let peak = 10f64.powf(-12.0 / 20.0);
    let n_sine = (2 * sr) as usize;
    let n_noise = (sr / 2) as usize;
    let mut v = Vec::with_capacity(n_sine + n_noise);
    for i in 0..n_sine {
        let t = i as f64 / sr as f64;
        let s = 0.5 * (2.0 * std::f64::consts::PI * 220.0 * t).sin()
            + 0.5 * (2.0 * std::f64::consts::PI * 880.0 * t).sin();
        v.push((peak * s) as f32);
    }
    let mut state: u32 = 0x1234_5678;
    for _ in 0..n_noise {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let u = (state as f64 / u32::MAX as f64) * 2.0 - 1.0;
        v.push((peak * u) as f32);
    }
    v
}

fn run_blocks(input: &[f32], mut f: impl FnMut(&[f32], &mut [f32])) -> Vec<f32> {
    let mut out = vec![0.0f32; input.len()];
    for (i, o) in input.chunks(COR10_BLOCK).zip(out.chunks_mut(COR10_BLOCK)) {
        f(i, o);
    }
    out
}

fn render_drive(model: DriveModel, sr: u32) -> Vec<f32> {
    let mut p = DriveProcessor::new(model);
    p.set_sample_rate(sr as f32);
    p.set_gain(COR10_DRIVE_GAIN);
    run_blocks(&cor10_signal(sr), |i, o| p.process(i, o))
}

fn render_amp(model: AmpModelId, sr: u32) -> Vec<f32> {
    let mut p = WhiteBoxAmpProcessor::new(model);
    p.set_sample_rate(sr as f32);
    run_blocks(&cor10_signal(sr), |i, o| p.process(i, o))
}

fn render_cab(sr: u32) -> Vec<f32> {
    let mut p = VirtualCabProcessor::new();
    p.set_sample_rate(sr as f32);
    run_blocks(&cor10_signal(sr), |i, o| p.process(i, o))
}

fn load_fixture(name: &str) -> Vec<f32> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/cor-10")
        .join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("fixture {} ausente: {e}", path.display()));
    assert_eq!(bytes.len() % 4, 0, "{name}: tamanho não múltiplo de 4");
    bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect()
}

fn assert_golden(name: &str, y: &[f32]) {
    let gold = load_fixture(name);
    assert_eq!(y.len(), gold.len(), "{name}: comprimento divergente");
    let peak = gold.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    assert!(peak > 0.0, "{name}: dourado silencioso");
    let mut max_err = 0.0f32;
    let mut at = 0usize;
    for (i, (a, b)) in y.iter().zip(gold.iter()).enumerate() {
        let e = (a - b).abs();
        if !(e <= max_err) {
            max_err = e;
            at = i;
        }
    }
    let lim = COR10_TOL_REL * peak;
    assert!(
        max_err <= lim,
        "{name}: max|y-y_dourado|={max_err:e} na amostra {at} > 1e-6*max|y_dourado|={lim:e} (pico={peak})"
    );
}

macro_rules! golden_drive {
    ($test:ident, $model:expr, $sr:expr, $file:expr) => {
        #[test]
        fn $test() {
            assert_golden($file, &render_drive($model, $sr));
        }
    };
}

macro_rules! golden_amp {
    ($test:ident, $model:expr, $sr:expr, $file:expr) => {
        #[test]
        fn $test() {
            assert_golden($file, &render_amp($model, $sr));
        }
    };
}

golden_drive!(cor_10_drive_ts808_48k_igual_ao_dourado, DriveModel::GreenScreamer808, 48_000, "drive_ts808_48000.f32");
golden_drive!(cor_10_drive_klon_48k_igual_ao_dourado, DriveModel::GoldenKloneCentaur, 48_000, "drive_klon_48000.f32");
golden_drive!(cor_10_drive_rat_48k_igual_ao_dourado, DriveModel::BlackRodentRat, 48_000, "drive_rat_48000.f32");
golden_drive!(cor_10_drive_ds1_48k_igual_ao_dourado, DriveModel::OrangeBossDs, 48_000, "drive_ds1_48000.f32");
golden_drive!(cor_10_drive_fuzzface_48k_igual_ao_dourado, DriveModel::FuzzFaceSiGe, 48_000, "drive_fuzzface_48000.f32");
golden_drive!(cor_10_drive_muff_48k_igual_ao_dourado, DriveModel::BigMuffPi, 48_000, "drive_muff_48000.f32");
golden_drive!(cor_10_drive_ts808_96k_igual_ao_dourado, DriveModel::GreenScreamer808, 96_000, "drive_ts808_96000.f32");
golden_drive!(cor_10_drive_klon_96k_igual_ao_dourado, DriveModel::GoldenKloneCentaur, 96_000, "drive_klon_96000.f32");
golden_drive!(cor_10_drive_rat_96k_igual_ao_dourado, DriveModel::BlackRodentRat, 96_000, "drive_rat_96000.f32");
golden_drive!(cor_10_drive_ds1_96k_igual_ao_dourado, DriveModel::OrangeBossDs, 96_000, "drive_ds1_96000.f32");
golden_drive!(cor_10_drive_fuzzface_96k_igual_ao_dourado, DriveModel::FuzzFaceSiGe, 96_000, "drive_fuzzface_96000.f32");
golden_drive!(cor_10_drive_muff_96k_igual_ao_dourado, DriveModel::BigMuffPi, 96_000, "drive_muff_96000.f32");

golden_amp!(cor_10_amp_jcm800_48k_igual_ao_dourado, AmpModelId::BritJcm800_2203, 48_000, "amp_jcm800_48000.f32");
golden_amp!(cor_10_amp_plexi_48k_igual_ao_dourado, AmpModelId::BritPlexi1959, 48_000, "amp_plexi_48000.f32");
golden_amp!(cor_10_amp_twin_48k_igual_ao_dourado, AmpModelId::UsTwinBlackface65, 48_000, "amp_twin_48000.f32");
golden_amp!(cor_10_amp_recto_48k_igual_ao_dourado, AmpModelId::CaliDualRectoRevF, 48_000, "amp_recto_48000.f32");
golden_amp!(cor_10_amp_ac30_48k_igual_ao_dourado, AmpModelId::UkAc30TopBoost, 48_000, "amp_ac30_48000.f32");
golden_amp!(cor_10_amp_jcm800_96k_igual_ao_dourado, AmpModelId::BritJcm800_2203, 96_000, "amp_jcm800_96000.f32");
golden_amp!(cor_10_amp_plexi_96k_igual_ao_dourado, AmpModelId::BritPlexi1959, 96_000, "amp_plexi_96000.f32");
golden_amp!(cor_10_amp_twin_96k_igual_ao_dourado, AmpModelId::UsTwinBlackface65, 96_000, "amp_twin_96000.f32");
golden_amp!(cor_10_amp_recto_96k_igual_ao_dourado, AmpModelId::CaliDualRectoRevF, 96_000, "amp_recto_96000.f32");
golden_amp!(cor_10_amp_ac30_96k_igual_ao_dourado, AmpModelId::UkAc30TopBoost, 96_000, "amp_ac30_96000.f32");

#[test]
fn cor_10_cab_sem_ir_48k_igual_ao_dourado() {
    assert_golden("cab_sem_ir_48000.f32", &render_cab(48_000));
}

#[test]
fn cor_10_cab_sem_ir_96k_igual_ao_dourado() {
    assert_golden("cab_sem_ir_96000.f32", &render_cab(96_000));
}
