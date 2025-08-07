use eframe::egui::ThemePreference;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Setting {
    cache_name: PathBuf,
    cache_info: CacheInfo,
    file_name: PathBuf,
    info: SettingInfo,
}

impl Setting {
    pub fn new() -> Self {
        let mut file_name = get_config_dir();

        let mut cache_name = file_name.clone();
        cache_name.push("cache.toml");

        file_name.push("config.toml");

        let info = Self::load_setting(&file_name);
        let mut cache_info = Self::load_cache(&cache_name);
        if cache_info.tag_index >= info.tags.len() {
            cache_info.tag_index = 0;
        }

        Self {
            cache_name,
            cache_info,
            file_name,
            info,
        }
    }

    fn load_setting(file_name: &Path) -> SettingInfo {
        let mut need_save = true;

        let mut info = SettingInfo {
            theme: Theme::System,
            audio_file: "assets/notify.wav".to_string(),
            play_audio: true,
            tags: vec![
                "Program".to_string(),
                "English".to_string(),
                "Read".to_string(),
            ],
            timer_list: vec![
                TimerSetting {
                    name: "Break".to_string(),
                    icon: "\u{2615}".to_string(),
                    limit_time: 5,
                    for_work: false,
                    count_up: false,
                    notify: true,
                },
                TimerSetting {
                    name: "Work".to_string(),
                    icon: "\u{1F4BB}".to_string(),
                    limit_time: 25,
                    for_work: true,
                    count_up: true,
                    notify: false,
                },
            ],
        };

        // Load
        if file_name.exists() {
            let toml_str = fs::read_to_string(file_name).unwrap();
            if let Ok(i) = toml::from_str(&toml_str) {
                info = i;
                need_save = false;
            }
        }

        // Save
        if need_save {
            fs::write(file_name, toml::to_string(&info).unwrap()).unwrap();
        }
        info
    }

    fn load_cache(file_name: &Path) -> CacheInfo {
        let mut info = CacheInfo {
            maximized: false,
            window: None,
            tag_index: 0,
        };

        if file_name.exists() {
            let toml_str = fs::read_to_string(file_name).unwrap();
            if let Ok(i) = toml::from_str(&toml_str) {
                info = i;
            }
        }
        info
    }

    pub fn save(&self) {
        fs::write(&self.file_name, toml::to_string(&self.info).unwrap()).unwrap();
    }

    pub fn save_cache(&self) {
        fs::write(&self.cache_name, toml::to_string(&self.cache_info).unwrap()).unwrap();
    }

    pub fn window_info(&self) -> Option<&WindowInfo> {
        self.cache_info.window.as_ref()
    }

    pub fn set_window_info(&mut self, info: WindowInfo) {
        self.cache_info.window = Some(info);
    }

    pub fn window_maximized(&self) -> bool {
        self.cache_info.maximized
    }

    pub fn set_window_maximized(&mut self, maximized: bool) {
        self.cache_info.maximized = maximized;
    }

    pub fn file_name(&self) -> &Path {
        &self.file_name
    }

    pub fn tags(&self) -> &[String] {
        self.info.tags.as_slice()
    }

    pub fn mut_tags(&mut self) -> &mut Vec<String> {
        &mut self.info.tags
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

    pub fn add_timer(&mut self, timer: TimerSetting) {
        self.info.timer_list.push(timer);
    }

    pub fn mut_timer_list(&mut self) -> &mut Vec<TimerSetting> {
        &mut self.info.timer_list
    }

    pub fn audio_file(&self) -> Option<&str> {
        if self.info.play_audio {
            Some(&self.info.audio_file)
        } else {
            None
        }
    }

    pub fn mut_audio_file(&mut self) -> &mut String {
        &mut self.info.audio_file
    }

    pub fn set_play_audio(&mut self, v: bool) {
        self.info.play_audio = v;
    }

    pub fn play_audio(&self) -> bool {
        self.info.play_audio
    }

    pub fn set_tag_index(&mut self, v: usize) {
        self.cache_info.tag_index = v;
    }

    pub fn tag_index(&self) -> usize {
        self.cache_info.tag_index
    }
}

pub fn get_config_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap();
    #[cfg(debug_assertions)]
    path.push("work_timer_egui_dbg");
    #[cfg(not(debug_assertions))]
    path.push("work_timer_egui");

    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path
}

// ----------------------------------------------------------------------------

#[derive(Deserialize, Serialize)]
struct CacheInfo {
    maximized: bool,
    window: Option<WindowInfo>,
    tag_index: usize,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WindowInfo {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

// ----------------------------------------------------------------------------

#[derive(Deserialize, Serialize)]
struct SettingInfo {
    theme: Theme,
    play_audio: bool,
    audio_file: String,
    tags: Vec<String>,
    timer_list: Vec<TimerSetting>,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum Theme {
    System,
    Dark,
    Light,
}

impl From<ThemePreference> for Theme {
    fn from(value: ThemePreference) -> Self {
        match value {
            ThemePreference::Dark => Self::Dark,
            ThemePreference::Light => Self::Light,
            ThemePreference::System => Self::System,
        }
    }
}

impl From<Theme> for ThemePreference {
    fn from(val: Theme) -> Self {
        match val {
            Theme::Dark => Self::Dark,
            Theme::Light => Self::Light,
            Theme::System => Self::System,
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TimerSetting {
    pub name: String,
    pub icon: String,
    pub limit_time: u64,
    pub for_work: bool,
    pub count_up: bool,
    pub notify: bool,
}

impl TimerSetting {
    pub fn new() -> Self {
        Self {
            name: "new".to_string(),
            icon: String::new(),
            limit_time: 1,
            for_work: false,
            count_up: false,
            notify: false,
        }
    }
}
