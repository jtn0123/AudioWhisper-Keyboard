use super::{
    Action, App, Modal,
    catalogue::{KEYBOARD, Kind},
};
use crate::{panel_scroll::Fling, touch::Motion};
impl App {
    pub fn activate(&mut self, id: i32) -> bool {
        let Some(c) = self
            .controls
            .iter()
            .find(|c| c.id == id && c.enabled)
            .cloned()
        else {
            return false;
        };
        self.focus = id;
        self.fling = None;
        match c.action {
            Action::None => return false,
            Action::Back => self.back(),
            Action::Route(route) => self.navigate(&route),
            Action::Picker => self.effects.push(vec!["picker".into()]),
            Action::Editor => {
                self.editor.focused = true;
                self.effects.push(vec!["editor".into()]);
            }
            Action::Switch(i) => {
                let key = KEYBOARD[i].switch;
                self.write(key, (!self.yes(key)).to_string());
            }
            Action::ModalSwitch => {
                if let Some(m) = &mut self.modal {
                    m.enabled = !m.enabled;
                }
            }
            Action::Row(i) => {
                let r = KEYBOARD[i];
                match r.kind {
                    Kind::Toggle => self.write(r.key, (!self.yes(r.key)).to_string()),
                    Kind::Choice(_) | Kind::Slider(_, _) => {
                        if self.editor.focused {
                            self.editor.focused = false;
                            self.editor.composing = None;
                            self.effects.push(vec!["hideEditor".into()]);
                        }
                        let mut values = vec![self.value(r.key)];
                        if let Kind::Slider(second, _) = r.kind
                            && !second.is_empty()
                        {
                            values.push(self.value(second));
                        }
                        self.modal = Some(Modal {
                            row: i,
                            selected: self.value(r.key),
                            values,
                            scroll: 0.0,
                            enabled: r.switch.is_empty() || self.yes(r.switch),
                        });
                        self.focus = -1;
                    }
                    _ => {}
                }
            }
            Action::Choose(choice) => {
                if let Some(m) = &mut self.modal {
                    m.selected = choice;
                }
            }
            Action::Cancel => {
                self.modal = None;
                self.focus = -1;
            }
            Action::Confirm => {
                if let Some(m) = self.modal.take() {
                    let r = KEYBOARD[m.row];
                    match r.kind {
                        Kind::Choice(_) => {
                            self.write(r.key, m.selected);
                            if !r.switch.is_empty() {
                                self.write(r.switch, m.enabled.to_string());
                            }
                        }
                        Kind::Slider(second, _) => {
                            self.write(r.key, m.values[0].clone());
                            if !second.is_empty() {
                                self.write(second, m.values[1].clone());
                            }
                        }
                        _ => {}
                    }
                }
                self.focus = -1;
            }
            Action::Default => {
                if let Some(m) = self.modal.take() {
                    let r = KEYBOARD[m.row];
                    self.reset(r.key);
                    if !r.switch.is_empty() {
                        self.reset(r.switch);
                    }
                    if let Kind::Slider(second, _) = r.kind
                        && !second.is_empty()
                    {
                        self.reset(second);
                    }
                }
                self.focus = -1;
            }
            Action::Slider(_) => {}
        }
        true
    }
    pub fn set_slider(&mut self, slot: usize, value: f32) -> bool {
        let Some(m) = &mut self.modal else {
            return false;
        };
        let r = KEYBOARD[m.row];
        let Kind::Slider(second, _) = r.kind else {
            return false;
        };
        let key = if slot == 0 { r.key } else { second };
        let Ok(p) = settings::catalogue::get(&settings::text(key)) else {
            return false;
        };
        let (Ok(min), Ok(max), Ok(step)) = (
            p.min.parse::<f32>(),
            p.max.parse::<f32>(),
            p.step.parse::<f32>(),
        ) else {
            return false;
        };
        if !value.is_finite() || slot >= m.values.len() {
            return false;
        }
        let v = (min + ((value.clamp(min, max) - min) / step).round() * step).clamp(min, max);
        m.values[slot] = if p.kind == "f" {
            format!("{v:.1}")
        } else {
            (v as i32).to_string()
        };
        true
    }
    pub fn adjust(&mut self, id: i32, direction: f32) -> bool {
        let Some(c) = self.controls.iter().find(|c| c.id == id).cloned() else {
            return false;
        };
        let Action::Slider(slot) = c.action else {
            return false;
        };
        let Some(range) = c.range else {
            return false;
        };
        let m = self.modal.as_ref().unwrap();
        let r = KEYBOARD[m.row];
        let Kind::Slider(second, _) = r.kind else {
            return false;
        };
        let p = settings::catalogue::get(&settings::text(if slot == 0 { r.key } else { second }))
            .unwrap();
        self.set_slider(
            slot,
            range[2] + direction * p.step.parse::<f32>().unwrap_or(1.0),
        )
    }
    pub fn focus_next(&mut self, reverse: bool) {
        let mut eligible = vec![];
        if let Some(m) = &self.modal {
            if !KEYBOARD[m.row].switch.is_empty() {
                eligible.push(8);
            }
            match KEYBOARD[m.row].kind {
                Kind::Choice(_) => {
                    eligible.extend((0..Self::choices(m.row).len()).map(|i| 3000 + i as i32))
                }
                Kind::Slider(_, _) => eligible.extend((0..m.values.len()).map(|i| 4000 + i as i32)),
                _ => {}
            }
            eligible.extend([5, 6, 7]);
        } else {
            if self.screen_index() == 1 {
                eligible.push(1);
                for (i, row) in KEYBOARD.iter().enumerate() {
                    if self.visible(i) && self.enabled(i) && !matches!(row.kind, Kind::Heading) {
                        eligible.push(1000 + i as i32);
                        if !row.switch.is_empty() {
                            eligible.push(2000 + i as i32);
                        }
                    }
                }
            } else {
                if !self.ime_enabled || !self.ime_selected {
                    eligible.push(40);
                }
                eligible.extend(100..112);
            }
            eligible.extend([2, 3]);
        }
        if eligible.is_empty() {
            return;
        }
        let next = match eligible.iter().position(|id| *id == self.focus) {
            Some(at) if reverse => (at + eligible.len() - 1) % eligible.len(),
            Some(at) => (at + 1) % eligible.len(),
            None if reverse => eligible.len() - 1,
            None => 0,
        };
        self.focus = eligible[next];
        self.reveal_focus = true;
    }
    pub fn motion(&mut self, e: &Motion) {
        if !e.valid() || e.action == 3 || e.ids.len() != 1 {
            self.down = None;
            self.dragging = false;
            self.fling = None;
            return;
        }
        let count = e.ids.len();
        for sample in 0..e.times.len() - 1 {
            let at = sample * count * 2;
            let (x, y, t) = (e.coords[at], e.coords[at + 1], e.times[sample + 1]);
            if e.action == 0 {
                self.fling = None;
                self.dragging = false;
                self.velocity = 0.0;
                let id = self
                    .controls
                    .iter()
                    .rev()
                    .find(|c| c.enabled && c.bounds.contains(x, y))
                    .map_or(-1, |c| c.id);
                self.down = Some((id, x, y, t));
                self.last = (y, t);
            } else if e.action == 2 {
                let Some((id, _, down_y, _)) = self.down else {
                    continue;
                };
                if let Some(c) = self.controls.iter().find(|c| c.id == id).cloned()
                    && let Action::Slider(slot) = c.action
                {
                    if let Some([min, max, _]) = c.range {
                        self.set_slider(
                            slot,
                            min + (max - min) * (x - c.bounds.left) / c.bounds.width(),
                        );
                    }
                } else if (y - down_y).abs() > 8.0 * self.config[2] || self.dragging {
                    self.dragging = true;
                    let dt = (t - self.last.1) as f32 / 1_000_000_000.0;
                    if dt > 0.0 {
                        self.velocity = ((self.last.0 - y) / dt)
                            .clamp(-12000.0 * self.config[2], 12000.0 * self.config[2]);
                    }
                    self.set_scroll(self.scroll() + self.last.0 - y);
                }
                self.last = (y, t);
            } else if e.action == 1 {
                if let Some((id, _, _, _)) = self.down.take() {
                    if self.dragging {
                        self.fling = Fling::new(t, self.scroll(), self.config[2], self.velocity);
                    } else if let Some(c) = self
                        .controls
                        .iter()
                        .find(|c| c.id == id && c.bounds.contains(x, y))
                        .cloned()
                    {
                        if let Action::Slider(slot) = c.action {
                            if let Some([min, max, _]) = c.range {
                                self.set_slider(
                                    slot,
                                    min + (max - min) * (x - c.bounds.left) / c.bounds.width(),
                                );
                            }
                        } else {
                            if c.action == Action::Editor {
                                self.place_editor_caret(x, y);
                            }
                            self.activate(id);
                        }
                    }
                }
                self.dragging = false;
            }
        }
    }
}
