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
