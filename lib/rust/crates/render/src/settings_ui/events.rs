use super::{
    Action, App, Modal,
    catalogue::{Do, Kind},
};
use crate::touch::Motion;
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
        self.end_scroll_cycle();
        match c.action {
            Action::None => return false,
            Action::Back => self.back(),
            Action::Route(route) => self.navigate(&route),
            Action::Picker => self.effects.push(vec!["picker".into()]),
            Action::Editor | Action::Field(_) => {
                self.own_field(match c.action {
                    Action::Field(i) => Some(i),
                    _ => None,
                });
                self.editor.focused = true;
                self.effects.push(vec!["editor".into()]);
            }
            Action::Switch(i) => {
                let key = self.rows()[i].switch;
                self.write(key, (!self.yes(key)).to_string());
            }
            Action::ModalSwitch => {
                if let Some(m) = &mut self.modal {
                    m.enabled = !m.enabled;
                }
            }
            Action::Row(i) => {
                let r = self.rows()[i];
                match r.kind {
                    Kind::Toggle => self.write(r.key, (!self.yes(r.key)).to_string()),
                    Kind::Effect(effect) => self.effect(effect),
                    Kind::Card(card) => {
                        let url = self.label(card.url);
                        self.effects.push(vec!["url".into(), url]);
                    }
                    Kind::Spell => self
                        .effects
                        .push(vec!["intent".into(), "spellCheckerSettings".into()]),
                    Kind::Choice(_) | Kind::Slider(_) | Kind::Confirm(..) => {
                        if self.editor.focused {
                            self.editor.focused = false;
                            self.editor.composing = None;
                            self.effects.push(vec!["hideEditor".into()]);
                        }
                        let mut values = vec![];
                        if let Kind::Slider(s) = r.kind {
                            values.push(self.value(r.key));
                            if !s.second.is_empty() {
                                values.push(self.value(s.second));
                            }
                        }
                        self.scrollbar_start[1] = -1;
                        self.scrollbar_last[1] = -1;
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
                    let r = self.rows()[m.row];
                    match r.kind {
                        Kind::Choice(_) => {
                            self.write(r.key, m.selected);
                            if !r.switch.is_empty() {
                                self.write(r.switch, m.enabled.to_string());
                            }
                        }
                        Kind::Slider(s) => {
                            self.write(r.key, m.values[0].clone());
                            if !s.second.is_empty() {
                                self.write(s.second, m.values[1].clone());
                            }
                        }
                        Kind::Confirm(ask) => {
                            let mut plan: Vec<String> =
                                ask.effect.iter().map(|s| (*s).to_owned()).collect();
                            if !ask.toast.is_empty() {
                                plan.push(self.label(ask.toast));
                            }
                            self.effects.push(plan);
                        }
                        _ => {}
                    }
                }
                self.focus = -1;
            }
            Action::Default => {
                if let Some(m) = self.modal.take() {
                    let r = self.rows()[m.row];
                    self.reset(r.key);
                    if !r.switch.is_empty() {
                        self.reset(r.switch);
                    }
                    if let Kind::Slider(s) = r.kind
                        && !s.second.is_empty()
                    {
                        self.reset(s.second);
                    }
                }
                self.focus = -1;
            }
            Action::Slider(_) | Action::SelectionHandle(_) => {}
        }
        true
    }
    /// A row's platform effect, as the plan Kotlin executes.
    fn effect(&mut self, effect: Do) {
        let plan = match effect {
            Do::Url(key) => vec![
                "url".into(),
                self.label(key)
                    .replace("{version}", self.fact("version_name")),
            ],
            // Kotlin fills `{error_message}` from the exception a failed copy throws.
            Do::Copy => vec![
                "copy".into(),
                self.version(),
                self.label("about__version_copied__title"),
                self.label("about__version_copied__error"),
            ],
            Do::Intent(name) => vec!["intent".into(), name.into()],
        };
        self.effects.push(plan);
    }
    pub fn set_slider(&mut self, slot: usize, value: f32) -> bool {
        let rows = self.rows();
        let Some(m) = &mut self.modal else {
            return false;
        };
        let r = rows[m.row];
        let Kind::Slider(s) = r.kind else {
            return false;
        };
        let key = if slot == 0 { r.key } else { s.second };
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
        let r = self.rows()[m.row];
        let Kind::Slider(s) = r.kind else {
            return false;
        };
        let p = settings::catalogue::get(&settings::text(if slot == 0 { r.key } else { s.second }))
            .unwrap();
        let changed = self.set_slider(
            slot,
            range[2] + direction * p.step.parse::<f32>().unwrap_or(1.0),
        );
        // M3's slider ends a key press with onValueChangeFinished.
        self.slider_finished(slot);
        changed
    }
    /// `onValueChangeFinished`: JetPref previews the selected value, here as a vibrate plan
    /// with the other haptic setting's stored value.
    pub fn slider_finished(&mut self, slot: usize) {
        let Some(m) = &self.modal else {
            return;
        };
        let r = self.rows()[m.row];
        let Kind::Slider(s) = r.kind else {
            return;
        };
        let Some(value) = m.values.get(slot).filter(|_| s.preview).cloned() else {
            return;
        };
        let [duration, strength] = [
            "input_feedback__haptic_vibration_duration",
            "input_feedback__haptic_vibration_strength",
        ]
        .map(|key| {
            if key == r.key {
                value.clone()
            } else {
                self.value(key)
            }
        });
        self.effects
            .push(vec!["vibrate".into(), duration, strength]);
    }
    pub fn focus_next(&mut self, reverse: bool) {
        let mut eligible = vec![];
        if let Some(m) = &self.modal {
            let r = self.rows()[m.row];
            if !r.switch.is_empty() {
                eligible.push(8);
            }
            match r.kind {
                Kind::Choice(_) => {
                    eligible.extend((0..self.choices(m.row).len()).map(|i| 3000 + i as i32))
                }
                Kind::Slider(_) => eligible.extend((0..m.values.len()).map(|i| 4000 + i as i32)),
                _ => {}
            }
            if !matches!(r.kind, Kind::Confirm(..)) {
                eligible.push(5);
            }
            eligible.extend([6, 7]);
        } else {
            if !self.is_home() {
                eligible.push(1);
                for (i, row) in self.rows().iter().enumerate() {
                    if !self.visible(i) || !self.enabled(i) {
                        continue;
                    }
                    match row.kind {
                        Kind::Heading
                        | Kind::Fixed
                        | Kind::Info
                        | Kind::Header
                        | Kind::Status(_)
                        | Kind::Note => {}
                        // The card's text is not focusable; its hyperlink is.
                        Kind::Card(_) => eligible.push(6000 + i as i32),
                        _ => eligible.push(1000 + i as i32),
                    }
                    if !row.switch.is_empty() {
                        eligible.push(2000 + i as i32);
                    }
                }
            } else {
                if !self.ime_enabled || !self.ime_selected {
                    eligible.push(40);
                }
                eligible.extend(100..112);
            }
            if self.screen().is_none_or(|s| s.preview) {
                eligible.extend([2, 3]);
            }
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
    /// A move that scrolls, handled per event like Compose's DragGestureNode: the touch slop is
    /// judged at the event's final position, crossing it scrolls by the over-slop only, and the
    /// velocity tracker gets the down, then every sample from the crossing event on, except
    /// that `awaitDragOrUp` passes over an event whose pointer has not moved: it neither scrolls
    /// nor reaches the tracker, so holding still before lifting stops the fling. Returns false
    /// for drags that belong to a slider or a selection handle.
    fn scroll_drag(&mut self, e: &Motion) -> bool {
        let Some((id, _, down_y, down_t)) = self.down else {
            return true;
        };
        if id == 20
            || id == 21
            || self
                .controls
                .iter()
                .any(|c| c.id == id && matches!(c.action, Action::Slider(_)))
        {
            return false;
        }
        let samples: Vec<(i64, f32)> = (0..e.times.len() - 1)
            .map(|s| (e.times[s + 1].div_euclid(1_000_000), e.coords[s * 2 + 1]))
            .collect();
        let (x, y) = (e.coords[e.coords.len() - 2], e.coords[e.coords.len() - 1]);
        if self.dragging && (x, y) == self.last {
            return true;
        }
        let mut delta = y - self.last.1;
        if !self.dragging {
            // ViewConfiguration's default 8 dp touch slop, as getDimensionPixelSize rounds it.
            let slop = (8.0 * self.config[2] + 0.5).floor();
            let total = y - down_y;
            if total == 0.0 || total.abs() < slop {
                self.last = (x, y);
                return true;
            }
            self.dragging = true;
            self.velocity.down(down_t.div_euclid(1_000_000), down_y);
            delta = total - total.signum() * slop;
        }
        self.velocity.change(&samples);
        self.drag_by(-delta);
        self.last = (x, y);
        true
    }
    pub fn motion(&mut self, e: &Motion) {
        if !e.valid() || e.action == 3 || e.ids.len() != 1 {
            self.down = None;
            self.dragging = false;
            self.fling = None;
            self.end_scroll_cycle();
            return;
        }
        if e.action == 2 && self.scroll_drag(e) {
            return;
        }
        let count = e.ids.len();
        for sample in 0..e.times.len() - 1 {
            let at = sample * count * 2;
            let (x, y, t) = (e.coords[at], e.coords[at + 1], e.times[sample + 1]);
            if e.action == 0 {
                self.fling = None;
                self.dragging = false;
                let id = self
                    .controls
                    .iter()
                    .rev()
                    .find(|c| c.enabled && c.bounds.contains(x, y))
                    .map_or(-1, |c| c.id);
                self.handle_drag_offset = self
                    .controls
                    .iter()
                    .find(|c| c.id == id)
                    .and_then(|c| {
                        if let Action::SelectionHandle(anchor) = c.action {
                            self.handle_caret_x(anchor)
                        } else {
                            None
                        }
                    })
                    .map_or(0.0, |caret| caret - x);
                if id == 20 || id == 21 {
                    self.handle_drag_other = if id == 20 {
                        self.editor.focus
                    } else {
                        self.editor.anchor
                    };
                }
                self.down = Some((id, x, y, t));
                self.last = (x, y);
            } else if e.action == 2 {
                let Some((id, ..)) = self.down else {
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
                } else if id == 20 || id == 21 {
                    // The handles disappear at a collapsed range. Keep the original
                    // grabbed endpoint/opposite anchor so the same drag can cross it.
                    self.drag_editor_handle(x + self.handle_drag_offset, y);
                }
                self.last = (x, y);
            } else if e.action == 1 {
                let slider = self.down.and_then(|(id, ..)| {
                    self.controls
                        .iter()
                        .find(|c| c.id == id)
                        .and_then(|c| match c.action {
                            Action::Slider(slot) => Some(slot),
                            _ => None,
                        })
                });
                if let Some((id, _, _, _)) = self.down.take() {
                    if self.dragging {
                        self.velocity.up(t.div_euclid(1_000_000));
                        // ViewConfiguration's default 8000 dp/s, as getDimensionPixelSize rounds it.
                        let maximum = (8000.0 * self.config[2] + 0.5).floor();
                        let velocity = -self.velocity.velocity(maximum);
                        self.release_drag(t, velocity);
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
                            let field = matches!(c.action, Action::Editor | Action::Field(_));
                            self.activate(id);
                            if field {
                                self.place_editor_caret(id, x, y);
                            }
                        }
                    }
                }
                if let Some(slot) = slider {
                    self.slider_finished(slot);
                }
                self.dragging = false;
            }
        }
    }
}
