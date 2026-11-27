//! Módulos independentes dos 5 amplificadores valvulados (Specs 054–058).
//!
//! Cada amplificador implementa `AudioProcessor` + `ModelDescriptor` de forma
//! autônoma e é registrado no `ProcessorRegistry` da Spec 053.

pub mod fender_twin;
pub mod marshall_1959;
pub mod marshall_jcm800;
pub mod mesa_dual_rectifier;
pub mod vox_ac30;

pub use fender_twin::FenderTwinReverbAb763;
pub use marshall_1959::Marshall1959SuperLead;
pub use marshall_jcm800::MarshallJcm800_2203;
pub use mesa_dual_rectifier::MesaDualRectifierRevG;
pub use vox_ac30::VoxAc30TopBoost1964;

use crate::os_filters::elliptic8_sos;
use crate::processor::{ParamDescriptor, ParamId, ParamTaper, ProcessorRegistry};

// IDs canônicos de parâmetros RT para `apply_param_rt`
pub const PID_POWER: ParamId = 1;
pub const PID_STANDBY: ParamId = 2;
pub const PID_GAIN: ParamId = 3;
pub const PID_VOLUME_2: ParamId = 4;
pub const PID_MASTER: ParamId = 5;
pub const PID_BASS: ParamId = 6;
pub const PID_MIDDLE: ParamId = 7;
pub const PID_TREBLE: ParamId = 8;
pub const PID_PRESENCE: ParamId = 9;
pub const PID_CUT: ParamId = 10;
pub const PID_REVERB: ParamId = 11;
pub const PID_VIBRATO_SPEED: ParamId = 12;
pub const PID_VIBRATO_DEPTH: ParamId = 13;
pub const PID_BRIGHT: ParamId = 14;
pub const PID_JUMPERED: ParamId = 15;
pub const PID_SPONGY: ParamId = 16;
pub const PID_INPUT_JACK: ParamId = 17;
pub const PID_CHANNEL_MODE: ParamId = 18;
pub const PID_IMPEDANCE_OHMS: ParamId = 19;
pub const PID_OUTPUT_LEVEL_DB: ParamId = 20;

pub const fn param_bool(id: ParamId, key: &'static str, label: &'static str, default: f32) -> ParamDescriptor {
    ParamDescriptor {
        id,
        key,
        label,
        unit: "bool",
        min: 0.0,
        max: 1.0,
        default,
        taper: ParamTaper::Boolean,
        enum_variants: &[],
    }
}

pub const fn param_knob(
    id: ParamId,
    key: &'static str,
    label: &'static str,
    default: f32,
    taper: ParamTaper,
) -> ParamDescriptor {
    ParamDescriptor {
        id,
        key,
        label,
        unit: "scale",
        min: 0.0,
        max: 10.0,
        default,
        taper,
        enum_variants: &[],
    }
}

pub const fn param_enum(
    id: ParamId,
    key: &'static str,
    label: &'static str,
    variants: &'static [&'static str],
) -> ParamDescriptor {
    ParamDescriptor {
        id,
        key,
        label,
        unit: "id",
        min: 0.0,
        max: 10.0,
        default: 0.0,
        taper: ParamTaper::Discrete,
        enum_variants: variants,
    }
}

/// Filtro elíptico de 8ª ordem reutilizável para oversampling temporário da Fase 2A
/// (será substituído pelo `PolyphaseIirOversampler` HIIR unificado na Fase 2B / Spec 079).
#[derive(Debug, Clone, Copy)]
pub struct EllipSos {
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub a1: f64,
    pub a2: f64,
    pub s1: f64,
    pub s2: f64,
}

impl Default for EllipSos {
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            s1: 0.0,
            s2: 0.0,
        }
    }
}

impl EllipSos {
    pub fn from_coeffs(c: [f64; 5]) -> Self {
        Self {
            b0: c[0],
            b1: c[1],
            b2: c[2],
            a1: c[3],
            a2: c[4],
            s1: 0.0,
            s2: 0.0,
        }
    }

    #[inline]
    pub fn update_coeffs(&mut self, c: [f64; 5]) {
        self.b0 = c[0];
        self.b1 = c[1];
        self.b2 = c[2];
        self.a1 = c[3];
        self.a2 = c[4];
    }

    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }

    #[inline(always)]
    pub fn process(&mut self, x: f64) -> f64 {
        let out = crate::denormal::flush_f64(self.b0 * x + self.s1);
        self.s1 = crate::denormal::flush_f64(self.b1 * x - self.a1 * out + self.s2);
        self.s2 = crate::denormal::flush_f64(self.b2 * x - self.a2 * out);
        out
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AmpOversamplerBank {
    pub factor: usize,
    pub sample_rate: f32,
    pub ai: [EllipSos; 4],
    pub aa: [EllipSos; 4],
}

impl Default for AmpOversamplerBank {
    fn default() -> Self {
        Self::new(2)
    }
}

impl AmpOversamplerBank {
    pub fn new(factor: usize) -> Self {
        let mut s = Self {
            factor: 0,
            sample_rate: 48_000.0,
            ai: [EllipSos::default(); 4],
            aa: [EllipSos::default(); 4],
        };
        s.configure(48_000.0, factor);
        s
    }

    pub fn configure(&mut self, sample_rate: f32, factor: usize) {
        let sr = sample_rate.max(8_000.0);
        let new_factor = match factor {
            1 => 1,
            2 => 2,
            4 => 4,
            _ => 8,
        };
        if self.factor == new_factor && (self.sample_rate - sr).abs() <= 1.0 {
            return;
        }
        let factor_changed = self.factor != new_factor;
        self.factor = new_factor;
        self.sample_rate = sr;
        if new_factor > 1 {
            let coeffs = elliptic8_sos(sr, new_factor);
            for i in 0..4 {
                self.ai[i].update_coeffs(coeffs[i]);
                self.aa[i].update_coeffs(coeffs[i]);
                if factor_changed {
                    self.ai[i].reset();
                    self.aa[i].reset();
                }
            }
        }
    }

    pub fn set_factor(&mut self, factor: usize) {
        self.configure(self.sample_rate, factor);
    }

    #[inline]
    pub fn latency_samples(&self) -> usize {
        match self.factor {
            1 => 0,
            2 => 4,
            4 => 5,
            _ => 6,
        }
    }

    pub fn reset(&mut self) {
        for f in &mut self.ai {
            f.reset();
        }
        for f in &mut self.aa {
            f.reset();
        }
    }

    #[inline(always)]
    pub fn upsample_step(&mut self, base_sample: f64, step_idx: usize) -> f64 {
        if self.factor <= 1 {
            return base_sample;
        }
        let mut x = if step_idx == 0 {
            (self.factor as f64) * base_sample
        } else {
            0.0
        };
        for f in &mut self.ai {
            x = f.process(x);
        }
        x
    }

    #[inline(always)]
    pub fn downsample_step(&mut self, os_sample: f64) -> f64 {
        if self.factor <= 1 {
            return os_sample;
        }
        let mut x = os_sample;
        for f in &mut self.aa {
            x = f.process(x);
        }
        x
    }
}

/// Registra todos os 5 amplificadores modulares (Specs 054–058) no `ProcessorRegistry`.
pub fn register_amp_processors(registry: &mut ProcessorRegistry) {
    registry.register(
        Marshall1959SuperLead::DESCRIPTOR,
        || Box::new(Marshall1959SuperLead::new()),
        &["plexi_1959", "brit_plexi_1959", "plexi"],
    );
    registry.register(
        MarshallJcm800_2203::DESCRIPTOR,
        || Box::new(MarshallJcm800_2203::new()),
        &["jcm800_2203", "brit_jcm800_2203", "brit800"],
    );
    registry.register(
        FenderTwinReverbAb763::DESCRIPTOR,
        || Box::new(FenderTwinReverbAb763::new()),
        &["twin_ab763", "us_twin_blackface_65", "twin_reverb", "usdeluxe"],
    );
    registry.register(
        MesaDualRectifierRevG::DESCRIPTOR,
        || Box::new(MesaDualRectifierRevG::new()),
        &["dual_rect", "cali_dual_recto_revf", "dual_rectifier", "rectifier_solo_100", "recto"],
    );
    registry.register(
        VoxAc30TopBoost1964::DESCRIPTOR,
        || Box::new(VoxAc30TopBoost1964::new()),
        &["ac30_tb", "uk_ac30_topboost", "ac30", "ac30_top_boost"],
    );
}
