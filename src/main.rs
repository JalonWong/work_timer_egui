#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod timer;

use eframe::egui::{
    self, Align, Button, Color32, FontId, Frame, Layout, RichText, Theme, Visuals, vec2,
};
use timer::{Status, Timer};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_min_inner_size([320.0, 280.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Work Timer",
        options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    )
}

struct MyEguiApp {
    limit_time: u64,
    total_time: u64,
    group: String,
    timer: Timer,
    board: TimerBoard,
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let counter_string = self.timer.update();

            ui.vertical_centered_justified(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("Theme").clicked() {
                        self.toggle_theme(ui);
                    }
                });

                self.board.ui(
                    ui,
                    self.timer.status(),
                    counter_string,
                    &self.group,
                    self.limit_time,
                );

                ui.with_layout(
                    Layout::bottom_up(Align::Center).with_cross_justify(true),
                    |ui| {
                        ui.label(self.total_string());
                        ui.separator();
                        ui.add_space(5.0);
                        if self.timer.status() == Status::Stopped {
                            if ui
                                .add(Button::new("\u{25B6} Start").min_size(vec2(40.0, 40.0)))
                                .clicked()
                            {
                                self.start(ui);
                            }
                        } else {
                            if ui
                                .add(Button::new("\u{23F9} Stop").min_size(vec2(40.0, 40.0)))
                                .clicked()
                            {
                                self.stop(ui);
                            }
                        }
                    },
                );
            });
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

        Self {
            group: "Work".to_string(),
            limit_time: 25,
            total_time: 0,
            timer: Timer::new(),
            board: TimerBoard::new(),
        }
    }

    fn toggle_theme(&mut self, ui: &mut egui::Ui) {
        if ui.visuals().dark_mode {
            ui.ctx().set_theme(Theme::Light);
        } else {
            ui.ctx().set_theme(Theme::Dark);
        }
        self.board.refresh_color(ui);
    }

    fn start(&mut self, ui: &mut egui::Ui) {
        self.timer.start(self.limit_time);
        self.board.update(ui, self.timer.status());
    }

    fn stop(&mut self, ui: &mut egui::Ui) {
        self.total_time += self.timer.stop();
        self.board.update(ui, self.timer.status());
    }

    fn total_string(&self) -> String {
        let time = self.total_time;
        const HOUR_SEC: u64 = 60 * 60;
        if time >= HOUR_SEC {
            format!(
                "Total working time {} h {} m",
                time / HOUR_SEC,
                (time % HOUR_SEC) / 60
            )
        } else {
            format!("Total working time {} m", time / 60)
        }
    }
}

// ----------------------------------------------------------------------------

struct TimerBoard {
    status: Status,
    frame: Frame,
}

impl TimerBoard {
    fn new() -> Self {
        Self {
            status: Status::Stopped,
            frame: Frame::new()
                .inner_margin(10)
                .outer_margin(5)
                .fill(Color32::TRANSPARENT),
        }
    }

    fn get_red(&self, ui: &mut egui::Ui) -> Color32 {
        if ui.ctx().theme() == Theme::Dark {
            Color32::from_rgb(140, 50, 50)
        } else {
            Color32::from_rgb(255, 120, 110)
        }
    }

    fn get_green(&self, ui: &mut egui::Ui) -> Color32 {
        if ui.ctx().theme() == Theme::Dark {
            Color32::from_rgb(50, 90, 30)
        } else {
            Color32::from_rgb(150, 235, 130)
        }
    }

    fn refresh_color(&mut self, ui: &mut egui::Ui) {
        self.frame.fill = match self.status {
            Status::Stopped => Color32::TRANSPARENT,
            Status::Started => self.get_green(ui),
            Status::TimeOut => self.get_red(ui),
        };
    }

    fn update(&mut self, ui: &mut egui::Ui, status: Status) {
        if status != self.status {
            self.status = status;
            self.refresh_color(ui);
        }

        if status != Status::Stopped {
            ui.ctx().request_repaint_after_secs(0.2);
        }
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        status: Status,
        counter_string: String,
        group: &str,
        limit_time: u64,
    ) {
        self.update(ui, status);
        self.frame.show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space((ui.available_height() - 200.0) / 2.0);
                ui.label(group);
                ui.label(RichText::new(counter_string).font(FontId::proportional(80.0)));
                ui.label(format!("Limit {} m", limit_time));
                ui.add_space(ui.available_height() - 80.0);
            });
        });
    }
}
