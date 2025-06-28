use std::time::{Duration, SystemTime};

use crate::history::{History, Record};
use chrono::{DateTime, Local};
use eframe::egui::{FontId, Id, Modal, RichText, Ui};
use egui_extras::{Column, TableBuilder};

pub struct HistoryWindow {
    show: bool,
    records: Vec<Record>,
}

impl HistoryWindow {
    pub fn new() -> Self {
        Self {
            show: false,
            records: Vec::new(),
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
                ui.heading("History");
                ui.separator();

                TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::remainder().at_least(180.0))
                    .column(Column::auto())
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
                    })
                    .body(|body| {
                        body.rows(18.0, self.records.len(), |mut row| {
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
                        });
                    });
            });
            if response.should_close() {
                self.show = false;
            }
        }
    }
}
