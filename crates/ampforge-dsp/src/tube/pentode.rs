//! Sub-modelo 2 (Spec 077): `PentodePreampStage` (`EF86`)

use crate::denormal::flush_f64;

#[derive(Debug, Clone, Copy)]
pub struct PentodePreampStage {
    pub screen_v_norm: f64,
    pub gain: f64,
    adaa_prev: f64,
    lp_state: f64,
    alpha_lp: f64,
}

impl PentodePreampStage {
    pub fn new(sample_rate_hz: f64, gain: f64) -> Self {
        let fs = sample_rate_hz.max(8000.0);
        let alpha_lp = (1.0 - (-2.0 * std::f64::consts::PI * 9000.0 / fs).exp()).clamp(0.01, 0.99);
        Self {
            screen_v_norm: 1.0,
            gain,
            adaa_prev: 0.0,
            lp_state: 0.0,
            alpha_lp,
        }
    }

    pub fn reset(&mut self) {
        self.adaa_prev = 0.0;
        self.lp_state = 0.0;
    }

    #[inline]
    pub fn process(&mut self, x: f64, b_plus_factor: f64) -> f64 {
        let h = (b_plus_factor * self.screen_v_norm).clamp(0.25, 2.0);
        let u = (x * self.gain * 1.25) / h;
        let dx = u - self.adaa_prev;
        let y = if dx.abs() < 1e-6 {
            (0.5 * (u + self.adaa_prev)).tanh()
        } else {
            let f1 = if u.abs() > 20.0 { u.abs() - std::f64::consts::LN_2 } else { u.cosh().ln() };
            let f0 = if self.adaa_prev.abs() > 20.0 {
                self.adaa_prev.abs() - std::f64::consts::LN_2
            } else {
                self.adaa_prev.cosh().ln()
            };
            (f1 - f0) / dx
        };
        self.adaa_prev = u;
        self.lp_state = flush_f64(self.lp_state + self.alpha_lp * (y * h - self.lp_state));
        self.lp_state
    }
}
