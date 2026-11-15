//! crates/ampforge-dsp/src/tuner.rs
//! Afinador Estroboscópico de Alta Precisão (±0.1 cent) e Polifônico de 6 Cordas

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunerMode {
    Strobe,
    Polyphonic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunerTemperament {
    EqualTemperament,
    SweetenedGuitar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunerOutputMode {
    Mute,
    Bypass,
}

#[derive(Debug, Clone)]
pub struct TunerPitchResult {
    pub note_name: String,
    pub cents_deviation: f32,
    pub frequency_hz: f32,
}

#[derive(Debug, Clone)]
pub struct PolyphonicStringStatus {
    pub string_index: usize,
    pub note: String,
    pub cents_deviation: f32,
}

pub struct TunerProcessor {
    is_active: bool,
    mode: TunerMode,
    temperament: TunerTemperament,
    output_mode: TunerOutputMode,
    a4_ref: f32,
    sample_rate: f32,
    mute_gain: f32,
    d_scratch: Vec<f32>,
    d_prime_scratch: Vec<f32>,
    last_poly_cents: [f32; 6],
}

impl Default for TunerProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TunerProcessor {
    pub fn new() -> Self {
        Self {
            is_active: false,
            mode: TunerMode::Strobe,
            temperament: TunerTemperament::EqualTemperament,
            output_mode: TunerOutputMode::Mute,
            a4_ref: 440.0,
            sample_rate: 48000.0,
            mute_gain: 1.0,
            d_scratch: vec![0.0; 4096],
            d_prime_scratch: vec![0.0; 4096],
            last_poly_cents: [0.0; 6],
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    pub fn toggle(&mut self) {
        self.is_active = !self.is_active;
    }

    pub fn on_footswitch_event(&mut self, _pressed: bool, hold_duration_ms: u64) {
        if hold_duration_ms >= 500 {
            self.is_active = true;
        }
    }

    pub fn mode(&self) -> TunerMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: TunerMode) {
        self.mode = mode;
    }

    pub fn temperament(&self) -> TunerTemperament {
        self.temperament
    }

    pub fn set_temperament(&mut self, temp: TunerTemperament) {
        self.temperament = temp;
    }

    pub fn output_mode(&self) -> TunerOutputMode {
        self.output_mode
    }

    pub fn set_output_mode(&mut self, mode: TunerOutputMode) {
        self.output_mode = mode;
    }

    pub fn a4_reference_hz(&self) -> f32 {
        self.a4_ref
    }

    pub fn set_a4_reference_hz(&mut self, hz: f32) {
        self.a4_ref = hz.clamp(430.0, 450.0);
    }

    pub fn detect_pitch(&mut self, signal: &[f32]) -> TunerPitchResult {
        if signal.len() < 256 {
            return TunerPitchResult {
                note_name: "---".to_string(),
                cents_deviation: 0.0,
                frequency_hz: 0.0,
            };
        }

        let n = signal.len();
        let rms = (signal.iter().map(|&s| s * s).sum::<f32>() / n as f32).sqrt();
        if rms < 0.00063 {
            return TunerPitchResult {
                note_name: "---".to_string(),
                cents_deviation: 0.0,
                frequency_hz: 0.0,
            };
        }

        if self.mode == TunerMode::Polyphonic {
            self.update_polyphonic_from_samples(signal);
        }

        // YIN Algorithm:
        // Cobre de 58 Hz a 1450 Hz (Spec 026: 60 Hz a 1.4 kHz)
        let max_lag = (n / 2).min((self.sample_rate / 58.0).ceil() as usize);
        let min_lag = (self.sample_rate / 1450.0).floor() as usize;

        if max_lag <= min_lag {
            return TunerPitchResult {
                note_name: "---".to_string(),
                cents_deviation: 0.0,
                frequency_hz: 0.0,
            };
        }

        if self.d_scratch.len() < max_lag {
            self.d_scratch.resize(max_lag, 0.0);
        }
        if self.d_prime_scratch.len() < max_lag {
            self.d_prime_scratch.resize(max_lag, 0.0);
        }

        for tau in 1..max_lag {
            let mut sum = 0.0f32;
            let len = n - max_lag;
            for j in 0..len {
                let diff = signal[j] - signal[j + tau];
                sum += diff * diff;
            }
            self.d_scratch[tau] = sum;
        }

        // 2. Cumulative mean normalized difference function (CMND)
        self.d_prime_scratch[0] = 1.0;
        let mut running_sum = 0.0f32;
        for tau in 1..max_lag {
            running_sum += self.d_scratch[tau];
            self.d_prime_scratch[tau] = if running_sum > 1e-9 {
                self.d_scratch[tau] * (tau as f32) / running_sum
            } else {
                1.0
            };
        }

        // 3. Absolute thresholding (first valley below 0.15)
        let threshold = 0.15f32;
        let mut best_tau = 0;
        let mut tau = min_lag;
        while tau < max_lag {
            if self.d_prime_scratch[tau] < threshold {
                while tau + 1 < max_lag && self.d_prime_scratch[tau + 1] < self.d_prime_scratch[tau] {
                    tau += 1;
                }
                best_tau = tau;
                break;
            }
            tau += 1;
        }

        if best_tau == 0 {
            // Fallback to absolute minimum if threshold was not crossed
            let mut min_val = f32::MAX;
            for t in min_lag..max_lag {
                if self.d_prime_scratch[t] < min_val {
                    min_val = self.d_prime_scratch[t];
                    best_tau = t;
                }
            }
        }

        // Se a periodicidade for fraca (ruído ou não-harmônico)
        if best_tau == 0 || self.d_prime_scratch[best_tau] > 0.35 {
            return TunerPitchResult {
                note_name: "---".to_string(),
                cents_deviation: 0.0,
                frequency_hz: 0.0,
            };
        }

        // 4. Parabolic interpolation for sub-sample accuracy on symmetric difference function d
        let tau_sub = if best_tau > min_lag && best_tau + 1 < max_lag {
            let s0 = self.d_scratch[best_tau - 1];
            let s1 = self.d_scratch[best_tau];
            let s2 = self.d_scratch[best_tau + 1];
            let denom = 2.0 * (s0 - 2.0 * s1 + s2);
            if denom.abs() > 1e-9 {
                best_tau as f32 + (s0 - s2) / denom
            } else {
                best_tau as f32
            }
        } else {
            best_tau as f32
        };

        let freq = self.sample_rate / tau_sub;

        if self.mode == TunerMode::Polyphonic {
            self.update_polyphonic_from_samples(signal);
        }

        if freq < 60.0 || freq > 1400.0 {
            return TunerPitchResult {
                note_name: "---".to_string(),
                cents_deviation: 0.0,
                frequency_hz: 0.0,
            };
        }

        // Convert freq to MIDI note and cents deviation from nearest semitone
        let midi_float = 69.0 + 12.0 * (freq / self.a4_ref).log2();
        let nearest_midi = midi_float.round() as i32;
        let mut cents = (midi_float - nearest_midi as f32) * 100.0;

        if self.temperament == TunerTemperament::SweetenedGuitar {
            cents -= Self::sweetened_offset_cents(nearest_midi);
        }

        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
        let note_idx = ((nearest_midi % 12) + 12) % 12;
        let octave = (nearest_midi / 12) - 1;
        let note_name = format!("{}{}", note_names[note_idx as usize], octave);

        TunerPitchResult {
            note_name,
            cents_deviation: cents,
            frequency_hz: freq,
        }
    }

    /// Peterson StroboStomp "GTR" Sweetened Guitar Tuning offsets (cents relative to Equal Temperament):
    /// E2 (-2.4), A2 (-2.0), D3 (-1.6), G3 (-3.2), B3 (-1.0), E4 (-2.0).
    fn sweetened_offset_cents(midi: i32) -> f32 {
        match midi {
            40 => -2.4, // E2
            45 => -2.0, // A2
            50 => -1.6, // D3
            55 => -3.2, // G3
            59 => -1.0, // B3
            64 => -2.0, // E4
            _ => match ((midi % 12) + 12) % 12 {
                4 => -2.2,  // E
                9 => -2.0,  // A
                2 => -1.6,  // D
                7 => -3.2,  // G
                11 => -1.0, // B
                _ => 0.0,
            },
        }
    }

    fn estimate_string_freq_hanning(samples: &[f32], f_ref: f32, sample_rate: f32) -> f32 {
        let n = samples.len();
        if n < 64 {
            return f_ref;
        }
        let shift = (n / 8).max(1);
        let m = n - shift;
        let inv_m1 = 1.0 / ((m - 1) as f64);
        let w_ref = 2.0 * std::f64::consts::PI * (f_ref as f64) / (sample_rate as f64);
        let mut re0 = 0.0f64;
        let mut im0 = 0.0f64;
        let mut re1 = 0.0f64;
        let mut im1 = 0.0f64;
        for i in 0..m {
            let win = 0.54 - 0.46 * (2.0 * std::f64::consts::PI * (i as f64) * inv_m1).cos();
            let angle = w_ref * (i as f64);
            let c = angle.cos();
            let s = angle.sin();
            let x0 = (samples[i] as f64) * win;
            let x1 = (samples[i + shift] as f64) * win;
            re0 += x0 * c;
            im0 -= x0 * s;
            re1 += x1 * c;
            im1 -= x1 * s;
        }
        // X1 * conj(X0) * exp(-j * w_ref * shift) -> phase is (w_actual - w_ref) * shift
        let cross_re = re1 * re0 + im1 * im0;
        let cross_im = im1 * re0 - re1 * im0;
        let shift_ang = w_ref * (shift as f64);
        let sc = shift_ang.cos();
        let ss = shift_ang.sin();
        let norm_re = cross_re * sc + cross_im * ss;
        let norm_im = cross_im * sc - cross_re * ss;
        let delta_w = norm_im.atan2(norm_re) / (shift as f64);
        ((w_ref + delta_w) * (sample_rate as f64) / (2.0 * std::f64::consts::PI)) as f32
    }

    fn update_polyphonic_from_samples(&mut self, samples: &[f32]) {
        let base_strings = [
            (40i32, 82.40689f32),
            (45, 110.00000),
            (50, 146.83238),
            (55, 195.99772),
            (59, 246.94165),
            (64, 329.62756),
        ];
        let n = samples.len().min(self.d_scratch.len());
        if n < 64 {
            return;
        }
        let a4_scale = self.a4_ref / 440.0;
        let mut f_refs = [0.0f32; 6];
        let mut a_coef = [0.0f64; 6];
        let mut b_coef = [0.0f64; 6];

        // Pass 1: estimate complex Fourier coefficients for each string (Hanning-windowed)
        let inv_n1 = 1.0 / ((n - 1) as f64);
        let norm = 4.0 / (n as f64); // 2/N * 2 for Hanning mean 0.5
        for (k, &(midi, f_eq)) in base_strings.iter().enumerate() {
            let offset_c = if self.temperament == TunerTemperament::SweetenedGuitar {
                Self::sweetened_offset_cents(midi)
            } else {
                0.0
            };
            f_refs[k] = f_eq * a4_scale * 2.0f32.powf(offset_c / 1200.0);
            let w = 2.0 * std::f64::consts::PI * (f_refs[k] as f64) / (self.sample_rate as f64);
            let mut sum_c = 0.0f64;
            let mut sum_s = 0.0f64;
            for i in 0..n {
                let win = 0.5 * (1.0 - (2.0 * std::f64::consts::PI * (i as f64) * inv_n1).cos());
                let x = (samples[i] as f64) * win;
                let ang = w * (i as f64);
                sum_c += x * ang.cos();
                sum_s += x * ang.sin();
            }
            a_coef[k] = sum_c * norm;
            b_coef[k] = sum_s * norm;
        }

        // Pass 2: subtract interfering strings and measure each string cleanly
        for s_idx in 0..6 {
            for i in 0..n {
                let mut r = samples[i] as f64;
                for j in 0..6 {
                    if j != s_idx {
                        let w =
                            2.0 * std::f64::consts::PI * (f_refs[j] as f64) / (self.sample_rate as f64);
                        let ang = w * (i as f64);
                        r -= a_coef[j] * ang.cos() + b_coef[j] * ang.sin();
                    }
                }
                self.d_scratch[i] = r as f32;
            }
            let f_ref = f_refs[s_idx];
            let f_est = Self::estimate_string_freq_hanning(&self.d_scratch[..n], f_ref, self.sample_rate);
            if f_est > 20.0 {
                self.last_poly_cents[s_idx] = 1200.0 * (f_est / f_ref).log2();
            } else {
                self.last_poly_cents[s_idx] = 0.0;
            }
        }
    }

    pub fn detect_polyphonic_status(&self) -> Vec<PolyphonicStringStatus> {
        let strings = [
            (1, "E2"),
            (2, "A2"),
            (3, "D3"),
            (4, "G3"),
            (5, "B3"),
            (6, "E4"),
        ];

        strings
            .iter()
            .enumerate()
            .map(|(i, &(idx, note))| PolyphonicStringStatus {
                string_index: idx,
                note: note.to_string(),
                cents_deviation: self.last_poly_cents[i],
            })
            .collect()
    }

    pub fn process_audio(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        if !self.is_active {
            if self.mute_gain < 1.0 {
                let ramp_step = 1.0 / (0.005 * self.sample_rate);
                for i in 0..input.len() {
                    self.mute_gain = (self.mute_gain + ramp_step).min(1.0);
                    output[i] = input[i] * self.mute_gain;
                }
            } else {
                output.copy_from_slice(input);
            }
            return;
        }

        match self.output_mode {
            TunerOutputMode::Mute => {
                // 5 ms smooth ramp down to 0.0
                let ramp_step = 1.0 / (0.005 * self.sample_rate);
                for i in 0..input.len() {
                    self.mute_gain = (self.mute_gain - ramp_step).max(0.0);
                    output[i] = input[i] * self.mute_gain;
                }
            }
            TunerOutputMode::Bypass => {
                output.copy_from_slice(input);
            }
        }
    }
}
