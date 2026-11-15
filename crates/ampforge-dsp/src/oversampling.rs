//! Polyphase Half-Band Oversampling Filters (RNF-004, CA-5; Spec 049 CA-12 & CA-13)
//! Suporta 1x, 2x, 4x, 8x com FIR half-band polifásico simétrico em ring buffer (zero shift)
//! e medição real via DFT em `stopband_attenuation_db()`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OversamplingFactor {
    X1 = 1,
    X2 = 2,
    X4 = 4,
    X8 = 8,
}

impl OversamplingFactor {
    pub fn multiplier(self) -> usize {
        self as usize
    }

    fn stages(self) -> usize {
        match self {
            OversamplingFactor::X1 => 0,
            OversamplingFactor::X2 => 1,
            OversamplingFactor::X4 => 2,
            OversamplingFactor::X8 => 3,
        }
    }
}

const STAGE0_TAPS: usize = 103;
const STAGE0_CENTER: usize = 51;
const STAGE0_HALF: usize = 26; // (51 + 1) / 2

const STAGE1_TAPS: usize = 43;
const STAGE1_CENTER: usize = 21;
const STAGE1_HALF: usize = 11; // (21 + 1) / 2

const STAGE2_TAPS: usize = 27;
const STAGE2_CENTER: usize = 13;
const STAGE2_HALF: usize = 7; // (13 + 1) / 2

const RING_SIZE: usize = 64; // Potência de 2 >= max(2 * STAGE0_HALF) = 48
const RING_MASK: usize = RING_SIZE - 1;

fn bessel_i0(x: f64) -> f64 {
    let half_x = 0.5 * x;
    let mut term = 1.0f64;
    let mut sum = 1.0f64;
    for k in 1..=28 {
        let kf = k as f64;
        term *= (half_x * half_x) / (kf * kf);
        sum += term;
        if term < 1e-16 * sum {
            break;
        }
    }
    sum
}

/// Constrói apenas os `HALF` coeficientes ímpares positivos (offset = 1, 3, 5, ...)
/// de um FIR half-band simétrico janela de Kaiser normalizado (DC gain = 1.0, center = 0.5).
fn build_halfband_odd_taps<const TAPS: usize, const CENTER: usize, const HALF: usize>(
    beta: f64,
) -> ([f32; HALF], f32) {
    let denom = bessel_i0(beta);
    let mut full = [0.0f64; TAPS];
    let mut sum = 0.0f64;
    for n in 0..TAPS {
        let k = n as isize - CENTER as isize;
        let sinc = if k == 0 {
            0.5
        } else if k % 2 == 0 {
            0.0
        } else {
            let arg = 0.5 * std::f64::consts::PI * (k as f64);
            arg.sin() / (std::f64::consts::PI * (k as f64))
        };
        let norm = (n as f64 - CENTER as f64) / (CENTER as f64);
        let arg = (1.0 - norm * norm).max(0.0).sqrt();
        let win = bessel_i0(beta * arg) / denom;
        full[n] = sinc * win;
        sum += full[n];
    }
    if sum > 0.0 {
        for v in &mut full {
            *v /= sum;
        }
    }
    let mut odd = [0.0f32; HALF];
    for i in 0..HALF {
        let offset = 2 * i + 1;
        odd[i] = full[CENTER + offset] as f32;
    }
    (odd, full[CENTER] as f32)
}

#[derive(Clone)]
struct PolyphaseStage<const HALF: usize> {
    odd_taps: [f32; HALF],
    center_tap: f32,
    up_ring: [f32; RING_SIZE],
    up_pos: usize,
    down_even_ring: [f32; RING_SIZE],
    down_odd_ring: [f32; RING_SIZE],
    down_pos: usize,
}

impl<const HALF: usize> PolyphaseStage<HALF> {
    fn new(odd_taps: [f32; HALF], center_tap: f32) -> Self {
        Self {
            odd_taps,
            center_tap,
            up_ring: [0.0; RING_SIZE],
            up_pos: 0,
            down_even_ring: [0.0; RING_SIZE],
            down_odd_ring: [0.0; RING_SIZE],
            down_pos: 0,
        }
    }

    #[inline(always)]
    fn upsample_2x(&mut self, x: f32) -> [f32; 2] {
        // Zero-stuffing (2*x, 0) em decomposição polifásica:
        // Ramo par (saída 0): convolução simétrica de tamanho 2*HALF com taps ímpares (2*x).
        // Ramo ímpar (saída 1): amostra atrasada de HALF-1 multiplicada por 2*center_tap (≈ 1.0).
        let pos = (self.up_pos + 1) & RING_MASK;
        self.up_pos = pos;
        self.up_ring[pos] = x * 2.0;

        let mut acc = 0.0f32;
        for i in 0..HALF {
            let a = self.up_ring[(pos.wrapping_sub(HALF - 1 - i)) & RING_MASK];
            let b = self.up_ring[(pos.wrapping_sub(HALF + i)) & RING_MASK];
            acc += (a + b) * self.odd_taps[i];
        }

        let y1 = self.up_ring[(pos.wrapping_sub(HALF - 1)) & RING_MASK] * self.center_tap;
        [acc, y1]
    }

    #[inline(always)]
    fn downsample_2x(&mut self, pair: [f32; 2]) -> f32 {
        // Para manter fase exata com `push_and_fir_down(pair[0]); push_and_fir_down(pair[1]);`:
        // Na saída de pair[0], `pair[0]` está nos índices pares (offsets ímpares do centro),
        // e `pair[1]` anterior está nos índices ímpares (incluindo o centro).
        let pos = (self.down_pos + 1) & RING_MASK;
        self.down_pos = pos;
        self.down_even_ring[pos] = pair[0];

        let mut acc = self.down_odd_ring[(pos.wrapping_sub(HALF)) & RING_MASK] * self.center_tap;
        for i in 0..HALF {
            let a = self.down_even_ring[(pos.wrapping_sub(HALF - 1 - i)) & RING_MASK];
            let b = self.down_even_ring[(pos.wrapping_sub(HALF + i)) & RING_MASK];
            acc += (a + b) * self.odd_taps[i];
        }

        self.down_odd_ring[pos] = pair[1];
        acc
    }
}

pub struct HalfBandOversampler {
    factor: OversamplingFactor,
    stage0: PolyphaseStage<STAGE0_HALF>,
    stage1: PolyphaseStage<STAGE1_HALF>,
    stage2: PolyphaseStage<STAGE2_HALF>,
}

impl HalfBandOversampler {
    pub fn new(factor: OversamplingFactor) -> Self {
        let (odd0, c0) = build_halfband_odd_taps::<STAGE0_TAPS, STAGE0_CENTER, STAGE0_HALF>(11.5);
        let (odd1, c1) = build_halfband_odd_taps::<STAGE1_TAPS, STAGE1_CENTER, STAGE1_HALF>(11.5);
        let (odd2, c2) = build_halfband_odd_taps::<STAGE2_TAPS, STAGE2_CENTER, STAGE2_HALF>(11.5);
        Self {
            factor,
            stage0: PolyphaseStage::new(odd0, c0),
            stage1: PolyphaseStage::new(odd1, c1),
            stage2: PolyphaseStage::new(odd2, c2),
        }
    }

    pub fn factor(&self) -> OversamplingFactor {
        self.factor
    }

    /// Mede a atenuação real na banda de rejeição do filtro implementado via resposta ao impulso + DFT (Spec 044 CA-5).
    pub fn stopband_attenuation_db(&self) -> f64 {
        let m = self.factor.multiplier();
        if m == 1 {
            return 0.0;
        }

        let _num_stages = self.factor.stages();
        let mut probe = Self::new(self.factor);

        // Coleta resposta ao impulso completa do interpolador implementado
        let base_len = 256usize;
        let up_len = base_len * m;
        let mut ir = vec![0.0f64; up_len];
        let mut tmp = [0.0f32; 8];
        for n in 0..base_len {
            let x = if n == 0 { 1.0f32 } else { 0.0f32 };
            probe.upsample_sample(x, &mut tmp[..m]);
            for k in 0..m {
                ir[n * m + k] = tmp[k] as f64;
            }
        }

        // Ganho DC real na banda passante (escalado por M pois a interpolação preserva amplitude)
        let dc_mag = ir.iter().sum::<f64>().abs().max(1e-12);

        // Varre a banda de rejeição real acima da Nyquist base (pi/M + transição): [1.20 * pi / M, pi]
        let num_bins = 256usize;
        let start_omega = 1.20 * std::f64::consts::PI / (m as f64);
        let end_omega = std::f64::consts::PI;
        let mut max_stop_mag = 1e-15f64;

        for b in 0..=num_bins {
            let omega = start_omega + (end_omega - start_omega) * (b as f64 / num_bins as f64);
            let mut re = 0.0f64;
            let mut im = 0.0f64;
            for (n, &h) in ir.iter().enumerate() {
                if h == 0.0 {
                    continue;
                }
                let angle = omega * (n as f64);
                re += h * angle.cos();
                im -= h * angle.sin();
            }
            let mag = (re * re + im * im).sqrt();
            if mag > max_stop_mag {
                max_stop_mag = mag;
            }
        }

        20.0 * (dc_mag / max_stop_mag).log10()
    }

    pub fn latency_samples(&self) -> usize {
        // Atraso de grupo roundtrip (upsample + downsample) em amostras na taxa base:
        // Stage 0: STAGE0_CENTER = 51 amostras na taxa base.
        // Stage 1: STAGE1_CENTER / 2 = 10.5 amostras na taxa base.
        // Stage 2: STAGE2_CENTER / 4 = 3.25 amostras na taxa base (total 8x = 64.75 -> 65).
        match self.factor {
            OversamplingFactor::X1 => 0,
            OversamplingFactor::X2 => STAGE0_CENTER,
            OversamplingFactor::X4 => STAGE0_CENTER + STAGE1_CENTER / 2,
            OversamplingFactor::X8 => {
                (STAGE0_CENTER as f64 + (STAGE1_CENTER as f64) * 0.5 + (STAGE2_CENTER as f64) * 0.25)
                    .round() as usize
            }
        }
    }

    /// Executa upsampling polifásico em cascata sem alocações na heap.
    #[inline]
    pub fn upsample_sample(&mut self, input: f32, out_buffer: &mut [f32]) {
        let m = self.factor.multiplier();
        if m == 1 {
            out_buffer[0] = input;
            return;
        }

        let s0 = self.stage0.upsample_2x(input);
        if m == 2 {
            out_buffer[0] = s0[0];
            out_buffer[1] = s0[1];
            return;
        }

        let a0 = self.stage1.upsample_2x(s0[0]);
        let a1 = self.stage1.upsample_2x(s0[1]);
        if m == 4 {
            out_buffer[0] = a0[0];
            out_buffer[1] = a0[1];
            out_buffer[2] = a1[0];
            out_buffer[3] = a1[1];
            return;
        }

        let s2_0 = self.stage2.upsample_2x(a0[0]);
        let s2_1 = self.stage2.upsample_2x(a0[1]);
        let s2_2 = self.stage2.upsample_2x(a1[0]);
        let s2_3 = self.stage2.upsample_2x(a1[1]);
        out_buffer[0] = s2_0[0];
        out_buffer[1] = s2_0[1];
        out_buffer[2] = s2_1[0];
        out_buffer[3] = s2_1[1];
        out_buffer[4] = s2_2[0];
        out_buffer[5] = s2_2[1];
        out_buffer[6] = s2_3[0];
        out_buffer[7] = s2_3[1];
    }

    /// Executa decimação polifásica em cascata sem alocações na heap.
    #[inline]
    pub fn downsample_block(&mut self, in_buffer: &[f32]) -> f32 {
        let m = self.factor.multiplier();
        if m == 1 {
            return in_buffer[0];
        }
        if m == 2 {
            return self.stage0.downsample_2x([in_buffer[0], in_buffer[1]]);
        }
        if m == 4 {
            let d0 = self.stage1.downsample_2x([in_buffer[0], in_buffer[1]]);
            let d1 = self.stage1.downsample_2x([in_buffer[2], in_buffer[3]]);
            return self.stage0.downsample_2x([d0, d1]);
        }
        let c0 = self.stage2.downsample_2x([in_buffer[0], in_buffer[1]]);
        let c1 = self.stage2.downsample_2x([in_buffer[2], in_buffer[3]]);
        let c2 = self.stage2.downsample_2x([in_buffer[4], in_buffer[5]]);
        let c3 = self.stage2.downsample_2x([in_buffer[6], in_buffer[7]]);
        let d0 = self.stage1.downsample_2x([c0, c1]);
        let d1 = self.stage1.downsample_2x([c2, c3]);
        self.stage0.downsample_2x([d0, d1])
    }
}
