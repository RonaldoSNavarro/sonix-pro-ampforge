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
    }

    pub fn mode(&self) -> EqMode {
        self.mode
    }

    pub fn set_input_gain_db(&mut self, db: f32) {
        if db.is_finite() {
            self.input_gain_db = db.clamp(-12.0, 12.0);
        }
    }

    pub fn input_gain_db(&self) -> f32 {
        self.input_gain_db
    }

    pub fn set_output_level_db(&mut self, db: f32) {
        if db.is_finite() {
            self.output_level_db = db.clamp(-12.0, 12.0);
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
            let clamped_db = gain_db.clamp(-18.0, 18.0);
            self.bands[idx] = ParametricBand {
                freq_hz: freq_hz.clamp(20.0, 20000.0),
                gain_db: clamped_db,
                q: q.clamp(0.1, 20.0),
            };
            self.graphic_bands_db[idx] = clamped_db;
        }
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());

        let total_trim_db = self.input_gain_db + self.output_level_db;
        let active = &self.bands[..self.num_bands];
        let all_flat = active.iter().all(|b| b.gain_db.abs() < 1e-4);

        if all_flat && total_trim_db.abs() < 1e-4 {
            output.copy_from_slice(input);
            return;
        }

        let in_trim = 10.0f32.powf(self.input_gain_db / 20.0);
        if (in_trim - 1.0).abs() > 1e-5 {
            for (o, &x) in output.iter_mut().zip(input.iter()) {
                *o = x * in_trim;
            }
        } else {
            output.copy_from_slice(input);
        }

        for (b_idx, band) in active.iter().enumerate() {
            if band.gain_db.abs() < 1e-4 {
                continue;
            }

            let a_lin = 10.0f64.powf(band.gain_db as f64 / 40.0);
            let omega =
                2.0 * std::f64::consts::PI * (band.freq_hz as f64) / (self.sample_rate as f64);
            let eff_q = if self.mode == EqMode::Graphic10BandPedal {
                (band.q as f64) * (1.0 + 0.025 * (band.gain_db.abs() as f64))
            } else {
                band.q as f64
            };
            let alpha = omega.sin() / (2.0 * eff_q);
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

            let st = &mut self.filter_states[b_idx];
            for i in 0..output.len() {
                let x = crate::denormal::flush_f64(output[i] as f64);
                let y = crate::denormal::flush_f64(
                    b0 * x + b1 * st[0] + b2 * st[1] - a1 * st[2] - a2 * st[3],
                );

                st[1] = st[0];
                st[0] = x;
                st[3] = st[2];
                st[2] = y;

                output[i] = crate::denormal::flush_f32(y as f32);
            }
        }

        let out_trim = 10.0f32.powf(self.output_level_db / 20.0);
        if (out_trim - 1.0).abs() > 1e-5 {
            for s in output.iter_mut() {
                *s *= out_trim;
            }
        }
    }
}
