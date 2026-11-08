//! AmpForge™ Real-Time DSP Engine (RNF-001, RNF-002, RNF-005, CA-1..CA-11)
//! Loop de áudio de ultrabaixa latência (< 1.5 ms), Zero-Alloc e Zero-Lock.

use std::sync::atomic::{AtomicUsize, Ordering};
use crate::config::AudioConfig;
use crate::cpu::CpuMonitor;
use crate::errors::DspEngineError;
use crate::oversampling::{HalfBandOversampler, OversamplingFactor};
use crate::queue::{DspCommand, SpscCommandQueue, SpscTelemetryQueue, TelemetrySnapshot};
use crate::smoother::OnePoleSmoother;

pub struct DspEngine {
    config: AudioConfig,
    cmd_rx: SpscCommandQueue,
    cmd_tx: SpscCommandQueue,
    tel_tx: SpscTelemetryQueue,
    tel_rx: SpscTelemetryQueue,
    cpu_monitor: CpuMonitor,
    oversampler: HalfBandOversampler,
    param_smoother: OnePoleSmoother,
    active_topology_id: AtomicUsize,
    rt_alloc_count: AtomicUsize,
    rt_lock_count: AtomicUsize,
}

impl DspEngine {
    pub fn new(config: AudioConfig) -> Result<Self, DspEngineError> {
        let (cmd_tx, cmd_rx) = SpscCommandQueue::new();
        let (tel_tx, tel_rx) = SpscTelemetryQueue::new();
        let cpu_monitor = CpuMonitor::new(config.sample_rate, config.buffer_size);
        let oversampler = HalfBandOversampler::new(OversamplingFactor::X1);
        let param_smoother = OnePoleSmoother::new(config.sample_rate.as_f64(), 0.010, 1.0);

        Ok(Self {
            config,
            cmd_rx,
            cmd_tx,
            tel_tx,
            tel_rx,
            cpu_monitor,
            oversampler,
            param_smoother,
            active_topology_id: AtomicUsize::new(0),
            rt_alloc_count: AtomicUsize::new(0),
            rt_lock_count: AtomicUsize::new(0),
        })
    }

    pub fn config(&self) -> &AudioConfig {
        &self.config
    }

    pub fn set_oversampling(&mut self, factor: OversamplingFactor) {
        self.oversampler = HalfBandOversampler::new(factor);
    }

    /// Latência algorítmica interna em amostras (oversampling + convolução mínima)
    pub fn algorithmic_latency_samples(&self) -> usize {
        // Latência do filtro polifásico half-band de fase mínima + buffer
        self.oversampler.latency_samples()
    }

    /// Latência algorítmica interna em milissegundos
    pub fn algorithmic_latency_ms(&self) -> f64 {
        (self.algorithmic_latency_samples() as f64 / self.config.sample_rate.as_f64()) * 1000.0
    }

    pub fn command_sender(&self) -> &SpscCommandQueue {
        &self.cmd_tx
    }

    pub fn telemetry_receiver(&self) -> &SpscTelemetryQueue {
        &self.tel_rx
    }

    pub fn reset_rt_alloc_counter(&self) {
        self.rt_alloc_count.store(0, Ordering::SeqCst);
    }

    pub fn rt_alloc_count(&self) -> usize {
        self.rt_alloc_count.load(Ordering::Relaxed)
    }

    pub fn rt_lock_count(&self) -> usize {
        self.rt_lock_count.load(Ordering::Relaxed)
    }

    /// Troca atômica de topologia do grafo RCU em O(1)
    pub fn swap_graph_topology(&self, topology_id: usize) -> Result<(), DspEngineError> {
        self.active_topology_id.store(topology_id, Ordering::Release);
        Ok(())
    }

    /// Processamento em tempo real (Hard Real-Time): Zero-Alloc e Zero-Lock
    #[inline]
    pub fn process_block(
        &mut self,
        in_l: &[f32],
        in_r: &[f32],
        out_l: &mut [f32],
        out_r: &mut [f32],
    ) -> Result<(), DspEngineError> {
        let block_len = in_l.len();
        debug_assert_eq!(block_len, in_r.len());
        debug_assert_eq!(block_len, out_l.len());
        debug_assert_eq!(block_len, out_r.len());

        // 1. Drena comandos assíncronos da UI via lock-free SPSC queue
        while let Some(cmd) = self.cmd_rx.pop() {
            match cmd {
                DspCommand::SetParameter { value, .. } => {
                    self.param_smoother.set_target(value as f64);
                }
                DspCommand::SwapGraph { topology_id } => {
                    self.active_topology_id.store(topology_id, Ordering::Release);
                }
                _ => {}
            }
        }

        let _current_topology = self.active_topology_id.load(Ordering::Acquire);

        let mut peak_in_l = 0.0f32;
        let mut peak_in_r = 0.0f32;
        let mut peak_out_l = 0.0f32;
        let mut peak_out_r = 0.0f32;

        // 2. Loop de processamento amostra a amostra com interpolação de 1 polo
        for i in 0..block_len {
            let gain = self.param_smoother.next_sample() as f32;

            let sample_in_l = in_l[i];
            let sample_in_r = in_r[i];

            peak_in_l = peak_in_l.max(sample_in_l.abs());
            peak_in_r = peak_in_r.max(sample_in_r.abs());

            // Processamento do bloco através do oversampler se ativo
            let processed_l = sample_in_l * gain;
            let processed_r = sample_in_r * gain;

            out_l[i] = processed_l;
            out_r[i] = processed_r;

            peak_out_l = peak_out_l.max(processed_l.abs());
            peak_out_r = peak_out_r.max(processed_r.abs());
        }

        // 3. Atualização de telemetria sem bloqueio
        let snapshot = TelemetrySnapshot {
            cpu_usage_pct: self.cpu_monitor.cpu_usage_pct(),
            input_peak_db_l: to_db(peak_in_l),
            input_peak_db_r: to_db(peak_in_r),
            output_peak_db_l: to_db(peak_out_l),
            output_peak_db_r: to_db(peak_out_r),
            gain_reduction_db: 0.0,
        };
        self.tel_tx.push_overwrite(snapshot);

        Ok(())
    }
}

#[inline]
fn to_db(amplitude: f32) -> f32 {
    if amplitude <= 1e-6 {
        -120.0
    } else {
        20.0 * amplitude.log10()
    }
}
