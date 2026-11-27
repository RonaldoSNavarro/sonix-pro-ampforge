//! Spec 056: Amplificador Fender Twin Reverb AB763 (Rev. 1963–1967 Blackface, 85W)
//!
//! - V1a (`1.5k||25uF`) -> `FmvToneStackSolver` AB763 (`R_src = 38 kOhm`, Bass/Treble audio 25%, Mid linear)
//!   -> `Volume` (`gain`, silencia em `0.0`) com **`Bright 120 pF` pós-Tone Stack**
//!   -> V1b -> (exclusivo no canal `Vibrato`: `SpringReverbTank` + Opto-LDR Tremolo)
//!   -> `12AT7` LTP + `4x6L6GC` de altíssimo headroom + NFB forte, sem `Master`/`Presence`/`Depth`.

use super::{
    param_bool, param_enum, param_knob, AmpOversamplerBank, PID_BASS, PID_BRIGHT, PID_GAIN,
    PID_INPUT_JACK, PID_MIDDLE, PID_OUTPUT_LEVEL_DB, PID_POWER, PID_REVERB, PID_STANDBY,
    PID_TREBLE, PID_VIBRATO_DEPTH, PID_VIBRATO_SPEED,
};
use crate::amp::{
    nominal_mod_settings_for_model, AmpFrontPanel, AmpInputJack, AmpModelId, ChassisModSettings,
    SpringReverbTank,
};
use crate::denormal::flush_f64;
use crate::modshop::{PreampTube, ToneStackTopology};
use crate::processor::{
    AudioBlockMut, AudioBlockRef, AudioProcessor, AudioSpec, ChannelLayout, DspError,
    ModelDescriptor, ParamDescriptor, ParamError, ParamId, ParamTaper, ParamValue, ProcessContext,
};
use crate::tonestack::{spec078_fmv_components, FmvToneStackSolver};
use crate::tube::{
    OutputTransformer, OutputTransformerModel, PhaseInverterLtpStage, PowerSupplyAndRectifier,
    PowerSupplySag, PushPullPowerStage, ReactiveLoadAndNfb, RectifierKind, TriodeParams,
    TriodeStage,
};
use std::f64::consts::PI;

const TWIN_JACKS: &[&str] = &["normal", "vibrato", "normal_low", "low"];

const TWIN_PARAMS: &[ParamDescriptor] = &[
    param_bool(PID_POWER, "amp_1.power", "Power", 1.0),
    param_bool(PID_STANDBY, "amp_1.standby", "Standby", 0.0),
    param_enum(PID_INPUT_JACK, "amp_1.input_jack", "Input Jack", TWIN_JACKS),
    param_bool(PID_BRIGHT, "amp_1.bright", "Bright", 0.0),
    param_knob(PID_GAIN, "amp_1.gain", "Volume", 5.0, ParamTaper::Audio25),
    param_knob(PID_TREBLE, "amp_1.treble", "Treble", 6.0, ParamTaper::Audio25),
    param_knob(PID_MIDDLE, "amp_1.middle", "Middle", 5.0, ParamTaper::Linear),
    param_knob(PID_BASS, "amp_1.bass", "Bass", 5.5, ParamTaper::Audio25),
    param_knob(PID_REVERB, "amp_1.reverb", "Reverb", 3.0, ParamTaper::Audio25),
    param_knob(PID_VIBRATO_SPEED, "amp_1.vibrato_speed", "Speed", 4.0, ParamTaper::Linear),
    param_knob(PID_VIBRATO_DEPTH, "amp_1.vibrato_depth", "Intensity", 3.5, ParamTaper::Log10),
];

pub struct FenderTwinReverbAb763 {
    pub panel: AmpFrontPanel,
    pub mod_settings: ChassisModSettings,
    pub sample_rate: f32,
    pub speaker_resonance_hz: f32,
    pub(super) os: AmpOversamplerBank,
    v1a: TriodeStage,
    tonestack: FmvToneStackSolver,
    v1b: TriodeStage,
    spring_reverb: SpringReverbTank,
    pi: PhaseInverterLtpStage,
    power_stage: PushPullPowerStage,
    sag: PowerSupplySag,
    xformer: OutputTransformer,
    reactive: ReactiveLoadAndNfb,
    pub(super) standby_env: f64,
    bright_hp_state: f64,
    lfo_phase: f64,
    rev_feed_state: f64,
    last_os_sr: f64,
}

impl Default for FenderTwinReverbAb763 {
    fn default() -> Self {
        Self::new()
    }
}

impl FenderTwinReverbAb763 {
    pub const DESCRIPTOR: ModelDescriptor = ModelDescriptor {
        module_id: "amp.fender.twin_reverb_ab763",
        model_revision: "1965-ab763-blackface-85w",
        display_name: "Fender Twin Reverb AB763",
        layout: ChannelLayout::Mono,
        params: TWIN_PARAMS,
    };

    pub fn new() -> Self {
        let mut panel = AmpFrontPanel::default();
        panel.input_gain = 5.0;
        panel.reverb = 3.0;
        panel.vibrato_speed = 4.0;
        panel.vibrato_depth = 3.5;
        panel.input_jack = AmpInputJack::Vibrato;
        panel.impedance_ohms = 4;
        let mod_settings = nominal_mod_settings_for_model(AmpModelId::UsTwinBlackface65);
        let sr = 48_000.0f64;
        let os_sr = sr * 2.0;

        let mut s = Self {
            panel,
            mod_settings: mod_settings.clone(),
            sample_rate: sr as f32,
            speaker_resonance_hz: 90.0,
            os: AmpOversamplerBank::new(2),
            v1a: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 1_500.0,
                    ck_farads: 25e-6,
                    cc_farads: 22e-9,
                    gain_linear: 1.85,
                    asym: mod_settings.triode_asym as f64 * 0.75,
                    nominal_b_plus_v: 310.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            tonestack: FmvToneStackSolver::new(),
            v1b: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 1_500.0,
                    ck_farads: 25e-6,
                    cc_farads: 22e-9,
                    gain_linear: 1.35,
                    asym: mod_settings.triode_asym as f64 * 0.75,
                    nominal_b_plus_v: 310.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            spring_reverb: SpringReverbTank::new(sr as f32),
            pi: PhaseInverterLtpStage::new(PreampTube::At7, 0.03),
            power_stage: PushPullPowerStage::new(mod_settings.power_tube, mod_settings.bias_ma as f64),
            sag: PowerSupplySag::new(
                PowerSupplyAndRectifier::new(RectifierKind::from(mod_settings.rectifier), 440.0),
                os_sr,
            ),
            xformer: OutputTransformer::new(OutputTransformerModel::SchumacherTwin, os_sr),
            reactive: ReactiveLoadAndNfb::new(os_sr),
            standby_env: 1.0,
            bright_hp_state: 0.0,
            lfo_phase: 0.0,
            rev_feed_state: 0.0,
            last_os_sr: os_sr,
        };
        s.sync_tonestack(true);
        s
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.os.configure(self.sample_rate, self.os.factor);
        self.spring_reverb.set_sample_rate(self.sample_rate);
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
            self.v1b.update_coeffs(os_sr);
            self.sag.set_sample_rate(os_sr);
            self.xformer.set_sample_rate(os_sr);
            self.reactive.set_sample_rate(os_sr);
            self.sync_tonestack(true);
        }
    }

    fn sync_tonestack(&mut self, immediate: bool) {
        let topo = if self.mod_settings.tonestack == ToneStackTopology::Model {
            ToneStackTopology::FenderTmbMidScoop
        } else {
            self.mod_settings.tonestack
        };
        let comps = spec078_fmv_components(topo, self.panel.bass, self.panel.middle, self.panel.treble);
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

        let topo = if self.mod_settings.tonestack == ToneStackTopology::Model {
            ToneStackTopology::FenderTmbMidScoop
        } else {
            self.mod_settings.tonestack
        };
        let comps = spec078_fmv_components(topo, self.panel.bass, self.panel.middle, self.panel.treble);
        self.tonestack.update_components(comps, os_sr, false);

        self.power_stage.tube = self.mod_settings.power_tube;
        self.power_stage.bias_ma = self.mod_settings.bias_ma as f64;
        self.sag.set_kind(RectifierKind::from(self.mod_settings.rectifier), os_sr);
        self.xformer.sat_amount = (self.mod_settings.xformer_sat as f64) / 10.0;
        // Twin AB763 não possui Presence nem Depth; NFB fixo forte
        self.reactive.configure(
            self.speaker_resonance_hz as f64,
            self.panel.impedance_ohms,
            true,
            self.mod_settings.nfb as f64,
            5.0,
            0.0,
        );

        let is_low = matches!(
            self.panel.input_jack,
            AmpInputJack::Low | AmpInputJack::NormalLow | AmpInputJack::Low1 | AmpInputJack::Low2
        );
        let input_pad = if is_low { 0.50 } else { 1.0 };

        let is_vibrato_ch = !self.panel.input_jack.is_normal_channel();
        let rev_mix = if is_vibrato_ch {
            (self.panel.reverb as f64 / 10.0).clamp(0.0, 1.0) * 3.10
        } else {
            0.0
        };
        let trem_depth = if is_vibrato_ch {
            (self.panel.vibrato_depth as f64 / 10.0).clamp(0.0, 1.0) * 0.70
        } else {
            0.0
        };

        // Volume silencia em 0.0; em 5.0 entrega clean cristalino (THD < 1.5% @ -12 dBFS)
        let vol_u = (self.panel.input_gain as f64 / 10.0).clamp(0.0, 1.0);
        let vol_gain = if vol_u <= 1e-4 {
            0.0
        } else {
            ((9.0f64.powf(vol_u) - 1.0) / 8.0) * 3.8
        };

        // Bright 120 pF sobre Volume (pós-Tone Stack, pré-V1b): máximo em volume baixo/médio
        let bright_boost = if self.panel.bright_switch && vol_u > 1e-4 {
            (1.0 - vol_u).powf(1.1) * 1.35
        } else {
            0.0
        };
        let alpha_bright = (1.0 - (-2.0 * PI * 2200.0 / os_sr).exp()).clamp(0.001, 0.99);

        let lfo_hz = 1.5 + (self.panel.vibrato_speed as f64 / 10.0) * 8.0;
        let lfo_inc = (2.0 * PI * lfo_hz) / fs_base;

        // V1a/V1b headroom aumentado (~310 V) para clean cristalino
        let plate_v = self.mod_settings.preamp_plate_v as f64 * 1.45;
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
            self.lfo_phase += lfo_inc;
            if self.lfo_phase >= 2.0 * PI {
                self.lfo_phase -= 2.0 * PI;
            }
            let trem_mod = if trem_depth > 1e-4 {
                1.0 - trem_depth * (0.5 + 0.5 * self.lfo_phase.sin())
            } else {
                1.0
            };

            let rev_tap = if rev_mix > 1e-4 {
                self.spring_reverb.process_sample(self.rev_feed_state)
            } else {
                0.0
            };

            let raw = if input[i].is_finite() { input[i] as f64 } else { 0.0 };
            let base_in = raw * input_pad;

            let mut out_acc = 0.0;
            let mut rev_acc = 0.0;
            for step in 0..os_factor {
                let s = self.os.upsample_step(base_in, step);
                let b_fac = self.sag.b_plus_factor();

                // V1a -> Tone Stack AB763 (R_src = 38 kOhm) -> Volume + Bright 120 pF -> V1b
                let v1a = self.v1a.process(s, plate_v * b_fac);
                let after_ts = self.tonestack.process(v1a) * 2.2;
                let after_vol = after_ts * vol_gain;

                self.bright_hp_state = flush_f64(
                    self.bright_hp_state + alpha_bright * (after_vol - self.bright_hp_state),
                );
                let after_bright = after_vol + (after_vol - self.bright_hp_state) * bright_boost;
                let v1b = self.v1b.process(after_bright, plate_v * b_fac);

                rev_acc += v1b;
                let with_rev = (v1b + rev_tap * rev_mix) * trem_mod;

                let pi_out = self.pi.process(with_rev, 0.0, b_fac);
                let power_sat = self.power_stage.process(pi_out, b_fac, nfb_lin);
                let xf_out = self.xformer.process(power_sat);
                let load_out = self.reactive.process(xf_out);
                let _ = self.sag.step(power_sat, self.mod_settings.sag as f64);

                out_acc = self.os.downsample_step(load_out * 0.52);
            }
            self.rev_feed_state = flush_f64(rev_acc / (os_factor as f64));

            let final_s = out_acc * out_lin * self.standby_env;
            output[i] = if final_s.is_finite() {
                (final_s as f32).clamp(-16.0, 16.0)
            } else {
                0.0
            };
        }
    }
}

impl AudioProcessor for FenderTwinReverbAb763 {
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
        self.tonestack.reset();
        self.v1b.reset();
        self.spring_reverb.reset();
        self.pi.reset();
        self.power_stage.reset();
        self.sag.reset();
        self.xformer.reset();
        self.reactive.reset();
        self.bright_hp_state = 0.0;
        self.rev_feed_state = 0.0;
    }

    fn apply_param_rt(&mut self, id: ParamId, value: ParamValue) -> Result<(), ParamError> {
        match (id, value) {
            (PID_POWER, ParamValue::Bool(b)) => self.panel.power = b,
            (PID_STANDBY, ParamValue::Bool(b)) => self.panel.standby = b,
            (PID_BRIGHT, ParamValue::Bool(b)) => self.panel.bright_switch = b,
            (PID_GAIN, ParamValue::Float(v)) => self.panel.set_input_gain(v),
            (PID_TREBLE, ParamValue::Float(v)) => self.panel.set_treble(v),
            (PID_MIDDLE, ParamValue::Float(v)) => self.panel.set_middle(v),
            (PID_BASS, ParamValue::Float(v)) => self.panel.set_bass(v),
            (PID_REVERB, ParamValue::Float(v)) => self.panel.set_reverb(v),
            (PID_VIBRATO_SPEED, ParamValue::Float(v)) => self.panel.set_vibrato_speed(v),
            (PID_VIBRATO_DEPTH, ParamValue::Float(v)) => self.panel.set_vibrato_depth(v),
            (PID_INPUT_JACK, ParamValue::Enum(s)) => {
                let j = AmpInputJack::from_str_name(s)
                    .ok_or(ParamError::InvalidEnumVariant { key: "amp_1.input_jack", variant: s })?;
                self.panel.input_jack = j;
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
