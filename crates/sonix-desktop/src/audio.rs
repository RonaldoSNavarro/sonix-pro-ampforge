//! crates/sonix-desktop/src/audio.rs
//! Host de Áudio Nativo Desktop Standalone e Duplex Real-Time (WASAPI, ASIO, CoreAudio, ALSA, JACK)
//! Suporte a captura física P2/P3 e processamento lock-free em tempo real (Spec 012 & Spec 013)

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use thiserror::Error;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use ampforge_dsp::amp::{AmpChannelMode, AmpInputJack, AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::cab::{ImpulseResponse, MicModel, SpeakerCabModel, VirtualCabProcessor};
use ampforge_dsp::dag::{MixerChannelConfig, MixerNode, SplitMode, SplitNode};
use ampforge_dsp::fx::delay::{DelayModel, DelayProcessor};
use ampforge_dsp::fx::drive::{DriveModel, DriveProcessor};
use ampforge_dsp::fx::dynamics::{CompressorProcessor, CompressorTopology, NoiseGateMode, NoiseGateProcessor};
pub use ampforge_dsp::fx::dynamics::GateState;
use ampforge_dsp::fx::eq::{EqMode, EqProcessor};
use ampforge_dsp::fx::modulation::{ModModel, ModulationProcessor};
use ampforge_dsp::fx::pitch::{PitchKey, PitchModel, PitchProcessor, ScaleType};
use ampforge_dsp::fx::reverb::{ReverbModel, ReverbProcessor};
use ampforge_dsp::fx::wah::{WahMode, WahProcessor};
use ampforge_dsp::looper::{LooperPosition, LooperProcessor};
use ampforge_dsp::modshop::{PowerTube, PreampTube, RectifierType, ToneStackTopology};
use ampforge_dsp::queue::SpscOwnedQueue;
use ampforge_dsp::tuner::{TunerOutputMode, TunerPitchResult, TunerProcessor};
pub use cpal::{Error as CpalError, ErrorKind as CpalErrorKind};

pub const STREAM_ERR_OK: u8 = 0;
pub const STREAM_ERR_DEVICE_NOT_AVAILABLE: u8 = 1;
pub const STREAM_ERR_OTHER: u8 = 2;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AudioDriverError {
    #[error("Dispositivo de áudio desconectado")]
    DeviceDisconnected,
    #[error("Falha ao inicializar driver de áudio: {0}")]
    InitFailed(String),
    #[error("Underflow de buffer de áudio em tempo real")]
    BufferUnderflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBackendType {
    Wasapi,
    Asio,
    CoreAudio,
    Alsa,
    Jack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioMode {
    Auto,
    WasapiExclusive,
    WasapiSharedLowLatency,
    WasapiShared,
    Asio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputChannel {
    Left,
    Right,
    Sum,
}

impl InputChannel {
    pub fn convert_interleaved(&self, interleaved: &[f32], channels: usize, out: &mut [f32]) {
        let ch = channels.max(1);
        let frames = out.len().min(interleaved.len() / ch);
        match self {
            InputChannel::Left => {
                for i in 0..frames {
                    out[i] = interleaved[i * ch];
                }
            }
            InputChannel::Right => {
                for i in 0..frames {
                    out[i] = if ch >= 2 {
                        interleaved[i * ch + 1]
                    } else {
                        interleaved[i * ch]
                    };
                }
            }
            InputChannel::Sum => {
                for i in 0..frames {
                    out[i] = if ch >= 2 {
                        0.5 * (interleaved[i * ch] + interleaved[i * ch + 1])
                    } else {
                        interleaved[i * ch]
                    };
                }
            }
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            InputChannel::Left => 0,
            InputChannel::Right => 1,
            InputChannel::Sum => 2,
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => InputChannel::Right,
            2 => InputChannel::Sum,
            _ => InputChannel::Left,
        }
    }

    pub fn from_str_opt(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "right" => InputChannel::Right,
            "sum" => InputChannel::Sum,
            _ => InputChannel::Left,
        }
    }

    pub fn convert_interleaved_i16(&self, interleaved: &[i16], channels: usize, out: &mut [f32]) {
        let ch = channels.max(1);
        let frames = out.len().min(interleaved.len() / ch);
        const INV_I16: f32 = 1.0 / 32768.0;
        match self {
            InputChannel::Left => {
                for i in 0..frames {
                    out[i] = interleaved[i * ch] as f32 * INV_I16;
                }
            }
            InputChannel::Right => {
                for i in 0..frames {
                    out[i] = if ch >= 2 {
                        interleaved[i * ch + 1] as f32 * INV_I16
                    } else {
                        interleaved[i * ch] as f32 * INV_I16
                    };
                }
            }
            InputChannel::Sum => {
                for i in 0..frames {
                    out[i] = if ch >= 2 {
                        0.5 * ((interleaved[i * ch] as f32 + interleaved[i * ch + 1] as f32) * INV_I16)
                    } else {
                        interleaved[i * ch] as f32 * INV_I16
                    };
                }
            }
        }
    }
}

/// Snapshot de Telemetria de Áudio Completo (Spec 018 CA-7 & Spec 036 CA-2/CA-3/CA-4)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioStatusTelemetry {
    pub backend: String,
    pub mode: String,
    pub fallback_reason: Option<String>,
    pub raw_processing: bool,
    pub sample_rate: u32,
    pub input_period_frames: u32,
    pub output_period_frames: u32,
    pub input_device: String,
    pub output_device: String,
    pub driver_input_latency_ms: f32,
    pub driver_output_latency_ms: f32,
    pub cpu_avg_pct: f32,
    pub cpu_peak_pct: f32,
    pub underruns: u64,
    pub overruns: u64,
    pub panics: u64,
    pub adc_clips: u64,
    pub measured_roundtrip_ms: Option<f32>,
    #[serde(default)]
    pub estimated_roundtrip_ms: f32,
    #[serde(default)]
    pub algorithmic_latency_samples: u32,
}

/// Buffer SPSC Lock-Free Seguro em Rust (Zero Locks, Zero Allocations) (Spec 018, CA-3)
pub struct LockFreeSpscRingBuffer {
    buffer: Box<[AtomicU32]>,
    capacity: usize,
    mask: usize,
    write_idx: AtomicUsize,
    read_idx: AtomicUsize,
}

impl LockFreeSpscRingBuffer {
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.next_power_of_two();
        let mut storage = Vec::with_capacity(cap);
        for _ in 0..cap {
            storage.push(AtomicU32::new(0));
        }
        Self {
            buffer: storage.into_boxed_slice(),
            capacity: cap,
            mask: cap - 1,
            write_idx: AtomicUsize::new(0),
            read_idx: AtomicUsize::new(0),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn occupied(&self) -> usize {
        let w = self.write_idx.load(Ordering::Acquire);
        let r = self.read_idx.load(Ordering::Acquire);
        w.wrapping_sub(r)
    }

    pub fn is_lock_free(&self) -> bool {
        true
    }

    pub fn push_slice(&self, slice: &[f32]) -> usize {
        let w = self.write_idx.load(Ordering::Relaxed);
        let r = self.read_idx.load(Ordering::Acquire);
        let occupied = w.wrapping_sub(r);
        let available = self.capacity.saturating_sub(occupied);
        let to_push = slice.len().min(available);

        for (i, &sample) in slice[..to_push].iter().enumerate() {
            let slot = (w.wrapping_add(i)) & self.mask;
            self.buffer[slot].store(sample.to_bits(), Ordering::Relaxed);
        }

        self.write_idx.store(w.wrapping_add(to_push), Ordering::Release);
        to_push
    }

    pub fn pop_slice(&self, slice: &mut [f32]) -> usize {
        let w = self.write_idx.load(Ordering::Acquire);
        let r = self.read_idx.load(Ordering::Relaxed);
        let occupied = w.wrapping_sub(r);
        let to_pop = slice.len().min(occupied);

        for (i, out) in slice[..to_pop].iter_mut().enumerate() {
            let slot = (r.wrapping_add(i)) & self.mask;
            let bits = self.buffer[slot].load(Ordering::Relaxed);
            *out = f32::from_bits(bits);
        }

        for out in slice[to_pop..].iter_mut() {
            *out = 0.0;
        }

        self.read_idx.store(r.wrapping_add(to_pop), Ordering::Release);
        to_pop
    }
}

/// Medição de latência por correlação cruzada com limiar de detecção (Spec 018, CA-9)
pub fn measure_loopback_latency(
    test_signal: &[f32],
    response_signal: &[f32],
    _sample_rate: f32,
) -> Result<u32, &'static str> {
    if test_signal.is_empty() || response_signal.len() < test_signal.len() {
        return Err("sinal de retorno não detectado");
    }

    let test_energy: f32 = test_signal.iter().map(|&x| x * x).sum();
    if test_energy < 1e-9 {
        return Err("sinal de retorno não detectado");
    }

    let max_lag = response_signal.len() - test_signal.len();
    let mut best_lag = 0usize;
    let mut max_corr = 0.0f32;

    for lag in 0..=max_lag {
        let mut corr = 0.0f32;
        let mut resp_energy = 0.0f32;

        for (i, &t) in test_signal.iter().enumerate() {
            let r = response_signal[lag + i];
            corr += t * r;
            resp_energy += r * r;
        }

        if resp_energy > 1e-9 {
            let norm_corr = corr / (test_energy * resp_energy).sqrt();
            if norm_corr > max_corr {
                max_corr = norm_corr;
                best_lag = lag;
            }
        }
    }

    if max_corr >= 0.25 {
        Ok(best_lag as u32)
    } else {
        Err("sinal de retorno não detectado")
    }
}

/// Host de Áudio Simulado para Testes de Contrato de Tempo Real e Baixa Latência (Spec 018 & 034)
pub struct SimulatedAudioHost {
    sample_rate: u32,
    period_frames: usize,
    duplex: RealtimeDuplexEngine,
    phase: f32,
    drift_ppm: f32,
    drift_fraction: f32,
    current_mode: AudioMode,
    fallback_reason: Option<String>,
    exclusive_supported: bool,
    shared_low_latency_supported: bool,
    panics: Arc<AtomicU64>,
    test_panic_triggered: bool,
    device_lost: bool,
    zero_fill_samples: u64,
    dsp_chain: RealtimeDspChain,
}

impl SimulatedAudioHost {
    pub fn new(sample_rate: u32, period_frames: usize) -> Self {
        let pf = period_frames.max(16);
        let buffer_cap = (pf * 8).next_power_of_two();
        let duplex = RealtimeDuplexEngine::with_target_period(buffer_cap, pf);

        let mut chain = RealtimeDspChain::new(sample_rate as f32);
        chain.prepare(sample_rate as f32, pf);

        Self {
            sample_rate,
            period_frames: pf,
            duplex,
            phase: 0.0,
            drift_ppm: 0.0,
            drift_fraction: 0.0,
            current_mode: AudioMode::Auto,
            fallback_reason: None,
            exclusive_supported: true,
            shared_low_latency_supported: true,
            panics: Arc::new(AtomicU64::new(0)),
            test_panic_triggered: false,
            device_lost: false,
            zero_fill_samples: 0,
            dsp_chain: chain,
        }
    }

    pub fn queue_occupancy(&self) -> usize {
        self.duplex.available_samples()
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn period_frames(&self) -> usize {
        self.period_frames
    }

    pub fn zero_fill_samples(&self) -> u64 {
        self.zero_fill_samples
    }

    pub fn reset_zero_fill_counter(&mut self) {
        self.zero_fill_samples = 0;
    }

    pub fn clean_input_ring(&self) -> Arc<CleanInputRing> {
        self.dsp_chain.clean_input_ring()
    }

    pub fn dsp_chain_mut(&mut self) -> &mut RealtimeDspChain {
        &mut self.dsp_chain
    }

    pub fn set_audio_device(
        &mut self,
        sample_rate: u32,
        buffer_size: usize,
    ) -> Result<AudioStatusTelemetry, String> {
        if !matches!(sample_rate, 44100 | 48000 | 88200 | 96000) {
            return Err(format!("Taxa de amostragem não suportada pelo dispositivo: {} Hz", sample_rate));
        }
        if !matches!(buffer_size, 32 | 64 | 128 | 256 | 512 | 1024) {
            return Err(format!("Tamanho de buffer recusado pelo dispositivo: {} frames", buffer_size));
        }
        let buffer_cap = (buffer_size * 8).next_power_of_two();
        self.duplex = RealtimeDuplexEngine::with_target_period(buffer_cap, buffer_size);
        self.sample_rate = sample_rate;
        self.period_frames = buffer_size;
        self.dsp_chain.prepare(sample_rate as f32, buffer_size);
        Ok(self.current_status_telemetry())
    }

    pub fn current_status_telemetry(&self) -> AudioStatusTelemetry {
        let lat_ms = (self.period_frames as f32 / self.sample_rate as f32) * 1000.0;
        let algo_lat = self.dsp_chain.algorithmic_latency_samples();
        let algo_ms = (algo_lat as f32 / self.sample_rate.max(1) as f32) * 1000.0;
        AudioStatusTelemetry {
            backend: "wasapi".to_string(),
            mode: match self.current_mode {
                AudioMode::WasapiExclusive => "wasapi_exclusive",
                AudioMode::WasapiSharedLowLatency => "wasapi_shared_low_latency",
                AudioMode::Asio => "asio_native",
                _ => "wasapi_shared",
            }
            .to_string(),
            fallback_reason: self.fallback_reason.clone(),
            raw_processing: true,
            sample_rate: self.sample_rate,
            input_period_frames: self.period_frames as u32,
            output_period_frames: self.period_frames as u32,
            input_device: "Simulated Input Device".to_string(),
            output_device: "Simulated Output Device".to_string(),
            driver_input_latency_ms: lat_ms,
            driver_output_latency_ms: lat_ms,
            cpu_avg_pct: 0.0,
            cpu_peak_pct: 0.0,
            underruns: 0,
            overruns: 0,
            panics: self.panics.load(Ordering::Relaxed),
            adc_clips: self.dsp_chain.adc_clips(),
            measured_roundtrip_ms: None,
            estimated_roundtrip_ms: lat_ms + lat_ms + algo_ms,
            algorithmic_latency_samples: algo_lat,
        }
    }

    pub fn set_drift_ppm(&mut self, ppm: f32) {
        self.drift_ppm = ppm;
    }

    pub fn trigger_test_panic(&mut self) {
        self.test_panic_triggered = true;
    }

    pub fn panic_count(&self) -> u64 {
        self.panics.load(Ordering::Relaxed)
    }

    pub fn is_device_lost(&self) -> bool {
        self.device_lost
    }

    pub fn simulate_device_disconnect(&mut self) {
        self.device_lost = true;
    }

    pub fn simulate_device_reconnected(&mut self, elapsed_s: f32) -> bool {
        if elapsed_s <= 2.5 {
            self.device_lost = false;
            true
        } else {
            false
        }
    }

    pub fn open_stream_with_rates(
        &mut self,
        in_rate: u32,
        out_rate: u32,
        mode: AudioMode,
    ) -> Result<(), String> {
        if in_rate != out_rate && mode == AudioMode::WasapiExclusive {
            return Err(format!(
                "Taxas de amostragem incompatíveis no modo exclusivo: captura={}Hz, reprodução={}Hz",
                in_rate, out_rate
            ));
        }
        self.current_mode = mode;
        self.sample_rate = out_rate;
        self.dsp_chain.prepare(out_rate as f32, self.period_frames);
        Ok(())
    }

    pub fn current_mode(&self) -> AudioMode {
        self.current_mode
    }

    pub fn set_exclusive_supported(&mut self, supported: bool) {
        self.exclusive_supported = supported;
    }

    pub fn set_shared_low_latency_supported(&mut self, supported: bool) {
        self.shared_low_latency_supported = supported;
    }

    pub fn fallback_reason(&self) -> Option<String> {
        self.fallback_reason.clone()
    }

    pub fn initialize_auto_backend(&mut self) -> AudioMode {
        if self.exclusive_supported {
            self.current_mode = AudioMode::WasapiExclusive;
        } else if self.shared_low_latency_supported {
            self.fallback_reason = Some("Modo exclusivo não suportado; recuando para compartilhado baixa latência".to_string());
            self.current_mode = AudioMode::WasapiSharedLowLatency;
        } else {
            self.fallback_reason = Some("Recuo para compartilhado padrão: sem suporte exclusivo ou baixa latência".to_string());
            self.current_mode = AudioMode::WasapiShared;
        }
        self.current_mode
    }

    pub fn tick_callback_pair(&mut self, sample_val: f32) {
        let period = self.period_frames;
        
        // Simulação de deriva de relógio no produtor (captura)
        let drift_step = self.drift_ppm * 1e-6 * (period as f32);
        self.drift_fraction += drift_step;
        
        let mut in_count = period;
        if self.drift_fraction >= 1.0 {
            in_count += 1;
            self.drift_fraction -= 1.0;
        } else if self.drift_fraction <= -1.0 {
            in_count = in_count.saturating_sub(1);
            self.drift_fraction += 1.0;
        }

        let in_buf = vec![sample_val; in_count];
        self.duplex.push_captured_samples(&in_buf);

        // Alvo em repouso = 1 período. Ocupação transitória pós-push = 2 períodos
        let occ = self.duplex.available_samples();
        let expected_transient = period * 2;
        let hysteresis = (period / 8).max(16);

        let mut samples_to_consume = period;
        if occ > expected_transient + hysteresis {
            samples_to_consume += 1;
        } else if occ < expected_transient.saturating_sub(hysteresis) {
            samples_to_consume = samples_to_consume.saturating_sub(1);
        }

        let mut out_buf = vec![0.0f32; samples_to_consume];
        let popped = self.duplex.pop_samples_raw(&mut out_buf);
        if popped < samples_to_consume {
            self.zero_fill_samples += (samples_to_consume - popped) as u64;
        }
    }

    pub fn tick_sine_1khz(&mut self) -> Vec<f32> {
        self.tick_sine_freq(1000.0)
    }

    pub fn tick_sine_freq(&mut self, freq_hz: f32) -> Vec<f32> {
        let period = self.period_frames;
        let mut out = vec![0.0f32; period];

        if self.device_lost {
            return out;
        }

        // Simulação de deriva de relógio no produtor (captura)
        let drift_step = self.drift_ppm * 1e-6 * (period as f32);
        self.drift_fraction += drift_step;

        let mut in_count = period;
        if self.drift_fraction >= 1.0 {
            in_count += 1;
            self.drift_fraction -= 1.0;
        } else if self.drift_fraction <= -1.0 {
            in_count = in_count.saturating_sub(1);
            self.drift_fraction += 1.0;
        }

        let w = 2.0 * std::f32::consts::PI * freq_hz / (self.sample_rate as f32);
        let mut in_buf = vec![0.0f32; in_count];
        for i in 0..in_count {
            in_buf[i] = (self.phase).sin();
            self.phase += w;
            if self.phase > 2.0 * std::f32::consts::PI {
                self.phase -= 2.0 * std::f32::consts::PI;
            }
        }

        // Simula captura produzindo na fila
        self.duplex.push_captured_samples(&in_buf);

        // Simula reprodução consumindo da fila
        let occ = self.duplex.available_samples();
        let expected_transient = period * 2;
        let hysteresis = (period / 8).max(16);

        let mut consume_len = period;
        if occ > expected_transient + hysteresis {
            consume_len += 1;
        } else if occ < expected_transient.saturating_sub(hysteresis) {
            consume_len = consume_len.saturating_sub(1);
        }

        let mut raw_slice = vec![0.0f32; consume_len];
        let popped = self.duplex.pop_samples_raw(&mut raw_slice);
        if popped < consume_len {
            self.zero_fill_samples += (consume_len - popped) as u64;
        }

        // Reamostragem suave de interpolação linear para exatamente `period` amostras
        let mut popped_buf = vec![0.0f32; period];
        for i in 0..period {
            let pos = (i as f32 / period as f32) * (raw_slice.len() as f32);
            let idx = (pos as usize).min(raw_slice.len().saturating_sub(1));
            let frac = pos - idx as f32;
            let next_idx = (idx + 1).min(raw_slice.len().saturating_sub(1));
            popped_buf[i] = raw_slice[idx] * (1.0 - frac) + raw_slice[next_idx] * frac;
        }

        // Processamento DSP e transporte com proteção contra pânico (Spec 018, CA-2 & CA-11)
        if self.test_panic_triggered {
            self.test_panic_triggered = false;
            self.panics.fetch_add(1, Ordering::Relaxed);
            out.fill(0.0);
        } else {
            out.copy_from_slice(&popped_buf);
        }

        out
    }

    pub fn push_sine_to_chain(&mut self, freq_hz: f32, frames: usize) {
        let w = 2.0 * std::f32::consts::PI * freq_hz / (self.sample_rate as f32);
        let mut in_buf = vec![0.0f32; frames];
        let mut out_buf = vec![0.0f32; frames];
        for s in in_buf.iter_mut() {
            *s = self.phase.sin() * 0.5;
            self.phase += w;
            if self.phase > 2.0 * std::f32::consts::PI {
                self.phase -= 2.0 * std::f32::consts::PI;
            }
        }
        self.dsp_chain.process(&in_buf, &mut out_buf);
    }
}

#[derive(Debug, Clone)]
pub struct DesktopAudioDevice {
    pub id: String,
    pub name: String,
    pub backend: AudioBackendType,
    pub is_input: bool,
    pub supported_sample_rates: Vec<u32>,
    pub supported_buffer_sizes: Vec<u32>,
}

pub trait AudioDeviceEnumerator: Send + Sync {
    fn enumerate_devices(&self) -> Vec<DesktopAudioDevice>;
}

#[derive(Debug, Default)]
pub struct CpalAudioEnumerator;

impl CpalAudioEnumerator {
    pub fn new() -> Self {
        Self
    }

    fn host_backend() -> AudioBackendType {
        if cfg!(target_os = "windows") {
            AudioBackendType::Wasapi
        } else if cfg!(target_os = "macos") {
            AudioBackendType::CoreAudio
        } else {
            AudioBackendType::Alsa
        }
    }

    fn collect_sample_rates(device: &cpal::Device, is_input: bool) -> Vec<u32> {
        let mut rates = Vec::new();
        let ranges = if is_input {
            device.supported_input_configs().ok().map(|it| {
                it.map(|r| (r.min_sample_rate(), r.max_sample_rate()))
                    .collect::<Vec<_>>()
            })
        } else {
            device.supported_output_configs().ok().map(|it| {
                it.map(|r| (r.min_sample_rate(), r.max_sample_rate()))
                    .collect::<Vec<_>>()
            })
        };
        if let Some(list) = ranges {
            for candidate in [44100u32, 48000, 88200, 96000] {
                if list.iter().any(|&(lo, hi)| candidate >= lo && candidate <= hi) {
                    rates.push(candidate);
                }
            }
        }
        if rates.is_empty() {
            if is_input {
                if let Ok(def) = device.default_input_config() {
                    rates.push(def.sample_rate());
                }
            } else if let Ok(def) = device.default_output_config() {
                rates.push(def.sample_rate());
            }
        }
        rates
    }
    fn enumerate_host_devices(
        host: &cpal::Host,
        backend: AudioBackendType,
        prefix: &str,
        out: &mut Vec<DesktopAudioDevice>,
    ) {
        let default_in_name = host
            .default_input_device()
            .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()));
        let default_out_name = host
            .default_output_device()
            .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()));

        let mut inputs_list = Vec::new();
        if let Ok(inputs) = host.input_devices() {
            for (idx, dev) in inputs.enumerate() {
                let name = dev
                    .description()
                    .map(|d| d.name().to_string())
                    .unwrap_or_else(|_| format!("Input Device {}", idx + 1));
                let rates = Self::collect_sample_rates(&dev, true);
                if rates.is_empty() {
                    continue;
                }
                let is_default = default_in_name.as_deref() == Some(name.as_str());
                inputs_list.push((
                    is_default,
                    DesktopAudioDevice {
                        id: format!("{}in_{}", prefix, idx),
                        name,
                        backend: backend.clone(),
                        is_input: true,
                        supported_sample_rates: rates,
                        supported_buffer_sizes: vec![64, 128, 256, 512, 1024],
                    },
                ));
            }
        }
        inputs_list.sort_by_key(|(is_def, _)| std::cmp::Reverse(*is_def));
        out.extend(inputs_list.into_iter().map(|(_, d)| d));

        let mut outputs_list = Vec::new();
        if let Ok(outputs) = host.output_devices() {
            for (idx, dev) in outputs.enumerate() {
                let name = dev
                    .description()
                    .map(|d| d.name().to_string())
                    .unwrap_or_else(|_| format!("Output Device {}", idx + 1));
                let rates = Self::collect_sample_rates(&dev, false);
                if rates.is_empty() {
                    continue;
                }
                let is_default = default_out_name.as_deref() == Some(name.as_str());
                outputs_list.push((
                    is_default,
                    DesktopAudioDevice {
                        id: format!("{}out_{}", prefix, idx),
                        name,
                        backend: backend.clone(),
                        is_input: false,
                        supported_sample_rates: rates,
                        supported_buffer_sizes: vec![64, 128, 256, 512, 1024],
                    },
                ));
            }
        }
        outputs_list.sort_by_key(|(is_def, _)| std::cmp::Reverse(*is_def));
        out.extend(outputs_list.into_iter().map(|(_, d)| d));
    }
}

impl AudioDeviceEnumerator for CpalAudioEnumerator {
    fn enumerate_devices(&self) -> Vec<DesktopAudioDevice> {
        let host = cpal::default_host();
        let backend = Self::host_backend();
        let mut out = Vec::new();
        Self::enumerate_host_devices(&host, backend, "", &mut out);

        #[cfg(feature = "asio")]
        if let Ok(asio_host) = cpal::host_from_id(cpal::HostId::Asio) {
            Self::enumerate_host_devices(&asio_host, AudioBackendType::Asio, "asio_", &mut out);
        }

        out
    }
}

pub struct DesktopAudioHost {
    stream_active: bool,
    local_ipc_only: bool,
    enumerator: Arc<dyn AudioDeviceEnumerator>,
}

impl Default for DesktopAudioHost {
    fn default() -> Self {
        Self::new()
    }
}

impl DesktopAudioHost {
    pub fn new() -> Self {
        Self {
            stream_active: false,
            local_ipc_only: true,
            enumerator: Arc::new(CpalAudioEnumerator::new()),
        }
    }

    pub fn with_enumerator(enumerator: Arc<dyn AudioDeviceEnumerator>) -> Self {
        Self {
            stream_active: false,
            local_ipc_only: true,
            enumerator,
        }
    }

    /// Enumera dispositivos reais do enumerador injetado (Spec 036 CA-1 & CA-9).
    pub fn enumerate_devices(&self) -> Vec<DesktopAudioDevice> {
        self.enumerator.enumerate_devices()
    }

    /// Alias compatível sem lista inventada (Spec 036 CA-1).
    pub fn enumerate_physical_devices(&self) -> Vec<DesktopAudioDevice> {
        self.enumerate_devices()
    }

    /// Informa suporte real do host para modos de áudio (Spec 036 CA-7 / Spec 049 CA-36..CA-37).
    pub fn has_mode_support(&self, mode: AudioMode) -> bool {
        match mode {
            AudioMode::Auto | AudioMode::WasapiShared | AudioMode::WasapiSharedLowLatency => true,
            AudioMode::Asio => cfg!(feature = "asio"),
            AudioMode::WasapiExclusive => false,
        }
    }

    pub fn validate_requested_mode(&self, mode_str: &str) -> Result<AudioMode, String> {
        let mode = match mode_str.trim().to_ascii_lowercase().as_str() {
            "" | "auto" => AudioMode::Auto,
            "wasapi" | "wasapi_shared" => AudioMode::WasapiShared,
            "wasapi_shared_low_latency" => AudioMode::WasapiSharedLowLatency,
            "wasapi_exclusive" => AudioMode::WasapiExclusive,
            "asio" | "asio_native" => AudioMode::Asio,
            other => {
                return Err(format!("mode_unavailable: modo desconhecido '{}'", other));
            }
        };
        if !self.has_mode_support(mode) {
            return Err(format!(
                "mode_unavailable: modo '{}' não implementado neste host (em implementação — Spec 036)",
                mode_str
            ));
        }
        Ok(mode)
    }

    pub fn start_stream(
        &mut self,
        _device: &DesktopAudioDevice,
        _sample_rate: u32,
        _buffer_size: u32,
    ) -> Result<(), AudioDriverError> {
        self.stream_active = true;
        Ok(())
    }

    pub fn is_stream_active(&self) -> bool {
        self.stream_active
    }

    pub fn is_local_ipc_only(&self) -> bool {
        self.local_ipc_only
    }

    /// Trata underrun ou ausência de microfone preenchendo o buffer com silêncio (Spec 013, CA-7)
    pub fn handle_audio_underrun(&self, buffer: &mut [f32]) {
        buffer.fill(0.0);
    }

    /// Desconexão graciosa sem estalos nem panic (Spec 012 & 013, CA-7)
    pub fn simulate_device_disconnect(&mut self, buffer: &mut [f32]) -> AudioDriverError {
        // Zero output buffer cleanly to avoid explosive noise
        buffer.fill(0.0);
        self.stream_active = false;
        AudioDriverError::DeviceDisconnected
    }

    /// Processa entrada F32 em qualquer quantidade de frames com seleção linear de canal (Spec 034 CA-5 & Spec 036 CA-6).
    pub fn process_input_callback_f32(
        data: &[f32],
        in_channels: usize,
        mono_buf: &mut [f32],
        duplex: &RealtimeDuplexEngine,
        telemetry: &TelemetryBuffer,
    ) {
        let ch = in_channels.max(1);
        let total_frames = data.len() / ch;
        if total_frames > 0 {
            telemetry.record_observed_input_period(total_frames as u32);
        }
        let chunk_cap = mono_buf.len().max(1);
        let input_channel = duplex.input_channel();
        let mut offset = 0usize;
        let mut peak = 0.0f32;
        while offset < total_frames {
            let frames = (total_frames - offset).min(chunk_cap);
            let start = offset * ch;
            let end = (offset + frames) * ch;
            input_channel.convert_interleaved(&data[start..end], ch, &mut mono_buf[..frames]);
            for &s in &mono_buf[..frames] {
                peak = peak.max(s.abs());
            }
            duplex.push_captured_samples(&mono_buf[..frames]);
            offset += frames;
        }
        let peak_db = if peak > 1e-6 { 20.0 * peak.log10() } else { -120.0 };
        telemetry.update_input_peak(peak_db);
    }

    #[allow(dead_code)]
    #[inline]
    fn record_observed_period(observed: &AtomicU32, sample_len: usize, channels: usize) {
        let frames = (sample_len / channels.max(1)).max(1) as u32;
        observed.store(frames, Ordering::Relaxed);
    }

    /// Processa entrada I16 em qualquer quantidade de frames com seleção linear de canal (Spec 034 CA-5 & Spec 036 CA-6).
    pub fn process_input_callback_i16(
        data: &[i16],
        in_channels: usize,
        mono_buf: &mut [f32],
        duplex: &RealtimeDuplexEngine,
        telemetry: &TelemetryBuffer,
    ) {
        let ch = in_channels.max(1);
        let total_frames = data.len() / ch;
        if total_frames > 0 {
            telemetry.record_observed_input_period(total_frames as u32);
        }
        let chunk_cap = mono_buf.len().max(1);
        let input_channel = duplex.input_channel();
        let mut offset = 0usize;
        let mut peak = 0.0f32;
        while offset < total_frames {
            let frames = (total_frames - offset).min(chunk_cap);
            let start = offset * ch;
            let end = (offset + frames) * ch;
            input_channel.convert_interleaved_i16(&data[start..end], ch, &mut mono_buf[..frames]);
            for &s in &mono_buf[..frames] {
                peak = peak.max(s.abs());
            }
            duplex.push_captured_samples(&mono_buf[..frames]);
            offset += frames;
        }
        let peak_db = if peak > 1e-6 { 20.0 * peak.log10() } else { -120.0 };
        telemetry.update_input_peak(peak_db);
    }

    /// Processa saída F32 usando um processador customizável e captura de pânico no callback (AUD-04 / Spec 018 CA-11).
    pub fn process_output_callback_f32_with_processor<F>(
        data: &mut [f32],
        out_channels: usize,
        out_sample_rate: f32,
        mono_in: &mut [f32],
        mono_out: &mut [f32],
        duplex: &RealtimeDuplexEngine,
        dsp_chain: &Mutex<RealtimeDspChain>,
        telemetry: &TelemetryBuffer,
        observed_period: Option<&AtomicU32>,
        mut processor: F,
    ) where
        F: FnMut(&mut RealtimeDspChain, &[f32], &mut [f32]),
    {
        let ch = out_channels.max(1);
        let total_frames = data.len() / ch;
        if total_frames == 0 {
            return;
        }
        telemetry.record_observed_output_period(total_frames as u32);
        if let Some(obs) = observed_period {
            let tf = total_frames as u32;
            if obs.load(Ordering::Relaxed) != tf {
                obs.store(tf, Ordering::Relaxed);
                duplex.set_target_period(total_frames);
            }
        }

        if telemetry.is_engine_faulted() {
            data.fill(0.0);
            telemetry.update_peaks(telemetry.input_peak_db(), -120.0, 0.0);
            return;
        }

        let chunk_cap = mono_in.len().min(mono_out.len()).max(1);
        let mut offset = 0usize;
        let mut peak = 0.0f32;
        let mut gate_telemetry: Option<(GateState, f32, f32)> = None;
        let t0 = std::time::Instant::now();

        while offset < total_frames {
            let frames = (total_frames - offset).min(chunk_cap);
            duplex.pop_samples_for_processing(&mut mono_in[..frames]);
            let mut faulted = false;
            if let Ok(mut dsp) = dsp_chain.try_lock() {
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    processor(&mut dsp, &mono_in[..frames], &mut mono_out[..frames]);
                }));
                match res {
                    Ok(()) => {
                        gate_telemetry = Some((dsp.gate_state(), dsp.gate_gain_reduction_db(), dsp.current_b_plus_v()));
                    }
                    Err(_payload) => {
                        mono_out[..frames].fill(0.0);
                        telemetry.mark_engine_faulted();
                        faulted = true;
                    }
                }
            } else {
                // Contenção não silencia áudio: encaminha o sinal de entrada em bypass sem zeros (Spec 034, CA-6)
                telemetry.record_dsp_contention_silence();
                mono_out[..frames].copy_from_slice(&mono_in[..frames]);
            }
            if faulted {
                data.fill(0.0);
                telemetry.push_waveform(&mono_out[..frames]);
                peak = 0.0;
                gate_telemetry = None;
                break;
            }
            for f in 0..frames {
                let s = mono_out[f];
                peak = peak.max(s.abs());
                let base = (offset + f) * ch;
                for c in 0..ch {
                    data[base + c] = s;
                }
            }
            telemetry.push_waveform(&mono_out[..frames]);
            offset += frames;
        }

        let elapsed_s = t0.elapsed().as_secs_f32();
        let budget_s = (total_frames as f32 / out_sample_rate.max(1.0)).max(1e-6);
        let cpu_pct = (elapsed_s / budget_s) * 100.0;
        let peak_db = if peak > 1e-6 { 20.0 * peak.log10() } else { -120.0 };
        telemetry.update_peaks(telemetry.input_peak_db(), peak_db, cpu_pct);
        if let Some((gs, ggr, b_plus_v)) = gate_telemetry {
            telemetry.update_gate_telemetry(gs, ggr);
            telemetry.update_b_plus_v(b_plus_v);
        }
    }

    /// Processa saída F32 em fatias de `mono_in.len()` sem truncar callbacks > 4096 frames e sem silêncio por contenção (Spec 034, CA-5 & CA-6).
    pub fn process_output_callback_f32(
        data: &mut [f32],
        out_channels: usize,
        out_sample_rate: f32,
        mono_in: &mut [f32],
        mono_out: &mut [f32],
        duplex: &RealtimeDuplexEngine,
        dsp_chain: &Mutex<RealtimeDspChain>,
        telemetry: &TelemetryBuffer,
        observed_period: Option<&AtomicU32>,
    ) {
        Self::process_output_callback_f32_with_processor(
            data,
            out_channels,
            out_sample_rate,
            mono_in,
            mono_out,
            duplex,
            dsp_chain,
            telemetry,
            observed_period,
            |dsp, inp, out| dsp.process(inp, out),
        );
    }

    /// Processa saída I16 em fatias de `mono_in.len()` sem truncamento e sem silêncio por contenção (Spec 034, CA-5 & CA-6).
    pub fn process_output_callback_i16(
        data: &mut [i16],
        out_channels: usize,
        out_sample_rate: f32,
        mono_in: &mut [f32],
        mono_out: &mut [f32],
        duplex: &RealtimeDuplexEngine,
        dsp_chain: &Mutex<RealtimeDspChain>,
        telemetry: &TelemetryBuffer,
        observed_period: Option<&AtomicU32>,
    ) {
        let ch = out_channels.max(1);
        let total_frames = data.len() / ch;
        if total_frames == 0 {
            return;
        }
        telemetry.record_observed_output_period(total_frames as u32);
        if let Some(obs) = observed_period {
            let tf = total_frames as u32;
            if obs.load(Ordering::Relaxed) != tf {
                obs.store(tf, Ordering::Relaxed);
                duplex.set_target_period(total_frames);
            }
        }

        if telemetry.is_engine_faulted() {
            data.fill(0);
            telemetry.update_peaks(telemetry.input_peak_db(), -120.0, 0.0);
            return;
        }

        let chunk_cap = mono_in.len().min(mono_out.len()).max(1);
        let mut offset = 0usize;
        let mut peak = 0.0f32;
        let mut gate_telemetry: Option<(GateState, f32, f32)> = None;
        let t0 = std::time::Instant::now();

        while offset < total_frames {
            let frames = (total_frames - offset).min(chunk_cap);
            duplex.pop_samples_for_processing(&mut mono_in[..frames]);
            let mut faulted = false;
            if let Ok(mut dsp) = dsp_chain.try_lock() {
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    dsp.process(&mono_in[..frames], &mut mono_out[..frames]);
                }));
                match res {
                    Ok(()) => {
                        gate_telemetry = Some((dsp.gate_state(), dsp.gate_gain_reduction_db(), dsp.current_b_plus_v()));
                    }
                    Err(_payload) => {
                        mono_out[..frames].fill(0.0);
                        telemetry.mark_engine_faulted();
                        faulted = true;
                    }
                }
            } else {
                telemetry.record_dsp_contention_silence();
                mono_out[..frames].copy_from_slice(&mono_in[..frames]);
            }
            if faulted {
                data.fill(0);
                telemetry.push_waveform(&mono_out[..frames]);
                peak = 0.0;
                gate_telemetry = None;
                break;
            }
            for f in 0..frames {
                let s = mono_out[f];
                peak = peak.max(s.abs());
                let s_i16 = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
                let base = (offset + f) * ch;
                for c in 0..ch {
                    data[base + c] = s_i16;
                }
            }
            telemetry.push_waveform(&mono_out[..frames]);
            offset += frames;
        }

        let elapsed_s = t0.elapsed().as_secs_f32();
        let budget_s = (total_frames as f32 / out_sample_rate.max(1.0)).max(1e-6);
        let cpu_pct = (elapsed_s / budget_s) * 100.0;
        let peak_db = if peak > 1e-6 { 20.0 * peak.log10() } else { -120.0 };
        telemetry.update_peaks(telemetry.input_peak_db(), peak_db, cpu_pct);
        if let Some((gs, ggr, b_plus_v)) = gate_telemetry {
            telemetry.update_gate_telemetry(gs, ggr);
            telemetry.update_b_plus_v(b_plus_v);
        }
    }

    /// Resolve taxas de amostragem alinhadas para entrada e saída duplex (Spec 050, CA-3).
    /// Recusa com erro quando a taxa pedida não é suportada ou quando não existe taxa comum.
    pub fn resolve_duplex_sample_rate(
        default_in_sr: u32,
        default_out_sr: u32,
        requested_sr: Option<u32>,
        supported_in_rates: &[u32],
        supported_out_rates: &[u32],
    ) -> Result<(u32, u32), AudioDriverError> {
        let in_supports = |sr: u32| supported_in_rates.is_empty() || supported_in_rates.contains(&sr);
        let out_supports = |sr: u32| supported_out_rates.is_empty() || supported_out_rates.contains(&sr);

        if let Some(sr) = requested_sr {
            if supported_in_rates.contains(&sr) && supported_out_rates.contains(&sr) {
                return Ok((sr, sr));
            }
            return Err(AudioDriverError::InitFailed(format!(
                "mode_unavailable: taxa solicitada {} Hz não é suportada simultaneamente por entrada ({:?}) e saída ({:?})",
                sr, supported_in_rates, supported_out_rates
            )));
        }
        if default_in_sr == default_out_sr && in_supports(default_in_sr) && out_supports(default_out_sr) {
            return Ok((default_in_sr, default_out_sr));
        }
        if supported_in_rates.contains(&default_out_sr) && out_supports(default_out_sr) {
            return Ok((default_out_sr, default_out_sr));
        }
        if supported_out_rates.contains(&default_in_sr) && in_supports(default_in_sr) {
            return Ok((default_in_sr, default_in_sr));
        }
        if let Some(&common) = supported_out_rates
            .iter()
            .find(|r| supported_in_rates.contains(r))
        {
            return Ok((common, common));
        }
        Err(AudioDriverError::InitFailed(format!(
            "mode_unavailable: entrada ({:?}) e saída ({:?}) não compartilham taxa de amostragem comum",
            supported_in_rates, supported_out_rates
        )))
    }

    /// Inicia streaming duplex real-time com hardware nativo WASAPI via CPAL
    pub fn start_realtime_wasapi_duplex(
        duplex: Arc<RealtimeDuplexEngine>,
        dsp_chain: Arc<Mutex<RealtimeDspChain>>,
        telemetry: Arc<TelemetryBuffer>,
    ) -> Result<RealtimeDuplexAudioStreams, AudioDriverError> {
        Self::start_realtime_wasapi_duplex_with_config(duplex, dsp_chain, telemetry, None, None, None, None)
    }

    /// Inicia streaming duplex real-time selecionando modo (WASAPI ou ASIO quando feature `asio` está ativa — Spec 049 CA-36/CA-37).
    pub fn start_realtime_duplex_with_mode(
        duplex: Arc<RealtimeDuplexEngine>,
        dsp_chain: Arc<Mutex<RealtimeDspChain>>,
        telemetry: Arc<TelemetryBuffer>,
        mode: AudioMode,
        requested_input_device: Option<&str>,
        requested_output_device: Option<&str>,
        requested_sample_rate: Option<u32>,
        requested_buffer_size: Option<u32>,
    ) -> Result<RealtimeDuplexAudioStreams, AudioDriverError> {
        if mode == AudioMode::Asio {
            #[cfg(feature = "asio")]
            {
                let asio_host = cpal::host_from_id(cpal::HostId::Asio).map_err(|e| {
                    AudioDriverError::InitFailed(format!(
                        "mode_unavailable: driver ASIO indisponível: {}",
                        e
                    ))
                })?;
                return Self::start_realtime_duplex_on_host(
                    &asio_host,
                    "asio",
                    "asio_native",
                    duplex,
                    dsp_chain,
                    telemetry,
                    requested_input_device,
                    requested_output_device,
                    requested_sample_rate,
                    requested_buffer_size,
                )
                .map_err(|e| AudioDriverError::InitFailed(format!("mode_unavailable: {}", e)));
            }
            #[cfg(not(feature = "asio"))]
            {
                return Err(AudioDriverError::InitFailed(
                    "mode_unavailable: suporte ASIO não compilado (feature `asio` requer CPAL_ASIO_DIR)".to_string(),
                ));
            }
        }
        let req_mode_str = match mode {
            AudioMode::Auto | AudioMode::WasapiShared => "wasapi_shared",
            AudioMode::WasapiSharedLowLatency => "wasapi_shared_low_latency",
            AudioMode::WasapiExclusive => "wasapi_exclusive",
            AudioMode::Asio => "asio_native",
        };
        let host = cpal::default_host();
        Self::start_realtime_duplex_on_host(
            &host,
            "wasapi",
            req_mode_str,
            duplex,
            dsp_chain,
            telemetry,
            requested_input_device,
            requested_output_device,
            requested_sample_rate,
            requested_buffer_size,
        )
    }

    /// Inicia streaming duplex real-time com dispositivos/taxa/buffer explícitos (Spec 034, CA-3, CA-4, CA-9).
    pub fn start_realtime_wasapi_duplex_with_config(
        duplex: Arc<RealtimeDuplexEngine>,
        dsp_chain: Arc<Mutex<RealtimeDspChain>>,
        telemetry: Arc<TelemetryBuffer>,
        requested_input_device: Option<&str>,
        requested_output_device: Option<&str>,
        requested_sample_rate: Option<u32>,
        requested_buffer_size: Option<u32>,
    ) -> Result<RealtimeDuplexAudioStreams, AudioDriverError> {
        let host = cpal::default_host();
        Self::start_realtime_duplex_on_host(
            &host,
            "wasapi",
            "wasapi_shared",
            duplex,
            dsp_chain,
            telemetry,
            requested_input_device,
            requested_output_device,
            requested_sample_rate,
            requested_buffer_size,
        )
    }

    /// Grava código de erro do callback CPAL sem alocar e sem locks (AUD-05).
    /// Prioriza `ErrorKind::DeviceNotAvailable` (1) sobre outros erros (2).
    #[inline]
    pub fn record_cpal_stream_error(slot: &AtomicU8, err: cpal::Error) {
        let code = match err.kind() {
            cpal::ErrorKind::DeviceNotAvailable => STREAM_ERR_DEVICE_NOT_AVAILABLE,
            _ => STREAM_ERR_OTHER,
        };
        let _ = slot.try_update(Ordering::AcqRel, Ordering::Relaxed, |cur| {
            if cur == STREAM_ERR_DEVICE_NOT_AVAILABLE {
                None
            } else {
                Some(code)
            }
        });
    }

    fn start_realtime_duplex_on_host(
        host: &cpal::Host,
        backend_name: &str,
        default_mode_name: &str,
        duplex: Arc<RealtimeDuplexEngine>,
        dsp_chain: Arc<Mutex<RealtimeDspChain>>,
        telemetry: Arc<TelemetryBuffer>,
        requested_input_device: Option<&str>,
        requested_output_device: Option<&str>,
        requested_sample_rate: Option<u32>,
        requested_buffer_size: Option<u32>,
    ) -> Result<RealtimeDuplexAudioStreams, AudioDriverError> {
        let in_device = if let Some(req) = requested_input_device {
            host.input_devices()
                .map_err(|e| AudioDriverError::InitFailed(format!("Falha ao enumerar entradas: {}", e)))?
                .find(|d| d.description().map(|desc| desc.name() == req).unwrap_or(false))
                .ok_or_else(|| AudioDriverError::InitFailed(format!("Dispositivo de entrada não encontrado: {}", req)))?
        } else {
            host.default_input_device()
                .ok_or_else(|| AudioDriverError::InitFailed("Nenhum dispositivo de entrada de áudio encontrado".to_string()))?
        };
        let out_device = if let Some(req) = requested_output_device {
            host.output_devices()
                .map_err(|e| AudioDriverError::InitFailed(format!("Falha ao enumerar saídas: {}", e)))?
                .find(|d| d.description().map(|desc| desc.name() == req).unwrap_or(false))
                .ok_or_else(|| AudioDriverError::InitFailed(format!("Dispositivo de saída não encontrado: {}", req)))?
        } else {
            host.default_output_device()
                .ok_or_else(|| AudioDriverError::InitFailed("Nenhum dispositivo de saída de áudio encontrado".to_string()))?
        };

        let in_config = in_device
            .default_input_config()
            .map_err(|e| AudioDriverError::InitFailed(format!("Falha na config de entrada: {}", e)))?;
        let out_config = out_device
            .default_output_config()
            .map_err(|e| AudioDriverError::InitFailed(format!("Falha na config de saída: {}", e)))?;

        let default_target = if default_mode_name == "wasapi_shared_low_latency" || default_mode_name == "wasapi_exclusive" {
            64
        } else {
            128
        };
        let target_period = requested_buffer_size.unwrap_or(default_target).max(32);
        let in_channels = in_config.channels() as usize;
        let out_channels = out_config.channels() as usize;

        let supported_in = CpalAudioEnumerator::collect_sample_rates(&in_device, true);
        let supported_out = CpalAudioEnumerator::collect_sample_rates(&out_device, false);
        let (aligned_in_sr, aligned_out_sr) = Self::resolve_duplex_sample_rate(
            in_config.sample_rate(),
            out_config.sample_rate(),
            requested_sample_rate,
            &supported_in,
            &supported_out,
        )?;

        let mut in_stream_config: cpal::StreamConfig = in_config.clone().into();
        in_stream_config.sample_rate = aligned_in_sr;
        in_stream_config.buffer_size = cpal::BufferSize::Fixed(target_period);

        let mut out_stream_config: cpal::StreamConfig = out_config.clone().into();
        out_stream_config.sample_rate = aligned_out_sr;
        out_stream_config.buffer_size = cpal::BufferSize::Fixed(target_period);

        telemetry.reset_observed_periods();
        let observed_in_period = Arc::new(AtomicU32::new(target_period));
        let observed_out_period = Arc::new(AtomicU32::new(target_period));
        let err_slot_base = telemetry.stream_error_code_handle();

        let duplex_in = duplex.clone();
        let tele_in = telemetry.clone();
        let obs_in_f32 = observed_in_period.clone();
        let mut mono_buf = vec![0.0f32; 4096];
        let (in_stream, in_opened_fixed) = match in_config.sample_format() {
            cpal::SampleFormat::F32 => {
                let err_cb = err_slot_base.clone();
                let build_res = in_device.build_input_stream(
                    in_stream_config.clone(),
                    move |data: &[f32], _| {
                        Self::record_observed_period(&obs_in_f32, data.len(), in_channels);
                        Self::process_input_callback_f32(data, in_channels, &mut mono_buf, &duplex_in, &tele_in);
                    },
                    move |err| Self::record_cpal_stream_error(&err_cb, err),
                    None,
                );
                match build_res {
                    Ok(s) => (s, true),
                    Err(_) => {
                        let mut mono_buf_fb = vec![0.0f32; 4096];
                        let duplex_fb = duplex.clone();
                        let tele_fb = telemetry.clone();
                        let obs_in_fb = observed_in_period.clone();
                        let err_fb = err_slot_base.clone();
                        let mut in_fb_config: cpal::StreamConfig = in_config.clone().into();
                        in_fb_config.sample_rate = aligned_in_sr;
                        let s = in_device.build_input_stream(
                            in_fb_config,
                            move |data: &[f32], _| {
                                Self::record_observed_period(&obs_in_fb, data.len(), in_channels);
                                Self::process_input_callback_f32(data, in_channels, &mut mono_buf_fb, &duplex_fb, &tele_fb);
                            },
                            move |err| Self::record_cpal_stream_error(&err_fb, err),
                            None,
                        ).map_err(|e| AudioDriverError::InitFailed(format!("Falha ao construir stream de entrada: {}", e)))?;
                        (s, false)
                    }
                }
            }
            cpal::SampleFormat::I16 => {
                let obs_in_i16 = observed_in_period.clone();
                let err_cb = err_slot_base.clone();
                let s = in_device.build_input_stream(
                    in_stream_config.clone(),
                    move |data: &[i16], _| {
                        Self::record_observed_period(&obs_in_i16, data.len(), in_channels);
                        Self::process_input_callback_i16(data, in_channels, &mut mono_buf, &duplex_in, &tele_in);
                    },
                    move |err| Self::record_cpal_stream_error(&err_cb, err),
                    None,
                ).or_else(|_| {
                    let mut mono_buf_fb = vec![0.0f32; 4096];
                    let duplex_fb = duplex.clone();
                    let tele_fb = telemetry.clone();
                    let obs_in_fb = observed_in_period.clone();
                    let err_fb = err_slot_base.clone();
                    let mut in_fb_config: cpal::StreamConfig = in_config.clone().into();
                    in_fb_config.sample_rate = aligned_in_sr;
                    in_device.build_input_stream(
                        in_fb_config,
                        move |data: &[i16], _| {
                            Self::record_observed_period(&obs_in_fb, data.len(), in_channels);
                            Self::process_input_callback_i16(data, in_channels, &mut mono_buf_fb, &duplex_fb, &tele_fb);
                        },
                        move |err| Self::record_cpal_stream_error(&err_fb, err),
                        None,
                    ).map_err(|e2| AudioDriverError::InitFailed(format!("Falha ao construir stream de entrada: {}", e2)))
                })?;
                (s, false)
            }
            _ => return Err(AudioDriverError::InitFailed("Formato de amostra de entrada não suportado".to_string())),
        };

        let effective_out_sr = out_stream_config.sample_rate;
        let out_sample_rate_f = effective_out_sr as f32;
        if let Ok(mut dsp) = dsp_chain.lock() {
            dsp.prepare(out_sample_rate_f, (target_period as usize).max(4096));
        }

        let obs_out_c = observed_out_period.clone();
        let (out_stream, out_opened_fixed) = match out_config.sample_format() {
            cpal::SampleFormat::F32 => {
                let mut mono_in_f32 = vec![0.0f32; 4096];
                let mut mono_out_f32 = vec![0.0f32; 4096];
                let duplex_f32 = duplex.clone();
                let dsp_chain_f32 = dsp_chain.clone();
                let tele_f32 = telemetry.clone();
                let err_cb = err_slot_base.clone();

                let build_res = out_device.build_output_stream(
                    out_stream_config.clone(),
                    move |data: &mut [f32], _| {
                        Self::process_output_callback_f32(
                            data,
                            out_channels,
                            out_sample_rate_f,
                            &mut mono_in_f32,
                            &mut mono_out_f32,
                            &duplex_f32,
                            &dsp_chain_f32,
                            &tele_f32,
                            Some(&obs_out_c),
                        );
                    },
                    move |err| Self::record_cpal_stream_error(&err_cb, err),
                    None,
                );
                match build_res {
                    Ok(s) => (s, true),
                    Err(_) => {
                        let mut mono_in_fb = vec![0.0f32; 4096];
                        let mut mono_out_fb = vec![0.0f32; 4096];
                        let duplex_fb = duplex.clone();
                        let dsp_chain_fb = dsp_chain.clone();
                        let tele_fb = telemetry.clone();
                        let obs_fb = observed_out_period.clone();
                        let err_fb = err_slot_base.clone();
                        let mut out_fb_config: cpal::StreamConfig = out_config.clone().into();
                        out_fb_config.sample_rate = aligned_out_sr;
                        let s = out_device.build_output_stream(
                            out_fb_config,
                            move |data: &mut [f32], _| {
                                Self::process_output_callback_f32(
                                    data,
                                    out_channels,
                                    out_sample_rate_f,
                                    &mut mono_in_fb,
                                    &mut mono_out_fb,
                                    &duplex_fb,
                                    &dsp_chain_fb,
                                    &tele_fb,
                                    Some(&obs_fb),
                                );
                            },
                            move |err| Self::record_cpal_stream_error(&err_fb, err),
                            None,
                        ).map_err(|e| AudioDriverError::InitFailed(format!("Falha ao construir stream de saída: {}", e)))?;
                        (s, false)
                    }
                }
            }
            cpal::SampleFormat::I16 => {
                let mut mono_in_i16 = vec![0.0f32; 4096];
                let mut mono_out_i16 = vec![0.0f32; 4096];
                let duplex_i16 = duplex.clone();
                let dsp_chain_i16 = dsp_chain.clone();
                let tele_i16 = telemetry.clone();
                let obs_i16 = observed_out_period.clone();
                let err_cb = err_slot_base.clone();

                let build_res = out_device.build_output_stream(
                    out_stream_config.clone(),
                    move |data: &mut [i16], _| {
                        Self::process_output_callback_i16(
                            data,
                            out_channels,
                            out_sample_rate_f,
                            &mut mono_in_i16,
                            &mut mono_out_i16,
                            &duplex_i16,
                            &dsp_chain_i16,
                            &tele_i16,
                            Some(&obs_i16),
                        );
                    },
                    move |err| Self::record_cpal_stream_error(&err_cb, err),
                    None,
                );
                match build_res {
                    Ok(s) => (s, true),
                    Err(_) => {
                        let mut mono_in_fb = vec![0.0f32; 4096];
                        let mut mono_out_fb = vec![0.0f32; 4096];
                        let duplex_fb = duplex.clone();
                        let dsp_chain_fb = dsp_chain.clone();
                        let tele_fb = telemetry.clone();
                        let obs_fb = observed_out_period.clone();
                        let err_fb = err_slot_base.clone();
                        let mut out_fb_config: cpal::StreamConfig = out_config.clone().into();
                        out_fb_config.sample_rate = aligned_out_sr;
                        let s = out_device.build_output_stream(
                            out_fb_config,
                            move |data: &mut [i16], _| {
                                Self::process_output_callback_i16(
                                    data,
                                    out_channels,
                                    out_sample_rate_f,
                                    &mut mono_in_fb,
                                    &mut mono_out_fb,
                                    &duplex_fb,
                                    &dsp_chain_fb,
                                    &tele_fb,
                                    Some(&obs_fb),
                                );
                            },
                            move |err| Self::record_cpal_stream_error(&err_fb, err),
                            None,
                        ).map_err(|e| AudioDriverError::InitFailed(format!("Falha ao construir stream de saída: {}", e)))?;
                        (s, false)
                    }
                }
            }
            _ => return Err(AudioDriverError::InitFailed("Formato de amostra de saída não suportado".to_string())),
        };

        duplex.set_target_period(target_period as usize);

        in_stream.play().map_err(|e| AudioDriverError::InitFailed(format!("Falha ao ativar stream de entrada: {}", e)))?;
        out_stream.play().map_err(|e| AudioDriverError::InitFailed(format!("Falha ao ativar stream de saída: {}", e)))?;

        let in_period_frames = if in_opened_fixed {
            target_period
        } else {
            let obs_tele = telemetry.observed_input_period_frames();
            let obs_local = observed_in_period.load(Ordering::Relaxed);
            if obs_tele > 0 {
                obs_tele
            } else if obs_local > 0 {
                obs_local
            } else {
                target_period
            }
        };
        let out_period_frames = if out_opened_fixed {
            target_period
        } else {
            let obs_tele = telemetry.observed_output_period_frames();
            let obs_local = observed_out_period.load(Ordering::Relaxed);
            if obs_tele > 0 {
                obs_tele
            } else if obs_local > 0 {
                obs_local
            } else {
                target_period
            }
        };
        duplex.set_target_period(out_period_frames as usize);

        let in_sr = in_stream_config.sample_rate;
        let out_sr = effective_out_sr;
        let in_lat_ms = (in_period_frames as f32 / in_sr as f32) * 1000.0;
        let out_lat_ms = (out_period_frames as f32 / out_sr as f32) * 1000.0;
        let algo_lat_samples = dsp_chain
            .try_lock()
            .map(|d| d.algorithmic_latency_samples())
            .unwrap_or(4);
        let duplex_queue_ms = (out_period_frames as f32 / out_sr.max(1) as f32) * 1000.0;
        let algo_lat_ms = (algo_lat_samples as f32 / out_sr.max(1) as f32) * 1000.0;

        let status_telemetry = AudioStatusTelemetry {
            backend: backend_name.to_string(),
            mode: if backend_name == "asio" {
                default_mode_name.to_string()
            } else if default_mode_name == "wasapi_shared_low_latency" && out_opened_fixed && out_period_frames <= 128 {
                "wasapi_shared_low_latency".to_string()
            } else if default_mode_name == "wasapi_exclusive" && out_opened_fixed {
                "wasapi_exclusive".to_string()
            } else {
                default_mode_name.to_string()
            },
            fallback_reason: if out_opened_fixed {
                None
            } else {
                Some("Stream aberto com período padrão do dispositivo".to_string())
            },
            raw_processing: false,
            sample_rate: out_sr,
            input_period_frames: in_period_frames,
            output_period_frames: out_period_frames,
            input_device: in_device.description().map(|d| d.name().to_string()).unwrap_or_else(|_| "Default Input".to_string()),
            output_device: out_device.description().map(|d| d.name().to_string()).unwrap_or_else(|_| "Default Output".to_string()),
            driver_input_latency_ms: in_lat_ms,
            driver_output_latency_ms: out_lat_ms,
            cpu_avg_pct: 0.0,
            cpu_peak_pct: 0.0,
            underruns: duplex.underruns(),
            overruns: duplex.overruns(),
            panics: 0,
            adc_clips: 0,
            measured_roundtrip_ms: None,
            estimated_roundtrip_ms: in_lat_ms + out_lat_ms + duplex_queue_ms + algo_lat_ms,
            algorithmic_latency_samples: algo_lat_samples,
        };

        Ok(RealtimeDuplexAudioStreams {
            input_stream: in_stream,
            output_stream: out_stream,
            telemetry: status_telemetry,
        })
    }
}

pub struct RealtimeDuplexAudioStreams {
    pub input_stream: cpal::Stream,
    pub output_stream: cpal::Stream,
    pub telemetry: AudioStatusTelemetry,
}

/// Origem do sinal de entrada selecionada sem alocação de String no callback RT (Spec 037, CA-1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSource {
    Device,
    TestRiff,
    DiFile,
}

impl InputSource {
    pub fn from_str_name(s: &str) -> Self {
        match s {
            "test_riff" => Self::TestRiff,
            "di_file" => Self::DiFile,
            _ => Self::Device,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Device => "device",
            Self::TestRiff => "test_riff",
            Self::DiFile => "di_file",
        }
    }
}

/// Modo de divisão paralela sem alocação de String no callback RT (Spec 037, CA-1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainSplitMode {
    Ab,
    Crossover,
    DynamicSplit,
}

impl ChainSplitMode {
    pub fn from_str_name(s: &str) -> Self {
        if s.eq_ignore_ascii_case("crossover") {
            Self::Crossover
        } else if s.eq_ignore_ascii_case("dynamic_split") || s.eq_ignore_ascii_case("dynamic") {
            Self::DynamicSplit
        } else {
            Self::Ab
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ab => "ab",
            Self::Crossover => "crossover",
            Self::DynamicSplit => "dynamic_split",
        }
    }
}

/// Blocos executáveis na grade DAG 4×12 ao vivo (Spec 049, Parte C — CA-23..CA-28)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DagBlockKind {
    Gate,
    Wah1,
    Pitch1,
    Drive,
    Split,
    Amp1,
    Cab1,
    Eq1,
    Comp2,
    Amp2,
    Cab2,
    Mod2,
    Merge,
    Dly1,
    Rev1,
}

impl DagBlockKind {
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "gate" => Some(Self::Gate),
            "wah_1" | "wah" => Some(Self::Wah1),
            "pitch_1" | "pitch" => Some(Self::Pitch1),
            "drive" => Some(Self::Drive),
            "split" => Some(Self::Split),
            "amp_1" | "amp" => Some(Self::Amp1),
            "cab_1" | "cab" => Some(Self::Cab1),
            "eq_1" | "eq" => Some(Self::Eq1),
            "comp_2" | "comp" => Some(Self::Comp2),
            "amp_2" => Some(Self::Amp2),
            "cab_2" => Some(Self::Cab2),
            "mod_2" | "mod" => Some(Self::Mod2),
            "merge" => Some(Self::Merge),
            "dly_1" | "delay" => Some(Self::Dly1),
            "rev_1" | "reverb" => Some(Self::Rev1),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gate => "gate",
            Self::Wah1 => "wah_1",
            Self::Pitch1 => "pitch_1",
            Self::Drive => "drive",
            Self::Split => "split",
            Self::Amp1 => "amp_1",
            Self::Cab1 => "cab_1",
            Self::Eq1 => "eq_1",
            Self::Comp2 => "comp_2",
            Self::Amp2 => "amp_2",
            Self::Cab2 => "cab_2",
            Self::Mod2 => "mod_2",
            Self::Merge => "merge",
            Self::Dly1 => "dly_1",
            Self::Rev1 => "rev_1",
        }
    }
}

/// Topologia compilada lock-free/zero-alloc para a grade 4×12 do callback RT (Spec 049, CA-23/CA-24)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompiledDagTopology {
    pub id: u64,
    pub slots: [Option<DagBlockKind>; 48],
}

impl CompiledDagTopology {
    pub fn default_layout(id: u64) -> Self {
        let mut slots = [None; 48];
        // Row 0 (0..12)
        slots[0] = Some(DagBlockKind::Gate);
        slots[1] = Some(DagBlockKind::Drive);
        slots[2] = Some(DagBlockKind::Pitch1);
        slots[3] = Some(DagBlockKind::Amp1);
        slots[4] = Some(DagBlockKind::Cab1);
        slots[6] = Some(DagBlockKind::Eq1);
        slots[9] = Some(DagBlockKind::Dly1);
        slots[10] = Some(DagBlockKind::Rev1);
        // Row 1 (12..24)
        slots[12 + 3] = Some(DagBlockKind::Comp2);
        slots[12 + 4] = Some(DagBlockKind::Amp2);
        slots[12 + 5] = Some(DagBlockKind::Cab2);
        slots[12 + 6] = Some(DagBlockKind::Mod2);
        Self { id, slots }
    }
}

/// Fontes suportadas pelo motor de Modifiers em tempo real (Spec 049, CA-31/CA-32)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompiledModifierSource {
    ExpPedal1,
    ExpPedal2,
    MidiCc(u8),
    InternalLfo1,
    InternalLfo2,
    EnvelopeFollower,
    SceneController,
}

impl CompiledModifierSource {
    pub fn from_str_name(s: &str) -> Option<Self> {
        let up = s.trim().to_ascii_uppercase();
        if up == "EXP_PEDAL_1" {
            Some(Self::ExpPedal1)
        } else if up == "EXP_PEDAL_2" {
            Some(Self::ExpPedal2)
        } else if up == "INTERNAL_LFO_1" {
            Some(Self::InternalLfo1)
        } else if up == "INTERNAL_LFO_2" {
            Some(Self::InternalLfo2)
        } else if up == "ENVELOPE_FOLLOWER" {
            Some(Self::EnvelopeFollower)
        } else if up == "SCENE_CONTROLLER" {
            Some(Self::SceneController)
        } else if let Some(rest) = up.strip_prefix("MIDI_CC_") {
            rest.parse::<u8>().ok().filter(|&cc| cc <= 127).map(Self::MidiCc)
        } else {
            None
        }
    }
}

/// Parâmetros contínuos alvos de Modifiers (Spec 049 CA-32: exclui parâmetros discretos)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierTargetParam {
    Amp1Gain,
    Amp1Master,
    Amp1Bass,
    Amp1Middle,
    Amp1Treble,
    Amp1Presence,
    DriveDrive,
    DriveLevel,
    Mod2Depth,
    Mod2RateHz,
    Dly1Mix,
    Dly1Intensity,
    Rev1Mix,
    GateThresholdDb,
    Wah1Position,
}

impl ModifierTargetParam {
    pub const COUNT: usize = 15;
    pub const ALL: [Self; Self::COUNT] = [
        Self::Amp1Gain,
        Self::Amp1Master,
        Self::Amp1Bass,
        Self::Amp1Middle,
        Self::Amp1Treble,
        Self::Amp1Presence,
        Self::DriveDrive,
        Self::DriveLevel,
        Self::Mod2Depth,
        Self::Mod2RateHz,
        Self::Dly1Mix,
        Self::Dly1Intensity,
        Self::Rev1Mix,
        Self::GateThresholdDb,
        Self::Wah1Position,
    ];

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    pub fn from_str_name(s: &str) -> Option<(Self, f32, f32)> {
        match s.trim() {
            "amp_1.gain" => Some((Self::Amp1Gain, 0.0, 10.0)),
            "amp_1.master" => Some((Self::Amp1Master, 0.0, 10.0)),
            "amp_1.bass" => Some((Self::Amp1Bass, 0.0, 10.0)),
            "amp_1.middle" => Some((Self::Amp1Middle, 0.0, 10.0)),
            "amp_1.treble" => Some((Self::Amp1Treble, 0.0, 10.0)),
            "amp_1.presence" => Some((Self::Amp1Presence, 0.0, 10.0)),
            "drive.drive" => Some((Self::DriveDrive, 0.0, 10.0)),
            "drive.level" => Some((Self::DriveLevel, 0.0, 10.0)),
            "mod_2.depth" => Some((Self::Mod2Depth, 0.0, 100.0)),
            "mod_2.rate_hz" => Some((Self::Mod2RateHz, 0.1, 10.0)),
            "dly_1.mix" | "dly_1.echo_volume" => Some((Self::Dly1Mix, 0.0, 100.0)),
            "dly_1.intensity" => Some((Self::Dly1Intensity, 0.0, 10.0)),
            "rev_1.mix" => Some((Self::Rev1Mix, 0.0, 100.0)),
            "gate.threshold_db" => Some((Self::GateThresholdDb, -90.0, 0.0)),
            "wah_1.position" => Some((Self::Wah1Position, 0.0, 1.0)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompiledModifierBinding {
    pub active: bool,
    pub target: ModifierTargetParam,
    pub source: CompiledModifierSource,
    pub min_val: f32,
    pub max_val: f32,
    pub curve_slope: f32,
    pub damping_ms: f32,
}

impl Default for CompiledModifierBinding {
    fn default() -> Self {
        Self {
            active: false,
            target: ModifierTargetParam::Amp1Gain,
            source: CompiledModifierSource::ExpPedal1,
            min_val: 0.0,
            max_val: 10.0,
            curve_slope: 0.5,
            damping_ms: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompiledModifierTable {
    pub bindings: [CompiledModifierBinding; 16],
    pub len: usize,
}

impl CompiledModifierTable {
    pub fn empty() -> Self {
        Self {
            bindings: [CompiledModifierBinding::default(); 16],
            len: 0,
        }
    }
}

/// Recursos pré-processados na thread de serviço transferidos por propriedade sem lock/alloc ao callback RT (Spec 037, CA-3/CA-4)
pub enum PreparedDspResource {
    CabinetIr { slot: u8, ir: ImpulseResponse },
    DiSamples(Vec<f32>),
    Topology(Box<CompiledDagTopology>),
    Modifiers(Box<CompiledModifierTable>),
}

/// Recursos aposentados pelo callback RT cuja desalocação é delegada ao reaper thread (Spec 037, CA-3)
pub enum RetiredDspResource {
    CabinetIr(ImpulseResponse),
    DiSamples(Vec<f32>),
    Topology(Box<CompiledDagTopology>),
    Modifiers(Box<CompiledModifierTable>),
}

/// Fila MPSC (produtores serializados por lock externo ao callback, consumidor SPSC lock-free)
/// de transferência de recursos pesados entre controle e thread de áudio (Spec 037 CA-3/CA-4, Spec 049 CA-7/CA-24)
pub struct LockFreeResourceQueue {
    inner: Arc<SpscOwnedQueue<PreparedDspResource, 16>>,
    producer_lock: Arc<Mutex<()>>,
    retired_queue: Arc<SpscOwnedQueue<RetiredDspResource, 64>>,
    retired_reclaimed: Arc<AtomicU64>,
    retired_consumer_lock: Arc<Mutex<()>>,
}

impl LockFreeResourceQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SpscOwnedQueue::new()),
            producer_lock: Arc::new(Mutex::new(())),
            retired_queue: Arc::new(SpscOwnedQueue::new()),
            retired_reclaimed: Arc::new(AtomicU64::new(0)),
            retired_consumer_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn push(&self, item: PreparedDspResource) -> Result<(), PreparedDspResource> {
        self.reclaim_retired();
        let _guard = self.producer_lock.lock().unwrap_or_else(|e| e.into_inner());
        self.inner.push(item)
    }

    pub fn push_topology(&self, topo: Box<CompiledDagTopology>) -> Result<(), Box<CompiledDagTopology>> {
        self.push(PreparedDspResource::Topology(topo)).map_err(|rej| match rej {
            PreparedDspResource::Topology(t) => t,
            _ => unreachable!(),
        })
    }

    pub fn push_modifiers(&self, table: Box<CompiledModifierTable>) -> Result<(), Box<CompiledModifierTable>> {
        self.push(PreparedDspResource::Modifiers(table)).map_err(|rej| match rej {
            PreparedDspResource::Modifiers(t) => t,
            _ => unreachable!(),
        })
    }

    /// Garante drenagem cooperativa fora da thread de áudio (Spec 049, CA-24)
    pub fn reclaim_retired(&self) {
        let _guard = self.retired_consumer_lock.lock().unwrap_or_else(|e| e.into_inner());
        while let Some(item) = self.retired_queue.pop() {
            drop(item);
            self.retired_reclaimed.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn reclaimed_count(&self) -> u64 {
        self.retired_reclaimed.load(Ordering::Relaxed)
    }

    fn retired_queue(&self) -> Arc<SpscOwnedQueue<RetiredDspResource, 64>> {
        Arc::clone(&self.retired_queue)
    }

    fn retired_reclaimed(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.retired_reclaimed)
    }

    fn retired_consumer_lock(&self) -> Arc<Mutex<()>> {
        Arc::clone(&self.retired_consumer_lock)
    }

    #[inline]
    pub fn pop(&self) -> Option<PreparedDspResource> {
        self.inner.pop()
    }
}

impl Clone for LockFreeResourceQueue {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            producer_lock: Arc::clone(&self.producer_lock),
            retired_queue: Arc::clone(&self.retired_queue),
            retired_reclaimed: Arc::clone(&self.retired_reclaimed),
            retired_consumer_lock: Arc::clone(&self.retired_consumer_lock),
        }
    }
}

impl Default for LockFreeResourceQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Coletor em background que libera `ImpulseResponse` / `Vec<f32>` aposentados fora do callback RT (Spec 037, CA-3)
struct RetiredReaper {
    queue: Arc<SpscOwnedQueue<RetiredDspResource, 64>>,
    reclaimed_count: Arc<AtomicU64>,
    shutdown: Arc<AtomicBool>,
    worker_handle: Option<JoinHandle<()>>,
    worker_thread: thread::Thread,
}

impl RetiredReaper {
    fn noop_with_queue(
        queue: Arc<SpscOwnedQueue<RetiredDspResource, 64>>,
        reclaimed_count: Arc<AtomicU64>,
    ) -> Self {
        Self {
            queue,
            reclaimed_count,
            shutdown: Arc::new(AtomicBool::new(true)),
            worker_handle: None,
            worker_thread: thread::current(),
        }
    }

    fn with_queue(
        queue: Arc<SpscOwnedQueue<RetiredDspResource, 64>>,
        reclaimed_count: Arc<AtomicU64>,
        consumer_lock: Arc<Mutex<()>>,
    ) -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));

        let q_w = queue.clone();
        let c_w = reclaimed_count.clone();
        let s_w = shutdown.clone();
        let l_w = consumer_lock;

        let worker_handle = thread::Builder::new()
            .name("sonix-rt-reaper".to_string())
            .spawn(move || {
                loop {
                    let mut drained = false;
                    {
                        let _guard = l_w.lock().unwrap_or_else(|e| e.into_inner());
                        while let Some(item) = q_w.pop() {
                            drop(item);
                            c_w.fetch_add(1, Ordering::Relaxed);
                            drained = true;
                        }
                    }
                    if s_w.load(Ordering::Acquire) {
                        let _guard = l_w.lock().unwrap_or_else(|e| e.into_inner());
                        while let Some(item) = q_w.pop() {
                            drop(item);
                            c_w.fetch_add(1, Ordering::Relaxed);
                        }
                        break;
                    }
                    if !drained {
                        thread::park();
                    }
                }
            })
            .expect("Falha ao criar thread sonix-rt-reaper");
        let worker_thread = worker_handle.thread().clone();

        Self {
            queue,
            reclaimed_count,
            shutdown,
            worker_handle: Some(worker_handle),
            worker_thread,
        }
    }

    #[inline]
    fn retire_ir(&self, ir: ImpulseResponse) -> bool {
        match self.queue.push(RetiredDspResource::CabinetIr(ir)) {
            Ok(()) => {
                self.worker_thread.unpark();
                true
            }
            Err(rejected) => {
                std::mem::forget(rejected);
                false
            }
        }
    }

    #[inline]
    fn retire_di(&self, di: Vec<f32>) -> bool {
        if di.is_empty() && di.capacity() == 0 {
            return true;
        }
        match self.queue.push(RetiredDspResource::DiSamples(di)) {
            Ok(()) => {
                self.worker_thread.unpark();
                true
            }
            Err(rejected) => {
                std::mem::forget(rejected);
                false
            }
        }
    }

    #[inline]
    fn retire_topology(&self, topo: Box<CompiledDagTopology>) -> bool {
        match self.queue.push(RetiredDspResource::Topology(topo)) {
            Ok(()) => {
                self.worker_thread.unpark();
                true
            }
            Err(rejected) => {
                std::mem::forget(rejected);
                false
            }
        }
    }

    #[inline]
    fn retire_modifiers(&self, table: Box<CompiledModifierTable>) -> bool {
        match self.queue.push(RetiredDspResource::Modifiers(table)) {
            Ok(()) => {
                self.worker_thread.unpark();
                true
            }
            Err(rejected) => {
                std::mem::forget(rejected);
                false
            }
        }
    }

    fn reclaimed_count(&self) -> u64 {
        self.reclaimed_count.load(Ordering::Relaxed)
    }
}

impl Drop for RetiredReaper {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        self.worker_thread.unpark();
        if let Some(h) = self.worker_handle.take() {
            let _ = h.join();
        }
    }
}

/// Comandos de parâmetros atômicos para a fila SPSC lock-free (Spec 028, CA-1 & Spec 034, CA-1)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DspParamUpdate {
    AmpGain(f32),
    AmpOverdrive(f32),
    AmpBass(f32),
    AmpMiddle(f32),
    AmpTreble(f32),
    AmpPresence(f32),
    AmpDepth(f32),
    AmpMaster(f32),
    AmpVolume2(f32),
    AmpCut(f32),
    AmpModel(AmpModelId),
    AmpBright(bool),
    AmpPower(bool),
    AmpStandby(bool),
    AmpInputJack(AmpInputJack),
    AmpJumpered(bool),
    AmpChannelMode(AmpChannelMode),
    AmpSpongy(bool),
    AmpReverb(f32),
    AmpVibratoSpeed(f32),
    AmpVibratoDepth(f32),
    AmpImpedanceOhms(u8),
    AmpPreampTube(PreampTube),
    AmpPowerTube(PowerTube),
    AmpRectifier(RectifierType),
    AmpToneStack(ToneStackTopology),
    EngineOversampling(usize),
    InputBoost(f32),
    InputModeLine(bool),
    GateThreshold(f32),
    GateEnabled(bool),
    GateMode(NoiseGateMode),
    GateHysteresis(f32),
    GateAttack(f32),
    GateHold(f32),
    GateRelease(f32),
    GateRange(f32),
    GateKeyHpf(f32),
    GateKeyLpf(f32),
    DriveEnabled(bool),
    DriveGain(f32),
    DriveTone(f32),
    DriveLevel(f32),
    DriveModel(DriveModel),
    AmpEnabled(bool),
    CabEnabled(bool),
    InputSource(u8),
    AmpBiasMa(f32),
    AmpSag(f32),
    AmpNfb(f32),
    AmpXformerSat(f32),
    AmpPreampPlateV(f32),
    AmpTriodeAsym(f32),
    CabResonanceHz(f32),
    CabHicutKhz(f32),
    CabIrEnabled(bool),
    CabMic1Model(MicModel),
    CabMic1Position(f32),
    CabMic1DistanceCm(f32),
    CabMic1LevelDb(f32),
    CabMic1Enabled(bool),
    CabMic2Model(MicModel),
    CabMic2Position(f32),
    CabMic2DistanceCm(f32),
    CabMic2LevelDb(f32),
    CabMic2Enabled(bool),
    SplitEnabled(bool),
    SplitMode(ChainSplitMode),
    SplitRatio(f32),
    SplitCrossoverHz(f32),
    MergeRow1LevelDb(f32),
    MergeRow1PhaseInvert(bool),
    Comp2Enabled(bool),
    Comp2ThresholdDb(f32),
    Comp2Ratio(f32),
    Comp2AttackMs(f32),
    Comp2ReleaseMs(f32),
    Comp2MakeupGainDb(f32),
    Eq1Enabled(bool),
    Mod2Enabled(bool),
    Mod2Model(ModModel),
    Mod2RateHz(f32),
    Mod2Depth(f32),
    Mod2Mix(f32),
    Dly1Enabled(bool),
    Dly1TimeMs(f32),
    Dly1Feedback(f32),
    Dly1Mix(f32),
    Rev1Enabled(bool),
    Rev1DecayS(f32),
    Rev1PredelayMs(f32),
    Rev1Damping(f32),
    Rev1Mix(f32),
    Rev1Spillover(bool),
    Amp2Enabled(bool),
    Amp2Model(AmpModelId),
    Cab2Enabled(bool),
    Cab2Speaker(SpeakerCabModel),
    TunerEnabled(bool),
    TunerOutputMode(TunerOutputMode),
    // Spec 032 / 033: Controles adicionais em fila SPSC lock-free
    DriveBias(f32),
    Comp2Output(f32),
    Comp2AllButtons(bool),
    Eq1Mode(EqMode),
    Eq1BandFreq(usize, f32),
    Eq1BandGain(usize, f32),
    Eq1BandQ(usize, f32),
    Eq1InputGainDb(f32),
    Eq1OutputLevelDb(f32),
    Dly1Mode(u8),
    Dly1RepeatRate(f32),
    Dly1Intensity(f32),
    Dly1EchoVolume(f32),
    Dly1ReverbVolume(f32),
    Dly1Bass(f32),
    Dly1Treble(f32),
    Dly1Spillover(bool),
    Cab1Speaker(SpeakerCabModel),
    Cab1Mic1AngleDeg(f32),
    Cab1Mic2AngleDeg(f32),
    Cab1Mic1PhaseInvert(bool),
    Cab1Mic2PhaseInvert(bool),
    TunerRefHz(f32),
    Pitch1Enabled(bool),
    Pitch1Model(PitchModel),
    Pitch1Semitones(f32),
    Pitch1Key(PitchKey),
    Pitch1Scale(ScaleType),
    Pitch1Mix(f32),
    LooperPosition(LooperPosition),
    LooperAction(LooperActionCmd),
    MasterLevelDb(f32),
    Cab2ResonanceHz(f32),
    Cab2HicutKhz(f32),
    Exp1Value(f32),
    Wah1Enabled(bool),
    Wah1Position(f32),
    Wah1Q(f32),
    Wah1Mode(WahMode),
    Amp2Gain(f32),
    Amp2Bass(f32),
    Amp2Middle(f32),
    Amp2Treble(f32),
    Amp2Master(f32),
    BeginPresetCrossfade,
    TriggerPresetCrossfade,
    #[cfg(feature = "fault-injection")]
    DebugInjectPanic,
}

/// Comandos discretos de transporte para o Looper (Spec 033)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LooperActionCmd {
    Record,
    Play,
    Overdub,
    Stop,
    Undo,
    Redo,
    Reverse,
    HalfSpeed,
    Clear,
}

/// Fila SPSC Lock-Free de Comandos de Parâmetros (Zero Locks no consumidor RT, Zero Allocations) (Spec 028, CA-1 & Spec 034, CA-2)
pub struct LockFreeParamQueue {
    inner: Arc<ampforge_dsp::queue::SpscBuffer<DspParamUpdate, 2048>>,
    producer_lock: Arc<Mutex<()>>,
}

impl LockFreeParamQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ampforge_dsp::queue::SpscBuffer::new()),
            producer_lock: Arc::new(Mutex::new(())),
        }
    }

    #[inline]
    pub fn push(&self, cmd: DspParamUpdate) -> Result<(), DspParamUpdate> {
        let _guard = self.producer_lock.lock().unwrap_or_else(|e| e.into_inner());
        self.inner.push(cmd)
    }

    pub fn push_batch(&self, cmds: &[DspParamUpdate]) -> Result<(), ()> {
        let _guard = self.producer_lock.lock().unwrap_or_else(|e| e.into_inner());
        if self.inner.remaining_capacity() < cmds.len() {
            return Err(());
        }
        for &cmd in cmds {
            if self.inner.push(cmd).is_err() {
                return Err(());
            }
        }
        Ok(())
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn remaining_capacity(&self) -> usize {
        self.inner.remaining_capacity()
    }

    /// Consumidor exclusivo na thread de áudio: 100% lock-free e wait-free.
    #[inline]
    pub fn pop(&self) -> Option<DspParamUpdate> {
        self.inner.pop()
    }
}

impl Clone for LockFreeParamQueue {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            producer_lock: Arc::clone(&self.producer_lock),
        }
    }
}

impl Default for LockFreeParamQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Motor Duplex em Tempo Real com Ring Buffer Lock-Free SPSC (Spec 013 CA-3, Spec 028 CA-1/CA-2 & Spec 036 CA-5/CA-6)
pub struct RealtimeDuplexEngine {
    ring: LockFreeSpscRingBuffer,
    target_period: AtomicUsize,
    underruns: AtomicU64,
    overruns: AtomicU64,
    input_channel: AtomicU8,
}

impl RealtimeDuplexEngine {
    pub fn new(capacity: usize) -> Self {
        Self {
            ring: LockFreeSpscRingBuffer::new(capacity),
            target_period: AtomicUsize::new(128),
            underruns: AtomicU64::new(0),
            overruns: AtomicU64::new(0),
            input_channel: AtomicU8::new(InputChannel::Left.to_u8()),
        }
    }

    pub fn with_target_period(capacity: usize, target_period: usize) -> Self {
        let tp = target_period.max(16);
        let engine = Self {
            ring: LockFreeSpscRingBuffer::new(capacity),
            target_period: AtomicUsize::new(tp),
            underruns: AtomicU64::new(0),
            overruns: AtomicU64::new(0),
            input_channel: AtomicU8::new(InputChannel::Left.to_u8()),
        };
        // Pré-carrega o buffer com exatamente 1 período de silêncio para iniciar equilibrado em regime estacionário
        let warmup = vec![0.0f32; tp];
        engine.ring.push_slice(&warmup);
        engine
    }

    pub fn set_input_channel(&self, channel: InputChannel) {
        self.input_channel.store(channel.to_u8(), Ordering::Relaxed);
    }

    pub fn input_channel(&self) -> InputChannel {
        InputChannel::from_u8(self.input_channel.load(Ordering::Relaxed))
    }

    pub fn underruns(&self) -> u64 {
        self.underruns.load(Ordering::Relaxed)
    }

    pub fn overruns(&self) -> u64 {
        self.overruns.load(Ordering::Relaxed)
    }

    pub fn is_lock_free(&self) -> bool {
        self.ring.is_lock_free()
    }

    pub fn capacity(&self) -> usize {
        self.ring.capacity()
    }

    pub fn target_period(&self) -> usize {
        self.target_period.load(Ordering::Relaxed)
    }

    pub fn set_target_period(&self, period: usize) {
        self.target_period.store(period.max(16), Ordering::Relaxed);
    }

    pub fn available_samples(&self) -> usize {
        self.ring.occupied()
    }

    pub fn push_captured_samples(&self, samples: &[f32]) -> usize {
        self.ring.push_slice(samples)
    }

    pub fn pop_samples_for_processing(&self, buf: &mut [f32]) -> usize {
        let target = self.target_period.load(Ordering::Relaxed);
        let occupied = self.ring.occupied();

        // Previne acúmulo de latência (Spec 028, CA-2):
        // Se a ocupação exceder 2 períodos (ou em caso de rajada/glitch do SO), drena o excesso
        // imediatamente para o alvo de 1 período (nunca acumulando >= 2400 amostras / 50ms).
        let max_threshold = target * 2;
        if occupied > max_threshold {
            self.overruns.fetch_add(1, Ordering::Relaxed);
            let to_drop = occupied.saturating_sub(target);
            let mut discard = [0.0f32; 128];
            let mut remaining = to_drop;
            while remaining > 0 {
                let chunk = remaining.min(discard.len());
                self.ring.pop_slice(&mut discard[..chunk]);
                remaining -= chunk;
            }
        }

        let popped = self.ring.pop_slice(buf);
        if popped < buf.len() {
            self.underruns.fetch_add(1, Ordering::Relaxed);
        }
        popped
    }

    pub fn pop_samples_raw(&self, buf: &mut [f32]) -> usize {
        self.ring.pop_slice(buf)
    }
}

/// Capacidade do ring buffer circular SPSC lock-free de entrada limpa (potência de 2: ~5.4s a 48kHz, ~2.7s a 96kHz)
const CLEAN_INPUT_RING_CAPACITY: usize = 262_144;
const CLEAN_INPUT_RING_MASK: usize = CLEAN_INPUT_RING_CAPACITY - 1;

/// Ring buffer circular lock-free (single-writer no callback de áudio, zero-alloc, zero-lock)
/// compartilhado para o afinador ao vivo e captura da calibração de entrada (Spec 029).
pub struct CleanInputRing {
    buffer: Vec<AtomicU32>,
    write_pos: AtomicU64,
    sample_rate_bits: AtomicU32,
}

impl CleanInputRing {
    pub fn new(sample_rate: f32) -> Self {
        let mut buffer = Vec::with_capacity(CLEAN_INPUT_RING_CAPACITY);
        for _ in 0..CLEAN_INPUT_RING_CAPACITY {
            buffer.push(AtomicU32::new(0.0f32.to_bits()));
        }
        Self {
            buffer,
            write_pos: AtomicU64::new(0),
            sample_rate_bits: AtomicU32::new(sample_rate.max(1.0).to_bits()),
        }
    }

    pub fn set_sample_rate(&self, sample_rate: f32) {
        self.sample_rate_bits
            .store(sample_rate.max(1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn sample_rate(&self) -> f32 {
        f32::from_bits(self.sample_rate_bits.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn total_written_relaxed(&self) -> u64 {
        self.write_pos.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn total_written(&self) -> u64 {
        self.write_pos.load(Ordering::Acquire)
    }

    #[inline]
    pub fn push_sample_at(&self, pos: u64, sample: f32) {
        self.buffer[(pos as usize) & CLEAN_INPUT_RING_MASK]
            .store(sample.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    pub fn commit_write_pos(&self, new_pos: u64) {
        self.write_pos.store(new_pos, Ordering::Release);
    }

    /// Lê até `dst.len()` amostras mais recentes em ordem cronológica (sem alocação de heap).
    pub fn read_latest(&self, dst: &mut [f32]) -> usize {
        self.read_latest_since(0, dst)
    }

    /// Lê até `dst.len()` amostras mais recentes gravadas a partir de `min_pos`.
    pub fn read_latest_since(&self, min_pos: u64, dst: &mut [f32]) -> usize {
        let head = self.write_pos.load(Ordering::Acquire);
        let earliest_in_ring = head.saturating_sub(CLEAN_INPUT_RING_CAPACITY as u64);
        let start = earliest_in_ring.max(min_pos);
        let available = head.saturating_sub(start) as usize;
        let count = available.min(dst.len());
        let read_from = head - (count as u64);
        for (i, slot) in dst[..count].iter_mut().enumerate() {
            let idx = (read_from + i as u64) as usize & CLEAN_INPUT_RING_MASK;
            *slot = f32::from_bits(self.buffer[idx].load(Ordering::Relaxed));
        }
        count
    }

    /// Copia faixa `[start_pos, end_pos)` (limitada às últimas `max_samples` amostras disponíveis).
    pub fn read_range(&self, start_pos: u64, max_samples: usize) -> Vec<f32> {
        let head = self.write_pos.load(Ordering::Acquire);
        let earliest_in_ring = head.saturating_sub(CLEAN_INPUT_RING_CAPACITY as u64);
        let effective_start = start_pos
            .max(earliest_in_ring)
            .max(head.saturating_sub(max_samples as u64));
        let count = head.saturating_sub(effective_start) as usize;
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let idx = (effective_start + i as u64) as usize & CLEAN_INPUT_RING_MASK;
            out.push(f32::from_bits(self.buffer[idx].load(Ordering::Relaxed)));
        }
        out
    }
}

/// Cadeia DSP Ativa em Tempo Real: Filtro DC + Boost + Noise Gate + White-Box Amp + Cab IR (Spec 013, CA-4 & Spec 014)
pub struct RealtimeDspChain {
    sample_rate: f32,
    max_block: usize,
    shared_sample_rate_bits: Arc<AtomicU32>,
    shared_algo_latency_samples: Arc<AtomicU32>,
    shared_b_plus_v_bits: Arc<AtomicU32>,
    gate: NoiseGateProcessor,
    gate_enabled: bool,
    drive: DriveProcessor,
    drive_enabled: bool,
    amp: WhiteBoxAmpProcessor,
    amp_enabled: bool,
    cab: VirtualCabProcessor,
    cab_enabled: bool,
    input_boost: f32,
    input_mode_line: bool,
    dc_x1: f64,
    dc_x2: f64,
    dc_y1: f64,
    dc_y2: f64,
    temp_boost_buf: Vec<f32>,
    temp_source_buf: Vec<f32>,
    temp_gate_buf: Vec<f32>,
    temp_drive_buf: Vec<f32>,
    temp_amp_buf: Vec<f32>,
    gain_smoother: ampforge_dsp::smoother::OnePoleSmoother,
    // Crossfade coefficients para bypass sem estalos (Spec 014, CA-9)
    cab_fade: f32,
    cab_target_fade: f32,
    fade_step: f32,
    // Spec 019: Clipping no conversor e seleção de fonte de sinal
    adc_clips: Arc<AtomicU64>,
    input_source: InputSource,
    test_riff_buffer: Vec<f32>,
    test_riff_pos: usize,
    di_buffer: Vec<f32>,
    di_pos: usize,

    // Spec 024: Efeitos e Caminho Paralelo do Grid ao Vivo
    split: SplitNode,
    split_enabled: bool,
    split_mode: ChainSplitMode,
    split_crossover_hz: f32,
    comp_2: CompressorProcessor,
    comp_2_enabled: bool,
    amp_2: WhiteBoxAmpProcessor,
    amp_2_enabled: bool,
    cab_2: VirtualCabProcessor,
    cab_2_enabled: bool,
    mod_2: ModulationProcessor,
    mod_2_enabled: bool,
    eq_1: EqProcessor,
    eq_1_enabled: bool,
    merge: MixerNode,
    merge_row_1_level_db: f32,
    merge_row_1_phase_invert: bool,
    master_level_db: f32,
    dly_1: DelayProcessor,
    dly_1_enabled: bool,
    rev_1: ReverbProcessor,
    rev_1_enabled: bool,
    pitch_1: PitchProcessor,
    pitch_1_enabled: bool,
    wah_1: WahProcessor,
    wah_1_enabled: bool,

    // Buffers intermediários pré-alocados para processamento em tempo real zero-alloc
    temp_split_a_buf: Vec<f32>,
    temp_split_b_buf: Vec<f32>,
    temp_cab_buf: Vec<f32>,
    temp_comp_buf: Vec<f32>,
    temp_amp2_buf: Vec<f32>,
    temp_cab2_buf: Vec<f32>,
    temp_mod_buf: Vec<f32>,
    temp_eq_buf: Vec<f32>,
    temp_merge_l_buf: Vec<f32>,
    temp_merge_r_buf: Vec<f32>,
    temp_delay_buf: Vec<f32>,
    temp_reverb_buf: Vec<f32>,
    #[allow(dead_code)]
    temp_pitch_buf: Vec<f32>,
    looper: LooperProcessor,
    looper_position: LooperPosition,
    temp_looper_buf: Vec<f32>,

    // Spec 025 / 045: Crossfade real de preset/cena sem estalos e sem buraco (20 ms)
    preset_fade: f32,
    preset_fade_step: f32,
    preset_crossfading: bool,
    shadow: Option<Box<RealtimeDspChain>>,
    shadow_out_buf: Vec<f32>,
    pending_shadow_sync: [DspParamUpdate; 256],
    pending_shadow_len: usize,
    mxcsr_no_process: u32,
    #[cfg(feature = "fault-injection")]
    inject_panic_next: bool,

    // Spec 026 / 029: Afinador ao vivo e captura limpa SPSC lock-free/zero-alloc
    tuner: TunerProcessor,
    tuner_enabled: bool,
    tuner_output_mode: TunerOutputMode,
    tuner_note_bytes: [u8; 8],
    tuner_note_len: u8,
    tuner_cents: f32,
    tuner_freq_hz: f32,
    clean_input_ring: Arc<CleanInputRing>,
    scope_output_ring: Arc<CleanInputRing>,
    tuner_start_pos: u64,
    tuner_scratch_read: Vec<f32>,
    tuner_mute_gain: f32,

    // Spec 028 / 037: Filas SPSC Lock-Free de parâmetros, recursos pesados e coleta off-RT
    param_queue: Arc<LockFreeParamQueue>,
    resource_queue: Arc<LockFreeResourceQueue>,
    retired_reaper: RetiredReaper,
    cpu_monitor: ampforge_dsp::cpu::CpuMonitor,

    // Spec 049 Parte C (CA-23..CA-28): Topologia DAG 4x12 dinâmica + tap pré-gabinete
    active_topology: Box<CompiledDagTopology>,
    shared_topology_id: Arc<AtomicU64>,
    pre_cab_row0_buf: Vec<f32>,
    temp_dag_scratch_buf: Vec<f32>,

    // Spec 049 Parte D (CA-30, CA-30b, CA-32): Banco de canais sem custo inativo + Modifiers zero-alloc
    active_channel_idx: usize,
    channel_amp_models: [AmpModelId; 4],
    channel_process_counts: [u64; 4],
    active_modifiers: Box<CompiledModifierTable>,
    modifier_base_values: [f32; ModifierTargetParam::COUNT],
    modifier_smoothed: [f32; 16],
    modifier_outputs: [f32; 16],
    modifier_exp1: f32,
    modifier_exp2: f32,
    modifier_scene: f32,
    modifier_midi_cc: [f32; 128],
    lfo1_phase: f32,
    lfo2_phase: f32,
    env_follower: f32,
}

impl RealtimeDspChain {
    pub fn new(sample_rate: f32) -> Self {
        let shadow = Some(Box::new(Self::new_inner(sample_rate, false)));
        let mut chain = Self::new_inner(sample_rate, true);
        chain.shadow = shadow;
        chain.shadow_out_buf = vec![0.0; 2048];
        chain
    }

    fn new_inner(sample_rate: f32, with_runtime_resources: bool) -> Self {
        let mut gate = NoiseGateProcessor::new(NoiseGateMode::IntelligentNaturalGate);
        gate.set_sample_rate(sample_rate);
        gate.set_threshold_db(-65.0);
        gate.set_attack_ms(1.0);
        gate.set_release_ms(30.0);

        let mut drive = DriveProcessor::new(DriveModel::GreenScreamer808);
        drive.set_sample_rate(sample_rate);

        let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
        amp.set_sample_rate(sample_rate);
        amp.set_oversampling_factor(2);
        amp.panel_mut().set_input_gain(8.0);
        amp.panel_mut().set_master_volume(7.0);
        amp.panel_mut().set_bass(5.0);
        amp.panel_mut().set_middle(5.0);
        amp.panel_mut().set_treble(5.0);
        amp.panel_mut().set_presence(5.0);

        let mut cab = VirtualCabProcessor::new();
        cab.set_sample_rate(sample_rate);
        cab.set_hicut_khz(8.2);

        let mut pitch_1 = PitchProcessor::new(PitchModel::PedalWhammy);
        pitch_1.prepare(sample_rate);
        pitch_1.set_bypassed(true);

        let mut wah_1 = WahProcessor::new();
        wah_1.set_sample_rate(sample_rate);
        wah_1.set_bypassed(true);

        // One-pole smoother com constante de tempo de 7ms (Spec 014, CA-8 / Spec 044 CA-4: >=63% no áudio em <=20ms e <63% em <5ms)
        let smoother = ampforge_dsp::smoother::OnePoleSmoother::new(sample_rate as f64, 0.007, 8.0);

        // 10ms fade step
        let fade_len_samples = (sample_rate * 0.010).max(1.0);
        let fade_step = 1.0 / fade_len_samples;

        // 20ms preset fade step (Spec 025, CA-2)
        let preset_fade_len_samples = (sample_rate * 0.020).max(1.0);
        let preset_fade_step = 1.0 / preset_fade_len_samples;

        // Spec 024: Inicialização dos nós e processadores do Grid ao Vivo
        let split = SplitNode::new(SplitMode::BalanceAb { ratio: 0.5 });
        let comp_2 = CompressorProcessor::new(CompressorTopology::StudioFet1176);

        let mut amp_2 = WhiteBoxAmpProcessor::new(AmpModelId::UsTwinBlackface65);
        amp_2.set_sample_rate(sample_rate);
        amp_2.set_oversampling_factor(2);
        amp_2.panel_mut().set_input_gain(5.0);
        amp_2.panel_mut().set_bass(5.0);
        amp_2.panel_mut().set_middle(5.0);
        amp_2.panel_mut().set_treble(5.0);
        amp_2.panel_mut().set_master_volume(7.0);

        let mut cab_2 = VirtualCabProcessor::new();
        cab_2.set_sample_rate(sample_rate);
        cab_2.set_speaker_model(ampforge_dsp::cab::SpeakerCabModel::AlnicoBlue2x12Open);
        cab_2.set_hicut_khz(9.0);

        let mut mod_2 = ModulationProcessor::new(ModModel::AnalogBbdChorus);
        mod_2.set_sample_rate(sample_rate);

        let mut eq_1 = EqProcessor::new(EqMode::Parametric8Band);
        eq_1.set_sample_rate(sample_rate);

        let mut merge = MixerNode::new();
        let mut ch0 = MixerChannelConfig::default();
        ch0.level_db = 0.0;
        let mut ch1 = MixerChannelConfig::default();
        ch1.level_db = -6.0;
        ch1.phase_invert = false;
        merge.configure_channel(0, ch0);
        merge.configure_channel(1, ch1);

        let mut dly_1 = DelayProcessor::new(DelayModel::MultiHeadTapeEcho);
        dly_1.set_sample_rate(sample_rate);
        dly_1.set_spillover_enabled(true);
        dly_1.set_bypassed(true);

        let mut rev_1 = ReverbProcessor::new(ReverbModel::StudioVintagePlate);
        rev_1.set_sample_rate(sample_rate);
        rev_1.set_spillover_enabled(true);
        rev_1.set_bypassed(true);

        let mut tuner = TunerProcessor::new();
        tuner.set_sample_rate(sample_rate);

        let mut looper = LooperProcessor::new();
        looper.prepare(sample_rate, 60);

        let initial_latency = amp.algorithmic_latency_samples();
        let initial_b_plus_v = amp.current_b_plus_v();

        let resource_queue = Arc::new(LockFreeResourceQueue::new());
        let retired_reaper = if with_runtime_resources {
            RetiredReaper::with_queue(
                resource_queue.retired_queue(),
                resource_queue.retired_reclaimed(),
                resource_queue.retired_consumer_lock(),
            )
        } else {
            RetiredReaper::noop_with_queue(
                resource_queue.retired_queue(),
                resource_queue.retired_reclaimed(),
            )
        };

        Self {
            sample_rate,
            max_block: 2048,
            shared_sample_rate_bits: Arc::new(AtomicU32::new(sample_rate.max(1.0).to_bits())),
            shared_algo_latency_samples: Arc::new(AtomicU32::new(initial_latency)),
            shared_b_plus_v_bits: Arc::new(AtomicU32::new(initial_b_plus_v.to_bits())),
            gate,
            gate_enabled: true,
            drive,
            drive_enabled: false,
            amp,
            amp_enabled: true,
            cab,
            cab_enabled: true,
            input_boost: 2.5,
            input_mode_line: false,
            dc_x1: 0.0,
            dc_x2: 0.0,
            dc_y1: 0.0,
            dc_y2: 0.0,
            temp_boost_buf: vec![0.0; 2048],
            temp_source_buf: vec![0.0; 2048],
            temp_gate_buf: vec![0.0; 2048],
            temp_drive_buf: vec![0.0; 2048],
            temp_amp_buf: vec![0.0; 2048],
            gain_smoother: smoother,
            cab_fade: 1.0,
            cab_target_fade: 1.0,
            fade_step,
            adc_clips: Arc::new(AtomicU64::new(0)),
            input_source: InputSource::Device,
            test_riff_buffer: if with_runtime_resources {
                crate::calibration::generate_karplus_strong_riff(sample_rate, 12345)
            } else {
                Vec::new()
            },
            test_riff_pos: 0,
            di_buffer: Vec::new(),
            di_pos: 0,

            split,
            split_enabled: false,
            split_mode: ChainSplitMode::Ab,
            split_crossover_hz: 500.0,
            comp_2,
            comp_2_enabled: false,
            amp_2,
            amp_2_enabled: true,
            cab_2,
            cab_2_enabled: true,
            mod_2,
            mod_2_enabled: false,
            eq_1,
            eq_1_enabled: false,
            merge,
            merge_row_1_level_db: -6.0,
            merge_row_1_phase_invert: false,
            master_level_db: 0.0,
            dly_1,
            dly_1_enabled: false,
            rev_1,
            rev_1_enabled: false,
            pitch_1,
            pitch_1_enabled: false,
            wah_1,
            wah_1_enabled: false,

            temp_split_a_buf: vec![0.0; 2048],
            temp_split_b_buf: vec![0.0; 2048],
            temp_cab_buf: vec![0.0; 2048],
            temp_comp_buf: vec![0.0; 2048],
            temp_amp2_buf: vec![0.0; 2048],
            temp_cab2_buf: vec![0.0; 2048],
            temp_mod_buf: vec![0.0; 2048],
            temp_eq_buf: vec![0.0; 2048],
            temp_merge_l_buf: vec![0.0; 2048],
            temp_merge_r_buf: vec![0.0; 2048],
            temp_delay_buf: vec![0.0; 2048],
            temp_reverb_buf: vec![0.0; 2048],
            temp_pitch_buf: vec![0.0; 2048],
            looper,
            looper_position: LooperPosition::PostChain,
            temp_looper_buf: vec![0.0; 2048],

            preset_fade: 1.0,
            preset_fade_step,
            preset_crossfading: false,
            shadow: None,
            shadow_out_buf: Vec::new(),
            pending_shadow_sync: [DspParamUpdate::TriggerPresetCrossfade; 256],
            pending_shadow_len: 0,
            mxcsr_no_process: 0,
            #[cfg(feature = "fault-injection")]
            inject_panic_next: false,

            tuner,
            tuner_enabled: false,
            tuner_output_mode: TunerOutputMode::Mute,
            tuner_note_bytes: *b"---\0\0\0\0\0",
            tuner_note_len: 3,
            tuner_cents: 0.0,
            tuner_freq_hz: 0.0,
            clean_input_ring: Arc::new(CleanInputRing::new(sample_rate)),
            scope_output_ring: Arc::new(CleanInputRing::new(sample_rate)),
            tuner_start_pos: 0,
            tuner_scratch_read: if with_runtime_resources { vec![0.0; 4096] } else { Vec::new() },
            tuner_mute_gain: 1.0,

            param_queue: Arc::new(LockFreeParamQueue::new()),
            resource_queue,
            retired_reaper,
            cpu_monitor: ampforge_dsp::cpu::CpuMonitor::new(
                ampforge_dsp::config::SampleRate::Rate48000,
                ampforge_dsp::config::BufferSize::Size128,
            ),
            active_topology: Box::new(CompiledDagTopology::default_layout(1)),
            shared_topology_id: Arc::new(AtomicU64::new(1)),
            pre_cab_row0_buf: vec![0.0; 2048],
            temp_dag_scratch_buf: vec![0.0; 2048],
            active_channel_idx: 0,
            channel_amp_models: [
                AmpModelId::BritJcm800_2203,
                AmpModelId::UsTwinBlackface65,
                AmpModelId::BritJcm800_2203,
                AmpModelId::BritJcm800_2203,
            ],
            channel_process_counts: [0; 4],
            active_modifiers: Box::new(CompiledModifierTable::empty()),
            modifier_base_values: [
                8.0, 7.0, 5.0, 5.0, 5.0, 5.0, // Amp1 Gain, Master, Bass, Middle, Treble, Presence
                5.0, 5.0,                     // Drive Drive, Level
                50.0, 1.0,                    // Mod2 Depth, RateHz
                25.0, 3.0,                    // Dly1 Mix, Intensity
                25.0, -65.0,                  // Rev1 Mix, GateThresholdDb
                0.5,                          // Wah1 Position
            ],
            modifier_smoothed: [0.0; 16],
            modifier_outputs: [0.0; 16],
            modifier_exp1: 0.0,
            modifier_exp2: 0.0,
            modifier_scene: 0.0,
            modifier_midi_cc: [0.0; 128],
            lfo1_phase: 0.0,
            lfo2_phase: 0.0,
            env_follower: 0.0,
        }
    }

    pub fn configure_channel_amp_model(&mut self, ch: usize, model: &str) {
        if ch < 4 {
            let id = ampforge_dsp::amp::find_model_by_name(model)
                .unwrap_or(AmpModelId::BritJcm800_2203);
            self.channel_amp_models[ch] = id;
            if ch == self.active_channel_idx {
                self.amp.set_target_model(id);
            }
        }
    }

    /// Troca o canal ativo; blocos tonais/por canal zeram estado, enquanto delay/reverb respeitam `spillover` (Spec 049, CA-3 / CA-30b).
    pub fn set_active_channel(&mut self, ch: usize) {
        let idx = ch.min(3);
        if idx != self.active_channel_idx {
            self.active_channel_idx = idx;
            if self.dc_x1.abs() < 1e-5 && self.dc_y1.abs() < 2e-3 {
                self.dc_x1 = 0.0;
                self.dc_x2 = 0.0;
                self.dc_y1 = 0.0;
                self.dc_y2 = 0.0;
            }
            if !self.dly_1.is_spillover_enabled() {
                self.dly_1.reset_state();
            }
            if !self.rev_1.is_spillover_enabled() {
                self.rev_1.reset_state();
            }
            self.mod_2.reset_state();
            self.amp.set_target_model(self.channel_amp_models[idx]);
        }
    }

    pub fn reset_channel_process_counts(&mut self) {
        self.channel_process_counts = [0; 4];
    }

    pub fn channel_process_counts(&self) -> [u64; 4] {
        self.channel_process_counts
    }

    pub fn set_modifier_source_value(&mut self, src: CompiledModifierSource, val: f32) {
        let v = val.clamp(0.0, 1.0);
        match src {
            CompiledModifierSource::ExpPedal1 => self.modifier_exp1 = v,
            CompiledModifierSource::ExpPedal2 => self.modifier_exp2 = v,
            CompiledModifierSource::SceneController => self.modifier_scene = v,
            CompiledModifierSource::MidiCc(cc) => {
                if (cc as usize) < 128 {
                    self.modifier_midi_cc[cc as usize] = v;
                }
            }
            _ => {}
        }
    }

    pub fn current_modifier_output(&self, idx: usize) -> f32 {
        self.modifier_outputs.get(idx).copied().unwrap_or(0.0)
    }

    #[inline]
    fn evaluate_modifiers_for_block(&mut self, input: &[f32]) {
        let len = input.len();
        if len == 0 {
            return;
        }
        let dt = len as f32 / self.sample_rate.max(1.0);
        self.lfo1_phase = (self.lfo1_phase + dt * 1.0) % 1.0;
        self.lfo2_phase = (self.lfo2_phase + dt * 0.5) % 1.0;

        let mut peak = 0.0f32;
        for &s in input {
            let a = s.abs();
            if a > peak {
                peak = a;
            }
        }
        let env_coeff = (-dt / 0.025).exp();
        self.env_follower = self.env_follower * env_coeff + peak * (1.0 - env_coeff);

        let n = self.active_modifiers.len.min(16);
        for i in 0..n {
            let b = self.active_modifiers.bindings[i];
            if !b.active {
                continue;
            }
            let raw_u = match b.source {
                CompiledModifierSource::ExpPedal1 => self.modifier_exp1,
                CompiledModifierSource::ExpPedal2 => self.modifier_exp2,
                CompiledModifierSource::SceneController => self.modifier_scene,
                CompiledModifierSource::MidiCc(cc) => self.modifier_midi_cc[cc.min(127) as usize],
                CompiledModifierSource::InternalLfo1 => {
                    0.5 + 0.5 * (2.0 * std::f32::consts::PI * self.lfo1_phase).sin()
                }
                CompiledModifierSource::InternalLfo2 => {
                    0.5 + 0.5 * (2.0 * std::f32::consts::PI * self.lfo2_phase).sin()
                }
                CompiledModifierSource::EnvelopeFollower => self.env_follower.clamp(0.0, 1.0),
            };
            let u = raw_u.clamp(0.0, 1.0);
            let curved_u = if (b.curve_slope - 0.5).abs() < 1e-4 {
                u
            } else {
                let exp = ((1.0 - b.curve_slope) / b.curve_slope.max(0.01)).clamp(0.1, 10.0);
                u.powf(exp)
            };
            let target_val = b.min_val + curved_u * (b.max_val - b.min_val);
            let smoothed = if b.damping_ms <= 0.1 {
                target_val
            } else {
                let tau = b.damping_ms * 0.001;
                let alpha = 1.0 - (-dt / tau).exp();
                self.modifier_smoothed[i] + alpha * (target_val - self.modifier_smoothed[i])
            };
            self.modifier_smoothed[i] = smoothed;
            self.modifier_outputs[i] = smoothed;

            self.apply_modifier_target(b.target, smoothed);
        }
    }

    #[inline]
    fn apply_modifier_target(&mut self, target: ModifierTargetParam, val: f32) {
        match target {
            ModifierTargetParam::Amp1Gain => self.gain_smoother.set_target(val as f64),
            ModifierTargetParam::Amp1Master => self.amp.panel_mut().set_master_volume(val),
            ModifierTargetParam::Amp1Bass => self.amp.panel_mut().set_bass(val),
            ModifierTargetParam::Amp1Middle => self.amp.panel_mut().set_middle(val),
            ModifierTargetParam::Amp1Treble => self.amp.panel_mut().set_treble(val),
            ModifierTargetParam::Amp1Presence => self.amp.panel_mut().set_presence(val),
            ModifierTargetParam::DriveDrive => self.drive.set_gain(val),
            ModifierTargetParam::DriveLevel => self.drive.set_level(val),
            ModifierTargetParam::Mod2Depth => self.mod_2.set_depth(val),
            ModifierTargetParam::Mod2RateHz => self.mod_2.set_rate_hz(val),
            ModifierTargetParam::Dly1Mix => self.dly_1.set_mix_pct(val),
            ModifierTargetParam::Dly1Intensity => self.dly_1.set_feedback_pct(val * 10.0),
            ModifierTargetParam::Rev1Mix => self.rev_1.set_mix_pct(val),
            ModifierTargetParam::GateThresholdDb => self.gate.set_threshold_db(val),
            ModifierTargetParam::Wah1Position => self.wah_1.set_position(val),
        }
    }

    fn sync_modifiers_from(&mut self, table: &CompiledModifierTable, outputs: &[f32; 16]) {
        let n = table.len.min(16);
        for i in 0..n {
            let b = table.bindings[i];
            if b.active {
                self.apply_modifier_target(b.target, outputs[i]);
            }
        }
    }

    fn restore_unmodified_targets(&mut self, new_table: &CompiledModifierTable) {
        let old_n = self.active_modifiers.len.min(16);
        let new_n = new_table.len.min(16);
        for &target in &ModifierTargetParam::ALL {
            let was_active = (0..old_n).any(|i| {
                self.active_modifiers.bindings[i].active
                    && self.active_modifiers.bindings[i].target == target
            });
            if !was_active {
                continue;
            }
            let still_active = (0..new_n).any(|i| {
                new_table.bindings[i].active && new_table.bindings[i].target == target
            });
            if !still_active {
                let base = self.modifier_base_values[target.index()];
                self.apply_modifier_target(target, base);
            }
        }
    }

    pub fn topology_id(&self) -> u64 {
        self.active_topology.id
    }

    pub fn shared_topology_id(&self) -> Arc<AtomicU64> {
        self.shared_topology_id.clone()
    }

    pub fn read_amp_out_p10_no_cab(&self, dst: &mut [f32]) {
        let n = dst.len().min(self.pre_cab_row0_buf.len());
        dst[..n].copy_from_slice(&self.pre_cab_row0_buf[..n]);
        if dst.len() > n {
            dst[n..].fill(0.0);
        }
    }

    pub fn adc_clips(&self) -> u64 {
        self.adc_clips.load(Ordering::Relaxed)
    }

    pub fn adc_clips_handle(&self) -> Arc<AtomicU64> {
        self.adc_clips.clone()
    }

    pub fn shared_sample_rate_bits(&self) -> Arc<AtomicU32> {
        self.shared_sample_rate_bits.clone()
    }

    pub fn shared_algo_latency_samples(&self) -> Arc<AtomicU32> {
        self.shared_algo_latency_samples.clone()
    }

    pub fn shared_b_plus_v_bits(&self) -> Arc<AtomicU32> {
        self.shared_b_plus_v_bits.clone()
    }

    pub fn param_queue(&self) -> Arc<LockFreeParamQueue> {
        self.param_queue.clone()
    }

    pub fn set_param_queue(&mut self, pq: Arc<LockFreeParamQueue>) {
        self.param_queue = pq;
    }

    pub fn resource_queue(&self) -> Arc<LockFreeResourceQueue> {
        self.resource_queue.clone()
    }

    pub fn retired_drop_count(&self) -> u64 {
        self.retired_reaper.reclaimed_count()
    }

    pub fn mxcsr_observado(&self) -> u32 {
        self.mxcsr_no_process
    }

    pub fn looper(&self) -> &LooperProcessor {
        &self.looper
    }

    pub fn looper_mut(&mut self) -> &mut LooperProcessor {
        &mut self.looper
    }

    pub fn push_param_update(&self, update: DspParamUpdate) -> Result<(), DspParamUpdate> {
        self.param_queue.push(update)
    }

    pub fn drain_param_queue(&mut self) -> usize {
        let mut count = 0;
        while let Some(update) = self.param_queue.pop() {
            self.apply_param_update(update);
            count += 1;
        }
        count
    }

    pub fn drain_resource_queue(&mut self) -> usize {
        let mut count = 0;
        while let Some(res) = self.resource_queue.pop() {
            match res {
                PreparedDspResource::CabinetIr { slot, ir } => {
                    if let Some(ref mut shadow) = self.shadow {
                        let shadow_cab = if slot == 1 {
                            &mut shadow.cab_2
                        } else {
                            &mut shadow.cab
                        };
                        if let Some(old) = shadow_cab.take_retired_ir() {
                            self.retired_reaper.retire_ir(old);
                        }
                        shadow_cab.set_target_ir(ir.clone());
                        if let Some(old) = shadow_cab.take_retired_ir() {
                            self.retired_reaper.retire_ir(old);
                        }
                    }
                    let cab_target = if slot == 1 {
                        &mut self.cab_2
                    } else {
                        &mut self.cab
                    };
                    if let Some(old) = cab_target.take_retired_ir() {
                        self.retired_reaper.retire_ir(old);
                    }
                    cab_target.set_target_ir(ir);
                    if let Some(old) = cab_target.take_retired_ir() {
                        self.retired_reaper.retire_ir(old);
                    }
                }
                PreparedDspResource::DiSamples(samples) => {
                    let old = std::mem::replace(&mut self.di_buffer, samples);
                    self.di_pos = 0;
                    self.retired_reaper.retire_di(old);
                }
                PreparedDspResource::Topology(new_topo) => {
                    let new_id = new_topo.id;
                    if let Some(ref mut shadow) = self.shadow {
                        *shadow.active_topology = *new_topo;
                        shadow.shared_topology_id.store(new_id, Ordering::Release);
                    }
                    let old_topo = std::mem::replace(&mut self.active_topology, new_topo);
                    self.shared_topology_id.store(new_id, Ordering::Release);
                    self.retired_reaper.retire_topology(old_topo);
                }
                PreparedDspResource::Modifiers(new_mods) => {
                    self.restore_unmodified_targets(&new_mods);
                    if let Some(ref mut shadow) = self.shadow {
                        shadow.restore_unmodified_targets(&new_mods);
                        *shadow.active_modifiers = *new_mods;
                    }
                    let old_mods = std::mem::replace(&mut self.active_modifiers, new_mods);
                    self.retired_reaper.retire_modifiers(old_mods);
                }
            }
            count += 1;
        }
        count
    }

    pub fn apply_param_update(&mut self, update: DspParamUpdate) {
        if matches!(
            update,
            DspParamUpdate::BeginPresetCrossfade | DspParamUpdate::TriggerPresetCrossfade
        ) {
            self.begin_preset_crossfade();
            return;
        }
        self.apply_param_update_inner(update);
        if self.preset_crossfading {
            if self.pending_shadow_len < self.pending_shadow_sync.len() {
                self.pending_shadow_sync[self.pending_shadow_len] = update;
                self.pending_shadow_len += 1;
            }
        } else if let Some(ref mut shadow) = self.shadow {
            shadow.apply_param_update_inner(update);
        }
    }

    fn apply_param_update_inner(&mut self, update: DspParamUpdate) {
        match update {
            DspParamUpdate::AmpGain(v) => self.set_amp_gain(v),
            DspParamUpdate::AmpOverdrive(v) => self.set_amp_overdrive(v),
            DspParamUpdate::AmpBass(v) => self.set_amp_bass(v),
            DspParamUpdate::AmpMiddle(v) => self.set_amp_middle(v),
            DspParamUpdate::AmpTreble(v) => self.set_amp_treble(v),
            DspParamUpdate::AmpPresence(v) => self.set_amp_presence(v),
            DspParamUpdate::AmpDepth(v) => self.set_amp_depth(v),
            DspParamUpdate::AmpMaster(v) => self.set_amp_master(v),
            DspParamUpdate::AmpVolume2(v) => self.set_amp_volume_2(v),
            DspParamUpdate::AmpCut(v) => self.set_amp_cut(v),
            DspParamUpdate::AmpModel(m) => {
                self.amp.set_target_model(m);
                self.shared_algo_latency_samples
                    .store(self.amp.algorithmic_latency_samples(), Ordering::Relaxed);
            }
            DspParamUpdate::AmpBright(b) => self.set_amp_bright(b),
            DspParamUpdate::AmpPower(b) => self.set_amp_power(b),
            DspParamUpdate::AmpStandby(b) => self.set_amp_standby(b),
            DspParamUpdate::AmpInputJack(j) => self.amp.set_input_jack(j),
            DspParamUpdate::AmpJumpered(b) => self.set_amp_jumpered(b),
            DspParamUpdate::AmpChannelMode(m) => self.amp.set_channel_mode(m),
            DspParamUpdate::AmpSpongy(b) => self.set_amp_spongy(b),
            DspParamUpdate::AmpReverb(v) => self.set_amp_reverb(v),
            DspParamUpdate::AmpVibratoSpeed(v) => self.set_amp_vibrato_speed(v),
            DspParamUpdate::AmpVibratoDepth(v) => self.set_amp_vibrato_depth(v),
            DspParamUpdate::AmpImpedanceOhms(o) => self.amp.set_secondary_impedance_ohms(o),
            DspParamUpdate::AmpPreampTube(t) => self.amp.set_preamp_tube_type(t),
            DspParamUpdate::AmpPowerTube(t) => self.amp.set_power_tube_type(t),
            DspParamUpdate::AmpRectifier(r) => self.amp.set_rectifier_type(r),
            DspParamUpdate::AmpToneStack(ts) => self.amp.set_tonestack_topology(ts),
            DspParamUpdate::EngineOversampling(f) => {
                self.amp.set_oversampling_factor(f);
                self.shared_algo_latency_samples
                    .store(self.amp.algorithmic_latency_samples(), Ordering::Relaxed);
            }
            DspParamUpdate::InputBoost(v) => self.set_input_boost(v),
            DspParamUpdate::InputModeLine(is_line) => self.input_mode_line = is_line,
            DspParamUpdate::GateThreshold(v) => self.set_gate_threshold_db(v),
            DspParamUpdate::GateEnabled(b) => self.set_gate_enabled(b),
            DspParamUpdate::GateMode(m) => self.gate.set_mode(m),
            DspParamUpdate::GateHysteresis(v) => self.set_gate_hysteresis_db(v),
            DspParamUpdate::GateAttack(v) => self.set_gate_attack_ms(v),
            DspParamUpdate::GateHold(v) => self.set_gate_hold_ms(v),
            DspParamUpdate::GateRelease(v) => self.set_gate_release_ms(v),
            DspParamUpdate::GateRange(v) => self.set_gate_range_db(v),
            DspParamUpdate::GateKeyHpf(v) => self.set_gate_key_hpf_hz(v),
            DspParamUpdate::GateKeyLpf(v) => self.set_gate_key_lpf_hz(v),
            DspParamUpdate::DriveEnabled(b) => self.set_drive_enabled(b),
            DspParamUpdate::DriveGain(v) => self.set_drive_gain(v),
            DspParamUpdate::DriveTone(v) => self.set_drive_tone(v),
            DspParamUpdate::DriveLevel(v) => self.set_drive_level(v),
            DspParamUpdate::DriveModel(m) => self.drive.set_model(m),
            DspParamUpdate::AmpEnabled(b) => self.set_amp_enabled(b),
            DspParamUpdate::CabEnabled(b) => self.set_cab_enabled(b),
            DspParamUpdate::InputSource(src) => {
                self.input_source = match src {
                    1 => InputSource::TestRiff,
                    2 => InputSource::DiFile,
                    _ => InputSource::Device,
                };
            }
            DspParamUpdate::AmpBiasMa(v) => self.set_amp_bias_ma(v),
            DspParamUpdate::AmpSag(v) => self.set_amp_sag(v),
            DspParamUpdate::AmpNfb(v) => self.set_amp_nfb(v),
            DspParamUpdate::AmpXformerSat(v) => self.set_amp_xformer_sat(v),
            DspParamUpdate::AmpPreampPlateV(v) => self.set_amp_preamp_plate_v(v),
            DspParamUpdate::AmpTriodeAsym(v) => self.set_amp_triode_asym(v),
            DspParamUpdate::CabResonanceHz(v) => self.set_cab_resonance_hz(v),
            DspParamUpdate::CabHicutKhz(v) => self.set_cab_hicut_khz(v),
            DspParamUpdate::CabIrEnabled(b) => self.set_cab_ir_enabled(b),
            DspParamUpdate::CabMic1Model(m) => self.cab.mic_slot_mut(0).model = m,
            DspParamUpdate::CabMic1Position(v) => self.set_cab_mic1_position(v),
            DspParamUpdate::CabMic1DistanceCm(v) => self.set_cab_mic1_distance_cm(v),
            DspParamUpdate::CabMic1LevelDb(v) => self.set_cab_mic1_level_db(v),
            DspParamUpdate::CabMic1Enabled(b) => self.set_cab_mic1_enabled(b),
            DspParamUpdate::CabMic2Model(m) => self.cab.mic_slot_mut(1).model = m,
            DspParamUpdate::CabMic2Position(v) => self.set_cab_mic2_position(v),
            DspParamUpdate::CabMic2DistanceCm(v) => self.set_cab_mic2_distance_cm(v),
            DspParamUpdate::CabMic2LevelDb(v) => self.set_cab_mic2_level_db(v),
            DspParamUpdate::CabMic2Enabled(b) => self.set_cab_mic2_enabled(b),
            DspParamUpdate::SplitEnabled(b) => self.set_split_enabled(b),
            DspParamUpdate::SplitMode(m) => {
                self.split_mode = m;
                self.apply_split_config();
            }
            DspParamUpdate::SplitRatio(r) => self.split = SplitNode::new(SplitMode::BalanceAb { ratio: r }),
            DspParamUpdate::SplitCrossoverHz(hz) => {
                self.split_crossover_hz = hz.clamp(100.0, 2000.0);
                self.apply_split_config();
            }
            DspParamUpdate::MergeRow1LevelDb(db) => {
                let inv = self.merge_row_1_phase_invert;
                self.set_merge_config(db, inv);
            }
            DspParamUpdate::MergeRow1PhaseInvert(inv) => {
                let db = self.merge_row_1_level_db;
                self.set_merge_config(db, inv);
            }
            DspParamUpdate::Comp2Enabled(b) => self.set_comp_2_enabled(b),
            DspParamUpdate::Comp2ThresholdDb(th) => self.comp_2.set_threshold_db(th),
            DspParamUpdate::Comp2Ratio(r) => self.comp_2.set_ratio(r),
            DspParamUpdate::Comp2AttackMs(att) => self.comp_2.set_attack_ms(att),
            DspParamUpdate::Comp2ReleaseMs(rel) => self.comp_2.set_release_ms(rel),
            DspParamUpdate::Comp2MakeupGainDb(gain) => self.set_comp_2_output(gain),
            DspParamUpdate::Eq1Enabled(b) => self.set_eq_1_enabled(b),
            DspParamUpdate::Mod2Enabled(b) => self.set_mod_2_enabled(b),
            DspParamUpdate::Mod2Model(m) => self.set_mod_2_model(m),
            DspParamUpdate::Mod2RateHz(rate) => {
                if rate.is_finite() {
                    self.modifier_base_values[ModifierTargetParam::Mod2RateHz.index()] = rate;
                }
                self.mod_2.set_rate_hz(rate);
            }
            DspParamUpdate::Mod2Depth(depth) => {
                if depth.is_finite() {
                    self.modifier_base_values[ModifierTargetParam::Mod2Depth.index()] = depth;
                }
                self.mod_2.set_depth(depth);
            }
            DspParamUpdate::Mod2Mix(mix) => self.mod_2.set_mix_pct(mix),
            DspParamUpdate::Dly1Enabled(b) => self.set_dly_1_enabled(b),
            DspParamUpdate::Dly1TimeMs(time) => self.dly_1.set_delay_time_ms(time),
            DspParamUpdate::Dly1Feedback(fb) => {
                if fb.is_finite() {
                    self.modifier_base_values[ModifierTargetParam::Dly1Intensity.index()] = fb * 0.1;
                }
                self.dly_1.set_feedback_pct(fb);
            }
            DspParamUpdate::Dly1Mix(mix) => {
                if mix.is_finite() {
                    self.modifier_base_values[ModifierTargetParam::Dly1Mix.index()] = mix;
                }
                self.dly_1.set_mix_pct(mix);
            }
            DspParamUpdate::Rev1Enabled(b) => self.set_rev_1_enabled(b),
            DspParamUpdate::Rev1DecayS(decay) => self.rev_1.set_decay_time_s(decay),
            DspParamUpdate::Rev1PredelayMs(pd) => self.rev_1.set_predelay_ms(pd),
            DspParamUpdate::Rev1Damping(damp) => self.rev_1.set_damping(damp),
            DspParamUpdate::Rev1Mix(mix) => self.rev_1.set_mix_pct(mix),
            DspParamUpdate::Rev1Spillover(b) => self.rev_1.set_spillover_enabled(b),
            DspParamUpdate::Amp2Enabled(b) => self.amp_2_enabled = b,
            DspParamUpdate::Amp2Model(m) => self.amp_2.set_target_model(m),
            DspParamUpdate::Amp2Gain(v) => {
                if v.is_finite() {
                    self.amp_2.panel_mut().set_input_gain(v.clamp(0.0, 10.0));
                }
            }
            DspParamUpdate::Amp2Bass(v) => {
                if v.is_finite() {
                    self.amp_2.panel_mut().set_bass(v.clamp(0.0, 10.0));
                }
            }
            DspParamUpdate::Amp2Middle(v) => {
                if v.is_finite() {
                    self.amp_2.panel_mut().set_middle(v.clamp(0.0, 10.0));
                }
            }
            DspParamUpdate::Amp2Treble(v) => {
                if v.is_finite() {
                    self.amp_2.panel_mut().set_treble(v.clamp(0.0, 10.0));
                }
            }
            DspParamUpdate::Amp2Master(v) => {
                if v.is_finite() {
                    let m = v.clamp(0.0, 10.0);
                    self.amp_2.panel_mut().set_master_volume(m);
                    let db = if m <= 0.05 { -60.0 } else { (m - 7.0) * 2.5 };
                    self.amp_2.panel_mut().set_output_level_db(db);
                }
            }
            DspParamUpdate::Wah1Enabled(b) => {
                self.wah_1_enabled = b;
                self.wah_1.set_bypassed(!b);
            }
            DspParamUpdate::Wah1Position(pos) => {
                if pos.is_finite() {
                    let p = pos.clamp(0.0, 1.0);
                    self.modifier_base_values[ModifierTargetParam::Wah1Position.index()] = p;
                    self.wah_1.set_position(p);
                }
            }
            DspParamUpdate::Wah1Q(q) => {
                self.wah_1.set_q(q);
            }
            DspParamUpdate::Wah1Mode(m) => {
                self.wah_1.set_mode(m);
            }
            DspParamUpdate::Cab2Enabled(b) => self.cab_2_enabled = b,
            DspParamUpdate::Cab2Speaker(spk) => self.cab_2.set_speaker_model(spk),
            DspParamUpdate::TunerEnabled(b) => self.set_tuner_enabled(b),
            DspParamUpdate::TunerOutputMode(m) => self.set_tuner_output_mode(m),
            DspParamUpdate::DriveBias(b) => self.set_drive_bias(b),
            DspParamUpdate::Comp2Output(out) => self.set_comp_2_output(out),
            DspParamUpdate::Comp2AllButtons(all) => self.comp_2.set_all_buttons_in(all),
            DspParamUpdate::Eq1Mode(m) => self.eq_1.set_mode(m),
            DspParamUpdate::Eq1BandFreq(idx, freq_hz) => {
                if idx < 10 {
                    let cur = self.eq_1.parametric_band(idx);
                    self.eq_1.set_parametric_band(idx, freq_hz, cur.gain_db, cur.q);
                }
            }
            DspParamUpdate::Eq1BandGain(idx, gain_db) => {
                if idx < 10 {
                    let cur = self.eq_1.parametric_band(idx);
                    self.eq_1.set_parametric_band(idx, cur.freq_hz, gain_db, cur.q);
                }
            }
            DspParamUpdate::Eq1BandQ(idx, q) => {
                if idx < 10 {
                    let cur = self.eq_1.parametric_band(idx);
                    self.eq_1.set_parametric_band(idx, cur.freq_hz, cur.gain_db, q);
                }
            }
            DspParamUpdate::Eq1InputGainDb(db) => self.eq_1.set_input_gain_db(db),
            DspParamUpdate::Eq1OutputLevelDb(db) => self.eq_1.set_output_level_db(db),
            DspParamUpdate::Dly1Mode(m) => self.dly_1.set_re201_mode(m),
            DspParamUpdate::Dly1RepeatRate(r) => self.set_dly_1_repeat_rate(r),
            DspParamUpdate::Dly1Intensity(i) => self.set_dly_1_intensity(i),
            DspParamUpdate::Dly1EchoVolume(v) => self.set_dly_1_echo_volume(v),
            DspParamUpdate::Dly1ReverbVolume(v) => self.dly_1.set_reverb_volume(v),
            DspParamUpdate::Dly1Bass(v) => self.dly_1.set_tone_bass(v),
            DspParamUpdate::Dly1Treble(v) => self.dly_1.set_tone_treble(v),
            DspParamUpdate::Dly1Spillover(b) => self.dly_1.set_spillover_enabled(b),
            DspParamUpdate::Cab1Speaker(spk) => {
                self.cab.set_speaker_model(spk);
                self.amp.set_speaker_resonance_hz(self.cab.resonance_hz());
            }
            DspParamUpdate::Cab1Mic1AngleDeg(deg) => self.set_cab_mic_angle_deg(0, deg),
            DspParamUpdate::Cab1Mic2AngleDeg(deg) => self.set_cab_mic_angle_deg(1, deg),
            DspParamUpdate::Cab1Mic1PhaseInvert(inv) => self.cab.mic_slot_mut(0).phase_invert = inv,
            DspParamUpdate::Cab1Mic2PhaseInvert(inv) => self.cab.mic_slot_mut(1).phase_invert = inv,
            DspParamUpdate::TunerRefHz(hz) => self.set_tuner_ref_hz(hz),
            DspParamUpdate::LooperPosition(pos) => {
                self.looper_position = pos;
                self.looper.set_position(pos);
            }
            DspParamUpdate::LooperAction(cmd) => self.apply_looper_action_cmd(cmd),
            DspParamUpdate::Pitch1Enabled(b) => {
                self.pitch_1_enabled = b;
                self.pitch_1.set_bypassed(!b);
                self.shared_algo_latency_samples
                    .store(self.algorithmic_latency_samples(), Ordering::Relaxed);
            }
            DspParamUpdate::Pitch1Model(m) => self.pitch_1.set_model(m),
            DspParamUpdate::Pitch1Semitones(st) => self.pitch_1.set_semitones(st),
            DspParamUpdate::Pitch1Key(k) => self.pitch_1.set_key(k),
            DspParamUpdate::Pitch1Scale(s) => self.pitch_1.set_scale(s),
            DspParamUpdate::Pitch1Mix(mix) => {
                self.pitch_1.set_mix_pct(mix);
                self.shared_algo_latency_samples
                    .store(self.algorithmic_latency_samples(), Ordering::Relaxed);
            }
            DspParamUpdate::MasterLevelDb(db) => {
                if db.is_finite() {
                    self.master_level_db = db.clamp(-60.0, 12.0);
                }
            }
            DspParamUpdate::Cab2ResonanceHz(hz) => {
                if hz.is_finite() {
                    self.cab_2.set_resonance_hz(hz);
                }
            }
            DspParamUpdate::Cab2HicutKhz(khz) => {
                if khz.is_finite() {
                    self.cab_2.set_hicut_khz(khz);
                }
            }
            DspParamUpdate::Exp1Value(norm) => {
                if norm.is_finite() {
                    let n = norm.clamp(0.0, 1.0);
                    self.modifier_exp1 = n;
                    self.modifier_base_values[ModifierTargetParam::Wah1Position.index()] = n;
                    self.wah_1.set_position(n);
                }
            }
            DspParamUpdate::BeginPresetCrossfade | DspParamUpdate::TriggerPresetCrossfade => {}
            #[cfg(feature = "fault-injection")]
            DspParamUpdate::DebugInjectPanic => {
                self.inject_panic_next = true;
            }
        }
    }

    fn swap_dsp_state(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.gate, &mut other.gate);
        std::mem::swap(&mut self.gate_enabled, &mut other.gate_enabled);
        std::mem::swap(&mut self.drive, &mut other.drive);
        std::mem::swap(&mut self.drive_enabled, &mut other.drive_enabled);
        std::mem::swap(&mut self.amp, &mut other.amp);
        std::mem::swap(&mut self.amp_enabled, &mut other.amp_enabled);
        std::mem::swap(&mut self.cab, &mut other.cab);
        std::mem::swap(&mut self.cab_enabled, &mut other.cab_enabled);
        std::mem::swap(&mut self.input_boost, &mut other.input_boost);
        std::mem::swap(&mut self.input_mode_line, &mut other.input_mode_line);
        std::mem::swap(&mut self.gain_smoother, &mut other.gain_smoother);
        std::mem::swap(&mut self.cab_fade, &mut other.cab_fade);
        std::mem::swap(&mut self.cab_target_fade, &mut other.cab_target_fade);
        std::mem::swap(&mut self.split, &mut other.split);
        std::mem::swap(&mut self.split_enabled, &mut other.split_enabled);
        std::mem::swap(&mut self.split_mode, &mut other.split_mode);
        std::mem::swap(&mut self.active_topology, &mut other.active_topology);
        self.shared_topology_id
            .store(self.active_topology.id, Ordering::Release);
        std::mem::swap(&mut self.split_crossover_hz, &mut other.split_crossover_hz);
        std::mem::swap(&mut self.comp_2, &mut other.comp_2);
        std::mem::swap(&mut self.comp_2_enabled, &mut other.comp_2_enabled);
        std::mem::swap(&mut self.amp_2, &mut other.amp_2);
        std::mem::swap(&mut self.amp_2_enabled, &mut other.amp_2_enabled);
        std::mem::swap(&mut self.cab_2, &mut other.cab_2);
        std::mem::swap(&mut self.cab_2_enabled, &mut other.cab_2_enabled);
        std::mem::swap(&mut self.mod_2, &mut other.mod_2);
        std::mem::swap(&mut self.mod_2_enabled, &mut other.mod_2_enabled);
        std::mem::swap(&mut self.eq_1, &mut other.eq_1);
        std::mem::swap(&mut self.eq_1_enabled, &mut other.eq_1_enabled);
        std::mem::swap(&mut self.merge, &mut other.merge);
        std::mem::swap(&mut self.merge_row_1_level_db, &mut other.merge_row_1_level_db);
        std::mem::swap(&mut self.merge_row_1_phase_invert, &mut other.merge_row_1_phase_invert);
        std::mem::swap(&mut self.master_level_db, &mut other.master_level_db);
        std::mem::swap(&mut self.pitch_1, &mut other.pitch_1);
        std::mem::swap(&mut self.pitch_1_enabled, &mut other.pitch_1_enabled);
        std::mem::swap(&mut self.wah_1, &mut other.wah_1);
        std::mem::swap(&mut self.wah_1_enabled, &mut other.wah_1_enabled);
        std::mem::swap(&mut self.modifier_base_values, &mut other.modifier_base_values);
    }

    pub fn begin_preset_crossfade(&mut self) {
        if let Some(mut shadow) = self.shadow.take() {
            if !self.preset_crossfading {
                for i in 0..self.pending_shadow_len {
                    shadow.apply_param_update_inner(self.pending_shadow_sync[i]);
                }
                self.pending_shadow_len = 0;
                self.swap_dsp_state(&mut shadow);
            }
            self.shadow = Some(shadow);
        }
        // Spec 049 CA-2/CA-3: blocos de tempo são instância única; sem spillover, limpa a cauda anterior
        if !self.dly_1.is_spillover_enabled() {
            self.dly_1.clear_tail();
        }
        if !self.rev_1.is_spillover_enabled() {
            self.rev_1.clear_tail();
        }
        self.preset_crossfading = true;
        self.preset_fade = 0.0;
    }

    pub fn trigger_preset_crossfade(&mut self) {
        self.begin_preset_crossfade();
    }

    pub fn is_preset_crossfading(&self) -> bool {
        self.preset_crossfading
    }

    pub fn clean_input_ring(&self) -> Arc<CleanInputRing> {
        self.clean_input_ring.clone()
    }

    pub fn scope_output_ring(&self) -> Arc<CleanInputRing> {
        self.scope_output_ring.clone()
    }

    pub fn current_b_plus_v(&self) -> f32 {
        self.amp.current_b_plus_v()
    }

    pub fn set_tuner_enabled(&mut self, enabled: bool) {
        self.tuner_enabled = enabled;
        self.tuner.set_active(enabled);
        if enabled {
            self.tuner_start_pos = self.clean_input_ring.total_written();
        } else {
            self.tuner_note_bytes = *b"---\0\0\0\0\0";
            self.tuner_note_len = 3;
            self.tuner_cents = 0.0;
            self.tuner_freq_hz = 0.0;
        }
    }

    pub fn set_tuner_output_mode(&mut self, mode: TunerOutputMode) {
        self.tuner_output_mode = mode;
        self.tuner.set_output_mode(mode);
    }

    pub fn is_tuner_enabled(&self) -> bool {
        self.tuner_enabled
    }

    pub fn update_tuner_pitch(&mut self) {
        if self.tuner_enabled {
            let n = self
                .clean_input_ring
                .read_latest_since(self.tuner_start_pos, &mut self.tuner_scratch_read);
            if n >= 1024 {
                let res = self.tuner.detect_pitch(&self.tuner_scratch_read[..n]);
                let raw = res.note_name.as_bytes();
                let copy_len = raw.len().min(8);
                self.tuner_note_bytes = [0u8; 8];
                self.tuner_note_bytes[..copy_len].copy_from_slice(&raw[..copy_len]);
                self.tuner_note_len = copy_len as u8;
                self.tuner_cents = res.cents_deviation;
                self.tuner_freq_hz = res.frequency_hz;
            }
        }
    }

    pub fn tuner_latest_result(&mut self) -> TunerPitchResult {
        self.update_tuner_pitch();
        let len = (self.tuner_note_len as usize).min(self.tuner_note_bytes.len());
        let note_name = std::str::from_utf8(&self.tuner_note_bytes[..len])
            .unwrap_or("---")
            .to_string();
        TunerPitchResult {
            note_name,
            cents_deviation: self.tuner_cents,
            frequency_hz: self.tuner_freq_hz,
        }
    }

    pub fn input_source(&self) -> &str {
        self.input_source.as_str()
    }

    pub fn set_input_source(&mut self, source: &str) {
        self.input_source = InputSource::from_str_name(source);
    }

    pub fn load_di_wav(&mut self, samples: Vec<f32>) {
        self.di_buffer = samples;
        self.di_pos = 0;
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn prepare(&mut self, sample_rate: f32, max_block: usize) {
        self.sample_rate = sample_rate;
        self.shared_sample_rate_bits
            .store(sample_rate.max(1.0).to_bits(), Ordering::Relaxed);
        self.gate.set_sample_rate(sample_rate);
        self.drive.set_sample_rate(sample_rate);
        self.amp.set_sample_rate(sample_rate);
        self.cab.set_sample_rate(sample_rate);
        self.split.set_sample_rate(sample_rate);
        self.comp_2.set_sample_rate(sample_rate);
        self.amp_2.set_sample_rate(sample_rate);
        self.cab_2.set_sample_rate(sample_rate);
        self.mod_2.set_sample_rate(sample_rate);
        self.eq_1.set_sample_rate(sample_rate);
        self.dly_1.set_sample_rate(sample_rate);
        self.rev_1.set_sample_rate(sample_rate);
        self.pitch_1.prepare(sample_rate);
        self.wah_1.set_sample_rate(sample_rate);
        self.tuner.set_sample_rate(sample_rate);
        self.looper.prepare(sample_rate, 60);
        self.clean_input_ring.set_sample_rate(sample_rate);
        self.scope_output_ring.set_sample_rate(sample_rate);

        self.gain_smoother = ampforge_dsp::smoother::OnePoleSmoother::new(
            sample_rate as f64,
            0.007,
            self.gain_smoother.current_value(),
        );
        self.cpu_monitor
            .update_raw_hz(sample_rate as f64, max_block.max(1));
        let fade_len_samples = (sample_rate * 0.010).max(1.0);
        self.fade_step = 1.0 / fade_len_samples;
        let preset_fade_len_samples = (sample_rate * 0.020).max(1.0);
        self.preset_fade_step = 1.0 / preset_fade_len_samples;
        self.shared_algo_latency_samples
            .store(self.amp.algorithmic_latency_samples(), Ordering::Relaxed);

        let target_block = max_block.max(2048);
        if self.temp_boost_buf.len() < target_block {
            self.temp_boost_buf.resize(target_block, 0.0);
            self.temp_source_buf.resize(target_block, 0.0);
            self.temp_gate_buf.resize(target_block, 0.0);
            self.temp_drive_buf.resize(target_block, 0.0);
            self.temp_amp_buf.resize(target_block, 0.0);
            self.temp_split_a_buf.resize(target_block, 0.0);
            self.temp_split_b_buf.resize(target_block, 0.0);
            self.temp_cab_buf.resize(target_block, 0.0);
            self.temp_comp_buf.resize(target_block, 0.0);
            self.temp_amp2_buf.resize(target_block, 0.0);
            self.temp_cab2_buf.resize(target_block, 0.0);
            self.temp_mod_buf.resize(target_block, 0.0);
            self.temp_eq_buf.resize(target_block, 0.0);
            self.temp_merge_l_buf.resize(target_block, 0.0);
            self.temp_merge_r_buf.resize(target_block, 0.0);
            self.temp_delay_buf.resize(target_block, 0.0);
            self.temp_reverb_buf.resize(target_block, 0.0);
            self.temp_looper_buf.resize(target_block, 0.0);
            self.pre_cab_row0_buf.resize(target_block, 0.0);
            self.temp_dag_scratch_buf.resize(target_block, 0.0);
        }
        if self.shadow_out_buf.len() < target_block {
            self.shadow_out_buf.resize(target_block, 0.0);
        }
        if let Some(ref mut shadow) = self.shadow {
            shadow.prepare(sample_rate, max_block);
        }
        self.max_block = self.temp_boost_buf.len().max(1);
    }

    pub fn current_effective_gain(&self) -> f32 {
        self.gain_smoother.current_value() as f32
    }

    pub fn cpu_monitor(&self) -> &ampforge_dsp::cpu::CpuMonitor {
        &self.cpu_monitor
    }

    pub fn cpu_usage_pct(&self) -> f32 {
        self.cpu_monitor.cpu_usage_pct()
    }

    pub fn inject_nan_state_for_test(&mut self) {
        self.dc_x1 = f64::NAN;
        self.dc_y1 = f64::NAN;
    }

    pub fn set_gate_threshold_db(&mut self, db: f32) {
        if !db.is_finite() { return; }
        self.modifier_base_values[ModifierTargetParam::GateThresholdDb.index()] = db;
        self.gate.set_threshold_db(db);
    }

    pub fn set_gate_enabled(&mut self, enabled: bool) {
        self.gate_enabled = enabled;
    }

    pub fn set_gate_mode(&mut self, mode: &str) {
        let m = if mode.eq_ignore_ascii_case("classic") {
            NoiseGateMode::ClassicExpanderGate
        } else {
            NoiseGateMode::IntelligentNaturalGate
        };
        self.gate.set_mode(m);
    }

    pub fn set_gate_hysteresis_db(&mut self, db: f32) {
        if !db.is_finite() { return; }
        self.gate.set_hysteresis_db(db);
    }

    pub fn set_gate_attack_ms(&mut self, ms: f32) {
        if !ms.is_finite() { return; }
        self.gate.set_attack_ms(ms);
    }

    pub fn set_gate_hold_ms(&mut self, ms: f32) {
        if !ms.is_finite() { return; }
        self.gate.set_hold_ms(ms);
    }

    pub fn set_gate_release_ms(&mut self, ms: f32) {
        if !ms.is_finite() { return; }
        self.gate.set_release_ms(ms);
    }

    pub fn set_gate_range_db(&mut self, db: f32) {
        if !db.is_finite() { return; }
        self.gate.set_range_db(db);
    }

    pub fn set_gate_key_hpf_hz(&mut self, hz: f32) {
        if !hz.is_finite() { return; }
        self.gate.set_key_hpf_hz(hz);
    }

    pub fn set_gate_key_lpf_hz(&mut self, hz: f32) {
        if !hz.is_finite() { return; }
        self.gate.set_key_lpf_hz(hz);
    }

    pub fn gate_state(&self) -> GateState {
        if !self.gate_enabled {
            GateState::Open
        } else {
            self.gate.state()
        }
    }

    pub fn gate_gain_reduction_db(&self) -> f32 {
        if !self.gate_enabled {
            0.0
        } else {
            self.gate.gain_reduction_db()
        }
    }

    pub fn set_input_boost(&mut self, boost: f32) {
        if !boost.is_finite() { return; }
        self.input_boost = boost.clamp(0.1, 8.0);
    }

    pub fn set_amp_gain(&mut self, gain: f32) {
        if !gain.is_finite() { return; }
        self.modifier_base_values[ModifierTargetParam::Amp1Gain.index()] = gain;
        self.gain_smoother.set_target(gain as f64);
    }

    pub fn set_amp_overdrive(&mut self, od: f32) {
        if !od.is_finite() { return; }
        self.amp.panel_mut().set_overdrive(od);
    }

    pub fn set_amp_bass(&mut self, bass: f32) {
        if !bass.is_finite() { return; }
        self.modifier_base_values[ModifierTargetParam::Amp1Bass.index()] = bass;
        self.amp.panel_mut().set_bass(bass);
    }

    pub fn set_amp_middle(&mut self, mid: f32) {
        if !mid.is_finite() { return; }
        self.modifier_base_values[ModifierTargetParam::Amp1Middle.index()] = mid;
        self.amp.panel_mut().set_middle(mid);
    }

    pub fn set_amp_treble(&mut self, treb: f32) {
        if !treb.is_finite() { return; }
        self.modifier_base_values[ModifierTargetParam::Amp1Treble.index()] = treb;
        self.amp.panel_mut().set_treble(treb);
    }

    pub fn set_amp_presence(&mut self, pres: f32) {
        if !pres.is_finite() { return; }
        self.modifier_base_values[ModifierTargetParam::Amp1Presence.index()] = pres;
        self.amp.panel_mut().set_presence(pres);
    }

    pub fn set_amp_depth(&mut self, depth: f32) {
        if !depth.is_finite() { return; }
        self.amp.panel_mut().set_depth(depth);
    }

    pub fn set_amp_master(&mut self, master: f32) {
        if !master.is_finite() { return; }
        self.modifier_base_values[ModifierTargetParam::Amp1Master.index()] = master;
        self.amp.panel_mut().set_master_volume(master);
    }

    pub fn set_amp_bright(&mut self, bright: bool) {
        self.amp.panel_mut().bright_switch = bright;
    }

    pub fn set_amp_volume_2(&mut self, vol: f32) {
        if !vol.is_finite() { return; }
        self.amp.panel_mut().set_volume_2(vol);
    }

    pub fn set_amp_cut(&mut self, cut: f32) {
        if !cut.is_finite() { return; }
        self.amp.panel_mut().set_cut(cut);
    }

    pub fn set_amp_power(&mut self, power: bool) {
        self.amp.set_power(power);
    }

    pub fn set_amp_standby(&mut self, standby: bool) {
        self.amp.set_standby(standby);
    }

    pub fn set_amp_input_jack(&mut self, jack: &str) {
        if let Some(j) = AmpInputJack::from_str_name(jack) {
            self.amp.set_input_jack(j);
        }
    }

    pub fn set_amp_jumpered(&mut self, jumpered: bool) {
        self.amp.set_jumpered(jumpered);
    }

    pub fn set_amp_channel_mode(&mut self, mode: &str) {
        if let Some(m) = AmpChannelMode::from_str_name(mode) {
            self.amp.set_channel_mode(m);
        }
    }

    pub fn set_amp_spongy(&mut self, spongy: bool) {
        self.amp.set_spongy_power(spongy);
    }

    pub fn set_amp_reverb(&mut self, reverb: f32) {
        if !reverb.is_finite() { return; }
        self.amp.panel_mut().set_reverb(reverb);
    }

    pub fn set_amp_vibrato_speed(&mut self, speed: f32) {
        if !speed.is_finite() { return; }
        self.amp.panel_mut().set_vibrato_speed(speed);
    }

    pub fn set_amp_vibrato_depth(&mut self, depth: f32) {
        if !depth.is_finite() { return; }
        self.amp.panel_mut().set_vibrato_depth(depth);
    }

    pub fn set_amp_impedance_ohms(&mut self, ohms: &str) {
        let val = match ohms {
            "4" => 4,
            "8" => 8,
            _ => 16,
        };
        self.amp.set_secondary_impedance_ohms(val);
    }

    pub fn set_amp_model(&mut self, model_name: &str) {
        if let Some(m) = ampforge_dsp::amp::find_model_by_name(model_name) {
            self.amp.set_target_model(m);
        }
    }

    pub fn set_amp_preamp_tube(&mut self, tube_str: &str) {
        let t = match tube_str {
            "12at7" => ampforge_dsp::modshop::PreampTube::At7,
            "ef86_variant" | "ef86" => ampforge_dsp::modshop::PreampTube::Ef86,
            _ => ampforge_dsp::modshop::PreampTube::Ax7A,
        };
        self.amp.set_preamp_tube_type(t);
    }

    pub fn set_amp_power_tube(&mut self, tube_str: &str) {
        let t = match tube_str {
            "6l6gc" => ampforge_dsp::modshop::PowerTube::Tube6L6Gc,
            "kt88" => ampforge_dsp::modshop::PowerTube::Kt88,
            "el84_variant" | "el84" => ampforge_dsp::modshop::PowerTube::El84,
            _ => ampforge_dsp::modshop::PowerTube::El34,
        };
        self.amp.set_power_tube_type(t);
    }

    pub fn set_amp_bias_ma(&mut self, ma: f32) {
        if ma.is_finite() {
            self.amp.set_power_tube_bias_ma(ma);
        }
    }

    pub fn set_amp_sag(&mut self, sag: f32) {
        if sag.is_finite() {
            self.amp.set_sag_amount(sag);
        }
    }

    pub fn set_amp_rectifier(&mut self, rect_str: &str) {
        let r = match rect_str {
            "gz34" => ampforge_dsp::modshop::RectifierType::Gz34_5Ar4,
            "5u4gb" => ampforge_dsp::modshop::RectifierType::Dual5U4Gb,
            _ => ampforge_dsp::modshop::RectifierType::SolidStateDiode,
        };
        self.amp.set_rectifier_type(r);
    }

    pub fn set_amp_nfb(&mut self, nfb: f32) {
        if nfb.is_finite() {
            self.amp.set_negative_feedback(nfb);
        }
    }

    pub fn set_amp_xformer_sat(&mut self, sat: f32) {
        if sat.is_finite() {
            self.amp.set_transformer_saturation(sat);
        }
    }

    pub fn set_amp_preamp_plate_v(&mut self, v: f32) {
        if v.is_finite() {
            self.amp.set_preamp_plate_voltage(v);
        }
    }

    pub fn set_amp_triode_asym(&mut self, asym: f32) {
        if asym.is_finite() {
            self.amp.set_triode_asymmetry(asym);
        }
    }

    pub fn set_amp_tonestack(&mut self, ts_str: &str) {
        let ts = match ts_str {
            "marshall" => ampforge_dsp::modshop::ToneStackTopology::MarshallCathodeFollower,
            "fender" => ampforge_dsp::modshop::ToneStackTopology::FenderTmbMidScoop,
            "vox" => ampforge_dsp::modshop::ToneStackTopology::VoxTopBoost,
            "baxandall" => ampforge_dsp::modshop::ToneStackTopology::PassiveJamesBaxandall,
            _ => ampforge_dsp::modshop::ToneStackTopology::Model,
        };
        self.amp.set_tonestack_topology(ts);
    }

    pub fn amp(&self) -> &WhiteBoxAmpProcessor {
        &self.amp
    }

    pub fn amp_mut(&mut self) -> &mut WhiteBoxAmpProcessor {
        &mut self.amp
    }

    pub fn set_oversampling_factor(&mut self, factor_str: &str) {
        let factor = match factor_str {
            "2x" => 2,
            "4x" => 4,
            _ => 8,
        };
        self.amp.set_oversampling_factor(factor);
    }

    pub fn algorithmic_latency_samples(&self) -> u32 {
        self.amp.algorithmic_latency_samples() + self.pitch_1.algorithmic_latency_samples() as u32
    }

    pub fn set_amp_enabled(&mut self, enabled: bool) {
        self.amp_enabled = enabled;
    }

    pub fn set_drive_enabled(&mut self, enabled: bool) {
        self.drive_enabled = enabled;
    }

    pub fn set_drive_gain(&mut self, gain: f32) {
        if !gain.is_finite() { return; }
        self.modifier_base_values[ModifierTargetParam::DriveDrive.index()] = gain;
        self.drive.set_gain(gain);
    }

    pub fn set_drive_tone(&mut self, tone: f32) {
        if !tone.is_finite() { return; }
        self.drive.set_tone(tone);
    }

    pub fn set_drive_level(&mut self, level: f32) {
        if !level.is_finite() { return; }
        self.modifier_base_values[ModifierTargetParam::DriveLevel.index()] = level;
        self.drive.set_level(level);
    }

    pub fn set_drive_model(&mut self, model: &str) {
        if let Some(m) = DriveModel::from_str_name(model) {
            self.drive.set_model(m);
        }
    }

    pub fn set_drive_bias(&mut self, bias: f32) {
        if !bias.is_finite() { return; }
        self.drive.set_bias(bias);
    }

    pub fn drive(&self) -> &DriveProcessor {
        &self.drive
    }

    pub fn drive_mut(&mut self) -> &mut DriveProcessor {
        &mut self.drive
    }

    pub fn set_cab_enabled(&mut self, enabled: bool) {
        self.cab_enabled = enabled;
        self.cab_target_fade = if enabled { 1.0 } else { 0.0 };
    }

    pub fn load_cab_ir(&mut self, bytes: &[u8]) -> Result<(), ampforge_dsp::cab::InvalidImpulseResponseError> {
        self.cab.load_custom_ir(bytes)
    }

    pub fn set_cab_speaker(&mut self, speaker: &str) {
        let model = match speaker {
            "greenback_4x12" | "greenback_4x12_closed" => SpeakerCabModel::Greenback4x12Closed,
            "v30_4x12" | "v30_4x12_closed" => SpeakerCabModel::V304x12Closed,
            "alnico_blue_2x12" | "alnico_blue_2x12_open" => SpeakerCabModel::AlnicoBlue2x12Open,
            "twin_2x12_open" | "twin_2x12" | "jensen_c12n_2x12_open" => SpeakerCabModel::Twin2x12Open,
            _ => SpeakerCabModel::Greenback4x12Closed,
        };
        self.cab.set_speaker_model(model);
    }

    pub fn set_cab_resonance_hz(&mut self, hz: f32) {
        self.cab.set_resonance_hz(hz);
        self.amp.set_speaker_resonance_hz(self.cab.resonance_hz());
    }

    pub fn set_cab_hicut_khz(&mut self, khz: f32) {
        self.cab.set_hicut_khz(khz);
    }

    pub fn set_cab_ir_enabled(&mut self, enabled: bool) {
        self.cab.set_ir_enabled(enabled);
    }

    pub fn set_cab_mic1_model(&mut self, model: &str) {
        let m = match model {
            "sm57" => ampforge_dsp::cab::MicModel::Sm57,
            "md421" => ampforge_dsp::cab::MicModel::Md421,
            "m88" => ampforge_dsp::cab::MicModel::M88,
            "r121" => ampforge_dsp::cab::MicModel::R121,
            "m160" => ampforge_dsp::cab::MicModel::M160,
            "u87" => ampforge_dsp::cab::MicModel::U87,
            "c414" => ampforge_dsp::cab::MicModel::C414,
            _ => ampforge_dsp::cab::MicModel::Sm57,
        };
        self.cab.mic_slot_mut(0).model = m;
    }

    pub fn set_cab_mic1_position(&mut self, pos: f32) {
        self.cab.mic_slot_mut(0).position_cap_to_edge = pos.clamp(0.0, 1.0);
    }

    pub fn set_cab_mic1_distance_cm(&mut self, dist: f32) {
        self.cab.mic_slot_mut(0).distance_cm = dist.clamp(0.0, 30.0);
    }

    pub fn set_cab_mic1_level_db(&mut self, db: f32) {
        self.cab.mic_slot_mut(0).level_db = db.clamp(-60.0, 12.0);
    }

    pub fn set_cab_mic1_enabled(&mut self, enabled: bool) {
        self.cab.mic_slot_mut(0).enabled = enabled;
    }

    pub fn set_cab_mic2_model(&mut self, model: &str) {
        let m = match model {
            "sm57" => ampforge_dsp::cab::MicModel::Sm57,
            "md421" => ampforge_dsp::cab::MicModel::Md421,
            "m88" => ampforge_dsp::cab::MicModel::M88,
            "r121" => ampforge_dsp::cab::MicModel::R121,
            "m160" => ampforge_dsp::cab::MicModel::M160,
            "u87" => ampforge_dsp::cab::MicModel::U87,
            "c414" => ampforge_dsp::cab::MicModel::C414,
            _ => ampforge_dsp::cab::MicModel::Sm57,
        };
        self.cab.mic_slot_mut(1).model = m;
    }

    pub fn set_cab_mic2_position(&mut self, pos: f32) {
        self.cab.mic_slot_mut(1).position_cap_to_edge = pos.clamp(0.0, 1.0);
    }

    pub fn set_cab_mic2_distance_cm(&mut self, dist: f32) {
        self.cab.mic_slot_mut(1).distance_cm = dist.clamp(0.0, 30.0);
    }

    pub fn set_cab_mic2_level_db(&mut self, db: f32) {
        self.cab.mic_slot_mut(1).level_db = db.clamp(-60.0, 12.0);
    }

    pub fn set_cab_mic2_enabled(&mut self, enabled: bool) {
        self.cab.mic_slot_mut(1).enabled = enabled;
    }

    pub fn set_cab_mic_angle_deg(&mut self, slot: usize, deg: f32) {
        if slot < 2 && deg.is_finite() {
            self.cab.mic_slot_mut(slot).angle_off_axis_deg = if deg >= 22.5 { 45.0 } else { 0.0 };
        }
    }

    pub fn set_cab_mic_phase_invert(&mut self, slot: usize, invert: bool) {
        if slot < 2 {
            self.cab.mic_slot_mut(slot).phase_invert = invert;
        }
    }

    // Spec 024: Setters e Getters dos Novos Blocos e Caminho Paralelo
    pub fn set_split_enabled(&mut self, enabled: bool) {
        self.split_enabled = enabled;
    }

    fn apply_split_config(&mut self) {
        match self.split_mode {
            ChainSplitMode::Crossover => {
                self.split.set_mode(SplitMode::Crossover { freq_hz: self.split_crossover_hz });
            }
            ChainSplitMode::DynamicSplit => {
                self.split.set_mode(SplitMode::DynamicSplit {
                    threshold_db: -30.0,
                    attack_ms: 5.0,
                    release_ms: 80.0,
                });
            }
            ChainSplitMode::Ab => {
                self.split.set_mode(SplitMode::BalanceAb { ratio: 0.5 });
            }
        }
    }

    pub fn set_split_mode(&mut self, mode: &str, crossover_hz: f32) {
        self.split_mode = ChainSplitMode::from_str_name(mode);
        self.split_crossover_hz = crossover_hz.clamp(100.0, 2000.0);
        self.apply_split_config();
    }

    pub fn set_merge_config(&mut self, level_db: f32, phase_invert: bool) {
        self.merge_row_1_level_db = level_db.clamp(-60.0, 6.0);
        self.merge_row_1_phase_invert = phase_invert;
        let mut cfg = MixerChannelConfig::default();
        cfg.level_db = self.merge_row_1_level_db;
        cfg.phase_invert = self.merge_row_1_phase_invert;
        self.merge.configure_channel(1, cfg);
    }

    pub fn set_comp_2_enabled(&mut self, enabled: bool) {
        self.comp_2_enabled = enabled;
    }

    pub fn set_comp_2_input(&mut self, input: f32) {
        let pre_gain_db = (input.clamp(0.0, 10.0) - 5.0) * 4.0;
        self.comp_2.set_threshold_db(-20.0 - pre_gain_db);
    }

    pub fn set_comp_2_output(&mut self, output: f32) {
        if !output.is_finite() { return; }
        let gain_db = (output.clamp(0.0, 10.0) - 5.0) * 4.8;
        self.comp_2.set_makeup_gain_db(gain_db);
    }

    pub fn set_comp_2_attack(&mut self, att: f32) {
        let att_clamped = att.clamp(1.0, 7.0);
        let attack_ms = 0.8f32 * (0.02f32 / 0.8f32).powf((att_clamped - 1.0) / 6.0);
        self.comp_2.set_attack_ms(attack_ms);
    }

    pub fn set_comp_2_release(&mut self, rel: f32) {
        let rel_clamped = rel.clamp(1.0, 7.0);
        let release_ms = 1100.0f32 * (50.0f32 / 1100.0f32).powf((rel_clamped - 1.0) / 6.0);
        self.comp_2.set_release_ms(release_ms);
    }

    pub fn set_comp_2_ratio(&mut self, ratio_str: &str) {
        let (r, all_in) = match ratio_str {
            "4" => (4.0, false),
            "8" => (8.0, false),
            "12" => (12.0, false),
            "20" => (20.0, false),
            "all" => (20.0, true),
            _ => (4.0, false),
        };
        self.comp_2.set_ratio(r);
        self.comp_2.set_all_buttons_in(all_in);
    }

    pub fn set_eq_1_enabled(&mut self, enabled: bool) {
        self.eq_1_enabled = enabled;
    }

    pub fn set_eq_1_mode(&mut self, mode: &str) {
        let m = match mode {
            "graphic_5_boogie" | "boogie_5" => EqMode::Graphic5BandBoogie,
            "graphic_10_mxr" | "mxr_10band" => EqMode::Graphic10BandPedal,
            _ => EqMode::Parametric8Band,
        };
        self.eq_1.set_mode(m);
    }

    pub fn set_eq_1_input_gain_db(&mut self, db: f32) {
        self.eq_1.set_input_gain_db(db);
    }

    pub fn set_eq_1_output_level_db(&mut self, db: f32) {
        self.eq_1.set_output_level_db(db);
    }

    pub fn set_eq_1_band(&mut self, band_idx: usize, freq_hz: f32, gain_db: f32, q: f32) {
        self.eq_1.set_parametric_band(band_idx, freq_hz, gain_db, q);
    }

    pub fn eq_1_band(&self, band_idx: usize) -> ampforge_dsp::fx::eq::ParametricBand {
        self.eq_1.parametric_band(band_idx)
    }

    pub fn set_mod_2_enabled(&mut self, enabled: bool) {
        self.mod_2_enabled = enabled;
    }

    pub fn set_mod_2_model(&mut self, model: ModModel) {
        self.mod_2.set_model(model);
    }

    pub fn set_mod_2_rate_hz(&mut self, rate: f32) {
        self.mod_2.set_rate_hz(rate);
    }

    pub fn set_mod_2_depth(&mut self, depth: f32) {
        self.mod_2.set_depth(depth);
    }

    pub fn set_mod_2_mix(&mut self, mix: f32) {
        self.mod_2.set_mix_pct(mix);
    }

    pub fn set_dly_1_enabled(&mut self, enabled: bool) {
        self.dly_1_enabled = enabled;
        self.dly_1.set_bypassed(!enabled);
    }

    pub fn set_dly_1_repeat_rate(&mut self, rate: f32) {
        let speed = (rate.clamp(0.0, 10.0) / 10.0) * 1.2 + 0.6;
        let base_delay_ms = 350.0;
        self.dly_1.set_delay_time_ms(base_delay_ms / speed);
    }

    pub fn set_dly_1_intensity(&mut self, intensity: f32) {
        self.dly_1.set_feedback_pct(intensity.clamp(0.0, 10.0) * 10.0);
    }

    pub fn set_dly_1_echo_volume(&mut self, vol: f32) {
        self.dly_1.set_mix_pct(vol.clamp(0.0, 10.0) * 10.0);
    }

    pub fn set_dly_1_spillover(&mut self, enabled: bool) {
        self.dly_1.set_spillover_enabled(enabled);
    }

    pub fn set_dly_1_mode(&mut self, mode: &str) {
        if let Ok(m) = mode.parse::<u8>() {
            self.dly_1.set_re201_mode(m);
        }
    }

    pub fn set_dly_1_reverb_volume(&mut self, vol: f32) {
        self.dly_1.set_reverb_volume(vol);
    }

    pub fn set_dly_1_bass(&mut self, bass: f32) {
        self.dly_1.set_tone_bass(bass);
    }

    pub fn set_dly_1_treble(&mut self, treble: f32) {
        self.dly_1.set_tone_treble(treble);
    }

    pub fn set_dly_1_tempo_bpm(&mut self, bpm: f32) {
        self.dly_1.set_tempo_bpm(bpm);
    }

    pub fn dly_1_tempo_bpm(&self) -> f32 {
        self.dly_1.tempo_bpm()
    }

    pub fn set_tuner_ref_hz(&mut self, hz: f32) {
        self.tuner.set_a4_reference_hz(hz);
    }

    pub fn set_looper_position(&mut self, pos: &str) {
        let p = if pos.eq_ignore_ascii_case("pre") {
            LooperPosition::PreChain
        } else {
            LooperPosition::PostChain
        };
        self.looper_position = p;
        self.looper.set_position(p);
    }

    pub fn apply_looper_action_cmd(&mut self, cmd: LooperActionCmd) {
        match cmd {
            LooperActionCmd::Record => self.looper.record(),
            LooperActionCmd::Play => self.looper.play(),
            LooperActionCmd::Overdub => self.looper.overdub(),
            LooperActionCmd::Stop => self.looper.stop(),
            LooperActionCmd::Undo => self.looper.undo(),
            LooperActionCmd::Redo => self.looper.redo(),
            LooperActionCmd::Reverse => self.looper.set_reverse(!self.looper.is_reverse()),
            LooperActionCmd::HalfSpeed => self.looper.set_half_speed(!self.looper.is_half_speed()),
            LooperActionCmd::Clear => self.looper.clear(),
        }
    }

    pub fn trigger_looper_action(&mut self, action: &str) {
        let cmd = match action {
            "record" | "rec" => Some(LooperActionCmd::Record),
            "play" => Some(LooperActionCmd::Play),
            "overdub" => Some(LooperActionCmd::Overdub),
            "stop" => Some(LooperActionCmd::Stop),
            "undo" => Some(LooperActionCmd::Undo),
            "redo" => Some(LooperActionCmd::Redo),
            "reverse" => Some(LooperActionCmd::Reverse),
            "half_speed" => Some(LooperActionCmd::HalfSpeed),
            "clear" => Some(LooperActionCmd::Clear),
            _ => None,
        };
        if let Some(c) = cmd {
            self.apply_looper_action_cmd(c);
        }
    }

    pub fn set_rev_1_enabled(&mut self, enabled: bool) {
        self.rev_1_enabled = enabled;
        self.rev_1.set_bypassed(!enabled);
    }

    pub fn set_rev_1_decay_s(&mut self, decay: f32) {
        self.rev_1.set_decay_time_s(decay);
    }

    pub fn set_rev_1_predelay_ms(&mut self, predelay: f32) {
        self.rev_1.set_predelay_ms(predelay);
    }

    pub fn set_rev_1_damping(&mut self, damp: f32) {
        self.rev_1.set_damping(damp);
    }

    pub fn set_rev_1_mix(&mut self, mix: f32) {
        if mix.is_finite() {
            self.modifier_base_values[ModifierTargetParam::Rev1Mix.index()] = mix;
        }
        self.rev_1.set_mix_pct(mix);
    }

    pub fn set_rev_1_spillover(&mut self, enabled: bool) {
        self.rev_1.set_spillover_enabled(enabled);
    }

    pub fn set_amp_2_enabled(&mut self, enabled: bool) {
        self.amp_2_enabled = enabled;
    }

    pub fn set_amp_2_model(&mut self, model_name: &str) {
        if let Some(m) = ampforge_dsp::amp::find_model_by_name(model_name) {
            self.amp_2.set_target_model(m);
        }
    }

    pub fn amp_2_mut(&mut self) -> &mut WhiteBoxAmpProcessor {
        &mut self.amp_2
    }

    pub fn set_cab_2_enabled(&mut self, enabled: bool) {
        self.cab_2_enabled = enabled;
    }

    pub fn set_cab_2_speaker(&mut self, speaker: &str) {
        let model = match speaker {
            "greenback_4x12" | "greenback_4x12_closed" => SpeakerCabModel::Greenback4x12Closed,
            "v30_4x12" | "v30_4x12_closed" => SpeakerCabModel::V304x12Closed,
            "alnico_blue_2x12" | "alnico_blue_2x12_open" => SpeakerCabModel::AlnicoBlue2x12Open,
            "twin_2x12_open" | "twin_2x12" | "jensen_c12n_2x12_open" => SpeakerCabModel::Twin2x12Open,
            _ => SpeakerCabModel::AlnicoBlue2x12Open,
        };
        self.cab_2.set_speaker_model(model);
    }

    pub fn cab_2_mut(&mut self) -> &mut VirtualCabProcessor {
        &mut self.cab_2
    }

    /// Processa bloco de áudio da guitarra passando pelo Grafo Completo do Grid (Spec 024 & Spec 037 Zero-Alloc)
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let _ftz = ampforge_dsp::denormal::DenormalGuard::new();
        self.mxcsr_no_process = ampforge_dsp::denormal::mxcsr_atual();
        let rt_start = std::time::Instant::now();
        self.drain_param_queue();
        self.drain_resource_queue();
        #[cfg(feature = "fault-injection")]
        if self.inject_panic_next {
            self.inject_panic_next = false;
            panic!("AUD-04 fault-injection: debug.inject_panic");
        }

        let total = input.len().min(output.len());
        let chunk_max = self.max_block.max(1);
        let mut offset = 0usize;
        while offset < total {
            let n = (total - offset).min(chunk_max);
            self.process_chunk(&input[offset..offset + n], &mut output[offset..offset + n]);
            offset += n;
        }

        if total > 0 {
            let elapsed_micros = (rt_start.elapsed().as_micros().max(1)).min(u64::MAX as u128) as u64;
            self.cpu_monitor.record_execution_time_micros(elapsed_micros);
        }
        self.shared_b_plus_v_bits
            .store(self.amp.current_b_plus_v().to_bits(), Ordering::Relaxed);
    }

    fn process_chunk(&mut self, input: &[f32], output: &mut [f32]) {
        let len = input.len();
        if len == 0 {
            return;
        }
        self.channel_process_counts[self.active_channel_idx] += 1;
        self.evaluate_modifiers_for_block(input);

        // 0. Contabiliza clipping no conversor de entrada (|x| >= 0.999) (Spec 019, CA-4)
        for &s in input {
            if s.abs() >= 0.999 {
                self.adc_clips.fetch_add(1, Ordering::Relaxed);
            }
        }

        // Seleção de fonte de entrada (Spec 019, CA-5): device, test_riff ou di_file
        let source_slice = &mut self.temp_source_buf[..len];
        if self.input_source == InputSource::TestRiff && !self.test_riff_buffer.is_empty() {
            let riff_len = self.test_riff_buffer.len();
            for s in source_slice.iter_mut() {
                *s = self.test_riff_buffer[self.test_riff_pos];
                self.test_riff_pos = (self.test_riff_pos + 1) % riff_len;
            }
        } else if self.input_source == InputSource::DiFile && !self.di_buffer.is_empty() {
            let di_len = self.di_buffer.len();
            for s in source_slice.iter_mut() {
                *s = self.di_buffer[self.di_pos];
                self.di_pos = (self.di_pos + 1) % di_len;
            }
        } else {
            source_slice.copy_from_slice(input);
        }

        // 1. Entrada pós-boost pronta para processamento (suavização ocorre na aplicação real do amp)
        // (Spec 044 CA-4: sem loop morto prévio de gain_smoother)

        // 2. DC Blocker de 1 polo em f64 (~0.08 Hz: plano em ±0.0001 dB @ 40 Hz, < -86 dBFS de resíduo em transientes)
        let r = 0.99999_f64;
        let mut ring_pos = self.clean_input_ring.total_written_relaxed();
        for i in 0..len {
            let x = source_slice[i] as f64;
            let y = x - self.dc_x1 + r * self.dc_y1;
            self.dc_x1 = ampforge_dsp::denormal::flush_f64(x);
            self.dc_y1 = ampforge_dsp::denormal::flush_f64(y);
            let yf = y as f32;
            source_slice[i] = yf;

            // Spec 026 / 029: Copia sinal limpo no ring SPSC lock-free (0 alocações, 0 locks)
            self.clean_input_ring.push_sample_at(ring_pos, yf);
            ring_pos = ring_pos.wrapping_add(1);
        }
        self.clean_input_ring.commit_write_pos(ring_pos);

        // Looper PreChain (instância única fora do par ativa/sombra, Spec 049 CA-1)
        if self.looper_position == LooperPosition::PreChain {
            let pre_looper_slice = &mut self.temp_looper_buf[..len];
            self.looper.process(&self.temp_source_buf[..len], pre_looper_slice);
            self.temp_source_buf[..len].copy_from_slice(pre_looper_slice);
        }

        if self.preset_crossfading {
            if let Some(mut shadow) = self.shadow.take() {
                shadow.sync_modifiers_from(&self.active_modifiers, &self.modifier_outputs);
                let clean = &self.temp_source_buf[..len];
                let shadow_out = &mut self.shadow_out_buf[..len];
                shadow.process_tonal_core(clean, shadow_out);
                self.shadow = Some(shadow);
            }
        }

        // Processa cadeia tonal ativa diretamente para `self.temp_merge_l_buf[..len]`
        self.process_tonal_core_from_source(len);

        // Retira IRs aposentadas de self e shadow
        if let Some(old_ir) = self.cab.take_retired_ir() {
            self.retired_reaper.retire_ir(old_ir);
        }
        if let Some(old_ir) = self.cab_2.take_retired_ir() {
            self.retired_reaper.retire_ir(old_ir);
        }

        // Crossfade real de preset/cena apenas na cadeia tonal (20 ms equal-power)
        for i in 0..len {
            if self.preset_crossfading {
                let old_val = self.shadow_out_buf[i];
                let new_val = self.temp_merge_l_buf[i];
                let angle = 0.5 * std::f32::consts::PI * self.preset_fade.clamp(0.0, 1.0);
                let w_old = angle.cos();
                let w_new = angle.sin();
                self.temp_merge_l_buf[i] = if old_val.is_finite() && new_val.is_finite() {
                    old_val * w_old + new_val * w_new
                } else if new_val.is_finite() {
                    new_val * w_new
                } else {
                    0.0
                };
                self.preset_fade += self.preset_fade_step;
                if self.preset_fade >= 1.0 {
                    self.preset_fade = 1.0;
                    self.preset_crossfading = false;
                }
            }
        }

        // 8. Pós-Merge (instância única): Delay 1 -> Reverb 1 -> Looper PostChain (Spec 049 CA-1..CA-3)
        self.dly_1
            .process(&self.temp_merge_l_buf[..len], &mut self.temp_delay_buf[..len]);
        self.rev_1
            .process(&self.temp_delay_buf[..len], &mut self.temp_reverb_buf[..len]);

        if self.looper_position == LooperPosition::PostChain {
            self.looper
                .process(&self.temp_reverb_buf[..len], &mut output[..len]);
        } else {
            output[..len].copy_from_slice(&self.temp_reverb_buf[..len]);
        }

        // 8b. Trim de Nível Master de saída (Spec 051, CA-11)
        if self.master_level_db.abs() > 1e-4 {
            let master_gain = if self.master_level_db <= -59.9 {
                0.0
            } else {
                10.0f32.powf(self.master_level_db / 20.0)
            };
            for s in output[..len].iter_mut() {
                *s *= master_gain;
            }
        }

        // 9. Limitador de Saída Segura + defesa NaN/Inf
        let mut has_non_finite = false;
        for s in output[..len].iter_mut() {
            let val = *s;
            if !val.is_finite() {
                *s = 0.0;
                has_non_finite = true;
            } else {
                *s = val.clamp(-1.0, 1.0);
            }
        }

        // Spec 047 CA-2: Captura lock-free e zero-alloc da saída do motor para o CRT Waveform
        let mut out_pos = self.scope_output_ring.total_written_relaxed();
        for &s in &output[..len] {
            self.scope_output_ring.push_sample_at(out_pos, s);
            out_pos = out_pos.wrapping_add(1);
        }
        self.scope_output_ring.commit_write_pos(out_pos);

        if !self.preset_crossfading && self.pending_shadow_len > 0 {
            if let Some(mut shadow) = self.shadow.take() {
                for i in 0..self.pending_shadow_len {
                    shadow.apply_param_update_inner(self.pending_shadow_sync[i]);
                }
                self.pending_shadow_len = 0;
                self.shadow = Some(shadow);
            }
        }

        // Spec 026: Rampa de silenciamento do afinador de 5 ms (CA-4)
        if self.tuner_enabled && self.tuner_output_mode == TunerOutputMode::Mute {
            let ramp_step = 1.0 / (0.005 * self.sample_rate);
            for s in output[..len].iter_mut() {
                self.tuner_mute_gain = (self.tuner_mute_gain - ramp_step).max(0.0);
                *s *= self.tuner_mute_gain;
            }
        } else if self.tuner_mute_gain < 1.0 {
            let ramp_step = 1.0 / (0.005 * self.sample_rate);
            for s in output[..len].iter_mut() {
                self.tuner_mute_gain = (self.tuner_mute_gain + ramp_step).min(1.0);
                *s *= self.tuner_mute_gain;
            }
        }

        if has_non_finite {
            self.dc_x1 = 0.0;
            self.dc_x2 = 0.0;
            self.dc_y1 = 0.0;
            self.dc_y2 = 0.0;
            self.cab_fade = self.cab_target_fade;
        }
    }

    #[inline]
    fn effective_input_gain(&self) -> f32 {
        // CA-38: -10 dBV (0.316228 V) no modo instrument vs +4 dBu (1.2276 V) no modo line => pad -11.78 dB (0.2576x)
        if self.input_mode_line {
            self.input_boost * 0.257_598_55
        } else {
            self.input_boost
        }
    }

    fn process_tonal_core_from_source(&mut self, len: usize) {
        let g = self.effective_input_gain();
        let boost_slice = &mut self.temp_boost_buf[..len];
        for i in 0..len {
            boost_slice[i] = self.temp_source_buf[i] * g;
        }
        self.process_tonal_core_inner(len);
    }

    fn process_tonal_core(&mut self, clean_input: &[f32], output: &mut [f32]) {
        let len = clean_input.len().min(output.len());
        if len == 0 {
            return;
        }
        let g = self.effective_input_gain();
        self.temp_source_buf[..len].copy_from_slice(&clean_input[..len]);
        let boost_slice = &mut self.temp_boost_buf[..len];
        for i in 0..len {
            boost_slice[i] = clean_input[i] * g;
        }
        self.process_tonal_core_inner(len);
        output[..len].copy_from_slice(&self.temp_merge_l_buf[..len]);
    }

    #[inline]
    fn execute_dag_block(&mut self, kind: DagBlockKind, in_is_row1: bool, len: usize) -> bool {
        let input_slice: &[f32] = if in_is_row1 {
            &self.temp_split_b_buf[..len]
        } else {
            &self.temp_merge_l_buf[..len]
        };
        let output_slice = &mut self.temp_dag_scratch_buf[..len];
        let mut ran_amp1 = false;
        match kind {
            DagBlockKind::Gate => {
                if self.gate_enabled {
                    self.gate
                        .process(input_slice, Some(&self.temp_source_buf[..len]), output_slice);
                } else {
                    output_slice.copy_from_slice(input_slice);
                }
            }
            DagBlockKind::Wah1 => {
                if self.wah_1_enabled {
                    self.wah_1.process(input_slice, output_slice);
                } else {
                    output_slice.copy_from_slice(input_slice);
                }
            }
            DagBlockKind::Pitch1 => {
                if self.pitch_1_enabled {
                    self.pitch_1.process(input_slice, output_slice);
                } else {
                    output_slice.copy_from_slice(input_slice);
                }
            }
            DagBlockKind::Drive => {
                if self.drive_enabled {
                    self.drive.process(input_slice, output_slice);
                } else {
                    output_slice.copy_from_slice(input_slice);
                }
            }
            DagBlockKind::Amp1 => {
                ran_amp1 = true;
                if self.amp_enabled {
                    let mut sub_off = 0usize;
                    while sub_off < len {
                        let sub_end = (sub_off + 16).min(len);
                        let mut g = self.gain_smoother.current_value();
                        for _ in sub_off..sub_end {
                            g = self.gain_smoother.next_sample();
                        }
                        self.amp.panel_mut().set_input_gain(g as f32);
                        self.amp
                            .process(&input_slice[sub_off..sub_end], &mut output_slice[sub_off..sub_end]);
                        sub_off = sub_end;
                    }
                } else {
                    for _ in 0..len {
                        self.gain_smoother.next_sample();
                    }
                    output_slice.copy_from_slice(input_slice);
                }
            }
            DagBlockKind::Cab1 => {
                self.pre_cab_row0_buf[..len].copy_from_slice(input_slice);
                if (self.cab_fade - self.cab_target_fade).abs() > 1e-4 {
                    self.cab.process(input_slice, output_slice);
                    for i in 0..len {
                        if self.cab_fade < self.cab_target_fade {
                            self.cab_fade = (self.cab_fade + self.fade_step).min(self.cab_target_fade);
                        } else {
                            self.cab_fade = (self.cab_fade - self.fade_step).max(self.cab_target_fade);
                        }
                        output_slice[i] =
                            input_slice[i] * (1.0 - self.cab_fade) + output_slice[i] * self.cab_fade;
                    }
                } else if self.cab_enabled {
                    self.cab.process(input_slice, output_slice);
                } else {
                    output_slice.copy_from_slice(input_slice);
                }
            }
            DagBlockKind::Eq1 => {
                if self.eq_1_enabled {
                    self.eq_1.process(input_slice, output_slice);
                } else {
                    output_slice.copy_from_slice(input_slice);
                }
            }
            DagBlockKind::Comp2 => {
                if self.comp_2_enabled {
                    self.comp_2.process(input_slice, output_slice);
                } else {
                    output_slice.copy_from_slice(input_slice);
                }
            }
            DagBlockKind::Amp2 => {
                if self.amp_2_enabled {
                    self.amp_2.process(input_slice, output_slice);
                } else {
                    output_slice.copy_from_slice(input_slice);
                }
            }
            DagBlockKind::Cab2 => {
                if self.cab_2_enabled {
                    self.cab_2.process(input_slice, output_slice);
                } else {
                    output_slice.copy_from_slice(input_slice);
                }
            }
            DagBlockKind::Mod2 => {
                if self.mod_2_enabled {
                    self.mod_2.process(input_slice, output_slice);
                } else {
                    output_slice.copy_from_slice(input_slice);
                }
            }
            DagBlockKind::Split | DagBlockKind::Merge | DagBlockKind::Dly1 | DagBlockKind::Rev1 => {
                return false;
            }
        }
        if in_is_row1 {
            self.temp_split_b_buf[..len].copy_from_slice(&self.temp_dag_scratch_buf[..len]);
        } else {
            self.temp_merge_l_buf[..len].copy_from_slice(&self.temp_dag_scratch_buf[..len]);
        }
        ran_amp1
    }

    fn process_tonal_core_inner(&mut self, len: usize) {
        self.temp_merge_l_buf[..len].copy_from_slice(&self.temp_boost_buf[..len]);
        let topo = *self.active_topology;

        let mut split_col = None;
        let mut first_r1_col = None;
        let mut explicit_merge_col = None;
        for col in 0..12usize {
            if split_col.is_none() && matches!(topo.slots[col], Some(DagBlockKind::Split)) {
                split_col = Some(col);
            }
            if first_r1_col.is_none() && topo.slots[12 + col].is_some() {
                first_r1_col = Some(col);
            }
            if matches!(topo.slots[col], Some(DagBlockKind::Merge)) {
                explicit_merge_col = Some(col);
            }
        }
        let effective_split_col = split_col.or(first_r1_col).unwrap_or(3);
        let effective_merge_col = explicit_merge_col
            .filter(|&c| c > effective_split_col)
            .unwrap_or(12);

        let mut split_active = false;
        let mut ran_amp1 = false;
        let mut ran_comp2 = false;
        let mut ran_mod2 = false;
        let has_wah_slot = topo.slots.iter().any(|s| matches!(s, Some(DagBlockKind::Wah1)));
        let mut ran_wah1 = false;

        for col in 0..12usize {
            if !has_wah_slot && !ran_wah1 && col == 1 && self.wah_1_enabled {
                self.wah_1
                    .process(&self.temp_merge_l_buf[..len], &mut self.temp_dag_scratch_buf[..len]);
                self.temp_merge_l_buf[..len].copy_from_slice(&self.temp_dag_scratch_buf[..len]);
                ran_wah1 = true;
            }
            if self.split_enabled && col == effective_split_col {
                self.split.process(
                    &self.temp_merge_l_buf[..len],
                    &mut self.temp_split_a_buf[..len],
                    &mut self.temp_split_b_buf[..len],
                );
                self.temp_merge_l_buf[..len].copy_from_slice(&self.temp_split_a_buf[..len]);
                split_active = true;
            }

            if let Some(kind) = topo.slots[col] {
                if kind == DagBlockKind::Comp2 {
                    ran_comp2 = true;
                } else if kind == DagBlockKind::Mod2 {
                    ran_mod2 = true;
                } else if kind == DagBlockKind::Wah1 {
                    ran_wah1 = true;
                }
                if self.execute_dag_block(kind, false, len) {
                    ran_amp1 = true;
                }
            }

            if split_active {
                if let Some(kind1) = topo.slots[12 + col] {
                    if self.execute_dag_block(kind1, true, len) {
                        ran_amp1 = true;
                    }
                }
                if col + 1 == effective_merge_col {
                    self.temp_split_a_buf[..len].copy_from_slice(&self.temp_merge_l_buf[..len]);
                    self.merge.process(
                        &[&self.temp_split_a_buf[..len], &self.temp_split_b_buf[..len]],
                        &mut self.temp_merge_l_buf[..len],
                        &mut self.temp_merge_r_buf[..len],
                    );
                    split_active = false;
                }
            } else if let Some(kind1) = topo.slots[12 + col] {
                if matches!(kind1, DagBlockKind::Comp2 | DagBlockKind::Mod2) {
                    if kind1 == DagBlockKind::Comp2 {
                        ran_comp2 = true;
                    } else if kind1 == DagBlockKind::Mod2 {
                        ran_mod2 = true;
                    }
                    let _ = self.execute_dag_block(kind1, false, len);
                }
            }
        }

        if !self.split_enabled && !ran_comp2 && self.comp_2_enabled {
            self.comp_2
                .process(&self.temp_merge_l_buf[..len], &mut self.temp_dag_scratch_buf[..len]);
            self.temp_merge_l_buf[..len].copy_from_slice(&self.temp_dag_scratch_buf[..len]);
        }

        if !self.split_enabled && !ran_mod2 && self.mod_2_enabled {
            self.mod_2
                .process(&self.temp_merge_l_buf[..len], &mut self.temp_dag_scratch_buf[..len]);
            self.temp_merge_l_buf[..len].copy_from_slice(&self.temp_dag_scratch_buf[..len]);
        }

        if !ran_amp1 {
            for _ in 0..len {
                self.gain_smoother.next_sample();
            }
        }
    }

    pub fn handles(&self) -> RealtimeDspChainHandles {
        RealtimeDspChainHandles {
            sample_rate: self.sample_rate,
            max_block: self.max_block,
            shared_sample_rate_bits: self.shared_sample_rate_bits.clone(),
            shared_algo_latency_samples: self.shared_algo_latency_samples.clone(),
            shared_b_plus_v_bits: self.shared_b_plus_v_bits.clone(),
            adc_clips: self.adc_clips.clone(),
            clean_input_ring: self.clean_input_ring.clone(),
            scope_output_ring: self.scope_output_ring.clone(),
            param_queue: self.param_queue.clone(),
            resource_queue: self.resource_queue.clone(),
            shared_topology_id: self.shared_topology_id.clone(),
        }
    }

    #[inline(never)]
    fn build_boxed_inner(sample_rate: f32) -> Box<Self> {
        Box::new(Self::new_inner(sample_rate, false))
    }

    #[inline(never)]
    pub fn rebuild_boxed_from_handles(handles: RealtimeDspChainHandles) -> Box<Self> {
        let mut chain = Self::build_boxed_inner(handles.sample_rate);
        let mut shadow = Self::build_boxed_inner(handles.sample_rate);

        chain.shared_sample_rate_bits = handles.shared_sample_rate_bits.clone();
        chain.shared_algo_latency_samples = handles.shared_algo_latency_samples.clone();
        chain.shared_b_plus_v_bits = handles.shared_b_plus_v_bits.clone();
        chain.adc_clips = handles.adc_clips.clone();
        chain.clean_input_ring = handles.clean_input_ring.clone();
        chain.scope_output_ring = handles.scope_output_ring.clone();
        chain.param_queue = handles.param_queue.clone();
        chain.resource_queue = handles.resource_queue.clone();
        chain.shared_topology_id = handles.shared_topology_id.clone();
        chain.test_riff_buffer =
            crate::calibration::generate_karplus_strong_riff(handles.sample_rate, 12345);
        chain.tuner_scratch_read = vec![0.0; 4096];
        chain.retired_reaper = RetiredReaper::with_queue(
            handles.resource_queue.retired_queue(),
            handles.resource_queue.retired_reclaimed(),
            handles.resource_queue.retired_consumer_lock(),
        );

        shadow.shared_sample_rate_bits = handles.shared_sample_rate_bits;
        shadow.shared_algo_latency_samples = handles.shared_algo_latency_samples;
        shadow.shared_b_plus_v_bits = handles.shared_b_plus_v_bits;
        shadow.adc_clips = handles.adc_clips;
        shadow.clean_input_ring = handles.clean_input_ring;
        shadow.scope_output_ring = handles.scope_output_ring;
        shadow.param_queue = handles.param_queue;
        shadow.resource_queue = handles.resource_queue;
        shadow.shared_topology_id = handles.shared_topology_id;

        chain.shadow = Some(shadow);
        chain.shadow_out_buf = vec![0.0; handles.max_block.max(2048)];
        chain.prepare(handles.sample_rate, handles.max_block);
        chain
    }

    pub fn apply_param_updates(&mut self, updates: &[DspParamUpdate]) {
        for &u in updates {
            self.apply_param_update(u);
        }
    }
}

#[derive(Clone)]
pub struct RealtimeDspChainHandles {
    pub sample_rate: f32,
    pub max_block: usize,
    pub shared_sample_rate_bits: Arc<AtomicU32>,
    pub shared_algo_latency_samples: Arc<AtomicU32>,
    pub shared_b_plus_v_bits: Arc<AtomicU32>,
    pub adc_clips: Arc<AtomicU64>,
    pub clean_input_ring: Arc<CleanInputRing>,
    pub scope_output_ring: Arc<CleanInputRing>,
    pub param_queue: Arc<LockFreeParamQueue>,
    pub resource_queue: Arc<LockFreeResourceQueue>,
    pub shared_topology_id: Arc<AtomicU64>,
}

/// Reconstrói `RealtimeDspChain` fora da thread de áudio usando o estado atual e limpa `engine_faulted` (AUD-04).
#[inline(never)]
pub fn recover_dsp_chain_with_state(
    dsp_ref: &Arc<Mutex<RealtimeDspChain>>,
    telemetry: &TelemetryBuffer,
    st: &crate::api::AppState,
) -> Option<u64> {
    if !telemetry.is_engine_faulted() {
        return None;
    }
    let handles = {
        let guard = dsp_ref.lock().unwrap_or_else(|e| e.into_inner());
        guard.handles()
    };
    let mut fresh = RealtimeDspChain::rebuild_boxed_from_handles(handles);
    let updates = crate::api::app_state_to_dsp_updates(st);
    fresh.apply_param_updates(&updates);
    let mods = crate::api::compile_modifiers_table(&st.modifiers);
    *fresh.active_topology = st.dag_topology;
    fresh.shared_topology_id.store(st.dag_topology.id, Ordering::Release);
    *fresh.active_modifiers = mods;
    if let Some(ref mut shadow) = fresh.shadow {
        *shadow.active_topology = st.dag_topology;
        *shadow.active_modifiers = mods;
    }
    {
        let mut guard = dsp_ref.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: ambos são referências mutáveis distintas; ptr::swap evita colocar duas cadeias na pilha em debug.
        unsafe {
            std::ptr::swap(&mut *guard as *mut RealtimeDspChain, &mut *fresh as *mut RealtimeDspChain);
        }
    }
    drop(fresh);
    let count = telemetry.clear_engine_faulted_and_record_recovery();
    Some(count)
}

/// Versão ligada ao `ApiContext` para os laços de telemetria (AUD-04).
pub fn recover_dsp_chain_if_faulted(ctx: &crate::api::ApiContext) -> Option<u64> {
    if !ctx.telemetry.is_engine_faulted() {
        return None;
    }
    let dsp_ref = ctx.dsp_chain.as_ref()?;
    let st = ctx
        .app_state
        .lock()
        .ok()
        .map(|g| g.clone())
        .unwrap_or_default();
    recover_dsp_chain_with_state(dsp_ref, &ctx.telemetry, &st)
}

/// Snapshot de Telemetria para UI / VU / Osciloscópio CRT (Spec 013, CA-6 & Spec 037, CA-5/8)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TelemetrySnapshot {
    pub input_peak_db: f32,
    pub output_peak_db: f32,
    pub cpu_usage_pct: f32,
    pub gate_state: String,
    pub gate_gr_db: f32,
    pub b_plus_v: f32,
    pub waveform: Vec<f32>,
    pub waveform_seq: u64,
    pub dsp_contention_silence_blocks: u64,
    pub engine_faulted: bool,
    pub panic_count: u64,
    pub recovery_count: u64,
    pub device_state: String,
    pub connected: bool,
}

/// Buffer atômico circular para telemetria sem bloqueio nem alocação da thread de áudio (Spec 013, CA-6 & Spec 037, CA-5)
pub struct TelemetryBuffer {
    input_peak_bits: AtomicU32,
    output_peak_bits: AtomicU32,
    cpu_pct_bits: AtomicU32,
    gate_state_code: AtomicU8,
    gate_gr_db_bits: AtomicU32,
    b_plus_v_bits: AtomicU32,
    waveform_ring: Box<[AtomicU32]>,
    waveform_write_seq: AtomicU64,
    waveform_capacity: usize,
    dsp_contention_silence_blocks: AtomicU64,
    observed_input_period_frames: AtomicU32,
    observed_output_period_frames: AtomicU32,
    engine_faulted: AtomicBool,
    panic_count: AtomicU64,
    recovery_count: AtomicU64,
    stream_error_code: Arc<AtomicU8>,
    device_lost: AtomicBool,
}

impl TelemetryBuffer {
    pub fn new(waveform_capacity: usize) -> Self {
        let cap = waveform_capacity.max(1);
        let ring: Vec<AtomicU32> = (0..cap).map(|_| AtomicU32::new(0.0f32.to_bits())).collect();
        Self {
            input_peak_bits: AtomicU32::new((-120.0f32).to_bits()),
            output_peak_bits: AtomicU32::new((-120.0f32).to_bits()),
            cpu_pct_bits: AtomicU32::new((0.0f32).to_bits()),
            gate_state_code: AtomicU8::new(0),
            gate_gr_db_bits: AtomicU32::new((0.0f32).to_bits()),
            b_plus_v_bits: AtomicU32::new((280.0f32).to_bits()),
            waveform_ring: ring.into_boxed_slice(),
            waveform_write_seq: AtomicU64::new(0),
            waveform_capacity: cap,
            dsp_contention_silence_blocks: AtomicU64::new(0),
            observed_input_period_frames: AtomicU32::new(0),
            observed_output_period_frames: AtomicU32::new(0),
            engine_faulted: AtomicBool::new(false),
            panic_count: AtomicU64::new(0),
            recovery_count: AtomicU64::new(0),
            stream_error_code: Arc::new(AtomicU8::new(STREAM_ERR_OK)),
            device_lost: AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn stream_error_code_handle(&self) -> Arc<AtomicU8> {
        self.stream_error_code.clone()
    }

    #[inline]
    pub fn take_stream_error_code(&self) -> u8 {
        self.stream_error_code.swap(STREAM_ERR_OK, Ordering::AcqRel)
    }

    #[inline]
    pub fn set_device_lost(&self, lost: bool) {
        self.device_lost.store(lost, Ordering::Release);
    }

    #[inline]
    pub fn is_device_lost(&self) -> bool {
        self.device_lost.load(Ordering::Acquire)
    }

    #[inline]
    pub fn device_state_str(&self) -> &'static str {
        if self.is_device_lost() {
            "lost"
        } else {
            "ok"
        }
    }

    #[inline]
    pub fn is_engine_faulted(&self) -> bool {
        self.engine_faulted.load(Ordering::Acquire)
    }

    #[inline]
    pub fn mark_engine_faulted(&self) {
        self.panic_count.fetch_add(1, Ordering::Relaxed);
        self.engine_faulted.store(true, Ordering::Release);
    }

    #[inline]
    pub fn clear_engine_faulted_and_record_recovery(&self) -> u64 {
        let c = self.recovery_count.fetch_add(1, Ordering::Relaxed) + 1;
        self.engine_faulted.store(false, Ordering::Release);
        c
    }

    #[inline]
    pub fn panic_count(&self) -> u64 {
        self.panic_count.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn recovery_count(&self) -> u64 {
        self.recovery_count.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn reset_observed_periods(&self) {
        self.observed_input_period_frames.store(0, Ordering::Relaxed);
        self.observed_output_period_frames.store(0, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_observed_input_period(&self, frames: u32) {
        if frames > 0 && self.observed_input_period_frames.load(Ordering::Relaxed) != frames {
            self.observed_input_period_frames.store(frames, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn record_observed_output_period(&self, frames: u32) {
        if frames > 0 && self.observed_output_period_frames.load(Ordering::Relaxed) != frames {
            self.observed_output_period_frames.store(frames, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn observed_input_period_frames(&self) -> u32 {
        self.observed_input_period_frames.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn observed_output_period_frames(&self) -> u32 {
        self.observed_output_period_frames.load(Ordering::Relaxed)
    }

    pub fn update_b_plus_v(&self, b_plus_v: f32) {
        self.b_plus_v_bits.store(b_plus_v.to_bits(), Ordering::Relaxed);
    }

    pub fn update_peaks(&self, in_db: f32, out_db: f32, cpu_pct: f32) {
        self.input_peak_bits.store(in_db.to_bits(), Ordering::Relaxed);
        self.output_peak_bits.store(out_db.to_bits(), Ordering::Relaxed);
        self.cpu_pct_bits.store(cpu_pct.to_bits(), Ordering::Relaxed);
    }

    pub fn update_gate_telemetry(&self, state: GateState, gr_db: f32) {
        let code = match state {
            GateState::Closed => 0,
            GateState::Hold => 1,
            GateState::Open => 2,
        };
        self.gate_state_code.store(code, Ordering::Relaxed);
        self.gate_gr_db_bits.store(gr_db.to_bits(), Ordering::Relaxed);
    }

    pub fn update_input_peak(&self, in_db: f32) {
        self.input_peak_bits.store(in_db.to_bits(), Ordering::Relaxed);
    }

    pub fn update_output_peak(&self, out_db: f32) {
        self.output_peak_bits.store(out_db.to_bits(), Ordering::Relaxed);
    }

    pub fn input_peak_db(&self) -> f32 {
        f32::from_bits(self.input_peak_bits.load(Ordering::Relaxed))
    }

    pub fn output_peak_db(&self) -> f32 {
        f32::from_bits(self.output_peak_bits.load(Ordering::Relaxed))
    }

    pub fn cpu_usage_pct(&self) -> f32 {
        f32::from_bits(self.cpu_pct_bits.load(Ordering::Relaxed))
    }

    pub fn record_dsp_contention_silence(&self) {
        self.dsp_contention_silence_blocks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dsp_contention_silence_blocks(&self) -> u64 {
        self.dsp_contention_silence_blocks.load(Ordering::Relaxed)
    }

    pub fn waveform_seq(&self) -> u64 {
        self.waveform_write_seq.load(Ordering::Acquire)
    }

    /// Escrita 100% lock-free / wait-free no ring buffer de osciloscópio CRT (Spec 037, CA-5)
    pub fn push_waveform(&self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        let cap = self.waveform_capacity;
        let tail_samples = if samples.len() > cap {
            &samples[samples.len() - cap..]
        } else {
            samples
        };
        let mut seq = self.waveform_write_seq.load(Ordering::Relaxed);
        for &s in tail_samples {
            let idx = (seq as usize) % cap;
            self.waveform_ring[idx].store(s.to_bits(), Ordering::Relaxed);
            seq = seq.wrapping_add(1);
        }
        self.waveform_write_seq.store(seq, Ordering::Release);
    }

    pub fn snapshot(&self) -> TelemetrySnapshot {
        let input_peak_db = f32::from_bits(self.input_peak_bits.load(Ordering::Relaxed));
        let output_peak_db = f32::from_bits(self.output_peak_bits.load(Ordering::Relaxed));
        let cpu_usage_pct = f32::from_bits(self.cpu_pct_bits.load(Ordering::Relaxed));
        let gate_state = match self.gate_state_code.load(Ordering::Relaxed) {
            1 => "HOLD",
            2 => "OPEN",
            _ => "CLOSED",
        }
        .to_string();
        let gate_gr_db = f32::from_bits(self.gate_gr_db_bits.load(Ordering::Relaxed));
        let b_plus_v = f32::from_bits(self.b_plus_v_bits.load(Ordering::Relaxed));
        let cap = self.waveform_capacity;
        let seq = self.waveform_write_seq.load(Ordering::Acquire);
        let mut waveform = Vec::with_capacity(cap);
        let start = seq.saturating_sub(cap as u64);
        let valid = (seq - start) as usize;
        for _ in 0..(cap - valid) {
            waveform.push(0.0);
        }
        for offset in 0..valid {
            let idx = ((start + offset as u64) as usize) % cap;
            waveform.push(f32::from_bits(self.waveform_ring[idx].load(Ordering::Relaxed)));
        }

        TelemetrySnapshot {
            input_peak_db,
            output_peak_db,
            cpu_usage_pct,
            gate_state,
            gate_gr_db,
            b_plus_v,
            waveform,
            waveform_seq: seq,
            dsp_contention_silence_blocks: self.dsp_contention_silence_blocks.load(Ordering::Relaxed),
            engine_faulted: self.engine_faulted.load(Ordering::Acquire),
            panic_count: self.panic_count.load(Ordering::Relaxed),
            recovery_count: self.recovery_count.load(Ordering::Relaxed),
            device_state: self.device_state_str().to_string(),
            connected: !self.is_device_lost(),
        }
    }
}
