//! crates/ampforge-dsp/src/fx/wah.rs
//! Modelagem comportamental Zero-Alloc de pedal Wah com indutor ressonante de 500 mH (Spec 052).
//!
//! Topologias disponíveis:
//! - [`WahMode::CryBabyGcb95`]: varredura logarítmica 350 Hz – 2200 Hz baseada em indutor Fasel 500 mH + capacitor 0.01 uF.
//! - [`WahMode::ClydeMcCoy`]: voicing vintage halo-inductor mais vocal e encorpado (300 Hz – 1900 Hz, Q acentuado).
//! - [`WahMode::AutoEnvelope`]: envelope follower dinâmico controlando a frequência de ressonância.

use std::f64::consts::PI;

/// Modos clássicos de voicing e controle do Wah (Spec 052 CA-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WahMode {
    #[default]
    CryBabyGcb95,
    ClydeMcCoy,
    AutoEnvelope,
}

impl WahMode {
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "crybaby_gcb95" | "gcb95" | "crybaby" | "default" => Some(Self::CryBabyGcb95),
            "clyde_mccoy" | "clyde" | "mccoy" | "vintage" => Some(Self::ClydeMcCoy),
            "auto_envelope" | "auto" | "envelope" => Some(Self::AutoEnvelope),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CryBabyGcb95 => "crybaby_gcb95",
            Self::ClydeMcCoy => "clyde_mccoy",
            Self::AutoEnvelope => "auto_envelope",
        }
    }
}

/// Processador Wah zero-alloc em tempo real baseado em filtro SVF trapezoidal (Cytomic TPT).
pub struct WahProcessor {
    sample_rate: f64,
    mode: WahMode,
    target_position: f32,
    pos_smoother: crate::smoother::ParamSmoother,
    q: f32,
    q_smoother: crate::smoother::ParamSmoother,
    has_processed: bool,
    bypassed: bool,
    // Estados do filtro TPT SVF em 64 bits (zero-alloc)
    ic1eq: f64,
    ic2eq: f64,
    // Envelope follower para WahMode::AutoEnvelope
    env_state: f64,
}

impl Default for WahProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl WahProcessor {
    pub fn new() -> Self {
        Self {
            sample_rate: 48_000.0,
            mode: WahMode::CryBabyGcb95,
            target_position: 0.5,
            pos_smoother: crate::smoother::ParamSmoother::new(48_000.0, 0.020, 0.5),
            q: 5.0,
            q_smoother: crate::smoother::ParamSmoother::new(48_000.0, 0.020, 5.0),
            has_processed: false,
            bypassed: false,
            ic1eq: 0.0,
            ic2eq: 0.0,
            env_state: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        if sr.is_finite() && sr > 1000.0 {
            self.sample_rate = (sr as f64).clamp(8_000.0, 384_000.0);
            self.pos_smoother.set_sample_rate(self.sample_rate);
            self.q_smoother.set_sample_rate(self.sample_rate);
        }
    }

    pub fn set_mode(&mut self, mode: WahMode) {
        self.mode = mode;
    }

    pub fn mode(&self) -> WahMode {
        self.mode
    }

    pub fn set_position(&mut self, pos: f32) {
        if pos.is_finite() {
            let clamped = pos.clamp(0.0, 1.0);
            self.target_position = clamped;
            if !self.has_processed {
                self.pos_smoother.reset(clamped as f64);
            } else {
                self.pos_smoother.set_target(clamped as f64);
            }
        }
    }

    pub fn set_q(&mut self, q: f32) {
        if q.is_finite() {
            let clamped = q.clamp(0.5, 12.0);
            self.q = clamped;
            if !self.has_processed {
                self.q_smoother.reset(clamped as f64);
            } else {
                self.q_smoother.set_target(clamped as f64);
            }
        }
    }

    pub fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    pub fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
        self.env_state = 0.0;
        self.has_processed = false;
        self.pos_smoother.reset(self.target_position as f64);
        self.q_smoother.reset(self.q as f64);
    }

    #[inline(always)]
    fn compute_svf_coeffs(
        mode: WahMode,
        pos: f64,
        base_q: f64,
        env_state: f64,
        sr: f64,
        max_cutoff: f64,
    ) -> (f64, f64, f64, f64, f64, f64) {
        let (f0_hz, q_eff, bp_gain, lp_bleed, dry_bleed) = match mode {
            WahMode::CryBabyGcb95 => {
                let f = 350.0 * (2200.0_f64 / 350.0_f64).powf(pos.clamp(0.0, 1.0));
                let q_val = 1.6 + 0.65 * base_q;
                (f, q_val, 0.38, 0.0, 0.06)
            }
            WahMode::ClydeMcCoy => {
                let f = 300.0 * (1900.0_f64 / 300.0_f64).powf(pos.clamp(0.0, 1.0));
                let q_val = 1.9 + 0.80 * base_q;
                (f, q_val, 0.34, 0.14, 0.08)
            }
            WahMode::AutoEnvelope => {
                let pos_dyn = (pos * 0.45 + env_state * 3.8).clamp(0.0, 1.0);
                let f = 350.0 * (2200.0_f64 / 350.0_f64).powf(pos_dyn);
                let q_val = 1.8 + 0.70 * base_q;
                (f, q_val, 0.40, 0.04, 0.08)
            }
        };
        let cutoff = f0_hz.clamp(80.0, max_cutoff);
        let g = (PI * cutoff / sr).tan();
        let k = 1.0 / q_eff.max(0.5);
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        (a1, a2, a3, bp_gain, lp_bleed, dry_bleed)
    }

    #[inline(always)]
    fn process_sample_with_coeffs(
        &mut self,
        x: f64,
        a1: f64,
        a2: f64,
        a3: f64,
        bp_gain: f64,
        lp_bleed: f64,
        dry_bleed: f64,
    ) -> f32 {
        let v3 = x - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;

        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        if self.ic1eq.abs() < 1e-24 {
            self.ic1eq = 0.0;
        }
        if self.ic2eq.abs() < 1e-24 {
            self.ic2eq = 0.0;
        }

        let tank = bp_gain * v1 + lp_bleed * v2;
        let sat_tank = 2.8 * ((tank + 0.015 * tank * tank) / 2.8).tanh();
        (sat_tank + dry_bleed * x) as f32
    }

    /// Processa bloco mono sem alocações na heap.
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let len = input.len().min(output.len());
        if len == 0 {
            return;
        }
        if !self.has_processed {
            self.pos_smoother.reset(self.target_position as f64);
            self.q_smoother.reset(self.q as f64);
            self.has_processed = true;
        } else {
            self.pos_smoother.set_target(self.target_position as f64);
            self.q_smoother.set_target(self.q as f64);
        }

        if self.bypassed {
            self.pos_smoother.advance(len);
            self.q_smoother.advance(len);
            output[..len].copy_from_slice(&input[..len]);
            return;
        }

        let sr = self.sample_rate.max(8_000.0);
        let max_cutoff = (sr * 0.42).min(6_500.0);

        if self.mode == WahMode::AutoEnvelope {
            let att_coeff = 1.0 - (-1.0 / (0.004 * sr)).exp();
            let rel_coeff = 1.0 - (-1.0 / (0.085 * sr)).exp();

            for i in 0..len {
                let x = if input[i].is_finite() { input[i] as f64 } else { 0.0 };
                let pos_s = self.pos_smoother.next_sample();
                let base_q = self.q_smoother.next_sample().clamp(0.5, 12.0);

                let abs_x = x.abs();
                let env_c = if abs_x > self.env_state { att_coeff } else { rel_coeff };
                self.env_state += env_c * (abs_x - self.env_state);
                if self.env_state < 1e-24 {
                    self.env_state = 0.0;
                }

                let (a1, a2, a3, bp_gain, lp_bleed, dry_bleed) =
                    Self::compute_svf_coeffs(self.mode, pos_s, base_q, self.env_state, sr, max_cutoff);
                output[i] = self.process_sample_with_coeffs(
                    x, a1, a2, a3, bp_gain, lp_bleed, dry_bleed,
                );
            }
            return;
        }

        if !self.pos_smoother.is_ramping() && !self.q_smoother.is_ramping() {
            let (a1, a2, a3, bp_gain, lp_bleed, dry_bleed) = Self::compute_svf_coeffs(
                self.mode,
                self.pos_smoother.current(),
                self.q_smoother.current().clamp(0.5, 12.0),
                0.0,
                sr,
                max_cutoff,
            );
            for i in 0..len {
                let x = if input[i].is_finite() { input[i] as f64 } else { 0.0 };
                output[i] = self.process_sample_with_coeffs(
                    x, a1, a2, a3, bp_gain, lp_bleed, dry_bleed,
                );
            }
            return;
        }

        for i in 0..len {
            let pos_s = self.pos_smoother.next_sample();
            let base_q = self.q_smoother.next_sample().clamp(0.5, 12.0);
            let (a1, a2, a3, bp_gain, lp_bleed, dry_bleed) =
                Self::compute_svf_coeffs(self.mode, pos_s, base_q, 0.0, sr, max_cutoff);
            let raw = input[i];
            let x = if raw.is_finite() { raw as f64 } else { 0.0 };
            output[i] =
                self.process_sample_with_coeffs(x, a1, a2, a3, bp_gain, lp_bleed, dry_bleed);
        }
    }
}
