//! Spec 054: Amplificador Marshall 1959 Super Lead / Plexi (Rev. 1969 SL/A)
//!
//! - Canais paralelos independentes: Ch I High Treble (V1b `2.7k||0.68uF`, `2.2nF`,
//!   `Volume I` com **bright cap `5000 pF` isolado apenas no Canal I**) e Ch II Normal
//!   (V1a `820||330uF`, `22nF`, `Volume II` sem bright cap);
//! - `Jumpered`: soma paralela Ch I + Ch II -> V2a (`820||0.68uF`) + `CathodeFollowerStage` V2b;
//! - Tone Stack FMV (`Mid` 10% log, `Bass` 10% log, `Treble` linear, `R_src = 1.3 kOhm`);
//! - LTP + `4xEL34` + `OutputTransformer::DagnallC1998` + `ReactiveLoadAndNfb` (`4 Ω` mais ressonante que `16 Ω`);
//! - Sem `Master Volume`.

use super::{
    param_bool, param_enum, param_knob, AmpOversamplerBank, PID_BASS, PID_GAIN,
    PID_IMPEDANCE_OHMS, PID_INPUT_JACK, PID_JUMPERED, PID_MIDDLE, PID_OUTPUT_LEVEL_DB, PID_POWER,
    PID_PRESENCE, PID_STANDBY, PID_TREBLE, PID_VOLUME_2,
};
use crate::amp::{AmpFrontPanel, AmpInputJack, AmpModelId, ChassisModSettings, nominal_mod_settings_for_model};
use crate::denormal::flush_f64;
use crate::modshop::ToneStackTopology;
use crate::processor::{
    AudioBlockMut, AudioBlockRef, AudioProcessor, AudioSpec, ChannelLayout, DspError,
    ModelDescriptor, ParamDescriptor, ParamError, ParamId, ParamTaper, ParamValue, ProcessContext,
};
use crate::tonestack::{spec078_fmv_components, FmvToneStackSolver};
use crate::tube::{
    BiasTopology, CathodeFollowerStage, OutputTransformer, OutputTransformerModel,
    PhaseInverterLtpStage, PowerSupplyAndRectifier, PowerSupplySag, PushPullPowerStage,
    ReactiveLoadAndNfb, RectifierKind, TriodeParams, TriodeStage,
};

const PLEXI_JACKS: &[&str] = &["high1", "low1", "high2", "low2", "jumpered"];
const PLEXI_IMPEDANCES: &[&str] = &["4", "8", "16"];

const PLEXI_PARAMS: &[ParamDescriptor] = &[
    param_bool(PID_POWER, "amp_1.power", "Power", 1.0),
    param_bool(PID_STANDBY, "amp_1.standby", "Standby", 0.0),
    param_knob(PID_PRESENCE, "amp_1.presence", "Presence", 5.0, ParamTaper::Log10),
    param_knob(PID_BASS, "amp_1.bass", "Bass", 5.5, ParamTaper::Log10),
    param_knob(PID_MIDDLE, "amp_1.middle", "Middle", 5.0, ParamTaper::Log10),
    param_knob(PID_TREBLE, "amp_1.treble", "Treble", 6.0, ParamTaper::Linear),
    param_knob(PID_GAIN, "amp_1.gain", "Volume I", 6.5, ParamTaper::Audio25),
    param_knob(PID_VOLUME_2, "amp_1.volume_2", "Volume II", 4.5, ParamTaper::Audio25),
    param_enum(PID_INPUT_JACK, "amp_1.input_jack", "Input Jack", PLEXI_JACKS),
    param_bool(PID_JUMPERED, "amp_1.jumpered", "Jumpered", 0.0),
    param_enum(PID_IMPEDANCE_OHMS, "amp_1.impedance_ohms", "Impedance", PLEXI_IMPEDANCES),
];

pub struct Marshall1959SuperLead {
    pub panel: AmpFrontPanel,
    pub mod_settings: ChassisModSettings,
    pub sample_rate: f32,
    pub speaker_resonance_hz: f32,
    pub(super) os: AmpOversamplerBank,
    v1b_ch1: TriodeStage,
    v1a_ch2: TriodeStage,
    v2a: TriodeStage,
    v2b_cf: CathodeFollowerStage,
    tonestack: FmvToneStackSolver,
    pi: PhaseInverterLtpStage,
    power_stage: PushPullPowerStage,
    bias: BiasTopology,
    sag: PowerSupplySag,
    xformer: OutputTransformer,
    reactive: ReactiveLoadAndNfb,
    pub(super) standby_env: f64,
    bright_ch1_hp_state: f64,
    last_os_sr: f64,
}

impl Default for Marshall1959SuperLead {
    fn default() -> Self {
        Self::new()
    }
}

impl Marshall1959SuperLead {
    pub const DESCRIPTOR: ModelDescriptor = ModelDescriptor {
        module_id: "amp.marshall.1959_super_lead",
        model_revision: "1969-jmp-sl-a-pre-july",
        display_name: "Marshall 1959 Super Lead (Plexi)",
        layout: ChannelLayout::Mono,
        params: PLEXI_PARAMS,
    };

    pub fn new() -> Self {
        let mut panel = AmpFrontPanel::default();
        panel.input_gain = 6.5;
        panel.volume_2 = 4.5;
        panel.input_jack = AmpInputJack::High1;
        panel.impedance_ohms = 16;
        let mod_settings = nominal_mod_settings_for_model(AmpModelId::BritPlexi1959);
        let sr = 48_000.0f64;
        let os_sr = sr * 2.0;

        let mut s = Self {
            panel,
            mod_settings: mod_settings.clone(),
            sample_rate: sr as f32,
            speaker_resonance_hz: 98.0,
            os: AmpOversamplerBank::new(2),
            v1b_ch1: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 2_700.0,
                    ck_farads: 0.68e-6,
                    cc_farads: 2.2e-9,
                    gain_linear: 2.2,
                    asym: mod_settings.triode_asym as f64,
                    nominal_b_plus_v: 250.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            v1a_ch2: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 820.0,
                    ck_farads: 330e-6,
                    cc_farads: 22e-9,
                    gain_linear: 1.9,
                    asym: mod_settings.triode_asym as f64 * 0.9,
                    nominal_b_plus_v: 250.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            v2a: TriodeStage::new(
                TriodeParams {
                    rk_ohms: 820.0,
                    ck_farads: 0.68e-6,
                    cc_farads: 22e-9,
                    gain_linear: 2.4,
                    asym: mod_settings.triode_asym as f64,
                    nominal_b_plus_v: 250.0,
                    ..TriodeParams::default()
                },
                os_sr,
            ),
            v2b_cf: CathodeFollowerStage::new(0.94, 1_300.0, 1.25),
            tonestack: FmvToneStackSolver::new(),
            pi: PhaseInverterLtpStage::default(),
            power_stage: PushPullPowerStage::new(mod_settings.power_tube, mod_settings.bias_ma as f64),
            bias: BiasTopology::fixed(),
            sag: PowerSupplySag::new(
                PowerSupplyAndRectifier::new(RectifierKind::from(mod_settings.rectifier), 460.0),
                os_sr,
            ),
            xformer: OutputTransformer::new(OutputTransformerModel::DagnallC1998, os_sr),
            reactive: ReactiveLoadAndNfb::new(os_sr),
            standby_env: 1.0,
            bright_ch1_hp_state: 0.0,
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
            self.v1b_ch1.update_coeffs(os_sr);
            self.v1a_ch2.update_coeffs(os_sr);
            self.v2a.update_coeffs(os_sr);
            self.sag.set_sample_rate(os_sr);
            self.xformer.set_sample_rate(os_sr);
            self.reactive.set_sample_rate(os_sr);
            self.sync_tonestack(true);
        }
    }

    fn sync_tonestack(&mut self, immediate: bool) {
        let topo = if self.mod_settings.tonestack == ToneStackTopology::Model {
            ToneStackTopology::MarshallCathodeFollower
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

        let comps = {
            let topo = if self.mod_settings.tonestack == ToneStackTopology::Model {
                ToneStackTopology::MarshallCathodeFollower
            } else {
                self.mod_settings.tonestack
            };
            spec078_fmv_components(topo, self.panel.bass, self.panel.middle, self.panel.treble)
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

        let is_low = matches!(self.panel.input_jack, AmpInputJack::Low1 | AmpInputJack::Low2 | AmpInputJack::Low);
        let input_pad = if is_low { 0.50 } else { 1.0 }; // -6.02 dB exato (Tonal Note)

        let ch1_on = self.panel.jumpered
            || matches!(
                self.panel.input_jack,
                AmpInputJack::High | AmpInputJack::Low | AmpInputJack::High1 | AmpInputJack::Low1 | AmpInputJack::Jumpered
            );
        let ch2_on = self.panel.jumpered
            || matches!(
                self.panel.input_jack,
                AmpInputJack::High2 | AmpInputJack::Low2 | AmpInputJack::Jumpered
            );

        // Volume I e II silenciam em 0.0; curva audio log para crunch dinâmico
        let vol1_u = (self.panel.input_gain as f64 / 10.0).clamp(0.0, 1.0);
        let vol1_gain = if vol1_u <= 1e-4 {
            0.0
        } else {
            ((9.0f64.powf(vol1_u) - 1.0) / 8.0) * 2.8
        };
        let vol2_u = (self.panel.volume_2 as f64 / 10.0).clamp(0.0, 1.0);
        let vol2_gain = if vol2_u <= 1e-4 {
            0.0
        } else {
            ((9.0f64.powf(vol2_u) - 1.0) / 8.0) * 2.4
        };

        // Bright cap 5000 pF sobre Volume I: máximo em Volume baixo, desaparece perto de 10
        let bright_ch1_boost = if ch1_on && vol1_u > 1e-4 {
            (1.0 - vol1_u).powf(1.3) * 1.45
        } else {
            0.0
        };
        let alpha_bright = (1.0 - (-2.0 * std::f64::consts::PI * 1100.0 / os_sr).exp()).clamp(0.001, 0.99);

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
            let base_in = raw * input_pad;

            let mut out_acc = 0.0;
            for step in 0..os_factor {
                let s = self.os.upsample_step(base_in, step);
                let b_fac = self.sag.b_plus_factor();

                // Canal I (High Treble) com bright cap 5000 pF isolado
                let c1 = if ch1_on && vol1_gain > 0.0 {
                    let v1b = self.v1b_ch1.process(s, plate_v * b_fac) * vol1_gain;
                    self.bright_ch1_hp_state = flush_f64(
                        self.bright_ch1_hp_state + alpha_bright * (v1b - self.bright_ch1_hp_state),
                    );
                    let hp = v1b - self.bright_ch1_hp_state;
                    v1b + hp * bright_ch1_boost
                } else {
                    0.0
                };

                // Canal II (Normal) sem bright cap
                let c2 = if ch2_on && vol2_gain > 0.0 {
                    self.v1a_ch2.process(s, plate_v * b_fac) * vol2_gain
                } else {
                    0.0
                };

                let mixed = 0.5 * (c1 + c2);
                let v2a = self.v2a.process(mixed, plate_v * b_fac);
                let v2b = self.v2b_cf.process(v2a, b_fac);
                let after_ts = self.tonestack.process(v2b) * 1.9;

                let pi_out = self.pi.process(after_ts * 1.35, 0.0, b_fac);
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

impl AudioProcessor for Marshall1959SuperLead {
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
        self.v1b_ch1.reset();
        self.v1a_ch2.reset();
        self.v2a.reset();
        self.v2b_cf.reset();
        self.tonestack.reset();
        self.pi.reset();
        self.power_stage.reset();
        self.bias.reset();
        self.sag.reset();
        self.xformer.reset();
        self.reactive.reset();
        self.bright_ch1_hp_state = 0.0;
    }

    fn apply_param_rt(&mut self, id: ParamId, value: ParamValue) -> Result<(), ParamError> {
        match (id, value) {
            (PID_POWER, ParamValue::Bool(b)) => self.panel.power = b,
            (PID_STANDBY, ParamValue::Bool(b)) => self.panel.standby = b,
            (PID_PRESENCE, ParamValue::Float(v)) => self.panel.set_presence(v),
            (PID_BASS, ParamValue::Float(v)) => self.panel.set_bass(v),
            (PID_MIDDLE, ParamValue::Float(v)) => self.panel.set_middle(v),
            (PID_TREBLE, ParamValue::Float(v)) => self.panel.set_treble(v),
            (PID_GAIN, ParamValue::Float(v)) => self.panel.set_input_gain(v),
            (PID_VOLUME_2, ParamValue::Float(v)) => self.panel.set_volume_2(v),
            (PID_JUMPERED, ParamValue::Bool(b)) => {
                self.panel.jumpered = b;
                if b {
                    self.panel.input_jack = AmpInputJack::Jumpered;
                }
            }
            (PID_INPUT_JACK, ParamValue::Enum(s)) => {
                let j = AmpInputJack::from_str_name(s)
                    .ok_or(ParamError::InvalidEnumVariant { key: "amp_1.input_jack", variant: s })?;
                self.panel.input_jack = j;
                self.panel.jumpered = j == AmpInputJack::Jumpered;
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
