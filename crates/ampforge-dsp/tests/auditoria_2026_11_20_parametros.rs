//! Testes de auditoria 2026-11-20 — Parâmetros e controles fiéis por modelo (AUD-08)

use std::f32::consts::PI;
use std::path::Path;

use ampforge_dsp::amp::{AmpFrontPanel, AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::regua::ganho_db;

fn gerar_seno(sr: f32, freq_hz: f32, amp: f32, duracao_s: f32) -> Vec<f32> {
    let n = (sr * duracao_s).round() as usize;
    (0..n)
        .map(|i| amp * (2.0 * PI * freq_hz * (i as f32) / sr).sin())
        .collect()
}

fn medir_ganho_pequeno_sinal_db(model: AmpModelId) -> f64 {
    let sr = 48_000.0;
    let input = gerar_seno(sr, 1000.0, 10.0f32.powf(-80.0 / 20.0), 1.5);
    let mut output = vec![0.0f32; input.len()];
    let mut amp = WhiteBoxAmpProcessor::new(model);
    amp.set_sample_rate(sr);
    *amp.panel_mut() = AmpFrontPanel::default();
    amp.process(&input, &mut output);
    ganho_db(&input, &output, sr, 1000.0)
}

fn comparar_extremos(
    model: AmpModelId,
    label: &str,
    freq_hz: f32,
    amp_in: f32,
    mut cfg0: impl FnMut(&mut WhiteBoxAmpProcessor),
    mut cfg10: impl FnMut(&mut WhiteBoxAmpProcessor),
) -> f64 {
    let sr = 48_000.0;
    let input = gerar_seno(sr, freq_hz, amp_in, 1.5);
    let mut out0 = vec![0.0f32; input.len()];
    let mut out10 = vec![0.0f32; input.len()];

    let mut amp0 = WhiteBoxAmpProcessor::new(model);
    amp0.set_sample_rate(sr);
    cfg0(&mut amp0);
    amp0.process(&input, &mut out0);

    let mut amp10 = WhiteBoxAmpProcessor::new(model);
    amp10.set_sample_rate(sr);
    cfg10(&mut amp10);
    amp10.process(&input, &mut out10);

    let tail0 = &out0[24_000..];
    let tail10 = &out10[24_000..];
    let mut sum_sq_ref = 0.0f64;
    let mut sum_sq_diff = 0.0f64;
    for (&a, &b) in tail0.iter().zip(tail10.iter()) {
        let af = a as f64;
        let df = (b - a) as f64;
        sum_sq_ref += af * af;
        sum_sq_diff += df * df;
    }
    let rms_ref = (sum_sq_ref / tail0.len() as f64).sqrt();
    let rms_diff = (sum_sq_diff / tail0.len() as f64).sqrt();
    let diff_db = if rms_diff == 0.0 {
        -240.0
    } else {
        20.0 * (rms_diff / rms_ref.max(1e-12)).max(1e-12).log10()
    };
    let g0 = ganho_db(&input, &out0, sr, freq_hz);
    let g10 = ganho_db(&input, &out10, sr, freq_hz);
    println!(
        "[AUD-08-EXTREMO] {:?} {} @ {} Hz: g0={:.2} dB, g10={:.2} dB, diff_rel={:.2} dB",
        model, label, freq_hz, g0, g10, diff_db
    );
    diff_db
}

#[test]
fn aud_08_medicao_ganho_pequeno_sinal() {
    let models = [
        AmpModelId::BritPlexi1959,
        AmpModelId::BritJcm800_2203,
        AmpModelId::UsTwinBlackface65,
        AmpModelId::CaliDualRectoRevF,
        AmpModelId::UkAc30TopBoost,
    ];
    for model in models {
        let g_db = medir_ganho_pequeno_sinal_db(model);
        println!("[AUD-08-GANHO] {}: {:.4} dB", model.as_str(), g_db);
        assert!(
            g_db.is_finite(),
            "ganho pequeno sinal de {} deve ser finito (foi {})",
            model.as_str(),
            g_db
        );
    }
}

#[test]
fn aud_08_controle_inexistente_nao_age() {
    assert_eq!(AmpModelId::ALL.len(), 5);
    assert!(AmpModelId::BritPlexi1959.has_presence());
    assert!(!AmpModelId::BritPlexi1959.has_master());
    assert!(AmpModelId::BritPlexi1959.has_middle());
    assert!(AmpModelId::BritJcm800_2203.has_presence());
    assert!(AmpModelId::BritJcm800_2203.has_master());
    assert!(!AmpModelId::UsTwinBlackface65.has_presence());
    assert!(!AmpModelId::UsTwinBlackface65.has_master());
    assert!(AmpModelId::UsTwinBlackface65.has_middle());
    assert!(AmpModelId::CaliDualRectoRevF.has_presence());
    assert!(AmpModelId::CaliDualRectoRevF.has_master());
    assert!(!AmpModelId::UkAc30TopBoost.has_presence());
    assert!(!AmpModelId::UkAc30TopBoost.has_master());
    assert!(!AmpModelId::UkAc30TopBoost.has_middle());

    let checks: Vec<(&str, f64)> = vec![
        (
            "BritPlexi1959 master (-12 dBFS @ 1 kHz)",
            comparar_extremos(
                AmpModelId::BritPlexi1959,
                "master_volume",
                1000.0,
                10.0f32.powf(-12.0 / 20.0),
                |a| a.panel_mut().set_master_volume(0.0),
                |a| a.panel_mut().set_master_volume(10.0),
            ),
        ),
        (
            "UsTwinBlackface65 presence (-12 dBFS @ 4 kHz)",
            comparar_extremos(
                AmpModelId::UsTwinBlackface65,
                "presence",
                4000.0,
                10.0f32.powf(-12.0 / 20.0),
                |a| a.panel_mut().set_presence(0.0),
                |a| a.panel_mut().set_presence(10.0),
            ),
        ),
        (
            "UsTwinBlackface65 master (-12 dBFS @ 1 kHz)",
            comparar_extremos(
                AmpModelId::UsTwinBlackface65,
                "master_volume",
                1000.0,
                10.0f32.powf(-12.0 / 20.0),
                |a| a.panel_mut().set_master_volume(0.0),
                |a| a.panel_mut().set_master_volume(10.0),
            ),
        ),
        (
            "UkAc30TopBoost presence (-12 dBFS @ 4 kHz)",
            comparar_extremos(
                AmpModelId::UkAc30TopBoost,
                "presence",
                4000.0,
                10.0f32.powf(-12.0 / 20.0),
                |a| a.panel_mut().set_presence(0.0),
                |a| a.panel_mut().set_presence(10.0),
            ),
        ),
        (
            "UkAc30TopBoost master (-12 dBFS @ 1 kHz)",
            comparar_extremos(
                AmpModelId::UkAc30TopBoost,
                "master_volume",
                1000.0,
                10.0f32.powf(-12.0 / 20.0),
                |a| a.panel_mut().set_master_volume(0.0),
                |a| a.panel_mut().set_master_volume(10.0),
            ),
        ),
        (
            "UkAc30TopBoost middle (-12 dBFS @ 600 Hz)",
            comparar_extremos(
                AmpModelId::UkAc30TopBoost,
                "middle",
                600.0,
                10.0f32.powf(-12.0 / 20.0),
                |a| a.panel_mut().set_middle(0.0),
                |a| a.panel_mut().set_middle(10.0),
            ),
        ),
    ];

    let mut falhas = Vec::new();
    for (nome, diff_db) in checks {
        if diff_db > -120.0 {
            falhas.push(format!("{}: diferenca {:.2} dB > -120 dB", nome, diff_db));
        }
    }

    // Verifica que amp_1.overdrive / amp_1.depth foram removidos do código de produção
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let forbidden: &[(&str, &[&str])] = &[
        (
            "crates/ampforge-dsp/src/amp.rs",
            &[
                "pub overdrive",
                "pub depth",
                "set_overdrive",
                "set_depth",
                "depth_filter",
                "od_gain_lin",
            ],
        ),
        (
            "crates/sonix-desktop/src/audio.rs",
            &["AmpOverdrive", "AmpDepth", "set_amp_overdrive", "set_amp_depth"],
        ),
        (
            "crates/sonix-desktop/src/api.rs",
            &["AmpOverdrive", "AmpDepth", "amp_overdrive", "amp_depth"],
        ),
        (
            "crates/sonix-plugin/src/lib.rs",
            &["amp_1.depth", "amp_depth", "set_amp_depth"],
        ),
    ];

    for (rel_path, needles) in forbidden {
        let content = std::fs::read_to_string(root.join(rel_path))
            .unwrap_or_else(|e| panic!("falha lendo {}: {}", rel_path, e));
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            for needle in *needles {
                if line.contains(needle) {
                    falhas.push(format!(
                        "{}:{} ainda referencia '{}'",
                        rel_path,
                        idx + 1,
                        needle
                    ));
                }
            }
        }
    }

    assert!(
        falhas.is_empty(),
        "AUD-08 falhou:\n{}",
        falhas.join("\n")
    );
}
