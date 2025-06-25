use std::path::{Path, PathBuf};
use std::process::Command;

use eframe::egui::{self, Ui, Window};

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

    pub fn ui(&mut self, ui: &mut Ui, _setting: &mut Setting) {
        Window::new("Setting")
            .open(&mut self.show)
            .resizable(true)
            .default_width(600.0)
            .show(ui.ctx(), |ui| {
                egui::Grid::new("my_grid").striped(true).show(ui, |ui| {
                    ui.label("Config file:");
                    if ui.button("Open").clicked() {
                        open_file(&self.file_name);
                    }

                    if ui
                        .button("Copy the Path")
                        .on_hover_text(self.file_name.to_str().unwrap())
                        .clicked()
                    {
                        ui.ctx()
                            .copy_text(self.file_name.to_str().unwrap().to_string());
                    }
                    // ui.label(&self.file_name);
                    ui.end_row();
                });
            });
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
