//! crates/ampforge-dsp/src/cab.rs
//! Bloco de Gabinete (Cab), Sala de Microfonação Dupla, Loader de IR e Acoplamento de Impedância (Spec 022)

use std::f32::consts::PI;

pub use crate::biquad::BiquadFilter;

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
const HALF_N: usize = 128;
const FFT_N: usize = 256;
const SPEC_BINS: usize = 129;
const MAX_PARTITIONS: usize = 512;
const SPEAKER_FIR_LEN: usize = 384;
const SPEAKER_FFT_N: usize = 1024;

/// Pontos de resposta em frequência amostrados em 1/3 de oitava dos datasheets dos fabricantes:
/// - Greenback4x12Closed: Celestion G12M Greenback (https://celestion.com/wp-content/uploads/2021/10/T1221_G12M-Greenback_Data_Sheet.pdf)
/// - V304x12Closed: Celestion Vintage 30 (https://celestion.com/wp-content/uploads/2021/10/T3904_Vintage-30_Data_Sheet.pdf)
/// - AlnicoBlue2x12Open: Celestion Alnico Blue (https://celestion.com/wp-content/uploads/2021/10/T1225_Celestion-Blue_Data_Sheet.pdf)
/// - Twin2x12Open: Jensen C12N (https://www.jensentone.com/sites/default/files/c12n_1.pdf)
pub fn speaker_curve_points_hz_db(model: SpeakerCabModel) -> &'static [(f32, f32)] {
    match model {
        SpeakerCabModel::Greenback4x12Closed => &[
            (63.0, -0.5), (80.0, 0.5), (100.0, 0.5), (125.0, 0.0), (160.0, 0.0),
            (200.0, 0.0), (250.0, 0.0), (315.0, 0.0), (400.0, -0.5), (500.0, -0.5),
            (630.0, 0.0), (800.0, 0.5), (1000.0, 0.0), (1250.0, 0.5), (1600.0, 1.5),
            (2000.0, 2.5), (2500.0, 3.0), (3150.0, 2.5), (4000.0, 1.5), (5000.0, -3.0),
            (6300.0, -9.0), (8000.0, -14.0), (10000.0, -18.0),
        ],
        SpeakerCabModel::V304x12Closed => &[
            (63.0, -9.0), (80.0, -4.5), (100.0, -1.0), (125.0, 0.5), (160.0, 0.0),
            (200.0, -0.5), (250.0, -0.5), (315.0, -0.5), (400.0, -1.0), (500.0, -0.5),
            (630.0, 0.0), (800.0, 0.5), (1000.0, 0.0), (1250.0, 1.0), (1600.0, 2.5),
            (2000.0, 3.5), (2500.0, 4.5), (3150.0, 4.5), (4000.0, 4.0), (5000.0, 1.5),
            (6300.0, -5.5), (8000.0, -11.0), (10000.0, -15.5),
        ],
        SpeakerCabModel::AlnicoBlue2x12Open => &[
            (63.0, -11.0), (80.0, -7.0), (100.0, -3.5), (125.0, -1.5), (160.0, -0.5),
            (200.0, 0.0), (250.0, 0.0), (315.0, 0.0), (400.0, -0.5), (500.0, -0.5),
            (630.0, 0.0), (800.0, 0.5), (1000.0, 0.0), (1250.0, 1.0), (1600.0, 2.0),
            (2000.0, 3.0), (2500.0, 3.5), (3150.0, 4.0), (4000.0, 3.5), (5000.0, 1.0),
            (6300.0, -4.5), (8000.0, -9.5), (10000.0, -14.0),
        ],
        SpeakerCabModel::Twin2x12Open => &[
            (63.0, -10.5), (80.0, -6.5), (100.0, -3.0), (125.0, -1.0), (160.0, 0.0),
            (200.0, 0.5), (250.0, 0.0), (315.0, -0.5), (400.0, -1.0), (500.0, -1.0),
            (630.0, -0.5), (800.0, 0.0), (1000.0, 0.0), (1250.0, 0.5), (1600.0, 1.5),
            (2000.0, 2.5), (2500.0, 3.5), (3150.0, 4.0), (4000.0, 4.5), (5000.0, 3.5),
            (6300.0, -1.0), (8000.0, -6.0), (10000.0, -11.0),
        ],
    }
}

fn interpolate_curve_db(points: &[(f32, f32)], freq_hz: f64) -> f64 {
    if freq_hz <= points[0].0 as f64 {
        return points[0].1 as f64;
    }
    let last = points[points.len() - 1];
    if freq_hz >= last.0 as f64 {
        // Continua rolloff suave além de 10 kHz
        let octs = (freq_hz / last.0 as f64).log2();
        return (last.1 as f64 - octs * 12.0).max(-40.0);
    }
    for w in points.windows(2) {
        let (f0, db0) = (w[0].0 as f64, w[0].1 as f64);
        let (f1, db1) = (w[1].0 as f64, w[1].1 as f64);
        if freq_hz >= f0 && freq_hz <= f1 {
            let t = (freq_hz.log2() - f0.log2()) / (f1.log2() - f0.log2()).max(1e-9);
            return db0 + t * (db1 - db0);
        }
    }
    0.0
}

fn fft_512(re: &mut [f64; SPEAKER_FFT_N], im: &mut [f64; SPEAKER_FFT_N], inverse: bool) {
    let mut j = 0usize;
    for i in 0..SPEAKER_FFT_N {
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
        let mut bit = SPEAKER_FFT_N >> 1;
        while bit > 0 && (j & bit) != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
    }

    let sign = if inverse { 1.0f64 } else { -1.0f64 };
    let mut len = 2usize;
    while len <= SPEAKER_FFT_N {
        let half = len >> 1;
        let angle = sign * 2.0 * std::f64::consts::PI / (len as f64);
        let wlen_re = angle.cos();
        let wlen_im = angle.sin();
        let mut start = 0usize;
        while start < SPEAKER_FFT_N {
            let mut w_re = 1.0f64;
            let mut w_im = 0.0f64;
            for k in 0..half {
                let u_re = re[start + k];
                let u_im = im[start + k];
                let v_re = re[start + k + half] * w_re - im[start + k + half] * w_im;
                let v_im = re[start + k + half] * w_im + im[start + k + half] * w_re;
                re[start + k] = u_re + v_re;
                im[start + k] = u_im + v_im;
                re[start + k + half] = u_re - v_re;
                im[start + k + half] = u_im - v_im;
                let next_re = w_re * wlen_re - w_im * wlen_im;
                w_im = w_re * wlen_im + w_im * wlen_re;
                w_re = next_re;
            }
            start += len;
        }
        len <<= 1;
    }

    if inverse {
        let inv_n = 1.0 / (SPEAKER_FFT_N as f64);
        for k in 0..SPEAKER_FFT_N {
            re[k] *= inv_n;
            im[k] *= inv_n;
        }
    }
}

fn build_min_phase_speaker_fir(model: SpeakerCabModel, sr: f32) -> [f32; SPEAKER_FIR_LEN] {
    let points = speaker_curve_points_hz_db(model);
    let mut log_re = [0.0f64; SPEAKER_FFT_N];
    let mut log_im = [0.0f64; SPEAKER_FFT_N];
    let fs = (sr as f64).max(1.0);
    for k in 0..=SPEAKER_FFT_N / 2 {
        let freq = (k as f64) * fs / (SPEAKER_FFT_N as f64);
        let db = if k == 0 {
            points[0].1 as f64
        } else {
            interpolate_curve_db(points, freq)
        };
        let mag = 10.0f64.powf(db / 20.0).max(1e-4);
        log_re[k] = mag.ln();
        if k > 0 && k < SPEAKER_FFT_N / 2 {
            log_re[SPEAKER_FFT_N - k] = log_re[k];
        }
    }

    // IFFT -> cepstro real
    fft_512(&mut log_re, &mut log_im, true);
    for n in 1..SPEAKER_FFT_N / 2 {
        log_re[n] *= 2.0;
        log_im[n] *= 2.0;
    }
    for n in (SPEAKER_FFT_N / 2 + 1)..SPEAKER_FFT_N {
        log_re[n] = 0.0;
        log_im[n] = 0.0;
    }

    // FFT -> espectro complexo log mínimo, depois exp() -> H_min
    fft_512(&mut log_re, &mut log_im, false);
    for k in 0..SPEAKER_FFT_N {
        let r = log_re[k].exp();
        let theta = log_im[k];
        log_re[k] = r * theta.cos();
        log_im[k] = r * theta.sin();
    }

    // IFFT -> FIR de fase mínima
    fft_512(&mut log_re, &mut log_im, true);
    let mut fir = [0.0f32; SPEAKER_FIR_LEN];
    for n in 0..SPEAKER_FIR_LEN {
        // Janela suave apenas nos últimos 16 taps para zerar borda
        let win = if n >= SPEAKER_FIR_LEN - 16 {
            let t = (n - (SPEAKER_FIR_LEN - 16)) as f64 / 16.0;
            0.5 * (1.0 + (std::f64::consts::PI * t).cos())
        } else {
            1.0
        };
        fir[n] = (log_re[n] * win) as f32;
    }
    fir
}

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

fn rfft_256_twiddles() -> &'static [Complex32; (HALF_N / 2) + 1] {
    static TWIDDLES: std::sync::OnceLock<[Complex32; (HALF_N / 2) + 1]> = std::sync::OnceLock::new();
    TWIDDLES.get_or_init(|| {
        let mut table = [Complex32::default(); (HALF_N / 2) + 1];
        for (k, slot) in table.iter_mut().enumerate() {
            let angle = -std::f32::consts::PI * (k as f32) / (HALF_N as f32);
            let (im, re) = angle.sin_cos();
            *slot = Complex32::new(re, im);
        }
        table
    })
}

fn fft_128_stage_twiddles() -> &'static [(f32, f32); 7] {
    static STAGES: std::sync::OnceLock<[(f32, f32); 7]> = std::sync::OnceLock::new();
    STAGES.get_or_init(|| {
        let mut arr = [(0.0f32, 0.0f32); 7];
        for (stage, slot) in arr.iter_mut().enumerate() {
            let half = 1usize << stage;
            let angle_step = -std::f32::consts::PI / (half as f32);
            *slot = (angle_step.cos(), angle_step.sin());
        }
        arr
    })
}

/// FFT Radix-2 Cooley-Tukey in-place de 128 pontos (zero-alloc).
fn fft_128(buf: &mut [Complex32; HALF_N], inverse: bool) {
    for i in 0..HALF_N {
        let rev = ((i as u8).reverse_bits() >> 1) as usize;
        if i < rev {
            buf.swap(i, rev);
        }
    }

    let stages = fft_128_stage_twiddles();
    let sign = if inverse { -1.0f32 } else { 1.0f32 };
    let mut half = 1usize;
    let mut stage = 0usize;
    while half < HALF_N {
        let step = half << 1;
        let (w_step_re, raw_im) = stages[stage];
        let w_step_im = raw_im * sign;
        stage += 1;

        let mut start = 0usize;
        while start < HALF_N {
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
        let inv_n = 1.0f32 / (HALF_N as f32);
        for c in buf.iter_mut() {
            c.re *= inv_n;
            c.im *= inv_n;
        }
    }
}

/// Real FFT de 256 amostras via FFT complexa de 128 pontos (retorna 129 bins).
fn rfft_256_forward(
    input: &[f32; FFT_N],
    out: &mut [Complex32; SPEC_BINS],
    scratch: &mut [Complex32; HALF_N],
) {
    for k in 0..HALF_N {
        scratch[k] = Complex32::new(input[2 * k], input[2 * k + 1]);
    }
    fft_128(scratch, false);

    out[0] = Complex32::new(scratch[0].re + scratch[0].im, 0.0);
    out[HALF_N] = Complex32::new(scratch[0].re - scratch[0].im, 0.0);

    let twiddles = rfft_256_twiddles();
    for k in 1..=(HALF_N / 2) {
        let c_k = scratch[k];
        let c_nk = scratch[HALF_N - k];
        let even_re = 0.5 * (c_k.re + c_nk.re);
        let even_im = 0.5 * (c_k.im - c_nk.im);
        let odd_re = 0.5 * (c_k.im + c_nk.im);
        let odd_im = -0.5 * (c_k.re - c_nk.re);

        let w = twiddles[k];
        let rot_re = w.re * odd_re - w.im * odd_im;
        let rot_im = w.re * odd_im + w.im * odd_re;

        out[k] = Complex32::new(even_re + rot_re, even_im + rot_im);
        out[HALF_N - k] = Complex32::new(even_re - rot_re, -(even_im - rot_im));
    }
}

/// Inverse Real FFT de 129 bins para 256 amostras reais via IFFT complexa de 128 pontos.
fn rfft_256_inverse(
    spec: &[Complex32; SPEC_BINS],
    out: &mut [f32; FFT_N],
    scratch: &mut [Complex32; HALF_N],
) {
    scratch[0] = Complex32::new(
        0.5 * (spec[0].re + spec[HALF_N].re),
        0.5 * (spec[0].re - spec[HALF_N].re),
    );

    let twiddles = rfft_256_twiddles();
    for k in 1..=(HALF_N / 2) {
        let y_k = spec[k];
        let y_nk = Complex32::new(spec[HALF_N - k].re, -spec[HALF_N - k].im);
        let even_re = 0.5 * (y_k.re + y_nk.re);
        let even_im = 0.5 * (y_k.im + y_nk.im);
        let diff_re = 0.5 * (y_k.re - y_nk.re);
        let diff_im = 0.5 * (y_k.im - y_nk.im);

        let w = twiddles[k];
        let w_re = w.re;
        let w_im = -w.im;
        let w_diff_re = w_re * diff_re - w_im * diff_im;
        let w_diff_im = w_re * diff_im + w_im * diff_re;
        let odd_re = -w_diff_im;
        let odd_im = w_diff_re;

        scratch[k] = Complex32::new(even_re + odd_re, even_im + odd_im);
        scratch[HALF_N - k] = Complex32::new(even_re - odd_re, -(even_im - odd_im));
    }

    fft_128(scratch, true);
    for k in 0..HALF_N {
        out[2 * k] = scratch[k].re;
        out[2 * k + 1] = scratch[k].im;
    }
}

#[derive(Debug)]
struct IrPartitionedData {
    head_taps: [f32; PART_B],
    head_len: usize,
    tail_partitions: Vec<[Complex32; SPEC_BINS]>,
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
        let mut scratch = [Complex32::default(); HALF_N];
        for p in 1..num_parts {
            let start = p * PART_B;
            let end = (start + PART_B).min(capped_len);
            let mut time_buf = [0.0f32; FFT_N];
            time_buf[..end - start].copy_from_slice(&samples[start..end]);
            let mut spec_buf = [Complex32::default(); SPEC_BINS];
            rfft_256_forward(&time_buf, &mut spec_buf, &mut scratch);
            tail_partitions.push(spec_buf);
        }

        Self {
            head_taps,
            head_len,
            tail_partitions,
        }
    }
}

struct PartitionedConvolverState {
    head_circ: [f32; PART_B * 2],
    head_pos: usize,
    in_block: [f32; PART_B],
    block_pos: usize,
    tail_out: [f32; PART_B],
    saved_overlap: [f32; PART_B],
    fdl: Vec<[Complex32; SPEC_BINS]>,
    fdl_pos: usize,
    fdl_valid_count: usize,
    acc_buf: [Complex32; SPEC_BINS],
    fft_scratch: [Complex32; HALF_N],
    time_scratch: [f32; FFT_N],
    mac_cursor: usize,
    last_call_macs: u32,
}

impl PartitionedConvolverState {
    fn new() -> Self {
        let _ = rfft_256_twiddles();
        let _ = fft_128_stage_twiddles();
        Self {
            head_circ: [0.0; PART_B * 2],
            head_pos: 0,
            in_block: [0.0; PART_B],
            block_pos: 0,
            tail_out: [0.0; PART_B],
            saved_overlap: [0.0; PART_B],
            fdl: vec![[Complex32::default(); SPEC_BINS]; MAX_PARTITIONS],
            fdl_pos: 0,
            fdl_valid_count: 0,
            acc_buf: [Complex32::default(); SPEC_BINS],
            fft_scratch: [Complex32::default(); HALF_N],
            time_scratch: [0.0; FFT_N],
            mac_cursor: 1,
            last_call_macs: 0,
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
        self.acc_buf.fill(Complex32::default());
        self.mac_cursor = 1;
        self.last_call_macs = 0;
    }

    #[inline]
    fn accumulate_partition(&mut self, p: usize, data: &IrPartitionedData) {
        let fdl_idx = (self.fdl_pos + MAX_PARTITIONS - p) % MAX_PARTITIONS;
        let x_spec = &self.fdl[fdl_idx];
        let h_spec = &data.tail_partitions[p];
        for ((acc, &a), &b) in self.acc_buf[..SPEC_BINS]
            .iter_mut()
            .zip(x_spec[..SPEC_BINS].iter())
            .zip(h_spec[..SPEC_BINS].iter())
        {
            acc.re += a.re * b.re - a.im * b.im;
            acc.im += a.re * b.im + a.im * b.re;
        }
        self.last_call_macs = self.last_call_macs.saturating_add(1);
    }

    #[inline]
    fn process_sample(&mut self, x: f32, data: &IrPartitionedData) -> f32 {
        // 1. Partição 0: FIR direto (latência zero, janela contígua sem desvio por tap)
        let pos = if self.head_pos == 0 {
            PART_B - 1
        } else {
            self.head_pos - 1
        };
        self.head_pos = pos;
        self.head_circ[pos] = x;
        self.head_circ[pos + PART_B] = x;
        let h_len = data.head_len;
        let hist = &self.head_circ[pos..pos + h_len];
        let h_taps = &data.head_taps[..h_len];
        let mut y_head = 0.0f32;
        for (&h, &f) in hist.iter().zip(h_taps.iter()) {
            y_head += h * f;
        }

        // 2. Cauda pré-computada por FFT distribuída no período de 128 amostras
        let y_tail = self.tail_out[self.block_pos];
        let sample_idx = self.block_pos;
        self.in_block[sample_idx] = x;
        self.block_pos += 1;

        if !data.tail_partitions.is_empty() {
            let max_p = data.tail_partitions.len().min(self.fdl_valid_count + 1);
            if sample_idx == 0 {
                self.acc_buf.fill(Complex32::default());
                self.mac_cursor = 1;
            }

            if sample_idx < PART_B - 1 {
                let tail_rest = max_p.saturating_sub(1);
                let target_end = 1 + ((sample_idx + 1) * tail_rest) / (PART_B - 1);
                let end_p = target_end.min(max_p);
                for p in self.mac_cursor..end_p {
                    self.accumulate_partition(p, data);
                }
                self.mac_cursor = end_p;
            } else {
                // Amostra 127: Real FFT do bloco atual + partição p=0 + Inverse Real FFT
                self.block_pos = 0;
                self.time_scratch[..PART_B].copy_from_slice(&self.in_block);
                self.time_scratch[PART_B..].fill(0.0);
                rfft_256_forward(
                    &self.time_scratch,
                    &mut self.fdl[self.fdl_pos],
                    &mut self.fft_scratch,
                );

                for p in self.mac_cursor..max_p {
                    self.accumulate_partition(p, data);
                }
                self.accumulate_partition(0, data);

                rfft_256_inverse(&self.acc_buf, &mut self.time_scratch, &mut self.fft_scratch);
                for i in 0..PART_B {
                    self.tail_out[i] = self.saved_overlap[i] + self.time_scratch[i];
                    self.saved_overlap[i] = self.time_scratch[PART_B + i];
                }

                self.fdl_pos = (self.fdl_pos + 1) % MAX_PARTITIONS;
                if self.fdl_valid_count < MAX_PARTITIONS {
                    self.fdl_valid_count += 1;
                }
                self.acc_buf.fill(Complex32::default());
                self.mac_cursor = 1;
            }
        } else if self.block_pos == PART_B {
            self.block_pos = 0;
        }

        y_head + y_tail
    }
}

#[derive(Debug)]
struct IrDropTracker(std::sync::Arc<std::sync::atomic::AtomicUsize>);

impl Drop for IrDropTracker {
    fn drop(&mut self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    }
}

#[derive(Debug, Clone)]
pub struct ImpulseResponse {
    sample_rate: f32,
    samples: std::sync::Arc<[f32]>,
    partitioned: std::sync::Arc<IrPartitionedData>,
    drop_tracker: Option<std::sync::Arc<IrDropTracker>>,
}

impl ImpulseResponse {
    pub fn from_samples(sample_rate: f32, samples: Vec<f32>) -> Self {
        let partitioned = std::sync::Arc::new(IrPartitionedData::from_samples(&samples));
        Self {
            sample_rate,
            samples: std::sync::Arc::from(samples),
            partitioned,
            drop_tracker: None,
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
            drop_tracker: None,
        }
    }

    pub fn with_drop_counter(mut self, counter: std::sync::Arc<std::sync::atomic::AtomicUsize>) -> Self {
        self.drop_tracker = Some(std::sync::Arc::new(IrDropTracker(counter)));
        self
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
    retired_irs: [Option<ImpulseResponse>; 8],
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
    hicut_smoother: crate::smoother::ParamSmoother,
    resonance_smoother: crate::smoother::ParamSmoother,
    has_processed: bool,
    speaker_fir: [f32; SPEAKER_FIR_LEN],
    speaker_hist: [f32; SPEAKER_FIR_LEN * 2],
    speaker_hist_pos: usize,
    fir_model: SpeakerCabModel,
    fir_sr: f32,
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
        let impedance_curve = spk.default_impedance_curve();
        let conditioning = CabAcousticConditioning::default();
        let init_hicut_hz = conditioning.high_cut_hz as f64;
        let init_res_hz = impedance_curve.low_resonance_hz as f64;

        let mut s = Self {
            speaker_model: spk,
            mics: [slot1, slot2],
            impedance_curve,
            conditioning,
            coupled_amp_id: None,
            sample_rate: 48000.0,
            custom_ir: None,
            target_ir: None,
            retired_irs: [const { None }; 8],
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
            hicut_smoother: crate::smoother::ParamSmoother::new(48000.0, 0.020, init_hicut_hz),
            resonance_smoother: crate::smoother::ParamSmoother::new(48000.0, 0.020, init_res_hz),
            has_processed: false,
            speaker_fir: build_min_phase_speaker_fir(spk, 48000.0),
            speaker_hist: [0.0; SPEAKER_FIR_LEN * 2],
            speaker_hist_pos: 0,
            fir_model: spk,
            fir_sr: 48000.0,
        };
        s.update_filters();
        s
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.ir_crossfade_step = 1.0 / (self.sample_rate * 0.020).max(1.0);
        self.hicut_smoother.set_sample_rate(self.sample_rate as f64);
        self.resonance_smoother.set_sample_rate(self.sample_rate as f64);
        self.update_filters();
    }

    pub fn set_speaker_model(&mut self, model: SpeakerCabModel) {
        self.speaker_model = model;
        self.impedance_curve = model.default_impedance_curve();
        let res_hz = self.impedance_curve.low_resonance_hz as f64;
        if !self.has_processed {
            self.resonance_smoother.reset(res_hz);
        } else {
            self.resonance_smoother.set_target(res_hz);
        }
        self.update_filters();
    }

    pub fn speaker_model(&self) -> SpeakerCabModel {
        self.speaker_model
    }

    pub fn speaker_fir_taps(&self) -> &[f32] {
        &self.speaker_fir
    }

    pub fn set_resonance_hz(&mut self, hz: f32) {
        let clamped = hz.clamp(65.0, 135.0);
        self.impedance_curve.low_resonance_hz = clamped;
        if !self.has_processed {
            self.resonance_smoother.reset(clamped as f64);
        } else {
            self.resonance_smoother.set_target(clamped as f64);
        }
    }

    pub fn resonance_hz(&self) -> f32 {
        self.impedance_curve.low_resonance_hz
    }

    pub fn set_hicut_khz(&mut self, khz: f32) {
        let hz = (khz * 1000.0).clamp(3500.0, 18000.0);
        self.conditioning.high_cut_hz = hz;
        if !self.has_processed {
            self.hicut_smoother.reset(hz as f64);
            self.update_filters();
        } else {
            self.hicut_smoother.set_target(hz as f64);
        }
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

    fn push_retired_ir(&mut self, ir: ImpulseResponse) {
        for slot in &mut self.retired_irs {
            if slot.is_none() {
                *slot = Some(ir);
                return;
            }
        }
        self.retired_irs[0] = Some(ir);
    }

    pub fn set_custom_ir(&mut self, ir: ImpulseResponse) {
        if let Some(old) = self.custom_ir.take() {
            self.push_retired_ir(old);
        }
        self.custom_ir = Some(ir);
        self.conv_custom.reset();
        self.ir_crossfade = 1.0;
        if let Some(old_t) = self.target_ir.take() {
            self.push_retired_ir(old_t);
        }
    }

    pub fn set_target_ir(&mut self, ir: ImpulseResponse) {
        if self.custom_ir.is_none() {
            self.set_custom_ir(ir);
        } else {
            if let Some(old_t) = self.target_ir.replace(ir) {
                self.push_retired_ir(old_t);
            }
            self.conv_target.reset();
            self.ir_crossfade = 0.0;
        }
    }

    pub fn take_retired_ir(&mut self) -> Option<ImpulseResponse> {
        for slot in &mut self.retired_irs {
            if slot.is_some() {
                return slot.take();
            }
        }
        None
    }

    pub fn has_retired_ir(&self) -> bool {
        self.retired_irs.iter().any(Option::is_some)
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

    fn update_hicut_coeffs(&mut self, hicut_hz: f64) {
        let sr = self.sample_rate as f64;
        let fc = hicut_hz.clamp(1000.0, (sr * 0.45).max(1001.0));
        // Butterworth 4ª ordem passa-baixas (2 estágios em cascata) para corte exato de -3 dB na fc (-24 dB/oct)
        let b1 = BiquadFilter::lowpass_bilinear(fc, 0.5411961, sr);
        let b2 = BiquadFilter::lowpass_bilinear(fc, 1.3065630, sr);
        self.hicut_filter1.update_coeffs(b1);
        self.hicut_filter2.update_coeffs(b2);
    }

    fn update_filters(&mut self) {
        let sr = self.sample_rate as f64;
        let fc = self.hicut_smoother.current();
        self.update_hicut_coeffs(fc);

        // Open back dipole cancelamento em graves (< 120 Hz)
        if self.speaker_model.is_open_back() {
            self.open_back_hp.update_coeffs(BiquadFilter::low_shelf_sqrt2(120.0, -6.0, sr));
        } else {
            self.open_back_hp.update_coeffs(BiquadFilter::default());
        }

        // Curva de resposta do alto-falante por FIR de fase mínima (calculada em prepare / troca de modelo/sr)
        if self.fir_model != self.speaker_model || (self.fir_sr - self.sample_rate).abs() > 0.5 {
            self.speaker_fir = build_min_phase_speaker_fir(self.speaker_model, self.sample_rate);
            self.fir_model = self.speaker_model;
            self.fir_sr = self.sample_rate;
        }
        self.speaker_voicing_filter.update_coeffs(BiquadFilter::default());
    }

    #[inline(always)]
    fn step_speaker_fir(&mut self, x: f32) -> f32 {
        let pos = if self.speaker_hist_pos == 0 {
            SPEAKER_FIR_LEN - 1
        } else {
            self.speaker_hist_pos - 1
        };
        self.speaker_hist_pos = pos;
        self.speaker_hist[pos] = x;
        self.speaker_hist[pos + SPEAKER_FIR_LEN] = x;

        let hist = &self.speaker_hist[pos..pos + SPEAKER_FIR_LEN];
        let fir = &self.speaker_fir[..SPEAKER_FIR_LEN];
        let mut acc = 0.0f32;
        for (&h, &f) in hist.iter().zip(fir.iter()) {
            acc += h * f;
        }
        crate::denormal::flush_f32(acc)
    }

    fn apply_hicut_output(&mut self, output: &mut [f32]) {
        if !self.hicut_smoother.is_ramping() {
            let fc = self.hicut_smoother.current();
            if fc >= 17990.0 {
                return;
            }
            self.update_hicut_coeffs(fc);
            for s in output.iter_mut() {
                let mut v = *s as f64;
                v = self.hicut_filter1.process_flushed(v);
                v = self.hicut_filter2.process_flushed(v);
                *s = v as f32;
            }
        } else {
            let mut offset = 0usize;
            while offset < output.len() {
                let chunk = (output.len() - offset).min(4);
                let fc = self.hicut_smoother.advance(chunk);
                self.update_hicut_coeffs(fc);
                for s in &mut output[offset..offset + chunk] {
                    let mut v = *s as f64;
                    v = self.hicut_filter1.process_flushed(v);
                    v = self.hicut_filter2.process_flushed(v);
                    *s = v as f32;
                }
                offset += chunk;
            }
        }
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        self.has_processed = true;
        output.fill(0.0);

        if self.ir_enabled && (self.custom_ir.is_some() || self.target_ir.is_some()) {
            self.resonance_smoother.advance(input.len());
            self.process_ir_convolution(input, output);
            self.apply_hicut_output(output);
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
                self.mic1_pos_filter.update_coeffs(BiquadFilter::high_shelf_sqrt2(3500.0, pos_cut_db, sr));
                &mut self.mic1_pos_filter
            } else {
                self.mic2_pos_filter.update_coeffs(BiquadFilter::high_shelf_sqrt2(3500.0, pos_cut_db, sr));
                &mut self.mic2_pos_filter
            };

            // Efeito de proximidade para microfones direcionais (mais grave em distâncias curtas)
            let dist_cm = mic.distance_cm.clamp(0.0, 30.0) as f64;
            let prox_boost_db = 9.0 * (1.0 - (dist_cm / 20.0).clamp(0.0, 1.0));
            let prox_filter = if slot_idx == 0 {
                self.mic1_prox_filter.update_coeffs(BiquadFilter::low_shelf_sqrt2(150.0, prox_boost_db, sr));
                &mut self.mic1_prox_filter
            } else {
                self.mic2_prox_filter.update_coeffs(BiquadFilter::low_shelf_sqrt2(150.0, prox_boost_db, sr));
                &mut self.mic2_prox_filter
            };

            // Filtro de eixo off-axis (0° plano, 45° atenua brilho extremo > 3.5 kHz)
            let angle_norm = (mic.angle_off_axis_deg / 45.0).clamp(0.0, 1.0) as f64;
            let angle_cut_db = -6.0 * angle_norm;
            let angle_filter = if slot_idx == 0 {
                self.mic1_angle_filter
                    .update_coeffs(BiquadFilter::high_shelf_sqrt2(3200.0, angle_cut_db, sr));
                &mut self.mic1_angle_filter
            } else {
                self.mic2_angle_filter
                    .update_coeffs(BiquadFilter::high_shelf_sqrt2(3200.0, angle_cut_db, sr));
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
                    model_f2.update_coeffs(BiquadFilter::low_shelf_sqrt2(160.0, -3.5, sr));
                }
                MicModel::Md421 => {
                    model_f1.update_coeffs(BiquadFilter::peaking(280.0, -2.5, 1.2, sr));
                    model_f2.update_coeffs(BiquadFilter::peaking(4500.0, 3.5, 1.3, sr));
                }
                MicModel::M88 => {
                    model_f1.update_coeffs(BiquadFilter::low_shelf_sqrt2(110.0, 2.5, sr));
                    model_f2.update_coeffs(BiquadFilter::peaking(4000.0, 3.0, 1.5, sr));
                }
                MicModel::R121 => {
                    model_f1.update_coeffs(BiquadFilter::low_shelf_sqrt2(130.0, 2.0, sr));
                    model_f2.update_coeffs(BiquadFilter::high_shelf_sqrt2(3800.0, -7.5, sr));
                }
                MicModel::M160 => {
                    model_f1.update_coeffs(BiquadFilter::peaking(1800.0, 1.5, 1.0, sr));
                    model_f2.update_coeffs(BiquadFilter::high_shelf_sqrt2(5500.0, -5.0, sr));
                }
                MicModel::U87 => {
                    model_f1.update_coeffs(BiquadFilter::high_shelf_sqrt2(8000.0, 2.5, sr));
                    model_f2.update_coeffs(BiquadFilter::peaking(250.0, 0.5, 0.9, sr));
                }
                MicModel::C414 => {
                    model_f1.update_coeffs(BiquadFilter::peaking(3500.0, 3.0, 1.2, sr));
                    model_f2.update_coeffs(BiquadFilter::high_shelf_sqrt2(9500.0, 4.0, sr));
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
                s = pos_filter.process_flushed(s);
                s = prox_filter.process_flushed(s);
                s = angle_filter.process_flushed(s);
                s = model_f1.process(s);
                s = model_f2.process(s);

                output[i] += (s * total_gain) as f32;
            }
        }

        // Condicionamento acústico do gabinete: FIR de fase mínima do falante, open-back e corte de agudos
        if !self.hicut_smoother.is_ramping() && !self.resonance_smoother.is_ramping() {
            for s in output.iter_mut() {
                let fir_out = self.step_speaker_fir(*s);
                let mut v = fir_out as f64;
                v = self.open_back_hp.process(v);
                v = self.hicut_filter1.process_flushed(v);
                v = self.hicut_filter2.process_flushed(v);
                *s = v as f32;
            }
        } else {
            let mut offset = 0usize;
            while offset < output.len() {
                let chunk = (output.len() - offset).min(4);
                let fc = self.hicut_smoother.advance(chunk);
                self.resonance_smoother.advance(chunk);
                self.update_hicut_coeffs(fc);
                for s in &mut output[offset..offset + chunk] {
                    let fir_out = self.step_speaker_fir(*s);
                    let mut v = fir_out as f64;
                    v = self.open_back_hp.process(v);
                    v = self.hicut_filter1.process_flushed(v);
                    v = self.hicut_filter2.process_flushed(v);
                    *s = v as f32;
                }
                offset += chunk;
            }
        }
    }

    pub fn ultima_chamada_macs(&self) -> u32 {
        self.conv_custom.last_call_macs
    }

    fn process_ir_convolution(&mut self, input: &[f32], output: &mut [f32]) {
        self.conv_custom.last_call_macs = 0;
        self.conv_target.last_call_macs = 0;
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
                    // Move a IR antiga para retired_irs sem desalocar na thread de áudio (Spec 037 CA-3 / AUD-07)
                    if let Some(old) = old_ir {
                        self.push_retired_ir(old);
                    }
                }
            }
        }
    }
}
