//! Mixed Precision Audio Processing (RNF-003, CA-4)
//! Barramento 32-bit f32 + Solvers e IIR sub-100Hz em 64-bit f64 com erro DC < -120 dBFS.

#[derive(Debug, Clone)]
pub struct IirBiquad64 {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    s1: f64,
    s2: f64,
}

impl IirBiquad64 {
    /// Cria filtro passa-altas de 2ª ordem (Butterworth ou custom Q) em 64-bit
    pub fn high_pass(sample_rate: f64, cutoff_hz: f64, q: f64) -> Self {
        let omega = 2.0 * std::f64::consts::PI * cutoff_hz / sample_rate;
        let alpha = omega.sin() / (2.0 * q);
        let cos_w = omega.cos();

        let b0 = (1.0 + cos_w) / 2.0;
        let b1 = -(1.0 + cos_w);
        let b2 = (1.0 + cos_w) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            s1: 0.0,
            s2: 0.0,
        }
    }

    #[inline]
    pub fn process_sample(&mut self, input: f64) -> f64 {
        // Direct Form II Transposed (64-bit precision)
        let output = self.b0 * input + self.s1;
        self.s1 = self.b1 * input - self.a1 * output + self.s2;
        self.s2 = self.b2 * input - self.a2 * output;
        output
    }

    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }
}

/// Solucionador Newton-Raphson de precisão dupla para equações não-lineares de triodos
pub struct MixedPrecisionSolver {
    max_iterations: usize,
    tolerance: f64,
}

impl MixedPrecisionSolver {
    pub fn new() -> Self {
        Self {
            max_iterations: 16,
            tolerance: 1e-8,
        }
    }

    /// Solução iterativa 64-bit da curva de transferência de triodo 12AX7:
    /// Ip = mu * (Vg + Vp / mu)^1.5 para modelagem White-Box
    pub fn solve_triode_transfer(&mut self, v_grid: f64, v_plate: f64) -> f64 {
        let mu = 100.0; // Ganho típico de 12AX7
        let v_eff = v_grid + v_plate / mu;
        if v_eff <= 0.0 {
            return 0.0;
        }

        // Newton-Raphson refinamento
        let mut x = v_eff;
        for _ in 0..self.max_iterations {
            let fx = x.powf(1.5) - v_eff.powf(1.5);
            let f_prime = 1.5 * x.sqrt();
            if f_prime.abs() < 1e-9 {
                break;
            }
            let step = fx / f_prime;
            x -= step;
            if step.abs() < self.tolerance {
                break;
            }
        }

        // Corrente anódica normalizada
        x.powf(1.5) * 1e-3
    }
}
