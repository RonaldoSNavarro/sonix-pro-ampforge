//! crates/ampforge-dsp/src/fx/modulation.rs
//! Processadores de Modulação (Chorus, Flanger, Phaser, Tremolo, Univibe, Rotary)

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
    tempo_sync: bool,
    note_division: NoteDivision,
    tempo_bpm: f32,
    sample_rate: f32,
    lfo_phase: f32,
    delay_buffer: Vec<f32>,
    write_pos: usize,
}

impl ModulationProcessor {
    pub fn new(model: ModModel) -> Self {
        Self {
            model,
            rate_hz: 1.5,
            depth: 5.0,
            mix_pct: 50.0,
            tempo_sync: false,
            note_division: NoteDivision::Quarter,
            tempo_bpm: 120.0,
            sample_rate: 48000.0,
            lfo_phase: 0.0,
            delay_buffer: vec![0.0f32; 4096],
            write_pos: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
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

    pub fn calculated_lfo_freq_hz(&self) -> f32 {
        if self.tempo_sync {
            let beats_per_sec = self.tempo_bpm / 60.0;
            beats_per_sec / self.note_division.quarter_note_multiplier()
        } else {
            self.rate_hz
        }
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        if self.mix_pct <= 0.0 {
            output.copy_from_slice(input);
            return;
        }

        let wet_ratio = self.mix_pct / 100.0;
        let dry_ratio = 1.0 - wet_ratio;
        let lfo_freq = self.calculated_lfo_freq_hz();
        let phase_inc = 2.0 * std::f32::consts::PI * lfo_freq / self.sample_rate;
        let buf_len = self.delay_buffer.len();

        let base_delay_samples = 0.008 * self.sample_rate; // 8 ms centro
        let mod_depth_samples = (self.depth / 10.0) * 0.004 * self.sample_rate; // modulação até 4 ms

        for i in 0..input.len() {
            let x = input[i];
            self.delay_buffer[self.write_pos] = x;

            let lfo_val = (self.lfo_phase).sin();
            self.lfo_phase += phase_inc;
            if self.lfo_phase > 2.0 * std::f32::consts::PI {
                self.lfo_phase -= 2.0 * std::f32::consts::PI;
            }

            let d_samples = (base_delay_samples + mod_depth_samples * lfo_val).clamp(1.0, (buf_len - 2) as f32);
            let read_pos = (self.write_pos as f32 + buf_len as f32 - d_samples) % buf_len as f32;
            let idx0 = read_pos.floor() as usize % buf_len;
            let idx1 = (idx0 + 1) % buf_len;
            let frac = read_pos - read_pos.floor();
            let delayed = self.delay_buffer[idx0] * (1.0 - frac) + self.delay_buffer[idx1] * frac;

            self.write_pos = (self.write_pos + 1) % buf_len;

            output[i] = x * dry_ratio + delayed * wet_ratio;
        }
    }
}
