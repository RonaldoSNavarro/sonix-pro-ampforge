//! crates/ampforge-dsp/src/tuner.rs
//! Afinador com Visual Estroboscópico (detecção YIN, ±0.1 cent, 38–1400 Hz; B0 30,9 Hz fora da faixa)
//! e Polifônico de 6 Cordas.

pub const DECISAO_SONIX_TEMPERAMENTO_E2: f32 = -2.4;
pub const DECISAO_SONIX_TEMPERAMENTO_A2: f32 = -2.0;
pub const DECISAO_SONIX_TEMPERAMENTO_D3: f32 = -1.6;
pub const DECISAO_SONIX_TEMPERAMENTO_G3: f32 = -3.2;
pub const DECISAO_SONIX_TEMPERAMENTO_B3: f32 = -1.0;
pub const DECISAO_SONIX_TEMPERAMENTO_E4: f32 = -2.0;

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

/// Separa o nome completo da nota (ex.: `"A2"`, `"F#3"`, `"---"`) em `(letra, oitava)`.
pub fn split_note_and_octave(note_name: &str) -> (&str, Option<i32>) {
    let trimmed = note_name.trim();
    if trimmed.is_empty() || trimmed == "---" {
        return ("---", None);
    }
    if let Some(pos) = trimmed.find(|c: char| c.is_ascii_digit() || c == '-') {
        let letter = &trimmed[..pos];
        if let Ok(oct) = trimmed[pos..].parse::<i32>() {
            if !letter.is_empty() {
                return (letter, Some(oct));
            }
        }
    }
    (trimmed, None)
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
            d_scratch: vec![0.0; 16384],
            d_prime_scratch: vec![0.0; 16384],
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
            self.last_poly_cents = [0.0; 6];
            return TunerPitchResult {
                note_name: "---".to_string(),
                cents_deviation: 0.0,
                frequency_hz: 0.0,
            };
        }

        let n = signal.len();
        let rms = (signal.iter().map(|&s| s * s).sum::<f32>() / n as f32).sqrt();
        if rms < 0.00063 {
            self.last_poly_cents = [0.0; 6];
            return TunerPitchResult {
                note_name: "---".to_string(),
                cents_deviation: 0.0,
                frequency_hz: 0.0,
            };
        }

        // Evita repetir nota antiga quando os últimos ~22 ms já entraram em silêncio (guitarra abafada).
        let tail_len = ((self.sample_rate * 0.022).round() as usize).clamp(256, n);
        let tail = &signal[n - tail_len..];
        let tail_rms = (tail.iter().map(|&s| s * s).sum::<f32>() / tail.len() as f32).sqrt();
        if tail_rms < 0.00063 {
            self.last_poly_cents = [0.0; 6];
            return TunerPitchResult {
                note_name: "---".to_string(),
                cents_deviation: 0.0,
                frequency_hz: 0.0,
            };
        }

        // Em modo polifônico, avalia todas as 6 cordas antes do retorno antecipado do YIN monofônico (acordes têm d' > 0.35).
        if self.mode == TunerMode::Polyphonic {
            self.update_polyphonic_from_samples(signal);
        }

        // Em modo estroboscópico com janela > 4096 (ex.: tuner_scratch=8192), se houver transição de nota
        // (ex.: D3 nos primeiros 4096 e A3 nos últimos 4096), prioriza os 4096 mais recentes.
        if n > 4096 && self.mode != TunerMode::Polyphonic {
            let recent = self.detect_pitch(&signal[n - 4096..]);
            if recent.note_name != "---" && recent.frequency_hz >= 65.0 {
                let tau_est = (self.sample_rate / recent.frequency_hz).round() as usize;
                if tau_est >= 2 && tau_est + 512 <= n - 4096 {
                    let mut diff_sum = 0.0f64;
                    let mut eng_sum = 0.0f64;
                    for j in 0..512 {
                        let a = signal[j] as f64;
                        let b = signal[j + tau_est] as f64;
                        diff_sum += (a - b) * (a - b);
                        eng_sum += a * a + b * b;
                    }
                    if eng_sum <= 1e-12 || (diff_sum / eng_sum) > 0.20 {
                        return recent;
                    }
                }
            }
        }

        // YIN Algorithm:
        // Cobre de 38 Hz a 1400 Hz (inclui E1 = 41,2 Hz; B0 = 30,9 Hz fica fora da faixa)
        let max_lag = (n / 2).min((self.sample_rate / 38.0).ceil() as usize);
        let min_lag = (self.sample_rate / 1450.0).floor().max(1.0) as usize;

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

        // 4. Ajuste local no mínimo de d(tau) + interpolação parabólica para precisão sub-amostra
        let mut center_tau = best_tau;
        if center_tau > min_lag
            && center_tau + 1 < max_lag
            && self.d_scratch[center_tau - 1] < self.d_scratch[center_tau]
            && self.d_scratch[center_tau - 1] <= self.d_scratch[center_tau + 1]
        {
            center_tau -= 1;
        } else if center_tau > min_lag
            && center_tau + 2 < max_lag
            && self.d_scratch[center_tau + 1] < self.d_scratch[center_tau]
            && self.d_scratch[center_tau + 1] <= self.d_scratch[center_tau - 1]
        {
            center_tau += 1;
        }
        let tau_sub = if center_tau > min_lag && center_tau + 1 < max_lag {
            let s0 = self.d_scratch[center_tau - 1];
            let s1 = self.d_scratch[center_tau];
            let s2 = self.d_scratch[center_tau + 1];
            let denom = 2.0 * (s0 - 2.0 * s1 + s2);
            if denom.abs() > 1e-9 && (s0 - 2.0 * s1 + s2) > 0.0 {
                let delta = ((s0 - s2) / denom).clamp(-0.5, 0.5);
                center_tau as f32 + delta
            } else {
                center_tau as f32
            }
        } else {
            center_tau as f32
        };

        let freq_init = self.sample_rate / tau_sub;
        let freq = if freq_init > 330.0 {
            let max_m = (n * 3 / 4).min(n.saturating_sub(6));
            let k = ((max_m as f64) / (tau_sub as f64)).floor().max(1.0) as usize;
            if k >= 2 {
                let m_est = ((k as f64) * (tau_sub as f64)).round() as usize;
                let half_win = ((tau_sub * 0.4).floor() as usize).max(4);
                let m_lo = m_est.saturating_sub(half_win).max(2);
                let m_hi = (m_est + half_win).min(n.saturating_sub(3));
                let win_len = n - (m_hi + 1);
                if win_len >= 64 && m_hi > m_lo + 1 {
                    let mut best_m = m_est.clamp(m_lo, m_hi);
                    let mut best_d = f64::INFINITY;
                    let diff_sq = |m: usize| -> f64 {
                        let mut sum = 0.0f64;
                        for j in 0..win_len {
                            let d = (signal[j] as f64) - (signal[j + m] as f64);
                            sum += d * d;
                        }
                        sum
                    };
                    for m in m_lo..=m_hi {
                        let d = diff_sq(m);
                        if d < best_d {
                            best_d = d;
                            best_m = m;
                        }
                    }
                    let m_sub = if best_m > m_lo && best_m < m_hi {
                        let d0 = diff_sq(best_m - 1);
                        let d1 = best_d;
                        let d2 = diff_sq(best_m + 1);
                        let denom = 2.0 * (d0 - 2.0 * d1 + d2);
                        if denom.abs() > 1e-12 && (d0 - 2.0 * d1 + d2) > 0.0 {
                            best_m as f64 + ((d0 - d2) / denom).clamp(-0.5, 0.5)
                        } else {
                            best_m as f64
                        }
                    } else {
                        best_m as f64
                    };
                    (self.sample_rate as f64 / (m_sub / k as f64)) as f32
                } else {
                    freq_init
                }
            } else {
                freq_init
            }
        } else {
            freq_init
        };

        if freq < 38.0 || freq > 1405.0 {
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

    /// Temperamento suavizado SONIX (cents relativos ao temperamento igual):
    /// E2 (-2.4), A2 (-2.0), D3 (-1.6), G3 (-3.2), B3 (-1.0), E4 (-2.0).
    fn sweetened_offset_cents(midi: i32) -> f32 {
        match midi {
            40 => DECISAO_SONIX_TEMPERAMENTO_E2,
            45 => DECISAO_SONIX_TEMPERAMENTO_A2,
            50 => DECISAO_SONIX_TEMPERAMENTO_D3,
            55 => DECISAO_SONIX_TEMPERAMENTO_G3,
            59 => DECISAO_SONIX_TEMPERAMENTO_B3,
            64 => DECISAO_SONIX_TEMPERAMENTO_E4,
            _ => match ((midi % 12) + 12) % 12 {
                4 => -2.2,                               // E
                9 => DECISAO_SONIX_TEMPERAMENTO_A2,      // A
                2 => DECISAO_SONIX_TEMPERAMENTO_D3,      // D
                7 => DECISAO_SONIX_TEMPERAMENTO_G3,      // G
                11 => DECISAO_SONIX_TEMPERAMENTO_B3,     // B
                _ => 0.0,
            },
        }
    }

    fn fft_radix2(re: &mut [f64], im: &mut [f64]) {
        let n = re.len();
        let mut j = 0usize;
        for i in 0..n {
            if i < j {
                re.swap(i, j);
                im.swap(i, j);
            }
            let mut bit = n >> 1;
            while bit > 0 && (j & bit) != 0 {
                j ^= bit;
                bit >>= 1;
            }
            j |= bit;
        }
        let mut len = 2usize;
        while len <= n {
            let half = len >> 1;
            let angle = -2.0 * std::f64::consts::PI / (len as f64);
            let wlen_re = angle.cos();
            let wlen_im = angle.sin();
            let mut start = 0usize;
            while start < n {
                let mut w_re = 1.0f64;
                let mut w_im = 0.0f64;
                for k in 0..half {
                    let u_re = re[start + k];
                    let u_im = im[start + k];
                    let v_re = re[start + k + half] * w_re - im[start + k + half] * w_im;
                    let v_im = re[start + k + half] * w_im + im[start + k + half] * w_re;
                    re[start + k] = u_re + v_re;
                    im[start + k] = u_im + v_im;
                    re[start + k + half] = u_re - v_re;
                    im[start + k + half] = u_im - v_im;
                    let next_re = w_re * wlen_re - w_im * wlen_im;
                    w_im = w_re * wlen_im + w_im * wlen_re;
                    w_re = next_re;
                }
                start += len;
            }
            len <<= 1;
        }
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
        let n = samples.len();
        if n < 256 {
            return;
        }
        let a4_scale = self.a4_ref / 440.0;
        let mut f_refs = [0.0f64; 6];
        for (k, &(midi, f_eq)) in base_strings.iter().enumerate() {
            let offset_c = if self.temperament == TunerTemperament::SweetenedGuitar {
                Self::sweetened_offset_cents(midi)
            } else {
                0.0
            };
            f_refs[k] = (f_eq * a4_scale * 2.0f32.powf(offset_c / 1200.0)) as f64;
        }

        // Para janelas curtas (< 16384 amostras), atenua vazamento de lóbulo principal entre fundamentais vizinhas
        let mut work_buf: Vec<f64> = samples.iter().map(|&s| s as f64).collect();
        if n < 16384 {
            let inv_n1 = 1.0 / ((n - 1) as f64);
            let norm = 4.0 / (n as f64);
            let mut a_c = [0.0f64; 6];
            let mut b_c = [0.0f64; 6];
            let sr_f = self.sample_rate as f64;
            for k in 0..6 {
                let w = 2.0 * std::f64::consts::PI * f_refs[k] / sr_f;
                let mut sc = 0.0f64;
                let mut ss = 0.0f64;
                for i in 0..n {
                    let win = 0.5 * (1.0 - (2.0 * std::f64::consts::PI * (i as f64) * inv_n1).cos());
                    let x = work_buf[i] * win;
                    let ang = w * (i as f64);
                    sc += x * ang.cos();
                    ss += x * ang.sin();
                }
                a_c[k] = sc * norm;
                b_c[k] = ss * norm;
            }
            // Calcula espectro por corda subtraindo as outras 5 fundamentais quando n < 16384
            for s_idx in 0..6 {
                let n_fft = (n * 4).next_power_of_two();
                let mut re = vec![0.0f64; n_fft];
                let mut im = vec![0.0f64; n_fft];
                for i in 0..n {
                    let mut r = work_buf[i];
                    for j in 0..6 {
                        if j != s_idx {
                            let w = 2.0 * std::f64::consts::PI * f_refs[j] / sr_f;
                            let ang = w * (i as f64);
                            r -= a_c[j] * ang.cos() + b_c[j] * ang.sin();
                        }
                    }
                    let win = 0.5 * (1.0 - (2.0 * std::f64::consts::PI * (i as f64) * inv_n1).cos());
                    re[i] = r * win;
                }
                Self::fft_radix2(&mut re, &mut im);
                let bin_hz = sr_f / (n_fft as f64);
                let half_bins = n_fft / 2;
                let f_ref = f_refs[s_idx];
                let f_lo = f_ref * 2.0f64.powf(-60.0 / 1200.0);
                let f_hi = f_ref * 2.0f64.powf(60.0 / 1200.0);
                let k_lo = (f_lo / bin_hz).floor().max(1.0) as usize;
                let k_hi = (f_hi / bin_hz).ceil().min((half_bins - 2) as f64) as usize;
                let mut best_k = 0usize;
                let mut best_m = 0.0f64;
                for k in (k_lo + 1)..k_hi {
                    let m = re[k].hypot(im[k]);
                    if m > re[k - 1].hypot(im[k - 1]) && m > re[k + 1].hypot(im[k + 1]) && m > best_m {
                        best_m = m;
                        best_k = k;
                    }
                }
                if best_k > 0 && best_m > 1e-12 {
                    let l0 = re[best_k - 1].hypot(im[best_k - 1]).max(1e-12).ln();
                    let l1 = best_m.max(1e-12).ln();
                    let l2 = re[best_k + 1].hypot(im[best_k + 1]).max(1e-12).ln();
                    let denom = 2.0 * (l0 - 2.0 * l1 + l2);
                    let delta = if denom.abs() > 1e-12 && (l0 - 2.0 * l1 + l2) < 0.0 {
                        ((l0 - l2) / denom).clamp(-0.5, 0.5)
                    } else {
                        0.0
                    };
                    let f_est = (best_k as f64 + delta) * bin_hz;
                    self.last_poly_cents[s_idx] = (1200.0 * (f_est / f_ref).log2()) as f32;
                } else {
                    self.last_poly_cents[s_idx] = 0.0;
                }
            }
            return;
        }
        let _ = &mut work_buf;

        // Janela Hann com zero-padding 4x (para n >= 16384 -> n_fft = 65536)
        let n_fft = (n * 4).next_power_of_two();
        let mut re = vec![0.0f64; n_fft];
        let mut im = vec![0.0f64; n_fft];
        let inv_n1 = 1.0 / ((n - 1) as f64);
        for i in 0..n {
            let win = 0.5 * (1.0 - (2.0 * std::f64::consts::PI * (i as f64) * inv_n1).cos());
            re[i] = (samples[i] as f64) * win;
        }
        Self::fft_radix2(&mut re, &mut im);

        let sr = self.sample_rate as f64;
        let bin_hz = sr / (n_fft as f64);
        let half_bins = n_fft / 2;
        let mag = |k: usize| -> f64 { re[k].hypot(im[k]) };

        let partial_collides_with_lower = |f_cand: f64, s_idx: usize| -> bool {
            for j in 0..s_idx {
                for harm in 1..=8 {
                    let f_low = f_refs[j] * (harm as f64);
                    let diff_cents = (1200.0 * (f_cand / f_low).log2()).abs();
                    if diff_cents <= 15.0 {
                        return true;
                    }
                }
            }
            false
        };

        let estimate_partial = |f_target: f64| -> Option<(f64, f64)> {
            let f_lo = f_target * 2.0f64.powf(-60.0 / 1200.0);
            let f_hi = f_target * 2.0f64.powf(60.0 / 1200.0);
            let k_lo = (f_lo / bin_hz).floor().max(1.0) as usize;
            let k_hi = (f_hi / bin_hz).ceil().min((half_bins - 2) as f64) as usize;
            if k_hi <= k_lo {
                return None;
            }
            let mut best_k = 0usize;
            let mut best_m = 0.0f64;
            for k in (k_lo + 1)..k_hi {
                let m = mag(k);
                if m > mag(k - 1) && m > mag(k + 1) && m > best_m {
                    best_m = m;
                    best_k = k;
                }
            }
            if best_k == 0 || best_m <= 1e-12 {
                return None;
            }
            let l0 = mag(best_k - 1).max(1e-12).ln();
            let l1 = best_m.max(1e-12).ln();
            let l2 = mag(best_k + 1).max(1e-12).ln();
            let denom = 2.0 * (l0 - 2.0 * l1 + l2);
            let delta = if denom.abs() > 1e-12 && (l0 - 2.0 * l1 + l2) < 0.0 {
                ((l0 - l2) / denom).clamp(-0.5, 0.5)
            } else {
                0.0
            };
            let f_est = (best_k as f64 + delta) * bin_hz;
            Some((f_est, best_m))
        };

        for s_idx in 0..6 {
            let f_ref = f_refs[s_idx];
            let mut num = 0.0f64;
            let mut den = 0.0f64;
            let mut used_primary = false;

            let mut f1_mag = 0.0f64;
            for harm in [1usize, 2usize] {
                let f_cand = f_ref * (harm as f64);
                if !partial_collides_with_lower(f_cand, s_idx) {
                    used_primary = true;
                    if let Some((f_est, m)) = estimate_partial(f_cand) {
                        if harm == 1 {
                            f1_mag = m;
                            num += m * f_est;
                            den += m;
                        } else if f1_mag <= 0.0 || m >= 0.08 * f1_mag {
                            num += m * (f_est / (harm as f64));
                            den += m;
                        }
                    }
                }
            }

            if !used_primary {
                // Se fundamental e 2o harmônico forem excluídos, usa o 3o (ou 5o se o 3o também coincidir com n <= 8)
                let fallback_harm = if !partial_collides_with_lower(f_ref * 3.0, s_idx) {
                    3usize
                } else if !partial_collides_with_lower(f_ref * 5.0, s_idx) {
                    5usize
                } else {
                    3usize
                };
                if let Some((f_est, m)) = estimate_partial(f_ref * (fallback_harm as f64)) {
                    num += m * (f_est / (fallback_harm as f64));
                    den += m;
                }
            }

            if den > 0.0 {
                let f_est = num / den;
                self.last_poly_cents[s_idx] = (1200.0 * (f_est / f_ref).log2()) as f32;
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
