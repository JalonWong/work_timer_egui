use eframe::egui::{Button, Frame, Margin, Response, SidePanel, Ui, vec2};

struct ButtonInfo {
    icon: String,
    text: String,
}

pub struct LeftPanel {
    is_expanded: bool,
    btn_list: Vec<ButtonInfo>,
    width: f32,
}

impl LeftPanel {
    pub fn new(width: f32, list: &[(&str, &str)]) -> Self {
        Self {
            is_expanded: false,
            width,
            btn_list: list
                .iter()
                .map(|i| ButtonInfo {
                    icon: i.0.to_string(),
                    text: i.1.to_string(),
                })
                .collect(),
        }
    }

    fn add_button(&self, ui: &mut Ui, text: String, tip: &str) -> Response {
        let btn_width = if self.is_expanded {
            self.width - 9.0
        } else {
            28.0
        };
        let btn = Button::new(text).min_size(vec2(btn_width, 30.0));
        let mut rst = ui.add(btn);
        if !self.is_expanded {
            rst = rst.on_hover_text(tip);
        }
        rst
    }

    fn add_list_button(&mut self, ui: &mut Ui, out: &mut Vec<usize>) {
        for (i, btn) in self.btn_list.iter().enumerate() {
            let mut text = btn.icon.clone();
            if self.is_expanded {
                text.push_str(" ");
                text.push_str(&btn.text);
            }

            if self.add_button(ui, text, &btn.text).clicked() {
                out.push(i);
            }
        }
    }

    fn expand_btn(&mut self, ui: &mut Ui) {
        let text = if self.is_expanded {
            "\u{2B05} Collapse"
        } else {
            "\u{27A1}"
        };

        if self.add_button(ui, text.to_string(), "Expand").clicked() {
            self.is_expanded = !self.is_expanded;
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Vec<usize> {
        let mut btn_status = Vec::new();
        let frame = Frame::default()
            .inner_margin(Margin {
                left: if self.is_expanded { 1 } else { 0 },
                right: 7,
                top: 2,
                bottom: 0,
            })
            .outer_margin(0);
        SidePanel::left("left_panel")
            .resizable(false)
            .frame(frame)
            .exact_width(if self.is_expanded { self.width } else { 36.0 })
            .show_inside(ui, |ui| {
                if self.is_expanded {
                    ui.vertical(|ui| {
                        self.expand_btn(ui);
                        self.add_list_button(ui, &mut btn_status);
                    });
                } else {
                    ui.vertical_centered(|ui| {
                        self.expand_btn(ui);
                        self.add_list_button(ui, &mut btn_status);
                    });
                }
            });
        btn_status
    }
}
