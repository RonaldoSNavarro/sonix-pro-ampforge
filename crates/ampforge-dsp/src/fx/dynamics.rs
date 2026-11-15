//! crates/ampforge-dsp/src/fx/dynamics.rs
//! Processadores de Dinâmica: Noise Gate Inteligente e Compressor Multi-Topologia

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseGateMode {
    IntelligentNaturalGate,
    ClassicExpanderGate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateState {
    Open,
    Hold,
    Closed,
}

/// Biquad escalar TDF-II para filtragem de sidechain e envelope sem alocações na heap.
#[derive(Debug, Clone, Copy)]
struct SidechainBiquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl SidechainBiquad {
    fn identity() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    fn set_hpf(&mut self, sample_rate: f32, cutoff_hz: f32, q: f32) {
        let sr = sample_rate.max(1000.0);
        let fc = cutoff_hz.clamp(10.0, sr * 0.45);
        let w0 = 2.0 * std::f32::consts::PI * fc / sr;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 + cos_w0) * 0.5) / a0;
        self.b1 = (-(1.0 + cos_w0)) / a0;
        self.b2 = ((1.0 + cos_w0) * 0.5) / a0;
        self.a1 = (-2.0 * cos_w0) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    fn set_lpf(&mut self, sample_rate: f32, cutoff_hz: f32, q: f32) {
        let sr = sample_rate.max(1000.0);
        let fc = cutoff_hz.clamp(10.0, sr * 0.45);
        let w0 = 2.0 * std::f32::consts::PI * fc / sr;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 - cos_w0) * 0.5) / a0;
        self.b1 = (1.0 - cos_w0) / a0;
        self.b2 = ((1.0 - cos_w0) * 0.5) / a0;
        self.a1 = (-2.0 * cos_w0) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    #[inline]
    fn process(&mut self, x: f32) -> f32 {
        let x = if x.is_finite() { x } else { 0.0 };
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        if !y.is_finite() {
            self.z1 = 0.0;
            self.z2 = 0.0;
            return 0.0;
        }
        y
    }
}

pub struct NoiseGateProcessor {
    mode: NoiseGateMode,
    state: GateState,
    threshold_db: f32,
    hysteresis_db: f32,
    ratio: f32,
    attack_ms: f32,
    hold_ms: f32,
    release_ms: f32,
    range_db: f32,
    key_hpf_hz: f32,
    key_lpf_hz: f32,
    sample_rate: f32,
    envelope: f32,
    current_gain: f32,
    hold_remaining_samples: usize,
    gain_reduction_db: f32,
    // Filtros sidechain LR4 (cascata de 2 biquads 2a ordem, -24 dB/oitava) + LPF duplo de potência (RMS^2)
    key_hpf_1: SidechainBiquad,
    key_hpf_2: SidechainBiquad,
    key_lpf_1: SidechainBiquad,
    key_lpf_2: SidechainBiquad,
    env_lpf_1: SidechainBiquad,
    env_lpf_2: SidechainBiquad,
}

impl NoiseGateProcessor {
    pub fn new(mode: NoiseGateMode) -> Self {
        let mut p = Self {
            mode,
            state: GateState::Closed,
            threshold_db: -50.0,
            hysteresis_db: 4.0,
            ratio: 10.0,
            attack_ms: 1.0,
            hold_ms: 35.0,
            release_ms: 80.0,
            range_db: -70.0,
            key_hpf_hz: 80.0,
            key_lpf_hz: 6000.0,
            sample_rate: 48000.0,
            envelope: 0.0,
            current_gain: 10.0f32.powf(-70.0 / 20.0),
            hold_remaining_samples: 0,
            gain_reduction_db: -70.0,
            key_hpf_1: SidechainBiquad::identity(),
            key_hpf_2: SidechainBiquad::identity(),
            key_lpf_1: SidechainBiquad::identity(),
            key_lpf_2: SidechainBiquad::identity(),
            env_lpf_1: SidechainBiquad::identity(),
            env_lpf_2: SidechainBiquad::identity(),
        };
        p.recompute_filters();
        p
    }

    fn recompute_filters(&mut self) {
        let q_lr = std::f32::consts::FRAC_1_SQRT_2;
        self.key_hpf_1.set_hpf(self.sample_rate, self.key_hpf_hz, q_lr);
        self.key_hpf_2.set_hpf(self.sample_rate, self.key_hpf_hz, q_lr);
        self.key_lpf_1.set_lpf(self.sample_rate, self.key_lpf_hz, q_lr);
        self.key_lpf_2.set_lpf(self.sample_rate, self.key_lpf_hz, q_lr);
        // LPF duplo criticamente amortecido (Q = 0.5) a 220 Hz sobre key_bp^2 dá RMS verdadeiro sem ripple nem overshoot
        self.env_lpf_1.set_lpf(self.sample_rate, 220.0, 0.5);
        self.env_lpf_2.set_lpf(self.sample_rate, 220.0, 0.5);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
        self.recompute_filters();
    }

    pub fn mode(&self) -> NoiseGateMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: NoiseGateMode) {
        self.mode = mode;
    }

    pub fn state(&self) -> GateState {
        self.state
    }

    pub fn threshold_db(&self) -> f32 {
        self.threshold_db
    }

    pub fn set_threshold_db(&mut self, val: f32) {
        if val.is_finite() {
            self.threshold_db = val.clamp(-90.0, 0.0);
        }
    }

    pub fn hysteresis_db(&self) -> f32 {
        self.hysteresis_db
    }

    pub fn set_hysteresis_db(&mut self, val: f32) {
        if val.is_finite() {
            self.hysteresis_db = val.clamp(0.0, 12.0);
        }
    }

    pub fn set_ratio(&mut self, val: f32) {
        if val.is_finite() {
            self.ratio = val.clamp(1.2, 100.0);
        }
    }

    pub fn set_attack_ms(&mut self, val: f32) {
        if val.is_finite() {
            self.attack_ms = val.clamp(0.1, 50.0);
        }
    }

    pub fn set_hold_ms(&mut self, val: f32) {
        if val.is_finite() {
            self.hold_ms = val.clamp(0.0, 500.0);
        }
    }

    pub fn set_release_ms(&mut self, val: f32) {
        if val.is_finite() {
            self.release_ms = val.clamp(5.0, 1000.0);
        }
    }

    pub fn range_db(&self) -> f32 {
        self.range_db
    }

    pub fn set_range_db(&mut self, val: f32) {
        if val.is_finite() {
            self.range_db = val.clamp(-90.0, 0.0);
        }
    }

    pub fn key_hpf_hz(&self) -> f32 {
        self.key_hpf_hz
    }

    pub fn set_key_hpf_hz(&mut self, val: f32) {
        if val.is_finite() {
            self.key_hpf_hz = val.clamp(20.0, 500.0);
            self.recompute_filters();
        }
    }

    pub fn key_lpf_hz(&self) -> f32 {
        self.key_lpf_hz
    }

    pub fn set_key_lpf_hz(&mut self, val: f32) {
        if val.is_finite() {
            self.key_lpf_hz = val.clamp(1000.0, 12000.0);
            self.recompute_filters();
        }
    }

    pub fn gain_reduction_db(&self) -> f32 {
        self.gain_reduction_db
    }

    pub fn process(&mut self, input: &[f32], sidechain_di: Option<&[f32]>, output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        let open_thresh_lin = 10.0f32.powf(self.threshold_db / 20.0);
        let close_thresh_lin = 10.0f32.powf((self.threshold_db - self.hysteresis_db) / 20.0);
        let range_lin = 10.0f32.powf(self.range_db / 20.0);
        let hold_samples = ((self.hold_ms * 0.001) * self.sample_rate).round() as usize;

        // Coeficientes de suavização de ganho (0.1..50ms ataque, 5..1000ms release até atingir faixa)
        let att_coeff = (-1.0 / (self.sample_rate * (self.attack_ms * 0.001))).exp();
        // ~4.6 constantes de tempo para decair praticamente toda a faixa durante release_ms
        let rel_tau_s = (self.release_ms * 0.001) / 4.6;
        let rel_coeff = (-1.0 / (self.sample_rate * rel_tau_s.max(1e-4))).exp();

        for i in 0..input.len() {
            // 1. Fonte do detector (DI limpo no modo Natural, entrada do bloco no Classic)
            let raw_key = match (self.mode, sidechain_di) {
                (NoiseGateMode::IntelligentNaturalGate, Some(di)) if i < di.len() => di[i],
                _ => input[i],
            };

            // 2. Filtro Key Sidechain LR4 (HPF + LPF somente no detector, jamais no áudio principal)
            let key_hp = self.key_hpf_2.process(self.key_hpf_1.process(raw_key));
            let key_bp = self.key_lpf_2.process(self.key_lpf_1.process(key_hp));

            // Detector True-RMS via potência instantânea suavizada sem overshoot
            let p_smooth = self
                .env_lpf_2
                .process(self.env_lpf_1.process(key_bp * key_bp));
            let env_est = p_smooth.max(0.0).sqrt();
            self.envelope = env_est;

            // 3. Máquina de estados com histerese + hold: Open -> Hold -> Closed
            match self.state {
                GateState::Open => {
                    if self.envelope < close_thresh_lin {
                        if hold_samples > 0 {
                            self.state = GateState::Hold;
                            self.hold_remaining_samples = hold_samples;
                        } else {
                            self.state = GateState::Closed;
                        }
                    }
                }
                GateState::Hold => {
                    if self.envelope >= open_thresh_lin {
                        self.state = GateState::Open;
                        self.hold_remaining_samples = 0;
                    } else if self.hold_remaining_samples > 0 {
                        self.hold_remaining_samples -= 1;
                    } else {
                        self.state = GateState::Closed;
                    }
                }
                GateState::Closed => {
                    if self.envelope >= open_thresh_lin {
                        self.state = GateState::Open;
                    }
                }
            }

            // 4. Rampa suave de ganho de saída
            let target_gain = match (self.mode, self.state) {
                (_, GateState::Open | GateState::Hold) => 1.0,
                (NoiseGateMode::ClassicExpanderGate, GateState::Closed) => {
                    let env_db = 20.0 * self.envelope.max(1e-6).log10();
                    let under_db = (self.threshold_db - env_db).max(0.0);
                    let gain_db = (-(under_db * (self.ratio - 1.0))).clamp(self.range_db, 0.0);
                    10.0f32.powf(gain_db / 20.0)
                }
                (NoiseGateMode::IntelligentNaturalGate, GateState::Closed) => range_lin,
            };

            if target_gain > self.current_gain {
                self.current_gain = (1.0 - att_coeff) * target_gain + att_coeff * self.current_gain;
            } else {
                self.current_gain = (1.0 - rel_coeff) * target_gain + rel_coeff * self.current_gain;
            }
            let gain = self.current_gain.clamp(range_lin, 1.0);

            output[i] = if input[i].is_finite() {
                input[i] * gain
            } else {
                0.0
            };

            self.gain_reduction_db = if gain > 1e-5 {
                20.0 * gain.log10()
            } else {
                -90.0
            };
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressorTopology {
    PedalRossDyna,
    StudioFet1176,
    OpticalLa2a,
    VcaMasterBus,
}

pub struct CompressorProcessor {
    pub topology: CompressorTopology,
    threshold_db: f32,
    ratio: f32,
    all_buttons_in: bool,
    attack_ms: f32,
    release_ms: f32,
    makeup_gain_db: f32,
    parallel_mix_pct: f32,
    sample_rate: f32,
    envelope: f32,
    opt_slow_env: f32,
    vca_rms_sq: f32,
    gain_reduction_db: f32,
}

impl CompressorProcessor {
    pub fn new(topology: CompressorTopology) -> Self {
        let (att, rel) = match topology {
            CompressorTopology::StudioFet1176 => (0.05, 200.0),
            _ => (10.0, 100.0),
        };
        Self {
            topology,
            threshold_db: -20.0,
            ratio: 4.0,
            all_buttons_in: false,
            attack_ms: att,
            release_ms: rel,
            makeup_gain_db: 0.0,
            parallel_mix_pct: 100.0,
            sample_rate: 48000.0,
            envelope: 0.0,
            opt_slow_env: 0.0,
            vca_rms_sq: 0.0,
            gain_reduction_db: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn set_threshold_db(&mut self, val: f32) {
        self.threshold_db = val.clamp(-60.0, 0.0);
    }

    pub fn set_ratio(&mut self, val: f32) {
        self.ratio = val.clamp(1.0, 20.0);
    }

    pub fn set_all_buttons_in(&mut self, enabled: bool) {
        self.all_buttons_in = enabled;
    }

    pub fn is_all_buttons_in(&self) -> bool {
        self.all_buttons_in
    }

    pub fn set_makeup_gain_db(&mut self, val: f32) {
        if val.is_finite() {
            self.makeup_gain_db = val.clamp(-24.0, 24.0);
        }
    }

    pub fn makeup_gain_db(&self) -> f32 {
        self.makeup_gain_db
    }

    pub fn set_attack_ms(&mut self, val: f32) {
        self.attack_ms = val.clamp(0.01, 100.0);
    }

    pub fn set_release_ms(&mut self, val: f32) {
        self.release_ms = val.clamp(1.0, 2000.0);
    }

    pub fn set_parallel_mix_pct(&mut self, val: f32) {
        self.parallel_mix_pct = val.clamp(0.0, 100.0);
    }

    pub fn gain_reduction_db(&self) -> f32 {
        self.gain_reduction_db
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        let base_threshold_lin = 10.0f32.powf(self.threshold_db / 20.0);
        let threshold_lin = if self.all_buttons_in {
            base_threshold_lin * 0.65
        } else {
            base_threshold_lin
        };
        let eff_ratio = if self.all_buttons_in {
            28.0
        } else {
            self.ratio
        };
        let makeup_lin = 10.0f32.powf(self.makeup_gain_db / 20.0);
        let wet_ratio = self.parallel_mix_pct / 100.0;
        let dry_ratio = 1.0 - wet_ratio;

        let (topo_att_scale, topo_rel_scale, knee_db) = match self.topology {
            CompressorTopology::StudioFet1176 => (0.45, 0.70, 0.0f32),
            CompressorTopology::PedalRossDyna => (2.4, 2.2, 5.0),
            CompressorTopology::OpticalLa2a => (3.5, 1.4, 6.0),
            CompressorTopology::VcaMasterBus => (1.1, 1.0, 3.0),
        };

        let att_ms = if self.all_buttons_in {
            (self.attack_ms * 0.5).max(0.01)
        } else {
            (self.attack_ms * topo_att_scale).max(0.01)
        };
        let rel_ms = if self.all_buttons_in {
            (self.release_ms * 0.45).max(10.0)
        } else {
            (self.release_ms * topo_rel_scale).max(5.0)
        };
        let att_coeff = (-1.0 / (self.sample_rate * (att_ms * 0.001))).exp();
        let rel_coeff = (-1.0 / (self.sample_rate * (rel_ms * 0.001))).exp();
        let slow_rel_coeff = (-1.0 / (self.sample_rate * 0.800)).exp();
        let rms_coeff = (-1.0 / (self.sample_rate * 0.010)).exp();

        for i in 0..input.len() {
            let x = if input[i].is_finite() { input[i] } else { 0.0 };
            let abs_x = x.abs();

            // Topology-specific detector & envelope
            let det = match self.topology {
                CompressorTopology::StudioFet1176 | CompressorTopology::PedalRossDyna => {
                    if abs_x > self.envelope {
                        self.envelope = (1.0 - att_coeff) * abs_x + att_coeff * self.envelope;
                    } else {
                        self.envelope = (1.0 - rel_coeff) * abs_x + rel_coeff * self.envelope;
                    }
                    self.envelope
                }
                CompressorTopology::OpticalLa2a => {
                    // Program-dependent T4 optical dual-stage release
                    if abs_x > self.envelope {
                        self.envelope = (1.0 - att_coeff) * abs_x + att_coeff * self.envelope;
                    } else {
                        self.envelope = (1.0 - rel_coeff) * abs_x + rel_coeff * self.envelope;
                    }
                    if self.envelope > self.opt_slow_env {
                        self.opt_slow_env = 0.05 * self.envelope + 0.95 * self.opt_slow_env;
                    } else {
                        self.opt_slow_env =
                            (1.0 - slow_rel_coeff) * self.envelope + slow_rel_coeff * self.opt_slow_env;
                    }
                    0.55 * self.envelope + 0.45 * self.opt_slow_env
                }
                CompressorTopology::VcaMasterBus => {
                    // True-RMS power detector
                    self.vca_rms_sq = (1.0 - rms_coeff) * (x * x) + rms_coeff * self.vca_rms_sq;
                    let rms_est = self.vca_rms_sq.max(0.0).sqrt();
                    if rms_est > self.envelope {
                        self.envelope = (1.0 - att_coeff) * rms_est + att_coeff * self.envelope;
                    } else {
                        self.envelope = (1.0 - rel_coeff) * rms_est + rel_coeff * self.envelope;
                    }
                    self.envelope
                }
            };

            let gain = if det <= 1e-9 {
                1.0
            } else {
                let det_db = 20.0 * det.log10();
                let thresh_db = 20.0 * threshold_lin.max(1e-9).log10();
                let overshoot_db = det_db - thresh_db;
                if knee_db > 0.0 && overshoot_db > -knee_db * 0.5 && overshoot_db < knee_db * 0.5 {
                    let k = overshoot_db + knee_db * 0.5;
                    let gr_db = -(1.0 - 1.0 / eff_ratio) * (k * k) / (2.0 * knee_db);
                    10.0f32.powf(gr_db / 20.0)
                } else if overshoot_db > 0.0 {
                    let gr_db = -overshoot_db * (1.0 - 1.0 / eff_ratio);
                    10.0f32.powf(gr_db / 20.0)
                } else {
                    1.0
                }
            };

            self.gain_reduction_db = 20.0 * gain.max(1e-6).log10();

            let compressed = x * gain;
            let shaped = if self.all_buttons_in {
                // "British Mode / All Buttons In": saturação FET estritamente monotônica sem inversão ou DC
                (compressed * 8.5).tanh() / 6.5
            } else if self.topology == CompressorTopology::PedalRossDyna {
                (compressed * 1.15).tanh() / 1.15
            } else {
                compressed
            };

            let wet = shaped * makeup_lin;
            let dry = x;
            output[i] = dry * dry_ratio + wet * wet_ratio;
        }
    }
}
