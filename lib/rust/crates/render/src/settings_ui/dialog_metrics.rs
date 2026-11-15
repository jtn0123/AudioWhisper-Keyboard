//! Measure the same rows used for painting before choosing a bounded dialog size.
use super::{App, catalogue::Kind};
use crate::text::Text;
impl App {
    pub(super) fn dialog_rows(&self, width: f32, text: &mut Text) -> Vec<(String, String, f32)> {
        let m = self.modal.as_ref().unwrap();
        let r = self.rows()[m.row];
        let [_, _, d, _] = self.config;
        match r.kind {
            Kind::Choice(_) => self
                .choices(m.row)
                .iter()
                .map(|choice| {
                    let label = self.choice_label(m.row, choice);
                    let summary = self.choice_description(m.row, choice, m.selected == *choice);
                    let rh = (self.measure(text, &label, self.sp(16), width - 84.0 * d)
                        + self.measure(text, &summary, self.sp(14), width - 84.0 * d)
                        + 16.0 * d)
                        .max(40.0 * d);
                    (label, summary, rh)
                })
                .collect(),
            Kind::Slider(s) => m
                .values
                .iter()
                .enumerate()
                .map(|(slot, value)| {
                    let label = if m.values.len() > 1 {
                        self.label(s.labels[slot])
                    } else {
                        String::new()
                    };
                    let value = self.value_label(&s.unit, value);
                    let rh = self
                        .measure(text, &label, self.sp(16), width - 48.0 * d)
                        .max(self.measure(text, &value, self.sp(16), width - 48.0 * d))
                        + if slot == 0 { 60.0 } else { 68.0 } * d;
                    (label, value, rh)
                })
                .collect(),
            // JetPrefAlertDialog's content: bodyLarge text in 24 dp horizontal padding.
            Kind::Confirm(_, name) => {
                let message = self
                    .label("action__reset_confirm_message")
                    .replace("{name}", &self.label(name));
                let rh = self.measure(text, &message, self.sp(16), width - 48.0 * d);
                vec![(message, String::new(), rh)]
            }
            _ => Vec::new(),
        }
    }
}
