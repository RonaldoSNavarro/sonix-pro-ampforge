//! Sub-modelo 3 (Spec 077): `CathodeFollowerStage` (`DC-Coupled 12AX7`)
//!
//! Ganho linear estritamente `< 1.0` (~0.94), baixa impedância de saída (`R_src ~ 1.3..1.5 kOhm`)
//! e compressão assimétrica suave apenas no semiciclo de condução grade-cátodo.

#[derive(Debug, Clone, Copy)]
pub struct CathodeFollowerStage {
    pub linear_gain: f64,
    pub r_src_ohms: f64,
    pub grid_knee: f64,
    adaa_prev: f64,
}

impl Default for CathodeFollowerStage {
    fn default() -> Self {
        Self::new(0.94, 1_300.0, 1.15)
    }
}

impl CathodeFollowerStage {
    pub fn new(linear_gain: f64, r_src_ohms: f64, grid_knee: f64) -> Self {
        Self {
            linear_gain: linear_gain.clamp(0.85, 0.99),
            r_src_ohms,
            grid_knee: grid_knee.max(0.2),
            adaa_prev: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.adaa_prev = 0.0;
    }

    #[inline(always)]
    fn cf_shape(u: f64, knee: f64) -> f64 {
        // Semiciclo positivo comprime suavemente (condução grade-cátodo); negativo é mais linear
        if u >= 0.0 {
            let k = u / knee;
            u / (1.0 + k * k * 0.35).sqrt()
        } else {
            let k = u / (knee * 2.2);
            u / (1.0 + k * k * 0.15).sqrt()
        }
    }

    #[inline]
    pub fn process(&mut self, x: f64, b_plus_factor: f64) -> f64 {
        let h = b_plus_factor.clamp(0.3, 2.0);
        let u = (x * self.linear_gain) / h;
        let y_mid = Self::cf_shape(0.5 * (u + self.adaa_prev), self.grid_knee);
        // ADAA1 trapezoidal leve para evitar aliasing em alto drive sem perder precisão DC
        let y = 0.5 * (Self::cf_shape(u, self.grid_knee) + y_mid);
        self.adaa_prev = u;
        y * h
    }
}
