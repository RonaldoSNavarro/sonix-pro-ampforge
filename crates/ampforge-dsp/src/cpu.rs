//! Real-Time CPU Meter & Overload Protection (RNF-007, CA-8, CA-9)
//! Mede o tempo de processamento por bloco e ativa proteção contra xruns em >= 88.0%.

use crate::config::{BufferSize, SampleRate};

pub struct CpuMonitor {
    sample_rate: SampleRate,
    buffer_size: BufferSize,
    deadline_micros: u64,
    current_usage_pct: f32,
    warning_flag: bool,
}

impl CpuMonitor {
    pub const OVERLOAD_THRESHOLD_PCT: f32 = 88.0;

    pub fn new(sample_rate: SampleRate, buffer_size: BufferSize) -> Self {
        let deadline_micros = ((buffer_size.as_usize() as f64 / sample_rate.as_f64()) * 1_000_000.0) as u64;
        Self {
            sample_rate,
            buffer_size,
            deadline_micros,
            current_usage_pct: 0.0,
            warning_flag: false,
        }
    }

    pub fn record_execution_time_micros(&mut self, elapsed_micros: u64) {
        if self.deadline_micros == 0 {
            self.current_usage_pct = 0.0;
            return;
        }

        let usage = (elapsed_micros as f64 / self.deadline_micros as f64) * 100.0;
        self.current_usage_pct = usage as f32;

        if self.current_usage_pct >= Self::OVERLOAD_THRESHOLD_PCT {
            self.warning_flag = true;
        } else {
            self.warning_flag = false;
        }
    }

    pub fn cpu_usage_pct(&self) -> f32 {
        self.current_usage_pct
    }

    pub fn is_overloaded(&self) -> bool {
        self.current_usage_pct >= Self::OVERLOAD_THRESHOLD_PCT
    }

    pub fn warning_flag(&self) -> bool {
        self.warning_flag
    }

    /// Verifica se é seguro admitir um novo bloco de processamento pesado sem causar dropout
    pub fn can_admit_block(&self, additional_cost_pct: f32) -> bool {
        if self.is_overloaded() {
            return false;
        }
        (self.current_usage_pct + additional_cost_pct) < Self::OVERLOAD_THRESHOLD_PCT
    }

    pub fn update_config(&mut self, sample_rate: SampleRate, buffer_size: BufferSize) {
        self.sample_rate = sample_rate;
        self.buffer_size = buffer_size;
        self.deadline_micros = ((buffer_size.as_usize() as f64 / sample_rate.as_f64()) * 1_000_000.0) as u64;
    }
}
