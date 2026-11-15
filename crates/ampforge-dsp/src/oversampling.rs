//! Polyphase Half-Band Oversampling Filters (RNF-004, CA-5; auditado pela Spec 044 CA-5)
//! Suporta 1x, 2x, 4x, 8x com FIR half-band polifásico simétrico janela de Kaiser (beta = 10.8)
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

const HB_TAPS: usize = 127;
const HB_CENTER: usize = 63;

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

fn build_kaiser_halfband_kernel() -> [f64; HB_TAPS] {
    let beta = 10.8f64;
    let denom = bessel_i0(beta);
    let mut kernel = [0.0f64; HB_TAPS];
    let mut sum = 0.0f64;
    for n in 0..HB_TAPS {
        let k = n as isize - HB_CENTER as isize;
        let sinc = if k == 0 {
            0.5
        } else if k % 2 == 0 {
            0.0
        } else {
            let arg = 0.5 * std::f64::consts::PI * (k as f64);
            arg.sin() / (std::f64::consts::PI * (k as f64))
        };
        let norm = (n as f64 - HB_CENTER as f64) / (HB_CENTER as f64);
        let arg = (1.0 - norm * norm).max(0.0).sqrt();
        let win = bessel_i0(beta * arg) / denom;
        kernel[n] = sinc * win;
        sum += kernel[n];
    }
    if sum > 0.0 {
        for v in &mut kernel {
            *v /= sum;
        }
    }
    kernel
}

#[derive(Clone)]
struct HalfBandStage {
    kernel: [f64; HB_TAPS],
    up_hist: [f64; HB_TAPS],
    down_hist: [f64; HB_TAPS],
}

impl HalfBandStage {
    fn new(kernel: [f64; HB_TAPS]) -> Self {
        Self {
            kernel,
            up_hist: [0.0; HB_TAPS],
            down_hist: [0.0; HB_TAPS],
        }
    }

    fn push_and_fir_up(&mut self, s: f64) -> f64 {
        for i in (1..HB_TAPS).rev() {
            self.up_hist[i] = self.up_hist[i - 1];
        }
        self.up_hist[0] = s;
        let mut acc = self.up_hist[HB_CENTER] * self.kernel[HB_CENTER];
        // Apenas taps ímpares relativos ao centro + centro são não nulos
        let mut offset = 1usize;
        while offset <= HB_CENTER {
            acc += (self.up_hist[HB_CENTER - offset] + self.up_hist[HB_CENTER + offset])
                * self.kernel[HB_CENTER + offset];
            offset += 2;
        }
        acc
    }

    #[inline]
    fn upsample_2x(&mut self, x: f64) -> [f64; 2] {
        // Zero-stuffing (2*x, 0) filtrado pelo FIR half-band unitário
        let y0 = self.push_and_fir_up(x * 2.0);
        let y1 = self.push_and_fir_up(0.0);
        [y0, y1]
    }

    fn push_and_fir_down(&mut self, s: f64) -> f64 {
        for i in (1..HB_TAPS).rev() {
            self.down_hist[i] = self.down_hist[i - 1];
        }
        self.down_hist[0] = s;
        let mut acc = self.down_hist[HB_CENTER] * self.kernel[HB_CENTER];
        let mut offset = 1usize;
        while offset <= HB_CENTER {
            acc += (self.down_hist[HB_CENTER - offset] + self.down_hist[HB_CENTER + offset])
                * self.kernel[HB_CENTER + offset];
            offset += 2;
        }
        acc
    }

    #[inline]
    fn downsample_2x(&mut self, pair: [f64; 2]) -> f64 {
        let y0 = self.push_and_fir_down(pair[0]);
        let _ = self.push_and_fir_down(pair[1]);
        y0
    }
}

pub struct HalfBandOversampler {
    factor: OversamplingFactor,
    stages: [HalfBandStage; 3],
}

impl HalfBandOversampler {
    pub fn new(factor: OversamplingFactor) -> Self {
        let kernel = build_kaiser_halfband_kernel();
        Self {
            factor,
            stages: [
                HalfBandStage::new(kernel),
                HalfBandStage::new(kernel),
                HalfBandStage::new(kernel),
            ],
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

        let num_stages = self.factor.stages();
        let kernel = build_kaiser_halfband_kernel();
        let mut probe = Self {
            factor: self.factor,
            stages: [
                HalfBandStage::new(kernel),
                HalfBandStage::new(kernel),
                HalfBandStage::new(kernel),
            ],
        };

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

        let _ = num_stages;
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
        // Atraso de grupo aproximado por estágio em amostras na taxa base
        match self.factor {
            OversamplingFactor::X1 => 0,
            OversamplingFactor::X2 => HB_CENTER / 2,
            OversamplingFactor::X4 => (HB_CENTER * 3) / 4,
            OversamplingFactor::X8 => (HB_CENTER * 7) / 8,
        }
    }

    /// Executa upsampling polifásico em cascata sem alocações na heap.
    pub fn upsample_sample(&mut self, input: f32, out_buffer: &mut [f32]) {
        let m = self.factor.multiplier();
        if m == 1 {
            out_buffer[0] = input;
            return;
        }

        let s0 = self.stages[0].upsample_2x(input as f64);
        if m == 2 {
            out_buffer[0] = s0[0] as f32;
            out_buffer[1] = s0[1] as f32;
            return;
        }

        let a0 = self.stages[1].upsample_2x(s0[0]);
        let a1 = self.stages[1].upsample_2x(s0[1]);
        if m == 4 {
            out_buffer[0] = a0[0] as f32;
            out_buffer[1] = a0[1] as f32;
            out_buffer[2] = a1[0] as f32;
            out_buffer[3] = a1[1] as f32;
            return;
        }

        let s2_0 = self.stages[2].upsample_2x(a0[0]);
        let s2_1 = self.stages[2].upsample_2x(a0[1]);
        let s2_2 = self.stages[2].upsample_2x(a1[0]);
        let s2_3 = self.stages[2].upsample_2x(a1[1]);
        out_buffer[0] = s2_0[0] as f32;
        out_buffer[1] = s2_0[1] as f32;
        out_buffer[2] = s2_1[0] as f32;
        out_buffer[3] = s2_1[1] as f32;
        out_buffer[4] = s2_2[0] as f32;
        out_buffer[5] = s2_2[1] as f32;
        out_buffer[6] = s2_3[0] as f32;
        out_buffer[7] = s2_3[1] as f32;
    }

    /// Executa decimação polifásica em cascata sem alocações na heap.
    pub fn downsample_block(&mut self, in_buffer: &[f32]) -> f32 {
        let m = self.factor.multiplier();
        if m == 1 {
            return in_buffer[0];
        }
        if m == 2 {
            return self.stages[0].downsample_2x([in_buffer[0] as f64, in_buffer[1] as f64]) as f32;
        }
        if m == 4 {
            let d0 = self.stages[1].downsample_2x([in_buffer[0] as f64, in_buffer[1] as f64]);
            let d1 = self.stages[1].downsample_2x([in_buffer[2] as f64, in_buffer[3] as f64]);
            return self.stages[0].downsample_2x([d0, d1]) as f32;
        }
        let c0 = self.stages[2].downsample_2x([in_buffer[0] as f64, in_buffer[1] as f64]);
        let c1 = self.stages[2].downsample_2x([in_buffer[2] as f64, in_buffer[3] as f64]);
        let c2 = self.stages[2].downsample_2x([in_buffer[4] as f64, in_buffer[5] as f64]);
        let c3 = self.stages[2].downsample_2x([in_buffer[6] as f64, in_buffer[7] as f64]);
        let d0 = self.stages[1].downsample_2x([c0, c1]);
        let d1 = self.stages[1].downsample_2x([c2, c3]);
        self.stages[0].downsample_2x([d0, d1]) as f32
    }
}
