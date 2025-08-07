use std::time::SystemTime;

use crate::{
    MyColor,
    history::{History, Record},
};
use chrono::{DateTime, Local};
use eframe::egui::{Id, Label, Modal, RichText, Sense, Sides, Ui};
use egui_extras::{Column, TableBuilder};
use rfd::FileDialog;

pub struct HistoryWindow {
    show: bool,
    records: Vec<Record>,
    delete_index: Option<usize>,
    modify_index: Option<usize>,
    modify_tag: String,
    time_window: TimeWindow,
}

impl HistoryWindow {
    pub fn new() -> Self {
        Self {
            show: false,
            records: Vec::new(),
            delete_index: None,
            modify_index: None,
            modify_tag: String::new(),
            time_window: TimeWindow::Day1,
        }
    }

    pub fn show(&mut self, history: &History) {
        self.time_window = TimeWindow::Day1;
        self.refresh_records(history);
        self.show = true;
    }

    fn close(&mut self) {
        self.show = false;
        self.records = Vec::new();
    }

    fn refresh_records(&mut self, history: &History) {
        let end = SystemTime::now();
        let start = match self.time_window {
            TimeWindow::Day1 => crate::get_time_from_offset_days(0),
            TimeWindow::Day7 => crate::get_time_from_offset_days(-6),
            TimeWindow::All => SystemTime::UNIX_EPOCH,
        };
        self.records = history.get_records(&start, &end, true);
    }

    pub fn ui(&mut self, ui: &mut Ui, history: &mut History) {
        if self.show {
            let modal = Modal::new(Id::new("history")).backdrop_color(MyColor::MODAL_BG);
            let response = modal.show(ui.ctx(), |ui| {
                if let Some(r) = crate::get_viewport_inner_rect(ui.ctx()) {
                    ui.set_max_height(r.height() - 45.0);
                }

                ui.horizontal(|ui| {
                    ui.heading("History");
                    ui.add_space(20.0);
                    if ui
                        .selectable_value(&mut self.time_window, TimeWindow::Day1, "1 Day")
                        .clicked()
                    {
                        self.refresh_records(history);
                    }
                    if ui
                        .selectable_value(&mut self.time_window, TimeWindow::Day7, "7 Days")
                        .clicked()
                    {
                        self.refresh_records(history);
                    }
                    if ui
                        .selectable_value(&mut self.time_window, TimeWindow::All, "All")
                        .clicked()
                    {
                        self.refresh_records(history);
                    }
                    ui.add_space(70.0);
                    if ui.button("Export to CSV").clicked() {
                        if let Some(csv_file) =
                            FileDialog::new().add_filter("CSV", &["csv"]).save_file()
                        {
                            history.export_to_csv(csv_file);
                        }
                    }
                });
                ui.separator();

                TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::remainder().at_least(180.0))
                    .column(Column::remainder().at_least(80.0))
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
                                if ui
                                    .add(Label::new(&record.tag).sense(Sense::click()))
                                    .clicked()
                                {
                                    self.modify_tag = record.tag.clone();
                                    self.modify_index = Some(index);
                                }
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
                self.close();
            }
            self.delete_record_ui(ui, history);
            self.modify_tag_ui(ui, history);
        }
    }

    fn delete_record_ui(&mut self, ui: &mut Ui, history: &mut History) {
        if let Some(index) = self.delete_index {
            let modal = Modal::new(Id::new("modal_delete")).backdrop_color(MyColor::MODAL_BG);
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
                            self.delete_index = None;
                        }

                        if ui.button("No").clicked() {
                            self.delete_index = None;
                        }
                    },
                );
            });
            if response.should_close() {
                self.delete_index = None;
            }
        }
    }

    fn modify_tag_ui(&mut self, ui: &mut Ui, history: &mut History) {
        if let Some(index) = self.modify_index {
            let modal = Modal::new(Id::new("history_modify_tag")).backdrop_color(MyColor::MODAL_BG);
            let response = modal.show(ui.ctx(), |ui| {
                ui.heading("Modify Tag");
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.text_edit_singleline(&mut self.modify_tag);
                    ui.add_space(10.0);
                });
                ui.add_space(10.0);

                Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Save").clicked() {
                            history.modify_tag(&self.records[index].start_time, &self.modify_tag);
                            self.records[index].tag = self.modify_tag.clone();
                            self.modify_index = None;
                        }

                        if ui.button("Cancel").clicked() {
                            self.modify_index = None;
                        }
                    },
                );
            });
            if response.should_close() {
                self.modify_index = None;
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum TimeWindow {
    Day1,
    Day7,
    All,
}
