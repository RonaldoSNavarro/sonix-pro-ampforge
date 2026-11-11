//! crates/ampforge-dsp/src/modshop.rs
//! Bancada Chassi Aberto (Mod Shop) e Osciloscópio CRT em Tempo Real

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreampTube {
    Ax7A,
    At7,
    Ay7,
    Ef86,
}

impl PreampTube {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ax7A => "12ax7",
            Self::At7 => "12at7",
            Self::Ay7 => "12ay7",
            Self::Ef86 => "ef86_variant",
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "12ax7" | "ecc83" => Some(Self::Ax7A),
            "12at7" => Some(Self::At7),
            "12ay7" => Some(Self::Ay7),
            "ef86_variant" | "ef86" => Some(Self::Ef86),
            _ => None,
        }
    }

    pub fn amplification_factor(&self) -> f32 {
        match self {
            Self::Ax7A => 100.0,
            Self::At7 => 60.0,
            Self::Ay7 => 45.0,
            Self::Ef86 => 90.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneStackTopology {
    Model,
    MarshallCathodeFollower,
    FenderTmbMidScoop,
    VoxTopBoost,
    PassiveJamesBaxandall,
    MesaMarkPreGain,
}

impl ToneStackTopology {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::MarshallCathodeFollower => "marshall",
            Self::FenderTmbMidScoop => "fender",
            Self::VoxTopBoost => "vox",
            Self::PassiveJamesBaxandall => "baxandall",
            Self::MesaMarkPreGain => "mesa_mark",
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "model" => Some(Self::Model),
            "marshall" => Some(Self::MarshallCathodeFollower),
            "fender" => Some(Self::FenderTmbMidScoop),
            "vox" => Some(Self::VoxTopBoost),
            "baxandall" => Some(Self::PassiveJamesBaxandall),
            "mesa_mark" => Some(Self::MesaMarkPreGain),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneStackPosition {
    PreDistortion,
    PostPreamp,
    PostPowerAmp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerTube {
    El34,
    Tube6L6Gc,
    Tube6V6,
    Kt88,
    El84,
}

impl PowerTube {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::El34 => "el34",
            Self::Tube6L6Gc => "6l6gc",
            Self::Tube6V6 => "6v6",
            Self::Kt88 => "kt88",
            Self::El84 => "el84_variant",
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "el34" => Some(Self::El34),
            "6l6gc" | "6l6" => Some(Self::Tube6L6Gc),
            "6v6" => Some(Self::Tube6V6),
            "kt88" => Some(Self::Kt88),
            "el84_variant" | "el84" => Some(Self::El84),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerClass {
    ClassA,
    ClassAbPushPull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RectifierType {
    SolidStateDiode,
    Gz34_5Ar4,
    Dual5U4Gb,
}

impl RectifierType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SolidStateDiode => "diode",
            Self::Gz34_5Ar4 => "gz34",
            Self::Dual5U4Gb => "5u4gb",
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "diode" | "solid_state" => Some(Self::SolidStateDiode),
            "gz34" | "5ar4" => Some(Self::Gz34_5Ar4),
            "5u4gb" | "5u4g" => Some(Self::Dual5U4Gb),
            _ => None,
        }
    }

    pub fn internal_resistance(&self) -> f64 {
        match self {
            Self::SolidStateDiode => 0.15,
            Self::Gz34_5Ar4 => 0.45,
            Self::Dual5U4Gb => 0.70,
        }
    }

    pub fn attack_time_sec(&self) -> f64 {
        match self {
            Self::SolidStateDiode => 0.010,
            Self::Gz34_5Ar4 => 0.030,
            Self::Dual5U4Gb => 0.040,
        }
    }

    pub fn release_time_sec(&self) -> f64 {
        match self {
            Self::SolidStateDiode => 0.020,
            Self::Gz34_5Ar4 => 0.080,
            Self::Dual5U4Gb => 0.120,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrtScopeMode {
    Waveform,
    Transfer,
    ToneStack,
}

#[derive(Debug, Clone)]
pub struct CrtScopeTelemetry {
    pub mode: CrtScopeMode,
    pub di_input: Vec<f32>,
    pub saturated_output: Vec<f32>,
    pub transfer_curve_xy: Vec<(f32, f32)>,
    pub tonestack_freq_response_db: Vec<(f32, f32)>,
    pub speaker_impedance_curve_ohm: Vec<(f32, f32)>,
}

#[derive(Debug, Clone)]
pub struct ChassisWorkbench {
    chassis_open: bool,
    preamp_tube: PreampTube,
    plate_voltage: f32, // 180 - 340 V
    triode_asymmetry: f32,
    triode_hardness: f32,
    interstage_hp_cut_hz: f32,
    tonestack_topology: ToneStackTopology,
    tonestack_position: ToneStackPosition,
    power_tube: PowerTube,
    power_class: PowerClass,
    power_tube_bias_ma: f32, // 20 - 55 mA
    negative_feedback: f32,
    phase_inverter_drive: f32,
    transformer_saturation: f32,
    rectifier: RectifierType,
    sag_db: f32,
    b_plus_time_constant_ms: f32,
    ghost_note_filtering_pct: f32,
    scope_mode: CrtScopeMode,
}

impl Default for ChassisWorkbench {
    fn default() -> Self {
        Self::new()
    }
}

impl ChassisWorkbench {
    pub fn new() -> Self {
        Self {
            chassis_open: false,
            preamp_tube: PreampTube::Ax7A,
            plate_voltage: 250.0,
            triode_asymmetry: 0.20,
            triode_hardness: 5.0,
            interstage_hp_cut_hz: 60.0,
            tonestack_topology: ToneStackTopology::MarshallCathodeFollower,
            tonestack_position: ToneStackPosition::PostPreamp,
            power_tube: PowerTube::El34,
            power_class: PowerClass::ClassAbPushPull,
            power_tube_bias_ma: 35.0,
            negative_feedback: 5.0,
            phase_inverter_drive: 5.0,
            transformer_saturation: 5.0,
            rectifier: RectifierType::SolidStateDiode,
            sag_db: 3.0,
            b_plus_time_constant_ms: 50.0,
            ghost_note_filtering_pct: 50.0,
            scope_mode: CrtScopeMode::Waveform,
        }
    }

    pub fn is_chassis_open(&self) -> bool {
        self.chassis_open
    }

    pub fn toggle_chassis(&mut self) {
        self.chassis_open = !self.chassis_open;
    }

    // Preamp subsystem
    pub fn preamp_tube(&self) -> PreampTube {
        self.preamp_tube
    }

    pub fn set_preamp_tube(&mut self, tube: PreampTube) {
        self.preamp_tube = tube;
    }

    pub fn effective_preamp_gain(&self) -> f32 {
        self.preamp_tube.amplification_factor() * (self.plate_voltage / 250.0)
    }

    pub fn plate_voltage(&self) -> f32 {
        self.plate_voltage
    }

    pub fn set_plate_voltage(&mut self, val: f32) {
        self.plate_voltage = val.clamp(180.0, 340.0);
    }

    pub fn triode_asymmetry(&self) -> f32 {
        self.triode_asymmetry
    }

    pub fn set_triode_asymmetry(&mut self, val: f32) {
        self.triode_asymmetry = val.clamp(0.0, 0.60);
    }

    pub fn triode_hardness(&self) -> f32 {
        self.triode_hardness
    }

    pub fn set_triode_hardness(&mut self, val: f32) {
        self.triode_hardness = val.clamp(0.0, 10.0);
    }

    pub fn phase_inverter_drive(&self) -> f32 {
        self.phase_inverter_drive
    }

    pub fn set_phase_inverter_drive(&mut self, val: f32) {
        self.phase_inverter_drive = val.clamp(0.0, 10.0);
    }

    pub fn b_plus_time_constant_ms(&self) -> f32 {
        self.b_plus_time_constant_ms
    }

    pub fn set_b_plus_time_constant_ms(&mut self, val: f32) {
        self.b_plus_time_constant_ms = val.clamp(10.0, 250.0);
    }

    pub fn ghost_note_filtering_pct(&self) -> f32 {
        self.ghost_note_filtering_pct
    }

    pub fn set_ghost_note_filtering_pct(&mut self, val: f32) {
        self.ghost_note_filtering_pct = val.clamp(0.0, 100.0);
    }

    pub fn interstage_hp_cut_hz(&self) -> f32 {
        self.interstage_hp_cut_hz
    }

    pub fn set_interstage_hp_cut_hz(&mut self, val: f32) {
        self.interstage_hp_cut_hz = val.clamp(20.0, 400.0);
    }

    // Tone Stack subsystem
    pub fn tonestack_topology(&self) -> ToneStackTopology {
        self.tonestack_topology
    }

    pub fn set_tonestack_topology(&mut self, top: ToneStackTopology) {
        self.tonestack_topology = top;
    }

    pub fn tonestack_position(&self) -> ToneStackPosition {
        self.tonestack_position
    }

    pub fn set_tonestack_position(&mut self, pos: ToneStackPosition) {
        self.tonestack_position = pos;
    }

    // Power Amp subsystem
    pub fn power_tube(&self) -> PowerTube {
        self.power_tube
    }

    pub fn set_power_tube(&mut self, tube: PowerTube) {
        self.power_tube = tube;
    }

    pub fn power_class(&self) -> PowerClass {
        self.power_class
    }

    pub fn set_power_class(&mut self, p_class: PowerClass) {
        self.power_class = p_class;
    }

    pub fn power_tube_bias_ma(&self) -> f32 {
        self.power_tube_bias_ma
    }

    pub fn set_power_tube_bias_ma(&mut self, val: f32) {
        self.power_tube_bias_ma = val.clamp(20.0, 55.0);
    }

    pub fn negative_feedback(&self) -> f32 {
        self.negative_feedback
    }

    pub fn set_negative_feedback(&mut self, val: f32) {
        self.negative_feedback = val.clamp(0.0, 10.0);
    }

    pub fn transformer_saturation(&self) -> f32 {
        self.transformer_saturation
    }

    pub fn set_transformer_saturation(&mut self, val: f32) {
        self.transformer_saturation = val.clamp(0.0, 10.0);
    }

    // Power Supply subsystem
    pub fn rectifier(&self) -> RectifierType {
        self.rectifier
    }

    pub fn set_rectifier(&mut self, rect: RectifierType) {
        self.rectifier = rect;
    }

    pub fn transient_sag_compression(&self, input_drive: f32) -> f32 {
        let base_sag = match self.rectifier {
            RectifierType::SolidStateDiode => 0.05,
            RectifierType::Gz34_5Ar4 => 0.25,
            RectifierType::Dual5U4Gb => 0.40,
        };
        base_sag * (self.sag_db / 3.0) * input_drive
    }

    // Tube glow calculation
    pub fn calculate_tube_glow(&self) -> (f32, f32) {
        // Bias ranges 20.0 to 55.0 mA
        let norm_bias = (self.power_tube_bias_ma - 20.0) / 35.0; // 0.0 to 1.0
        let radius = 12.0 + norm_bias * 20.0;                   // 12px to 32px
        let opacity = 0.25 + norm_bias * 0.70;                  // 0.25 to 0.95
        (radius, opacity)
    }

    // CRT Scope mode & telemetry
    pub fn scope_mode(&self) -> CrtScopeMode {
        self.scope_mode
    }

    pub fn set_scope_mode(&mut self, mode: CrtScopeMode) {
        self.scope_mode = mode;
    }

    pub fn generate_scope_telemetry(&self) -> CrtScopeTelemetry {
        match self.scope_mode {
            CrtScopeMode::Waveform => {
                let n = 128;
                let mut di = Vec::with_capacity(n);
                let mut sat = Vec::with_capacity(n);
                for i in 0..n {
                    let t = i as f32 / n as f32;
                    let s = (2.0 * std::f32::consts::PI * t).sin();
                    di.push(s);
                    sat.push((s * 2.0).tanh());
                }
                CrtScopeTelemetry {
                    mode: CrtScopeMode::Waveform,
                    di_input: di,
                    saturated_output: sat,
                    transfer_curve_xy: Vec::new(),
                    tonestack_freq_response_db: Vec::new(),
                    speaker_impedance_curve_ohm: Vec::new(),
                }
            }
            CrtScopeMode::Transfer => {
                let n = 100;
                let mut curve = Vec::with_capacity(n);
                for i in 0..n {
                    let x = -2.0 + (4.0 * i as f32 / (n - 1) as f32);
                    let y = (x + self.triode_asymmetry).tanh();
                    curve.push((x, y));
                }
                CrtScopeTelemetry {
                    mode: CrtScopeMode::Transfer,
                    di_input: Vec::new(),
                    saturated_output: Vec::new(),
                    transfer_curve_xy: curve,
                    tonestack_freq_response_db: Vec::new(),
                    speaker_impedance_curve_ohm: Vec::new(),
                }
            }
            CrtScopeMode::ToneStack => {
                let n = 50;
                let mut ts = Vec::with_capacity(n);
                let mut imp = Vec::with_capacity(n);
                for i in 0..n {
                    let f = 20.0 * (1000.0f32).powf(i as f32 / (n - 1) as f32);
                    // Mid scoop or tone curve
                    let db = match self.tonestack_topology {
                        ToneStackTopology::FenderTmbMidScoop => -12.0 * (-((f - 500.0) / 300.0).powi(2)).exp(),
                        _ => -6.0 * (-((f - 800.0) / 400.0).powi(2)).exp(),
                    };
                    ts.push((f, db));
                    // Speaker impedance resonance curve at ~80Hz and rising top
                    let z = 8.0 + 32.0 / (1.0 + ((f - 85.0) / 15.0).powi(2)) + (f / 2000.0);
                    imp.push((f, z));
                }
                CrtScopeTelemetry {
                    mode: CrtScopeMode::ToneStack,
                    di_input: Vec::new(),
                    saturated_output: Vec::new(),
                    transfer_curve_xy: Vec::new(),
                    tonestack_freq_response_db: ts,
                    speaker_impedance_curve_ohm: imp,
                }
            }
        }
    }

    pub fn compute_static_transfer_sample(x: f64, asym: f64, plate_v: f64) -> f64 {
        let v_scale = (plate_v / 250.0).max(0.1);
        let x_scaled = x / v_scale;
        let x_asym = if asym > 1e-4 {
            if x_scaled >= 0.0 {
                x_scaled * (1.0 + 0.5 * asym)
            } else {
                x_scaled * (1.0 - 0.5 * asym)
            }
        } else {
            x_scaled
        };
        (x_asym / (1.0 + x_asym.abs())) * v_scale
    }

    pub fn compute_transfer_curve(&self, num_points: usize) -> Vec<(f32, f32)> {
        let mut pts = Vec::with_capacity(num_points);
        let n = num_points.max(2);
        for i in 0..n {
            let norm = i as f64 / (n - 1) as f64;
            let x = -1.0 + norm * 2.0;
            let y = Self::compute_static_transfer_sample(x, self.triode_asymmetry as f64, self.plate_voltage as f64);
            pts.push((x as f32, y as f32));
        }
        pts
    }
}
