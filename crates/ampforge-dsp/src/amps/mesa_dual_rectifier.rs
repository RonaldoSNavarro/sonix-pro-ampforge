//! Spec 057: Amplificador Mesa/Boogie Dual Rectifier Solo Head (Rev. G, 1994–2000)
//!
//! - 4 modos (`Clean`, `Pushed`, `Vintage`, `Modern`) com crossfade de 10 ms na troca;
//! - Estágios com shelves de cátodo reais + HPFs inter-estágio firmes (150–250 Hz em Vintage/Modern)
//!   + Cold Clipper `39 kOhm` + `CathodeFollowerStage` + `MesaRectifierToneStackSolver` Rev G;
//! - **`Presence` ativo fora da malha NFB no modo `Modern`** (rede passiva pré-PI `25 kOhm + shunt 3.9 nF`
//!   entregando >= +6 dB @ 4–5 kHz enquanto `eff_nfb = 0` preserva o pico de ressonância em 98 Hz);
//! - Variac `Bold / Spongy` (`-20% B+`), `Silicon Diode` vs `Dual 5U4G`, `Master` silenciando em `0.0`.

use super::{
    param_bool, param_enum, param_knob, AmpOversamplerBank, PID_BASS, PID_CHANNEL_MODE, PID_GAIN,
    PID_MASTER, PID_MIDDLE, PID_OUTPUT_LEVEL_DB, PID_POWER, PID_PRESENCE, PID_SPONGY, PID_STANDBY,
    PID_TREBLE,
};
use crate::amp::{
    nominal_mod_settings_for_model, AmpChannelMode, AmpFrontPanel, AmpModelId, ChassisModSettings,
};
use crate::biquad::BiquadFilter;
use crate::denormal::flush_f64;
use crate::modshop::ToneStackTopology;
use crate::processor::{
    AudioBlockMut, AudioBlockRef, AudioProcessor, AudioSpec, ChannelLayout, DspError,
    ModelDescriptor, ParamDescriptor, ParamError, ParamId, ParamTaper, ParamValue, ProcessContext,
};
use crate::tonestack::{spec078_fmv_components, FmvToneStackSolver, MesaRectifierToneStackSolver};
use crate::tube::{
    CathodeFollowerStage, OutputTransformer, OutputTransformerModel, PhaseInverterLtpStage,
    PowerSupplyAndRectifier, PowerSupplySag, PushPullPowerStage, ReactiveLoadAndNfb, RectifierKind,
    TriodeParams, TriodeStage,
};
use std::f64::consts::PI;

const RECTO_MODES: &[&str] = &["clean", "pushed", "vintage", "modern"];

const RECTO_PARAMS: &[ParamDescriptor] = &[
    param_bool(PID_POWER, "amp_1.power", "Power", 1.0),
    param_bool(PID_STANDBY, "amp_1.standby", "Standby", 0.0),
    param_enum(PID_CHANNEL_MODE, "amp_1.channel_mode", "Channel Mode", RECTO_MODES),
    param_knob(PID_GAIN, "amp_1.gain", "Gain", 6.5, ParamTaper::Audio25),
    param_knob(PID_TREBLE, "amp_1.treble", "Treble", 6.0, ParamTaper::Audio25),
    param_knob(PID_MIDDLE, "amp_1.middle", "Mid", 5.0, ParamTaper::Linear),
    param_knob(PID_BASS, "amp_1.bass", "Bass", 5.5, ParamTaper::Audio25),
    param_knob(PID_PRESENCE, "amp_1.presence", "Presence", 5.0, ParamTaper::Audio25),
    param_knob(PID_MASTER, "amp_1.master", "Master", 5.0, ParamTaper::Audio25),
    param_bool(PID_SPONGY, "amp_1.spongy", "Spongy", 0.0),
];

pub struct MesaDualRectifierRevG {
    pub panel: AmpFrontPanel,
    pub mod_settings: ChassisModSettings,
    pub sample_rate: f32,
    pub speaker_resonance_hz: f32,
    pub(super) os: AmpOversamplerBank,
    s1: TriodeStage,
    s2: TriodeStage,
    s3_cold: TriodeStage,
    s4: TriodeStage,
    cf: CathodeFollowerStage,
    ts_mesa: MesaRectifierToneStackSolver,
    ts_custom: FmvToneStackSolver,
    modern_pres_filter: BiquadFilter,
    pi: PhaseInverterLtpStage,
    power_stage: PushPullPowerStage,
    sag: PowerSupplySag,
    xformer: OutputTransformer,
    reactive: ReactiveLoadAndNfb,
    hp1_state: f64,
    hp1_x1: f64,
    hp2_state: f64,
    hp2_x1: f64,
    pub(super) standby_env: f64,
    last_os_sr: f64,
    active_mode: AmpChannelMode,
    prev_mode: AmpChannelMode,
    mode_xfade_rem: usize,
    mode_xfade_total: usize,
}

impl Default for MesaDualRectifierRevG {
    fn default() -> Self {
        Self::new()
    }
}

impl MesaDualRectifierRevG {
    pub const DESCRIPTOR: ModelDescriptor = ModelDescriptor {
        module_id: "amp.mesa.dual_rectifier_rev_g",
        model_revision: "1994-rev-g-2ch-solo-100",
        display_name: "Mesa/Boogie Dual Rectifier Rev G",
        layout: ChannelLayout::Mono,
        params: RECTO_PARAMS,
    };

    pub fn new() -> Self {
        let mut panel = AmpFrontPanel::default();
        panel.input_gain = 6.5;
        panel.master_volume = 5.0;
        panel.channel_mode = AmpChannelMode::Modern;
        let mod_settings = nominal_mod_settings_for_model(AmpModelId::CaliDualRectoRevF);
        let sr = 48_000.0f64;
        let os_sr = sr * 2.0;

        let mut s = Self {
            panel,
            mod_settings: mod_settings.clone(),
            sample_rate: sr as f32,
            speaker_resonance_hz: 108.0,
            os: AmpOversamplerBank::new(2),
            s1: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 1_800.0,
                    ck_farads: 1.0e-6, // tight sub-bass shelf antes do clipping
                    cc_farads: 22e-9,
                    gain_linear: 2.1,
                    asym: mod_settings.triode_asym as f64,
                    nominal_b_plus_v: 290.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            s2: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 1_500.0,
                    ck_farads: 0.68e-6,
                    cc_farads: 22e-9,
                    gain_linear: 2.3,
                    asym: mod_settings.triode_asym as f64 * 1.15,
                    nominal_b_plus_v: 290.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            s3_cold: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 39_000.0,
                    ck_farads: 0.0, // Cold clipper 39 kOhm sem bypass
                    cc_farads: 22e-9,
                    gain_linear: 2.4,
                    asym: mod_settings.triode_asym as f64 * 1.6,
                    nominal_b_plus_v: 290.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            s4: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 1_800.0,
                    ck_farads: 1.0e-6,
                    cc_farads: 22e-9,
                    gain_linear: 2.0,
                    asym: mod_settings.triode_asym as f64 * 1.1,
                    nominal_b_plus_v: 290.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            cf: CathodeFollowerStage::new(0.94, 1_300.0, 1.25),
            ts_mesa: MesaRectifierToneStackSolver::new(),
            ts_custom: FmvToneStackSolver::new(),
            modern_pres_filter: BiquadFilter::identity(),
            pi: PhaseInverterLtpStage::default(),
            power_stage: PushPullPowerStage::new(mod_settings.power_tube, mod_settings.bias_ma as f64),
            sag: PowerSupplySag::new(
                PowerSupplyAndRectifier::new(RectifierKind::from(mod_settings.rectifier), 470.0),
                os_sr,
            ),
            xformer: OutputTransformer::new(OutputTransformerModel::Mesa562003, os_sr),
            reactive: ReactiveLoadAndNfb::new(os_sr),
            hp1_state: 0.0,
            hp1_x1: 0.0,
            hp2_state: 0.0,
            hp2_x1: 0.0,
            standby_env: 1.0,
            last_os_sr: os_sr,
            active_mode: AmpChannelMode::Modern,
            prev_mode: AmpChannelMode::Modern,
            mode_xfade_rem: 0,
            mode_xfade_total: 480,
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
            self.s1.update_coeffs(os_sr);
            self.s2.update_coeffs(os_sr);
            self.s3_cold.update_coeffs(os_sr);
            self.s4.update_coeffs(os_sr);
            self.sag.set_sample_rate(os_sr);
            self.xformer.set_sample_rate(os_sr);
            self.reactive.set_sample_rate(os_sr);
            self.sync_tonestack(true);
        }
    }

    fn sync_tonestack(&mut self, immediate: bool) {
        self.ts_mesa.update(
            self.panel.bass,
            self.panel.middle,
            self.panel.treble,
            self.last_os_sr,
            immediate,
        );
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

        if self.panel.channel_mode != self.active_mode {
            self.prev_mode = self.active_mode;
            self.active_mode = self.panel.channel_mode;
            self.mode_xfade_total = (fs_base * 0.010).round().max(1.0) as usize;
            self.mode_xfade_rem = self.mode_xfade_total;
        }

        self.ts_mesa.update(
            self.panel.bass,
            self.panel.middle,
            self.panel.treble,
            os_sr,
            false,
        );
        let use_custom_ts = self.mod_settings.tonestack != ToneStackTopology::Model
            && self.mod_settings.tonestack != ToneStackTopology::RectoPostGain;
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
        self.sag.supply.variac_scale = if self.panel.spongy { 0.80 } else { 1.0 };
        self.xformer.sat_amount = (self.mod_settings.xformer_sat as f64) / 10.0;

        let is_modern = self.active_mode == AmpChannelMode::Modern;
        let nfb_enabled = !is_modern;
        self.reactive.configure(
            self.speaker_resonance_hz as f64,
            self.panel.impedance_ohms,
            nfb_enabled,
            self.mod_settings.nfb as f64,
            if is_modern { 5.0 } else { self.panel.presence as f64 },
            0.0,
        );

        // No modo Modern, Presence atua como rede passiva pré-PI (25 kOhm + shunt 3.9 nF) fora da malha NFB:
        // presença em 10 eleva ~+8 dB em 4-5 kHz em relação a 0.0 (Spec 057 CA-7).
        if is_modern {
            let pres_u = (self.panel.presence as f64 / 10.0).clamp(0.0, 1.0);
            let pres_db = (pres_u - 0.5) * 9.0;
            self.modern_pres_filter
                .update_coeffs(BiquadFilter::high_shelf(2_900.0, pres_db, os_sr));
        }

        let gain_u = (self.panel.input_gain as f64 / 10.0).clamp(0.0, 1.0);
        let gain_drive = if gain_u <= 1e-4 {
            0.0
        } else {
            ((9.0f64.powf(gain_u) - 1.0) / 8.0) * 2.6
        };
        let master_u = (self.panel.master_volume as f64 / 10.0).clamp(0.0, 1.0);
        let master_gain = if master_u <= 1e-4 {
            0.0
        } else {
            ((9.0f64.powf(master_u) - 1.0) / 8.0) * 2.1
        };

        let mode_params = |m: AmpChannelMode| -> (f64, usize, f64) {
            match m {
                AmpChannelMode::Clean => (0.65, 2usize, 45.0),
                AmpChannelMode::Vintage => (2.15, 4usize, 180.0),
                AmpChannelMode::Modern => (3.10, 4usize, 220.0),
            }
        };
        let (mode_drive, n_stages, hp_fc) = mode_params(self.active_mode);
        let (prev_drive, prev_stages, _) = mode_params(self.prev_mode);
        let alpha_hp = (1.0 / (1.0 + 2.0 * PI * hp_fc / os_sr)).clamp(0.85, 0.9999);

        let plate_v = self.mod_settings.preamp_plate_v as f64;
        let out_lin = 10.0f64.powf(self.panel.output_level_db as f64 / 20.0);
        let live_target = if self.panel.power && !self.panel.standby { 1.0 } else { 0.0 };
        let alpha_standby = (1.0 - (-1.0 / (fs_base * 0.00014)).exp()).clamp(0.001, 0.99);
        let eff_nfb = if nfb_enabled { self.mod_settings.nfb as f64 } else { 0.0 };
        let nfb_lin = 1.0 / (1.0 + (eff_nfb / 10.0) * 1.15);

        let len = input.len().min(output.len());
        for i in 0..len {
            self.standby_env += alpha_standby * (live_target - self.standby_env);
            if self.standby_env < 1e-9 && live_target == 0.0 {
                self.standby_env = 0.0;
            }
            let xfade_w = if self.mode_xfade_rem > 0 {
                self.mode_xfade_rem -= 1;
                let p = 1.0 - (self.mode_xfade_rem as f64 / self.mode_xfade_total.max(1) as f64);
                0.5 * (1.0 - (p.clamp(0.0, 1.0) * PI).cos())
            } else {
                1.0
            };
            let eff_drive = if xfade_w < 1.0 {
                prev_drive * (1.0 - xfade_w) + mode_drive * xfade_w
            } else {
                mode_drive
            };
            let need_4stage = n_stages > 2 || (xfade_w < 1.0 && prev_stages > 2);

            let raw = if input[i].is_finite() { input[i] as f64 } else { 0.0 };
            let mut out_acc = 0.0;
            for step in 0..os_factor {
                let s = self.os.upsample_step(raw, step);
                let b_fac = self.sag.b_plus_factor();

                let g1 = s * gain_drive * eff_drive;
                let x1 = self.s1.process(g1, plate_v * b_fac);
                let hp1 = flush_f64(alpha_hp * (self.hp1_state + x1 - self.hp1_x1));
                self.hp1_x1 = flush_f64(x1);
                self.hp1_state = hp1;

                let x2 = self.s2.process(hp1, plate_v * b_fac);
                let x4 = if need_4stage {
                    let hp2 = flush_f64(alpha_hp * (self.hp2_state + x2 - self.hp2_x1));
                    self.hp2_x1 = flush_f64(x2);
                    self.hp2_state = hp2;
                    let x3 = self.s3_cold.process(hp2, plate_v * b_fac);
                    self.s4.process(x3, plate_v * b_fac)
                } else {
                    x2
                };
                let pre_out = if xfade_w < 1.0 {
                    let p_sig = if prev_stages <= 2 { x2 } else { x4 };
                    let n_sig = if n_stages <= 2 { x2 } else { x4 };
                    p_sig * (1.0 - xfade_w) + n_sig * xfade_w
                } else if n_stages <= 2 {
                    x2
                } else {
                    x4
                };

                let cf_out = self.cf.process(pre_out, b_fac);
                let after_ts = if use_custom_ts {
                    self.ts_custom.process(cf_out)
                } else {
                    self.ts_mesa.process(cf_out)
                } * 1.95;

                let post_pres = if is_modern {
                    self.modern_pres_filter.process(after_ts)
                } else {
                    after_ts
                };

                let post_master = post_pres * master_gain;
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

impl AudioProcessor for MesaDualRectifierRevG {
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
        self.s1.reset();
        self.s2.reset();
        self.s3_cold.reset();
        self.s4.reset();
        self.cf.reset();
        self.ts_mesa.reset();
        self.ts_custom.reset();
        self.pi.reset();
        self.power_stage.reset();
        self.sag.reset();
        self.xformer.reset();
        self.reactive.reset();
        self.hp1_state = 0.0;
        self.hp1_x1 = 0.0;
        self.hp2_state = 0.0;
        self.hp2_x1 = 0.0;
    }

    fn apply_param_rt(&mut self, id: ParamId, value: ParamValue) -> Result<(), ParamError> {
        match (id, value) {
            (PID_POWER, ParamValue::Bool(b)) => self.panel.power = b,
            (PID_STANDBY, ParamValue::Bool(b)) => self.panel.standby = b,
            (PID_CHANNEL_MODE, ParamValue::Enum(s)) => {
                self.panel.channel_mode = match s {
                    "clean" => AmpChannelMode::Clean,
                    "pushed" | "vintage" => AmpChannelMode::Vintage,
                    "modern" => AmpChannelMode::Modern,
                    _ => {
                        return Err(ParamError::InvalidEnumVariant {
                            key: "amp_1.channel_mode",
                            variant: s,
                        })
                    }
                };
            }
            (PID_GAIN, ParamValue::Float(v)) => self.panel.set_input_gain(v),
            (PID_TREBLE, ParamValue::Float(v)) => self.panel.set_treble(v),
            (PID_MIDDLE, ParamValue::Float(v)) => self.panel.set_middle(v),
            (PID_BASS, ParamValue::Float(v)) => self.panel.set_bass(v),
            (PID_PRESENCE, ParamValue::Float(v)) => self.panel.set_presence(v),
            (PID_MASTER, ParamValue::Float(v)) => self.panel.set_master_volume(v),
            (PID_SPONGY, ParamValue::Bool(b)) => self.panel.spongy = b,
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
