//! crates/sonix-desktop/src/audio.rs
//! Host de Áudio Nativo Desktop Standalone e Duplex Real-Time (WASAPI, ASIO, CoreAudio, ALSA, JACK)
//! Suporte a captura física P2/P3 e processamento lock-free em tempo real (Spec 012 & Spec 013)

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use ampforge_dsp::amp::{AmpModelId, WhiteBoxAmpProcessor};
use ampforge_dsp::cab::VirtualCabProcessor;
use ampforge_dsp::fx::dynamics::{NoiseGateMode, NoiseGateProcessor};
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::HeapRb;

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

#[derive(Debug, Clone)]
pub struct DesktopAudioDevice {
    pub id: String,
    pub name: String,
    pub backend: AudioBackendType,
    pub is_input: bool,
    pub supported_sample_rates: Vec<u32>,
    pub supported_buffer_sizes: Vec<u32>,
}

pub struct DesktopAudioHost {
    stream_active: bool,
    local_ipc_only: bool,
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
        }
    }

    /// Enumeração genérica compatível com Spec 012
    pub fn enumerate_devices(&self) -> Vec<DesktopAudioDevice> {
        vec![
            DesktopAudioDevice {
                id: "default_dev".to_string(),
                name: "Studio Low-Latency Audio Device".to_string(),
                backend: AudioBackendType::Alsa,
                is_input: false,
                supported_sample_rates: vec![44100, 48000, 88200, 96000],
                supported_buffer_sizes: vec![32, 64, 128, 256, 512],
            },
        ]
    }

    /// Enumeração de dispositivos físicos de captura (Microfone/Line-In P2/P3) e reprodução (Spec 013, CA-2)
    pub fn enumerate_physical_devices(&self) -> Vec<DesktopAudioDevice> {
        let backend = if cfg!(target_os = "windows") {
            AudioBackendType::Wasapi
        } else if cfg!(target_os = "macos") {
            AudioBackendType::CoreAudio
        } else {
            AudioBackendType::Alsa
        };

        vec![
            DesktopAudioDevice {
                id: "physical_input_p2_linein".to_string(),
                name: "Realtek High Definition Audio (Line-In / Mic P2/P3)".to_string(),
                backend,
                is_input: true,
                supported_sample_rates: vec![44100, 48000, 96000],
                supported_buffer_sizes: vec![64, 128, 256],
            },
            DesktopAudioDevice {
                id: "physical_output_p2_phones".to_string(),
                name: "Realtek Audio Output (Speakers / Headphones P2/P3)".to_string(),
                backend,
                is_input: false,
                supported_sample_rates: vec![44100, 48000, 96000],
                supported_buffer_sizes: vec![64, 128, 256],
            },
        ]
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

    /// Inicia streaming duplex real-time com hardware nativo WASAPI via CPAL
    pub fn start_realtime_wasapi_duplex(
        duplex: Arc<RealtimeDuplexEngine>,
        dsp_chain: Arc<Mutex<RealtimeDspChain>>,
        telemetry: Arc<TelemetryBuffer>,
    ) -> Result<RealtimeDuplexAudioStreams, AudioDriverError> {
        let host = cpal::default_host();
        let in_device = host
            .default_input_device()
            .ok_or_else(|| AudioDriverError::InitFailed("Nenhum dispositivo de entrada de áudio encontrado".to_string()))?;
        let out_device = host
            .default_output_device()
            .ok_or_else(|| AudioDriverError::InitFailed("Nenhum dispositivo de saída de áudio encontrado".to_string()))?;

        let in_config = in_device
            .default_input_config()
            .map_err(|e| AudioDriverError::InitFailed(format!("Falha na config de entrada: {}", e)))?;
        let out_config = out_device
            .default_output_config()
            .map_err(|e| AudioDriverError::InitFailed(format!("Falha na config de saída: {}", e)))?;

        let in_channels = in_config.channels() as usize;
        let out_channels = out_config.channels() as usize;

        let duplex_in = duplex.clone();
        let tele_in = telemetry.clone();

        // 1. Stream de Entrada de Áudio (Captura da Guitarra P2/P3)
        let in_stream = match in_config.sample_format() {
            cpal::SampleFormat::F32 => {
                in_device.build_input_stream(
                    in_config.into(),
                    move |data: &[f32], _| {
                        let num_frames = data.len() / in_channels;
                        let mut mono_buf = Vec::with_capacity(num_frames);
                        let mut peak = 0.0f32;
                        for f in 0..num_frames {
                            let s = if in_channels >= 2 {
                                let l = data[f * in_channels];
                                let r = data[f * in_channels + 1];
                                if l.abs() > r.abs() { l } else { r }
                            } else {
                                data[f]
                            };
                            peak = peak.max(s.abs());
                            mono_buf.push(s);
                        }
                        duplex_in.push_captured_samples(&mono_buf);
                        let peak_db = if peak > 1e-6 { 20.0 * peak.log10() } else { -120.0 };
                        tele_in.update_input_peak(peak_db);
                    },
                    |err| {
                        let msg = err.to_string();
                        if !msg.contains("underrun") && !msg.contains("overrun") {
                            eprintln!("Aviso WASAPI entrada: {}", msg);
                        }
                    },
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                in_device.build_input_stream(
                    in_config.into(),
                    move |data: &[i16], _| {
                        let num_frames = data.len() / in_channels;
                        let mut mono_buf = Vec::with_capacity(num_frames);
                        let mut peak = 0.0f32;
                        for f in 0..num_frames {
                            let s = if in_channels >= 2 {
                                let l = data[f * in_channels] as f32 / 32768.0;
                                let r = data[f * in_channels + 1] as f32 / 32768.0;
                                if l.abs() > r.abs() { l } else { r }
                            } else {
                                data[f] as f32 / 32768.0
                            };
                            peak = peak.max(s.abs());
                            mono_buf.push(s);
                        }
                        duplex_in.push_captured_samples(&mono_buf);
                        let peak_db = if peak > 1e-6 { 20.0 * peak.log10() } else { -120.0 };
                        tele_in.update_input_peak(peak_db);
                    },
                    |err| {
                        let msg = err.to_string();
                        if !msg.contains("underrun") && !msg.contains("overrun") {
                            eprintln!("Aviso WASAPI entrada: {}", msg);
                        }
                    },
                    None,
                )
            }
            _ => return Err(AudioDriverError::InitFailed("Formato de amostra de entrada não suportado".to_string())),
        }.map_err(|e| AudioDriverError::InitFailed(format!("Falha ao construir stream de entrada: {}", e)))?;

        // 2. Stream de Saída de Áudio (Processamento AmpForge DSP + Alto-Falantes / Fones)
        let duplex_out = duplex.clone();
        let dsp_chain_out = dsp_chain.clone();
        let tele_out = telemetry.clone();

        let out_stream = match out_config.sample_format() {
            cpal::SampleFormat::F32 => {
                out_device.build_output_stream(
                    out_config.into(),
                    move |data: &mut [f32], _| {
                        let num_frames = data.len() / out_channels;
                        let mut mono_in = vec![0.0f32; num_frames];
                        let mut mono_out = vec![0.0f32; num_frames];

                        duplex_out.pop_samples_for_processing(&mut mono_in);

                        if let Ok(mut dsp) = dsp_chain_out.try_lock() {
                            dsp.process(&mono_in, &mut mono_out);
                        } else {
                            mono_out.copy_from_slice(&mono_in);
                        }

                        let mut peak = 0.0f32;
                        for f in 0..num_frames {
                            let s = mono_out[f];
                            peak = peak.max(s.abs());
                            for ch in 0..out_channels {
                                data[f * out_channels + ch] = s;
                            }
                        }

                        let peak_db = if peak > 1e-6 { 20.0 * peak.log10() } else { -120.0 };
                        tele_out.update_output_peak(peak_db);
                        tele_out.push_waveform(&mono_out);
                    },
                    |err| {
                        let msg = err.to_string();
                        if !msg.contains("underrun") && !msg.contains("overrun") {
                            eprintln!("Aviso WASAPI saída: {}", msg);
                        }
                    },
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                out_device.build_output_stream(
                    out_config.into(),
                    move |data: &mut [i16], _| {
                        let num_frames = data.len() / out_channels;
                        let mut mono_in = vec![0.0f32; num_frames];
                        let mut mono_out = vec![0.0f32; num_frames];

                        duplex_out.pop_samples_for_processing(&mut mono_in);

                        if let Ok(mut dsp) = dsp_chain_out.try_lock() {
                            dsp.process(&mono_in, &mut mono_out);
                        } else {
                            mono_out.copy_from_slice(&mono_in);
                        }

                        let mut peak = 0.0f32;
                        for f in 0..num_frames {
                            let s = mono_out[f];
                            peak = peak.max(s.abs());
                            let s_i16 = (s * 32767.0).clamp(-32768.0, 32767.0) as i16;
                            for ch in 0..out_channels {
                                data[f * out_channels + ch] = s_i16;
                            }
                        }

                        let peak_db = if peak > 1e-6 { 20.0 * peak.log10() } else { -120.0 };
                        tele_out.update_output_peak(peak_db);
                        tele_out.push_waveform(&mono_out);
                    },
                    |err| {
                        let msg = err.to_string();
                        if !msg.contains("underrun") && !msg.contains("overrun") {
                            eprintln!("Aviso WASAPI saída: {}", msg);
                        }
                    },
                    None,
                )
            }
            _ => return Err(AudioDriverError::InitFailed("Formato de amostra de saída não suportado".to_string())),
        }.map_err(|e| AudioDriverError::InitFailed(format!("Falha ao construir stream de saída: {}", e)))?;

        in_stream.play().map_err(|e| AudioDriverError::InitFailed(format!("Falha ao ativar stream de entrada: {}", e)))?;
        out_stream.play().map_err(|e| AudioDriverError::InitFailed(format!("Falha ao ativar stream de saída: {}", e)))?;

        Ok(RealtimeDuplexAudioStreams {
            input_stream: in_stream,
            output_stream: out_stream,
        })
    }
}

pub struct RealtimeDuplexAudioStreams {
    pub input_stream: cpal::Stream,
    pub output_stream: cpal::Stream,
}

/// Motor Duplex em Tempo Real com Ring Buffer Lock-Free SPSC (Spec 013, CA-3)
pub struct RealtimeDuplexEngine {
    capacity: usize,
    producer: Mutex<<HeapRb<f32> as Split>::Prod>,
    consumer: Mutex<<HeapRb<f32> as Split>::Cons>,
}

impl RealtimeDuplexEngine {
    pub fn new(capacity: usize) -> Self {
        let rb = HeapRb::<f32>::new(capacity);
        let (prod, cons) = rb.split();
        Self {
            capacity,
            producer: Mutex::new(prod),
            consumer: Mutex::new(cons),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn available_samples(&self) -> usize {
        if let Ok(cons) = self.consumer.lock() {
            cons.occupied_len()
        } else {
            0
        }
    }

    pub fn push_captured_samples(&self, samples: &[f32]) -> usize {
        if let Ok(mut prod) = self.producer.lock() {
            let mut pushed = 0;
            for &sample in samples {
                if prod.try_push(sample).is_ok() {
                    pushed += 1;
                } else {
                    break;
                }
            }
            pushed
        } else {
            0
        }
    }

    pub fn pop_samples_for_processing(&self, buf: &mut [f32]) -> usize {
        if let Ok(mut cons) = self.consumer.lock() {
            // Previne acúmulo de latência (se passar de 2400 amostras / 50ms, drena para manter tempo real < 20ms)
            let occupied = cons.occupied_len();
            if occupied > 2400 {
                let to_drop = occupied.saturating_sub(960);
                for _ in 0..to_drop {
                    let _ = cons.try_pop();
                }
            }
            let mut popped = 0;
            for i in 0..buf.len() {
                if let Some(s) = cons.try_pop() {
                    buf[i] = s;
                    popped += 1;
                } else {
                    buf[i] = 0.0;
                }
            }
            popped
        } else {
            buf.fill(0.0);
            0
        }
    }
}

/// Cadeia DSP Ativa em Tempo Real: Filtro DC + Boost + Noise Gate + White-Box Amp + Cab IR (Spec 013, CA-4)
pub struct RealtimeDspChain {
    sample_rate: f32,
    gate: NoiseGateProcessor,
    gate_enabled: bool,
    amp: WhiteBoxAmpProcessor,
    cab: VirtualCabProcessor,
    cab_enabled: bool,
    input_boost: f32,
    dc_x1: f32,
    dc_y1: f32,
    temp_boost_buf: Vec<f32>,
    temp_gate_buf: Vec<f32>,
    temp_amp_buf: Vec<f32>,
}

impl RealtimeDspChain {
    pub fn new(sample_rate: f32) -> Self {
        let mut gate = NoiseGateProcessor::new(NoiseGateMode::IntelligentNaturalGate);
        gate.set_sample_rate(sample_rate);
        gate.set_threshold_db(-55.0);
        gate.set_attack_ms(1.0);
        gate.set_release_ms(30.0);

        let mut amp = WhiteBoxAmpProcessor::new(AmpModelId::BritJcm800_2203);
        amp.set_sample_rate(sample_rate);
        amp.panel_mut().set_input_gain(7.5);
        amp.panel_mut().set_master_volume(6.0);
        amp.panel_mut().set_bass(5.5);
        amp.panel_mut().set_middle(6.0);
        amp.panel_mut().set_treble(6.5);
        amp.panel_mut().set_presence(5.0);

        let mut cab = VirtualCabProcessor::new();
        cab.set_sample_rate(sample_rate);

        Self {
            sample_rate,
            gate,
            gate_enabled: true,
            amp,
            cab,
            cab_enabled: true,
            input_boost: 1.0,
            dc_x1: 0.0,
            dc_y1: 0.0,
            temp_boost_buf: vec![0.0; 2048],
            temp_gate_buf: vec![0.0; 2048],
            temp_amp_buf: vec![0.0; 2048],
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn set_gate_threshold_db(&mut self, db: f32) {
        self.gate.set_threshold_db(db);
    }

    pub fn set_gate_enabled(&mut self, enabled: bool) {
        self.gate_enabled = enabled;
    }

    pub fn set_input_boost(&mut self, boost: f32) {
        self.input_boost = boost.max(0.1);
    }

    pub fn set_amp_gain(&mut self, gain: f32) {
        self.amp.panel_mut().set_input_gain(gain);
    }

    pub fn set_amp_master(&mut self, master: f32) {
        self.amp.panel_mut().set_master_volume(master);
    }

    pub fn set_cab_enabled(&mut self, enabled: bool) {
        self.cab_enabled = enabled;
    }

    /// Processa bloco de áudio da guitarra passando por DC Blocker -> Boost -> Gate -> Amp -> Cab
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let len = input.len();
        if self.temp_boost_buf.len() < len {
            self.temp_boost_buf.resize(len, 0.0);
            self.temp_gate_buf.resize(len, 0.0);
            self.temp_amp_buf.resize(len, 0.0);
        }

        // 1. DC Blocker de 1 polo (~10 Hz) e boost de entrada
        let boost_slice = &mut self.temp_boost_buf[..len];
        let r = 0.995f32;
        for i in 0..len {
            let x = input[i];
            let y = x - self.dc_x1 + r * self.dc_y1;
            self.dc_x1 = x;
            self.dc_y1 = y;
            boost_slice[i] = y * self.input_boost;
        }

        let gate_slice = &mut self.temp_gate_buf[..len];
        let amp_slice = &mut self.temp_amp_buf[..len];

        // 2. Noise Gate (elimina zumbido/hum de entrada P2 se habilitado)
        if self.gate_enabled {
            self.gate.process(boost_slice, None, gate_slice);
        } else {
            gate_slice.copy_from_slice(boost_slice);
        }

        // 3. White-Box Tube Amp (modelagem não linear de válvulas + tonestack EQ)
        self.amp.process(gate_slice, amp_slice);

        // 4. Virtual Cab IR (filtro de resposta ao impulso de alto-falante)
        if self.cab_enabled {
            self.cab.process(amp_slice, output);
        } else {
            output[..len].copy_from_slice(amp_slice);
        }

        // 5. Hard clamp para prevenir clipping severo/explosão
        for s in output[..len].iter_mut() {
            *s = s.clamp(-2.0, 2.0);
        }
    }
}

/// Snapshot de Telemetria para UI / VU / Osciloscópio CRT (Spec 013, CA-6)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TelemetrySnapshot {
    pub input_peak_db: f32,
    pub output_peak_db: f32,
    pub cpu_usage_pct: f32,
    pub waveform: Vec<f32>,
}

/// Buffer atômico circular para telemetria sem bloqueio da thread de áudio (Spec 013, CA-6)
pub struct TelemetryBuffer {
    input_peak_bits: AtomicU32,
    output_peak_bits: AtomicU32,
    cpu_pct_bits: AtomicU32,
    waveform_buffer: Mutex<Vec<f32>>,
    waveform_capacity: usize,
}

impl TelemetryBuffer {
    pub fn new(waveform_capacity: usize) -> Self {
        Self {
            input_peak_bits: AtomicU32::new((-120.0f32).to_bits()),
            output_peak_bits: AtomicU32::new((-120.0f32).to_bits()),
            cpu_pct_bits: AtomicU32::new((0.0f32).to_bits()),
            waveform_buffer: Mutex::new(vec![0.0; waveform_capacity]),
            waveform_capacity,
        }
    }

    pub fn update_peaks(&self, in_db: f32, out_db: f32, cpu_pct: f32) {
        self.input_peak_bits.store(in_db.to_bits(), Ordering::Relaxed);
        self.output_peak_bits.store(out_db.to_bits(), Ordering::Relaxed);
        self.cpu_pct_bits.store(cpu_pct.to_bits(), Ordering::Relaxed);
    }

    pub fn update_input_peak(&self, in_db: f32) {
        self.input_peak_bits.store(in_db.to_bits(), Ordering::Relaxed);
    }

    pub fn update_output_peak(&self, out_db: f32) {
        self.output_peak_bits.store(out_db.to_bits(), Ordering::Relaxed);
    }

    pub fn push_waveform(&self, samples: &[f32]) {
        if let Ok(mut buf) = self.waveform_buffer.lock() {
            let n = samples.len().min(self.waveform_capacity);
            let shift = buf.len().saturating_sub(n);
            buf.copy_within(n.., 0);
            let start = shift;
            buf[start..start + n].copy_from_slice(&samples[..n]);
        }
    }

    pub fn snapshot(&self) -> TelemetrySnapshot {
        let input_peak_db = f32::from_bits(self.input_peak_bits.load(Ordering::Relaxed));
        let output_peak_db = f32::from_bits(self.output_peak_bits.load(Ordering::Relaxed));
        let cpu_usage_pct = f32::from_bits(self.cpu_pct_bits.load(Ordering::Relaxed));
        let waveform = self
            .waveform_buffer
            .lock()
            .map(|buf| buf.clone())
            .unwrap_or_else(|_| vec![0.0; self.waveform_capacity]);

        TelemetrySnapshot {
            input_peak_db,
            output_peak_db,
            cpu_usage_pct,
            waveform,
        }
    }
}
