//! A slider dialog's values: a drag or a key step snapped to the preference's range and step,
//! and JetPref's preview of the value chosen.
use super::{Action, App, catalogue::Kind};
impl App {
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
}
