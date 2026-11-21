//! One-Pole Parameter Smoother (RNF-006, CA-6)
//! Elimina zipper noise através de interpolação exponencial contínua por amostra.

#[derive(Debug, Clone)]
pub struct OnePoleSmoother {
    target: f64,
    current: f64,
    alpha: f64,
    sample_rate: f64,
    time_constant_sec: f64,
}

impl OnePoleSmoother {
    pub fn new(sample_rate: f64, time_constant_sec: f64, initial_val: f64) -> Self {
        let clamped_tc = time_constant_sec.clamp(0.005, 0.020); // 5ms a 20ms
        let alpha = 1.0 - (-1.0 / (sample_rate * clamped_tc)).exp();
        Self {
            target: initial_val,
            current: initial_val,
            alpha,
            sample_rate,
            time_constant_sec: clamped_tc,
        }
    }

    #[inline]
    pub fn set_target(&mut self, target: f64) {
        self.target = target;
    }

    #[inline]
    pub fn target(&self) -> f64 {
        self.target
    }

    #[inline]
    pub fn current_value(&self) -> f64 {
        self.current
    }

    #[inline]
    pub fn next_sample(&mut self) -> f64 {
        self.current += self.alpha * (self.target - self.current);
        self.current
    }

    #[inline]
    pub fn is_settled(&self) -> bool {
        (self.target - self.current).abs() < 1e-5
    }

    pub fn set_time_constant(&mut self, time_constant_sec: f64) {
        self.time_constant_sec = time_constant_sec.clamp(0.005, 0.020);
        self.alpha = 1.0 - (-1.0 / (self.sample_rate * self.time_constant_sec)).exp();
    }
}

/// Linear Parameter Smoother (20 ms ramp, Audição 2026-11-20 `AUD-11`).
/// Zero-alloc; `set_target` é idempotente quando o alvo não mudou.
#[derive(Debug, Clone, Copy)]
pub struct ParamSmoother {
    current: f64,
    target: f64,
    step: f64,
    remaining_samples: usize,
    sample_rate: f64,
    ramp_time_sec: f64,
}

impl ParamSmoother {
    pub fn new(sample_rate: f64, ramp_time_sec: f64, initial_val: f64) -> Self {
        let sr = if sample_rate.is_finite() && sample_rate > 1.0 {
            sample_rate
        } else {
            48_000.0
        };
        let ramp = if ramp_time_sec.is_finite() && ramp_time_sec > 0.0 {
            ramp_time_sec
        } else {
            0.020
        };
        let init = if initial_val.is_finite() {
            initial_val
        } else {
            0.0
        };
        Self {
            current: init,
            target: init,
            step: 0.0,
            remaining_samples: 0,
            sample_rate: sr,
            ramp_time_sec: ramp,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        if sample_rate.is_finite() && sample_rate > 1.0 {
            self.sample_rate = sample_rate;
        }
    }

    #[inline]
    pub fn reset(&mut self, value: f64) {
        if value.is_finite() {
            self.current = value;
            self.target = value;
            self.step = 0.0;
            self.remaining_samples = 0;
        }
    }

    #[inline]
    pub fn set_target(&mut self, target: f64) {
        if !target.is_finite() || (target - self.target).abs() <= 1e-9 {
            return;
        }
        self.target = target;
        if (target - self.current).abs() <= 1e-9 {
            self.current = target;
            self.step = 0.0;
            self.remaining_samples = 0;
            return;
        }
        let samples = (self.sample_rate * self.ramp_time_sec).round().max(1.0) as usize;
        self.remaining_samples = samples;
        self.step = (target - self.current) / (samples as f64);
    }

    #[inline]
    pub fn current(&self) -> f64 {
        self.current
    }

    #[inline]
    pub fn target(&self) -> f64 {
        self.target
    }

    #[inline]
    pub fn is_ramping(&self) -> bool {
        self.remaining_samples > 0
    }

    #[inline]
    pub fn next_sample(&mut self) -> f64 {
        if self.remaining_samples == 0 {
            return self.current;
        }
        self.current += self.step;
        self.remaining_samples -= 1;
        if self.remaining_samples == 0 {
            self.current = self.target;
            self.step = 0.0;
        }
        self.current
    }

    #[inline]
    pub fn advance(&mut self, samples: usize) -> f64 {
        if self.remaining_samples == 0 || samples == 0 {
            return self.current;
        }
        if samples >= self.remaining_samples {
            self.current = self.target;
            self.step = 0.0;
            self.remaining_samples = 0;
        } else {
            self.current += self.step * (samples as f64);
            self.remaining_samples -= samples;
        }
        self.current
    }
}
