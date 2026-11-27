//! Spec 058: Amplificador Vox AC30/6 Top Boost (Rev. 1964 JMI, 30W 4×EL84)
//!
//! - Canal `Normal` (`normal_high`, `normal_low` -6 dB): controlado por `amp_1.volume_2` (`NORMAL VOL`),
//!   bypassa completamente a seção Top Boost;
//! - Canal `Brilliant / Top Boost` (`brilliant_high`, `brilliant_low`, `top_boost`, `vibrato`):
//!   controlado por `amp_1.gain` (`BRILL VOL`) -> estágio extra ECC83 + `CathodeFollowerStage`
//!   -> `VoxTopBoostToneStackSolver` (`OS/010`, `R_src = 1.5 kOhm`);
//! - `Vib-Trem` (`vibrato_speed`, `vibrato_depth`, `ac30_vib_trem_mode`);
//! - Pós-LTP: `Tone Cut` diferencial (`250 kOhm + 4.7 nF`) -> `4xEL84` `CathodeBias` (`50 Ω || 250 µF`)
//!   + `GZ34` + `NFB = 0`, sem `Master`, `Middle`, `Presence` ou `Depth`.

use super::{
    param_bool, param_enum, param_knob, AmpOversamplerBank, PID_BASS, PID_CUT, PID_GAIN,
    PID_INPUT_JACK, PID_OUTPUT_LEVEL_DB, PID_POWER, PID_STANDBY, PID_TREBLE, PID_VIBRATO_DEPTH,
    PID_VIBRATO_SPEED, PID_VOLUME_2,
};
use crate::amp::{
    nominal_mod_settings_for_model, Ac30VibTremMode, AmpFrontPanel, AmpInputJack, AmpModelId,
    ChassisModSettings,
};
use crate::denormal::flush_f64;
use crate::modshop::ToneStackTopology;
use crate::processor::{
    AudioBlockMut, AudioBlockRef, AudioProcessor, AudioSpec, ChannelLayout, DspError,
    ModelDescriptor, ParamDescriptor, ParamError, ParamId, ParamTaper, ParamValue, ProcessContext,
};
use crate::tonestack::{spec078_fmv_components, FmvToneStackSolver, VoxTopBoostToneStackSolver};
use crate::tube::{
    BiasTopology, CathodeFollowerStage, OutputTransformer, OutputTransformerModel,
    PhaseInverterLtpStage, PowerSupplyAndRectifier, PowerSupplySag, PushPullPowerStage,
    ReactiveLoadAndNfb, RectifierKind, TriodeParams, TriodeStage,
};
use std::f64::consts::PI;

const AC30_JACKS: &[&str] = &["normal_high", "normal_low", "brilliant_high", "brilliant_low", "top_boost", "vibrato"];

const AC30_PARAMS: &[ParamDescriptor] = &[
    param_bool(PID_POWER, "amp_1.power", "Power", 1.0),
    param_bool(PID_STANDBY, "amp_1.standby", "Standby", 0.0),
    param_enum(PID_INPUT_JACK, "amp_1.input_jack", "Input Jack", AC30_JACKS),
    param_knob(PID_GAIN, "amp_1.gain", "Brill Vol", 5.5, ParamTaper::Audio25),
    param_knob(PID_VOLUME_2, "amp_1.volume_2", "Normal Vol", 5.0, ParamTaper::Audio25),
    param_knob(PID_TREBLE, "amp_1.treble", "Treble", 6.0, ParamTaper::Audio25),
    param_knob(PID_BASS, "amp_1.bass", "Bass", 5.5, ParamTaper::Audio25),
    param_knob(PID_CUT, "amp_1.cut", "Tone Cut", 5.0, ParamTaper::Log10),
    param_knob(PID_VIBRATO_SPEED, "amp_1.vibrato_speed", "Speed", 4.0, ParamTaper::Linear),
    param_knob(PID_VIBRATO_DEPTH, "amp_1.vibrato_depth", "Depth", 0.0, ParamTaper::Log10),
];

pub struct VoxAc30TopBoost1964 {
    pub panel: AmpFrontPanel,
    pub mod_settings: ChassisModSettings,
    pub sample_rate: f32,
    pub speaker_resonance_hz: f32,
    pub(super) os: AmpOversamplerBank,
    v1: TriodeStage,
    v2a_tb: TriodeStage,
    v2b_cf: CathodeFollowerStage,
    ts_vox: VoxTopBoostToneStackSolver,
    ts_custom: FmvToneStackSolver,
    pi: PhaseInverterLtpStage,
    power_stage: PushPullPowerStage,
    bias: BiasTopology,
    sag: PowerSupplySag,
    xformer: OutputTransformer,
    reactive: ReactiveLoadAndNfb,
    cut_lp_state: f64,
    pub(super) standby_env: f64,
    lfo_phase: f64,
    vib_buf: [f64; 512],
    vib_pos: usize,
    last_os_sr: f64,
}

impl Default for VoxAc30TopBoost1964 {
    fn default() -> Self {
        Self::new()
    }
}

impl VoxAc30TopBoost1964 {
    pub const DESCRIPTOR: ModelDescriptor = ModelDescriptor {
        module_id: "amp.vox.ac30_6_top_boost",
        model_revision: "1964-jmi-ac30-6-integrated-tb",
        display_name: "Vox AC30/6 Top Boost",
        layout: ChannelLayout::Mono,
        params: AC30_PARAMS,
    };

    pub fn new() -> Self {
        let mut panel = AmpFrontPanel::default();
        panel.input_gain = 5.5;
        panel.volume_2 = 5.0;
        panel.cut = 5.0;
        panel.input_jack = AmpInputJack::BrilliantHigh;
        panel.impedance_ohms = 16;
        let mod_settings = nominal_mod_settings_for_model(AmpModelId::UkAc30TopBoost);
        let sr = 48_000.0f64;
        let os_sr = sr * 2.0;

        let mut s = Self {
            panel,
            mod_settings: mod_settings.clone(),
            sample_rate: sr as f32,
            speaker_resonance_hz: 82.0,
            os: AmpOversamplerBank::new(2),
            v1: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 1_500.0,
                    ck_farads: 25e-6,
                    cc_farads: 22e-9,
                    gain_linear: 2.0,
                    asym: mod_settings.triode_asym as f64,
                    nominal_b_plus_v: 250.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            v2a_tb: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 1_500.0,
                    ck_farads: 25e-6,
                    cc_farads: 22e-9,
                    gain_linear: 1.9,
                    asym: mod_settings.triode_asym as f64 * 1.1,
                    nominal_b_plus_v: 250.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            v2b_cf: CathodeFollowerStage::new(0.94, 1_500.0, 1.2),
            ts_vox: VoxTopBoostToneStackSolver::new(),
            ts_custom: FmvToneStackSolver::new(),
            pi: PhaseInverterLtpStage::default(),
            power_stage: PushPullPowerStage::new(mod_settings.power_tube, mod_settings.bias_ma as f64),
            bias: BiasTopology::cathode_biased(50.0, 250e-6, os_sr),
            sag: PowerSupplySag::new(
                PowerSupplyAndRectifier::new(RectifierKind::from(mod_settings.rectifier), 340.0),
                os_sr,
            ),
            xformer: OutputTransformer::new(OutputTransformerModel::WodenAc30, os_sr),
            reactive: ReactiveLoadAndNfb::new(os_sr),
            cut_lp_state: 0.0,
            standby_env: 1.0,
            lfo_phase: 0.0,
            vib_buf: [0.0; 512],
            vib_pos: 0,
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
            self.v1.update_coeffs(os_sr);
            self.v2a_tb.update_coeffs(os_sr);
            self.bias.set_sample_rate(os_sr);
            self.sag.set_sample_rate(os_sr);
            self.xformer.set_sample_rate(os_sr);
            self.reactive.set_sample_rate(os_sr);
            self.sync_tonestack(true);
        }
    }

    fn sync_tonestack(&mut self, immediate: bool) {
        self.ts_vox
            .update(self.panel.bass, self.panel.treble, self.last_os_sr, immediate);
        if self.mod_settings.tonestack != ToneStackTopology::Model {
            let comps = spec078_fmv_components(
                self.mod_settings.tonestack,
                self.panel.bass,
                self.panel.middle,
                self.panel.treble,
            );
            self.ts_custom.update_components(comps, self.last_os_sr, immediate);
        }
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

        self.ts_vox
            .update(self.panel.bass, self.panel.treble, os_sr, false);
        let use_custom_ts = self.mod_settings.tonestack != ToneStackTopology::Model
            && self.mod_settings.tonestack != ToneStackTopology::VoxTopBoost;
        if use_custom_ts {
            let comps = spec078_fmv_components(
                self.mod_settings.tonestack,
                self.panel.bass,
                self.panel.middle,
                self.panel.treble,
            );
            self.ts_custom.update_components(comps, os_sr, false);
        }

        self.power_stage.tube = self.mod_settings.power_tube;
        self.power_stage.bias_ma = self.mod_settings.bias_ma as f64;
        self.sag.set_kind(RectifierKind::from(self.mod_settings.rectifier), os_sr);
        self.xformer.sat_amount = (self.mod_settings.xformer_sat as f64) / 10.0;
        // AC30 1964 não tem NFB global (nfb_enabled = false)
        self.reactive.configure(
            self.speaker_resonance_hz as f64,
            self.panel.impedance_ohms,
            false,
            0.0,
            5.0,
            0.0,
        );

        let is_normal = self.panel.input_jack.is_normal_channel();
        let is_low = self.panel.input_jack.is_low_pad();
        let input_pad = if is_low { 0.50 } else { 1.0 };

        // Normal channel usa volume_2 (NORMAL VOL); Brilliant/Top Boost usa gain (BRILL VOL)
        let active_vol = if is_normal {
            self.panel.volume_2
        } else {
            self.panel.input_gain
        };
        let vol_u = (active_vol as f64 / 10.0).clamp(0.0, 1.0);
        let vol_gain = if vol_u <= 1e-4 {
            0.0
        } else {
            ((9.0f64.powf(vol_u) - 1.0) / 8.0) * 3.0
        };

        // Tone Cut pós-LTP: 18 kHz (cut=0) -> 750 Hz (cut=10)
        let fc_cut_hz = 18000.0 * (750.0f64 / 18000.0).powf((self.panel.cut as f64 / 10.0).clamp(0.0, 1.0));
        let alpha_cut = (1.0 - (-2.0 * PI * fc_cut_hz / os_sr).exp()).clamp(0.005, 0.99);

        let is_vibtrem = self.panel.input_jack == AmpInputJack::Vibrato && self.panel.vibrato_depth > 0.01;
        let is_trem = is_vibtrem && self.panel.ac30_vib_trem_mode == Ac30VibTremMode::Tremolo;
        let is_vib = is_vibtrem && self.panel.ac30_vib_trem_mode == Ac30VibTremMode::Vibrato;
        let lfo_hz = 1.5 + (self.panel.vibrato_speed as f64 / 10.0) * 8.0;
        let lfo_inc = (2.0 * PI * lfo_hz) / fs_base;
        let depth_norm = (self.panel.vibrato_depth as f64 / 10.0).clamp(0.0, 1.0);

        let plate_v = self.mod_settings.preamp_plate_v as f64;
        let out_lin = 10.0f64.powf(self.panel.output_level_db as f64 / 20.0);
        let live_target = if self.panel.power && !self.panel.standby { 1.0 } else { 0.0 };
        let alpha_standby = (1.0 - (-1.0 / (fs_base * 0.00014)).exp()).clamp(0.001, 0.99);

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
            let trem_mod = if is_trem {
                1.0 - (depth_norm * 0.70) * (0.5 + 0.5 * self.lfo_phase.sin())
            } else {
                1.0
            };

            let raw = if input[i].is_finite() { input[i] as f64 } else { 0.0 };
            let base_in = raw * input_pad;

            let mut out_acc = 0.0;
            for step in 0..os_factor {
                let s = self.os.upsample_step(base_in, step);
                let b_fac = self.sag.b_plus_factor();

                let v1_out = self.v1.process(s * vol_gain, plate_v * b_fac);
                let pre_out = if is_normal && !use_custom_ts {
                    // Canal Normal bypassa completamente o circuito Top Boost
                    v1_out
                } else {
                    let v2a = self.v2a_tb.process(v1_out, plate_v * b_fac);
                    let v2b = self.v2b_cf.process(v2a, b_fac);
                    let ts_out = if use_custom_ts {
                        self.ts_custom.process(v2b)
                    } else {
                        self.ts_vox.process(v2b)
                    };
                    ts_out * 2.0
                };

                let pi_out = self.pi.process(pre_out * trem_mod, 0.0, b_fac);

                // Tone Cut diferencial pós-LTP (ativo em ambos os canais!)
                self.cut_lp_state = flush_f64(self.cut_lp_state + alpha_cut * (pi_out - self.cut_lp_state));
                let post_cut = self.cut_lp_state;

                let cathode_headroom = self.bias.process_excursion(post_cut);
                let power_sat = self.power_stage.process(post_cut * 1.45, b_fac * cathode_headroom, 1.0);
                let xf_out = self.xformer.process(power_sat);
                let load_out = self.reactive.process(xf_out);
                let _ = self.sag.step(power_sat, self.mod_settings.sag as f64);

                out_acc = self.os.downsample_step(load_out * 0.42);
            }

            if is_vib {
                self.vib_buf[self.vib_pos] = flush_f64(out_acc);
                let d_ms = 1.5 + 0.5 * depth_norm * self.lfo_phase.sin();
                let d_samp = (d_ms * 0.001 * fs_base).clamp(0.0, 510.0);
                let r_pos = (self.vib_pos as f64 + 512.0 - d_samp) % 512.0;
                let idx0 = (r_pos.floor() as usize) & 511;
                let idx1 = (idx0 + 1) & 511;
                let frac = r_pos - r_pos.floor();
                out_acc = self.vib_buf[idx0] * (1.0 - frac) + self.vib_buf[idx1] * frac;
                self.vib_pos = (self.vib_pos + 1) & 511;
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

impl AudioProcessor for VoxAc30TopBoost1964 {
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
        self.v1.reset();
        self.v2a_tb.reset();
        self.v2b_cf.reset();
        self.ts_vox.reset();
        self.ts_custom.reset();
        self.pi.reset();
        self.power_stage.reset();
        self.bias.reset();
        self.sag.reset();
        self.xformer.reset();
        self.reactive.reset();
        self.cut_lp_state = 0.0;
        self.vib_buf.fill(0.0);
        self.vib_pos = 0;
    }

    fn apply_param_rt(&mut self, id: ParamId, value: ParamValue) -> Result<(), ParamError> {
        match (id, value) {
            (PID_POWER, ParamValue::Bool(b)) => self.panel.power = b,
            (PID_STANDBY, ParamValue::Bool(b)) => self.panel.standby = b,
            (PID_GAIN, ParamValue::Float(v)) => self.panel.set_input_gain(v),
            (PID_VOLUME_2, ParamValue::Float(v)) => self.panel.set_volume_2(v),
            (PID_TREBLE, ParamValue::Float(v)) => self.panel.set_treble(v),
            (PID_BASS, ParamValue::Float(v)) => self.panel.set_bass(v),
            (PID_CUT, ParamValue::Float(v)) => self.panel.set_cut(v),
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
