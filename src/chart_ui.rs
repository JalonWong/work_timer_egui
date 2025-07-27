use crate::history::History;

use chrono::{DateTime, Days, Local, NaiveDate};
use eframe::egui::{Id, Modal, Ui};
use egui_plot::{AxisHints, Bar, BarChart, Legend, Plot};
use std::{
    collections::HashMap,
    time::SystemTime,
};

pub struct ChartWindow {
    show: bool,
    records: Vec<(String, Vec<u64>)>,
    time_window: TimeWindow,
}

impl ChartWindow {
    pub fn new() -> Self {
        Self {
            show: false,
            records: Vec::new(),
            time_window: TimeWindow::Day7,
        }
    }

    pub fn show(&mut self, history: &History) {
        self.time_window = TimeWindow::Day7;
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
            TimeWindow::Day7 => crate::get_time_from_offset_days(-6),
            TimeWindow::Day30 => crate::get_time_from_offset_days(-29),
            TimeWindow::All => SystemTime::UNIX_EPOCH,
        };

        let records = history.get_records(&start, &end, false);

        // Statistic
        let mut first_date = None;
        let mut tag_record_map: HashMap<&str, HashMap<NaiveDate, u64>> = HashMap::new();
        for record in records.iter() {
            let local_time: DateTime<Local> = record.start_time.into();
            let date = local_time.date_naive();
            if first_date.is_none() {
                first_date = Some(date);
            }
            tag_record_map
                .entry(&record.tag)
                .or_insert_with(HashMap::new)
                .entry(date)
                .and_modify(|duration| *duration += record.duration)
                .or_insert(record.duration);
        }

        // Supplement zero
        if let Some(mut date) = first_date {
            while date <= Local::now().date_naive() {
                for (_, date_duration_map) in tag_record_map.iter_mut() {
                    date_duration_map.entry(date).or_insert(0);
                }
                date = date.checked_add_days(Days::new(1)).unwrap();
            }
        }

        // Convert map to sorted vector
        self.records.clear();
        let mut tags: Vec<_> = tag_record_map.keys().collect();
        tags.sort();
        for tag in tags {
            let date_duration_map = tag_record_map.get(tag).unwrap();

            let mut bars = Vec::new();
            let mut dates: Vec<_> = date_duration_map.keys().collect();
            dates.sort();
            for date in dates.iter().rev() {
                let duration = date_duration_map.get(date).unwrap();
                bars.push(*duration);
            }
            self.records.push((tag.to_string(), bars));
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, history: &History) {
        if self.show {
            let modal = Modal::new(Id::new("chart")).backdrop_color(crate::MODAL_BG);
            let response = modal.show(ui.ctx(), |ui| {
                if let Some(r) = crate::get_viewport_inner_rect(ui.ctx()) {
                    ui.set_height(r.height() - 45.0);
                    ui.set_width(r.width() - 45.0);
                }

                ui.horizontal(|ui| {
                    ui.heading("Chart");
                    ui.add_space(20.0);
                    if ui
                        .selectable_value(&mut self.time_window, TimeWindow::Day7, "7 Days")
                        .clicked()
                    {
                        self.refresh_records(history);
                    }
                    if ui
                        .selectable_value(&mut self.time_window, TimeWindow::Day30, "30 Days")
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
                    ui.add_space(ui.available_width() - 30.0);
                    if ui.button("\u{274C}").clicked() {
                        self.close();
                    }
                });
                ui.separator();

                let mut charts = Vec::new();
                for (tag, durations) in self.records.iter() {
                    let bars = durations
                        .iter()
                        .enumerate()
                        .map(|(i, duration)| Bar::new(-(i as f64) + 0.5, *duration as f64 / 3600.0))
                        .collect();

                    let name = tag.clone();
                    let mut chart =
                        BarChart::new(tag, bars)
                            .width(1.0)
                            .element_formatter(Box::new(move |b, _| {
                                format!(
                                    "{}\n{}\n{:.1} hours",
                                    name,
                                    x_to_date(b.argument).format("%Y-%m-%d"),
                                    b.value
                                )
                            }));
                    if !charts.is_empty() {
                        let others: Vec<&BarChart> = charts.iter().map(|c| c).collect();
                        chart = chart.stack_on(&others);
                    }
                    charts.push(chart);
                }

                let x_axes = vec![
                    AxisHints::new_x()
                        .label("Date")
                        .formatter(|mark, _| x_to_date(mark.value).format("%m-%d").to_string()),
                ];
                let y_axes = vec![AxisHints::new_y().label("Hours")];

                Plot::new("Stacked Bar Chart Demo")
                    .legend(Legend::default())
                    .data_aspect(1.0)
                    .allow_scroll([true, false])
                    .allow_drag(true)
                    .custom_x_axes(x_axes)
                    .custom_y_axes(y_axes)
                    .label_formatter(|_, val| {
                        format!(
                            "{}\n{:.1} hours",
                            x_to_date(val.x).format("%Y-%m-%d"),
                            val.y
                        )
                    })
                    .show(ui, |plot_ui| {
                        for c in charts {
                            plot_ui.bar_chart(c);
                        }
                    })
                    .response
            });
            if response.should_close() {
                self.close();
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum TimeWindow {
    Day7,
    Day30,
    All,
}

fn x_to_date(x: f64) -> NaiveDate {
    let mut date = Local::now().date_naive();
    if x < 0.0 {
        date = date
            .checked_sub_days(Days::new((-x + 0.99999) as u64))
            .unwrap();
    } else {
        date = date.checked_add_days(Days::new(x as u64)).unwrap();
    }
    date
}
