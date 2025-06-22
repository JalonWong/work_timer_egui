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

        file_name.push("config.toml");

        let mut need_save = true;

        let mut info = SettingInfo {
            version: 4,
            theme: Theme::System,
            audio_file: "assets/notify.wav".to_string(),
            play_audio: true,
            timer_list: vec![
                TimerSetting {
                    name: "Break".to_string(),
                    icon: "\u{2615}".to_string(),
                    limit_time: 5,
                    work_type: false,
                    count_up: false,
                    notify: true,
                },
                TimerSetting {
                    name: "Work".to_string(),
                    icon: "\u{1F4BB}".to_string(),
                    limit_time: 25,
                    work_type: true,
                    count_up: true,
                    notify: false,
                },
            ],
        };

        // Load
        if file_name.exists() {
            let toml_str = fs::read_to_string(&file_name).unwrap();
            if let Ok(i) = toml::from_str(&toml_str) {
                info = i;
                need_save = false;
            }
        }

        // Save
        if need_save {
            fs::write(&file_name, toml::to_string(&info).unwrap()).unwrap();
        }

        Self {
            file_name,
            info,
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

    pub fn audio_file(&self) -> Option<&str> {
        if self.info.play_audio {
            Some(&self.info.audio_file)
        } else {
            None
        }
    }
}

#[derive(Deserialize, Serialize)]
struct SettingInfo {
    version: u32,
    theme: Theme,
    play_audio: bool,
    audio_file: String,
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
    pub limit_time: u64,
    pub work_type: bool,
    pub count_up: bool,
    notify: bool,
}

impl TimerSetting {
    pub fn notify(&self) -> bool {
        self.notify
    }
}
