use std::sync::Arc;

use eframe::egui::{self, Button, Color32, Frame, Id, Modal, Sides, Ui, vec2};

use crate::setting::Setting;

pub struct TagsWindow {
    show: bool,
    reorder: bool,
    modify_index: Option<usize>,
    modify_tag: String,
}

impl TagsWindow {
    pub fn new() -> Self {
        Self {
            show: false,
            reorder: false,
            modify_index: None,
            modify_tag: String::new(),
        }
    }

    pub fn show(&mut self) {
        self.reorder = false;
        self.show = true;
    }

    pub fn ui(&mut self, ui: &mut Ui, setting: &mut Setting) {
        if self.show {
            let mut from = None;
            let mut to = None;

            let modal = Modal::new(Id::new("tags")).backdrop_color(crate::MODAL_BG);
            let response = modal.show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Tags");
                    if ui.button("New").clicked() {
                        self.modify_tag.clear();
                        self.modify_index = Some(usize::MAX);
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
                    ui.set_min_size(vec2(180.0, 64.0));
                    for (i, tag) in setting.tags().iter().enumerate() {
                        if let (Some(a), Some(b)) = self.item_ui(ui, i, tag) {
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
                let tags = setting.mut_tags();
                to -= (from < to) as usize;
                let item = tags.remove(from);
                to = to.min(tags.len());
                tags.insert(to, item);
            }

            if response.should_close() {
                setting.save();
                self.show = false;
            }
        }

        self.modify_tag_ui(ui, setting);
    }

    fn item_ui(&mut self, ui: &mut Ui, i: usize, tag: &str) -> (Option<Arc<usize>>, Option<usize>) {
        let mut from: Option<Arc<usize>> = None;
        let mut to: Option<usize> = None;

        let item_id = Id::new(("tags", tag, i));
        if self.reorder {
            let response = ui
                .dnd_drag_source(item_id, i, |ui| {
                    ui.label(format!("\u{2B0D} {}", tag));
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
            if ui.add(Button::new(tag).frame(false)).clicked() {
                self.modify_tag = tag.to_string();
                self.modify_index = Some(i);
            }
        }
        (from, to)
    }

    fn modify_tag_ui(&mut self, ui: &mut Ui, setting: &mut Setting) {
        if let Some(index) = self.modify_index {
            let modal = Modal::new(Id::new("tags_modify_tag")).backdrop_color(crate::MODAL_BG);
            let response = modal.show(ui.ctx(), |ui| {
                let title = if index == usize::MAX {
                    "New Tag"
                } else {
                    "Modify Tag"
                };
                ui.heading(title);
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
                            if !self.modify_tag.is_empty() {
                                if index == usize::MAX {
                                    setting.mut_tags().push(self.modify_tag.clone());
                                } else {
                                    setting.mut_tags()[index] = self.modify_tag.clone();
                                }
                            }
                            self.modify_index = None;
                        }

                        if ui.button("Cancel").clicked() {
                            self.modify_index = None;
                        }

                        if index != usize::MAX && ui.button("Delete").clicked() {
                            setting.mut_tags().remove(index);
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
