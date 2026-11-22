use super::{Action, App, Control, catalogue::Kind};
use crate::{
    panels::{Command, Item, Panel, rect},
    style::Style,
    text::Text,
};
use keys::Rect;
impl App {
    pub(super) fn paint_box(&self, p: &mut Panel, b: Rect, color: u32, radius: f32) {
        self.paint_clipped_box(p, b, color, radius, b);
    }
    /// A box drawn only inside `clip`, for shapes inside a scrolling list or dialog.
    pub(super) fn paint_clipped_box(
        &self,
        p: &mut Panel,
        b: Rect,
        color: u32,
        radius: f32,
        clip: Rect,
    ) {
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
        item.clip = clip;
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
                self.reveal_target = Some(self.shown_scroll() + b.top - clip.top);
            } else if b.bottom > clip.bottom {
                self.reveal_target = Some(self.shown_scroll() + b.bottom - clip.bottom);
            }
        }
        // A text field shows keyboard focus with its own focused outline.
        if c.id == self.focus && !matches!(c.action, Action::Field(_)) {
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
                // An M3 top bar's navigation icon is onSurface; a text field's trailing icon is
                // onSurfaceVariant, focused or not.
                if c.id == 1 { color } else { self.colors[2] },
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
            let is_home = matches!(c.action, Action::Route(_)) && self.is_home() && c.id >= 100;
            let screen_row = (1000..3000).contains(&c.id) || is_home;
            let row = (1000..2000).contains(&c.id).then(|| (c.id - 1000) as usize);
            let [inset_left, inset_right] = match row {
                Some(i) => self.row_insets(i),
                None if is_home => [56.0 * d, 16.0 * d],
                None if c.role == "radio" => self.radio_insets(),
                None => [16.0 * d, 16.0 * d],
            };
            let left = b.left + inset_left;
            let right = b.right - inset_right;
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
                } else if screen_row && self.multiline(sh) {
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
            if let Some(icon) = row.map(|i| self.rows()[i].icon).filter(|n| !n.is_empty()) {
                // M3 ListItem's leading content: centred, or top-aligned in a three-line item.
                let y = if self.multiline(sh) {
                    b.top + 12.0 * d
                } else {
                    b.top + (b.height() - 24.0 * d) / 2.0
                };
                self.paint_icon(
                    p,
                    icon,
                    rect(b.left + 16.0 * d, y, 24.0 * d, 24.0 * d),
                    self.enabled_color(self.colors[2], c.enabled),
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
            let fixed = row.is_some_and(|i| matches!(self.rows()[i].kind, Kind::Fixed));
            if let Some(checked) = c
                .checked
                .filter(|_| c.role == "toggle")
                .or(fixed.then_some(true))
            {
                self.paint_switch(
                    p,
                    if screen_row && self.multiline(sh) {
                        // A three-line item's trailing content sits 12 dp below its top.
                        let target = self.px(48.0) + 2.0 * self.px(12.0);
                        rect(b.left, b.top, b.width(), target)
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
                self.paint_clipped_box(
                    p,
                    rect(cx - 10.0 * d, cy - 10.0 * d, 20.0 * d, 20.0 * d),
                    ink,
                    10.0 * d,
                    clip,
                );
                self.paint_clipped_box(
                    p,
                    rect(cx - 8.0 * d, cy - 8.0 * d, 16.0 * d, 16.0 * d),
                    self.colors[11],
                    8.0 * d,
                    clip,
                );
                if c.selected {
                    self.paint_clipped_box(
                        p,
                        rect(cx - 6.0 * d, cy - 6.0 * d, 12.0 * d, 12.0 * d),
                        ink,
                        6.0 * d,
                        clip,
                    );
                }
            }
        }
        c.bounds = crate::panel_layout::intersection(c.bounds, clip);
        self.controls.push(c);
    }
    pub(super) fn paint_icon(&self, p: &mut Panel, name: &str, b: Rect, color: u32, clip: Rect) {
        if let Some(icon) = self.strings.get(&format!("@icon:{name}")) {
            self.paint_path(p, icon, b, color, clip);
        }
    }
    /// An icon's path data, as the keyboard's icons reach Rust, at `b` in `color`.
    pub(super) fn paint_path(&self, p: &mut Panel, path: &str, b: Rect, color: u32, clip: Rect) {
        let mut item = Item::new(
            -10000 - p.items.len() as i32,
            b,
            String::new(),
            "settings-icon",
            Command::None,
        );
        item.icon = path.into();
        item.icon_style = Style {
            size: b.height(),
            foreground: color,
            ..Default::default()
        };
        item.clip = clip;
        p.items.push(item);
    }
}
