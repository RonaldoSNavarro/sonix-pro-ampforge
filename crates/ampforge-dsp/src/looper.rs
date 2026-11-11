//! crates/ampforge-dsp/src/looper.rs
//! Looper de Performance Integrado com Posicionamento Pre/Post Chain

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LooperState {
    Idle,
    Record,
    Play,
    Overdub,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LooperPosition {
    PreChain,
    PostChain,
}

pub struct LooperProcessor {
    state: LooperState,
    position: LooperPosition,
    sample_rate: f32,
    max_capacity: usize,
    base_buffer: Vec<f32>,
    overdub_buffer: Vec<f32>,
    undo_buffer: Vec<f32>,
    loop_length: usize,
    play_pos: f32,
    reverse: bool,
    half_speed: bool,
    can_undo: bool,
    can_redo: bool,
}

impl Default for LooperProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl LooperProcessor {
    pub fn new() -> Self {
        // Pre-allocate 60 seconds at 48 kHz
        Self::new_with_max_capacity(60 * 48000)
    }

    pub fn new_with_max_capacity(capacity: usize) -> Self {
        Self {
            state: LooperState::Idle,
            position: LooperPosition::PostChain,
            sample_rate: 48000.0,
            max_capacity: capacity,
            base_buffer: vec![0.0; capacity],
            overdub_buffer: vec![0.0; capacity],
            undo_buffer: vec![0.0; capacity],
            loop_length: 0,
            play_pos: 0.0,
            reverse: false,
            half_speed: false,
            can_undo: false,
            can_redo: false,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(1.0);
    }

    pub fn state(&self) -> LooperState {
        self.state
    }

    pub fn position(&self) -> LooperPosition {
        self.position
    }

    pub fn set_position(&mut self, pos: LooperPosition) {
        self.position = pos;
    }

    pub fn is_reverse(&self) -> bool {
        self.reverse
    }

    pub fn set_reverse(&mut self, rev: bool) {
        self.reverse = rev;
    }

    pub fn is_half_speed(&self) -> bool {
        self.half_speed
    }

    pub fn set_half_speed(&mut self, half: bool) {
        self.half_speed = half;
    }

    pub fn loop_length_samples(&self) -> usize {
        self.loop_length
    }

    pub fn progress_pct(&self) -> f32 {
        if self.loop_length == 0 {
            0.0
        } else {
            (self.play_pos / self.loop_length as f32 * 100.0).clamp(0.0, 100.0)
        }
    }

    pub fn can_undo(&self) -> bool {
        self.can_undo
    }

    pub fn can_redo(&self) -> bool {
        self.can_redo
    }

    pub fn record(&mut self) {
        self.state = LooperState::Record;
        self.loop_length = 0;
        self.play_pos = 0.0;
        self.can_undo = false;
        self.can_redo = false;
    }

    pub fn play(&mut self) {
        if self.state == LooperState::Record {
            // Closing base recording loop
            self.state = LooperState::Play;
            self.play_pos = 0.0;
        } else if self.loop_length > 0 {
            self.state = LooperState::Play;
        }
    }

    pub fn overdub(&mut self) {
        if self.loop_length > 0 {
            // Backup current overdub for undo: copia apenas as loop_length amostras (Spec 017 CA-6, CA-9)
            self.undo_buffer[..self.loop_length].copy_from_slice(&self.overdub_buffer[..self.loop_length]);
            self.state = LooperState::Overdub;
            self.can_undo = true;
            self.can_redo = false;
        }
    }

    pub fn stop(&mut self) {
        self.state = LooperState::Stop;
        self.play_pos = 0.0;
    }

    pub fn undo(&mut self) {
        if self.state == LooperState::Overdub {
            // Encerra a gravação e descarta a camada em andamento, voltando para Play e restaurando undo_buffer (Spec 017 CA-7)
            self.state = LooperState::Play;
            self.overdub_buffer[..self.loop_length].copy_from_slice(&self.undo_buffer[..self.loop_length]);
            self.can_redo = false;
        } else if self.can_undo {
            // Troca referências O(1) sem alocação (Spec 017 CA-3, CA-5, CA-6)
            std::mem::swap(&mut self.overdub_buffer, &mut self.undo_buffer);
            self.can_undo = false;
            self.can_redo = true;
        }
    }

    pub fn redo(&mut self) {
        if self.can_redo {
            std::mem::swap(&mut self.overdub_buffer, &mut self.undo_buffer);
            self.can_undo = true;
            self.can_redo = false;
        }
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());

        match self.state {
            LooperState::Idle => {
                output.copy_from_slice(input);
            }
            LooperState::Record => {
                for i in 0..input.len() {
                    if self.loop_length < self.max_capacity {
                        self.base_buffer[self.loop_length] = input[i];
                        self.overdub_buffer[self.loop_length] = 0.0;
                        self.undo_buffer[self.loop_length] = 0.0;
                        self.loop_length += 1;
                        output[i] = input[i];
                    } else {
                        // Max buffer reached: auto switch to Play
                        self.state = LooperState::Play;
                        self.play_pos = 0.0;
                        // Fill remaining output samples with input plus playback
                        let len = self.loop_length.max(1);
                        for j in i..input.len() {
                            let idx = self.play_pos.floor() as usize % len;
                            output[j] = input[j] + self.base_buffer[idx];
                            self.play_pos = (self.play_pos + 1.0) % len as f32;
                        }
                        break;
                    }
                }
            }
            LooperState::Play | LooperState::Overdub => {
                let len = self.loop_length.max(1);
                let speed_step = if self.half_speed { 0.5 } else { 1.0 };

                for i in 0..input.len() {
                    let idx = self.play_pos.floor() as usize % len;
                    let read_idx = if self.reverse { (len - 1) - idx } else { idx };

                    let base_sample = self.base_buffer[read_idx];
                    let overdub_sample = self.overdub_buffer[read_idx];
                    let loop_out = base_sample + overdub_sample;

                    if self.state == LooperState::Overdub {
                        self.overdub_buffer[read_idx] += input[i];
                    }

                    output[i] = input[i] + loop_out;

                    self.play_pos += speed_step;
                    if self.play_pos >= len as f32 {
                        self.play_pos -= len as f32;
                    }
                }
            }
            LooperState::Stop => {
                output.copy_from_slice(input);
            }
        }
    }
}
