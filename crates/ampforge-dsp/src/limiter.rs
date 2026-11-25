//! Limitador de saída zero-latency com joelho suave acima de -1.0 dBFS (Spec 093 CA-6).

/// Threshold linear correspondente a `-1.0 dBFS` (`10^(-1/20)`).
const SOFT_KNEE_THRESHOLD: f32 = 0.891_250_94;
/// Headroom restante até `0.0 dBFS` (`1.0 - SOFT_KNEE_THRESHOLD`).
const SOFT_KNEE_HEADROOM: f32 = 1.0 - SOFT_KNEE_THRESHOLD;

/// Limitador de saída transparente abaixo de `-1.0 dBFS` com curva `tanh` contínua ($C^1$)
/// até o teto rígido `[-1.0, 1.0]` e sanitização de `NaN`/`Inf`.
#[derive(Debug, Clone, Copy, Default)]
pub struct OutputSoftLimiter;

impl OutputSoftLimiter {
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    /// Processa uma amostra sem latência algorítmica e sem alocações.
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        if !x.is_finite() {
            return 0.0;
        }
        let abs_x = x.abs();
        if abs_x <= SOFT_KNEE_THRESHOLD {
            return x;
        }
        let excess = abs_x - SOFT_KNEE_THRESHOLD;
        let compressed = SOFT_KNEE_THRESHOLD + SOFT_KNEE_HEADROOM * (excess / SOFT_KNEE_HEADROOM).tanh();
        compressed.min(1.0).copysign(x)
    }

    /// Processa um slice in-place. Retorna `true` se houve ao menos uma amostra não-finita sanitizada.
    #[inline]
    pub fn process_slice(&mut self, samples: &mut [f32]) -> bool {
        let mut had_non_finite = false;
        for s in samples.iter_mut() {
            let v = *s;
            if !v.is_finite() {
                *s = 0.0;
                had_non_finite = true;
            } else {
                *s = self.process_sample(v);
            }
        }
        had_non_finite
    }
}
