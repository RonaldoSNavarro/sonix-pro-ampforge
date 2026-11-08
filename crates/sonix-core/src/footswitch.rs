//! crates/sonix-core/src/footswitch.rs
//! Modos Operacionais dos 8 Footswitches Físicos (Preset, Scene, Stomp, Hybrid)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FootswitchMode {
    Preset,
    Scene,
    Stomp,
    HybridStompScene,
}

#[derive(Debug, Clone)]
pub struct FootswitchUnit {
    pub id: usize, // 1 to 8
    pub oled_top_line: String,
    pub oled_main_text: String,
    pub color_hex: String,
}

pub struct FootswitchArray {
    mode: FootswitchMode,
    units: [FootswitchUnit; 8],
}

impl Default for FootswitchArray {
    fn default() -> Self {
        Self::new()
    }
}

impl FootswitchArray {
    pub fn new() -> Self {
        let mut arr = Self {
            mode: FootswitchMode::HybridStompScene,
            units: std::array::from_fn(|i| FootswitchUnit {
                id: i + 1,
                oled_top_line: String::new(),
                oled_main_text: String::new(),
                color_hex: "#00E676".to_string(),
            }),
        };
        arr.reconfigure_units();
        arr
    }

    pub fn mode(&self) -> FootswitchMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: FootswitchMode) {
        self.mode = mode;
        self.reconfigure_units();
    }

    pub fn unit(&self, idx: usize) -> &FootswitchUnit {
        &self.units[idx]
    }

    fn reconfigure_units(&mut self) {
        match self.mode {
            FootswitchMode::Preset => {
                for i in 0..8 {
                    self.units[i].oled_top_line = "PRESET".to_string();
                    self.units[i].oled_main_text = format!("P{:02}", i + 1);
                    self.units[i].color_hex = "#00B0FF".to_string();
                }
            }
            FootswitchMode::Scene => {
                let scene_letters = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];
                for i in 0..8 {
                    self.units[i].oled_top_line = "SCENE".to_string();
                    self.units[i].oled_main_text = format!("SCENE {}", scene_letters[i]);
                    self.units[i].color_hex = "#7C4DFF".to_string();
                }
            }
            FootswitchMode::Stomp => {
                for i in 0..8 {
                    self.units[i].oled_top_line = "STOMP".to_string();
                    self.units[i].oled_main_text = format!("FX {}", i + 1);
                    self.units[i].color_hex = "#FF9100".to_string();
                }
            }
            FootswitchMode::HybridStompScene => {
                // FS 1..4 = Stomps
                for i in 0..4 {
                    self.units[i].oled_top_line = "STOMP".to_string();
                    self.units[i].oled_main_text = format!("FX {}", i + 1);
                    self.units[i].color_hex = "#FF9100".to_string();
                }
                // FS 5..8 = Scenes A..D
                let scene_letters = ['A', 'B', 'C', 'D'];
                for i in 0..4 {
                    self.units[i + 4].oled_top_line = "SCENE".to_string();
                    self.units[i + 4].oled_main_text = format!("SCENE {}", scene_letters[i]);
                    self.units[i + 4].color_hex = "#7C4DFF".to_string();
                }
            }
        }
    }
}
