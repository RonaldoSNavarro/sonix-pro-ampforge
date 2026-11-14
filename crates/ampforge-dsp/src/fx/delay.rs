//! crates/ampforge-dsp/src/fx/delay.rs
//! Processadores de Delay (Roland RE-201 Space Echo 12 Modos, BBD, Digital, Ducking e Spillover) (Spec 007, 024, 032)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayModel {
    MultiHeadTapeEcho,
    AnalogBbdDelay,
    PristineDigitalDelay,
    DualParallelDelay,
    ReverseDelay,
    PingPongStereo,
}

#[derive(Debug, Clone, Copy)]
struct WetBiquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Default for WetBiquad {
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        }
    }
}

impl WetBiquad {
    fn low_shelf(fc: f32, gain_db: f32, sr: f32) -> Self {
        if gain_db.abs() < 0.05 {
            return Self::default();
        }
        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * (fc / sr).clamp(0.0005, 0.45);
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / std::f32::consts::SQRT_2;
        let sq = 2.0 * a.sqrt() * alpha;
        let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + sq;
        Self {
            b0: (a * ((a + 1.0) - (a - 1.0) * cos_w0 + sq)) / a0,
            b1: (2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0)) / a0,
            b2: (a * ((a + 1.0) - (a - 1.0) * cos_w0 - sq)) / a0,
            a1: (-2.0 * ((a - 1.0) + (a + 1.0) * cos_w0)) / a0,
            a2: ((a + 1.0) + (a - 1.0) * cos_w0 - sq) / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    fn high_shelf(fc: f32, gain_db: f32, sr: f32) -> Self {
        if gain_db.abs() < 0.05 {
            return Self::default();
        }
        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * (fc / sr).clamp(0.0005, 0.45);
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / std::f32::consts::SQRT_2;
        let sq = 2.0 * a.sqrt() * alpha;
        let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + sq;
        Self {
            b0: (a * ((a + 1.0) + (a - 1.0) * cos_w0 + sq)) / a0,
            b1: (-2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0)) / a0,
            b2: (a * ((a + 1.0) + (a - 1.0) * cos_w0 - sq)) / a0,
            a1: (2.0 * ((a - 1.0) - (a + 1.0) * cos_w0)) / a0,
            a2: ((a + 1.0) - (a - 1.0) * cos_w0 - sq) / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    fn update_coeffs(&mut self, c: Self) {
        self.b0 = c.b0;
        self.b1 = c.b1;
        self.b2 = c.b2;
        self.a1 = c.a1;
        self.a2 = c.a2;
    }

    #[inline(always)]
    fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

const SPRING_LENS: [usize; 4] = [397, 463, 557, 641];

pub struct DelayProcessor {
    pub model: DelayModel,
    sample_rate: f32,
    delay_time_ms: f32,
    feedback_pct: f32,
    mix_pct: f32,
    tempo_bpm: f32,
    // Spec 032: Controles fiéis do RE-201 Space Echo (12 modos, mola interna, EQ somente no wet)
    re201_mode: u8,
    reverb_volume: f32,
    tone_bass: f32,
    tone_treble: f32,
    wet_bass_bq: WetBiquad,
    wet_treble_bq: WetBiquad,
    spring_combs: [[f32; 1024]; 4],
    spring_pos: [usize; 4],
    spring_lp: [f32; 4],
    wow_phase: f32,
    // Ducking
    ducking_enabled: bool,
    ducking_threshold_db: f32,
    ducking_attenuation_db: f32,
    ducking_release_ms: f32,
    ducking_gain_reduction_db: f32,
    ducking_envelope: f32,
    // Spillover & Bypass
    bypassed: bool,
    spillover_enabled: bool,
    // Pre-allocated circular buffer (up to 4000 ms at 96 kHz: 384,000 samples)
    buffer: Vec<f32>,
    write_pos: usize,
}

impl DelayProcessor {
    pub fn new(model: DelayModel) -> Self {
        let max_samples = (4.0 * 96000.0) as usize;
        let mut p = Self {
            model,
            sample_rate: 48000.0,
            delay_time_ms: 350.0,
            feedback_pct: 35.0,
            mix_pct: 40.0,
            tempo_bpm: 120.0,
            re201_mode: 1,
            reverb_volume: 0.0,
            tone_bass: 5.0,
            tone_treble: 5.0,
            wet_bass_bq: WetBiquad::default(),
            wet_treble_bq: WetBiquad::default(),
            spring_combs: [[0.0; 1024]; 4],
            spring_pos: [0; 4],
            spring_lp: [0.0; 4],
            wow_phase: 0.0,
            ducking_enabled: false,
            ducking_threshold_db: -30.0,
            ducking_attenuation_db: -12.0,
            ducking_release_ms: 150.0,
            ducking_gain_reduction_db: 0.0,
            ducking_envelope: 0.0,
            bypassed: false,
            spillover_enabled: false,
            buffer: vec![0.0; max_samples],
            write_pos: 0,
        };
        p.update_wet_eq();
        p
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.update_wet_eq();
    }

    pub fn tempo_bpm(&self) -> f32 {
        self.tempo_bpm
    }

    pub fn set_tempo_bpm(&mut self, bpm: f32) {
        self.tempo_bpm = bpm.clamp(20.0, 300.0);
    }

    pub fn delay_time_ms(&self) -> f32 {
        self.delay_time_ms
    }

    pub fn set_delay_time_ms(&mut self, ms: f32) {
        self.delay_time_ms = ms.clamp(1.0, 4000.0);
    }

    pub fn set_feedback_pct(&mut self, fb: f32) {
        self.feedback_pct = fb.clamp(0.0, 150.0);
    }

    pub fn set_mix_pct(&mut self, mix: f32) {
        self.mix_pct = mix.clamp(0.0, 100.0);
    }

    pub fn re201_mode(&self) -> u8 {
        self.re201_mode
    }

    pub fn set_re201_mode(&mut self, mode: u8) {
        self.re201_mode = mode.clamp(1, 12);
    }

    pub fn reverb_volume(&self) -> f32 {
        self.reverb_volume
    }

    pub fn set_reverb_volume(&mut self, vol: f32) {
        if vol.is_finite() {
            self.reverb_volume = vol.clamp(0.0, 10.0);
        }
    }

    pub fn set_tone_bass(&mut self, bass: f32) {
        if bass.is_finite() {
            self.tone_bass = bass.clamp(0.0, 10.0);
            self.update_wet_eq();
        }
    }

    pub fn set_tone_treble(&mut self, treble: f32) {
        if treble.is_finite() {
            self.tone_treble = treble.clamp(0.0, 10.0);
            self.update_wet_eq();
        }
    }

    fn update_wet_eq(&mut self) {
        let bass_db = (self.tone_bass - 5.0) * 2.4; // -12..+12 dB
        let treble_db = (self.tone_treble - 5.0) * 2.4; // -12..+12 dB
        self.wet_bass_bq
            .update_coeffs(WetBiquad::low_shelf(220.0, bass_db, self.sample_rate));
        self.wet_treble_bq
            .update_coeffs(WetBiquad::high_shelf(2200.0, treble_db, self.sample_rate));
    }

    pub fn set_ducking_enabled(&mut self, enabled: bool) {
        self.ducking_enabled = enabled;
    }

    pub fn set_ducking_threshold_db(&mut self, db: f32) {
        self.ducking_threshold_db = db.clamp(-60.0, 0.0);
    }

    pub fn set_ducking_attenuation_db(&mut self, db: f32) {
        self.ducking_attenuation_db = db.clamp(-30.0, 0.0);
    }

    pub fn ducking_gain_reduction_db(&self) -> f32 {
        self.ducking_gain_reduction_db
    }

    pub fn set_bypassed(&mut self, bypassed: bool) {
        self.bypassed = bypassed;
    }

    pub fn set_spillover_enabled(&mut self, enabled: bool) {
        self.spillover_enabled = enabled;
    }

    #[inline(always)]
    fn read_delay_tap(&self, delay_samples: f32) -> f32 {
        let buf_len = self.buffer.len();
        let d_clamped = delay_samples.clamp(1.0, (buf_len - 2) as f32);
        let d_int = d_clamped.floor() as usize;
        let d_frac = d_clamped - d_int as f32;
        let p0 = (self.write_pos + buf_len - d_int) % buf_len;
        let p1 = (p0 + buf_len - 1) % buf_len;
        self.buffer[p0] * (1.0 - d_frac) + self.buffer[p1] * d_frac
    }

    #[inline(always)]
    fn step_spring(&mut self, x: f32, fb_gain: f32) -> f32 {
        let mut sum = 0.0f32;
        for c in 0..4 {
            let len = SPRING_LENS[c];
            let pos = self.spring_pos[c];
            let y = self.spring_combs[c][pos];
            self.spring_lp[c] = 0.65 * y + 0.35 * self.spring_lp[c];
            self.spring_combs[c][pos] = (x + self.spring_lp[c] * fb_gain).clamp(-1.5, 1.5);
            self.spring_pos[c] = if pos + 1 >= len { 0 } else { pos + 1 };
            sum += y;
        }
        sum * 0.25
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());

        let wet_ratio = self.mix_pct / 100.0;
        let dry_ratio = 1.0 - wet_ratio;
        let base_delay_samples = (self.delay_time_ms * 0.001 * self.sample_rate).max(1.0);
        let buf_len = self.buffer.len();
        let feedback_gain = self.feedback_pct / 100.0;

        // Pesos das 3 cabeças e reverb interno por modo RE-201
        let (h1_w, h2_w, h3_w, spring_on, echo_on) = if self.model == DelayModel::MultiHeadTapeEcho {
            match self.re201_mode {
                1 => (1.0, 0.0, 0.0, false, true),
                2 => (0.0, 1.0, 0.0, false, true),
                3 => (0.0, 0.0, 1.0, false, true),
                4 => (0.0, 0.55, 0.55, false, true),
                5 => (0.55, 0.55, 0.0, true, true),
                6 => (0.55, 0.0, 0.55, true, true),
                7 => (0.0, 0.55, 0.55, true, true),
                8 => (0.4, 0.35, 0.35, true, true),
                9 => (0.6, 0.45, 0.0, true, true),
                10 => (0.0, 0.5, 0.5, true, true),
                11 => (0.45, 0.4, 0.35, true, true),
                12 => (0.0, 0.0, 0.0, true, false), // Modo 12 RE-201: Reverb Only
                _ => (1.0, 0.0, 0.0, false, true),
            }
        } else {
            (1.0, 0.0, 0.0, false, true)
        };

        let spring_mix = if spring_on {
            (self.reverb_volume / 10.0).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let wow_step = 2.0 * std::f32::consts::PI * 1.15 / self.sample_rate;
        let wow_depth_samples = if self.model == DelayModel::MultiHeadTapeEcho && self.re201_mode > 1 {
            0.35
        } else {
            0.0
        };

        let threshold_lin = 10.0f32.powf(self.ducking_threshold_db / 20.0);
        let max_duck_gain = 10.0f32.powf(self.ducking_attenuation_db / 20.0);
        let duck_rel_coeff = (-1.0 / (self.sample_rate * (self.ducking_release_ms * 0.001))).exp();

        for i in 0..input.len() {
            let in_sample = if self.bypassed && self.spillover_enabled {
                0.0
            } else if self.bypassed {
                output[i] = input[i];
                continue;
            } else {
                input[i]
            };

            if self.ducking_enabled {
                let abs_in = input[i].abs();
                if abs_in > self.ducking_envelope {
                    self.ducking_envelope = abs_in;
                } else {
                    self.ducking_envelope =
                        (1.0 - duck_rel_coeff) * abs_in + duck_rel_coeff * self.ducking_envelope;
                }

                if self.ducking_envelope > threshold_lin {
                    self.ducking_gain_reduction_db = self.ducking_attenuation_db;
                } else {
                    self.ducking_gain_reduction_db = 0.0;
                }
            } else {
                self.ducking_gain_reduction_db = 0.0;
            }

            let duck_mult = if self.ducking_enabled && self.ducking_envelope > threshold_lin {
                max_duck_gain
            } else {
                1.0
            };

            self.wow_phase += wow_step;
            if self.wow_phase >= 2.0 * std::f32::consts::PI {
                self.wow_phase -= 2.0 * std::f32::consts::PI;
            }
            let wow = self.wow_phase.sin() * wow_depth_samples;

            let delayed = if echo_on {
                let mut acc = 0.0f32;
                if h1_w > 0.0 {
                    acc += h1_w * self.read_delay_tap((base_delay_samples + wow).max(1.0));
                }
                if h2_w > 0.0 {
                    acc += h2_w * self.read_delay_tap((base_delay_samples * 1.92 + wow).max(1.0));
                }
                if h3_w > 0.0 {
                    acc += h3_w * self.read_delay_tap((base_delay_samples * 2.83 + wow).max(1.0));
                }
                acc
            } else {
                0.0
            };

            // EQ Bass & Treble atuam apenas no caminho úmido da fita
            let echo_eq = if echo_on {
                let b = self.wet_bass_bq.process(delayed);
                self.wet_treble_bq.process(b)
            } else {
                0.0
            };

            // Feedback de fita com saturação magnética
            let fb_signal = if echo_on {
                (echo_eq * feedback_gain).tanh()
            } else {
                0.0
            };
            self.buffer[self.write_pos] = in_sample + fb_signal;
            self.write_pos = (self.write_pos + 1) % buf_len;

            // Mola interna nos modos 5-12
            let spring_wet = if spring_mix > 0.0001 {
                self.step_spring(in_sample + echo_eq * 0.25, 0.78) * spring_mix * 1.6
            } else {
                0.0
            };

            let wet = (echo_eq + spring_wet) * duck_mult;
            if self.bypassed && self.spillover_enabled {
                output[i] = input[i] + wet;
            } else if !echo_on {
                // Mode 12: Direct signal + Spring Reverb (echo_volume silencioso para eco, reverb_volume comanda a mola)
                output[i] = input[i] * (1.0 - 0.35 * spring_mix) + spring_wet * duck_mult;
            } else {
                output[i] = input[i] * dry_ratio + wet * wet_ratio;
            }
        }
    }
}
