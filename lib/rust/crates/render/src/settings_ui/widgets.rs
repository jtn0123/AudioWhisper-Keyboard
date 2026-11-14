use super::layout::height;
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
            foreground: color,
            ..Default::default()
        };
        p.items.push(item);
    }
    pub(super) fn add(&mut self, p: &mut Panel, mut c: Control, clip: Rect, text: &mut Text) {
        let d = self.config[2];
        let f = self.config[3];
        let b = c.bounds;
        let color = if c.enabled {
            self.colors[1]
        } else {
            self.colors[2] & 0x00ffffff | 0x66000000
        };
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
                    b.left + 16.0 * d,
                    b.top + (b.height() - 24.0 * d) / 2.0,
                    24.0 * d,
                    24.0 * d,
                ),
                color,
                clip,
            );
        } else if c.role == "edit" {
            self.paint_editor(p, &c, text, clip);
        } else if c.role == "slider" {
            let [min, max, value] = c.range.unwrap();
            let y = (b.top + b.bottom) / 2.0;
            self.paint_box(
                p,
                rect(b.left, y - 2.0 * d, b.width(), 4.0 * d),
                self.colors[5],
                2.0 * d,
            );
            let x = b.left + b.width() * (value - min) / (max - min);
            self.paint_box(
                p,
                rect(b.left, y - 2.0 * d, x - b.left, 4.0 * d),
                self.colors[3],
                2.0 * d,
            );
            self.paint_box(
                p,
                rect(x - 10.0 * d, y - 10.0 * d, 20.0 * d, 20.0 * d),
                self.colors[3],
                10.0 * d,
            );
        } else {
            let only_switch = matches!(c.action, Action::Switch(_) | Action::ModalSwitch);
            let is_home =
                matches!(c.action, Action::Route(_)) && self.screen_index() == 0 && c.id >= 100;
            let left = b.left
                + if is_home || (self.modal.is_none() && c.id >= 1000 && c.role != "heading") {
                    56.0
                } else {
                    16.0
                } * d;
            let right = b.right
                - if c.role == "toggle" && !only_switch {
                    72.0 * d
                } else {
                    16.0 * d
                };
            let lh = height(text, &c.label, 16.0 * d * f, (right - left).max(1.0));
            let sh = height(text, &c.summary, 14.0 * d * f, (right - left).max(1.0));
            let top = b.top + ((b.height() - lh - sh) / 2.0).max(0.0);
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
                    &c.label,
                    rect(left, top, (right - left).max(1.0), lh),
                    if c.role == "heading" { 14.0 } else { 16.0 } * d * f,
                    if c.role == "heading" {
                        self.colors[3]
                    } else {
                        color
                    },
                    clip,
                );
                if !c.summary.is_empty() {
                    self.paint_text(
                        p,
                        &c.summary,
                        rect(
                            left,
                            top + lh,
                            (right - left).max(1.0),
                            b.height() - 12.0 * d - lh,
                        ),
                        14.0 * d * f,
                        self.colors[2],
                        clip,
                    );
                }
            }
            if let Some(checked) = c.checked.filter(|_| c.role == "toggle") {
                let x = b.right - 60.0 * d;
                let y = (b.top + b.bottom) / 2.0;
                self.paint_box(
                    p,
                    rect(x, y - 16.0 * d, 52.0 * d, 32.0 * d),
                    if checked {
                        self.colors[3]
                    } else {
                        self.colors[2]
                    },
                    16.0 * d,
                );
                self.paint_box(
                    p,
                    rect(
                        x + if checked { 26.0 } else { 4.0 } * d,
                        y - 12.0 * d,
                        24.0 * d,
                        24.0 * d,
                    ),
                    if checked {
                        self.colors[4]
                    } else {
                        self.colors[5]
                    },
                    12.0 * d,
                );
            }
            if c.role == "radio" {
                self.paint_text(
                    p,
                    if c.selected { "◉" } else { "○" },
                    rect(b.right - 40.0 * d, top, 24.0 * d, 24.0 * d),
                    22.0 * d,
                    self.colors[3],
                    clip,
                );
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
