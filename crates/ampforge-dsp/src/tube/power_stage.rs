//! Sub-modelo 5 (Spec 077): `PushPullPowerStage` (`EL34`, `6L6GC`, `EL84`, `KT88`, `6V6GT`)
//!
//! Opera com ADAA1 completo tanto no ramo principal quanto no ramo linearizado por NFB
//! (eliminando `soft_rail` sem anti-aliasing).

use crate::modshop::PowerTube;

#[derive(Debug, Clone, Copy)]
pub struct PushPullPowerStage {
    pub tube: PowerTube,
    pub bias_ma: f64,
    adaa_prev: f64,
    adaa_lin_prev: f64,
}

impl PushPullPowerStage {
    pub fn new(tube: PowerTube, bias_ma: f64) -> Self {
        Self {
            tube,
            bias_ma: bias_ma.clamp(15.0, 60.0),
            adaa_prev: 0.0,
            adaa_lin_prev: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.adaa_prev = 0.0;
        self.adaa_lin_prev = 0.0;
    }

    #[inline]
    pub fn nominal_headroom(&self) -> f64 {
        match self.tube {
            PowerTube::Kt88 => 1.35,
            PowerTube::Tube6L6Gc => 1.25,
            PowerTube::El34 => 1.00,
            PowerTube::Tube6V6 => 0.85,
            PowerTube::El84 => 0.78,
        }
    }

    #[inline(always)]
    fn adaa1_tanh(u: f64, prev: &mut f64) -> f64 {
        let dx = u - *prev;
        let y = if dx.abs() < 1e-6 {
            (0.5 * (u + *prev)).tanh()
        } else {
            let f1 = if u.abs() > 20.0 { u.abs() - std::f64::consts::LN_2 } else { u.cosh().ln() };
            let f0 = if prev.abs() > 20.0 {
                prev.abs() - std::f64::consts::LN_2
            } else {
                prev.cosh().ln()
            };
            (f1 - f0) / dx
        };
        *prev = u;
        y
    }

    #[inline]
    pub fn process(&mut self, x: f64, b_plus_factor: f64, nfb_linearization: f64) -> f64 {
        let headroom = (self.nominal_headroom() * b_plus_factor).max(0.2);

        // Crossover notch dependente de bias frio (bias_ma < 32 mA)
        let bias_deficit = ((32.0 - self.bias_ma) / 12.0).clamp(0.0, 1.0);
        let u_raw = x / headroom;
        let u_notch = if bias_deficit > 0.01 {
            let notch = 0.025 * bias_deficit;
            u_raw - notch * u_raw / (1.0 + 400.0 * u_raw * u_raw)
        } else {
            u_raw
        };

        let sat_ol = Self::adaa1_tanh(u_notch, &mut self.adaa_prev);
        let sat_lin = Self::adaa1_tanh(u_raw * 0.6, &mut self.adaa_lin_prev) / 0.6;
        let lin = nfb_linearization.clamp(0.0, 1.0);
        (sat_lin + (sat_ol - sat_lin) * lin) * headroom
    }
}
