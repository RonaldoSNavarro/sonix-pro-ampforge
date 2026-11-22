//! crates/ampforge-dsp/src/fx/delay.rs
//! Processadores de Delay (Roland RE-201 Space Echo 12 Modos, BBD, Digital, Ducking e Spillover) (Spec 007, 024, 032)

use super::reverb::{ReverbModel, ReverbProcessor};

pub const DECISAO_SONIX_RE201_HEAD2_RATIO: f32 = 1.92;
pub const DECISAO_SONIX_RE201_HEAD3_RATIO: f32 = 2.83;
pub const DECISAO_SONIX_RE201_ORDEM_PARES: [(bool, bool, bool); 3] = [
    (true, true, false),  // Modo 8: H1+H2
    (false, true, true),  // Modo 9: H2+H3
    (true, false, true),  // Modo 10: H1+H3
];
pub const DECISAO_SONIX_RE201_WOW_CENTS: f32 = 3.0;
pub const DECISAO_SONIX_RE201_WOW_HZ: f32 = 1.15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayModel {
    MultiHeadTapeEcho,
    AnalogBbdDelay,
    PristineDigitalDelay,
    DualParallelDelay,
    ReverseDelay,
    PingPongStereo,
}

use crate::biquad::BiquadTdf2F32 as WetBiquad;

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
    spring_reverb: ReverbProcessor,
    wow_phase: f32,
    wow_peak: f32,
    wow_gate: f32,
    current_delay_samples: f32,
    bbd_fb_lp: f32,
    // Ducking
    ducking_enabled: bool,
    ducking_threshold_db: f32,
    ducking_attenuation_db: f32,
    ducking_release_ms: f32,
    ducking_gain_reduction_db: f32,
    ducking_envelope: f32,
    duck_mult: f32,
    // Spillover & Bypass
    bypassed: bool,
    spillover_enabled: bool,
    // Pre-allocated circular buffer (up to 4000 ms at 96 kHz: 384,000 samples)
    buffer: Vec<f32>,
    write_pos: usize,
    // Pre-allocated reverse grain buffer
    rev_buffer: Vec<f32>,
    rev_write_pos: usize,
    rev_grain_ready: bool,
    // Contador O(1) para invalidar caudas sem memset no callback RT
    samples_since_clear: usize,
}

impl DelayProcessor {
    pub fn new(model: DelayModel) -> Self {
        let max_samples = (4.0 * 96000.0) as usize;
        let mut spring_reverb = ReverbProcessor::new(ReverbModel::TubeSpringReverb);
        spring_reverb.set_predelay_ms(0.0);
        spring_reverb.set_decay_time_s(2.5);
        spring_reverb.set_damping(3.5);
        spring_reverb.set_mix_pct(100.0);
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
            spring_reverb,
            wow_phase: 0.0,
            wow_peak: 0.0,
            wow_gate: 0.0,
            current_delay_samples: -1.0,
            bbd_fb_lp: 0.0,
            ducking_enabled: false,
            ducking_threshold_db: -30.0,
            ducking_attenuation_db: -12.0,
            ducking_release_ms: 150.0,
            ducking_gain_reduction_db: 0.0,
            ducking_envelope: 0.0,
            duck_mult: 1.0,
            bypassed: false,
            spillover_enabled: false,
            buffer: vec![0.0; max_samples],
            write_pos: 0,
            rev_buffer: vec![0.0; max_samples],
            rev_write_pos: 0,
            rev_grain_ready: false,
            samples_since_clear: usize::MAX,
        };
        p.update_wet_eq();
        p
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.spring_reverb.set_sample_rate(self.sample_rate);
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

    pub fn reset_state(&mut self) {
        self.clear_tail();
        self.write_pos = 0;
        self.ducking_envelope = 0.0;
        self.wow_phase = 0.0;
        self.wow_peak = 0.0;
        self.wow_gate = 0.0;
        self.wet_bass_bq.z1 = 0.0;
        self.wet_bass_bq.z2 = 0.0;
        self.wet_treble_bq.z1 = 0.0;
        self.wet_treble_bq.z2 = 0.0;
    }

    pub fn set_spillover_enabled(&mut self, enabled: bool) {
        self.spillover_enabled = enabled;
    }

    pub fn is_spillover_enabled(&self) -> bool {
        self.spillover_enabled
    }

    pub fn clear_tail(&mut self) {
        // O(1): evita memset de ~3 MB no callback de áudio
        self.samples_since_clear = 0;
        self.spring_reverb.clear_tail();
        self.rev_write_pos = 0;
        self.rev_grain_ready = false;
        self.bbd_fb_lp = 0.0;
    }

    #[inline(always)]
    fn read_delay_tap(&self, delay_samples: f32) -> f32 {
        let buf_len = self.buffer.len();
        let d_clamped = delay_samples.clamp(1.0, (buf_len - 2) as f32);
        let d_int = d_clamped.floor() as usize;
        let d_frac = d_clamped - d_int as f32;
        let s0 = if d_int > self.samples_since_clear {
            0.0
        } else {
            let p0 = (self.write_pos + buf_len - d_int) % buf_len;
            self.buffer[p0]
        };
        let s1 = if d_int + 1 > self.samples_since_clear {
            0.0
        } else {
            let p1 = (self.write_pos + buf_len - d_int - 1) % buf_len;
            self.buffer[p1]
        };
        s0 * (1.0 - d_frac) + s1 * d_frac
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());

        let wet_ratio = self.mix_pct / 100.0;
        let dry_ratio = 1.0 - wet_ratio;
        let target_delay_samples = (self.delay_time_ms * 0.001 * self.sample_rate).max(1.0);
        if self.current_delay_samples < 0.0 {
            self.current_delay_samples = target_delay_samples;
        }
        let glide_alpha = 1.0 - (-1.0 / (self.sample_rate * 0.030)).exp();
        let bbd_lp_alpha = 1.0 - (-2.0 * std::f32::consts::PI * 2400.0 / self.sample_rate).exp();
        let buf_len = self.buffer.len();
        let feedback_gain = self.feedback_pct / 100.0;

        // Tabela única dos 12 modos RE-201 com ganho 1/√n para cabeças ativas (AUD-28):
        // 1..4 só eco; 5..7 cabeça única + mola; 8..10 pares + mola; 11 todas + mola; 12 só mola.
        let (h1_on, h2_on, h3_on, spring_on, echo_on) =
            if self.model == DelayModel::MultiHeadTapeEcho {
                match self.re201_mode {
                    1 => (true, false, false, false, true),
                    2 => (false, true, false, false, true),
                    3 => (false, false, true, false, true),
                    4 => (false, true, true, false, true),
                    5 => (true, false, false, true, true),
                    6 => (false, true, false, true, true),
                    7 => (false, false, true, true, true),
                    8 => {
                        let (a, b, c) = DECISAO_SONIX_RE201_ORDEM_PARES[0];
                        (a, b, c, true, true)
                    }
                    9 => {
                        let (a, b, c) = DECISAO_SONIX_RE201_ORDEM_PARES[1];
                        (a, b, c, true, true)
                    }
                    10 => {
                        let (a, b, c) = DECISAO_SONIX_RE201_ORDEM_PARES[2];
                        (a, b, c, true, true)
                    }
                    11 => (true, true, true, true, true),
                    12 => (false, false, false, true, false), // Modo 12 RE-201: Reverb Only
                    _ => (true, false, false, false, true),
                }
            } else {
                (true, false, false, false, true)
            };
        let n_active = (h1_on as u8 + h2_on as u8 + h3_on as u8) as f32;
        let head_w = if n_active > 0.0 {
            1.0 / n_active.sqrt()
        } else {
            0.0
        };

        let spring_mix = if spring_on {
            (self.reverb_volume / 10.0).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let wow_step = 2.0 * std::f32::consts::PI * DECISAO_SONIX_RE201_WOW_HZ / self.sample_rate;
        let wow_depth_samples = if self.model == DelayModel::MultiHeadTapeEcho && echo_on {
            let r_peak = 2.0f32.powf(DECISAO_SONIX_RE201_WOW_CENTS / 1200.0) - 1.0;
            r_peak * self.sample_rate
                / (2.0 * std::f32::consts::PI * DECISAO_SONIX_RE201_WOW_HZ)
        } else {
            0.0
        };
        let wow_peak_rel = (-1.0 / (self.sample_rate * 0.200)).exp();
        let wow_gate_alpha = 1.0 - (-1.0 / (self.sample_rate * 0.002)).exp();

        let threshold_lin = 10.0f32.powf(self.ducking_threshold_db / 20.0);
        let max_duck_gain = 10.0f32.powf(self.ducking_attenuation_db / 20.0);
        let duck_rel_coeff = (-1.0 / (self.sample_rate * (self.ducking_release_ms * 0.001))).exp();
        let duck_att_alpha = 1.0 - (-1.0 / (self.sample_rate * 0.015)).exp();
        let duck_rec_alpha = 1.0 - duck_rel_coeff;

        for i in 0..input.len() {
            let in_sample = if self.bypassed && self.spillover_enabled {
                0.0
            } else if self.bypassed {
                output[i] = input[i];
                continue;
            } else {
                input[i]
            };

            self.current_delay_samples +=
                glide_alpha * (target_delay_samples - self.current_delay_samples);

            if self.ducking_enabled {
                let abs_in = input[i].abs();
                if abs_in > self.ducking_envelope {
                    self.ducking_envelope = crate::denormal::flush_f32(abs_in);
                } else {
                    self.ducking_envelope = crate::denormal::flush_f32(
                        (1.0 - duck_rel_coeff) * abs_in + duck_rel_coeff * self.ducking_envelope,
                    );
                }

                if self.ducking_envelope > threshold_lin {
                    self.ducking_gain_reduction_db = self.ducking_attenuation_db;
                } else {
                    self.ducking_gain_reduction_db = 0.0;
                }
            } else {
                self.ducking_gain_reduction_db = 0.0;
            }

            let target_duck_mult = if self.ducking_enabled && self.ducking_envelope > threshold_lin {
                max_duck_gain
            } else {
                1.0
            };
            let d_alpha = if target_duck_mult < self.duck_mult {
                duck_att_alpha
            } else {
                duck_rec_alpha
            };
            self.duck_mult += d_alpha * (target_duck_mult - self.duck_mult);

            let abs_raw = in_sample.abs();
            if abs_raw > self.wow_peak {
                self.wow_peak = crate::denormal::flush_f32(abs_raw);
            } else {
                self.wow_peak = crate::denormal::flush_f32(self.wow_peak * wow_peak_rel);
            }
            let active_flag = if abs_raw > self.wow_peak * 1e-4 && abs_raw > 1e-9 {
                1.0f32
            } else {
                0.0f32
            };
            self.wow_gate = crate::denormal::flush_f32(
                self.wow_gate + wow_gate_alpha * (active_flag - self.wow_gate),
            );
            let wow_scale = ((self.wow_gate - 0.15) / 0.35).clamp(0.0, 1.0);

            self.wow_phase += wow_step;
            if self.wow_phase >= 2.0 * std::f32::consts::PI {
                self.wow_phase -= 2.0 * std::f32::consts::PI;
            }
            let wow = self.wow_phase.sin() * wow_depth_samples * wow_scale;

            let mut echo_eq = 0.0f32;
            if echo_on {
                match self.model {
                    DelayModel::MultiHeadTapeEcho => {
                        let mut acc = 0.0f32;
                        if h1_on {
                            acc += head_w
                                * self.read_delay_tap((self.current_delay_samples + wow).max(1.0));
                        }
                        if h2_on {
                            acc += head_w
                                * self.read_delay_tap(
                                    (self.current_delay_samples
                                        * DECISAO_SONIX_RE201_HEAD2_RATIO
                                        + wow)
                                        .max(1.0),
                                );
                        }
                        if h3_on {
                            acc += head_w
                                * self.read_delay_tap(
                                    (self.current_delay_samples
                                        * DECISAO_SONIX_RE201_HEAD3_RATIO
                                        + wow)
                                        .max(1.0),
                                );
                        }
                        let b = self.wet_bass_bq.process(acc);
                        echo_eq = self.wet_treble_bq.process(b);
                        let fb_signal = (echo_eq * feedback_gain).tanh();
                        self.buffer[self.write_pos] =
                            crate::denormal::flush_f32(in_sample + fb_signal);
                        self.write_pos = (self.write_pos + 1) % buf_len;
                    }
                    DelayModel::AnalogBbdDelay => {
                        let tap = self.read_delay_tap(self.current_delay_samples.max(1.0));
                        let b = self.wet_bass_bq.process(tap);
                        echo_eq = self.wet_treble_bq.process(b);
                        // BBD HF rolloff inside feedback loop + soft clipping
                        self.bbd_fb_lp = crate::denormal::flush_f32(
                            self.bbd_fb_lp + bbd_lp_alpha * (echo_eq - self.bbd_fb_lp),
                        );
                        let fb_signal = (self.bbd_fb_lp * feedback_gain).tanh();
                        self.buffer[self.write_pos] =
                            crate::denormal::flush_f32(in_sample + fb_signal);
                        self.write_pos = (self.write_pos + 1) % buf_len;
                    }
                    DelayModel::PristineDigitalDelay => {
                        let tap = self.read_delay_tap(self.current_delay_samples.max(1.0));
                        let b = self.wet_bass_bq.process(tap);
                        echo_eq = self.wet_treble_bq.process(b);
                        let fb_signal = (echo_eq * feedback_gain).clamp(-2.0, 2.0);
                        self.buffer[self.write_pos] =
                            crate::denormal::flush_f32(in_sample + fb_signal);
                        self.write_pos = (self.write_pos + 1) % buf_len;
                    }
                    DelayModel::DualParallelDelay => {
                        let d_main = self.current_delay_samples.max(1.0);
                        let t1 = self.read_delay_tap((d_main * 0.75).max(1.0));
                        let t2 = self.read_delay_tap(d_main);
                        let combined = 0.5 * (t1 + t2);
                        let b = self.wet_bass_bq.process(combined);
                        echo_eq = self.wet_treble_bq.process(b);
                        let fb_signal = (t2 * feedback_gain).clamp(-2.0, 2.0);
                        self.buffer[self.write_pos] =
                            crate::denormal::flush_f32(in_sample + fb_signal);
                        self.write_pos = (self.write_pos + 1) % buf_len;
                    }
                    DelayModel::PingPongStereo => {
                        let d_main = self.current_delay_samples.max(1.0);
                        let t_ping = self.read_delay_tap((d_main * 0.5).max(1.0));
                        let t_pong = self.read_delay_tap(d_main);
                        let combined = 0.5 * (t_ping + t_pong);
                        let b = self.wet_bass_bq.process(combined);
                        echo_eq = self.wet_treble_bq.process(b);
                        let fb_signal = (t_pong * feedback_gain).clamp(-2.0, 2.0);
                        self.buffer[self.write_pos] =
                            crate::denormal::flush_f32(in_sample + fb_signal);
                        self.write_pos = (self.write_pos + 1) % buf_len;
                    }
                    DelayModel::ReverseDelay => {
                        let grain_len =
                            (target_delay_samples.round() as usize).clamp(64, buf_len);
                        let read_idx = grain_len - 1 - (self.rev_write_pos % grain_len);
                        let rev_raw = if self.rev_grain_ready {
                            self.rev_buffer[read_idx]
                        } else {
                            0.0
                        };
                        let b = self.wet_bass_bq.process(rev_raw);
                        echo_eq = self.wet_treble_bq.process(b);
                        let fb_signal = (echo_eq * feedback_gain).clamp(-2.0, 2.0);
                        self.rev_buffer[self.rev_write_pos] =
                            crate::denormal::flush_f32(in_sample + fb_signal);
                        self.rev_write_pos += 1;
                        if self.rev_write_pos >= grain_len {
                            self.rev_write_pos = 0;
                            self.rev_grain_ready = true;
                        }
                    }
                }
            }

            // Mola interna em paralelo ao eco nos modos 5-12 usando TubeSpringReverb (AUD-28)
            let spring_wet = if spring_mix > 0.0001 {
                let mut spring_out = 0.0f32;
                self.spring_reverb.process(
                    std::slice::from_ref(&in_sample),
                    std::slice::from_mut(&mut spring_out),
                );
                spring_out * spring_mix
            } else {
                0.0
            };
            self.samples_since_clear = self.samples_since_clear.saturating_add(1);

            let out_s = if self.bypassed && self.spillover_enabled {
                input[i] + (echo_eq * wet_ratio + spring_wet) * self.duck_mult
            } else if !echo_on {
                // Mode 12: Direct signal + Spring Reverb (reverb_volume comanda a mola)
                input[i] * (1.0 - 0.35 * spring_mix) + spring_wet * self.duck_mult
            } else {
                input[i] * dry_ratio + (echo_eq * wet_ratio + spring_wet) * self.duck_mult
            };
            output[i] = crate::denormal::flush_f32(out_s);
        }
    }
}
