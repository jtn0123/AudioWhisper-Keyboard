use super::layout::height;
use super::{
    Action, App, Control,
    catalogue::{KEYBOARD, Kind},
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
impl App {
    pub(super) fn dialog(&mut self, p: &mut Panel, text: &mut Text) {
        let m = self.modal.as_ref().unwrap().clone();
        let r = KEYBOARD[m.row];
        let [w, h, d, f] = self.config;
        let width = (w - 48.0 * d).min(560.0 * d);
        let left = (w - width) / 2.0;
        let padding = if h < 600.0 * d { 8.0 * d } else { 32.0 * d };
        let top = padding;
        let bottom = h - padding;
        let button_height = (48.0 * f).max(48.0) * d;
        let footer = button_height + 24.0 * d;
        let area = rect(left, top, width, bottom - top);
        self.paint_box(p, rect(0.0, 0.0, w, h), 0x60000000, 0.0);
        self.paint_box(p, area, self.colors[0], 28.0 * d);
        self.controls.clear();
        let title = self.label(r.label);
        let title_width = width
            - if r.switch.is_empty() {
                48.0 * d
            } else {
                112.0 * d
            };
        let th = height(text, &title, 24.0 * d * f, title_width) + 32.0 * d;
        self.paint_text(
            p,
            &title,
            rect(left + 24.0 * d, top + 24.0 * d, title_width, th),
            24.0 * d * f,
            self.colors[1],
            area,
        );
        let clip = rect(
            left,
            top + th + 16.0 * d,
            width,
            (bottom - top - th - footer - 16.0 * d).max(1.0),
        );
        let mut y = clip.top - m.scroll;
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
                    bounds: rect(
                        left + width - 72.0 * d,
                        top + 16.0 * d,
                        64.0 * d,
                        (th - 16.0 * d).max(48.0 * d),
                    ),
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
                for (j, choice) in Self::choices(m.row).iter().enumerate() {
                    let label = self.choice_label(m.row, choice);
                    let summary = if m.selected == *choice {
                        if let Kind::Choice(prefix) = r.kind {
                            self.strings
                                .get(&format!(
                                    "enum__{prefix}__{}__description",
                                    choice.to_ascii_lowercase()
                                ))
                                .cloned()
                                .unwrap_or_default()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    let rh = (height(text, &label, 16.0 * d * f, width - 80.0 * d)
                        + height(text, &summary, 14.0 * d * f, width - 80.0 * d)
                        + 24.0 * d)
                        .max(56.0 * d);
                    self.add(
                        p,
                        Control {
                            id: 3000 + j as i32,
                            bounds: rect(left + 8.0 * d, y, width - 16.0 * d, rh),
                            label,
                            summary,
                            role: "radio",
                            enabled: m.enabled,
                            checked: None,
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
            Kind::Slider(second, unit) => {
                for (slot, value) in m.values.iter().enumerate() {
                    let key = if slot == 0 { r.key } else { second };
                    let pref = settings::catalogue::get(&settings::text(key)).unwrap();
                    let orientation = if r.key.contains("spacing") {
                        if slot == 0 {
                            "screen_orientation__vertical"
                        } else {
                            "screen_orientation__horizontal"
                        }
                    } else if slot == 0 {
                        "screen_orientation__portrait"
                    } else {
                        "screen_orientation__landscape"
                    };
                    let label = if m.values.len() > 1 {
                        self.label(orientation) + ": "
                    } else {
                        String::new()
                    } + &self.label(unit).replace("{v}", value);
                    let rh = height(text, &label, 16.0 * d * f, width - 48.0 * d) + 16.0 * d;
                    self.paint_text(
                        p,
                        &label,
                        rect(left + 24.0 * d, y, width - 48.0 * d, rh),
                        16.0 * d * f,
                        self.colors[1],
                        clip,
                    );
                    y += rh;
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
                            bounds: rect(left + 32.0 * d, y, width - 64.0 * d, 56.0 * d),
                            label,
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
                    y += 72.0 * d;
                }
            }
            _ => {}
        }
        self.max_scroll = (y + m.scroll - clip.bottom).max(0.0);
        self.set_scroll(m.scroll);
        for (id, key, action, x) in [
            (5, "action__default", Action::Default, left + 8.0 * d),
            (6, "action__cancel", Action::Cancel, left + width / 3.0),
            (7, "action__ok", Action::Confirm, left + width * 2.0 / 3.0),
        ] {
            self.add(
                p,
                Control {
                    id,
                    bounds: rect(
                        x,
                        bottom - button_height - 8.0 * d,
                        width / 3.0 - 8.0 * d,
                        button_height,
                    ),
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
