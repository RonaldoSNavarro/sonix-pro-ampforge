//! Sub-modelo 1 (Spec 077): `TriodeStage`
//!
//! Modela estágio triodo (`12AX7`, `12AT7`, `12AU7`, `12AY7`) com:
//! - Shelf real de cátodo `Rk || Ck` (atenuação de sub-graves abaixo de `1/(2*pi*Rk*Ck)`);
//! - Condução de grade + blocking distortion com recuperação `tau_grid = R_grid_leak * Cc`;
//! - Saturação assimétrica ADAA1 escalada dinamicamente por `B+(t) / B+_nom`;
//! - Polo passa-baixas de capacitância Miller `C_miller`.

use crate::denormal::flush_f64;
use crate::modshop::PreampTube;
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct TriodeParams {
    pub tube: PreampTube,
    pub rp_ohms: f64,
    pub rk_ohms: f64,
    pub ck_farads: f64,
    pub cc_farads: f64,
    pub r_grid_leak_ohms: f64,
    pub c_miller_hz: f64,
    pub gain_linear: f64,
    pub asym: f64,
    pub nominal_b_plus_v: f64,
}

impl Default for TriodeParams {
    fn default() -> Self {
        Self {
            tube: PreampTube::Ax7A,
            rp_ohms: 100_000.0,
            rk_ohms: 1_500.0,
            ck_farads: 25e-6,
            cc_farads: 22e-9,
            r_grid_leak_ohms: 1_000_000.0,
            c_miller_hz: 7_500.0,
            gain_linear: 1.0,
            asym: 0.16,
            nominal_b_plus_v: 250.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TriodeStage {
    params: TriodeParams,
    sr: f64,
    // Shelf de cátodo Rk || Ck (1ª ordem bilinear)
    ck_b0: f64,
    ck_b1: f64,
    ck_a1: f64,
    ck_x1: f64,
    ck_y1: f64,
    // Condução de grade / blocking bias shift
    grid_bias_shift: f64,
    alpha_grid_att: f64,
    alpha_grid_rel: f64,
    // Filtro Miller LP
    alpha_miller: f64,
    miller_state: f64,
    // Estado ADAA1
    adaa_prev: f64,
}

impl TriodeStage {
    pub fn new(params: TriodeParams, sample_rate_hz: f64) -> Self {
        let mut s = Self {
            params,
            sr: sample_rate_hz.max(8000.0),
            ck_b0: 1.0,
            ck_b1: 0.0,
            ck_a1: 0.0,
            ck_x1: 0.0,
            ck_y1: 0.0,
            grid_bias_shift: 0.0,
            alpha_grid_att: 0.01,
            alpha_grid_rel: 0.001,
            alpha_miller: 0.5,
            miller_state: 0.0,
            adaa_prev: 0.0,
        };
        s.update_coeffs(sample_rate_hz);
        s
    }

    pub fn set_params(&mut self, params: TriodeParams) {
        self.params = params;
        self.update_coeffs(self.sr);
    }

    pub fn update_coeffs(&mut self, sample_rate_hz: f64) {
        let fs = sample_rate_hz.max(8000.0);
        self.sr = fs;

        // Shelf de cátodo: Rk || Ck -> ganho DC ~ 1/(1 + gm*Rk), ganho HF = 1.0
        // Zero em f_z = 1 / (2*pi*Rk*Ck), Polo em f_p = (1 + gm*Rk) / (2*pi*Rk*Ck)
        if self.params.ck_farads > 1e-12 && self.params.rk_ohms > 1.0 {
            let gm = 1.6e-3; // transcondutância típica 12AX7
            let deg = 1.0 + gm * self.params.rk_ohms;
            let f_z = 1.0 / (2.0 * PI * self.params.rk_ohms * self.params.ck_farads);
            let f_p = (deg * f_z).min(fs * 0.45);
            let k = 2.0 * fs;
            let wz = 2.0 * PI * f_z;
            let wp = 2.0 * PI * f_p;
            let a0 = k + wp;
            self.ck_b0 = (k + wz) / a0;
            self.ck_b1 = (wz - k) / a0;
            self.ck_a1 = (wp - k) / a0;
        } else {
            // Cátodo sem bypass (ex: cold clipper 10k / 39k): atenuação moderada preservando excitação de saturação
            let deg = (1.0 / (1.0 + 1.6e-3 * self.params.rk_ohms * 0.05)).max(0.45);
            self.ck_b0 = deg;
            self.ck_b1 = 0.0;
            self.ck_a1 = 0.0;
        }

        // Constante de tempo de blocking distortion: tau_grid = R_grid_leak * Cc
        let tau_rel = (self.params.r_grid_leak_ohms * self.params.cc_farads).clamp(0.002, 0.25);
        self.alpha_grid_att = (1.0 - (-1.0 / (fs * 0.0005)).exp()).clamp(0.001, 0.99);
        self.alpha_grid_rel = (1.0 - (-1.0 / (fs * tau_rel)).exp()).clamp(0.00001, 0.5);

        let fc_m = self.params.c_miller_hz.clamp(500.0, fs * 0.45);
        self.alpha_miller = (1.0 - (-2.0 * PI * fc_m / fs).exp()).clamp(0.001, 0.99);
    }

    pub fn reset(&mut self) {
        self.ck_x1 = 0.0;
        self.ck_y1 = 0.0;
        self.grid_bias_shift = 0.0;
        self.miller_state = 0.0;
        self.adaa_prev = 0.0;
    }

    #[inline(always)]
    fn adaa_triode(x: f64, x_prev: &mut f64, kp: f64, kn: f64) -> f64 {
        let anti = |u: f64| -> f64 {
            let k = if u >= 0.0 { kp } else { kn };
            let ku = k * u;
            ((1.0 + ku * ku).sqrt() - 1.0) / (k * k)
        };
        let eval = |u: f64| -> f64 {
            let k = if u >= 0.0 { kp } else { kn };
            u / (1.0 + (k * u) * (k * u)).sqrt()
        };
        let dx = x - *x_prev;
        let y = if dx.abs() < 1e-6 {
            eval(0.5 * (x + *x_prev))
        } else {
            (anti(x) - anti(*x_prev)) / dx
        };
        *x_prev = x;
        y
    }

    #[inline]
    pub fn process(&mut self, x: f64, b_plus_v: f64) -> f64 {
        let (tube_gain, knee) = match self.params.tube {
            PreampTube::Ax7A => (1.0, 1.0),
            PreampTube::At7 => (0.58, 0.85),
            PreampTube::Ay7 => (0.45, 0.78),
            PreampTube::Ef86 => (1.22, 1.35),
        };

        // 1. Shelf de cátodo Rk || Ck
        let x_ck = flush_f64(self.ck_b0 * x + self.ck_b1 * self.ck_x1 - self.ck_a1 * self.ck_y1);
        self.ck_x1 = flush_f64(x);
        self.ck_y1 = x_ck;

        // 2. Condução de grade e blocking bias shift
        let driven = x_ck * self.params.gain_linear * tube_gain;
        let headroom = (b_plus_v / self.params.nominal_b_plus_v.max(50.0)).clamp(0.25, 2.0);
        let grid_thresh = 0.65 * headroom;
        let excess = (driven - grid_thresh).max(0.0);
        let target_shift = (excess * 0.28).min(0.45 * headroom);
        let a_g = if target_shift > self.grid_bias_shift {
            self.alpha_grid_att
        } else {
            self.alpha_grid_rel
        };
        self.grid_bias_shift = flush_f64(self.grid_bias_shift + a_g * (target_shift - self.grid_bias_shift));

        let shifted = driven - self.grid_bias_shift;

        // 3. Saturação assimétrica ADAA1 escalada por B+
        let kp = (1.0 + 1.2 * self.params.asym) * knee;
        let kn = ((1.0 - 0.55 * self.params.asym) * knee).max(0.35);
        let sat = Self::adaa_triode(shifted / headroom, &mut self.adaa_prev, kp, kn) * headroom;

        // 4. Filtro passa-baixas Miller
        self.miller_state = flush_f64(self.miller_state + self.alpha_miller * (sat - self.miller_state));
        self.miller_state
    }
}
