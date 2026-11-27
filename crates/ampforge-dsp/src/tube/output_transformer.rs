//! Sub-modelo 9 (Spec 077): `OutputTransformer`
//!
//! Saturação magnética de núcleo ferromagnético dependente do **fluxo magnético**
//! $\Phi(t) = \int v(t)\,dt$ via leaky integrator na mesma escala do sinal e
//! permeabilidade saturante $\mu(\Phi) = 1 / \sqrt{1 + (\Phi/\Phi_{\text{sat}})^2}$,
//! garantindo $\text{THD}(80\text{ Hz}) \ge 2.5\times \text{THD}(2000\text{ Hz})$
//! **sem inversão de polaridade** nem engolimento de palm mute!

use crate::denormal::flush_f64;
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputTransformerModel {
    DagnallC1998,
    Drake1203,
    SchumacherTwin,
    Mesa562003,
    WodenAc30,
}

#[derive(Debug, Clone, Copy)]
pub struct OutputTransformer {
    pub model: OutputTransformerModel,
    pub sat_amount: f64,
    sr: f64,
    flux_state: f64,
    flux_alpha: f64,
    flux_norm: f64,
    lp_state: f64,
    alpha_lp: f64,
    hp_x1: f64,
    hp_y1: f64,
    alpha_hp: f64,
}

impl OutputTransformer {
    pub fn new(model: OutputTransformerModel, sample_rate_hz: f64) -> Self {
        let mut s = Self {
            model,
            sat_amount: 0.5,
            sr: sample_rate_hz.max(8000.0),
            flux_state: 0.0,
            flux_alpha: 0.99,
            flux_norm: 1.0,
            lp_state: 0.0,
            alpha_lp: 0.5,
            hp_x1: 0.0,
            hp_y1: 0.0,
            alpha_hp: 0.995,
        };
        s.set_sample_rate(sample_rate_hz);
        s
    }

    pub fn set_sample_rate(&mut self, sample_rate_hz: f64) {
        let fs = sample_rate_hz.max(8000.0);
        self.sr = fs;

        // Polo do integrador com fuga em ~25 Hz: abaixo disso o fluxo estabiliza;
        // entre 40 Hz e 4 kHz a amplitude de fluxo escala naturalmente com 1/f.
        let w_leak = 2.0 * PI * 25.0;
        self.flux_alpha = (-w_leak / fs).exp().clamp(0.8, 0.9999);
        // Normaliza o fluxo para que em 80 Hz / amplitude 0.707 o núcleo entre em região não-linear suave
        self.flux_norm = (2.0 * PI * 140.0) / fs;

        let (fc_hp, fc_lp): (f64, f64) = match self.model {
            OutputTransformerModel::DagnallC1998 => (36.0, 12_500.0),
            OutputTransformerModel::Drake1203 => (40.0, 11_800.0),
            OutputTransformerModel::SchumacherTwin => (28.0, 16_000.0),
            OutputTransformerModel::Mesa562003 => (32.0, 13_500.0),
            OutputTransformerModel::WodenAc30 => (48.0, 14_500.0),
        };
        self.alpha_lp = (1.0 - (-2.0 * PI * fc_lp.min(fs * 0.45) / fs).exp()).clamp(0.05, 0.99);
        self.alpha_hp = (1.0 / (1.0 + 2.0 * PI * fc_hp / fs)).clamp(0.90, 0.9999);
    }

    pub fn reset(&mut self) {
        self.flux_state = 0.0;
        self.lp_state = 0.0;
        self.hp_x1 = 0.0;
        self.hp_y1 = 0.0;
    }

    /// Processa a tensão primária mantendo estritamente a mesma polaridade do sinal de entrada.
    #[inline]
    pub fn process(&mut self, v_in: f64) -> f64 {
        // 1. HPF de indutância de magnetização primária L_p (remove DC sem inverter fase)
        let hp = flush_f64(self.alpha_hp * (self.hp_y1 + v_in - self.hp_x1));
        self.hp_x1 = flush_f64(v_in);
        self.hp_y1 = hp;

        // 2. Fluxo magnético Phi(t) = int v(t) dt (leaky integrator -> |Phi| proporcional a V/f!)
        self.flux_state = flush_f64(self.flux_alpha * self.flux_state + self.flux_norm * hp);

        // 3. Permeabilidade saturante mu(Phi) in (0, 1] — modula o ganho de forma par em Phi,
        // preservando 100% do sinal de v_in (zero inversão de polaridade!) e gerando 3º harmônico
        // proporcional a V/f: THD(80 Hz) >> THD(2000 Hz).
        let drive = self.sat_amount.clamp(0.0, 2.0);
        let phi_sat_inv = 0.85 * drive;
        let phi_scaled = self.flux_state * phi_sat_inv;
        let mu = 1.0 / (1.0 + phi_scaled * phi_scaled).sqrt();
        let core_out = hp * (0.25 + 0.75 * mu);

        // 4. LPF de indutância de dispersão L_leak
        self.lp_state = flush_f64(self.lp_state + self.alpha_lp * (core_out - self.lp_state));
        self.lp_state
    }
}
