//! crates/ampforge-dsp/src/amp.rs
//! Modelagem White-Box de Amplificadores Valvulados e Painel Frontal Autêntico

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmpModelId {
    BritJcm800_2203,
    BritPlexi1959,
    UsTwinBlackface65,
    CaliDualRectoRevF,
    UkAc30TopBoost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmpPanelTheme {
    BrushedBrass,
    BrushedChrome,
    BlackoutMetal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmpChannelVoicing {
    Norm,
    HighLead,
}

#[derive(Debug, Clone)]
pub struct AmpModelDescriptor {
    pub id: AmpModelId,
    pub name: &'static str,
    pub brand_text: &'static str,
    pub panel_theme: AmpPanelTheme,
}

#[derive(Debug, Clone)]
pub struct AmpFrontPanel {
    pub input_gain: f32,
    pub overdrive: f32,
    pub bass: f32,
    pub middle: f32,
    pub treble: f32,
    pub presence: f32,
    pub depth: f32,
    pub master_volume: f32,
    pub output_level_db: f32,
    pub bright_switch: bool,
    pub channel_toggle: AmpChannelVoicing,
}

impl Default for AmpFrontPanel {
    fn default() -> Self {
        Self {
            input_gain: 5.0,
            overdrive: 5.0,
            bass: 5.0,
            middle: 5.0,
            treble: 5.0,
            presence: 5.0,
            depth: 5.0,
            master_volume: 5.0,
            output_level_db: 0.0,
            bright_switch: false,
            channel_toggle: AmpChannelVoicing::Norm,
        }
    }
}

impl AmpFrontPanel {
    fn sanitize_val(val: f32, min: f32, max: f32, default_val: f32) -> f32 {
        if val.is_nan() {
            default_val
        } else {
            val.clamp(min, max)
        }
    }

    pub fn set_input_gain(&mut self, val: f32) {
        self.input_gain = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_overdrive(&mut self, val: f32) {
        self.overdrive = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_bass(&mut self, val: f32) {
        self.bass = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_middle(&mut self, val: f32) {
        self.middle = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_treble(&mut self, val: f32) {
        self.treble = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_presence(&mut self, val: f32) {
        self.presence = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_depth(&mut self, val: f32) {
        self.depth = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_master_volume(&mut self, val: f32) {
        self.master_volume = Self::sanitize_val(val, 0.0, 10.0, 5.0);
    }

    pub fn set_output_level_db(&mut self, val: f32) {
        self.output_level_db = Self::sanitize_val(val, -24.0, 12.0, 0.0);
    }
}

#[derive(Debug, Clone)]
pub struct ChassisModSettings {
    pub b_plus_voltage: f32,
    pub cathode_bias_resistor_ohm: f32,
    pub sag_capacitance_uf: f32,
    pub negative_feedback_ratio: f32,
}

impl Default for ChassisModSettings {
    fn default() -> Self {
        Self {
            b_plus_voltage: 480.0,
            cathode_bias_resistor_ohm: 1500.0,
            sag_capacitance_uf: 50.0,
            negative_feedback_ratio: 0.1,
        }
    }
}

pub struct AmpModelCatalog {
    models: Vec<AmpModelDescriptor>,
}

impl Default for AmpModelCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl AmpModelCatalog {
    pub fn new() -> Self {
        Self {
            models: vec![
                AmpModelDescriptor {
                    id: AmpModelId::BritJcm800_2203,
                    name: "brit_jcm800_2203",
                    brand_text: "MARSHALL™ / BRIT 800",
                    panel_theme: AmpPanelTheme::BrushedBrass,
                },
                AmpModelDescriptor {
                    id: AmpModelId::BritPlexi1959,
                    name: "brit_plexi_1959",
                    brand_text: "PLEXI 1959 SUPER LEAD",
                    panel_theme: AmpPanelTheme::BrushedBrass,
                },
                AmpModelDescriptor {
                    id: AmpModelId::UsTwinBlackface65,
                    name: "us_twin_blackface_65",
                    brand_text: "US '65 TWIN REVERB",
                    panel_theme: AmpPanelTheme::BrushedChrome,
                },
                AmpModelDescriptor {
                    id: AmpModelId::CaliDualRectoRevF,
                    name: "cali_dual_recto_revf",
                    brand_text: "CALI RECTIFIER REV-F",
                    panel_theme: AmpPanelTheme::BlackoutMetal,
                },
                AmpModelDescriptor {
                    id: AmpModelId::UkAc30TopBoost,
                    name: "uk_ac30_topboost",
                    brand_text: "UK AC30 TOP BOOST",
                    panel_theme: AmpPanelTheme::BrushedBrass,
                },
            ],
        }
    }

    pub fn get_descriptor(&self, id: AmpModelId) -> &AmpModelDescriptor {
        self.models
            .iter()
            .find(|m| m.id == id)
            .expect("Modelo de amplificador desconhecido")
    }

    pub fn find_model_by_name(&self, name: &str) -> Result<AmpModelId, String> {
        self.models
            .iter()
            .find(|m| m.name == name)
            .map(|m| m.id)
            .ok_or_else(|| format!("Modelo de amplificador '{}' não encontrado", name))
    }
}

pub struct WhiteBoxAmpProcessor {
    pub model_id: AmpModelId,
    panel: AmpFrontPanel,
    mod_settings: ChassisModSettings,
    sample_rate: f32,
    // Internal state variables for IIR filters
    bright_state: f64,
    presence_state: f64,
    depth_state: f64,
}

impl WhiteBoxAmpProcessor {
    pub fn new(model_id: AmpModelId) -> Self {
        Self {
            model_id,
            panel: AmpFrontPanel::default(),
            mod_settings: ChassisModSettings::default(),
            sample_rate: 48000.0,
            bright_state: 0.0,
            presence_state: 0.0,
            depth_state: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn panel(&self) -> &AmpFrontPanel {
        &self.panel
    }

    pub fn panel_mut(&mut self) -> &mut AmpFrontPanel {
        &mut self.panel
    }

    pub fn mod_settings(&self) -> &ChassisModSettings {
        &self.mod_settings
    }

    pub fn mod_settings_mut(&mut self) -> &mut ChassisModSettings {
        &mut self.mod_settings
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());

        let gain_v1 = (self.panel.input_gain / 10.0).max(0.01);
        let lead_mult = match self.panel.channel_toggle {
            AmpChannelVoicing::Norm => 1.0,
            AmpChannelVoicing::HighLead => 2.5,
        };

        // Bright switch physical circuit:
        // Capacitor C_bright in parallel with (1 - wiper) part of the pot.
        // As input_gain -> 10, the pot wiper shorts the capacitor, so boost -> 0 dB!
        // At low input_gain (e.g. <= 3), the bypass boost is high (>= +3.5 dB at 4kHz).
        let bright_effective_boost = if self.panel.bright_switch {
            let pot_ratio = (10.0 - self.panel.input_gain) / 10.0;
            // 470 pF bright cap boost scaling
            pot_ratio * 0.95
        } else {
            0.0
        };

        let master_pot = self.panel.master_volume / 10.0;
        let power_saturation_drive = 1.0 + master_pot * 6.0;

        // Linear output level multiplier (clean post-power amp)
        let out_lin = 10.0f64.powf(self.panel.output_level_db as f64 / 20.0);

        // Feedback loop filters for presence and depth
        let pres_coeff = (self.panel.presence / 10.0) as f64;
        let depth_coeff = (self.panel.depth / 10.0) as f64;

        for i in 0..input.len() {
            let mut s = input[i] as f64;

            // 1. Input Bright Cap Filter (high-frequency emphasis depending on pot position)
            if bright_effective_boost > 0.001 {
                let high_pass = s - self.bright_state;
                self.bright_state = 0.85 * self.bright_state + 0.15 * s;
                s += high_pass * (bright_effective_boost as f64 * 1.5);
            } else {
                self.bright_state = 0.85 * self.bright_state + 0.15 * s;
            }

            // 2. Preamp Triode V1 Stage with Voicing
            s *= (gain_v1 * lead_mult) as f64;
            // Asymmetric soft clipping
            s = s / (1.0 + s.abs());

            // 3. Power Amp / Phase Inverter Stage (driven by masterVolume)
            let power_in = s * power_saturation_drive as f64;
            // Symmetrical push-pull saturation: tanh-like soft clipping with sag
            let mut power_out = power_in.tanh();

            // 4. Negative Feedback Loop Filters: Depth (resonant low end) & Presence (treble boost)
            let depth_hp = power_out - self.depth_state;
            self.depth_state = 0.98 * self.depth_state + 0.02 * power_out;
            let depth_res = depth_hp * depth_coeff * 0.8;

            let pres_hp = power_out - self.presence_state;
            self.presence_state = 0.6 * self.presence_state + 0.4 * power_out;
            let pres_boost = pres_hp * pres_coeff * 0.8;

            power_out += depth_res + pres_boost;

            // 5. Final Output Level (Clean linear scaling)
            output[i] = (power_out * out_lin) as f32;
        }
    }

    /// Computes Total Harmonic Distortion (THD) ratio of a periodic signal
    pub fn compute_thd(&self, signal: &[f32]) -> f32 {
        if signal.is_empty() {
            return 0.0;
        }
        let mean: f32 = signal.iter().sum::<f32>() / signal.len() as f32;
        let ac: Vec<f32> = signal.iter().map(|&x| x - mean).collect();

        // Perform basic DFT at fundamental frequency (assuming ~1000 Hz with N=2048, sr=48000)
        let n = ac.len();
        let fs = self.sample_rate;
        let f0 = 1000.0;
        let bin_k = (f0 * n as f32 / fs).round() as usize;

        let calc_power_at_bin = |k: usize| -> f32 {
            let mut real = 0.0f32;
            let mut imag = 0.0f32;
            let angle = 2.0 * std::f32::consts::PI * (k as f32) / (n as f32);
            for (idx, &sample) in ac.iter().enumerate() {
                real += sample * (angle * idx as f32).cos();
                imag += sample * (angle * idx as f32).sin();
            }
            real * real + imag * imag
        };

        let p_fund = calc_power_at_bin(bin_k).max(1e-12);
        let mut p_harmonics = 0.0f32;
        for h in 2..=5 {
            let hk = bin_k * h;
            if hk < n / 2 {
                p_harmonics += calc_power_at_bin(hk);
            }
        }

        (p_harmonics / p_fund).sqrt()
    }
}
