//! crates/ampforge-dsp/src/fx/modulation.rs
//! Processadores de Modulação (Chorus, Flanger, Phaser, Tremolo, Univibe, Rotary)

pub const DECISAO_SONIX_LDR_ATTACK_MS: f32 = 5.0;
pub const DECISAO_SONIX_LDR_RELEASE_MS: f32 = 60.0;
pub const DECISAO_SONIX_ROTARY_HORN_SLOW_HZ: f32 = 0.80;
pub const DECISAO_SONIX_ROTARY_HORN_FAST_HZ: f32 = 6.67;
pub const DECISAO_SONIX_ROTARY_DRUM_SLOW_HZ: f32 = 0.67;
pub const DECISAO_SONIX_ROTARY_DRUM_FAST_HZ: f32 = 5.70;
pub const DECISAO_SONIX_ROTARY_HORN_ACCEL_S: f32 = 0.85;
pub const DECISAO_SONIX_ROTARY_DRUM_ACCEL_S: f32 = 3.50;

use crate::biquad::BiquadTdf2F32 as ModBiquad;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModModel {
    AnalogBbdChorus,
    StereoTriChorus,
    ThroughZeroFlanger,
    Orange90Phaser,
    OpticalBiasTremolo,
    PhotocellUnivibe,
    RotarySpeaker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteDivision {
    Whole,
    Half,
    Quarter,
    QuarterTriplet,
    Eighth,
    EighthDotted,
    Sixteenth,
}

impl NoteDivision {
    pub fn quarter_note_multiplier(&self) -> f32 {
        match self {
            Self::Whole => 4.0,
            Self::Half => 2.0,
            Self::Quarter => 1.0,
            Self::QuarterTriplet => 2.0 / 3.0,
            Self::Eighth => 0.5,
            Self::EighthDotted => 0.75,
            Self::Sixteenth => 0.25,
        }
    }
}

pub struct ModulationProcessor {
    model: ModModel,
    rate_hz: f32,
    depth: f32,
    mix_pct: f32,
    feedback: f32,
    lfo_frozen: bool,
    tempo_sync: bool,
    note_division: NoteDivision,
    tempo_bpm: f32,
    sample_rate: f32,
    lfo_phase: f32,
    delay_buffer: Vec<f32>,
    write_pos: usize,
    ap_x1: [f32; 4],
    ap_y1: [f32; 4],
    flanger_fb_state: f32,
    tzf_inverted_polarity: bool,
    tzf_dry_buffer: Vec<f32>,
    tzf_dry_pos: usize,
    ldr_env: f32,
    bbd_lp_state: f32,
    rotary_fast: bool,
    rotary_horn_hz: f32,
    rotary_drum_hz: f32,
    rotary_horn_phase: f32,
    rotary_drum_phase: f32,
    rotary_lp1: ModBiquad,
    rotary_lp2: ModBiquad,
    rotary_hp1: ModBiquad,
    rotary_hp2: ModBiquad,
}

impl ModulationProcessor {
    pub fn reset_state(&mut self) {
        self.delay_buffer.fill(0.0);
        self.write_pos = 0;
        self.ap_x1 = [0.0; 4];
        self.ap_y1 = [0.0; 4];
        self.flanger_fb_state = 0.0;
        self.tzf_dry_buffer.fill(0.0);
        self.tzf_dry_pos = 0;
        self.ldr_env = 0.0;
        self.bbd_lp_state = 0.0;
        self.rotary_horn_phase = 0.0;
        self.rotary_drum_phase = 0.0;
        self.rotary_lp1.reset();
        self.rotary_lp2.reset();
        self.rotary_hp1.reset();
        self.rotary_hp2.reset();
    }

    pub fn new(model: ModModel) -> Self {
        let sr = 48000.0;
        Self {
            model,
            rate_hz: 1.5,
            depth: 5.0,
            mix_pct: 50.0,
            feedback: 0.70,
            lfo_frozen: false,
            tempo_sync: false,
            note_division: NoteDivision::Quarter,
            tempo_bpm: 120.0,
            sample_rate: sr,
            lfo_phase: if matches!(model, ModModel::ThroughZeroFlanger) {
                0.985
            } else {
                0.0
            },
            delay_buffer: vec![0.0f32; 8192],
            write_pos: 0,
            ap_x1: [0.0; 4],
            ap_y1: [0.0; 4],
            flanger_fb_state: 0.0,
            tzf_inverted_polarity: false,
            tzf_dry_buffer: vec![0.0f32; 512],
            tzf_dry_pos: 0,
            ldr_env: 0.0,
            bbd_lp_state: 0.0,
            rotary_fast: false,
            rotary_horn_hz: DECISAO_SONIX_ROTARY_HORN_SLOW_HZ,
            rotary_drum_hz: DECISAO_SONIX_ROTARY_DRUM_SLOW_HZ,
            rotary_horn_phase: 0.0,
            rotary_drum_phase: 0.0,
            rotary_lp1: ModBiquad::butterworth_lp(800.0, sr),
            rotary_lp2: ModBiquad::butterworth_lp(800.0, sr),
            rotary_hp1: ModBiquad::butterworth_hp(800.0, sr),
            rotary_hp2: ModBiquad::butterworth_hp(800.0, sr),
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.rotary_lp1 = ModBiquad::butterworth_lp(800.0, self.sample_rate);
        self.rotary_lp2 = ModBiquad::butterworth_lp(800.0, self.sample_rate);
        self.rotary_hp1 = ModBiquad::butterworth_hp(800.0, self.sample_rate);
        self.rotary_hp2 = ModBiquad::butterworth_hp(800.0, self.sample_rate);
    }

    pub fn set_tzf_inverted_polarity(&mut self, inverted: bool) {
        self.tzf_inverted_polarity = inverted;
    }

    pub fn set_rotary_fast(&mut self, fast: bool) {
        self.rotary_fast = fast;
    }

    pub fn rotary_horn_hz(&self) -> f32 {
        self.rotary_horn_hz
    }

    pub fn rotary_drum_hz(&self) -> f32 {
        self.rotary_drum_hz
    }

    pub fn rate_hz(&self) -> f32 {
        self.rate_hz
    }

    pub fn set_rate_hz(&mut self, rate: f32) {
        self.rate_hz = rate.clamp(0.05, 10.0);
    }

    pub fn depth(&self) -> f32 {
        self.depth
    }

    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 10.0);
    }

    pub fn model(&self) -> ModModel {
        self.model
    }

    pub fn set_model(&mut self, model: ModModel) {
        self.model = model;
    }

    pub fn mix_pct(&self) -> f32 {
        self.mix_pct
    }

    pub fn set_mix_pct(&mut self, mix: f32) {
        self.mix_pct = mix.clamp(0.0, 100.0);
    }

    pub fn set_tempo_sync(&mut self, sync: bool) {
        self.tempo_sync = sync;
    }

    pub fn set_tempo_bpm(&mut self, bpm: f32) {
        self.tempo_bpm = bpm.clamp(20.0, 300.0);
    }

    pub fn set_note_division(&mut self, div: NoteDivision) {
        self.note_division = div;
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        if feedback.is_finite() {
            self.feedback = feedback.clamp(-0.95, 0.95);
        }
    }

    pub fn set_lfo_frozen(&mut self, frozen: bool) {
        self.lfo_frozen = frozen;
    }

    pub fn calculated_lfo_freq_hz(&self) -> f32 {
        if self.tempo_sync {
            let beats_per_sec = self.tempo_bpm / 60.0;
            beats_per_sec / self.note_division.quarter_note_multiplier()
        } else {
            self.rate_hz
        }
    }

    #[inline(always)]
    fn read_delay_linear_allow_zero(&self, d_samples: f32) -> f32 {
        let buf_len = self.delay_buffer.len();
        let d_clamped = d_samples.clamp(0.0, (buf_len - 2) as f32);
        let read_pos = (self.write_pos as f32 + buf_len as f32 - d_clamped) % buf_len as f32;
        let idx0 = read_pos.floor() as usize % buf_len;
        let idx1 = (idx0 + 1) % buf_len;
        let frac = read_pos - read_pos.floor();
        self.delay_buffer[idx0] * (1.0 - frac) + self.delay_buffer[idx1] * frac
    }

    #[inline(always)]
    fn read_tzf_dry_linear(&self, d_samples: f32) -> f32 {
        let buf_len = self.tzf_dry_buffer.len();
        let d_clamped = d_samples.clamp(0.0, (buf_len - 2) as f32);
        let read_pos = (self.tzf_dry_pos as f32 + buf_len as f32 - d_clamped) % buf_len as f32;
        let idx0 = read_pos.floor() as usize % buf_len;
        let idx1 = (idx0 + 1) % buf_len;
        let frac = read_pos - read_pos.floor();
        self.tzf_dry_buffer[idx0] * (1.0 - frac) + self.tzf_dry_buffer[idx1] * frac
    }

    #[inline(always)]
    fn step_allpass4(&mut self, input: f32, freqs_hz: [f32; 4]) -> f32 {
        let mut s = input;
        let pi_over_sr = std::f32::consts::PI / self.sample_rate;
        for k in 0..4 {
            let w = (pi_over_sr * freqs_hz[k].clamp(20.0, self.sample_rate * 0.45)).tan();
            let a = (w - 1.0) / (w + 1.0);
            let y = crate::denormal::flush_f32(a * s + self.ap_x1[k] - a * self.ap_y1[k]);
            self.ap_x1[k] = crate::denormal::flush_f32(s);
            self.ap_y1[k] = y;
            s = y;
        }
        s
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        if self.mix_pct <= 0.0 {
            output.copy_from_slice(input);
            return;
        }

        let wet_ratio = self.mix_pct / 100.0;
        let dry_ratio = 1.0 - wet_ratio;
        let depth_norm = self.depth / 10.0;
        let lfo_freq = self.calculated_lfo_freq_hz();
        let two_pi = 2.0 * std::f32::consts::PI;
        let phase_inc = if self.lfo_frozen {
            0.0
        } else {
            two_pi * lfo_freq / self.sample_rate
        };
        let buf_len = self.delay_buffer.len();
        let dry_len = self.tzf_dry_buffer.len();
        let bbd_lp_alpha = 1.0 - (-two_pi * 4500.0 / self.sample_rate).exp();
        let alpha_ldr_att = 1.0 - (-1.0 / ((DECISAO_SONIX_LDR_ATTACK_MS * 0.001) * self.sample_rate)).exp();
        let alpha_ldr_rel = 1.0 - (-1.0 / ((DECISAO_SONIX_LDR_RELEASE_MS * 0.001) * self.sample_rate)).exp();
        let alpha_horn_acc = 1.0 - (-1.0 / (DECISAO_SONIX_ROTARY_HORN_ACCEL_S * self.sample_rate)).exp();
        let alpha_drum_acc = 1.0 - (-1.0 / (DECISAO_SONIX_ROTARY_DRUM_ACCEL_S * self.sample_rate)).exp();
        let target_horn_hz = if self.rotary_fast {
            DECISAO_SONIX_ROTARY_HORN_FAST_HZ
        } else {
            DECISAO_SONIX_ROTARY_HORN_SLOW_HZ
        };
        let target_drum_hz = if self.rotary_fast {
            DECISAO_SONIX_ROTARY_DRUM_FAST_HZ
        } else {
            DECISAO_SONIX_ROTARY_DRUM_SLOW_HZ
        };

        for i in 0..input.len() {
            let x = input[i];
            let lfo_sin = self.lfo_phase.sin();
            let mut dry_for_mix = x;

            let wet = match self.model {
                ModModel::OpticalBiasTremolo => {
                    // Pure optical/bias amplitude modulation with asymmetric LDR follower (5 ms attack / 60 ms release)
                    let lfo_target = (0.5 + 0.5 * lfo_sin).powi(2);
                    let alpha = if lfo_target > self.ldr_env {
                        alpha_ldr_att
                    } else {
                        alpha_ldr_rel
                    };
                    self.ldr_env = crate::denormal::flush_f32(
                        self.ldr_env + alpha * (lfo_target - self.ldr_env),
                    );
                    let gain = 1.0 - depth_norm * 0.78 * self.ldr_env;
                    x * gain
                }
                ModModel::ThroughZeroFlanger => {
                    // Dry delayed by 1.0 ms; wet sweeps 0..2 ms crossing 1.0 ms with optional inverted polarity
                    let d_dry_samples = 0.001 * self.sample_rate;
                    self.tzf_dry_buffer[self.tzf_dry_pos] = crate::denormal::flush_f32(x);
                    dry_for_mix = self.read_tzf_dry_linear(d_dry_samples);
                    self.tzf_dry_pos = (self.tzf_dry_pos + 1) % dry_len;

                    let center_ms = 1.0;
                    let mod_ms = 1.0 * depth_norm * lfo_sin;
                    let d_samples = ((center_ms + mod_ms) * 0.001 * self.sample_rate)
                        .clamp(0.0, (buf_len - 2) as f32);
                    self.delay_buffer[self.write_pos] = crate::denormal::flush_f32(
                        (x + self.flanger_fb_state * self.feedback).clamp(-4.0, 4.0),
                    );
                    let delayed = self.read_delay_linear_allow_zero(d_samples);
                    self.flanger_fb_state = crate::denormal::flush_f32(delayed);
                    self.write_pos = (self.write_pos + 1) % buf_len;
                    if self.tzf_inverted_polarity {
                        -delayed
                    } else {
                        delayed
                    }
                }
                ModModel::Orange90Phaser => {
                    // 4-stage 1st-order all-pass cascade -> N/2 = 2 notches
                    let f_min = 350.0f32;
                    let f_max = 3200.0f32;
                    let norm = (0.5 + 0.5 * lfo_sin * depth_norm).clamp(0.0, 1.0);
                    let f_c = f_min * (f_max / f_min).powf(norm);
                    let x_in = (x + self.ap_y1[3] * (self.feedback * 0.5)).clamp(-4.0, 4.0);
                    self.step_allpass4(x_in, [f_c, f_c, f_c, f_c])
                }
                ModModel::PhotocellUnivibe => {
                    // Asymmetric non-sinusoidal photocell LFO driving 4 staggered all-pass stages
                    let u = (0.5 - 0.5 * self.lfo_phase.cos()).clamp(0.0, 1.0).powf(2.2);
                    let base = 180.0 * (1800.0f32 / 180.0).powf((u * depth_norm).clamp(0.0, 1.0));
                    let x_in = (x + self.ap_y1[3] * (self.feedback * 0.35)).clamp(-4.0, 4.0);
                    let ap_out =
                        self.step_allpass4(x_in, [base, base * 1.41, base * 2.0, base * 2.83]);
                    let am = 1.0 - 0.12 * depth_norm * u;
                    ap_out * am
                }
                ModModel::AnalogBbdChorus => {
                    // Single modulated voice + BBD 4.5 kHz HF rolloff
                    self.delay_buffer[self.write_pos] = crate::denormal::flush_f32(x);
                    let base_d = 0.008 * self.sample_rate;
                    let mod_d = depth_norm * 0.0035 * self.sample_rate * lfo_sin;
                    let delayed = self.read_delay_linear_allow_zero((base_d + mod_d).max(1.0));
                    self.write_pos = (self.write_pos + 1) % buf_len;
                    self.bbd_lp_state = crate::denormal::flush_f32(
                        self.bbd_lp_state + bbd_lp_alpha * (delayed - self.bbd_lp_state),
                    );
                    self.bbd_lp_state
                }
                ModModel::StereoTriChorus => {
                    // 3 voices offset by 0, 120, 240 deg summed (mono)
                    self.delay_buffer[self.write_pos] = crate::denormal::flush_f32(x);
                    let base_d = 0.008 * self.sample_rate;
                    let mod_amp = depth_norm * 0.0035 * self.sample_rate;
                    let p0 = self.lfo_phase.sin();
                    let p1 = (self.lfo_phase + two_pi / 3.0).sin();
                    let p2 = (self.lfo_phase + 2.0 * two_pi / 3.0).sin();
                    let v0 = self.read_delay_linear_allow_zero((base_d + mod_amp * p0).max(1.0));
                    let v1 = self.read_delay_linear_allow_zero((base_d + mod_amp * p1).max(1.0));
                    let v2 = self.read_delay_linear_allow_zero((base_d + mod_amp * p2).max(1.0));
                    self.write_pos = (self.write_pos + 1) % buf_len;
                    (v0 + v1 + v2) * (1.0 / 3.0)
                }
                ModModel::RotarySpeaker => {
                    // LR4 800 Hz crossover -> dual rotors (horn AM + Doppler, drum AM)
                    let low_band = self.rotary_lp2.process(self.rotary_lp1.process(x));
                    let high_band = self.rotary_hp2.process(self.rotary_hp1.process(x));

                    self.rotary_horn_hz += alpha_horn_acc * (target_horn_hz - self.rotary_horn_hz);
                    self.rotary_drum_hz += alpha_drum_acc * (target_drum_hz - self.rotary_drum_hz);

                    if !self.lfo_frozen {
                        self.rotary_horn_phase += two_pi * self.rotary_horn_hz / self.sample_rate;
                        if self.rotary_horn_phase >= two_pi {
                            self.rotary_horn_phase -= two_pi;
                        }
                        self.rotary_drum_phase += two_pi * self.rotary_drum_hz / self.sample_rate;
                        if self.rotary_drum_phase >= two_pi {
                            self.rotary_drum_phase -= two_pi;
                        }
                    }

                    self.delay_buffer[self.write_pos] = crate::denormal::flush_f32(high_band);
                    let base_d = 0.002 * self.sample_rate;
                    let mod_d = depth_norm * 0.0008 * self.sample_rate * self.rotary_horn_phase.sin();
                    let horn_doppler = self.read_delay_linear_allow_zero((base_d + mod_d).max(1.0));
                    self.write_pos = (self.write_pos + 1) % buf_len;

                    let horn_am = 1.0 - 0.55 * depth_norm * (0.5 - 0.5 * self.rotary_horn_phase.cos());
                    let drum_am = 1.0 - 0.50 * depth_norm * (0.5 - 0.5 * self.rotary_drum_phase.cos());
                    horn_doppler * horn_am + low_band * drum_am
                }
            };

            self.lfo_phase += phase_inc;
            if self.lfo_phase >= two_pi {
                self.lfo_phase -= two_pi;
            }

            output[i] = crate::denormal::flush_f32(dry_for_mix * dry_ratio + wet * wet_ratio);
        }
    }
}
