//! Sub-modelo 6 (Spec 077): `BiasTopology` (`FixedBias` vs `CathodeBias`)
//!
//! No modo `CathodeBias` (ex.: Vox AC30 `50 Ω || 250 µF`), sinais fortes deslocam
//! dinamicamente o ponto de polarização (bias excursion), produzindo compressão orgânica.

use crate::denormal::flush_f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiasMode {
    FixedBias,
    CathodeBias,
}

#[derive(Debug, Clone, Copy)]
pub struct BiasTopology {
    pub mode: BiasMode,
    pub rk_ohms: f64,
    pub ck_farads: f64,
    excursion_v: f64,
    alpha_att: f64,
    alpha_rel: f64,
}

impl BiasTopology {
    pub fn fixed() -> Self {
        Self {
            mode: BiasMode::FixedBias,
            rk_ohms: 0.0,
            ck_farads: 0.0,
            excursion_v: 0.0,
            alpha_att: 0.01,
            alpha_rel: 0.001,
        }
    }

    pub fn cathode_biased(rk_ohms: f64, ck_farads: f64, sample_rate_hz: f64) -> Self {
        let mut s = Self {
            mode: BiasMode::CathodeBias,
            rk_ohms,
            ck_farads,
            excursion_v: 0.0,
            alpha_att: 0.01,
            alpha_rel: 0.001,
        };
        s.set_sample_rate(sample_rate_hz);
        s
    }

    pub fn set_sample_rate(&mut self, sample_rate_hz: f64) {
        let fs = sample_rate_hz.max(8000.0);
        let tau = (self.rk_ohms * self.ck_farads).clamp(0.005, 0.25);
        self.alpha_att = (1.0 - (-1.0 / (fs * 0.008)).exp()).clamp(0.0005, 0.5);
        self.alpha_rel = (1.0 - (-1.0 / (fs * tau)).exp()).clamp(0.0001, 0.2);
    }

    pub fn reset(&mut self) {
        self.excursion_v = 0.0;
    }

    #[inline]
    pub fn excursion(&self) -> f64 {
        self.excursion_v
    }

    /// Atualiza o deslocamento de bias e retorna o fator de headroom instantâneo.
    #[inline]
    pub fn process_excursion(&mut self, signal: f64) -> f64 {
        if self.mode == BiasMode::FixedBias {
            return 1.0;
        }
        let target = (signal.abs() * 0.24).min(0.32);
        let alpha = if target > self.excursion_v {
            self.alpha_att
        } else {
            self.alpha_rel
        };
        self.excursion_v = flush_f64(self.excursion_v + alpha * (target - self.excursion_v));
        (1.0 - self.excursion_v).max(0.60)
    }
}
