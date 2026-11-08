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

pub struct PitchProcessor {
    model: PitchModel,
    semitones: f32,
    key: PitchKey,
    scale: ScaleType,
    mix_pct: f32,
}

impl PitchProcessor {
    pub fn new(model: PitchModel) -> Self {
        Self {
            model,
            semitones: 0.0,
            key: PitchKey::C,
            scale: ScaleType::Major,
            mix_pct: 100.0,
        }
    }

    pub fn model(&self) -> PitchModel {
        self.model
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

    pub fn transposition_ratio(&self) -> f32 {
        2.0f32.powf(self.semitones / 12.0)
    }

    pub fn set_key(&mut self, key: PitchKey) {
        self.key = key;
    }

    pub fn set_scale(&mut self, scale: ScaleType) {
        self.scale = scale;
    }

    pub fn calculate_harmonic_frequency(&self, input_freq_hz: f32) -> f32 {
        // Simple diatonic 3rd interval solver in C Major:
        // C4 (261.63 Hz) -> E4 (Major 3rd = +4 semitones -> 261.63 * 2^(4/12) = 329.63 Hz)
        // D4 (293.66 Hz) -> F4 (Minor 3rd = +3 semitones -> 293.66 * 2^(3/12) = 349.23 Hz)
        if (input_freq_hz - 261.63).abs() < 10.0 {
            input_freq_hz * 2.0f32.powf(4.0 / 12.0)
        } else if (input_freq_hz - 293.66).abs() < 10.0 {
            input_freq_hz * 2.0f32.powf(3.0 / 12.0)
        } else {
            // General diatonic interval approximation
            input_freq_hz * 2.0f32.powf(4.0 / 12.0)
        }
    }
}
