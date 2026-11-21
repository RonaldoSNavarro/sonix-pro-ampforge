//! Passive interactive Tone Stack circuit modeling (FMV / Top Boost / Baxandall)
//! Discretized from exact nodal KCL continuous-time transfer function via bilinear transform.
//! Zero heap allocations, f64 internal precision.

use crate::modshop::ToneStackTopology;
use std::f64::consts::PI;

const R_REF: f64 = 100_000.0;
const C_REF: f64 = 10e-9; // tau_ref = R_REF * C_REF = 1e-3 s

#[derive(Debug, Clone, Copy)]
pub struct FmvComponents {
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

#[inline]
fn audio_taper(x: f64) -> f64 {
    // 10% log/audio taper curve commonly found on vintage CTS/Alpha pots
    let u = x.clamp(0.0, 1.0);
    (0.01 * 100.0f64.powf(u) - 0.01) / 0.99
}

#[inline]
fn map_pot_mid_10pct(x: f64) -> f64 {
    let u = x.clamp(0.0, 1.0);
    // Smooth linear/10%-log hybrid for vintage Mid controls
    0.35 * u + 0.65 * audio_taper(u)
}

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
            // Marshall 1959 / 2203 cathode-follower fed FMV stack (33k, 470pF, 22nF, 22nF, 220k Treble, 22k Mid)
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
            m_pos: map_pot_mid_10pct(m),
        },
        ToneStackTopology::FenderTmbMidScoop => FmvComponents {
            // Fender Twin Reverb AB763 (100k, 250pF, 0.1uF=100nF, 0.047uF=47nF, 10k Mid)
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
            // Vox AC30/6 Top Boost (100k slope, 50pF treble, 22nF/22nF bass, fixed 10k tail)
            r_treble: 220_000.0,
            r_bass: 1_000_000.0,
            r_mid: 10_000.0,
            r_slope: 100_000.0,
            r_load: 1_000_000.0,
            c1: 50e-12,
            c2: 22e-9,
            c3: 22e-9,
            t_pos: audio_taper(t),
            b_pos: audio_taper(b),
            // Original Top Boost has no Middle pot (fixed 10k resistor to ground)
            m_pos: 1.0,
        },
        ToneStackTopology::MesaMarkPreGain => FmvComponents {
            // Mesa/Boogie Dual Rectifier FMV stack (39k, 470pF, 22nF, 22nF)
            r_treble: 250_000.0,
            r_bass: 250_000.0,
            r_mid: 25_000.0,
            r_slope: 39_000.0,
            r_load: 1_000_000.0,
            c1: 470e-12,
            c2: 22e-9,
            c3: 22e-9,
            t_pos: audio_taper(t),
            b_pos: audio_taper(b),
            m_pos: map_pot_mid_10pct(m),
        },
        ToneStackTopology::PassiveJamesBaxandall => FmvComponents {
            // James passive Baxandall style modeled on dual-shelf FMV-compatible bounds
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

/// Computes normalized polynomial transfer function H(p) = (b0+b1 p+b2 p^2+b3 p^3) / (a0+a1 p+a2 p^2+a3 p^3),
/// where p = s * (R_REF * C_REF).
fn fmv_transfer_polynomials(c: FmvComponents) -> ([f64; 4], [f64; 4]) {
    let r_t1 = ((1.0 - c.t_pos) * c.r_treble).max(50.0);
    let r_t2 = (c.t_pos * c.r_treble).max(50.0);
    let r_b = (c.b_pos * c.r_bass).max(50.0);
    let r_m = (c.m_pos * c.r_mid).max(20.0);

    let g_t1 = R_REF / r_t1;
    let g_t2 = R_REF / r_t2;
    let g_b = R_REF / r_b;
    let g_m = R_REF / r_m;
    let g_4 = R_REF / c.r_slope.max(100.0);
    let g_l = R_REF / c.r_load.max(10_000.0);

    let c1 = c.c1 / C_REF;
    let c2 = c.c2 / C_REF;
    let c3 = c.c3 / C_REF;

    // 5x5 KCL matrix M(p) * [V1, Vout, Vbm, Vm, Vs]^T = [c1*p, 0, 0, 0, g4]^T
    let zero = [0.0; 6];
    let mut m = [[zero; 5]; 5];
    m[0][0] = lin(g_t1, c1);
    m[0][1] = lin(-g_t1, 0.0);

    m[1][0] = lin(-g_t1, 0.0);
    m[1][1] = lin(g_t1 + g_t2 + g_l, 0.0);
    m[1][2] = lin(-g_t2, 0.0);

    m[2][1] = lin(-g_t2, 0.0);
    m[2][2] = lin(g_t2 + g_b, c3);
    m[2][3] = lin(-g_b, 0.0);
    m[2][4] = lin(0.0, -c3);

    m[3][2] = lin(-g_b, 0.0);
    m[3][3] = lin(g_b + g_m, c2);
    m[3][4] = lin(0.0, -c2);

    m[4][2] = lin(0.0, -c3);
    m[4][3] = lin(0.0, -c2);
    m[4][4] = lin(g_4, c2 + c3);

    let den = det_5x5(&m);

    // Replace column 1 (Vout) with RHS = [c1*p, 0, 0, 0, g4]^T
    let mut m_num = m;
    m_num[0][1] = lin(0.0, c1);
    m_num[1][1] = zero;
    m_num[2][1] = zero;
    m_num[3][1] = zero;
    m_num[4][1] = lin(g_4, 0.0);

    let num = det_5x5(&m_num);

    ([num[0], num[1], num[2], num[3]], [den[0], den[1], den[2], den[3]])
}

/// Evaluates continuous-time |H(j * 2 * pi * freq_hz)| in dB from the exact schematic RC network.
pub fn analog_tonestack_magnitude_db(
    topo: ToneStackTopology,
    bass: f32,
    middle: f32,
    treble: f32,
    freq_hz: f64,
) -> f64 {
    let comps = resolved_fmv_components(topo, bass, middle, treble);
    let (b, a) = fmv_transfer_polynomials(comps);
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

/// 3rd-order Direct Form I passive Tone Stack filter (bilinear transform).
/// Uses signal histories (`x_hist`, `y_hist`) so smooth coefficient ramps avoid
/// Direct Form II state-jump transients at high oversampling rates.
#[derive(Debug, Clone, Copy)]
pub struct PassiveToneStack {
    b: [f64; 4],
    a: [f64; 4], // a[0] normalized to 1.0, stores a1..a3 in a[1..4]
    x_hist: [f64; 3],
    y_hist: [f64; 3],
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
        let (bc, ac) = fmv_transfer_polynomials(comps);

        // Bilinear transform: p = K * (1 - z^-1) / (1 + z^-1), where p = s * (R_REF * C_REF)
        let k = 2.0 * sample_rate_hz.max(8000.0) * (R_REF * C_REF);
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

    #[inline(always)]
    pub fn process(&mut self, x: f64) -> f64 {
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
}
