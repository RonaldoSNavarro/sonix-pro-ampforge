//! crates/ampforge-dsp/src/fx/eq.rs
//! Equalizadores: Paramétrico de 8 Bandas (RTA) e Gráficos de 5/10 Bandas

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

pub struct EqProcessor {
    mode: EqMode,
    sample_rate: f32,
    bands: Vec<ParametricBand>,
    graphic_bands_db: Vec<f32>,
    // 64-bit filter state [x1, x2, y1, y2] per band
    filter_states: Vec<[f64; 4]>,
}

impl EqProcessor {
    pub fn new(mode: EqMode) -> Self {
        let (num_bands, default_freqs) = match mode {
            EqMode::Parametric8Band => (
                8,
                vec![60.0, 120.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0],
            ),
            EqMode::Graphic5BandBoogie => (5, vec![80.0, 240.0, 750.0, 2200.0, 6600.0]),
            EqMode::Graphic10BandPedal => (
                10,
                vec![
                    31.25, 62.5, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
                ],
            ),
        };

        let bands = default_freqs
            .into_iter()
            .map(|freq_hz| ParametricBand {
                freq_hz,
                gain_db: 0.0,
                q: 1.414,
            })
            .collect();

        Self {
            mode,
            sample_rate: 48000.0,
            bands,
            graphic_bands_db: vec![0.0; num_bands],
            filter_states: vec![[0.0; 4]; num_bands],
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn mode(&self) -> EqMode {
        self.mode
    }

    pub fn graphic_frequencies(&self) -> Vec<f32> {
        self.bands.iter().map(|b| b.freq_hz).collect()
    }

    pub fn graphic_bands_db(&self) -> &[f32] {
        &self.graphic_bands_db
    }

    pub fn set_graphic_band_db(&mut self, idx: usize, db: f32) {
        if idx < self.graphic_bands_db.len() {
            self.graphic_bands_db[idx] = db.clamp(-15.0, 15.0);
            if idx < self.bands.len() {
                self.bands[idx].gain_db = self.graphic_bands_db[idx];
            }
        }
    }

    pub fn parametric_band(&self, idx: usize) -> ParametricBand {
        if idx < self.bands.len() {
            self.bands[idx]
        } else {
            ParametricBand::default()
        }
    }

    pub fn set_parametric_band(&mut self, idx: usize, freq_hz: f32, gain_db: f32, q: f32) {
        if idx < self.bands.len() {
            self.bands[idx] = ParametricBand {
                freq_hz: freq_hz.clamp(20.0, 20000.0),
                gain_db: gain_db.clamp(-18.0, 18.0),
                q: q.clamp(0.1, 20.0),
            };
        }
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());

        // Fast path: if all gains are 0.0 dB, copy directly (bit-exact transparency)
        let all_flat = self.bands.iter().all(|b| b.gain_db.abs() < 1e-4);
        if all_flat {
            output.copy_from_slice(input);
            return;
        }

        output.copy_from_slice(input);

        // Process peaking biquad filters in float64 for high numerical stability
        for (b_idx, band) in self.bands.iter().enumerate() {
            if band.gain_db.abs() < 1e-4 {
                continue;
            }

            let a_lin = 10.0f64.powf(band.gain_db as f64 / 40.0);
            let omega = 2.0 * std::f64::consts::PI * (band.freq_hz as f64) / (self.sample_rate as f64);
            let alpha = omega.sin() / (2.0 * band.q as f64);
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
                let x = output[i] as f64;
                let y = b0 * x + b1 * st[0] + b2 * st[1] - a1 * st[2] - a2 * st[3];

                st[1] = st[0];
                st[0] = x;
                st[3] = st[2];
                st[2] = y;

                output[i] = y as f32;
            }
        }
    }
}
