//! COR-10 — gerador dos fixtures dourados.
//!
//! Executado num worktree de `9ab9bb8` (commit anterior ao COR-07), que é a
//! referência de fidelidade. Grava em `tests/fixtures/cor-10/` (raiz do
//! workspace) a saída de cada processador, `f32` little-endian, um arquivo por
//! processador e taxa.
//!
//! Sinal fixo: 2 s de seno 220 Hz + 880 Hz com pico de −12 dBFS, seguidos de
//! 0,5 s de ruído branco uniforme de semente fixa com pico de −12 dBFS.
//! Processamento em blocos de 64 amostras.
//!
//! Uso: `cargo run -p ampforge-dsp --example gerar_dourados_cor10`
//!
//! O gerador de sinal e a tabela de processadores são duplicados, sem
//! alteração, em `crates/ampforge-dsp/tests/cor_10_dourados.rs`.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use ampforge_dsp::amp::{AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::cab::VirtualCabProcessor;
use ampforge_dsp::fx::{DriveModel, DriveProcessor};

pub const COR10_BLOCK: usize = 64;
pub const COR10_RATES: [u32; 2] = [48_000, 96_000];
pub const COR10_DRIVE_GAIN: f32 = 7.5;

pub fn cor10_signal(sr: u32) -> Vec<f32> {
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

pub fn cor10_drives() -> [(&'static str, DriveModel); 6] {
    [
        ("ts808", DriveModel::GreenScreamer808),
        ("klon", DriveModel::GoldenKloneCentaur),
        ("rat", DriveModel::BlackRodentRat),
        ("ds1", DriveModel::OrangeBossDs),
        ("fuzzface", DriveModel::FuzzFaceSiGe),
        ("muff", DriveModel::BigMuffPi),
    ]
}

pub fn cor10_amps() -> [(&'static str, AmpModelId); 5] {
    [
        ("jcm800", AmpModelId::BritJcm800_2203),
        ("plexi", AmpModelId::BritPlexi1959),
        ("twin", AmpModelId::UsTwinBlackface65),
        ("recto", AmpModelId::CaliDualRectoRevF),
        ("ac30", AmpModelId::UkAc30TopBoost),
    ]
}

fn run_blocks(input: &[f32], mut f: impl FnMut(&[f32], &mut [f32])) -> Vec<f32> {
    let mut out = vec![0.0f32; input.len()];
    for (i, o) in input.chunks(COR10_BLOCK).zip(out.chunks_mut(COR10_BLOCK)) {
        f(i, o);
    }
    out
}

pub fn cor10_render_drive(model: DriveModel, sr: u32) -> Vec<f32> {
    let mut p = DriveProcessor::new(model);
    p.set_sample_rate(sr as f32);
    p.set_gain(COR10_DRIVE_GAIN);
    run_blocks(&cor10_signal(sr), |i, o| p.process(i, o))
}

pub fn cor10_render_amp(model: AmpModelId, sr: u32) -> Vec<f32> {
    let mut p = WhiteBoxAmpProcessor::new(model);
    p.set_sample_rate(sr as f32);
    run_blocks(&cor10_signal(sr), |i, o| p.process(i, o))
}

pub fn cor10_render_cab(sr: u32) -> Vec<f32> {
    let mut p = VirtualCabProcessor::new();
    p.set_sample_rate(sr as f32);
    run_blocks(&cor10_signal(sr), |i, o| p.process(i, o))
}

fn write_f32(dir: &PathBuf, name: &str, data: &[f32]) {
    let path = dir.join(name);
    let mut f = fs::File::create(&path).expect("criar fixture");
    let mut bytes = Vec::with_capacity(data.len() * 4);
    for x in data {
        assert!(x.is_finite(), "{name}: amostra não finita");
        bytes.extend_from_slice(&x.to_le_bytes());
    }
    f.write_all(&bytes).expect("gravar fixture");
    let peak = data.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    println!("{} amostras={} pico={:.6}", path.display(), data.len(), peak);
}

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/cor-10");
    fs::create_dir_all(&dir).expect("criar diretório de fixtures");
    for sr in COR10_RATES {
        for (name, model) in cor10_drives() {
            write_f32(&dir, &format!("drive_{name}_{sr}.f32"), &cor10_render_drive(model, sr));
        }
        for (name, model) in cor10_amps() {
            write_f32(&dir, &format!("amp_{name}_{sr}.f32"), &cor10_render_amp(model, sr));
        }
        write_f32(&dir, &format!("cab_sem_ir_{sr}.f32"), &cor10_render_cab(sr));
    }
}
