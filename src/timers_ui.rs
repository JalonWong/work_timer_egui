use std::sync::Arc;

use eframe::egui::{self, Button, Color32, Frame, Id, Modal, Sides, Ui, vec2};

use crate::{
    MyColor,
    setting::{Setting, TimerSetting},
};

pub struct TimersWindow {
    show: bool,
    reorder: bool,
    delete_index: Option<usize>,
    timer_info_list: Vec<TimerInfo>,
}

impl TimersWindow {
    pub fn new() -> Self {
        Self {
            show: false,
            reorder: false,
            delete_index: None,
            timer_info_list: Vec::new(),
        }
    }

    pub fn show(&mut self, setting: &Setting) {
        self.refresh_info(setting);
        self.reorder = false;
        self.show = true;
    }

    fn refresh_info(&mut self, setting: &Setting) {
        self.timer_info_list.clear();
        for timer in setting.timer_list() {
            self.timer_info_list.push(TimerInfo::from_setting(timer));
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, setting: &mut Setting) {
        if self.show {
            let mut from = None;
            let mut to = None;

            let modal = Modal::new(Id::new("timers")).backdrop_color(MyColor::MODAL_BG);
            let response = modal.show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Timers");
                    if ui.button("New").clicked() {
                        setting.add_timer(TimerSetting::new());
                        self.refresh_info(setting);
                    }
                    if ui
                        .add(Button::new("Reorder").selected(self.reorder))
                        .clicked()
                    {
                        self.reorder = !self.reorder;
                    }
                });
                ui.separator();

                let frame = Frame::default().inner_margin(4.0);
                let (_, dropped_payload) = ui.dnd_drop_zone::<usize, ()>(frame, |ui| {
                    ui.set_min_size(vec2(200.0, 64.0));
                    for (i, timer) in setting.mut_timer_list().iter_mut().enumerate() {
                        if let (Some(a), Some(b)) = self.drag_item_ui(ui, i, timer) {
                            from = Some(a);
                            to = Some(b);
                        }
                    }
                });

                if let Some(dragged_payload) = dropped_payload {
                    // The user dropped onto the column, but not on any one item.
                    from = Some(dragged_payload);
                    to = Some(usize::MAX);
                }
            });

            if let (Some(from), Some(mut to)) = (from, to) {
                let from: usize = *from;
                let timers = setting.mut_timer_list();
                to -= (from < to) as usize;
                let item = timers.remove(from);
                to = to.min(timers.len());
                timers.insert(to, item);
                self.refresh_info(setting);
            }

            if response.should_close() {
                setting.save();
                self.show = false;
            }
        }

        self.delete_timer_window_ui(ui, setting);
    }

    fn drag_item_ui(
        &mut self,
        ui: &mut Ui,
        i: usize,
        timer: &mut TimerSetting,
    ) -> (Option<Arc<usize>>, Option<usize>) {
        let mut from: Option<Arc<usize>> = None;
        let mut to: Option<usize> = None;

        let item_id = Id::new(("timers", timer.name.as_str(), i));
        if self.reorder {
            let response = ui
                .dnd_drag_source(item_id, i, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("\u{2B0D}");
                        self.item_ui(ui, i, timer);
                    });
                })
                .response;
            if let (Some(pointer), Some(hovered_payload)) = (
                ui.input(|i| i.pointer.interact_pos()),
                response.dnd_hover_payload::<usize>(),
            ) {
                let rect = response.rect;
                // Preview insertion:
                let stroke = egui::Stroke::new(1.0, Color32::WHITE);
                let insert_row_idx = if *hovered_payload == i {
                    // We are dragged onto ourselves
                    ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                    i
                } else if pointer.y < rect.center().y {
                    // Above us
                    ui.painter().hline(rect.x_range(), rect.top(), stroke);
                    i
                } else {
                    // Below us
                    ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                    i + 1
                };

                if let Some(dragged_payload) = response.dnd_release_payload() {
                    // The user dropped onto this item.
                    from = Some(dragged_payload);
                    to = Some(insert_row_idx);
                }
            }
        } else {
            self.item_ui(ui, i, timer);
        }
        (from, to)
    }

    fn item_ui(&mut self, ui: &mut Ui, i: usize, timer: &mut TimerSetting) {
        let timer_info = &mut self.timer_info_list[i];
        Frame::default()
            .fill(MyColor::background(ui))
            .inner_margin(4.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.add_sized([15.0, 20.0], egui::TextEdit::singleline(&mut timer.icon));
                        if ui.text_edit_singleline(&mut timer_info.name).lost_focus() {
                            if timer_info.name.is_empty() {
                                timer_info.name = timer.name.clone();
                            } else {
                                timer.name = timer_info.name.clone();
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Limit time:");
                        if ui
                            .text_edit_singleline(&mut timer_info.limit_time)
                            .lost_focus()
                        {
                            if let Ok(mut value) = timer_info.limit_time.parse::<u64>() {
                                if value == 0 {
                                    value = 1;
                                    timer_info.limit_time = value.to_string();
                                }
                                timer.limit_time = value;
                            } else {
                                timer_info.limit_time = timer.limit_time.to_string();
                            }
                        }
                        ui.label("minutes");
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut timer.for_work, "Work timer");
                        ui.checkbox(&mut timer.count_up, "Count up");
                        ui.checkbox(&mut timer.notify, "Notify when timeout");
                    });
                    if ui.button("Delete").clicked() {
                        self.delete_index = Some(i);
                    }
                });
            });
    }

    fn delete_timer_window_ui(&mut self, ui: &mut Ui, setting: &mut Setting) {
        if let Some(index) = self.delete_index {
            let modal = Modal::new(Id::new("timer_delete")).backdrop_color(MyColor::MODAL_BG);
            let response = modal.show(ui.ctx(), |ui| {
                ui.set_width(350.0);
                ui.heading("Are you sure you want to delete this timer?");
                let tiemr = &setting.timer_list()[index];
                ui.add_space(10.0);
                ui.label(format!("{} {}", tiemr.icon, tiemr.name));
                ui.add_space(20.0);
                Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Yes").clicked() {
                            setting.mut_timer_list().remove(index);
                            self.refresh_info(setting);
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
}

struct TimerInfo {
    name: String,
    limit_time: String,
}

impl TimerInfo {
    fn from_setting(setting: &TimerSetting) -> Self {
        Self {
            name: setting.name.clone(),
            limit_time: setting.limit_time.to_string(),
        }
    }
}
