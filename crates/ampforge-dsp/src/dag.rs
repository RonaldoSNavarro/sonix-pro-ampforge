//! crates/ampforge-dsp/src/dag.rs
//! Matriz de Roteamento de Sinal 4x12 (DAG) e Nós de Split / Mixer

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlotId {
    pub row: usize,
    pub col: usize,
}

impl SlotId {
    pub fn new(row: usize, col: usize) -> Result<Self, String> {
        if row >= 4 {
            return Err(format!("Linha inválida: {} (máx 3)", row));
        }
        if col >= 12 {
            return Err(format!("Coluna inválida: {} (máx 11)", col));
        }
        Ok(Self { row, col })
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.row * 12 + self.col
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    Gate,
    Compressor,
    Drive,
    Amp,
    Cab,
    Eq,
    Modulation,
    Pitch,
    Delay,
    Reverb,
    Looper,
    Tuner,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockInstance {
    pub id: String,
    pub block_type: BlockType,
    pub bypassed: bool,
    pub params: HashMap<String, f32>,
}

impl BlockInstance {
    pub fn new(id: impl Into<String>, block_type: BlockType) -> Self {
        Self {
            id: id.into(),
            block_type,
            bypassed: false,
            params: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SlotCell {
    Empty,
    Shunt,
    Block(BlockInstance),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowInput {
    InstIn1,
    AuxIn2,
    FxReturnLr,
    UsbReamp34,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowOutput {
    MainXlrLr,
    AmpOutP10NoCab,
    FxSend,
    UsbRecOut,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridMatrix4x12 {
    cells: Vec<SlotCell>,
}

impl Default for GridMatrix4x12 {
    fn default() -> Self {
        Self::new()
    }
}

impl GridMatrix4x12 {
    pub fn new() -> Self {
        Self {
            cells: vec![SlotCell::Empty; 48],
        }
    }

    #[inline]
    pub fn rows(&self) -> usize {
        4
    }

    #[inline]
    pub fn cols(&self) -> usize {
        12
    }

    #[inline]
    pub fn total_slots(&self) -> usize {
        48
    }

    pub fn get_slot(&self, slot: SlotId) -> &SlotCell {
        &self.cells[slot.index()]
    }

    pub fn get_slot_mut(&mut self, slot: SlotId) -> &mut SlotCell {
        let idx = slot.index();
        &mut self.cells[idx]
    }

    pub fn set_slot(&mut self, slot: SlotId, cell: SlotCell) {
        let idx = slot.index();
        self.cells[idx] = cell;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SplitMode {
    BalanceAb {
        ratio: f32, // 0.0 (all A) to 1.0 (all B), 0.5 center
    },
    Crossover {
        freq_hz: f32,
    },
    DynamicSplit {
        threshold_db: f32,
        attack_ms: f32,
        release_ms: f32,
    },
}

#[derive(Debug, Clone)]
pub struct SplitNode {
    mode: SplitMode,
    sample_rate: f32,
    // State for crossover filters (2nd order Linkwitz-Riley / cascaded 1st order)
    lp_y1: f32,
    // State for dynamic split envelope
    envelope: f32,
}

impl SplitNode {
    pub fn new(mode: SplitMode) -> Self {
        Self {
            mode,
            sample_rate: 48000.0,
            lp_y1: 0.0,
            envelope: 0.0,
        }
    }

    pub fn mode(&self) -> &SplitMode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: SplitMode) {
        self.mode = mode;
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn process(&mut self, input: &[f32], out_a: &mut [f32], out_b: &mut [f32]) {
        assert_eq!(input.len(), out_a.len());
        assert_eq!(input.len(), out_b.len());

        match &self.mode {
            SplitMode::BalanceAb { ratio } => {
                let r = ratio.clamp(0.0, 1.0);
                let gain_a = (1.0 - r).sqrt();
                let gain_b = r.sqrt();
                for i in 0..input.len() {
                    out_a[i] = input[i] * gain_a;
                    out_b[i] = input[i] * gain_b;
                }
            }
            SplitMode::Crossover { freq_hz } => {
                // Complementary filter pair (All-pass sum reconstruction)
                // H_lp(z) = (1 - a) / (1 - a*z^-1)
                // H_hp(z) = 1 - H_lp(z)
                // Sum: H_lp(z) + H_hp(z) = 1.0 (flat magnitude reconstruction within 0.0 dB)
                let fc = freq_hz.clamp(20.0, self.sample_rate * 0.45);
                let omega = 2.0 * std::f32::consts::PI * fc / self.sample_rate;
                let a = (-omega).exp();

                for i in 0..input.len() {
                    let x = input[i];
                    let lp = (1.0 - a) * x + a * self.lp_y1;
                    self.lp_y1 = lp;
                    let hp = x - lp;
                    out_a[i] = lp;
                    out_b[i] = hp;
                }
            }
            SplitMode::DynamicSplit {
                threshold_db,
                attack_ms,
                release_ms,
            } => {
                let threshold_linear = 10.0f32.powf(*threshold_db / 20.0);
                let att_coeff = (-1.0 / (self.sample_rate * (attack_ms.max(0.1) * 0.001))).exp();
                let rel_coeff = (-1.0 / (self.sample_rate * (release_ms.max(0.1) * 0.001))).exp();

                for i in 0..input.len() {
                    let abs_x = input[i].abs();
                    if abs_x > self.envelope {
                        self.envelope = (1.0 - att_coeff) * abs_x + att_coeff * self.envelope;
                    } else {
                        self.envelope = (1.0 - rel_coeff) * abs_x + rel_coeff * self.envelope;
                    }

                    // Sigmoid blend based on threshold
                    let diff_db = 20.0 * (self.envelope.max(1e-6) / threshold_linear).log10();
                    let blend = 1.0 / (1.0 + (-0.5 * diff_db).exp()); // 0.0 when quiet, 1.0 when loud

                    // Quiet signals -> Branch A, Loud signals -> Branch B
                    out_a[i] = input[i] * (1.0 - blend);
                    out_b[i] = input[i] * blend;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MixerChannelConfig {
    pub level_db: f32, // -inf to +12.0
    pub pan: f32,      // -1.0 (L) to +1.0 (R)
    pub phase_invert: bool,
}

impl Default for MixerChannelConfig {
    fn default() -> Self {
        Self {
            level_db: 0.0,
            pan: 0.0,
            phase_invert: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MixerNode {
    channels: Vec<MixerChannelConfig>,
    master_level_db: f32,
}

impl Default for MixerNode {
    fn default() -> Self {
        Self::new()
    }
}

impl MixerNode {
    pub fn new() -> Self {
        Self {
            channels: vec![MixerChannelConfig::default(); 4],
            master_level_db: 0.0,
        }
    }

    pub fn configure_channel(&mut self, ch: usize, config: MixerChannelConfig) {
        if ch >= self.channels.len() {
            self.channels.resize(ch + 1, MixerChannelConfig::default());
        }
        self.channels[ch] = config;
    }

    pub fn channel_config(&self, ch: usize) -> MixerChannelConfig {
        if ch < self.channels.len() {
            self.channels[ch]
        } else {
            MixerChannelConfig::default()
        }
    }

    pub fn set_master_level_db(&mut self, db: f32) {
        self.master_level_db = db;
    }

    pub fn master_level_db(&self) -> f32 {
        self.master_level_db
    }

    pub fn process(&self, inputs: &[&[f32]], out_l: &mut [f32], out_r: &mut [f32]) {
        let n = out_l.len();
        out_l.fill(0.0);
        out_r.fill(0.0);

        let master_gain = if self.master_level_db <= -100.0 {
            0.0
        } else {
            10.0f32.powf(self.master_level_db / 20.0)
        };

        for (ch_idx, in_buf) in inputs.iter().enumerate() {
            let cfg = self.channel_config(ch_idx);
            let mut lin_gain = if cfg.level_db <= -100.0 {
                0.0
            } else {
                10.0f32.powf(cfg.level_db / 20.0)
            };
            if cfg.phase_invert {
                lin_gain = -lin_gain;
            }

            // Pan calculation (constant power)
            let pan_normalized = (cfg.pan.clamp(-1.0, 1.0) + 1.0) * 0.5; // 0.0 (L) to 1.0 (R)
            let angle = pan_normalized * (std::f32::consts::PI * 0.5);
            let gain_l = lin_gain * angle.cos();
            let gain_r = lin_gain * angle.sin();

            let samples_to_process = n.min(in_buf.len());
            for i in 0..samples_to_process {
                out_l[i] += in_buf[i] * gain_l;
                out_r[i] += in_buf[i] * gain_r;
            }
        }

        // Apply master level
        for i in 0..n {
            out_l[i] *= master_gain;
            out_r[i] *= master_gain;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagCycleError {
    FeedbackCycleDetected { from: SlotId, to: SlotId },
}

impl std::fmt::Display for DagCycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FeedbackCycleDetected { from, to } => {
                write!(f, "Ciclo de realimentação inválido detectado entre {:?} e {:?}", from, to)
            }
        }
    }
}

impl std::error::Error for DagCycleError {}

#[derive(Debug, Clone)]
pub struct DagGraph {
    pub grid: GridMatrix4x12,
    pub row_inputs: [RowInput; 4],
    pub row_outputs: [RowOutput; 4],
    connections: Vec<(SlotId, SlotId)>,
}

impl Default for DagGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DagGraph {
    pub fn new() -> Self {
        Self {
            grid: GridMatrix4x12::new(),
            row_inputs: [
                RowInput::InstIn1,
                RowInput::AuxIn2,
                RowInput::FxReturnLr,
                RowInput::UsbReamp34,
            ],
            row_outputs: [
                RowOutput::MainXlrLr,
                RowOutput::MainXlrLr,
                RowOutput::MainXlrLr,
                RowOutput::MainXlrLr,
            ],
            connections: Vec::new(),
        }
    }

    pub fn set_row_input(&mut self, row: usize, input: RowInput) {
        if row < 4 {
            self.row_inputs[row] = input;
        }
    }

    pub fn row_input(&self, row: usize) -> RowInput {
        if row < 4 {
            self.row_inputs[row]
        } else {
            RowInput::InstIn1
        }
    }

    pub fn set_row_output(&mut self, row: usize, output: RowOutput) {
        if row < 4 {
            self.row_outputs[row] = output;
        }
    }

    pub fn row_output(&self, row: usize) -> RowOutput {
        if row < 4 {
            self.row_outputs[row]
        } else {
            RowOutput::MainXlrLr
        }
    }

    pub fn set_slot(&mut self, slot: SlotId, cell: SlotCell) {
        self.grid.set_slot(slot, cell);
    }

    pub fn get_slot(&self, slot: SlotId) -> &SlotCell {
        self.grid.get_slot(slot)
    }

    pub fn add_connection(&mut self, from: SlotId, to: SlotId) -> Result<(), DagCycleError> {
        // Anti-cycle rule: connections must strictly progress forward across columns
        // (col_b <= col_a implies cycle or self-feedback)
        if to.col <= from.col {
            return Err(DagCycleError::FeedbackCycleDetected { from, to });
        }

        // Check if an indirect cycle would be formed
        let mut temp_connections = self.connections.clone();
        temp_connections.push((from, to));
        if Self::has_cycle(&temp_connections) {
            return Err(DagCycleError::FeedbackCycleDetected { from, to });
        }

        self.connections.push((from, to));
        Ok(())
    }

    fn has_cycle(connections: &[(SlotId, SlotId)]) -> bool {
        // Kahn's algorithm or DFS
        let mut in_degree: HashMap<SlotId, usize> = HashMap::new();
        let mut adj: HashMap<SlotId, Vec<SlotId>> = HashMap::new();

        for &(u, v) in connections {
            adj.entry(u).or_default().push(v);
            in_degree.entry(v).or_insert(0);
            *in_degree.entry(v).or_default() += 1;
            in_degree.entry(u).or_insert(0);
        }

        let mut queue: VecDeque<SlotId> = in_degree
            .iter()
            .filter(|&(_, &deg)| deg == 0)
            .map(|(&node, _)| node)
            .collect();

        let mut visited = 0;
        let total = in_degree.len();

        while let Some(u) = queue.pop_front() {
            visited += 1;
            if let Some(neighbors) = adj.get(&u) {
                for &v in neighbors {
                    if let Some(deg) = in_degree.get_mut(&v) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(v);
                        }
                    }
                }
            }
        }

        visited < total
    }

    pub fn is_cab_bypassed_for_output(&self, row: usize) -> bool {
        self.row_output(row) == RowOutput::AmpOutP10NoCab
    }

    pub fn toggle_bypass(&mut self, slot: SlotId) {
        if let SlotCell::Block(ref mut instance) = self.grid.get_slot_mut(slot) {
            instance.bypassed = !instance.bypassed;
        }
    }

    pub fn is_slot_bypassed(&self, slot: SlotId) -> bool {
        if let SlotCell::Block(ref instance) = self.grid.get_slot(slot) {
            instance.bypassed
        } else {
            false
        }
    }

    pub fn set_parameter(&mut self, slot: SlotId, name: &str, val: f32) -> Result<(), String> {
        if let SlotCell::Block(ref mut instance) = self.grid.get_slot_mut(slot) {
            instance.params.insert(name.to_string(), val);
            Ok(())
        } else {
            Err("Slot não contém um bloco".to_string())
        }
    }

    pub fn get_parameter(&self, slot: SlotId, name: &str) -> Option<f32> {
        if let SlotCell::Block(ref instance) = self.grid.get_slot(slot) {
            instance.params.get(name).copied()
        } else {
            None
        }
    }
}
