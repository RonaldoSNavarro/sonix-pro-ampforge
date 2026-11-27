//! Solvers analíticos de redes passivas de Tone Stack (Spec 078).
//!
//! 4 solvers físicos separados em `f64` com zero alocação em tempo real:
//! 1. `FmvToneStackSolver` — Yeh & Smith (DAFx-06) com impedância de fonte `r_src`
//! 2. `VoxTopBoostToneStackSolver` — Esquema JMI `OS/010` sem potenciômetro Mid
//! 3. `JamesBaxandallToneStackSolver` — Ponte passiva de 2 bandas shelving independentes
//! 4. `MesaRectifierToneStackSolver` — Dual Rectifier Rev G (`RF-1G`, cathode follower)

use crate::modshop::ToneStackTopology;
use std::f64::consts::PI;

const R_REF: f64 = 100_000.0;
const C_REF: f64 = 10e-9; // tau_ref = R_REF * C_REF = 1e-3 s

/// Curva linear (potenciômetro tipo B).
#[inline]
pub fn linear_taper(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

/// Curva logarítmica 10% em meio curso (ex.: Marshall 1959HW Bass e Middle Tonal Note 2).
#[inline]
pub fn log10_taper(x: f64) -> f64 {
    let u = x.clamp(0.0, 1.0);
    (0.01 * 100.0f64.powf(u) - 0.01) / 0.99
}

/// Curva áudio 25% em meio curso (típica de pots CTS/Alpha Fender, Mesa e Vox).
#[inline]
pub fn audio25_taper(x: f64) -> f64 {
    let u = x.clamp(0.0, 1.0);
    (9.0f64.powf(u) - 1.0) / 8.0
}

#[derive(Debug, Clone, Copy)]
pub struct FmvComponents {
    pub r_src: f64,
    pub r_treble: f64,
    pub r_bass: f64,
    pub r_mid: f64,
    pub r_slope: f64,
    pub r_load: f64,
    pub c1: f64,
    pub c2: f64,
    pub c3: f64,
    pub t_pos: f64,
    pub b_pos: f64,
    pub m_pos: f64,
}

// Mesa/Boogie Dual Rectifier Solo Head Rev G (1994–2000, PCB RF-1G — Spec 057 / 078)
const RECTO_REV_G_R_SRC: f64 = 1_300.0;
const RECTO_REV_G_R_TREBLE: f64 = 250_000.0;
const RECTO_REV_G_R_BASS: f64 = 250_000.0;
const RECTO_REV_G_R_MID: f64 = 25_000.0;
const RECTO_REV_G_R_SLOPE: f64 = 39_000.0;
const RECTO_REV_G_R_LOAD: f64 = 1_000_000.0;
const RECTO_REV_G_C1: f64 = 500e-12;
const RECTO_REV_G_C2: f64 = 22e-9;
const RECTO_REV_G_C3: f64 = 22e-9;

#[inline]
fn audio_taper(x: f64) -> f64 {
    let u = x.clamp(0.0, 1.0);
    (0.01 * 100.0f64.powf(u) - 0.01) / 0.99
}

const DECISAO_SONIX_RECTO_R_TREBLE: f64 = 250_000.0;
const DECISAO_SONIX_RECTO_R_BASS: f64 = 250_000.0;
const DECISAO_SONIX_RECTO_R_MID: f64 = 25_000.0;
const DECISAO_SONIX_RECTO_R_SLOPE: f64 = 39_000.0;
const DECISAO_SONIX_RECTO_R_LOAD: f64 = 1_000_000.0;
const DECISAO_SONIX_RECTO_C1: f64 = 470e-12;
const DECISAO_SONIX_RECTO_C2: f64 = 22e-9;
const DECISAO_SONIX_RECTO_C3: f64 = 22e-9;

/// Componentes legados (`r_src = 0`) para compatibilidade exata com `AmpCore` e golden fixtures (`COR-10` / `AUD-45`).
pub fn resolved_fmv_components(
    topo: ToneStackTopology,
    bass: f32,
    middle: f32,
    treble: f32,
) -> FmvComponents {
    let b = (bass as f64 / 10.0).clamp(0.0, 1.0);
    let m = (middle as f64 / 10.0).clamp(0.0, 1.0);
    let t = (treble as f64 / 10.0).clamp(0.0, 1.0);

    match topo {
        ToneStackTopology::MarshallCathodeFollower | ToneStackTopology::Model => FmvComponents {
            r_src: 0.0,
            r_treble: 220_000.0,
            r_bass: 1_000_000.0,
            r_mid: 22_000.0,
            r_slope: 33_000.0,
            r_load: 1_000_000.0,
            c1: 470e-12,
            c2: 22e-9,
            c3: 22e-9,
            t_pos: 0.25 * t + 0.75 * audio_taper(t),
            b_pos: audio_taper(b),
            m_pos: audio_taper(m),
        },
        ToneStackTopology::FenderTmbMidScoop => FmvComponents {
            r_src: 0.0,
            r_treble: 250_000.0,
            r_bass: 250_000.0,
            r_mid: 10_000.0,
            r_slope: 100_000.0,
            r_load: 1_000_000.0,
            c1: 250e-12,
            c2: 100e-9,
            c3: 47e-9,
            t_pos: audio_taper(t),
            b_pos: audio_taper(b),
            m_pos: m,
        },
        ToneStackTopology::VoxTopBoost => FmvComponents {
            r_src: 0.0,
            r_treble: 1_000_000.0,
            r_bass: 1_000_000.0,
            r_mid: 10_000.0,
            r_slope: 100_000.0,
            r_load: 1_000_000.0,
            c1: 50e-12,
            c2: 22e-9,
            c3: 22e-9,
            t_pos: audio_taper(t),
            b_pos: audio_taper(b),
            m_pos: 1.0,
        },
        ToneStackTopology::RectoPostGain | ToneStackTopology::MesaMarkPreGain => FmvComponents {
            r_src: 0.0,
            r_treble: DECISAO_SONIX_RECTO_R_TREBLE,
            r_bass: DECISAO_SONIX_RECTO_R_BASS,
            r_mid: DECISAO_SONIX_RECTO_R_MID,
            r_slope: DECISAO_SONIX_RECTO_R_SLOPE,
            r_load: DECISAO_SONIX_RECTO_R_LOAD,
            c1: DECISAO_SONIX_RECTO_C1,
            c2: DECISAO_SONIX_RECTO_C2,
            c3: DECISAO_SONIX_RECTO_C3,
            t_pos: audio_taper(t),
            b_pos: audio_taper(b),
            m_pos: m,
        },
        ToneStackTopology::PassiveJamesBaxandall => FmvComponents {
            r_src: 0.0,
            r_treble: 100_000.0,
            r_bass: 100_000.0,
            r_mid: 47_000.0,
            r_slope: 47_000.0,
            r_load: 1_000_000.0,
            c1: 1e-9,
            c2: 47e-9,
            c3: 47e-9,
            t_pos: t,
            b_pos: b,
            m_pos: 0.5,
        },
    }
}

/// Componentes físicos com impedância da fonte `R_src` e tapers reais (Spec 078 / Amps modulares 054–058).
pub fn spec078_fmv_components(
    topo: ToneStackTopology,
    bass: f32,
    middle: f32,
    treble: f32,
) -> FmvComponents {
    let b = (bass as f64 / 10.0).clamp(0.0, 1.0);
    let m = (middle as f64 / 10.0).clamp(0.0, 1.0);
    let t = (treble as f64 / 10.0).clamp(0.0, 1.0);

    match topo {
        ToneStackTopology::MarshallCathodeFollower | ToneStackTopology::Model => FmvComponents {
            // Marshall 1959HW / 2203 cathode-follower fed FMV stack (R_src = 1.3 kOhm)
            r_src: 1_300.0,
            r_treble: 220_000.0,
            r_bass: 1_000_000.0,
            r_mid: 22_000.0,
            r_slope: 33_000.0,
            r_load: 1_000_000.0,
            c1: 470e-12,
            c2: 22e-9,
            c3: 22e-9,
            t_pos: linear_taper(t),
            b_pos: log10_taper(b),
            m_pos: log10_taper(m),
        },
        ToneStackTopology::FenderTmbMidScoop => FmvComponents {
            // Fender Twin Reverb AB763 plate-driven (R_src = 38 kOhm, Bass/Treble audio 25%, Mid linear)
            r_src: 38_000.0,
            r_treble: 250_000.0,
            r_bass: 250_000.0,
            r_mid: 10_000.0,
            r_slope: 100_000.0,
            r_load: 1_000_000.0,
            c1: 250e-12,
            c2: 100e-9,
            c3: 47e-9,
            t_pos: audio25_taper(t),
            b_pos: audio25_taper(b),
            m_pos: linear_taper(m),
        },
        ToneStackTopology::VoxTopBoost => FmvComponents {
            // Vox AC30/6 Top Boost OS/010 (R_src = 1.5 kOhm, sem pot Mid)
            r_src: 1_500.0,
            r_treble: 1_000_000.0,
            r_bass: 1_000_000.0,
            r_mid: 10_000.0,
            r_slope: 100_000.0,
            r_load: 1_000_000.0,
            c1: 50e-12,
            c2: 22e-9,
            c3: 22e-9,
            t_pos: audio25_taper(t),
            b_pos: audio25_taper(b),
            m_pos: 1.0,
        },
        ToneStackTopology::RectoPostGain | ToneStackTopology::MesaMarkPreGain => FmvComponents {
            // Dual Rectifier Rev G (R_src = 1.3 kOhm, C1 = 500 pF, Mid B25k linear)
            r_src: RECTO_REV_G_R_SRC,
            r_treble: RECTO_REV_G_R_TREBLE,
            r_bass: RECTO_REV_G_R_BASS,
            r_mid: RECTO_REV_G_R_MID,
            r_slope: RECTO_REV_G_R_SLOPE,
            r_load: RECTO_REV_G_R_LOAD,
            c1: RECTO_REV_G_C1,
            c2: RECTO_REV_G_C2,
            c3: RECTO_REV_G_C3,
            t_pos: audio25_taper(t),
            b_pos: audio25_taper(b),
            m_pos: linear_taper(m),
        },
        ToneStackTopology::PassiveJamesBaxandall => FmvComponents {
            r_src: 1_300.0,
            r_treble: 100_000.0,
            r_bass: 100_000.0,
            r_mid: 47_000.0,
            r_slope: 47_000.0,
            r_load: 1_000_000.0,
            c1: 1e-9,
            c2: 47e-9,
            c3: 47e-9,
            t_pos: linear_taper(t),
            b_pos: linear_taper(b),
            m_pos: 0.5,
        },
    }
}

/// Constrói componentes JCM800 2203 com tapers específicos (Bass audio 25%, Mid/Treble linear B).
pub fn jcm800_fmv_components(bass: f32, middle: f32, treble: f32) -> FmvComponents {
    let b = (bass as f64 / 10.0).clamp(0.0, 1.0);
    let m = (middle as f64 / 10.0).clamp(0.0, 1.0);
    let t = (treble as f64 / 10.0).clamp(0.0, 1.0);
    FmvComponents {
        r_src: 1_300.0,
        r_treble: 220_000.0,
        r_bass: 1_000_000.0,
        r_mid: 22_000.0,
        r_slope: 33_000.0,
        r_load: 1_000_000.0,
        c1: 470e-12,
        c2: 22e-9,
        c3: 22e-9,
        t_pos: linear_taper(t),
        b_pos: audio25_taper(b),
        m_pos: linear_taper(m),
    }
}

type Poly = [f64; 6];

#[inline]
fn poly_add(a: Poly, b: Poly) -> Poly {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3], a[4] + b[4], a[5] + b[5]]
}

#[inline]
fn poly_sub(a: Poly, b: Poly) -> Poly {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3], a[4] - b[4], a[5] - b[5]]
}

#[inline]
fn poly_mul(a: Poly, b: Poly) -> Poly {
    let mut out = [0.0f64; 6];
    for i in 0..6 {
        if a[i] == 0.0 {
            continue;
        }
        for j in 0..(6 - i) {
            out[i + j] += a[i] * b[j];
        }
    }
    out
}

#[inline]
fn lin(g: f64, c: f64) -> Poly {
    [g, c, 0.0, 0.0, 0.0, 0.0]
}

fn det_3x3(m: &[[Poly; 3]; 3]) -> Poly {
    let c0 = poly_sub(poly_mul(m[1][1], m[2][2]), poly_mul(m[1][2], m[2][1]));
    let c1 = poly_sub(poly_mul(m[1][0], m[2][2]), poly_mul(m[1][2], m[2][0]));
    let c2 = poly_sub(poly_mul(m[1][0], m[2][1]), poly_mul(m[1][1], m[2][0]));
    poly_add(
        poly_sub(poly_mul(m[0][0], c0), poly_mul(m[0][1], c1)),
        poly_mul(m[0][2], c2),
    )
}

fn det_4x4(m: &[[Poly; 4]; 4]) -> Poly {
    let mut acc = [0.0f64; 6];
    for col in 0..4 {
        let pivot = m[0][col];
        if pivot[0] == 0.0 && pivot[1] == 0.0 {
            continue;
        }
        let mut sub = [[[0.0; 6]; 3]; 3];
        for r in 1..4 {
            let mut sc = 0;
            for c in 0..4 {
                if c == col {
                    continue;
                }
                sub[r - 1][sc] = m[r][c];
                sc += 1;
            }
        }
        let term = poly_mul(pivot, det_3x3(&sub));
        acc = if col % 2 == 0 {
            poly_add(acc, term)
        } else {
            poly_sub(acc, term)
        };
    }
    acc
}

fn det_5x5(m: &[[Poly; 5]; 5]) -> Poly {
    let mut acc = [0.0f64; 6];
    for col in 0..5 {
        let pivot = m[0][col];
        if pivot[0] == 0.0 && pivot[1] == 0.0 {
            continue;
        }
        let mut sub = [[[0.0; 6]; 4]; 4];
        for r in 1..5 {
            let mut sc = 0;
            for c in 0..5 {
                if c == col {
                    continue;
                }
                sub[r - 1][sc] = m[r][c];
                sc += 1;
            }
        }
        let term = poly_mul(pivot, det_4x4(&sub));
        acc = if col % 2 == 0 {
            poly_add(acc, term)
        } else {
            poly_sub(acc, term)
        };
    }
    acc
}

fn det_6x6(m: &[[Poly; 6]; 6]) -> Poly {
    let mut acc = [0.0f64; 6];
    for col in 0..6 {
        let pivot = m[0][col];
        if pivot[0] == 0.0 && pivot[1] == 0.0 {
            continue;
        }
        let mut sub = [[[0.0; 6]; 5]; 5];
        for r in 1..6 {
            let mut sc = 0;
            for c in 0..6 {
                if c == col {
                    continue;
                }
                sub[r - 1][sc] = m[r][c];
                sc += 1;
            }
        }
        let term = poly_mul(pivot, det_5x5(&sub));
        acc = if col % 2 == 0 {
            poly_add(acc, term)
        } else {
            poly_sub(acc, term)
        };
    }
    acc
}

const HB_A0: f64 = 0.10557280900008414; // tan^2(pi / 10)
const HB_A1: f64 = 0.5278640450004207;  // tan^2(2 * pi / 10)

/// Polinômios KCL de 3ª ordem da rede FMV incluindo impedância da fonte `r_src` (Yeh & Smith DAFx-06).
pub fn fmv_transfer_polynomials(c: FmvComponents) -> ([f64; 4], [f64; 4]) {
    let r_t1 = ((1.0 - c.t_pos) * c.r_treble).max(1e-2);
    let r_t2 = (c.t_pos * c.r_treble).max(1e-2);
    let r_b = (c.b_pos * c.r_bass + (1.0 - c.m_pos) * c.r_mid).max(1e-2);
    let r_m = (c.m_pos * c.r_mid).max(1e-2);

    let g_t1 = R_REF / r_t1;
    let g_t2 = R_REF / r_t2;
    let g_b = R_REF / r_b;
    let g_m = R_REF / r_m;
    let g_4 = R_REF / c.r_slope.max(1.0);
    let g_l = if c.r_load.is_finite() && c.r_load > 0.0 {
        R_REF / c.r_load.max(1.0)
    } else {
        0.0
    };

    let c1 = c.c1 / C_REF;
    let c2 = c.c2 / C_REF;
    let c3 = c.c3 / C_REF;
    let zero = [0.0; 6];

    if c.r_src <= 1e-2 {
        let mut m = [[zero; 5]; 5];
        m[0][0] = lin(g_t1, c1);
        m[0][1] = lin(-g_t1, 0.0);

        m[1][0] = lin(-g_t1, 0.0);
        m[1][1] = lin(g_t1 + g_t2 + g_l, 0.0);
        m[1][2] = lin(-g_t2, 0.0);

        m[2][1] = lin(-g_t2, 0.0);
        m[2][2] = lin(g_t2 + g_b, c2);
        m[2][3] = lin(-g_b, 0.0);
        m[2][4] = lin(0.0, -c2);

        m[3][2] = lin(-g_b, 0.0);
        m[3][3] = lin(g_b + g_m, c3);
        m[3][4] = lin(0.0, -c3);

        m[4][2] = lin(0.0, -c2);
        m[4][3] = lin(0.0, -c3);
        m[4][4] = lin(g_4, c2 + c3);

        let den = det_5x5(&m);
        let mut m_num = m;
        m_num[0][1] = lin(0.0, c1);
        m_num[1][1] = zero;
        m_num[2][1] = zero;
        m_num[3][1] = zero;
        m_num[4][1] = lin(g_4, 0.0);
        let num = det_5x5(&m_num);
        return ([num[0], num[1], num[2], num[3]], [den[0], den[1], den[2], den[3]]);
    }

    // Sistema 6×6: [V0, V1, Vout, Vbm, Vm, Vs] com R_src entre Vin e V0
    let g_src = R_REF / c.r_src;
    let mut m = [[zero; 6]; 6];
    m[0][0] = lin(g_src + g_4, c1);
    m[0][1] = lin(0.0, -c1);
    m[0][5] = lin(-g_4, 0.0);

    m[1][0] = lin(0.0, -c1);
    m[1][1] = lin(g_t1, c1);
    m[1][2] = lin(-g_t1, 0.0);

    m[2][1] = lin(-g_t1, 0.0);
    m[2][2] = lin(g_t1 + g_t2 + g_l, 0.0);
    m[2][3] = lin(-g_t2, 0.0);

    m[3][2] = lin(-g_t2, 0.0);
    m[3][3] = lin(g_t2 + g_b, c2);
    m[3][4] = lin(-g_b, 0.0);
    m[3][5] = lin(0.0, -c2);

    m[4][3] = lin(-g_b, 0.0);
    m[4][4] = lin(g_b + g_m, c3);
    m[4][5] = lin(0.0, -c3);

    m[5][0] = lin(-g_4, 0.0);
    m[5][3] = lin(0.0, -c2);
    m[5][4] = lin(0.0, -c3);
    m[5][5] = lin(g_4, c2 + c3);

    let den = det_6x6(&m);

    let mut m_num = m;
    m_num[0][2] = lin(g_src, 0.0);
    m_num[1][2] = zero;
    m_num[2][2] = zero;
    m_num[3][2] = zero;
    m_num[4][2] = zero;
    m_num[5][2] = zero;
    let num = det_6x6(&m_num);

    ([num[0], num[1], num[2], num[3]], [den[0], den[1], den[2], den[3]])
}

/// Função de transferência nodal KCL da rede Vox AC30/6 Top Boost (esquema JMI OS/010) com `r_src`.
pub fn vox_top_boost_transfer_polynomials(c: FmvComponents) -> ([f64; 4], [f64; 4]) {
    let r_t1 = ((1.0 - c.t_pos) * c.r_treble).max(1e-2);
    let r_t2 = (c.t_pos * c.r_treble).max(1e-2);
    let r_b1 = (c.b_pos * c.r_bass).max(1e-2);
    let r_b2 = ((1.0 - c.b_pos) * c.r_bass).max(1e-2);
    let r_tail = c.r_mid.max(1e-2);

    let g_t1 = R_REF / r_t1;
    let g_t2 = R_REF / r_t2;
    let g_b1 = R_REF / r_b1;
    let g_b2 = R_REF / r_b2;
    let g_tail = R_REF / r_tail;
    let g_4 = R_REF / c.r_slope.max(1.0);
    let g_l = if c.r_load.is_finite() && c.r_load > 0.0 {
        R_REF / c.r_load.max(1.0)
    } else {
        0.0
    };

    let c1 = c.c1 / C_REF;
    let c2 = c.c2 / C_REF;
    let c3 = c.c3 / C_REF;
    let zero = [0.0; 6];

    if c.r_src <= 1e-2 {
        let mut m = [[zero; 5]; 5];
        m[0][0] = lin(g_t1, c1);
        m[0][1] = lin(-g_t1, 0.0);

        m[1][0] = lin(-g_t1, 0.0);
        m[1][1] = lin(g_t1 + g_t2 + g_l, 0.0);
        m[1][2] = lin(-g_t2, 0.0);

        m[2][1] = lin(-g_t2, 0.0);
        m[2][2] = lin(g_t2 + g_b1, c2);
        m[2][3] = lin(-g_b1, 0.0);
        m[2][4] = lin(0.0, -c2);

        m[3][2] = lin(-g_b1, 0.0);
        m[3][3] = lin(g_b1 + g_b2 + g_tail, c3);
        m[3][4] = lin(0.0, -c3);

        m[4][2] = lin(0.0, -c2);
        m[4][3] = lin(0.0, -c3);
        m[4][4] = lin(g_4, c2 + c3);

        let den = det_5x5(&m);
        let mut m_num = m;
        m_num[0][1] = lin(0.0, c1);
        m_num[1][1] = zero;
        m_num[2][1] = zero;
        m_num[3][1] = zero;
        m_num[4][1] = lin(g_4, 0.0);
        let num = det_5x5(&m_num);
        return ([num[0], num[1], num[2], num[3]], [den[0], den[1], den[2], den[3]]);
    }

    let g_src = R_REF / c.r_src;
    let mut m = [[zero; 6]; 6];
    m[0][0] = lin(g_src + g_4, c1);
    m[0][1] = lin(0.0, -c1);
    m[0][5] = lin(-g_4, 0.0);

    m[1][0] = lin(0.0, -c1);
    m[1][1] = lin(g_t1, c1);
    m[1][2] = lin(-g_t1, 0.0);

    m[2][1] = lin(-g_t1, 0.0);
    m[2][2] = lin(g_t1 + g_t2 + g_l, 0.0);
    m[2][3] = lin(-g_t2, 0.0);

    m[3][2] = lin(-g_t2, 0.0);
    m[3][3] = lin(g_t2 + g_b1, c2);
    m[3][4] = lin(-g_b1, 0.0);
    m[3][5] = lin(0.0, -c2);

    m[4][3] = lin(-g_b1, 0.0);
    m[4][4] = lin(g_b1 + g_b2 + g_tail, c3);
    m[4][5] = lin(0.0, -c3);

    m[5][0] = lin(-g_4, 0.0);
    m[5][3] = lin(0.0, -c2);
    m[5][4] = lin(0.0, -c3);
    m[5][5] = lin(g_4, c2 + c3);

    let den = det_6x6(&m);
    let mut m_num = m;
    m_num[0][2] = lin(g_src, 0.0);
    m_num[1][2] = zero;
    m_num[2][2] = zero;
    m_num[3][2] = zero;
    m_num[4][2] = zero;
    m_num[5][2] = zero;
    let num = det_6x6(&m_num);

    ([num[0], num[1], num[2], num[3]], [den[0], den[1], den[2], den[3]])
}

/// Polinômios da rede passiva James/Baxandall de 2 controles shelving independentes.
/// Topologia passiva em ponte: ramo grave shelving (`f_b ~ 120 Hz`) + ramo agudo shelving (`f_t ~ 2.2 kHz`)
/// com atenuação de inserção passiva garantindo `|H(s)| <= 1.0` em todas as posições.
pub fn james_baxandall_transfer_polynomials(bass: f32, treble: f32) -> ([f64; 4], [f64; 4]) {
    let b = linear_taper(bass as f64 / 10.0);
    let t = linear_taper(treble as f64 / 10.0);

    // Dois shelves passivos em cascata normalizados para passividade estrita (ganho máximo <= 1.0)
    // Shelf grave: (1 + k_b * tau_b * p_s) / (1 + tau_b * p_s) escalado por g_b <= 1.0
    // Em p = s * (R_REF * C_REF): tau = (R_REF * C_REF) = 1ms -> w_b = 2*pi*120Hz -> tau_b = 1/(2*pi*120*1e-3)
    let tau_b = 1.0 / (2.0 * PI * 140.0 * R_REF * C_REF);
    let tau_t = 1.0 / (2.0 * PI * 2200.0 * R_REF * C_REF);

    // Ganho DC do ramo grave: de 0.125 (-18 dB) em b=0 até 1.0 (0 dB) em b=1; em b=0.5 -> ~0.353
    let dc_b = 0.125 * (8.0f64).powf(b);
    let hf_b = 0.35355; // referência média fixa em ~1 kHz para manter passividade
    // Ganho HF do ramo agudo: de 0.125 em t=0 até 1.0 em t=1; em t=0.5 -> ~0.353
    let hf_t = 0.125 * (8.0f64).powf(t);
    let dc_t = 0.35355;

    // H_b(p) = (dc_b + hf_b * tau_b * p) / (1 + tau_b * p)
    // H_t(p) = (dc_t + hf_t * tau_t * p) / (1 + tau_t * p)
    // Como max(dc_b, hf_b) <= 1.0 e max(dc_t, hf_t) <= 1.0, |H_b(jw)| <= 1.0 e |H_t(jw)| <= 1.0!
    let b0 = dc_b * dc_t;
    let b1 = dc_b * hf_t * tau_t + hf_b * tau_b * dc_t;
    let b2 = hf_b * tau_b * hf_t * tau_t;

    let a0 = 1.0;
    let a1 = tau_b + tau_t;
    let a2 = tau_b * tau_t;

    ([b0, b1, b2, 0.0], [a0, a1, a2, 0.0])
}

#[inline]
fn eval_polynomial_magnitude_db(b: [f64; 4], a: [f64; 4], freq_hz: f64) -> f64 {
    let w = 2.0 * PI * freq_hz * (R_REF * C_REF);
    let w2 = w * w;
    let w3 = w2 * w;

    let num_re = b[0] - b[2] * w2;
    let num_im = b[1] * w - b[3] * w3;
    let den_re = a[0] - a[2] * w2;
    let den_im = a[1] * w - a[3] * w3;

    let num_sq = num_re * num_re + num_im * num_im;
    let den_sq = (den_re * den_re + den_im * den_im).max(1e-24);
    10.0 * (num_sq / den_sq).max(1e-18).log10()
}

/// Avalia |H(j * 2 * pi * freq_hz)| em dB para componentes FMV arbitrários (incluindo `r_src`).
pub fn analog_tonestack_magnitude_db_for_components(comps: FmvComponents, freq_hz: f64) -> f64 {
    let (b, a) = fmv_transfer_polynomials(comps);
    eval_polynomial_magnitude_db(b, a, freq_hz)
}

/// Avalia |H(j * 2 * pi * freq_hz)| em dB para qualquer topologia de Tone Stack.
pub fn analog_tonestack_magnitude_db(
    topo: ToneStackTopology,
    bass: f32,
    middle: f32,
    treble: f32,
    freq_hz: f64,
) -> f64 {
    let comps = resolved_fmv_components(topo, bass, middle, treble);
    let (b, a) = match topo {
        ToneStackTopology::VoxTopBoost => vox_top_boost_transfer_polynomials(comps),
        ToneStackTopology::PassiveJamesBaxandall => james_baxandall_transfer_polynomials(bass, treble),
        _ => fmv_transfer_polynomials(comps),
    };
    eval_polynomial_magnitude_db(b, a, freq_hz)
}

/// Núcleo IIR bilinear em `f64` (Forma Direta I com interpolação suave anti-zipper de coeficientes).
#[derive(Debug, Clone, Copy)]
struct Bilinear3rdOrderCore {
    b: [f64; 4],
    a: [f64; 4],
    target_b: [f64; 4],
    target_a: [f64; 4],
    step_b: [f64; 4],
    step_a: [f64; 4],
    ramp_rem: usize,
    initialized: bool,
    x_hist: [f64; 3],
    y_hist: [f64; 3],
    internal_os2x: bool,
    up0_x: f64,
    up0_y: f64,
    up1_x: f64,
    up1_y: f64,
    dn0_x: f64,
    dn0_y: f64,
    dn1_x: f64,
    dn1_y: f64,
}

impl Bilinear3rdOrderCore {
    fn new() -> Self {
        Self {
            b: [1.0, 0.0, 0.0, 0.0],
            a: [1.0, 0.0, 0.0, 0.0],
            target_b: [1.0, 0.0, 0.0, 0.0],
            target_a: [1.0, 0.0, 0.0, 0.0],
            step_b: [0.0; 4],
            step_a: [0.0; 4],
            ramp_rem: 0,
            initialized: false,
            x_hist: [0.0; 3],
            y_hist: [0.0; 3],
            internal_os2x: false,
            up0_x: 0.0,
            up0_y: 0.0,
            up1_x: 0.0,
            up1_y: 0.0,
            dn0_x: 0.0,
            dn0_y: 0.0,
            dn1_x: 0.0,
            dn1_y: 0.0,
        }
    }

    fn reset(&mut self) {
        self.x_hist = [0.0; 3];
        self.y_hist = [0.0; 3];
        self.up0_x = 0.0;
        self.up0_y = 0.0;
        self.up1_x = 0.0;
        self.up1_y = 0.0;
        self.dn0_x = 0.0;
        self.dn0_y = 0.0;
        self.dn1_x = 0.0;
        self.dn1_y = 0.0;
        self.b = self.target_b;
        self.a = self.target_a;
        self.ramp_rem = 0;
    }

    fn apply_transfer_polynomials(
        &mut self,
        bc: [f64; 4],
        ac: [f64; 4],
        sample_rate_hz: f64,
        immediate: bool,
    ) {
        self.internal_os2x = sample_rate_hz < 88_200.0;
        let eff_sr = if self.internal_os2x {
            sample_rate_hz.max(8000.0) * 2.0
        } else {
            sample_rate_hz.max(8000.0)
        };

        let k = 2.0 * eff_sr * (R_REF * C_REF);
        let k2 = k * k;
        let k3 = k2 * k;

        let transform = |c: [f64; 4]| -> [f64; 4] {
            let c0 = c[0];
            let c1 = c[1] * k;
            let c2 = c[2] * k2;
            let c3 = c[3] * k3;
            [
                c0 + c1 + c2 + c3,
                3.0 * c0 + c1 - c2 - 3.0 * c3,
                3.0 * c0 - c1 - c2 + 3.0 * c3,
                c0 - c1 + c2 - c3,
            ]
        };

        let bz = transform(bc);
        let az = transform(ac);
        let inv_a0 = if az[0].abs() > 1e-18 { 1.0 / az[0] } else { 1.0 };

        let new_b = [bz[0] * inv_a0, bz[1] * inv_a0, bz[2] * inv_a0, bz[3] * inv_a0];
        let new_a = [1.0, az[1] * inv_a0, az[2] * inv_a0, az[3] * inv_a0];

        self.target_b = new_b;
        self.target_a = new_a;

        if immediate || !self.initialized {
            self.initialized = true;
            self.b = new_b;
            self.a = new_a;
            self.ramp_rem = 0;
        } else {
            const RAMP_SAMPLES: usize = 64;
            let inv_n = 1.0 / (RAMP_SAMPLES as f64);
            for i in 0..4 {
                self.step_b[i] = (new_b[i] - self.b[i]) * inv_n;
                self.step_a[i] = (new_a[i] - self.a[i]) * inv_n;
            }
            self.ramp_rem = RAMP_SAMPLES;
        }
    }

    #[inline(always)]
    fn ap1(x: f64, a: f64, x1: &mut f64, y1: &mut f64) -> f64 {
        let y = crate::denormal::flush_f64(a * (x - *y1) + *x1);
        *x1 = crate::denormal::flush_f64(x);
        *y1 = y;
        y
    }

    #[inline(always)]
    fn process_df1(&mut self, x: f64) -> f64 {
        if self.ramp_rem > 0 {
            for i in 0..4 {
                self.b[i] += self.step_b[i];
                self.a[i] += self.step_a[i];
            }
            self.ramp_rem -= 1;
            if self.ramp_rem == 0 {
                self.b = self.target_b;
                self.a = self.target_a;
            }
        }
        let y = crate::denormal::flush_f64(
            self.b[0] * x
                + self.b[1] * self.x_hist[0]
                + self.b[2] * self.x_hist[1]
                + self.b[3] * self.x_hist[2]
                - self.a[1] * self.y_hist[0]
                - self.a[2] * self.y_hist[1]
                - self.a[3] * self.y_hist[2],
        );
        self.x_hist[2] = self.x_hist[1];
        self.x_hist[1] = self.x_hist[0];
        self.x_hist[0] = crate::denormal::flush_f64(x);
        self.y_hist[2] = self.y_hist[1];
        self.y_hist[1] = self.y_hist[0];
        self.y_hist[0] = y;
        y
    }

    #[inline(always)]
    fn process(&mut self, x: f64) -> f64 {
        if self.internal_os2x {
            let s0 = Self::ap1(x, HB_A0, &mut self.up0_x, &mut self.up0_y);
            let s1 = Self::ap1(x, HB_A1, &mut self.up1_x, &mut self.up1_y);
            let y0 = self.process_df1(s0);
            let y1 = self.process_df1(s1);
            let d0 = Self::ap1(y1, HB_A0, &mut self.dn0_x, &mut self.dn0_y);
            let d1 = Self::ap1(y0, HB_A1, &mut self.dn1_x, &mut self.dn1_y);
            0.5 * (d0 + d1)
        } else {
            self.process_df1(x)
        }
    }
}

/// Solver 1 (Spec 078): Rede FMV de 3ª ordem com impedância de fonte `r_src` (Marshall / Fender).
#[derive(Debug, Clone, Copy)]
pub struct FmvToneStackSolver {
    core: Bilinear3rdOrderCore,
}

impl Default for FmvToneStackSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl FmvToneStackSolver {
    pub fn new() -> Self {
        Self {
            core: Bilinear3rdOrderCore::new(),
        }
    }

    pub fn reset(&mut self) {
        self.core.reset();
    }

    pub fn update_components(&mut self, comps: FmvComponents, sample_rate_hz: f64, immediate: bool) {
        let (bc, ac) = fmv_transfer_polynomials(comps);
        self.core
            .apply_transfer_polynomials(bc, ac, sample_rate_hz, immediate);
    }

    #[inline(always)]
    pub fn process(&mut self, x: f64) -> f64 {
        self.core.process(x)
    }
}

/// Solver 2 (Spec 078): Rede Vox Top Boost `OS/010` (sem potenciômetro Mid físico).
#[derive(Debug, Clone, Copy)]
pub struct VoxTopBoostToneStackSolver {
    core: Bilinear3rdOrderCore,
}

impl Default for VoxTopBoostToneStackSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl VoxTopBoostToneStackSolver {
    pub fn new() -> Self {
        Self {
            core: Bilinear3rdOrderCore::new(),
        }
    }

    pub fn reset(&mut self) {
        self.core.reset();
    }

    pub fn update(&mut self, bass: f32, treble: f32, sample_rate_hz: f64, immediate: bool) {
        let comps = spec078_fmv_components(ToneStackTopology::VoxTopBoost, bass, 5.0, treble);
        let (bc, ac) = vox_top_boost_transfer_polynomials(comps);
        self.core
            .apply_transfer_polynomials(bc, ac, sample_rate_hz, immediate);
    }

    #[inline(always)]
    pub fn process(&mut self, x: f64) -> f64 {
        self.core.process(x)
    }
}

/// Solver 3 (Spec 078): Rede passiva James/Baxandall de 2 controles shelving independentes.
#[derive(Debug, Clone, Copy)]
pub struct JamesBaxandallToneStackSolver {
    core: Bilinear3rdOrderCore,
}

impl Default for JamesBaxandallToneStackSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl JamesBaxandallToneStackSolver {
    pub fn new() -> Self {
        Self {
            core: Bilinear3rdOrderCore::new(),
        }
    }

    pub fn reset(&mut self) {
        self.core.reset();
    }

    pub fn update(&mut self, bass: f32, treble: f32, sample_rate_hz: f64, immediate: bool) {
        let (bc, ac) = james_baxandall_transfer_polynomials(bass, treble);
        self.core
            .apply_transfer_polynomials(bc, ac, sample_rate_hz, immediate);
    }

    #[inline(always)]
    pub fn process(&mut self, x: f64) -> f64 {
        self.core.process(x)
    }
}

/// Solver 4 (Spec 078): Rede Mesa/Boogie Dual Rectifier Rev G (`RF-1G`, `R_src = 1.3 kΩ`, `C1 = 500 pF`).
#[derive(Debug, Clone, Copy)]
pub struct MesaRectifierToneStackSolver {
    core: Bilinear3rdOrderCore,
}

impl Default for MesaRectifierToneStackSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl MesaRectifierToneStackSolver {
    pub fn new() -> Self {
        Self {
            core: Bilinear3rdOrderCore::new(),
        }
    }

    pub fn reset(&mut self) {
        self.core.reset();
    }

    pub fn update(
        &mut self,
        bass: f32,
        middle: f32,
        treble: f32,
        sample_rate_hz: f64,
        immediate: bool,
    ) {
        let comps = spec078_fmv_components(ToneStackTopology::RectoPostGain, bass, middle, treble);
        let (bc, ac) = fmv_transfer_polynomials(comps);
        self.core
            .apply_transfer_polynomials(bc, ac, sample_rate_hz, immediate);
    }

    #[inline(always)]
    pub fn process(&mut self, x: f64) -> f64 {
        self.core.process(x)
    }
}

/// 3rd-order Direct Form I passive Tone Stack filter (bilinear transform).
/// Uses signal histories (`x_hist`, `y_hist`) so smooth coefficient ramps avoid
/// Direct Form II state-jump transients at high oversampling rates.
#[derive(Debug, Clone, Copy)]
pub struct PassiveToneStack {
    b: [f64; 4],
    a: [f64; 4], // a[0] normalized to 1.0, stores a1..a3 in a[1..4]
    x_hist: [f64; 3],
    y_hist: [f64; 3],
    internal_os2x: bool,
    up0_x: f64,
    up0_y: f64,
    up1_x: f64,
    up1_y: f64,
    dn0_x: f64,
    dn0_y: f64,
    dn1_x: f64,
    dn1_y: f64,
    last_topo: ToneStackTopology,
    last_b: f32,
    last_m: f32,
    last_t: f32,
    last_sr: f64,
}

impl Default for PassiveToneStack {
    fn default() -> Self {
        Self::new()
    }
}

impl PassiveToneStack {
    pub fn new() -> Self {
        let mut s = Self {
            b: [1.0, 0.0, 0.0, 0.0],
            a: [1.0, 0.0, 0.0, 0.0],
            x_hist: [0.0; 3],
            y_hist: [0.0; 3],
            internal_os2x: false,
            up0_x: 0.0,
            up0_y: 0.0,
            up1_x: 0.0,
            up1_y: 0.0,
            dn0_x: 0.0,
            dn0_y: 0.0,
            dn1_x: 0.0,
            dn1_y: 0.0,
            last_topo: ToneStackTopology::MarshallCathodeFollower,
            last_b: -1.0,
            last_m: -1.0,
            last_t: -1.0,
            last_sr: 0.0,
        };
        s.update(ToneStackTopology::MarshallCathodeFollower, 5.0, 5.0, 5.0, 384_000.0);
        s
    }

    pub fn reset(&mut self) {
        self.x_hist = [0.0; 3];
        self.y_hist = [0.0; 3];
        self.up0_x = 0.0;
        self.up0_y = 0.0;
        self.up1_x = 0.0;
        self.up1_y = 0.0;
        self.dn0_x = 0.0;
        self.dn0_y = 0.0;
        self.dn1_x = 0.0;
        self.dn1_y = 0.0;
    }

    fn apply_transfer_polynomials(&mut self, bc: [f64; 4], ac: [f64; 4], sample_rate_hz: f64) {
        self.internal_os2x = sample_rate_hz < 88_200.0;
        let eff_sr = if self.internal_os2x {
            sample_rate_hz.max(8000.0) * 2.0
        } else {
            sample_rate_hz.max(8000.0)
        };

        // Bilinear transform: p = K * (1 - z^-1) / (1 + z^-1), where p = s * (R_REF * C_REF)
        let k = 2.0 * eff_sr * (R_REF * C_REF);
        let k2 = k * k;
        let k3 = k2 * k;

        let transform = |c: [f64; 4]| -> [f64; 4] {
            let c0 = c[0];
            let c1 = c[1] * k;
            let c2 = c[2] * k2;
            let c3 = c[3] * k3;
            [
                c0 + c1 + c2 + c3,
                3.0 * c0 + c1 - c2 - 3.0 * c3,
                3.0 * c0 - c1 - c2 + 3.0 * c3,
                c0 - c1 + c2 - c3,
            ]
        };

        let bz = transform(bc);
        let az = transform(ac);
        let inv_a0 = if az[0].abs() > 1e-18 { 1.0 / az[0] } else { 1.0 };

        self.b = [bz[0] * inv_a0, bz[1] * inv_a0, bz[2] * inv_a0, bz[3] * inv_a0];
        self.a = [1.0, az[1] * inv_a0, az[2] * inv_a0, az[3] * inv_a0];
    }

    pub fn update_with_components(&mut self, comps: FmvComponents, sample_rate_hz: f64) {
        self.last_sr = -1.0;
        let (bc, ac) = fmv_transfer_polynomials(comps);
        self.apply_transfer_polynomials(bc, ac, sample_rate_hz);
    }

    pub fn update(
        &mut self,
        topo: ToneStackTopology,
        bass: f32,
        middle: f32,
        treble: f32,
        sample_rate_hz: f64,
    ) {
        if topo == self.last_topo
            && (bass - self.last_b).abs() < 1e-4
            && (middle - self.last_m).abs() < 1e-4
            && (treble - self.last_t).abs() < 1e-4
            && (sample_rate_hz - self.last_sr).abs() < 1.0
        {
            return;
        }
        self.last_topo = topo;
        self.last_b = bass;
        self.last_m = middle;
        self.last_t = treble;
        self.last_sr = sample_rate_hz;

        let comps = resolved_fmv_components(topo, bass, middle, treble);
        let (bc, ac) = if topo == ToneStackTopology::VoxTopBoost {
            vox_top_boost_transfer_polynomials(comps)
        } else {
            fmv_transfer_polynomials(comps)
        };
        self.apply_transfer_polynomials(bc, ac, sample_rate_hz);
    }

    #[inline(always)]
    fn ap1(x: f64, a: f64, x1: &mut f64, y1: &mut f64) -> f64 {
        let y = crate::denormal::flush_f64(a * (x - *y1) + *x1);
        *x1 = crate::denormal::flush_f64(x);
        *y1 = y;
        y
    }

    #[inline(always)]
    fn process_df1(&mut self, x: f64) -> f64 {
        let y = crate::denormal::flush_f64(
            self.b[0] * x
                + self.b[1] * self.x_hist[0]
                + self.b[2] * self.x_hist[1]
                + self.b[3] * self.x_hist[2]
                - self.a[1] * self.y_hist[0]
                - self.a[2] * self.y_hist[1]
                - self.a[3] * self.y_hist[2],
        );
        self.x_hist[2] = self.x_hist[1];
        self.x_hist[1] = self.x_hist[0];
        self.x_hist[0] = crate::denormal::flush_f64(x);
        self.y_hist[2] = self.y_hist[1];
        self.y_hist[1] = self.y_hist[0];
        self.y_hist[0] = y;
        y
    }

    #[inline(always)]
    pub fn process(&mut self, x: f64) -> f64 {
        if self.internal_os2x {
            let s0 = Self::ap1(x, HB_A0, &mut self.up0_x, &mut self.up0_y);
            let s1 = Self::ap1(x, HB_A1, &mut self.up1_x, &mut self.up1_y);
            let y0 = self.process_df1(s0);
            let y1 = self.process_df1(s1);
            let d0 = Self::ap1(y1, HB_A0, &mut self.dn0_x, &mut self.dn0_y);
            let d1 = Self::ap1(y0, HB_A1, &mut self.dn1_x, &mut self.dn1_y);
            0.5 * (d0 + d1)
        } else {
            self.process_df1(x)
        }
    }
}
