use std::time::{Duration, SystemTime};

use crate::history::{History, Record};
use chrono::{DateTime, Local};
use eframe::egui::{Id, Modal, RichText, Sides, Ui};
use egui_extras::{Column, TableBuilder};

pub struct HistoryWindow {
    show: bool,
    records: Vec<Record>,
    delete_index: Option<usize>,
}

impl HistoryWindow {
    pub fn new() -> Self {
        Self {
            show: false,
            records: Vec::new(),
            delete_index: None,
        }
    }

    pub fn show(&mut self, history: &History) {
        let end = SystemTime::now();
        let start = end
            .checked_sub(Duration::from_secs(3 * 24 * 60 * 60))
            .unwrap();
        self.records = history.get_records(&start, &end, true);
        self.show = true;
    }

    pub fn ui(&mut self, ui: &mut Ui, history: &mut History) {
        if self.show {
            let modal = Modal::new(Id::new("history")).backdrop_color(crate::MODAL_BG);
            let response = modal.show(ui.ctx(), |ui| {
                if let Some(r) = crate::get_viewport_inner_rect(ui.ctx()) {
                    ui.set_max_height(r.height() * 0.85);
                }

                ui.heading("History");
                ui.separator();

                TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::remainder().at_least(180.0))
                    .column(Column::auto())
                    .column(Column::remainder().at_least(40.0))
                    .column(Column::remainder().at_least(40.0))
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Start Time");
                        });
                        header.col(|ui| {
                            ui.strong("Duration");
                        });
                        header.col(|ui| {
                            ui.strong("Tag");
                        });
                        header.col(|ui| {
                            ui.strong("Del");
                        });
                    })
                    .body(|body| {
                        body.rows(18.0, self.records.len(), |mut row| {
                            let index = row.index();
                            let record = &self.records[row.index()];
                            row.col(|ui| {
                                let local_time: DateTime<Local> = record.start_time.into();
                                let text = local_time.format("%Y-%m-%d %H:%M:%S").to_string();
                                ui.label(RichText::new(text).monospace());
                            });
                            row.col(|ui| {
                                let text = crate::timer::secs_to_string(record.duration, "");
                                ui.label(RichText::new(text).monospace());
                            });
                            row.col(|ui| {
                                ui.label(&record.tag);
                            });
                            row.col(|ui| {
                                if ui.button("\u{274E}").clicked() {
                                    self.delete_index = Some(index);
                                }
                            });
                        });
                    });
            });
            if response.should_close() {
                self.show = false;
            }
            self.delete_record_ui(ui, history);
        }
    }

    fn delete_record_ui(&mut self, ui: &mut Ui, history: &mut History) {
        if let Some(index) = self.delete_index {
            let modal = Modal::new(Id::new("delete")).backdrop_color(crate::MODAL_BG);
            let response = modal.show(ui.ctx(), |ui| {
                ui.set_width(350.0);
                ui.heading("Are you sure you want to delete this record?");
                ui.add_space(32.0);
                Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Yes").clicked() {
                            history.remove(&self.records[index].start_time);
                            self.records.remove(index);
                            self.delete_index.take();
                        }

                        if ui.button("No").clicked() {
                            self.delete_index.take();
                        }
                    },
                );
            });
            if response.should_close() {
                self.delete_index.take();
            }
        }
    }
}
