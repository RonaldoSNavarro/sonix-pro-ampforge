//! crates/ampforge-dsp/src/fx/pitch.rs
//! Processadores de Pitch (Whammy, Drop Capo Polifônico, Harmonizer Diatônico, Micro Detune)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitchModel {
    PedalWhammy,
    PolyphonicDropCapo,
    IntelligentDualHarmonizer,
    MicroPitchDetune,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitchKey {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleType {
    Major,
    NaturalMinor,
    HarmonicMinor,
    MelodicMinor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
}

const PITCH_BUF_LEN: usize = 16384;
const PITCH_BUF_MASK: usize = PITCH_BUF_LEN - 1;
const ANALYSIS_BUF_LEN: usize = 2048;

pub struct PitchProcessor {
    model: PitchModel,
    semitones: f32,
    key: PitchKey,
    scale: ScaleType,
    mix_pct: f32,
    sample_rate: f32,
    bypassed: bool,
    buf: Vec<f32>,
    write_pos: usize,
    phase: f32,
    d0: f32,
    d1: f32,
    grains_initialized: bool,
    analysis_buf: Vec<f32>,
    analysis_pos: usize,
    samples_since_detect: usize,
    detected_freq_hz: f32,
}

impl PitchProcessor {
    pub fn new(model: PitchModel) -> Self {
        Self {
            model,
            semitones: 0.0,
            key: PitchKey::C,
            scale: ScaleType::Major,
            mix_pct: 100.0,
            sample_rate: 48000.0,
            bypassed: false,
            buf: vec![0.0; PITCH_BUF_LEN],
            write_pos: 0,
            phase: 0.0,
            d0: 1024.0,
            d1: 1536.0,
            grains_initialized: false,
            analysis_buf: vec![0.0; ANALYSIS_BUF_LEN],
            analysis_pos: 0,
            samples_since_detect: 0,
            detected_freq_hz: 261.63,
        }
    }

    pub fn model(&self) -> PitchModel {
        self.model
    }

    pub fn set_model(&mut self, model: PitchModel) {
        self.model = model;
    }

    pub fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    pub fn is_bypassed(&self) -> bool {
        self.bypassed
    }

    pub fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.buf.fill(0.0);
        self.write_pos = 0;
        self.phase = 0.0;
        let sr_scale = (self.sample_rate / 48000.0).clamp(0.25, 4.0);
        self.d0 = 1024.0 * sr_scale;
        self.d1 = 1536.0 * sr_scale;
        self.grains_initialized = false;
        self.analysis_buf.fill(0.0);
        self.analysis_pos = 0;
        self.samples_since_detect = 0;
        self.detected_freq_hz = 261.63;
    }

    pub fn algorithmic_latency_samples(&self) -> usize {
        if self.bypassed || self.mix_pct <= 0.0 {
            0
        } else {
            (1024.0 * (self.sample_rate / 48000.0).clamp(0.25, 4.0)).round() as usize
        }
    }

    pub fn set_mix_pct(&mut self, mix: f32) {
        self.mix_pct = mix.clamp(0.0, 100.0);
    }

    pub fn mix_pct(&self) -> f32 {
        self.mix_pct
    }

    pub fn set_semitones(&mut self, st: f32) {
        self.semitones = st.clamp(-24.0, 24.0);
    }

    pub fn semitones(&self) -> f32 {
        self.semitones
    }

    pub fn transposition_ratio(&self) -> f32 {
        2.0f32.powf(self.semitones / 12.0)
    }

    pub fn set_key(&mut self, key: PitchKey) {
        self.key = key;
    }

    pub fn key(&self) -> PitchKey {
        self.key
    }

    pub fn set_scale(&mut self, scale: ScaleType) {
        self.scale = scale;
    }

    pub fn scale(&self) -> ScaleType {
        self.scale
    }

    fn key_root_pc(key: PitchKey) -> i32 {
        match key {
            PitchKey::C => 0,
            PitchKey::CSharp => 1,
            PitchKey::D => 2,
            PitchKey::DSharp => 3,
            PitchKey::E => 4,
            PitchKey::F => 5,
            PitchKey::FSharp => 6,
            PitchKey::G => 7,
            PitchKey::GSharp => 8,
            PitchKey::A => 9,
            PitchKey::ASharp => 10,
            PitchKey::B => 11,
        }
    }

    fn scale_intervals(scale: ScaleType) -> [i32; 7] {
        match scale {
            ScaleType::Major => [0, 2, 4, 5, 7, 9, 11],
            ScaleType::NaturalMinor => [0, 2, 3, 5, 7, 8, 10],
            ScaleType::HarmonicMinor => [0, 2, 3, 5, 7, 8, 11],
            ScaleType::MelodicMinor => [0, 2, 3, 5, 7, 9, 11],
            ScaleType::Dorian => [0, 2, 3, 5, 7, 9, 10],
            ScaleType::Phrygian => [0, 1, 3, 5, 7, 8, 10],
            ScaleType::Lydian => [0, 2, 4, 6, 7, 9, 11],
            ScaleType::Mixolydian => [0, 2, 4, 5, 7, 9, 10],
            ScaleType::Locrian => [0, 1, 3, 5, 6, 8, 10],
        }
    }

    pub fn calculate_harmonic_frequency(&self, input_freq_hz: f32) -> f32 {
        if input_freq_hz <= 0.0 || !input_freq_hz.is_finite() {
            return 0.0;
        }
        let midi = 69.0 + 12.0 * (input_freq_hz / 440.0).log2();
        let midi_round = midi.round() as i32;
        let pc = midi_round.rem_euclid(12);
        let root = Self::key_root_pc(self.key);
        let rel = (pc - root).rem_euclid(12);
        let intervals = Self::scale_intervals(self.scale);

        let mut best_idx = 0usize;
        let mut best_dist = i32::MAX;
        for (i, &deg) in intervals.iter().enumerate() {
            let d = (deg - rel).abs();
            if d < best_dist {
                best_dist = d;
                best_idx = i;
            }
        }

        let target_deg = best_idx + 2;
        let octave_wrap = (target_deg / 7) as i32 * 12;
        let target_pc = intervals[target_deg % 7] + octave_wrap;
        let interval_st = (target_pc - intervals[best_idx]) as f32;
        input_freq_hz * 2.0f32.powf(interval_st / 12.0)
    }

    #[inline]
    fn read_linear(&self, delay_samples: f32) -> f32 {
        let clamped = delay_samples.clamp(0.0, (PITCH_BUF_LEN - 2) as f32);
        let idx0_offset = clamped.floor() as usize;
        let frac = clamped - idx0_offset as f32;
        let p0 = (self.write_pos + PITCH_BUF_LEN - idx0_offset) & PITCH_BUF_MASK;
        let p1 = (p0 + PITCH_BUF_LEN - 1) & PITCH_BUF_MASK;
        self.buf[p0] * (1.0 - frac) + self.buf[p1] * frac
    }

    fn detect_fundamental_hz(&self) -> Option<f32> {
        let n = ANALYSIS_BUF_LEN;
        let min_lag = ((self.sample_rate / 1200.0).floor() as usize).max(8);
        let max_lag = ((self.sample_rate / 70.0).ceil() as usize).min(n / 2 - 2);
        if min_lag >= max_lag {
            return None;
        }

        let mut energy = 0.0f32;
        for &s in &self.analysis_buf {
            energy += s * s;
        }
        if energy < 1e-5 {
            return None;
        }

        let mut best_lag = 0usize;
        let mut best_corr = 0.0f32;
        let window = n / 2;

        for lag in min_lag..=max_lag {
            let mut sum = 0.0f32;
            let mut e0 = 0.0f32;
            let mut e1 = 0.0f32;
            for i in 0..window {
                let idx0 = (self.analysis_pos + i) & (n - 1);
                let idx1 = (self.analysis_pos + i + lag) & (n - 1);
                let a = self.analysis_buf[idx0];
                let b = self.analysis_buf[idx1];
                sum += a * b;
                e0 += a * a;
                e1 += b * b;
            }
            let denom = (e0 * e1).sqrt();
            if denom > 1e-9 {
                let corr = sum / denom;
                if corr > best_corr {
                    best_corr = corr;
                    best_lag = lag;
                }
            }
        }

        if best_lag > 0 && best_corr > 0.55 {
            Some(self.sample_rate / best_lag as f32)
        } else {
            None
        }
    }

    fn find_aligned_delay(&self, active_delay: f32, reset_base: f32, search_len: usize) -> f32 {
        let active_idx = active_delay.round() as isize;
        let base_idx = reset_base.round() as isize;
        let mut best_offset = 0isize;
        let mut best_corr = f32::NEG_INFINITY;
        let win = 48isize;

        for off in (0..=search_len as isize).step_by(2) {
            let cand = base_idx + off;
            let mut dot = 0.0f32;
            let mut e_cand = 0.0f32;
            for k in (0..win).step_by(2) {
                let p_a = (self.write_pos as isize + PITCH_BUF_LEN as isize - (active_idx + k))
                    as usize
                    & PITCH_BUF_MASK;
                let p_c = (self.write_pos as isize + PITCH_BUF_LEN as isize - (cand + k)) as usize
                    & PITCH_BUF_MASK;
                let a = self.buf[p_a];
                let c = self.buf[p_c];
                dot += a * c;
                e_cand += c * c;
            }
            let score = if e_cand > 1e-9 {
                dot / e_cand.sqrt()
            } else {
                dot
            };
            if score > best_corr {
                best_corr = score;
                best_offset = off;
            }
        }
        (base_idx + best_offset) as f32
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let len = input.len().min(output.len());
        if self.bypassed || self.mix_pct <= 0.0 {
            output[..len].copy_from_slice(&input[..len]);
            return;
        }

        let sr_scale = (self.sample_rate / 48000.0).clamp(0.25, 4.0);
        let center_delay = 1024.0 * sr_scale;
        let grain_len = 1024.0 * sr_scale;
        let search_len = (700.0 * sr_scale) as usize;
        let mix = (self.mix_pct / 100.0).clamp(0.0, 1.0);

        let base_ratio = match self.model {
            PitchModel::MicroPitchDetune => {
                let st = if self.semitones.abs() < 0.01 {
                    0.15
                } else {
                    self.semitones
                };
                2.0f32.powf(st / 12.0)
            }
            PitchModel::PedalWhammy
            | PitchModel::PolyphonicDropCapo
            | PitchModel::IntelligentDualHarmonizer => 2.0f32.powf(self.semitones / 12.0),
        };

        for i in 0..len {
            let x = input[i];
            self.write_pos = (self.write_pos + 1) & PITCH_BUF_MASK;
            self.buf[self.write_pos] = crate::denormal::flush_f32(x);

            if self.model == PitchModel::IntelligentDualHarmonizer {
                self.analysis_buf[self.analysis_pos] = crate::denormal::flush_f32(x);
                self.analysis_pos = (self.analysis_pos + 1) & (ANALYSIS_BUF_LEN - 1);
                self.samples_since_detect += 1;
                if self.samples_since_detect >= 512 {
                    self.samples_since_detect = 0;
                    if let Some(f0) = self.detect_fundamental_hz() {
                        self.detected_freq_hz = f0;
                    }
                }
            }

            let ratio = if self.model == PitchModel::IntelligentDualHarmonizer {
                let harm = self.calculate_harmonic_frequency(self.detected_freq_hz);
                (harm / self.detected_freq_hz.max(1.0)) * base_ratio
            } else {
                base_ratio
            };

            let wet = if (ratio - 1.0).abs() < 1e-5 {
                self.read_linear(center_delay)
            } else {
                let reset_base = if ratio > 1.0 {
                    1600.0 * sr_scale
                } else {
                    512.0 * sr_scale
                };
                if !self.grains_initialized {
                    self.d0 = reset_base;
                    self.d1 = reset_base;
                    self.grains_initialized = true;
                }

                let prev_p0 = self.phase;
                let prev_p1 = (prev_p0 + 0.5).fract();
                let inc = (ratio - 1.0).abs() / grain_len;
                let next_p0 = (prev_p0 + inc).fract();
                let next_p1 = (next_p0 + 0.5).fract();

                // Delay drift per sample: d_{n+1} = d_n - (ratio - 1.0)
                let delta_d = ratio - 1.0;
                self.d0 -= delta_d;
                self.d1 -= delta_d;

                if next_p0 < prev_p0 {
                    // Grain 0 just wrapped (w0 == 0): phase-lock d0 to active grain 1
                    self.d0 = self.find_aligned_delay(self.d1, reset_base, search_len);
                }
                if next_p1 < prev_p1 {
                    // Grain 1 just wrapped (w1 == 0): phase-lock d1 to active grain 0
                    self.d1 = self.find_aligned_delay(self.d0, reset_base, search_len);
                }

                self.phase = next_p0;
                let p0 = next_p0;
                let p1 = next_p1;
                let w0 = 0.5 * (1.0 - (std::f32::consts::TAU * p0).cos());
                let w1 = 0.5 * (1.0 - (std::f32::consts::TAU * p1).cos());
                let y0 = self.read_linear(self.d0);
                let y1 = self.read_linear(self.d1);
                y0 * w0 + y1 * w1
            };

            let dry = self.read_linear(center_delay);
            output[i] = dry * (1.0 - mix) + wet * mix;
        }
    }
}
