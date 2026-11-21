use super::{Action, App, Control, catalogue::Kind};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
impl App {
    pub(super) fn dialog(&mut self, p: &mut Panel, text: &mut Text) {
        if self.licence_modal() {
            return self.licence_dialog(p, text);
        }
        let m = self.modal.as_ref().unwrap().clone();
        let r = self.rows()[m.row];
        let [w, h, d, _] = self.config;
        let window = if self.dialog_window_width > 0.0 {
            self.dialog_window_width
        } else {
            320.0 * d
        };
        let width = if matches!(r.kind, Kind::Time) {
            self.time_width()
        } else {
            (window - 32.0 * d)
                .max(280.0 * d)
                .min(560.0 * d)
                .min(w - 32.0 * d)
        };
        let left = (w - width) / 2.0;
        // Compose's AlertDialog keeps 16 dp from the window edge on tall screens (measured on device: 42 px at 2.625).
        let padding = if h < 600.0 * d { 8.0 * d } else { 16.0 * d };
        let (button_height, button_widths, footer) = self.button_metrics(width, text);
        let title = self.label(match r.kind {
            Kind::Confirm(ask) => ask.title,
            _ => r.label,
        });
        let title_width = width
            - if r.switch.is_empty() {
                48.0 * d
            } else {
                116.0 * d
            };
        let drawn_title = self.row_text(text, &title, self.sp(24), title_width, 1);
        let title_height = self.measure(text, &drawn_title, self.sp(24), title_width);
        let th = if r.switch.is_empty() {
            title_height + 40.0 * d
        } else {
            title_height.max(48.0 * d) + 32.0 * d
        };
        let rows = self.dialog_rows(width, text);
        let content: f32 = rows.iter().map(|row| row.2).sum();
        let dialog_height = (th + content + footer).min((h - 2.0 * padding).max(1.0));
        let top = (h - dialog_height) / 2.0;
        let bottom = top + dialog_height;
        let area = rect(left, top, width, dialog_height);
        self.paint_box(p, rect(0.0, 0.0, w, h), 0x99000000, 0.0);
        self.paint_box(p, area, self.colors[11], 28.0 * d);
        self.controls.clear();
        self.paint_text(
            p,
            &drawn_title,
            rect(left + 24.0 * d, top + 24.0 * d, title_width, th),
            self.sp(24),
            self.colors[1],
            area,
        );
        let clip = rect(left, top + th, width, (bottom - top - th - footer).max(1.0));
        let mut y = clip.top - super::shown(m.scroll);
        self.controls.push(Control {
            id: 4,
            bounds: clip,
            label: title.clone(),
            summary: String::new(),
            role: "scroll",
            enabled: true,
            checked: None,
            selected: false,
            action: Action::None,
            range: None,
        });
        if !r.switch.is_empty() {
            self.add(
                p,
                Control {
                    id: 8,
                    bounds: rect(left + width - 72.0 * d, top + 16.0 * d, 64.0 * d, 48.0 * d),
                    label: title.clone(),
                    summary: String::new(),
                    role: "toggle",
                    enabled: true,
                    checked: Some(m.enabled),
                    selected: false,
                    action: Action::ModalSwitch,
                    range: None,
                },
                area,
                text,
            );
        }
        match r.kind {
            Kind::Choice(_) => {
                for (j, choice) in self.choices(m.row).iter().enumerate() {
                    let (label, summary, rh) = rows[j].clone();
                    self.add(
                        p,
                        Control {
                            id: 3000 + j as i32,
                            bounds: rect(left + self.px(8.0), y, width - 2.0 * self.px(8.0), rh),
                            label,
                            summary,
                            role: "radio",
                            enabled: m.enabled,
                            checked: Some(m.selected == *choice),
                            selected: m.selected == *choice,
                            action: Action::Choose(choice.clone()),
                            range: None,
                        },
                        clip,
                        text,
                    );
                    y += rh;
                }
            }
            Kind::Slider(s) => {
                for (slot, value) in m.values.iter().enumerate() {
                    let key = if slot == 0 { r.key } else { s.second };
                    let pref = settings::catalogue::get(&settings::text(key)).unwrap();
                    let (orientation, value_label, row_height) = &rows[slot];
                    y += if slot == 0 { 8.0 } else { 16.0 } * d;
                    let rh = row_height - if slot == 0 { 60.0 } else { 68.0 } * d;
                    let value_width = self.android(text, value_label, self.sp(16), None).width;
                    let value_x = if orientation.is_empty() {
                        left + (width - value_width) / 2.0
                    } else {
                        left + width - 24.0 * d - value_width
                    };
                    self.paint_text(
                        p,
                        value_label,
                        rect(value_x, y, value_width + 1.0, rh),
                        self.sp(16),
                        self.colors[2],
                        clip,
                    );
                    self.paint_text(
                        p,
                        orientation,
                        rect(left + 24.0 * d, y, width - 48.0 * d - value_width, rh),
                        self.sp(16),
                        self.colors[2],
                        clip,
                    );
                    let label = if orientation.is_empty() {
                        value_label.clone()
                    } else {
                        format!("{orientation}: {value_label}")
                    };
                    y += rh + 4.0 * d;
                    let min: f32 = pref.min.parse().unwrap();
                    let max: f32 = pref.max.parse().unwrap();
                    let current = value
                        .parse::<f32>()
                        .ok()
                        .filter(|v| v.is_finite())
                        .unwrap_or(min)
                        .clamp(min, max);
                    self.add(
                        p,
                        Control {
                            id: 4000 + slot as i32,
                            bounds: rect(left + 24.0 * d, y, width - 48.0 * d, 48.0 * d),
                            label: label.clone(),
                            summary: String::new(),
                            role: "slider",
                            enabled: true,
                            checked: None,
                            selected: false,
                            action: Action::Slider(slot),
                            range: Some([min, max, current]),
                        },
                        clip,
                        text,
                    );
                    y += 48.0 * d;
                }
            }
            Kind::Color => y = self.color_content(p, text, (left, y), width, clip),
            Kind::Time => y = self.time_content(p, text, (left, y), width, clip),
            Kind::Confirm(..) | Kind::Backup(_) => {
                let (message, _, rh) = &rows[0];
                let at = rect(left + 24.0 * d, y, width - 48.0 * d, *rh);
                self.paint_text(p, message, at, self.sp(16), self.colors[2], clip);
                // What a restore replaces is read out, not only drawn.
                if matches!(r.kind, Kind::Backup(_)) {
                    let node = super::cards::plain(3000, at, message.clone(), "text");
                    self.push_control(node, clip);
                }
                y += rh;
            }
            _ => {}
        }
        self.max_scroll = (y + super::shown(m.scroll) - clip.bottom).max(0.0);
        self.stretch.extent = clip.height();
        self.set_scroll(m.scroll);
        self.paint_scrollbar(p, clip);
        for (id, key, action, x, button_y, button_width) in
            self.button_positions([left, bottom, width, button_height], button_widths, footer)
        {
            self.add(
                p,
                Control {
                    id,
                    bounds: rect(x, button_y, button_width, button_height),
                    label: self.label(key),
                    summary: String::new(),
                    role: "button",
                    enabled: true,
                    checked: None,
                    selected: false,
                    action,
                    range: None,
                },
                area,
                text,
            );
        }
    }
}
