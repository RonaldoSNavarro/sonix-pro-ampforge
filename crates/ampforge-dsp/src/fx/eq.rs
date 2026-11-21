//! crates/ampforge-dsp/src/fx/eq.rs
//! Equalizadores Zero-Alloc: Paramétrico de 8 Bandas (RTA) e Gráficos Boogie 5 / MXR 10 Bandas (Spec 006, 024, 032)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqMode {
    Parametric8Band,
    Graphic5BandBoogie,
    Graphic10BandPedal,
}

#[derive(Debug, Clone, Copy)]
pub struct ParametricBand {
    pub freq_hz: f32,
    pub gain_db: f32,
    pub q: f32,
}

impl Default for ParametricBand {
    fn default() -> Self {
        Self {
            freq_hz: 1000.0,
            gain_db: 0.0,
            q: 1.0,
        }
    }
}

const PARAM_8_FREQS: [f32; 8] = [60.0, 120.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0];
const BOOGIE_5_FREQS: [f32; 5] = [80.0, 240.0, 750.0, 2200.0, 6600.0];
const MXR_10_FREQS: [f32; 10] = [
    31.25, 62.5, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];

pub struct EqProcessor {
    mode: EqMode,
    sample_rate: f32,
    num_bands: usize,
    bands: [ParametricBand; 10],
    graphic_bands_db: [f32; 10],
    // 64-bit filter state [x1, x2, y1, y2] per band
    filter_states: [[f64; 4]; 10],
    input_gain_db: f32,
    output_level_db: f32,
    band_freq_smoothers: [crate::smoother::ParamSmoother; 10],
    band_gain_smoothers: [crate::smoother::ParamSmoother; 10],
    band_q_smoothers: [crate::smoother::ParamSmoother; 10],
    input_gain_smoother: crate::smoother::ParamSmoother,
    output_level_smoother: crate::smoother::ParamSmoother,
    has_processed: bool,
}

impl EqProcessor {
    pub fn new(mode: EqMode) -> Self {
        let mut p = Self {
            mode,
            sample_rate: 48000.0,
            num_bands: 8,
            bands: [ParametricBand::default(); 10],
            graphic_bands_db: [0.0; 10],
            filter_states: [[0.0; 4]; 10],
            input_gain_db: 0.0,
            output_level_db: 0.0,
            band_freq_smoothers: [crate::smoother::ParamSmoother::new(48000.0, 0.020, 1000.0); 10],
            band_gain_smoothers: [crate::smoother::ParamSmoother::new(48000.0, 0.020, 0.0); 10],
            band_q_smoothers: [crate::smoother::ParamSmoother::new(48000.0, 0.020, 1.414); 10],
            input_gain_smoother: crate::smoother::ParamSmoother::new(48000.0, 0.020, 0.0),
            output_level_smoother: crate::smoother::ParamSmoother::new(48000.0, 0.020, 0.0),
            has_processed: false,
        };
        p.apply_mode_freqs(mode);
        p
    }

    fn apply_mode_freqs(&mut self, mode: EqMode) {
        self.mode = mode;
        let freqs: &[f32] = match mode {
            EqMode::Parametric8Band => &PARAM_8_FREQS,
            EqMode::Graphic5BandBoogie => &BOOGIE_5_FREQS,
            EqMode::Graphic10BandPedal => &MXR_10_FREQS,
        };
        self.num_bands = freqs.len();
        let default_q = match mode {
            EqMode::Graphic10BandPedal => 2.1,
            EqMode::Graphic5BandBoogie => 1.75,
            EqMode::Parametric8Band => 1.414,
        };
        for (i, &freq_hz) in freqs.iter().enumerate() {
            self.bands[i].freq_hz = freq_hz;
            self.bands[i].q = default_q;
            self.bands[i].gain_db = self.graphic_bands_db[i];
            self.band_freq_smoothers[i].reset(freq_hz as f64);
            self.band_q_smoothers[i].reset(default_q as f64);
            self.band_gain_smoothers[i].reset(self.graphic_bands_db[i] as f64);
        }
    }

    /// Alterna modo sem nenhuma alocação de heap (Spec 032, CA-6 / CA-8).
    pub fn set_mode(&mut self, mode: EqMode) {
        if self.mode != mode {
            self.filter_states = [[0.0; 4]; 10];
            self.apply_mode_freqs(mode);
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        let sr = self.sample_rate as f64;
        self.input_gain_smoother.set_sample_rate(sr);
        self.output_level_smoother.set_sample_rate(sr);
        for i in 0..10 {
            self.band_freq_smoothers[i].set_sample_rate(sr);
            self.band_gain_smoothers[i].set_sample_rate(sr);
            self.band_q_smoothers[i].set_sample_rate(sr);
        }
    }

    pub fn mode(&self) -> EqMode {
        self.mode
    }

    pub fn set_input_gain_db(&mut self, db: f32) {
        if db.is_finite() {
            let clamped = db.clamp(-12.0, 12.0);
            self.input_gain_db = clamped;
            if !self.has_processed {
                self.input_gain_smoother.reset(clamped as f64);
            } else {
                self.input_gain_smoother.set_target(clamped as f64);
            }
        }
    }

    pub fn input_gain_db(&self) -> f32 {
        self.input_gain_db
    }

    pub fn set_output_level_db(&mut self, db: f32) {
        if db.is_finite() {
            let clamped = db.clamp(-12.0, 12.0);
            self.output_level_db = clamped;
            if !self.has_processed {
                self.output_level_smoother.reset(clamped as f64);
            } else {
                self.output_level_smoother.set_target(clamped as f64);
            }
        }
    }

    pub fn output_level_db(&self) -> f32 {
        self.output_level_db
    }

    pub fn graphic_frequencies(&self) -> Vec<f32> {
        self.bands[..self.num_bands]
            .iter()
            .map(|b| b.freq_hz)
            .collect()
    }

    pub fn graphic_bands_db(&self) -> &[f32] {
        &self.graphic_bands_db[..self.num_bands]
    }

    pub fn set_graphic_band_db(&mut self, idx: usize, db: f32) {
        if idx < 10 && db.is_finite() {
            let clamped = db.clamp(-15.0, 15.0);
            self.graphic_bands_db[idx] = clamped;
            self.bands[idx].gain_db = clamped;
            if !self.has_processed {
                self.band_gain_smoothers[idx].reset(clamped as f64);
            } else {
                self.band_gain_smoothers[idx].set_target(clamped as f64);
            }
        }
    }

    pub fn parametric_band(&self, idx: usize) -> ParametricBand {
        if idx < 10 {
            self.bands[idx]
        } else {
            ParametricBand::default()
        }
    }

    pub fn set_parametric_band(&mut self, idx: usize, freq_hz: f32, gain_db: f32, q: f32) {
        if idx < 10 {
            let clamped_freq = freq_hz.clamp(20.0, 20000.0);
            let clamped_db = gain_db.clamp(-18.0, 18.0);
            let clamped_q = q.clamp(0.1, 20.0);
            self.bands[idx] = ParametricBand {
                freq_hz: clamped_freq,
                gain_db: clamped_db,
                q: clamped_q,
            };
            self.graphic_bands_db[idx] = clamped_db;
            if !self.has_processed {
                self.band_freq_smoothers[idx].reset(clamped_freq as f64);
                self.band_gain_smoothers[idx].reset(clamped_db as f64);
                self.band_q_smoothers[idx].reset(clamped_q as f64);
            } else {
                self.band_freq_smoothers[idx].set_target(clamped_freq as f64);
                self.band_gain_smoothers[idx].set_target(clamped_db as f64);
                self.band_q_smoothers[idx].set_target(clamped_q as f64);
            }
        }
    }

    #[inline(always)]
    fn apply_band_biquad(
        mode: EqMode,
        sr: f64,
        st: &mut [f64; 4],
        slice: &mut [f32],
        freq_hz: f64,
        gain_db: f64,
        q: f64,
    ) {
        let a_lin = 10.0f64.powf(gain_db / 40.0);
        let omega = 2.0 * std::f64::consts::PI * freq_hz / sr;
        let eff_q = if mode == EqMode::Graphic10BandPedal {
            q * (1.0 + 0.025 * gain_db.abs())
        } else {
            q
        };
        let alpha = omega.sin() / (2.0 * eff_q.max(0.05));
        let cos_w = omega.cos();

        let b0 = 1.0 + alpha * a_lin;
        let b1 = -2.0 * cos_w;
        let b2 = 1.0 - alpha * a_lin;
        let a0 = 1.0 + alpha / a_lin;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha / a_lin;

        let inv_a0 = 1.0 / a0;
        let b0 = b0 * inv_a0;
        let b1 = b1 * inv_a0;
        let b2 = b2 * inv_a0;
        let a1 = a1 * inv_a0;
        let a2 = a2 * inv_a0;

        for s in slice.iter_mut() {
            let x = crate::denormal::flush_f64(*s as f64);
            let y = crate::denormal::flush_f64(
                b0 * x + b1 * st[0] + b2 * st[1] - a1 * st[2] - a2 * st[3],
            );
            st[1] = st[0];
            st[0] = x;
            st[3] = st[2];
            st[2] = y;
            *s = crate::denormal::flush_f32(y as f32);
        }
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        self.has_processed = true;

        let any_ramping = self.input_gain_smoother.is_ramping()
            || self.output_level_smoother.is_ramping()
            || (0..self.num_bands).any(|i| {
                self.band_freq_smoothers[i].is_ramping()
                    || self.band_gain_smoothers[i].is_ramping()
                    || self.band_q_smoothers[i].is_ramping()
            });

        let total_trim_db = self.input_gain_db + self.output_level_db;
        let active = &self.bands[..self.num_bands];
        let all_flat = active.iter().all(|b| b.gain_db.abs() < 1e-4);

        if !any_ramping && all_flat && total_trim_db.abs() < 1e-4 {
            output.copy_from_slice(input);
            return;
        }

        let sr = self.sample_rate as f64;
        if !any_ramping {
            let in_trim = 10.0f32.powf((self.input_gain_smoother.current() as f32) / 20.0);
            if (in_trim - 1.0).abs() > 1e-5 {
                for (o, &x) in output.iter_mut().zip(input.iter()) {
                    *o = x * in_trim;
                }
            } else {
                output.copy_from_slice(input);
            }

            for b_idx in 0..self.num_bands {
                let g = self.band_gain_smoothers[b_idx].current();
                if g.abs() < 1e-4 {
                    continue;
                }
                let f = self.band_freq_smoothers[b_idx].current();
                let q = self.band_q_smoothers[b_idx].current();
                Self::apply_band_biquad(
                    self.mode,
                    sr,
                    &mut self.filter_states[b_idx],
                    output,
                    f,
                    g,
                    q,
                );
            }

            let out_trim = 10.0f32.powf((self.output_level_smoother.current() as f32) / 20.0);
            if (out_trim - 1.0).abs() > 1e-5 {
                for s in output.iter_mut() {
                    *s *= out_trim;
                }
            }
            return;
        }

        let len = input.len();
        let mut offset = 0usize;
        while offset < len {
            let chunk = (len - offset).min(4);
            for k in 0..chunk {
                let in_db = self.input_gain_smoother.next_sample() as f32;
                let in_trim = 10.0f32.powf(in_db / 20.0);
                output[offset + k] = input[offset + k] * in_trim;
            }

            for b_idx in 0..self.num_bands {
                let band_ramping = self.band_gain_smoothers[b_idx].is_ramping()
                    || self.band_freq_smoothers[b_idx].is_ramping()
                    || self.band_q_smoothers[b_idx].is_ramping();
                let f = self.band_freq_smoothers[b_idx].advance(chunk);
                let g = self.band_gain_smoothers[b_idx].advance(chunk);
                let q = self.band_q_smoothers[b_idx].advance(chunk);
                if g.abs() < 1e-4 && !band_ramping {
                    continue;
                }
                Self::apply_band_biquad(
                    self.mode,
                    sr,
                    &mut self.filter_states[b_idx],
                    &mut output[offset..offset + chunk],
                    f,
                    g,
                    q,
                );
            }

            for k in 0..chunk {
                let out_db = self.output_level_smoother.next_sample() as f32;
                let out_trim = 10.0f32.powf(out_db / 20.0);
                output[offset + k] *= out_trim;
            }

            offset += chunk;
        }
    }
}
