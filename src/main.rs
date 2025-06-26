#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod audio;
mod history;
mod left_panel_ui;
mod setting;
mod setting_ui;
mod timer;

use audio::Audio;
use chrono::prelude::*;
use eframe::egui::{
    self, Align, Button, CentralPanel, Color32, ComboBox, Context, FontId, Frame, Layout, RichText,
    Theme, Ui, ViewportCommand, Visuals, WindowLevel, pos2, vec2,
};
use history::History;
use left_panel_ui::LeftPanel;
use setting::Setting;
use setting_ui::SettingWindow;
use std::{
    fs,
    time::{Duration, SystemTime},
};
use timer::{Status, Timer};

use crate::setting::TimerSetting;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let setting = Setting::new();

    let png_bytes = fs::read("assets/timer.png").unwrap();
    let icon = eframe::icon_data::from_png_bytes(&png_bytes).unwrap();
    let mut viewport = egui::ViewportBuilder::default()
        .with_min_inner_size([400.0, 330.0])
        .with_icon(icon)
        .with_maximized(setting.window_maximized());

    if let Some(info) = setting.window_info() {
        viewport = viewport
            .with_position(pos2(info.x, info.y))
            .with_inner_size(vec2(info.width, info.height));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Work Timer",
        options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc, setting)))),
    )
}

struct MyEguiApp {
    main_panel: MainPanel,
    left_panel: LeftPanel,
    setting: Setting,
    setting_window: SettingWindow,
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let btn_status = self.left_panel.ui(ui);
            for btn in btn_status {
                match btn {
                    0 => self.toggle_theme(ui),
                    1 => self.setting_window.show(),
                    _ => (),
                }
            }

            self.main_panel.ui(ctx, ui, &self.setting);
            self.setting_window.ui(ui, &mut self.setting);
            if ctx.input(|i| i.viewport().close_requested()) {
                self.on_close(ctx);
            }
        });
    }
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>, setting: Setting) -> Self {
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

        let setting_window = SettingWindow::new(setting.file_name());

        match setting.theme() {
            setting::Theme::Dark => cc.egui_ctx.set_theme(Theme::Dark),
            setting::Theme::Light => cc.egui_ctx.set_theme(Theme::Light),
            setting::Theme::System => (),
        }

        let history = History::new();

        Self {
            main_panel: MainPanel::new(history),
            left_panel: LeftPanel::new(110.0, &[("\u{1F313}", "Theme"), ("\u{26ED}", "Setting")]),
            setting,
            setting_window,
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
        self.main_panel.timer_panel.change_color(ui);
        self.setting.save();
    }

    fn on_close(&mut self, ctx: &Context) {
        self.main_panel
            .stop(&self.setting.tags()[self.main_panel.tag_index]);

        // Save window info
        ctx.viewport(|v| {
            if v.input.viewport().maximized.unwrap_or(false) {
                self.setting.set_window_maximized(true);
            } else {
                self.setting.set_window_maximized(false);
                match (v.input.viewport().inner_rect, v.input.viewport().outer_rect) {
                    (Some(inner), Some(outer)) => {
                        self.setting.set_window_info(setting::WindowInfo {
                            x: outer.left(),
                            y: outer.top(),
                            width: inner.width(),
                            height: inner.height(),
                        });
                    }
                    _ => (),
                }
            }
        });
        self.setting.save_cache();
    }
}

// ----------------------------------------------------------------------------

struct MainPanel {
    timer_panel: TimerPanel,
    total_time: u64,
    timer: Timer,
    audio: Audio,
    history: History,
    tag_index: usize,
    on_top: bool,
}

impl MainPanel {
    fn new(history: History) -> Self {
        Self {
            total_time: Self::init_total_time(&history),
            timer: Timer::new(),
            timer_panel: TimerPanel::new(),
            audio: Audio::new(),
            history,
            tag_index: 0,
            on_top: false,
        }
    }

    fn ui(&mut self, ctx: &Context, ui: &mut Ui, setting: &Setting) {
        let (is_timeout, counter_string) = self.timer.update();

        if self.timer.status() != Status::Stopped {
            ctx.request_repaint_after_secs(0.2);
        }

        if is_timeout && self.timer.notify() {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
            self.on_top = true;
        } else if self.on_top {
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
            self.on_top = false;
        }

        CentralPanel::default().show_inside(ui, |ui| {
            ui.with_layout(
                Layout::bottom_up(Align::Center).with_cross_justify(true),
                |ui| {
                    ui.label(self.total_string());
                    ui.separator();
                    self.timer_buttons_ui(ui, setting);
                    ui.add_space(12.0);
                    self.tags_ui(ui, setting.tags());
                    self.timer_panel.ui(ui, self.timer.status(), counter_string);
                },
            );
        });
    }

    fn timer_buttons_ui(&mut self, ui: &mut Ui, setting: &Setting) {
        let n = setting.timer_list().len();
        ui.add_space(30.0);
        ui.horizontal(|ui| {
            ui.columns(n, |columns| {
                for (i, t) in setting.timer_list().iter().enumerate() {
                    columns[i].vertical_centered_justified(|ui| {
                        let the_same = self.timer.current_name() == Some(&t.name);
                        let text = if the_same {
                            "\u{23F9} Stop".to_string()
                        } else {
                            format!("{} {}", &t.icon, &t.name)
                        };
                        let btn = Button::new(&text).min_size(vec2(40.0, 40.0));
                        if ui.add(btn).clicked() {
                            self.audio.cancel_notify();
                            if self.timer.status() != Status::Stopped {
                                self.stop(&setting.tags()[self.tag_index]);
                            }
                            if !the_same {
                                self.start(text, t, setting.audio_file());
                            }
                        }
                    });
                }
            });
        });
    }

    fn tags_ui(&mut self, ui: &mut Ui, tags: &[String]) {
        let tag = ComboBox::from_id_salt("Tag")
            .width(ui.available_width())
            .show_index(ui, &mut self.tag_index, tags.len(), |i| tags[i].to_string());
        tag.on_hover_text("Tag. It's saved in the history when you stop the timer.");
    }

    fn start(&mut self, text: String, t: &TimerSetting, audio_file: Option<&str>) {
        self.timer_panel.set_info(text, t.limit_time);
        self.timer.start(t);
        if t.notify() {
            if let Some(name) = audio_file {
                self.audio.schedule_notify(name, t.limit_time * 60);
            }
        }
    }

    fn stop(&mut self, tag: &str) {
        if let Some((duration, name)) = self.timer.stop() {
            self.total_time += duration;
            self.history
                .add_record(self.timer.get_start_time(), duration, &name, tag);
        }
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

    fn init_total_time(history: &History) -> u64 {
        let local: DateTime<Local> = Local::now();
        let target_hour = (local.hour() + 24 - 3) % 24; // minus 3 am
        let end = SystemTime::now();
        let start = end
            .checked_sub(Duration::from_secs(target_hour as u64 * 60 * 60))
            .unwrap();
        history
            .get_records(&start, &end)
            .iter()
            .map(|r| r.duration)
            .sum()
    }
}

// ----------------------------------------------------------------------------

struct TimerPanel {
    status: Status,
    frame: Frame,
    name: String,
    limit_time: u64,
}

impl TimerPanel {
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
            Color32::from_rgb(40, 90, 60)
        } else {
            Color32::from_rgb(140, 235, 130)
        }
    }

    fn change_color(&mut self, ui: &mut Ui) {
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
            self.change_color(ui);
        }
    }

    fn ui(&mut self, ui: &mut Ui, status: Status, counter_string: String) {
        self.update(ui, status);
        self.frame.show(ui, |ui| {
            ui.add_space(ui.available_height() / 2.0 - 68.0);
            ui.label(format!("Limit {} m", self.limit_time));
            ui.label(RichText::new(counter_string).font(FontId::proportional(80.0)));
            ui.label(&self.name);
            ui.add_space(ui.available_height());
        });
    }
}
