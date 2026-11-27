//! Spec 055: Amplificador Marshall JCM800 2203 (Rev. 1981 Master Volume Lead)
//!
//! - Jack `HIGH`: V1a (`2.7k||0.68uF`) -> `Pre-Amp` (`gain`) + bright cap `1000 pF`
//!   -> V1b Cold Clipper (`Rk = 10 kOhm` sem bypass, forte H2)
//!   -> divisor RC analítico `470k||470pF` sobre `470k` (`fz = 720.8 Hz`, `fp = 1441.6 Hz`)
//!   -> V2a + V2b `CathodeFollowerStage` -> FMV (`Bass` audio25, `Mid`/`Treble` linear, `R_src = 1.3k`)
//!   -> `Master Volume` (silencia em `0.0`) -> LTP + `4xEL34` + NFB + `Presence`.
//! - Jack `LOW`: bypassa V1a + pad resistivo `-6.02 dB` (`0.5`).

use super::{
    param_bool, param_enum, param_knob, AmpOversamplerBank, PID_BASS, PID_GAIN,
    PID_IMPEDANCE_OHMS, PID_INPUT_JACK, PID_MASTER, PID_MIDDLE, PID_OUTPUT_LEVEL_DB, PID_POWER,
    PID_PRESENCE, PID_STANDBY, PID_TREBLE,
};
use crate::amp::{AmpFrontPanel, AmpInputJack, AmpModelId, ChassisModSettings, nominal_mod_settings_for_model};
use crate::denormal::flush_f64;
use crate::modshop::ToneStackTopology;
use crate::processor::{
    AudioBlockMut, AudioBlockRef, AudioProcessor, AudioSpec, ChannelLayout, DspError,
    ModelDescriptor, ParamDescriptor, ParamError, ParamId, ParamTaper, ParamValue, ProcessContext,
};
use crate::tonestack::{jcm800_fmv_components, spec078_fmv_components, FmvToneStackSolver};
use crate::tube::{
    CathodeFollowerStage, OutputTransformer, OutputTransformerModel, PhaseInverterLtpStage,
    PowerSupplyAndRectifier, PowerSupplySag, PushPullPowerStage, ReactiveLoadAndNfb, RectifierKind,
    TriodeParams, TriodeStage,
};
use std::f64::consts::PI;

const JCM_JACKS: &[&str] = &["high", "low"];
const JCM_IMPEDANCES: &[&str] = &["4", "8", "16"];

const JCM800_PARAMS: &[ParamDescriptor] = &[
    param_bool(PID_POWER, "amp_1.power", "Power", 1.0),
    param_bool(PID_STANDBY, "amp_1.standby", "Standby", 0.0),
    param_knob(PID_PRESENCE, "amp_1.presence", "Presence", 5.0, ParamTaper::Log10),
    param_knob(PID_BASS, "amp_1.bass", "Bass", 5.5, ParamTaper::Audio25),
    param_knob(PID_MIDDLE, "amp_1.middle", "Middle", 5.0, ParamTaper::Linear),
    param_knob(PID_TREBLE, "amp_1.treble", "Treble", 6.0, ParamTaper::Linear),
    param_knob(PID_MASTER, "amp_1.master", "Master", 5.0, ParamTaper::Audio25),
    param_knob(PID_GAIN, "amp_1.gain", "Pre-Amp", 6.5, ParamTaper::Audio25),
    param_enum(PID_INPUT_JACK, "amp_1.input_jack", "Input Jack", JCM_JACKS),
    param_enum(PID_IMPEDANCE_OHMS, "amp_1.impedance_ohms", "Impedance", JCM_IMPEDANCES),
];

/// Filtro lead-lag analítico exato da rede `470 kOhm || 470 pF` sobre `470 kOhm`
/// ($f_z = 720.8\text{ Hz}$, $f_p = 1441.6\text{ Hz}$, bilinear em `f64`).
#[derive(Debug, Clone, Copy)]
struct PeakingDivider470k {
    b0: f64,
    b1: f64,
    a1: f64,
    x1: f64,
    y1: f64,
}

impl PeakingDivider470k {
    fn new(fs: f64) -> Self {
        let mut s = Self {
            b0: 1.0,
            b1: 0.0,
            a1: 0.0,
            x1: 0.0,
            y1: 0.0,
        };
        s.update(fs);
        s
    }

    fn update(&mut self, fs: f64) {
        // H(s) = 0.5 * (1 + s/wz) / (1 + s/wp), wz = 2*pi*720.8, wp = 2*pi*1441.6
        let fz = 720.8;
        let fp = 1441.6;
        let k = 2.0 * fs.max(8000.0);
        let wz = 2.0 * PI * fz;
        let wp = 2.0 * PI * fp;
        let a0 = 1.0 + k / wp;
        let b0 = 0.5 * (1.0 + k / wz) / a0;
        let b1 = 0.5 * (1.0 - k / wz) / a0;
        let a1 = (1.0 - k / wp) / a0;
        self.b0 = b0;
        self.b1 = b1;
        self.a1 = a1;
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }

    #[inline(always)]
    fn process(&mut self, x: f64) -> f64 {
        let y = flush_f64(self.b0 * x + self.b1 * self.x1 - self.a1 * self.y1);
        self.x1 = flush_f64(x);
        self.y1 = y;
        y
    }
}

pub struct MarshallJcm800_2203 {
    pub panel: AmpFrontPanel,
    pub mod_settings: ChassisModSettings,
    pub sample_rate: f32,
    pub speaker_resonance_hz: f32,
    pub(super) os: AmpOversamplerBank,
    v1a: TriodeStage,
    v1b_cold: TriodeStage,
    divider: PeakingDivider470k,
    v2a: TriodeStage,
    v2b_cf: CathodeFollowerStage,
    tonestack: FmvToneStackSolver,
    pi: PhaseInverterLtpStage,
    power_stage: PushPullPowerStage,
    sag: PowerSupplySag,
    xformer: OutputTransformer,
    reactive: ReactiveLoadAndNfb,
    pub(super) standby_env: f64,
    bright_hp_state: f64,
    last_os_sr: f64,
}

impl Default for MarshallJcm800_2203 {
    fn default() -> Self {
        Self::new()
    }
}

impl MarshallJcm800_2203 {
    pub const DESCRIPTOR: ModelDescriptor = ModelDescriptor {
        module_id: "amp.marshall.jcm800_2203",
        model_revision: "1981-jcm800-2203-jmp-circuit",
        display_name: "Marshall JCM800 2203",
        layout: ChannelLayout::Mono,
        params: JCM800_PARAMS,
    };

    pub fn new() -> Self {
        let mut panel = AmpFrontPanel::default();
        panel.input_gain = 6.5;
        panel.master_volume = 5.0;
        panel.input_jack = AmpInputJack::High;
        panel.impedance_ohms = 16;
        let mod_settings = nominal_mod_settings_for_model(AmpModelId::BritJcm800_2203);
        let sr = 48_000.0f64;
        let os_sr = sr * 2.0;

        let mut s = Self {
            panel,
            mod_settings: mod_settings.clone(),
            sample_rate: sr as f32,
            speaker_resonance_hz: 98.0,
            os: AmpOversamplerBank::new(2),
            v1a: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 2_700.0,
                    ck_farads: 0.68e-6,
                    cc_farads: 22e-9,
                    gain_linear: 2.5,
                    asym: mod_settings.triode_asym as f64,
                    nominal_b_plus_v: 280.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            v1b_cold: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 10_000.0,
                    ck_farads: 0.0, // sem bypass: cold clipper
                    cc_farads: 22e-9,
                    gain_linear: 5.5,
                    asym: (mod_settings.triode_asym as f64) * 2.2,
                    nominal_b_plus_v: 280.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            divider: PeakingDivider470k::new(os_sr),
            v2a: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 820.0,
                    ck_farads: 0.0,
                    cc_farads: 22e-9,
                    gain_linear: 2.1,
                    asym: mod_settings.triode_asym as f64,
                    nominal_b_plus_v: 280.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            v2b_cf: CathodeFollowerStage::new(0.94, 1_300.0, 1.25),
            tonestack: FmvToneStackSolver::new(),
            pi: PhaseInverterLtpStage::default(),
            power_stage: PushPullPowerStage::new(mod_settings.power_tube, mod_settings.bias_ma as f64),
            sag: PowerSupplySag::new(
                PowerSupplyAndRectifier::new(RectifierKind::from(mod_settings.rectifier), 450.0),
                os_sr,
            ),
            xformer: OutputTransformer::new(OutputTransformerModel::DagnallC1998, os_sr),
            reactive: ReactiveLoadAndNfb::new(os_sr),
            standby_env: 1.0,
            bright_hp_state: 0.0,
            last_os_sr: os_sr,
        };
        s.sync_tonestack(true);
        s
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.os.configure(self.sample_rate, self.os.factor);
        let os_sr = self.sample_rate as f64 * self.os.factor as f64;
        self.update_rates(os_sr);
    }

    pub fn set_oversampling_factor(&mut self, factor: usize) {
        self.os.configure(self.sample_rate, factor);
        let os_sr = self.sample_rate as f64 * self.os.factor as f64;
        self.update_rates(os_sr);
    }

    fn update_rates(&mut self, os_sr: f64) {
        if (os_sr - self.last_os_sr).abs() > 1.0 {
            self.last_os_sr = os_sr;
            self.v1a.update_coeffs(os_sr);
            self.v1b_cold.update_coeffs(os_sr);
            self.divider.update(os_sr);
            self.v2a.update_coeffs(os_sr);
            self.sag.set_sample_rate(os_sr);
            self.xformer.set_sample_rate(os_sr);
            self.reactive.set_sample_rate(os_sr);
            self.sync_tonestack(true);
        }
    }

    fn sync_tonestack(&mut self, immediate: bool) {
        let comps = if self.mod_settings.tonestack == ToneStackTopology::Model {
            jcm800_fmv_components(self.panel.bass, self.panel.middle, self.panel.treble)
        } else {
            spec078_fmv_components(
                self.mod_settings.tonestack,
                self.panel.bass,
                self.panel.middle,
                self.panel.treble,
            )
        };
        self.tonestack.update_components(comps, self.last_os_sr, immediate);
    }

    #[inline]
    pub fn current_b_plus_v(&self) -> f32 {
        self.sag.current_b_plus_v() as f32
    }

    pub fn process_mono_slice(&mut self, input: &[f32], output: &mut [f32]) {
        let os_factor = self.os.factor;
        let fs_base = self.sample_rate.max(8000.0) as f64;
        let os_sr = fs_base * (os_factor as f64);
        self.update_rates(os_sr);

        let comps = if self.mod_settings.tonestack == ToneStackTopology::Model {
            jcm800_fmv_components(self.panel.bass, self.panel.middle, self.panel.treble)
        } else {
            spec078_fmv_components(
                self.mod_settings.tonestack,
                self.panel.bass,
                self.panel.middle,
                self.panel.treble,
            )
        };
        self.tonestack.update_components(comps, os_sr, false);

        self.power_stage.tube = self.mod_settings.power_tube;
        self.power_stage.bias_ma = self.mod_settings.bias_ma as f64;
        self.sag.set_kind(RectifierKind::from(self.mod_settings.rectifier), os_sr);
        self.xformer.sat_amount = (self.mod_settings.xformer_sat as f64) / 10.0;
        self.reactive.configure(
            self.speaker_resonance_hz as f64,
            self.panel.impedance_ohms,
            true,
            self.mod_settings.nfb as f64,
            self.panel.presence as f64,
            0.0,
        );

        let is_low = matches!(
            self.panel.input_jack,
            AmpInputJack::Low | AmpInputJack::Low1 | AmpInputJack::Low2
        );
        let gain_u = (self.panel.input_gain as f64 / 10.0).clamp(0.0, 1.0);
        let pre_gain = if gain_u <= 1e-4 {
            0.0
        } else {
            ((9.0f64.powf(gain_u) - 1.0) / 8.0) * 3.2
        };

        let master_u = (self.panel.master_volume as f64 / 10.0).clamp(0.0, 1.0);
        let master_gain = if master_u <= 1e-4 {
            0.0
        } else {
            ((9.0f64.powf(master_u) - 1.0) / 8.0) * 2.2
        };

        // Bright cap 1000 pF sobre Pre-Amp Volume
        let bright_boost = if gain_u > 1e-4 { (1.0 - gain_u).powf(1.2) * 0.75 } else { 0.0 };
        let alpha_bright = (1.0 - (-2.0 * PI * 1400.0 / os_sr).exp()).clamp(0.001, 0.99);

        let plate_v = self.mod_settings.preamp_plate_v as f64;
        let out_lin = 10.0f64.powf(self.panel.output_level_db as f64 / 20.0);
        let live_target = if self.panel.power && !self.panel.standby { 1.0 } else { 0.0 };
        let alpha_standby = (1.0 - (-1.0 / (fs_base * 0.00014)).exp()).clamp(0.001, 0.99);
        let nfb_lin = 1.0 / (1.0 + (self.mod_settings.nfb as f64 / 10.0) * 1.15);

        let len = input.len().min(output.len());
        for i in 0..len {
            self.standby_env += alpha_standby * (live_target - self.standby_env);
            if self.standby_env < 1e-9 && live_target == 0.0 {
                self.standby_env = 0.0;
            }
            let raw = if input[i].is_finite() { input[i] as f64 } else { 0.0 };

            let mut out_acc = 0.0;
            for step in 0..os_factor {
                let s = self.os.upsample_step(raw, step);
                let b_fac = self.sag.b_plus_factor();

                // Jack HIGH passa por V1a; Jack LOW bypassa V1a + pad -6.02 dB (0.5)
                let after_v1a = if is_low {
                    s * 0.5
                } else {
                    self.v1a.process(s, plate_v * b_fac)
                };

                // Pre-Amp Volume + bright cap 1000 pF
                let pre_pot = after_v1a * pre_gain;
                self.bright_hp_state = flush_f64(
                    self.bright_hp_state + alpha_bright * (pre_pot - self.bright_hp_state),
                );
                let pre_bright = pre_pot + (pre_pot - self.bright_hp_state) * bright_boost;

                // V1b Cold Clipper -> divisor analítico 470k||470pF -> V2a -> V2b CF -> FMV
                let v1b = self.v1b_cold.process(pre_bright, plate_v * b_fac);
                let div = self.divider.process(v1b) * 1.65;
                let v2a = self.v2a.process(div, plate_v * b_fac);
                let v2b = self.v2b_cf.process(v2a, b_fac);
                let after_ts = self.tonestack.process(v2b) * 1.85;

                // Master Volume real (silencia em 0.0)
                let post_master = after_ts * master_gain;
                let pi_out = self.pi.process(post_master, 0.0, b_fac);
                let power_sat = self.power_stage.process(pi_out, b_fac, nfb_lin);
                let xf_out = self.xformer.process(power_sat);
                let load_out = self.reactive.process(xf_out);
                let _ = self.sag.step(power_sat, self.mod_settings.sag as f64);

                out_acc = self.os.downsample_step(load_out * 0.42);
            }

            let final_s = out_acc * out_lin * self.standby_env;
            output[i] = if final_s.is_finite() {
                (final_s as f32).clamp(-16.0, 16.0)
            } else {
                0.0
            };
        }
    }
}

impl AudioProcessor for MarshallJcm800_2203 {
    fn descriptor(&self) -> &ModelDescriptor {
        &Self::DESCRIPTOR
    }

    fn prepare(&mut self, spec: AudioSpec) -> Result<(), DspError> {
        self.set_sample_rate(spec.sample_rate as f32);
        self.reset();
        Ok(())
    }

    fn reset(&mut self) {
        self.os.reset();
        self.v1a.reset();
        self.v1b_cold.reset();
        self.divider.reset();
        self.v2a.reset();
        self.v2b_cf.reset();
        self.tonestack.reset();
        self.pi.reset();
        self.power_stage.reset();
        self.sag.reset();
        self.xformer.reset();
        self.reactive.reset();
        self.bright_hp_state = 0.0;
    }

    fn apply_param_rt(&mut self, id: ParamId, value: ParamValue) -> Result<(), ParamError> {
        match (id, value) {
            (PID_POWER, ParamValue::Bool(b)) => self.panel.power = b,
            (PID_STANDBY, ParamValue::Bool(b)) => self.panel.standby = b,
            (PID_PRESENCE, ParamValue::Float(v)) => self.panel.set_presence(v),
            (PID_BASS, ParamValue::Float(v)) => self.panel.set_bass(v),
            (PID_MIDDLE, ParamValue::Float(v)) => self.panel.set_middle(v),
            (PID_TREBLE, ParamValue::Float(v)) => self.panel.set_treble(v),
            (PID_MASTER, ParamValue::Float(v)) => self.panel.set_master_volume(v),
            (PID_GAIN, ParamValue::Float(v)) => self.panel.set_input_gain(v),
            (PID_INPUT_JACK, ParamValue::Enum(s)) => {
                let j = AmpInputJack::from_str_name(s)
                    .ok_or(ParamError::InvalidEnumVariant { key: "amp_1.input_jack", variant: s })?;
                self.panel.input_jack = j;
            }
            (PID_IMPEDANCE_OHMS, ParamValue::Enum(s)) => {
                self.panel.impedance_ohms = match s {
                    "4" => 4,
                    "8" => 8,
                    _ => 16,
                };
            }
            (PID_OUTPUT_LEVEL_DB, ParamValue::Float(v)) => self.panel.set_output_level_db(v),
            _ => return Err(ParamError::UnknownParamId(id)),
        }
        Ok(())
    }

    fn process(
        &mut self,
        ctx: &ProcessContext,
        input: &AudioBlockRef<'_>,
        output: &mut AudioBlockMut<'_>,
    ) {
        if ctx.oversampling_factor != 0 && ctx.oversampling_factor != self.os.factor {
            self.set_oversampling_factor(ctx.oversampling_factor);
        }
        self.speaker_resonance_hz = ctx.speaker_resonance_hz;
        self.process_mono_slice(input.channel(0), output.channel_mut(0));
    }

    fn latency_samples(&self) -> usize {
        self.os.latency_samples()
    }
}
