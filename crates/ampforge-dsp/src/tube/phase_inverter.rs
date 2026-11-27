//! Sub-modelo 4 (Spec 077): `PhaseInverterLtpStage` (`Long-Tailed Pair` 12AX7 / 12AT7)

use crate::modshop::PreampTube;

#[derive(Debug, Clone, Copy)]
pub struct PhaseInverterLtpStage {
    pub tube: PreampTube,
    pub plate_imbalance: f64,
    adaa_prev: f64,
}

impl Default for PhaseInverterLtpStage {
    fn default() -> Self {
        Self::new(PreampTube::Ax7A, 0.06)
    }
}

impl PhaseInverterLtpStage {
    pub fn new(tube: PreampTube, plate_imbalance: f64) -> Self {
        Self {
            tube,
            plate_imbalance: plate_imbalance.clamp(0.0, 0.25),
            adaa_prev: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.adaa_prev = 0.0;
    }

    #[inline]
    pub fn process(&mut self, signal: f64, nfb_feedback: f64, b_plus_factor: f64) -> f64 {
        let headroom = match self.tube {
            PreampTube::At7 => 1.35 * b_plus_factor,
            PreampTube::Ay7 => 1.15 * b_plus_factor,
            _ => 1.0 * b_plus_factor,
        }
        .max(0.25);

        let diff = (signal - nfb_feedback) / headroom;
        let dx = diff - self.adaa_prev;
        let sat = if dx.abs() < 1e-6 {
            (0.5 * (diff + self.adaa_prev)).tanh()
        } else {
            let f1 = if diff.abs() > 20.0 {
                diff.abs() - std::f64::consts::LN_2
            } else {
                diff.cosh().ln()
            };
            let f0 = if self.adaa_prev.abs() > 20.0 {
                self.adaa_prev.abs() - std::f64::consts::LN_2
            } else {
                self.adaa_prev.cosh().ln()
            };
            (f1 - f0) / dx
        };
        self.adaa_prev = diff;

        // Desbalanceamento sutil de resistores de placa (82k / 100k) introduz leve H2
        let asym = sat + self.plate_imbalance * sat * sat;
        asym * headroom
    }
}
