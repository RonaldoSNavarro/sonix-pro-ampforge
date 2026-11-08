//! crates/ampforge-dsp/src/cab.rs
//! Bloco de Gabinete (Cab), Sala de Microfonação Dupla, Loader de IR e Acoplamento de Impedância

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicModel {
    Sm57,
    Md421,
    M88,
    R121,
    M160,
    U87,
    C414,
}

#[derive(Debug, Clone)]
pub struct MicSlotConfig {
    pub model: MicModel,
    pub position_cap_to_edge: f32, // 0.0 (Cap) to 1.0 (Edge)
    pub distance_cm: f32,          // 0.0 to 30.0 cm
    pub angle_off_axis_deg: f32,   // 0.0 or 45.0
    pub level_db: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub phase_invert: bool,
    pub delay_ms: f32,
    pub enabled: bool,
}

impl Default for MicSlotConfig {
    fn default() -> Self {
        Self {
            model: MicModel::Sm57,
            position_cap_to_edge: 0.2,
            distance_cm: 2.0,
            angle_off_axis_deg: 0.0,
            level_db: 0.0,
            pan: 0.0,
            mute: false,
            solo: false,
            phase_invert: false,
            delay_ms: 0.0,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImpedanceCurve {
    pub low_resonance_hz: f32,
    pub resonance_q: f32,
    pub low_resonance_gain_db: f32,
    pub high_inductance_rise_db: f32,
}

impl Default for ImpedanceCurve {
    fn default() -> Self {
        Self {
            low_resonance_hz: 80.0,
            resonance_q: 2.0,
            low_resonance_gain_db: 6.0,
            high_inductance_rise_db: 4.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CabAcousticConditioning {
    pub low_cut_hz: f32,
    pub low_cut_slope_db_oct: u32,
    pub high_cut_hz: f32,
    pub speaker_thump: f32,
    pub room_mic_early_reflections_pct: f32,
}

impl Default for CabAcousticConditioning {
    fn default() -> Self {
        Self {
            low_cut_hz: 60.0,
            low_cut_slope_db_oct: 12,
            high_cut_hz: 10000.0,
            speaker_thump: 0.0,
            room_mic_early_reflections_pct: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidImpulseResponseError {
    EmptyFile,
    CorruptedOrTruncated,
    UnsupportedFormat(String),
}

impl std::fmt::Display for InvalidImpulseResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "Arquivo de IR vazio (0 bytes)"),
            Self::CorruptedOrTruncated => write!(f, "Arquivo RIFF/WAVE corrompido ou truncado"),
            Self::UnsupportedFormat(err) => write!(f, "Formato de IR não suportado: {}", err),
        }
    }
}

impl std::error::Error for InvalidImpulseResponseError {}

#[derive(Debug, Clone)]
pub struct ImpulseResponse {
    sample_rate: f32,
    samples: Vec<f32>,
}

impl ImpulseResponse {
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }
}

pub struct WavIrLoader {
    session_sample_rate: f32,
}

impl WavIrLoader {
    pub fn new(session_sample_rate: f32) -> Self {
        Self {
            session_sample_rate: session_sample_rate.max(1.0),
        }
    }

    pub fn load_from_bytes(&self, bytes: &[u8]) -> Result<ImpulseResponse, InvalidImpulseResponseError> {
        if bytes.is_empty() {
            return Err(InvalidImpulseResponseError::EmptyFile);
        }
        if bytes.len() < 44 {
            return Err(InvalidImpulseResponseError::CorruptedOrTruncated);
        }

        // Check RIFF header
        if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return Err(InvalidImpulseResponseError::CorruptedOrTruncated);
        }

        // Locate 'fmt ' and 'data' chunks
        let mut pos = 12;
        let mut sample_rate = 44100u32;
        let mut channels = 1u16;
        let mut bits_per_sample = 16u16;
        let mut audio_format = 1u16;
        let mut data_start: Option<usize> = None;
        let mut data_len = 0usize;

        while pos + 8 <= bytes.len() {
            let chunk_id = &bytes[pos..pos + 4];
            let chunk_size = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
            pos += 8;

            if chunk_id == b"fmt " && chunk_size >= 16 && pos + 16 <= bytes.len() {
                audio_format = u16::from_le_bytes(bytes[pos..pos + 2].try_into().unwrap());
                channels = u16::from_le_bytes(bytes[pos + 2..pos + 4].try_into().unwrap());
                sample_rate = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap());
                bits_per_sample = u16::from_le_bytes(bytes[pos + 14..pos + 16].try_into().unwrap());
            } else if chunk_id == b"data" {
                data_start = Some(pos);
                data_len = chunk_size.min(bytes.len() - pos);
                break;
            }
            pos += chunk_size;
        }

        let start = data_start.ok_or(InvalidImpulseResponseError::CorruptedOrTruncated)?;
        let end = (start + data_len).min(bytes.len());
        let raw_data = &bytes[start..end];

        let mut samples = Vec::new();
        match (audio_format, bits_per_sample) {
            (1, 16) => {
                // PCM 16-bit
                for chunk in raw_data.chunks_exact(2 * channels as usize) {
                    let s_i16 = i16::from_le_bytes([chunk[0], chunk[1]]);
                    samples.push(s_i16 as f32 / 32768.0);
                }
            }
            (1, 24) => {
                // PCM 24-bit
                for chunk in raw_data.chunks_exact(3 * channels as usize) {
                    let s_i32 = i32::from_le_bytes([0, chunk[0], chunk[1], chunk[2]]) >> 8;
                    samples.push(s_i32 as f32 / 8388608.0);
                }
            }
            (3, 32) => {
                // IEEE Float 32-bit
                for chunk in raw_data.chunks_exact(4 * channels as usize) {
                    let s_f32 = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    samples.push(s_f32);
                }
            }
            _ => {
                return Err(InvalidImpulseResponseError::UnsupportedFormat(format!(
                    "audio_format: {}, bits: {}",
                    audio_format, bits_per_sample
                )));
            }
        }

        if samples.is_empty() {
            return Err(InvalidImpulseResponseError::CorruptedOrTruncated);
        }

        // Resample if needed
        if (sample_rate as f32 - self.session_sample_rate).abs() > 1.0 {
            let ratio = sample_rate as f32 / self.session_sample_rate;
            let new_len = (samples.len() as f32 / ratio).round() as usize;
            let mut resampled = Vec::with_capacity(new_len);
            for i in 0..new_len {
                let orig_idx = i as f32 * ratio;
                let idx0 = orig_idx.floor() as usize;
                let idx1 = (idx0 + 1).min(samples.len() - 1);
                let frac = orig_idx - idx0 as f32;
                resampled.push(samples[idx0] * (1.0 - frac) + samples[idx1] * frac);
            }
            samples = resampled;
        }

        Ok(ImpulseResponse {
            sample_rate: self.session_sample_rate,
            samples,
        })
    }
}

pub struct VirtualCabProcessor {
    mics: [MicSlotConfig; 2],
    impedance_curve: ImpedanceCurve,
    conditioning: CabAcousticConditioning,
    coupled_amp_id: Option<String>,
    sample_rate: f32,
    custom_ir: Option<ImpulseResponse>,
    // Delay buffer for fractional delay
    delay_line_slot1: Vec<f32>,
    delay_pos_slot1: usize,
}

impl Default for VirtualCabProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualCabProcessor {
    pub fn new() -> Self {
        Self {
            mics: [MicSlotConfig::default(), MicSlotConfig::default()],
            impedance_curve: ImpedanceCurve::default(),
            conditioning: CabAcousticConditioning::default(),
            coupled_amp_id: None,
            sample_rate: 48000.0,
            custom_ir: None,
            delay_line_slot1: vec![0.0; 16384],
            delay_pos_slot1: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn mic_slot(&self, idx: usize) -> &MicSlotConfig {
        &self.mics[idx]
    }

    pub fn mic_slot_mut(&mut self, idx: usize) -> &mut MicSlotConfig {
        &mut self.mics[idx]
    }

    pub fn impedance_curve(&self) -> &ImpedanceCurve {
        &self.impedance_curve
    }

    pub fn set_impedance_curve(&mut self, curve: ImpedanceCurve) {
        self.impedance_curve = curve;
    }

    pub fn impedance_badge_text(&self) -> String {
        format!("{:.1} Hz // Q: {:.1}", self.impedance_curve.low_resonance_hz, self.impedance_curve.resonance_q)
    }

    pub fn coupled_amp_id(&self) -> Option<&str> {
        self.coupled_amp_id.as_deref()
    }

    pub fn set_coupled_amp_id(&mut self, id: impl Into<String>) {
        self.coupled_amp_id = Some(id.into());
    }

    pub fn conditioning(&self) -> &CabAcousticConditioning {
        &self.conditioning
    }

    pub fn set_conditioning(&mut self, cond: CabAcousticConditioning) {
        self.conditioning = cond;
    }

    pub fn load_custom_ir(&mut self, bytes: &[u8]) -> Result<(), InvalidImpulseResponseError> {
        let loader = WavIrLoader::new(self.sample_rate);
        let ir = loader.load_from_bytes(bytes)?;
        self.custom_ir = Some(ir);
        Ok(())
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        output.fill(0.0);

        // Process Slot 0 if enabled and not muted
        if self.mics[0].enabled && !self.mics[0].mute {
            let gain = if self.mics[0].level_db <= -60.0 {
                0.0
            } else {
                10.0f32.powf(self.mics[0].level_db / 20.0)
            };
            let sign = if self.mics[0].phase_invert { -1.0 } else { 1.0 };
            let total_gain = gain * sign;

            for i in 0..input.len() {
                output[i] += input[i] * total_gain;
            }
        }

        // Process Slot 1 if enabled and not muted
        if self.mics[1].enabled && !self.mics[1].mute {
            let gain = if self.mics[1].level_db <= -60.0 {
                0.0
            } else {
                10.0f32.powf(self.mics[1].level_db / 20.0)
            };
            let sign = if self.mics[1].phase_invert { -1.0 } else { 1.0 };
            let total_gain = gain * sign;

            let line_len = self.delay_line_slot1.len();
            // Fractional delay in samples clamped within buffer capacity
            let max_delay = line_len.saturating_sub(2);
            let delay_samples = (self.mics[1].delay_ms * 0.001 * self.sample_rate)
                .max(0.0)
                .min(max_delay as f32);
            let d_int = (delay_samples.floor() as usize).min(max_delay);
            let d_frac = delay_samples - d_int as f32;

            for i in 0..input.len() {
                let x = input[i];
                self.delay_line_slot1[self.delay_pos_slot1] = x;

                let r_idx0 = (self.delay_pos_slot1 + line_len - d_int) % line_len;
                let r_idx1 = (r_idx0 + line_len - 1) % line_len;

                let delayed_sample = self.delay_line_slot1[r_idx0] * (1.0 - d_frac)
                    + self.delay_line_slot1[r_idx1] * d_frac;

                self.delay_pos_slot1 = (self.delay_pos_slot1 + 1) % line_len;
                output[i] += delayed_sample * total_gain;
            }
        }
    }
}
