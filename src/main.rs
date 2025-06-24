#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod audio;
mod left_panel_ui;
mod setting;
mod timer;

use audio::Audio;
use eframe::egui::{
    self, Align, Button, CentralPanel, Color32, FontId, Frame, Layout, RichText, Theme, Ui,
    ViewportCommand, Visuals, Window, WindowLevel, vec2,
};
use left_panel_ui::LeftPanel;
use setting::Setting;
use std::fs;
use timer::{Status, Timer};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let png_bytes = fs::read("assets/timer.png").unwrap();
    let icon = eframe::icon_data::from_png_bytes(&png_bytes).unwrap();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size([400.0, 300.0])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Work Timer",
        options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    )
}

struct MyEguiApp {
    total_time: u64,
    timer: Timer,
    board: TimerBoard,
    left_panel: LeftPanel,
    setting: Setting,
    setting_window: SettingWindow,
    notify: bool,
    audio: Audio,
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (is_timeout, counter_string) = self.timer.update();

            let btn_status = self.left_panel.ui(ui);
            for btn in btn_status {
                match btn {
                    0 => self.toggle_theme(ui),
                    1 => self.setting_window.show(),
                    _ => (),
                }
            }

            if is_timeout && self.notify {
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::Minimized(false));
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
                self.audio.play_notify(self.setting.audio_file());
                ui.ctx()
                    .send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
            }

            CentralPanel::default().show_inside(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    self.board.ui(ui, self.timer.status(), counter_string);
                });
                ui.with_layout(
                    Layout::bottom_up(Align::Center).with_cross_justify(true),
                    |ui| {
                        ui.label(self.total_string());
                        ui.separator();
                        self.timer_buttons_ui(ui);
                    },
                );
            });

            self.setting_window.ui(ui, &mut self.setting);
        });
    }
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let mut style = egui::Style::default();
        style.override_font_id = Some(egui::FontId::proportional(15.0));
        cc.egui_ctx.set_style_of(Theme::Dark, style.clone());
        cc.egui_ctx.set_style_of(Theme::Light, style);

        let mut v = Visuals::dark();
        v.override_text_color = Some(Color32::from_rgb(240, 240, 240));
        cc.egui_ctx.set_visuals_of(Theme::Dark, v);

        let mut v = Visuals::light();
        v.override_text_color = Some(Color32::from_rgb(20, 20, 20));
        cc.egui_ctx.set_visuals_of(Theme::Light, v);

        let setting = Setting::new();
        let setting_window = SettingWindow::new(setting.file_name());

        match setting.theme() {
            setting::Theme::Dark => cc.egui_ctx.set_theme(Theme::Dark),
            setting::Theme::Light => cc.egui_ctx.set_theme(Theme::Light),
            setting::Theme::System => (),
        }

        Self {
            total_time: 0,
            timer: Timer::new(),
            board: TimerBoard::new(),
            left_panel: LeftPanel::new(110.0, &[("\u{1F313}", "Theme"), ("\u{26ED}", "Setting")]),
            setting,
            setting_window,
            notify: false,
            audio: Audio::new(),
        }
    }

    fn toggle_theme(&mut self, ui: &mut Ui) {
        if ui.visuals().dark_mode {
            ui.ctx().set_theme(Theme::Light);
            self.setting.set_theme(setting::Theme::Light);
        } else {
            ui.ctx().set_theme(Theme::Dark);
            self.setting.set_theme(setting::Theme::Dark);
        }
        self.board.refresh_color(ui);
        self.setting.save();
    }

    fn total_string(&self) -> String {
        let time = self.total_time;
        const HOUR_SEC: u64 = 60 * 60;
        if time >= HOUR_SEC {
            format!(
                "Working Time {} h {} m",
                time / HOUR_SEC,
                (time % HOUR_SEC) / 60
            )
        } else {
            format!("Working Time {} m", time / 60)
        }
    }

    fn timer_buttons_ui(&mut self, ui: &mut Ui) {
        let n = self.setting.timer_list().len();
        ui.add_space(25.0);
        ui.horizontal(|ui| {
            ui.columns(n, |columns| {
                for (i, t) in self.setting.timer_list().iter().enumerate() {
                    columns[i].vertical_centered_justified(|ui| {
                        let the_same = self.timer.current_name() == Some(&t.name);
                        let text = if the_same {
                            "\u{23F9} Stop".to_string()
                        } else {
                            format!("{} {}", &t.icon, &t.name)
                        };
                        let btn = Button::new(&text).min_size(vec2(40.0, 40.0));
                        if ui.add(btn).clicked() {
                            if self.timer.status() != Status::Stopped {
                                // Stop
                                self.total_time += self.timer.stop();
                            }
                            if !the_same {
                                // Start
                                self.notify = t.notify();
                                self.board.set_info(text, t.limit_time);
                                self.timer.start(t);
                            }
                        }
                    });
                }
            });
        });
    }
}

// ----------------------------------------------------------------------------

struct TimerBoard {
    status: Status,
    frame: Frame,
    name: String,
    limit_time: u64,
}

impl TimerBoard {
    fn new() -> Self {
        Self {
            status: Status::Stopped,
            frame: Frame::new()
                .inner_margin(10)
                .outer_margin(5)
                .fill(Color32::TRANSPARENT),
            name: "".to_string(),
            limit_time: 0,
        }
    }

    fn get_red(&self, ui: &mut Ui) -> Color32 {
        if ui.ctx().theme() == Theme::Dark {
            Color32::from_rgb(140, 50, 50)
        } else {
            Color32::from_rgb(255, 120, 110)
        }
    }

    fn get_green(&self, ui: &mut Ui) -> Color32 {
        if ui.ctx().theme() == Theme::Dark {
            Color32::from_rgb(60, 90, 60)
        } else {
            Color32::from_rgb(140, 235, 130)
        }
    }

    fn refresh_color(&mut self, ui: &mut Ui) {
        self.frame.fill = match self.status {
            Status::Stopped => Color32::TRANSPARENT,
            Status::Started => self.get_green(ui),
            Status::TimeOut => self.get_red(ui),
        };
    }

    fn set_info(&mut self, name: String, limit_time: u64) {
        self.name = name;
        self.limit_time = limit_time;
    }

    fn update(&mut self, ui: &mut Ui, status: Status) {
        if status != self.status {
            self.status = status;
            self.refresh_color(ui);
        }

        if status != Status::Stopped {
            ui.ctx().request_repaint_after_secs(0.2);
        }
    }

    fn ui(&mut self, ui: &mut Ui, status: Status, counter_string: String) {
        self.update(ui, status);
        self.frame.show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 2.0 - 105.0);
                ui.label(&self.name);
                ui.label(RichText::new(counter_string).font(FontId::proportional(80.0)));
                ui.label(format!("Limit {} m", self.limit_time));
                ui.add_space(ui.available_height() - 80.0);
            });
        });
    }
}

// ----------------------------------------------------------------------------

struct SettingWindow {
    file_name: String,
    show: bool,
}

impl SettingWindow {
    fn new(file_name: &str) -> Self {
        Self {
            file_name: file_name.to_string(),
            show: false,
        }
    }

    fn show(&mut self) {
        self.show = true;
    }

    fn ui(&mut self, ui: &mut Ui, _setting: &mut Setting) {
        Window::new("Setting")
            .open(&mut self.show)
            .resizable(true)
            .default_width(600.0)
            .show(ui.ctx(), |ui| {
                egui::Grid::new("my_grid").striped(true).show(ui, |ui| {
                    ui.label("Edit file:");
                    ui.label(&self.file_name);
                    ui.end_row();
                });
            });
    }
}
