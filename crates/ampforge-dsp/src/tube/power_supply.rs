//! Sub-modelos 7 e 8 (Spec 077): `PowerSupplyAndRectifier` e `PowerSupplySag`
//!
//! Modela `SolidStateSilicon` (queda <= 5%), `TubeGz34` e `Tube5U4G` (12%..22% sag
//! sob burst forte e recuperação suave sem overshoot).

use crate::denormal::flush_f64;
use crate::modshop::RectifierType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RectifierKind {
    SolidStateSilicon,
    TubeGz34,
    Tube5U4G,
}

impl From<RectifierType> for RectifierKind {
    fn from(r: RectifierType) -> Self {
        match r {
            RectifierType::SolidStateDiode => Self::SolidStateSilicon,
            RectifierType::Gz34_5Ar4 => Self::TubeGz34,
            RectifierType::Dual5U4Gb => Self::Tube5U4G,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PowerSupplyAndRectifier {
    pub kind: RectifierKind,
    pub nominal_b_plus_v: f64,
    pub variac_scale: f64,
}

impl PowerSupplyAndRectifier {
    pub fn new(kind: RectifierKind, nominal_b_plus_v: f64) -> Self {
        Self {
            kind,
            nominal_b_plus_v: nominal_b_plus_v.clamp(250.0, 550.0),
            variac_scale: 1.0,
        }
    }

    #[inline]
    pub fn r_psu_ohms(&self) -> f64 {
        match self.kind {
            RectifierKind::SolidStateSilicon => 25.0,
            RectifierKind::TubeGz34 => 110.0,
            RectifierKind::Tube5U4G => 180.0,
        }
    }

    /// Fator de queda máxima proporcional à resistência interna da retificadora.
    #[inline]
    pub fn max_sag_ratio(&self) -> f64 {
        match self.kind {
            RectifierKind::SolidStateSilicon => 0.045,
            RectifierKind::TubeGz34 => 0.145,
            RectifierKind::Tube5U4G => 0.215,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PowerSupplySag {
    pub supply: PowerSupplyAndRectifier,
    sag_drop: f64,
    alpha_att: f64,
    alpha_rel: f64,
}

impl PowerSupplySag {
    pub fn new(supply: PowerSupplyAndRectifier, sample_rate_hz: f64) -> Self {
        let mut s = Self {
            supply,
            sag_drop: 0.0,
            alpha_att: 0.01,
            alpha_rel: 0.001,
        };
        s.set_sample_rate(sample_rate_hz);
        s
    }

    pub fn set_kind(&mut self, kind: RectifierKind, sample_rate_hz: f64) {
        self.supply.kind = kind;
        self.set_sample_rate(sample_rate_hz);
    }

    pub fn set_sample_rate(&mut self, sample_rate_hz: f64) {
        let fs = sample_rate_hz.max(8000.0);
        let (tau_att, tau_rel) = match self.supply.kind {
            RectifierKind::SolidStateSilicon => (0.008, 0.035),
            RectifierKind::TubeGz34 => (0.025, 0.120),
            RectifierKind::Tube5U4G => (0.035, 0.220),
        };
        self.alpha_att = (1.0 - (-1.0 / (fs * tau_att)).exp()).clamp(0.0001, 0.9);
        self.alpha_rel = (1.0 - (-1.0 / (fs * tau_rel)).exp()).clamp(0.00001, 0.5);
    }

    pub fn reset(&mut self) {
        self.sag_drop = 0.0;
    }

    #[inline]
    pub fn sag_drop_ratio(&self) -> f64 {
        self.sag_drop
    }

    #[inline]
    pub fn current_b_plus_v(&self) -> f64 {
        self.supply.nominal_b_plus_v * (1.0 - self.sag_drop) * self.supply.variac_scale
    }

    #[inline]
    pub fn b_plus_factor(&self) -> f64 {
        ((1.0 - self.sag_drop) * self.supply.variac_scale).clamp(0.35, 1.25)
    }

    /// Atualiza a queda dinâmica de tensão B+ conforme a corrente demandada pelo power stage.
    #[inline]
    pub fn step(&mut self, load_signal: f64, sag_amount_0_10: f64) -> f64 {
        let amt = (sag_amount_0_10 / 10.0).clamp(0.0, 1.0);
        let drive = (load_signal.abs() * 1.4).min(1.0);
        let target = amt * self.supply.max_sag_ratio() * drive;
        let alpha = if target > self.sag_drop {
            self.alpha_att
        } else {
            self.alpha_rel
        };
        self.sag_drop = flush_f64((self.sag_drop + alpha * (target - self.sag_drop)).clamp(0.0, 0.40));
        self.b_plus_factor()
    }
}
