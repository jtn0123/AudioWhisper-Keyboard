//! Measure the same rows used for painting before choosing a bounded dialog size.
use super::{App, catalogue::Kind};
use crate::text::Text;
impl App {
    /// `dp` in whole pixels, as Compose's `roundToPx` gives every dp padding and size.
    pub(super) fn px(&self, dp: f32) -> f32 {
        (dp * self.config[2]).round()
    }
    /// A choice row's text insets inside its 8 dp padding: the 52 dp radio and the 16 dp end,
    /// each rounded to whole pixels as Compose's `roundToPx` does (535 px of text, not 535.5,
    /// in the 288 dp dialog at 2.625).
    pub(super) fn radio_insets(&self) -> [f32; 2] {
        [self.px(52.0), self.px(16.0)]
    }
    pub(super) fn dialog_rows(&self, width: f32, text: &mut Text) -> Vec<(String, String, f32)> {
        let m = self.modal.as_ref().unwrap();
        let r = self.rows()[m.row];
        let [_, _, d, _] = self.config;
        match r.kind {
            Kind::Choice(_) => {
                let [start, end] = self.radio_insets();
                let tw = width - 2.0 * self.px(8.0) - start - end;
                self.choices(m.row)
                    .iter()
                    .map(|choice| {
                        let label = self.choice_label(m.row, choice);
                        let summary = self.choice_description(m.row, choice, m.selected == *choice);
                        let rh = (self.measure(text, &label, self.sp(16), tw)
                            + self.measure(text, &summary, self.sp(14), tw)
                            + 16.0 * d)
                            .max(40.0 * d);
                        (label, summary, rh)
                    })
                    .collect()
            }
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
            Kind::Confirm(ask) => {
                let mut message = self.label(ask.message);
                if !ask.name.is_empty() {
                    message = message.replace("{name}", &self.label(ask.name));
                }
                let rh = self.measure(text, &message, self.sp(16), width - 48.0 * d);
                vec![(message, String::new(), rh)]
            }
            Kind::Color => vec![(String::new(), String::new(), self.color_height(width, text))],
            _ => Vec::new(),
        }
    }
}
