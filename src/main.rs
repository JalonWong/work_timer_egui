#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod audio;
mod chart_ui;
mod history;
mod history_ui;
mod left_panel_ui;
mod setting;
mod setting_ui;
mod tags_ui;
mod timer;
mod timers_ui;

use audio::Audio;
use chart_ui::ChartWindow;
use chrono::Local;
use eframe::egui::{
    self, Align, Button, CentralPanel, Color32, ComboBox, Context, FontId, Frame, Layout, RichText,
    TextStyle, Theme, Ui, ViewportCommand, Visuals, WindowLevel, pos2, vec2,
};
use history::History;
use history_ui::HistoryWindow;
use left_panel_ui::LeftPanel;
use setting::Setting;
use setting_ui::SettingWindow;
use std::{fs, path::PathBuf, time::SystemTime};
use tags_ui::TagsWindow;
use timer::{Status, Timer};
use timers_ui::TimersWindow;

use crate::setting::TimerSetting;

fn main() -> eframe::Result {
    let setting = Setting::new();

    let app_path = get_app_path();
    let png_bytes = fs::read(app_path.join("assets/timer.png")).unwrap();
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
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc, setting, app_path)))),
    )
}

fn get_app_path() -> PathBuf {
    #[cfg(debug_assertions)]
    let app_path = PathBuf::from("./");
    #[cfg(all(not(debug_assertions), not(target_os = "macos")))]
    let app_path = {
        let exe_dir = std::env::current_exe().unwrap();
        exe_dir.parent().unwrap().to_path_buf()
    };
    #[cfg(all(not(debug_assertions), target_os = "macos"))]
    let app_path = {
        let exe_dir = std::env::current_exe().unwrap();
        exe_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("Resources")
    };
    app_path
}

struct MyEguiApp {
    main_panel: MainPanel,
    left_panel: LeftPanel,
    setting: Setting,
    setting_window: SettingWindow,
    history: History,
    history_window: HistoryWindow,
    chart_window: ChartWindow,
    tags_window: TagsWindow,
    timers_window: TimersWindow,
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let btn_status = self.left_panel.ui(ui);
            for btn in btn_status {
                match btn {
                    0 => self.chart_window.show(&self.history),
                    1 => self.history_window.show(&self.history),
                    2 => self.tags_window.show(),
                    3 => self.timers_window.show(&self.setting),
                    4 => self.setting_window.show(),
                    _ => (),
                }
            }

            self.main_panel
                .ui(ctx, ui, &self.setting, &mut self.history);
            self.chart_window.ui(ui, &self.history);
            self.history_window.ui(ui, &mut self.history);
            self.setting_window.ui(ui, &mut self.setting);
            if self.setting_window.is_show() {
                self.main_panel.timer_panel.change_color(ui);
            }
            self.tags_window.ui(ui, &mut self.setting);
            self.timers_window.ui(ui, &mut self.setting);
            if ctx.input(|i| i.viewport().close_requested()) {
                self.on_close(ctx);
            }
        });
    }
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>, setting: Setting, app_path: PathBuf) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let style = eframe::egui::Style {
            text_styles: [
                (TextStyle::Heading, FontId::proportional(30.0)),
                (TextStyle::Body, FontId::proportional(15.0)),
                (TextStyle::Button, FontId::proportional(15.0)),
                (TextStyle::Monospace, FontId::monospace(15.0)),
            ]
            .into(),
            ..Default::default()
        };
        cc.egui_ctx.set_style_of(Theme::Dark, style.clone());
        cc.egui_ctx.set_style_of(Theme::Light, style);

        let mut v = Visuals::dark();
        v.override_text_color = Some(Color32::from_rgb(240, 240, 240));
        cc.egui_ctx.set_visuals_of(Theme::Dark, v);

        let mut v = Visuals::light();
        v.override_text_color = Some(Color32::from_rgb(20, 20, 20));
        cc.egui_ctx.set_visuals_of(Theme::Light, v);

        let setting_window = SettingWindow::new();

        cc.egui_ctx.set_theme(setting.theme());

        let history = History::new();

        Self {
            main_panel: MainPanel::new(
                Self::init_total_time(&history),
                setting.tag_index(),
                app_path,
            ),
            left_panel: LeftPanel::new(
                110.0,
                &[
                    ("\u{1F4CA}", "Chart"),
                    ("\u{1F4C4}", "History"),
                    ("\u{1F3F7}", "Tags"),
                    ("\u{23F0}", "Timers"),
                    ("\u{26ED}", "Setting"),
                ],
            ),
            setting,
            setting_window,
            history,
            history_window: HistoryWindow::new(),
            chart_window: ChartWindow::new(),
            tags_window: TagsWindow::new(),
            timers_window: TimersWindow::new(),
        }
    }

    fn init_total_time(history: &History) -> u64 {
        let end = SystemTime::now();
        let start = get_time_from_offset_days(0);
        history
            .get_records(&start, &end, false)
            .iter()
            .map(|r| r.duration)
            .sum()
    }

    fn on_close(&mut self, ctx: &Context) {
        self.main_panel.stop(
            &self.setting.tags()[self.main_panel.tag_index],
            &mut self.history,
        );

        self.setting.set_tag_index(self.main_panel.tag_index);

        // Save window info
        ctx.viewport(|v| {
            if v.input.viewport().maximized.unwrap_or(false) {
                self.setting.set_window_maximized(true);
            } else {
                self.setting.set_window_maximized(false);
                if let (Some(inner), Some(outer)) =
                    (v.input.viewport().inner_rect, v.input.viewport().outer_rect)
                {
                    self.setting.set_window_info(setting::WindowInfo {
                        x: outer.left(),
                        y: outer.top(),
                        width: inner.width(),
                        height: inner.height(),
                    });
                }
            }
        });
        self.setting.save_cache();
    }
}

fn get_viewport_inner_rect(ctx: &Context) -> Option<egui::Rect> {
    ctx.viewport(|v| v.input.viewport().inner_rect)
}

// ----------------------------------------------------------------------------

struct MainPanel {
    timer_panel: TimerPanel,
    total_time: u64,
    timer: Timer,
    audio: Audio,
    tag_index: usize,
    on_top: bool,
    app_path: PathBuf,
}

impl MainPanel {
    fn new(total_time: u64, tag_index: usize, app_path: PathBuf) -> Self {
        Self {
            total_time,
            timer: Timer::new(),
            timer_panel: TimerPanel::new(),
            audio: Audio::new(),
            tag_index,
            on_top: false,
            app_path,
        }
    }

    fn ui(&mut self, ctx: &Context, ui: &mut Ui, setting: &Setting, history: &mut History) {
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
                    self.timer_buttons_ui(ui, setting, history);
                    ui.add_space(6.0);
                    self.tags_ui(ui, setting.tags());
                    self.timer_panel.ui(ui, self.timer.status(), counter_string);
                },
            );
        });
    }

    fn timer_buttons_ui(&mut self, ui: &mut Ui, setting: &Setting, history: &mut History) {
        let n = setting.timer_list().len();
        ui.add_space(25.0);
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
                                self.stop(&setting.tags()[self.tag_index], history);
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
        let tag = ComboBox::from_id_salt("tag")
            .width(ui.available_width())
            .show_index(ui, &mut self.tag_index, tags.len(), |i| tags[i].to_string());
        tag.on_hover_text("Tag. It's saved in the history when you stop the timer.");
    }

    fn start(&mut self, text: String, t: &TimerSetting, audio_file: Option<&str>) {
        self.timer_panel.set_info(text, t.limit_time);
        self.timer.start(t);
        if t.notify
            && let Some(audio_file) = audio_file
        {
            let name = if audio_file.starts_with("assets/") {
                self.app_path.join(audio_file)
            } else {
                PathBuf::from(audio_file)
            };
            self.audio.schedule_notify(name, t.limit_time * 60);
        }
    }

    fn stop(&mut self, tag: &str, history: &mut History) {
        if let Some((duration, _)) = self.timer.stop() {
            self.total_time += duration;
            history.add_record(self.timer.get_start_time(), duration, tag);
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

    fn change_color(&mut self, ui: &mut Ui) {
        self.frame.fill = match self.status {
            Status::Stopped => Color32::TRANSPARENT,
            Status::Started => MyColor::green(ui),
            Status::TimeOut => MyColor::red(ui),
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

pub fn get_time_from_offset_days(days: i64) -> SystemTime {
    let date = Local::now().date_naive() + chrono::Duration::days(days);
    let time = date.and_hms_opt(0, 0, 0).unwrap();
    time.and_local_timezone(chrono::Local)
        .single()
        .unwrap()
        .into()
}

// ----------------------------------------------------------------------------

struct MyColor {}

impl MyColor {
    const MODAL_BG: Color32 = Color32::from_rgba_premultiplied(70, 70, 70, 225);

    fn red(ui: &mut Ui) -> Color32 {
        if ui.ctx().theme() == Theme::Dark {
            Color32::from_rgb(140, 50, 50)
        } else {
            Color32::from_rgb(255, 120, 110)
        }
    }

    fn green(ui: &mut Ui) -> Color32 {
        if ui.ctx().theme() == Theme::Dark {
            Color32::from_rgb(40, 90, 60)
        } else {
            Color32::from_rgb(140, 235, 130)
        }
    }

    fn background(ui: &mut Ui) -> Color32 {
        if ui.ctx().theme() == Theme::Dark {
            Color32::from_rgb(40, 40, 40)
        } else {
            Color32::from_rgb(200, 200, 200)
        }
    }
}
