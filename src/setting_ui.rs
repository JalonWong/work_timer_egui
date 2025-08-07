use eframe::egui::{self, Grid, Id, Label, Modal, TextWrapMode, Ui, vec2};
use rfd::FileDialog;

use crate::{MyColor, setting::Setting};

pub struct SettingWindow {
    show: bool,
}

impl SettingWindow {
    pub fn new() -> Self {
        Self { show: false }
    }

    pub fn show(&mut self) {
        self.show = true;
    }

    pub fn is_show(&self) -> bool {
        self.show
    }

    pub fn ui(&mut self, ui: &mut Ui, setting: &mut Setting) {
        if self.show {
            let modal = Modal::new(Id::new("setting")).backdrop_color(MyColor::MODAL_BG);
            let response = modal.show(ui.ctx(), |ui| {
                ui.heading("Setting");
                ui.separator();

                let grid = Grid::new("my_grid").striped(true).spacing(vec2(8.0, 14.0));
                grid.show(ui, |ui| {
                    ui.set_width(200.0);

                    ui.label("Theme:");
                    egui::widgets::global_theme_preference_buttons(ui);
                    ui.end_row();

                    ui.label("Audio:");
                    ui.vertical(|ui| {
                        ui.allocate_space(vec2(250.0, 0.0));
                        ui.add(
                            Label::new(setting.mut_audio_file().as_str())
                                .wrap_mode(TextWrapMode::Extend),
                        );
                        ui.horizontal(|ui| {
                            if ui.button("Set audio file").clicked() {
                                if let Some(audio_file) = FileDialog::new()
                                    .add_filter("audio", &["wav", "mp3"])
                                    .pick_file()
                                {
                                    setting.mut_audio_file().clear();
                                    setting
                                        .mut_audio_file()
                                        .push_str(&audio_file.display().to_string());
                                }
                            }
                            if ui.button("Reset").clicked() {
                                setting.mut_audio_file().clear();
                                setting.mut_audio_file().push_str("assets/notify.wav");
                            }
                        });

                        let mut play_audio = setting.play_audio();
                        if ui
                            .checkbox(&mut play_audio, "Play audio when notified")
                            .clicked()
                        {
                            setting.set_play_audio(play_audio);
                        }
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
