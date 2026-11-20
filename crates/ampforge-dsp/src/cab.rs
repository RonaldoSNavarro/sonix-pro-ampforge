//! crates/ampforge-dsp/src/cab.rs
//! Bloco de Gabinete (Cab), Sala de Microfonação Dupla, Loader de IR e Acoplamento de Impedância (Spec 022)

use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct BiquadFilter {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl Default for BiquadFilter {
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
}

impl BiquadFilter {
    pub fn lowpass(fc: f64, q: f64, sr: f64) -> Self {
        let k = (std::f64::consts::PI * (fc / sr).clamp(0.0001, 0.499)).tan();
        let k2 = k * k;
        let inv_q = 1.0 / q.max(0.1);
        let a0 = 1.0 + inv_q * k + k2;
        let b0 = k2 / a0;
        let b1 = 2.0 * b0;
        let b2 = b0;
        let a1 = 2.0 * (k2 - 1.0) / a0;
        let a2 = (1.0 - inv_q * k + k2) / a0;

        Self {
            b0,
            b1,
            b2,
            a1,
            a2,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn high_shelf(fc: f64, gain_db: f64, sr: f64) -> Self {
        let a = 10.0f64.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f64::consts::PI * (fc / sr).clamp(0.0001, 0.499);
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / std::f64::consts::SQRT_2;
        let sqrt_a_2_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + sqrt_a_2_alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - sqrt_a_2_alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + sqrt_a_2_alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - sqrt_a_2_alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn peaking(fc: f64, gain_db: f64, q: f64, sr: f64) -> Self {
        let a = 10.0f64.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f64::consts::PI * (fc / sr).clamp(0.0001, 0.499);
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q.max(0.1));

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn low_shelf(fc: f64, gain_db: f64, sr: f64) -> Self {
        let a = 10.0f64.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f64::consts::PI * (fc / sr).clamp(0.0001, 0.499);
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / std::f64::consts::SQRT_2;
        let sqrt_a_2_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + sqrt_a_2_alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - sqrt_a_2_alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + sqrt_a_2_alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - sqrt_a_2_alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    pub fn update_coeffs(&mut self, other: Self) {
        self.b0 = other.b0;
        self.b1 = other.b1;
        self.b2 = other.b2;
        self.a1 = other.a1;
        self.a2 = other.a2;
    }

    #[inline(always)]
    pub fn process(&mut self, x: f64) -> f64 {
        let y = crate::denormal::flush_f64(
            self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2,
        );
        self.x2 = self.x1;
        self.x1 = crate::denormal::flush_f64(x);
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpeakerCabModel {
    Greenback4x12Closed,
    V304x12Closed,
    AlnicoBlue2x12Open,
    Twin2x12Open,
}

impl SpeakerCabModel {
    pub fn nominal_resonance_hz(&self) -> f32 {
        match self {
            Self::Greenback4x12Closed => 98.0,
            Self::V304x12Closed => 108.0,
            Self::AlnicoBlue2x12Open => 82.0,
            Self::Twin2x12Open => 90.0,
        }
    }

    pub fn is_open_back(&self) -> bool {
        matches!(self, Self::AlnicoBlue2x12Open | Self::Twin2x12Open)
    }

    pub fn default_impedance_curve(&self) -> ImpedanceCurve {
        match self {
            Self::Greenback4x12Closed => ImpedanceCurve {
                low_resonance_hz: 98.0,
                resonance_q: 2.2,
                low_resonance_gain_db: 7.0,
                high_inductance_rise_db: 4.5,
            },
            Self::V304x12Closed => ImpedanceCurve {
                low_resonance_hz: 108.0,
                resonance_q: 2.5,
                low_resonance_gain_db: 7.5,
                high_inductance_rise_db: 5.0,
            },
            Self::AlnicoBlue2x12Open => ImpedanceCurve {
                low_resonance_hz: 82.0,
                resonance_q: 1.8,
                low_resonance_gain_db: 5.5,
                high_inductance_rise_db: 3.5,
            },
            Self::Twin2x12Open => ImpedanceCurve {
                low_resonance_hz: 90.0,
                resonance_q: 1.7,
                low_resonance_gain_db: 5.0,
                high_inductance_rise_db: 4.5,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MicModel {
    Sm57,
    Md421,
    M88,
    R121,
    M160,
    U87,
    C414,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MicSlotConfig {
    pub model: MicModel,
    pub position_cap_to_edge: f32, // 0.0 (Cap) to 1.0 (Edge)
    pub distance_cm: f32,          // 0.0 to 30.0 cm
    pub angle_off_axis_deg: f32,   // 0.0 or 45.0
    pub level_db: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub phase_invert: bool,
    pub delay_ms: f32,
    pub enabled: bool,
}

impl Default for MicSlotConfig {
    fn default() -> Self {
        Self {
            model: MicModel::Sm57,
            position_cap_to_edge: 0.2,
            distance_cm: 2.0,
            angle_off_axis_deg: 0.0,
            level_db: 0.0,
            pan: 0.0,
            mute: false,
            solo: false,
            phase_invert: false,
            delay_ms: 0.0,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImpedanceCurve {
    pub low_resonance_hz: f32,
    pub resonance_q: f32,
    pub low_resonance_gain_db: f32,
    pub high_inductance_rise_db: f32,
}

impl Default for ImpedanceCurve {
    fn default() -> Self {
        SpeakerCabModel::Greenback4x12Closed.default_impedance_curve()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CabAcousticConditioning {
    pub low_cut_hz: f32,
    pub low_cut_slope_db_oct: u32,
    pub high_cut_hz: f32,
    pub speaker_thump: f32,
    pub room_mic_early_reflections_pct: f32,
}

impl Default for CabAcousticConditioning {
    fn default() -> Self {
        Self {
            low_cut_hz: 60.0,
            low_cut_slope_db_oct: 12,
            high_cut_hz: 4200.0,
            speaker_thump: 0.0,
            room_mic_early_reflections_pct: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidImpulseResponseError {
    EmptyFile,
    CorruptedOrTruncated,
    UnsupportedFormat(String),
}

impl std::fmt::Display for InvalidImpulseResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "Arquivo de IR vazio (0 bytes)"),
            Self::CorruptedOrTruncated => write!(f, "Arquivo RIFF/WAVE corrompido ou truncado"),
            Self::UnsupportedFormat(err) => write!(f, "Formato de IR não suportado: {}", err),
        }
    }
}

impl std::error::Error for InvalidImpulseResponseError {}

const PART_B: usize = 128;
const FFT_N: usize = 256;
const MAX_PARTITIONS: usize = 512;

#[derive(Debug, Clone, Copy, Default)]
struct Complex32 {
    re: f32,
    im: f32,
}

impl Complex32 {
    #[inline(always)]
    fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }
}

/// FFT Radix-2 Cooley-Tukey in-place de 256 pontos (zero-alloc).
fn fft_256(buf: &mut [Complex32; FFT_N], inverse: bool) {
    // Bit-reversal permutation (8 bits para 256)
    for i in 0..FFT_N {
        let rev = (i as u8).reverse_bits() as usize;
        if i < rev {
            buf.swap(i, rev);
        }
    }

    let sign = if inverse { 1.0f32 } else { -1.0f32 };
    let mut half = 1usize;
    while half < FFT_N {
        let step = half << 1;
        let angle_step = sign * std::f32::consts::PI / (half as f32);
        let w_step_re = angle_step.cos();
        let w_step_im = angle_step.sin();

        let mut start = 0usize;
        while start < FFT_N {
            let mut w_re = 1.0f32;
            let mut w_im = 0.0f32;
            for k in 0..half {
                let even = start + k;
                let odd = even + half;
                let b = buf[odd];
                let t_re = w_re * b.re - w_im * b.im;
                let t_im = w_re * b.im + w_im * b.re;
                let a = buf[even];
                buf[even] = Complex32::new(a.re + t_re, a.im + t_im);
                buf[odd] = Complex32::new(a.re - t_re, a.im - t_im);

                let next_re = w_re * w_step_re - w_im * w_step_im;
                let next_im = w_re * w_step_im + w_im * w_step_re;
                w_re = next_re;
                w_im = next_im;
            }
            start += step;
        }
        half = step;
    }

    if inverse {
        let inv_n = 1.0f32 / (FFT_N as f32);
        for c in buf.iter_mut() {
            c.re *= inv_n;
            c.im *= inv_n;
        }
    }
}

#[derive(Debug)]
struct IrPartitionedData {
    head_taps: [f32; PART_B],
    head_len: usize,
    tail_partitions: Vec<[Complex32; FFT_N]>,
}

impl IrPartitionedData {
    fn from_samples(samples: &[f32]) -> Self {
        let capped_len = samples.len().min(MAX_PARTITIONS * PART_B);
        let mut head_taps = [0.0f32; PART_B];
        let head_len = capped_len.min(PART_B);
        head_taps[..head_len].copy_from_slice(&samples[..head_len]);

        let num_parts = (capped_len + PART_B - 1) / PART_B;
        let tail_count = num_parts.saturating_sub(1);
        let mut tail_partitions = Vec::with_capacity(tail_count);
        for p in 1..num_parts {
            let start = p * PART_B;
            let end = (start + PART_B).min(capped_len);
            let mut buf = [Complex32::default(); FFT_N];
            for (i, &s) in samples[start..end].iter().enumerate() {
                buf[i] = Complex32::new(s, 0.0);
            }
            fft_256(&mut buf, false);
            tail_partitions.push(buf);
        }

        Self {
            head_taps,
            head_len,
            tail_partitions,
        }
    }
}

struct PartitionedConvolverState {
    head_circ: [f32; PART_B],
    head_pos: usize,
    in_block: [f32; PART_B],
    block_pos: usize,
    tail_out: [f32; PART_B],
    saved_overlap: [f32; PART_B],
    fdl: Vec<[Complex32; FFT_N]>,
    fdl_pos: usize,
    fdl_valid_count: usize,
    fft_buf: [Complex32; FFT_N],
    acc_buf: [Complex32; FFT_N],
}

impl PartitionedConvolverState {
    fn new() -> Self {
        Self {
            head_circ: [0.0; PART_B],
            head_pos: 0,
            in_block: [0.0; PART_B],
            block_pos: 0,
            tail_out: [0.0; PART_B],
            saved_overlap: [0.0; PART_B],
            fdl: vec![[Complex32::default(); FFT_N]; MAX_PARTITIONS],
            fdl_pos: 0,
            fdl_valid_count: 0,
            fft_buf: [Complex32::default(); FFT_N],
            acc_buf: [Complex32::default(); FFT_N],
        }
    }

    fn reset(&mut self) {
        self.head_circ.fill(0.0);
        self.head_pos = 0;
        self.in_block.fill(0.0);
        self.block_pos = 0;
        self.tail_out.fill(0.0);
        self.saved_overlap.fill(0.0);
        self.fdl_pos = 0;
        self.fdl_valid_count = 0;
    }

    #[inline]
    fn process_sample(&mut self, x: f32, data: &IrPartitionedData) -> f32 {
        // 1. Partição 0: FIR direto (latência zero)
        self.head_circ[self.head_pos] = x;
        let mut y_head = 0.0f32;
        let h_len = data.head_len;
        let h_taps = &data.head_taps;
        for k in 0..h_len {
            let idx = (self.head_pos + PART_B - k) & (PART_B - 1);
            y_head += self.head_circ[idx] * h_taps[k];
        }
        self.head_pos = (self.head_pos + 1) & (PART_B - 1);

        // 2. Cauda pré-computada por FFT
        let y_tail = self.tail_out[self.block_pos];
        self.in_block[self.block_pos] = x;
        self.block_pos += 1;

        // 3. Ao completar bloco de 128 amostras, computa FFT Overlap-Add das partições p >= 1
        if self.block_pos == PART_B {
            self.block_pos = 0;
            if !data.tail_partitions.is_empty() {
                for i in 0..PART_B {
                    self.fft_buf[i] = Complex32::new(self.in_block[i], 0.0);
                    self.fft_buf[PART_B + i] = Complex32::default();
                }
                fft_256(&mut self.fft_buf, false);
                self.fdl[self.fdl_pos] = self.fft_buf;

                self.acc_buf.fill(Complex32::default());
                let valid = self.fdl_valid_count + 1;
                let max_p = data.tail_partitions.len().min(valid);
                for p in 0..max_p {
                    let fdl_idx = (self.fdl_pos + MAX_PARTITIONS - p) % MAX_PARTITIONS;
                    let x_spec = &self.fdl[fdl_idx];
                    let h_spec = &data.tail_partitions[p];
                    for k in 0..FFT_N {
                        let a = x_spec[k];
                        let b = h_spec[k];
                        self.acc_buf[k].re += a.re * b.re - a.im * b.im;
                        self.acc_buf[k].im += a.re * b.im + a.im * b.re;
                    }
                }
                fft_256(&mut self.acc_buf, true);

                for i in 0..PART_B {
                    self.tail_out[i] = self.saved_overlap[i] + self.acc_buf[i].re;
                    self.saved_overlap[i] = self.acc_buf[PART_B + i].re;
                }

                self.fdl_pos = (self.fdl_pos + 1) % MAX_PARTITIONS;
                if self.fdl_valid_count < MAX_PARTITIONS {
                    self.fdl_valid_count += 1;
                }
            }
        }

        y_head + y_tail
    }
}

#[derive(Debug, Clone)]
pub struct ImpulseResponse {
    sample_rate: f32,
    samples: std::sync::Arc<[f32]>,
    partitioned: std::sync::Arc<IrPartitionedData>,
}

impl ImpulseResponse {
    pub fn from_samples(sample_rate: f32, samples: Vec<f32>) -> Self {
        let partitioned = std::sync::Arc::new(IrPartitionedData::from_samples(&samples));
        Self {
            sample_rate,
            samples: std::sync::Arc::from(samples),
            partitioned,
        }
    }

    pub fn from_samples_with_normalization(sample_rate: f32, mut samples: Vec<f32>, normalize: bool) -> Self {
        if normalize && !samples.is_empty() {
            let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
            let energy = sum_sq.sqrt().max(1e-9);
            for s in samples.iter_mut() {
                *s /= energy;
            }
        }
        let partitioned = std::sync::Arc::new(IrPartitionedData::from_samples(&samples));
        Self {
            sample_rate,
            samples: std::sync::Arc::from(samples),
            partitioned,
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }
}

pub struct WavIrLoader {
    session_sample_rate: f32,
}

impl WavIrLoader {
    pub fn new(session_sample_rate: f32) -> Self {
        Self {
            session_sample_rate: session_sample_rate.max(1.0),
        }
    }

    pub fn validate_ir_duration(num_samples: usize, sample_rate: f32) -> Result<(), InvalidImpulseResponseError> {
        let max_samples = (sample_rate * 0.50001).round() as usize;
        if num_samples > max_samples {
            return Err(InvalidImpulseResponseError::UnsupportedFormat(format!(
                "ir_too_long: IR tem {} amostras ({:.1} ms), excedendo limite de 500 ms",
                num_samples,
                (num_samples as f32 / sample_rate) * 1000.0
            )));
        }
        Ok(())
    }

    pub fn resample_sinc(&self, input: &[f32], in_sr: f32, out_sr: f32) -> ImpulseResponse {
        if (in_sr - out_sr).abs() < 1.0 {
            return ImpulseResponse::from_samples(out_sr, input.to_vec());
        }
        let ratio = in_sr / out_sr;
        let out_len = (input.len() as f32 / ratio).round() as usize;
        let mut out = Vec::with_capacity(out_len);
        let filter_half_width = 24isize; // 48 taps windowed sinc
        let cutoff = if out_sr < in_sr { out_sr / in_sr } else { 1.0 };
        for i in 0..out_len {
            let center = i as f32 * ratio;
            let center_idx = center.round() as isize;
            let mut sum = 0.0f32;
            for tap in (center_idx - filter_half_width)..=(center_idx + filter_half_width) {
                if tap >= 0 && (tap as usize) < input.len() {
                    let d = center - tap as f32;
                    let sinc = if d.abs() < 1e-6 {
                        cutoff
                    } else {
                        let pi_d = PI * d * cutoff;
                        cutoff * (pi_d.sin() / pi_d)
                    };
                    let w = (d / filter_half_width as f32).clamp(-1.0, 1.0);
                    let window = 0.42 + 0.50 * (PI * w).cos() + 0.08 * (2.0 * PI * w).cos();
                    sum += input[tap as usize] * (sinc * window);
                }
            }
            out.push(sum);
        }
        ImpulseResponse::from_samples(out_sr, out)
    }

    pub fn load_from_bytes(&self, bytes: &[u8]) -> Result<ImpulseResponse, InvalidImpulseResponseError> {
        self.load_from_bytes_internal(bytes, true).map(|(ir, _)| ir)
    }

    pub fn load_from_bytes_with_info(
        &self,
        bytes: &[u8],
    ) -> Result<(ImpulseResponse, u16), InvalidImpulseResponseError> {
        self.load_from_bytes_internal(bytes, true)
    }

    pub fn load_wav_unlimited(&self, bytes: &[u8]) -> Result<ImpulseResponse, InvalidImpulseResponseError> {
        self.load_from_bytes_internal(bytes, false).map(|(ir, _)| ir)
    }

    fn load_from_bytes_internal(
        &self,
        bytes: &[u8],
        enforce_max_duration: bool,
    ) -> Result<(ImpulseResponse, u16), InvalidImpulseResponseError> {
        if bytes.is_empty() {
            return Err(InvalidImpulseResponseError::EmptyFile);
        }
        if bytes.len() < 44 {
            return Err(InvalidImpulseResponseError::CorruptedOrTruncated);
        }

        if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return Err(InvalidImpulseResponseError::CorruptedOrTruncated);
        }

        let mut pos = 12;
        let mut sample_rate = 44100u32;
        let mut channels = 1u16;
        let mut bits_per_sample = 16u16;
        let mut audio_format = 1u16;
        let mut fmt_parsed = false;
        let mut data_start: Option<usize> = None;
        let mut data_len = 0usize;

        while pos + 8 <= bytes.len() {
            let chunk_id = &bytes[pos..pos + 4];
            let Ok(chunk_size_bytes) = bytes[pos + 4..pos + 8].try_into() else { break; };
            let chunk_size = u32::from_le_bytes(chunk_size_bytes) as usize;
            pos += 8;

            if chunk_id == b"fmt " && chunk_size >= 16 && pos + 16 <= bytes.len() {
                audio_format = u16::from_le_bytes(bytes[pos..pos + 2].try_into().unwrap());
                channels = u16::from_le_bytes(bytes[pos + 2..pos + 4].try_into().unwrap());
                sample_rate = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap());
                bits_per_sample = u16::from_le_bytes(bytes[pos + 14..pos + 16].try_into().unwrap());
                fmt_parsed = true;
            } else if chunk_id == b"data" {
                data_start = Some(pos);
                data_len = chunk_size.min(bytes.len().saturating_sub(pos));
                break;
            }
            let Some(next_pos) = pos.checked_add(chunk_size) else {
                break;
            };
            pos = next_pos;
        }

        if !fmt_parsed || channels == 0 || sample_rate == 0 {
            return Err(InvalidImpulseResponseError::CorruptedOrTruncated);
        }

        if !(8_000..=384_000).contains(&sample_rate) {
            return Err(InvalidImpulseResponseError::UnsupportedFormat(format!(
                "Sample rate {} fora da faixa permitida (8000 a 384000 Hz)",
                sample_rate
            )));
        }

        let start = data_start.ok_or(InvalidImpulseResponseError::CorruptedOrTruncated)?;
        let end = (start + data_len).min(bytes.len());
        let raw_data = &bytes[start..end];

        let mut samples = Vec::new();
        match (audio_format, bits_per_sample) {
            (1, 16) => {
                let frame_size = 2 * channels as usize;
                if frame_size == 0 { return Err(InvalidImpulseResponseError::CorruptedOrTruncated); }
                for chunk in raw_data.chunks_exact(frame_size) {
                    let s_i16 = i16::from_le_bytes([chunk[0], chunk[1]]);
                    samples.push(s_i16 as f32 / 32768.0);
                }
            }
            (1, 24) => {
                let frame_size = 3 * channels as usize;
                if frame_size == 0 { return Err(InvalidImpulseResponseError::CorruptedOrTruncated); }
                for chunk in raw_data.chunks_exact(frame_size) {
                    let s_i32 = i32::from_le_bytes([0, chunk[0], chunk[1], chunk[2]]) >> 8;
                    samples.push(s_i32 as f32 / 8388608.0);
                }
            }
            (3, 32) => {
                let frame_size = 4 * channels as usize;
                if frame_size == 0 { return Err(InvalidImpulseResponseError::CorruptedOrTruncated); }
                for chunk in raw_data.chunks_exact(frame_size) {
                    let s_f32 = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    samples.push(s_f32);
                }
            }
            _ => {
                return Err(InvalidImpulseResponseError::UnsupportedFormat(format!(
                    "audio_format: {}, bits: {}",
                    audio_format, bits_per_sample
                )));
            }
        }

        if samples.is_empty() {
            return Err(InvalidImpulseResponseError::CorruptedOrTruncated);
        }

        // Validação de duração máxima de 500 ms para IR de gabinete (Spec 022, CA-11)
        if enforce_max_duration {
            Self::validate_ir_duration(samples.len(), sample_rate as f32)?;
        }

        let ir = if (sample_rate as f32 - self.session_sample_rate).abs() > 1.0 {
            self.resample_sinc(&samples, sample_rate as f32, self.session_sample_rate)
        } else {
            ImpulseResponse::from_samples(self.session_sample_rate, samples)
        };

        Ok((ir, channels))
    }
}

pub struct VirtualCabProcessor {
    speaker_model: SpeakerCabModel,
    mics: [MicSlotConfig; 2],
    impedance_curve: ImpedanceCurve,
    conditioning: CabAcousticConditioning,
    coupled_amp_id: Option<String>,
    sample_rate: f32,
    custom_ir: Option<ImpulseResponse>,
    target_ir: Option<ImpulseResponse>,
    retired_ir: Option<ImpulseResponse>,
    ir_enabled: bool,
    ir_crossfade: f32,
    ir_crossfade_step: f32,
    // Delay buffer para mic 1 e mic 2
    delay_line_mic1: Vec<f32>,
    delay_pos_mic1: usize,
    delay_line_mic2: Vec<f32>,
    delay_pos_mic2: usize,
    // Convolução particionada zero-latência (cabeça FIR direta + partições FFT Overlap-Add)
    conv_custom: PartitionedConvolverState,
    conv_target: PartitionedConvolverState,
    // Filtros de modelagem de microfone e gabinete
    hicut_filter1: BiquadFilter,
    hicut_filter2: BiquadFilter,
    open_back_hp: BiquadFilter,
    speaker_voicing_filter: BiquadFilter,
    mic1_pos_filter: BiquadFilter,
    mic2_pos_filter: BiquadFilter,
    mic1_prox_filter: BiquadFilter,
    mic2_prox_filter: BiquadFilter,
    mic1_angle_filter: BiquadFilter,
    mic2_angle_filter: BiquadFilter,
    mic1_model_f1: BiquadFilter,
    mic1_model_f2: BiquadFilter,
    mic2_model_f1: BiquadFilter,
    mic2_model_f2: BiquadFilter,
}

impl Default for VirtualCabProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualCabProcessor {
    pub fn new() -> Self {
        let slot1 = MicSlotConfig::default();
        let mut slot2 = MicSlotConfig::default();
        slot2.enabled = false;

        let spk = SpeakerCabModel::Greenback4x12Closed;

        let mut s = Self {
            speaker_model: spk,
            mics: [slot1, slot2],
            impedance_curve: spk.default_impedance_curve(),
            conditioning: CabAcousticConditioning::default(),
            coupled_amp_id: None,
            sample_rate: 48000.0,
            custom_ir: None,
            target_ir: None,
            retired_ir: None,
            ir_enabled: false,
            ir_crossfade: 1.0,
            ir_crossfade_step: 1.0 / (48000.0 * 0.020),
            delay_line_mic1: vec![0.0; 16384],
            delay_pos_mic1: 0,
            delay_line_mic2: vec![0.0; 16384],
            delay_pos_mic2: 0,
            conv_custom: PartitionedConvolverState::new(),
            conv_target: PartitionedConvolverState::new(),
            hicut_filter1: BiquadFilter::default(),
            hicut_filter2: BiquadFilter::default(),
            open_back_hp: BiquadFilter::default(),
            speaker_voicing_filter: BiquadFilter::default(),
            mic1_pos_filter: BiquadFilter::default(),
            mic2_pos_filter: BiquadFilter::default(),
            mic1_prox_filter: BiquadFilter::default(),
            mic2_prox_filter: BiquadFilter::default(),
            mic1_angle_filter: BiquadFilter::default(),
            mic2_angle_filter: BiquadFilter::default(),
            mic1_model_f1: BiquadFilter::default(),
            mic1_model_f2: BiquadFilter::default(),
            mic2_model_f1: BiquadFilter::default(),
            mic2_model_f2: BiquadFilter::default(),
        };
        s.update_filters();
        s
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.ir_crossfade_step = 1.0 / (self.sample_rate * 0.020).max(1.0);
        self.update_filters();
    }

    pub fn set_speaker_model(&mut self, model: SpeakerCabModel) {
        self.speaker_model = model;
        self.impedance_curve = model.default_impedance_curve();
        self.update_filters();
    }

    pub fn speaker_model(&self) -> SpeakerCabModel {
        self.speaker_model
    }

    pub fn set_resonance_hz(&mut self, hz: f32) {
        self.impedance_curve.low_resonance_hz = hz.clamp(65.0, 135.0);
    }

    pub fn resonance_hz(&self) -> f32 {
        self.impedance_curve.low_resonance_hz
    }

    pub fn set_hicut_khz(&mut self, khz: f32) {
        self.conditioning.high_cut_hz = (khz * 1000.0).clamp(3500.0, 18000.0);
        self.update_filters();
    }

    pub fn hicut_khz(&self) -> f32 {
        self.conditioning.high_cut_hz / 1000.0
    }

    pub fn set_ir_enabled(&mut self, enabled: bool) {
        self.ir_enabled = enabled;
    }

    pub fn ir_enabled(&self) -> bool {
        self.ir_enabled
    }

    pub fn set_custom_ir(&mut self, ir: ImpulseResponse) {
        if let Some(old) = self.custom_ir.take() {
            self.retired_ir = Some(old);
        }
        self.custom_ir = Some(ir);
        self.conv_custom.reset();
        self.ir_crossfade = 1.0;
        if let Some(old_t) = self.target_ir.take() {
            if self.retired_ir.is_none() {
                self.retired_ir = Some(old_t);
            }
        }
    }

    pub fn set_target_ir(&mut self, ir: ImpulseResponse) {
        if self.custom_ir.is_none() {
            self.set_custom_ir(ir);
        } else {
            if let Some(old_t) = self.target_ir.replace(ir) {
                self.retired_ir = Some(old_t);
            }
            self.conv_target.reset();
            self.ir_crossfade = 0.0;
        }
    }

    pub fn take_retired_ir(&mut self) -> Option<ImpulseResponse> {
        self.retired_ir.take()
    }

    pub fn has_retired_ir(&self) -> bool {
        self.retired_ir.is_some()
    }

    pub fn reset_ir_state(&mut self) {
        self.conv_custom.reset();
        self.conv_target.reset();
    }

    pub fn calculate_impedance_peak(&self, f_min: f32, f_max: f32, f_step: f32) -> (f32, f32) {
        let f_res = self.impedance_curve.low_resonance_hz;
        let q = self.impedance_curve.resonance_q;
        let mut best_f = f_res;
        let mut max_z = 0.0f32;
        let mut f = f_min;
        while f <= f_max {
            let ratio = f / f_res;
            let denom = 1.0 + q * q * (ratio - 1.0 / ratio).powi(2);
            let z = 8.0 + 32.0 / denom + (f / 3000.0);
            if z > max_z {
                max_z = z;
                best_f = f;
            }
            f += f_step;
        }
        (best_f, max_z)
    }

    pub fn mic_slot(&self, idx: usize) -> &MicSlotConfig {
        &self.mics[idx]
    }

    pub fn mic_slot_mut(&mut self, idx: usize) -> &mut MicSlotConfig {
        &mut self.mics[idx]
    }

    pub fn impedance_curve(&self) -> &ImpedanceCurve {
        &self.impedance_curve
    }

    pub fn set_impedance_curve(&mut self, curve: ImpedanceCurve) {
        self.impedance_curve = curve;
    }

    pub fn impedance_badge_text(&self) -> String {
        format!("{:.1} Hz // Q: {:.1}", self.impedance_curve.low_resonance_hz, self.impedance_curve.resonance_q)
    }

    pub fn coupled_amp_id(&self) -> Option<&str> {
        self.coupled_amp_id.as_deref()
    }

    pub fn set_coupled_amp_id(&mut self, id: impl Into<String>) {
        self.coupled_amp_id = Some(id.into());
    }

    pub fn conditioning(&self) -> &CabAcousticConditioning {
        &self.conditioning
    }

    pub fn set_conditioning(&mut self, cond: CabAcousticConditioning) {
        self.conditioning = cond;
        self.update_filters();
    }

    pub fn load_custom_ir(&mut self, bytes: &[u8]) -> Result<(), InvalidImpulseResponseError> {
        let loader = WavIrLoader::new(self.sample_rate);
        let ir = loader.load_from_bytes(bytes)?;
        self.set_custom_ir(ir);
        self.ir_enabled = true;
        Ok(())
    }

    fn update_filters(&mut self) {
        let sr = self.sample_rate as f64;
        let fc = self.conditioning.high_cut_hz.clamp(1000.0, self.sample_rate * 0.45) as f64;

        // Butterworth 4ª ordem passa-baixas (2 estágios em cascata) para corte exato de -3 dB na fc (-24 dB/oct)
        let b1 = BiquadFilter::lowpass(fc, 0.5411961, sr);
        let b2 = BiquadFilter::lowpass(fc, 1.3065630, sr);
        self.hicut_filter1.update_coeffs(b1);
        self.hicut_filter2.update_coeffs(b2);

        // Open back dipole cancelamento em graves (< 120 Hz)
        if self.speaker_model.is_open_back() {
            self.open_back_hp.update_coeffs(BiquadFilter::low_shelf(120.0, -6.0, sr));
        } else {
            self.open_back_hp.update_coeffs(BiquadFilter::default());
        }

        // Voicing acústico por falante (Jensen Twin 2x12 Open brilhante; Greenback neutro para manter precisão de hicut)
        match self.speaker_model {
            SpeakerCabModel::Twin2x12Open => {
                self.speaker_voicing_filter.update_coeffs(BiquadFilter::high_shelf(2200.0, 3.8, sr));
            }
            SpeakerCabModel::AlnicoBlue2x12Open => {
                self.speaker_voicing_filter.update_coeffs(BiquadFilter::high_shelf(2800.0, 2.0, sr));
            }
            SpeakerCabModel::V304x12Closed => {
                self.speaker_voicing_filter.update_coeffs(BiquadFilter::peaking(2200.0, 1.5, 1.8, sr));
            }
            SpeakerCabModel::Greenback4x12Closed => {
                self.speaker_voicing_filter.update_coeffs(BiquadFilter::default());
            }
        }
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        output.fill(0.0);

        if self.ir_enabled && (self.custom_ir.is_some() || self.target_ir.is_some()) {
            self.process_ir_convolution(input, output);
            return;
        }

        self.update_filters();
        let sr = self.sample_rate as f64;

        // Processa cada slot de microfone
        for slot_idx in 0..2 {
            let mic = &self.mics[slot_idx];
            if !mic.enabled || mic.mute {
                continue;
            }

            let gain = if mic.level_db <= -60.0 {
                0.0
            } else {
                10.0f32.powf(mic.level_db / 20.0)
            };
            let sign = if mic.phase_invert { -1.0 } else { 1.0 };
            let total_gain = (gain * sign) as f64;

            // Filtro de posição cap-to-edge (0.0 = centro/brilhante, 1.0 = borda/-8.5 dB em altas frequências)
            let pos = mic.position_cap_to_edge.clamp(0.0, 1.0);
            let pos_cut_db = -8.5 * pos as f64;
            let pos_filter = if slot_idx == 0 {
                self.mic1_pos_filter.update_coeffs(BiquadFilter::high_shelf(3500.0, pos_cut_db, sr));
                &mut self.mic1_pos_filter
            } else {
                self.mic2_pos_filter.update_coeffs(BiquadFilter::high_shelf(3500.0, pos_cut_db, sr));
                &mut self.mic2_pos_filter
            };

            // Efeito de proximidade para microfones direcionais (mais grave em distâncias curtas)
            let dist_cm = mic.distance_cm.clamp(0.0, 30.0) as f64;
            let prox_boost_db = 9.0 * (1.0 - (dist_cm / 20.0).clamp(0.0, 1.0));
            let prox_filter = if slot_idx == 0 {
                self.mic1_prox_filter.update_coeffs(BiquadFilter::low_shelf(150.0, prox_boost_db, sr));
                &mut self.mic1_prox_filter
            } else {
                self.mic2_prox_filter.update_coeffs(BiquadFilter::low_shelf(150.0, prox_boost_db, sr));
                &mut self.mic2_prox_filter
            };

            // Filtro de eixo off-axis (0° plano, 45° atenua brilho extremo > 3.5 kHz)
            let angle_norm = (mic.angle_off_axis_deg / 45.0).clamp(0.0, 1.0) as f64;
            let angle_cut_db = -6.0 * angle_norm;
            let angle_filter = if slot_idx == 0 {
                self.mic1_angle_filter
                    .update_coeffs(BiquadFilter::high_shelf(3200.0, angle_cut_db, sr));
                &mut self.mic1_angle_filter
            } else {
                self.mic2_angle_filter
                    .update_coeffs(BiquadFilter::high_shelf(3200.0, angle_cut_db, sr));
                &mut self.mic2_angle_filter
            };

            let (model_f1, model_f2) = if slot_idx == 0 {
                (&mut self.mic1_model_f1, &mut self.mic1_model_f2)
            } else {
                (&mut self.mic2_model_f1, &mut self.mic2_model_f2)
            };
            match mic.model {
                MicModel::Sm57 => {
                    model_f1.update_coeffs(BiquadFilter::peaking(5600.0, 4.0, 2.5, sr));
                    model_f2.update_coeffs(BiquadFilter::low_shelf(160.0, -3.5, sr));
                }
                MicModel::Md421 => {
                    model_f1.update_coeffs(BiquadFilter::peaking(280.0, -2.5, 1.2, sr));
                    model_f2.update_coeffs(BiquadFilter::peaking(4500.0, 3.5, 1.3, sr));
                }
                MicModel::M88 => {
                    model_f1.update_coeffs(BiquadFilter::low_shelf(110.0, 2.5, sr));
                    model_f2.update_coeffs(BiquadFilter::peaking(4000.0, 3.0, 1.5, sr));
                }
                MicModel::R121 => {
                    model_f1.update_coeffs(BiquadFilter::low_shelf(130.0, 2.0, sr));
                    model_f2.update_coeffs(BiquadFilter::high_shelf(3800.0, -7.5, sr));
                }
                MicModel::M160 => {
                    model_f1.update_coeffs(BiquadFilter::peaking(1800.0, 1.5, 1.0, sr));
                    model_f2.update_coeffs(BiquadFilter::high_shelf(5500.0, -5.0, sr));
                }
                MicModel::U87 => {
                    model_f1.update_coeffs(BiquadFilter::high_shelf(8000.0, 2.5, sr));
                    model_f2.update_coeffs(BiquadFilter::peaking(250.0, 0.5, 0.9, sr));
                }
                MicModel::C414 => {
                    model_f1.update_coeffs(BiquadFilter::peaking(3500.0, 3.0, 1.2, sr));
                    model_f2.update_coeffs(BiquadFilter::high_shelf(9500.0, 4.0, sr));
                }
            }

            // Atraso de propagação de microfone: delta t = d_cm * 0.01 / 343.0 s
            let delay_sec = (dist_cm * 0.01) / 343.0;
            let delay_samples = delay_sec * sr;

            let (delay_line, delay_pos) = if slot_idx == 0 {
                (&mut self.delay_line_mic1, &mut self.delay_pos_mic1)
            } else {
                (&mut self.delay_line_mic2, &mut self.delay_pos_mic2)
            };

            let line_len = delay_line.len();
            let max_delay = line_len.saturating_sub(2);
            let d_clamped = delay_samples.max(0.0).min(max_delay as f64);
            let d_int = (d_clamped.floor() as usize).min(max_delay);
            let d_frac = (d_clamped - d_int as f64) as f32;

            for i in 0..input.len() {
                let x = input[i];
                delay_line[*delay_pos] = x;

                let r_idx0 = (*delay_pos + line_len - d_int) % line_len;
                let r_idx1 = (r_idx0 + line_len - 1) % line_len;
                let delayed = delay_line[r_idx0] * (1.0 - d_frac) + delay_line[r_idx1] * d_frac;
                *delay_pos = (*delay_pos + 1) % line_len;

                // Passa pelos filtros acústicos do microfone e resposta do modelo
                let mut s = delayed as f64;
                s = pos_filter.process(s);
                s = prox_filter.process(s);
                s = angle_filter.process(s);
                s = model_f1.process(s);
                s = model_f2.process(s);

                output[i] += (s * total_gain) as f32;
            }
        }

        // Condicionamento acústico do gabinete: voicing do falante, corte de agudos e open-back
        for s in output.iter_mut() {
            let mut v = *s as f64;
            v = self.open_back_hp.process(v);
            v = self.speaker_voicing_filter.process(v);
            v = self.hicut_filter1.process(v);
            v = self.hicut_filter2.process(v);
            *s = v as f32;
        }
    }

    fn process_ir_convolution(&mut self, input: &[f32], output: &mut [f32]) {
        for i in 0..input.len() {
            let x = input[i];

            let y1 = if let Some(ref ir) = self.custom_ir {
                self.conv_custom.process_sample(x, &ir.partitioned)
            } else {
                0.0
            };

            let y2 = if let Some(ref ir) = self.target_ir {
                self.conv_target.process_sample(x, &ir.partitioned)
            } else {
                0.0
            };

            let y = if self.target_ir.is_some() {
                let a = self.ir_crossfade;
                (1.0 - a) * y1 + a * y2
            } else {
                y1
            };

            output[i] = y;

            if self.target_ir.is_some() {
                self.ir_crossfade += self.ir_crossfade_step;
                if self.ir_crossfade >= 1.0 {
                    let old_ir = self.custom_ir.take();
                    self.custom_ir = self.target_ir.take();
                    std::mem::swap(&mut self.conv_custom, &mut self.conv_target);
                    self.conv_target.reset();
                    self.ir_crossfade = 1.0;
                    // Move a IR antiga para retired_ir sem desalocar na thread de áudio (Spec 037, CA-3)
                    if let Some(old) = old_ir {
                        if self.retired_ir.is_none() {
                            self.retired_ir = Some(old);
                        } else {
                            std::mem::forget(self.retired_ir.replace(old));
                        }
                    }
                }
            }
        }
    }
}
