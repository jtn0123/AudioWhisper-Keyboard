use super::{Action, App, Control};
use crate::{
    panels::{Command, Item, Panel, rect},
    style::Style,
    text::Text,
};
use keys::Rect;
impl App {
    pub(super) fn paint_box(&self, p: &mut Panel, b: Rect, color: u32, radius: f32) {
        let mut item = Item::new(
            -10000 - p.items.len() as i32,
            b,
            String::new(),
            "settings-box",
            Command::None,
        );
        let style = Style {
            background: color,
            corners: [radius; 4],
            ..Default::default()
        };
        item.styles = [style.clone(), style];
        p.items.push(item);
    }
    pub(super) fn paint_text(
        &self,
        p: &mut Panel,
        label: &str,
        b: Rect,
        size: f32,
        color: u32,
        clip: Rect,
    ) {
        let mut item = Item::new(
            -10000 - p.items.len() as i32,
            b,
            label.into(),
            "settings-subheader",
            Command::None,
        );
        item.clip = clip;
        item.text = Style {
            size,
            line_height: self.line_height(size),
            letter_spacing: self.letter_spacing(size),
            foreground: color,
            ..Default::default()
        };
        p.items.push(item);
    }
    pub(super) fn add(&mut self, p: &mut Panel, mut c: Control, clip: Rect, text: &mut Text) {
        let d = self.config[2];
        let b = c.bounds;
        let color = self.enabled_color(self.colors[1], c.enabled);
        if self.reveal_focus && c.id == self.focus {
            if b.top < clip.top {
                self.reveal_target = Some(self.scroll() + b.top - clip.top);
            } else if b.bottom > clip.bottom {
                self.reveal_target = Some(self.scroll() + b.bottom - clip.bottom);
            }
        }
        if c.id == self.focus {
            self.paint_box(p, b, self.colors[4], 8.0 * d);
        }
        if c.id == 1 || c.id == 3 {
            self.paint_icon(
                p,
                if c.id == 1 { "back" } else { "picker" },
                rect(
                    b.left + if c.id == 3 { 12.0 } else { 16.0 } * d,
                    b.top + (b.height() - 24.0 * d) / 2.0,
                    24.0 * d,
                    24.0 * d,
                ),
                color,
                clip,
            );
        } else if c.role == "edit" {
            self.paint_editor(p, &c, text, clip);
        } else if self.modal.is_some() && (5..=7).contains(&c.id) {
            self.paint_button(p, &c.label, b, text, clip);
        } else if c.role == "slider" {
            self.paint_slider(p, &c, clip);
        } else {
            let only_switch = matches!(c.action, Action::Switch(_) | Action::ModalSwitch);
            let is_home =
                matches!(c.action, Action::Route(_)) && self.screen_index() == 0 && c.id >= 100;
            let screen_row = (1000..3000).contains(&c.id) || is_home;
            let left = b.left
                + if c.role == "radio" {
                    52.0
                } else if screen_row {
                    56.0
                } else {
                    16.0
                } * d;
            let right = b.right
                - if matches!(c.action, Action::Row(i) if !super::catalogue::KEYBOARD[i].switch.is_empty())
                {
                    88.0 * d
                } else if c.role == "toggle" && !only_switch {
                    80.0 * d
                } else {
                    16.0 * d
                };
            let label = if screen_row {
                self.row_text(text, &c.label, self.sp(16), right - left, 1)
            } else {
                c.label.clone()
            };
            let summary = if screen_row {
                self.row_text(text, &c.summary, self.sp(14), right - left, 2)
            } else {
                c.summary.clone()
            };
            let lh = self.measure(text, &label, self.sp(16), (right - left).max(1.0));
            let sh = self.measure(text, &summary, self.sp(14), (right - left).max(1.0));
            let top = b.top
                + if c.role == "radio" && !summary.is_empty() {
                    8.0 * d
                } else if screen_row && sh > self.sp(20) + 0.5 {
                    12.0 * d
                } else {
                    ((b.height() - lh - sh) / 2.0).max(0.0)
                };
            if is_home {
                self.paint_icon(
                    p,
                    &format!("home{}", c.id - 100),
                    rect(
                        b.left + 16.0 * d,
                        b.top + (b.height() - 24.0 * d) / 2.0,
                        24.0 * d,
                        24.0 * d,
                    ),
                    self.colors[2],
                    clip,
                );
            }
            if !only_switch {
                self.paint_text(
                    p,
                    &label,
                    rect(left, top, (right - left).max(1.0), lh),
                    self.sp(16),
                    if c.role == "heading" {
                        self.colors[9]
                    } else if c.role == "radio" {
                        self.enabled_color(self.colors[2], c.enabled)
                    } else {
                        color
                    },
                    clip,
                );
                if c.role == "heading" {
                    p.items.last_mut().unwrap().text.weight = 700;
                }
                if !c.summary.is_empty() {
                    self.paint_text(
                        p,
                        &summary,
                        rect(
                            left,
                            top + lh,
                            (right - left).max(1.0),
                            b.height() - 12.0 * d - lh,
                        ),
                        self.sp(14),
                        self.enabled_color(self.colors[2], c.enabled),
                        clip,
                    );
                }
            }
            if let Some(checked) = c.checked.filter(|_| c.role == "toggle") {
                self.paint_switch(
                    p,
                    if screen_row && sh > self.sp(20) + 0.5 {
                        rect(b.left, b.top, b.width(), 72.0 * d)
                    } else {
                        b
                    },
                    checked,
                    c.enabled,
                    matches!(c.action, Action::Switch(_)),
                    clip,
                );
            }
            if c.role == "radio" {
                let cx = b.left + 28.0 * d;
                let cy = b.top + 20.0 * d;
                let ink = self.enabled_color(
                    if c.selected {
                        self.colors[3]
                    } else {
                        self.colors[2]
                    },
                    c.enabled,
                );
                self.paint_box(
                    p,
                    rect(cx - 10.0 * d, cy - 10.0 * d, 20.0 * d, 20.0 * d),
                    ink,
                    10.0 * d,
                );
                self.paint_box(
                    p,
                    rect(cx - 8.0 * d, cy - 8.0 * d, 16.0 * d, 16.0 * d),
                    self.colors[11],
                    8.0 * d,
                );
                if c.selected {
                    self.paint_box(
                        p,
                        rect(cx - 6.0 * d, cy - 6.0 * d, 12.0 * d, 12.0 * d),
                        ink,
                        6.0 * d,
                    );
                }
            }
        }
        c.bounds = crate::panel_layout::intersection(c.bounds, clip);
        self.controls.push(c);
    }
    pub(super) fn paint_icon(&self, p: &mut Panel, name: &str, b: Rect, color: u32, clip: Rect) {
        if let Some(icon) = self.strings.get(&format!("@icon:{name}")) {
            let mut item = Item::new(
                -10000 - p.items.len() as i32,
                b,
                String::new(),
                "settings-icon",
                Command::None,
            );
            item.icon = icon.clone();
            item.icon_style = Style {
                size: b.height(),
                foreground: color,
                ..Default::default()
            };
            item.clip = clip;
            p.items.push(item);
        }
    }
}
