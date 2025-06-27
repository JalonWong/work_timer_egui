use std::path::{Path, PathBuf};
use std::process::Command;

use eframe::egui::{self, Grid, Id, Modal, Ui, vec2};

use crate::setting::Setting;

pub struct SettingWindow {
    file_name: PathBuf,
    show: bool,
}

impl SettingWindow {
    pub fn new(file_name: &Path) -> Self {
        Self {
            file_name: file_name.to_path_buf(),
            show: false,
        }
    }

    pub fn show(&mut self) {
        self.show = true;
    }

    pub fn is_show(&self) -> bool {
        self.show
    }

    pub fn ui(&mut self, ui: &mut Ui, setting: &mut Setting) {
        if self.show {
            let modal = Modal::new(Id::new("setting")).backdrop_color(crate::MODAL_BG);
            let response = modal.show(ui.ctx(), |ui| {
                ui.heading("Setting");
                ui.separator();

                let grid = Grid::new("my_grid").striped(true).spacing(vec2(8.0, 14.0));
                grid.show(ui, |ui| {
                    ui.set_width(200.0);
                    ui.label("Config file:");
                    if ui.button("Open").clicked() {
                        open_file(&self.file_name);
                    }
                    ui.end_row();

                    ui.label("Theme:");
                    egui::widgets::global_theme_preference_buttons(ui);
                    ui.end_row();

                    ui.label("Audio:");
                    ui.vertical(|ui| {
                        ui.allocate_space(vec2(250.0, 0.0));
                        let mut play_audio = setting.play_audio();
                        if ui
                            .checkbox(&mut play_audio, "Play audio when notified")
                            .clicked()
                        {
                            setting.set_play_audio(play_audio);
                        }
                        ui.text_edit_singleline(setting.mut_audio_file());
                    });
                    ui.end_row();

                    const VERSION: &str = env!("CARGO_PKG_VERSION");
                    ui.label("Version:");
                    ui.label(VERSION);
                    ui.end_row();

                    ui.label("Source Code:");
                    use egui::special_emojis::GITHUB;
                    ui.hyperlink_to(
                        format!("{GITHUB} GitHub"),
                        "https://github.com/JalonWong/work_timer_egui",
                    );
                    ui.end_row();
                });
            });
            if response.should_close() {
                let theme = ui.ctx().options(|opt| opt.theme_preference);
                setting.set_theme(theme.into());
                setting.save();
                self.show = false;
            }
        }
    }
}

fn open_file(file_path: &Path) {
    let file_path = file_path.to_str().unwrap();
    #[cfg(target_os = "windows")]
    Command::new("cmd")
        .args(&["/C", "start", "", file_path])
        .spawn()
        .ok();

    #[cfg(target_os = "linux")]
    Command::new("xdg-open").arg(file_path).spawn().ok();

    #[cfg(target_os = "macos")]
    Command::new("open").arg(file_path).spawn().ok();
}
