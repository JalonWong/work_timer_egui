use dirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use toml;

pub struct Setting {
    file_name: PathBuf,
    info: SettingInfo,
}

impl Setting {
    pub fn new() -> Self {
        let mut file_name = dirs::config_dir().unwrap();
        file_name.push("work_timer_egui");
        if !file_name.exists() {
            fs::create_dir_all(&file_name).unwrap();
        }

        file_name.push("setting.toml");

        Self {
            file_name,
            info: SettingInfo {
                theme: Theme::System,
                timer_list: vec![
                    TimerSetting {
                        name: "Work".to_string(),
                        icon: "\u{1F528}".to_string(),
                        work_type: true,
                        count_up: true,
                        limit_time: 25,
                    },
                    TimerSetting {
                        name: "Break".to_string(),
                        icon: "\u{2615}".to_string(),
                        work_type: false,
                        count_up: false,
                        limit_time: 5,
                    },
                ],
            },
        }
    }

    pub fn load(&mut self) {
        if self.file_name.exists() {
            self.info = toml::from_str(&fs::read_to_string(&self.file_name).unwrap()).unwrap();
        } else {
            self.save();
        }
    }

    pub fn save(&self) {
        fs::write(&self.file_name, toml::to_string(&self.info).unwrap()).unwrap();
    }

    pub fn file_name(&self) -> &str {
        self.file_name.to_str().unwrap()
    }

    pub fn theme(&self) -> Theme {
        self.info.theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.info.theme = theme;
    }

    pub fn timer_list(&self) -> &[TimerSetting] {
        &self.info.timer_list
    }
}

#[derive(Deserialize, Serialize)]
struct SettingInfo {
    theme: Theme,
    timer_list: Vec<TimerSetting>,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum Theme {
    System,
    Dark,
    Light,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TimerSetting {
    pub name: String,
    pub icon: String,
    pub work_type: bool,
    pub count_up: bool,
    pub limit_time: u64,
}
