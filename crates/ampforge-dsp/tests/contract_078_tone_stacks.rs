//! Testes de contrato da Spec 078: Solvers Analíticos de Redes Passivas de Tone Stack
//! Cobre:
//! CA-01: Aderência à Equação de Laplace de 3ª Ordem (Yeh DAFx-06) <= 0.35 dB
//! CA-02: Passividade Estrita |H(f)| <= 1.001 nos 27 vértices {0, 5, 10}^3
//! CA-03: Interatividade Autêntica do VoxTopBoostToneStackSolver (OS/010)
//! CA-04: Impacto Mensurável da Impedância da Fonte R_src (1.3 kOhm vs 38 kOhm)
//! CA-05: Tapers Reais Sem Zona Morta em 5.0 e Segurança RT

use ampforge_dsp::modshop::ToneStackTopology;
use ampforge_dsp::tonestack::{
    analog_tonestack_magnitude_db_for_components, audio25_taper, jcm800_fmv_components,
    linear_taper, log10_taper, spec078_fmv_components, FmvToneStackSolver,
    JamesBaxandallToneStackSolver, MesaRectifierToneStackSolver, VoxTopBoostToneStackSolver,
};

fn measure_fmv_solver_gain_db(solver: &mut FmvToneStackSolver, topo: ToneStackTopology, b: f32, m: f32, t: f32, freq_hz: f64) -> f64 {
    let sr = 48_000.0;
    solver.reset();
    solver.update_components(spec078_fmv_components(topo, b, m, t), sr, true);
    let total = 4096;
    let steady = 2048;
    let mut sum_sq_in = 0.0;
    let mut sum_sq_out = 0.0;
    for i in 0..total {
        let x = 0.2512 * (2.0 * std::f64::consts::PI * freq_hz * (i as f64) / sr).sin();
        let y = solver.process(x);
        if i >= steady {
            sum_sq_in += x * x;
            sum_sq_out += y * y;
        }
    }
    10.0 * (sum_sq_out / sum_sq_in.max(1e-24)).max(1e-18).log10()
}

fn measure_vox_solver_gain_db(solver: &mut VoxTopBoostToneStackSolver, b: f32, t: f32, freq_hz: f64) -> f64 {
    let sr = 48_000.0;
    solver.reset();
    solver.update(b, t, sr, true);
    let total = 4096;
    let steady = 2048;
    let mut sum_sq_in = 0.0;
    let mut sum_sq_out = 0.0;
    for i in 0..total {
        let x = 0.2512 * (2.0 * std::f64::consts::PI * freq_hz * (i as f64) / sr).sin();
        let y = solver.process(x);
        if i >= steady {
            sum_sq_in += x * x;
            sum_sq_out += y * y;
        }
    }
    10.0 * (sum_sq_out / sum_sq_in.max(1e-24)).max(1e-18).log10()
}

#[test]
fn ca01_laplace_adherence_fmv_fender_and_marshall() {
    let mut ts = FmvToneStackSolver::new();
    let freqs = [50.0, 100.0, 250.0, 450.0, 700.0, 1000.0, 2000.0, 4000.0, 8000.0, 10000.0];

    for topo in [ToneStackTopology::FenderTmbMidScoop, ToneStackTopology::MarshallCathodeFollower] {
        for &f in &freqs {
            let dsp_db = measure_fmv_solver_gain_db(&mut ts, topo, 5.0, 5.0, 5.0, f);
            let comps = spec078_fmv_components(topo, 5.0, 5.0, 5.0);
            let ana_db = analog_tonestack_magnitude_db_for_components(comps, f);
            let err = (dsp_db - ana_db).abs();
            assert!(
                err <= 0.35,
                "topo {:?} f={}Hz: dsp={:.3}dB vs ana={:.3}dB (err={:.3}dB)",
                topo, f, dsp_db, ana_db, err
            );
        }
    }
}

#[test]
fn ca02_strict_passivity_all_4_solvers_across_27_vertices() {
    let mut fmv = FmvToneStackSolver::new();
    let mut vox = VoxTopBoostToneStackSolver::new();
    let mut james = JamesBaxandallToneStackSolver::new();
    let mut mesa = MesaRectifierToneStackSolver::new();

    for &b in &[0.0f32, 5.0, 10.0] {
        for &m in &[0.0f32, 5.0, 10.0] {
            for &t in &[0.0f32, 5.0, 10.0] {
                let comps = jcm800_fmv_components(b, m, t);
                fmv.reset();
                fmv.update_components(comps, 48_000.0, true);
                vox.reset();
                vox.update(b, t, 48_000.0, true);
                james.reset();
                james.update(b, t, 48_000.0, true);
                mesa.reset();
                mesa.update(b, m, t, 48_000.0, true);

                // Checar ganho em múltiplas frequências via impulso unitário / energia
                for &f in &[80.0, 400.0, 1000.0, 4000.0, 10000.0] {
                    let mut sq_in = 0.0;
                    let mut sq_fmv = 0.0;
                    let mut sq_vox = 0.0;
                    let mut sq_james = 0.0;
                    let mut sq_mesa = 0.0;
                    for i in 0..2048 {
                        let x = (2.0 * std::f64::consts::PI * f * (i as f64) / 48_000.0).sin();
                        let yf = fmv.process(x);
                        let yv = vox.process(x);
                        let yj = james.process(x);
                        let ym = mesa.process(x);
                        if i >= 1024 {
                            sq_in += x * x;
                            sq_fmv += yf * yf;
                            sq_vox += yv * yv;
                            sq_james += yj * yj;
                            sq_mesa += ym * ym;
                        }
                    }
                    for (name, sq) in [("fmv", sq_fmv), ("vox", sq_vox), ("james", sq_james), ("mesa", sq_mesa)] {
                        let gain = (sq / sq_in).sqrt();
                        assert!(
                            gain <= 1.001,
                            "{} violou passividade em ({},{},{}) f={}: gain={}",
                            name, b, m, t, f, gain
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn ca03_vox_top_boost_interactive_cut_and_mid_prominence() {
    let mut ts = VoxTopBoostToneStackSolver::new();
    let g100_hi = measure_vox_solver_gain_db(&mut ts, 9.0, 9.0, 100.0);
    let g700_hi = measure_vox_solver_gain_db(&mut ts, 9.0, 9.0, 700.0);
    let g4k_hi = measure_vox_solver_gain_db(&mut ts, 9.0, 9.0, 4000.0);
    let avg_ends_hi = 0.5 * (g100_hi + g4k_hi);
    assert!(
        avg_ends_hi - g700_hi >= 8.0,
        "Em (9,9) médios em 700Hz devem afundar >= 8 dB: avg_ends={:.2}, 700Hz={:.2}",
        avg_ends_hi, g700_hi
    );

    let g700_lo = measure_vox_solver_gain_db(&mut ts, 1.0, 1.0, 700.0);
    let g4k_lo = measure_vox_solver_gain_db(&mut ts, 1.0, 1.0, 4000.0);
    assert!(
        g700_lo - g700_hi >= 4.0 && g700_lo > g4k_lo,
        "Em (1,1) médios em 700Hz devem sobressair vs (9,9) e agudos: 700(1,1)={:.2}, 700(9,9)={:.2}, 4k(1,1)={:.2}",
        g700_lo, g700_hi, g4k_lo
    );
}

#[test]
fn ca04_source_impedance_r_src_measurable_impact() {
    let mut c_low = spec078_fmv_components(ToneStackTopology::FenderTmbMidScoop, 5.0, 5.0, 5.0);
    c_low.r_src = 1_300.0;
    let mut c_high = c_low;
    c_high.r_src = 38_000.0;

    let db_low = analog_tonestack_magnitude_db_for_components(c_low, 1000.0);
    let db_high = analog_tonestack_magnitude_db_for_components(c_high, 1000.0);
    assert!(
        db_low - db_high >= 2.5,
        "Elevar R_src 1.3k -> 38k deve aumentar perda de inserção >= 2.5 dB em 1kHz: {:.2} vs {:.2}",
        db_low, db_high
    );
}

#[test]
fn ca05_tapers_without_dead_zone_and_rt_stability() {
    assert!((linear_taper(0.5) - 0.5).abs() < 1e-6);
    assert!((audio25_taper(0.5) - 0.25).abs() < 1e-6);
    assert!(log10_taper(0.5) > 0.08 && log10_taper(0.5) < 0.11);

    let mut ts = FmvToneStackSolver::new();
    for i in 0..10_000 {
        let knob = ((i % 100) as f32) / 10.0;
        let comps = spec078_fmv_components(ToneStackTopology::MarshallCathodeFollower, knob, 10.0 - knob, knob);
        ts.update_components(comps, 48_000.0, false);
        let y = ts.process(0.25 * ((i as f64) * 0.1).sin());
        assert!(y.is_finite() && y.abs() <= 2.0);
    }
}
