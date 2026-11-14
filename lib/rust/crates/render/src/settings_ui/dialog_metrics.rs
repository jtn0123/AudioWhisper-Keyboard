//! Measure the same rows used for painting before choosing a bounded dialog size.
use super::{
    App,
    catalogue::{KEYBOARD, Kind},
    layout::height,
};
use crate::text::Text;
impl App {
    pub(super) fn dialog_rows(&self, width: f32, text: &mut Text) -> Vec<(String, String, f32)> {
        let m = self.modal.as_ref().unwrap();
        let r = KEYBOARD[m.row];
        let [_, _, d, f] = self.config;
        match r.kind {
            Kind::Choice(prefix) => Self::choices(m.row)
                .iter()
                .map(|choice| {
                    let label = self.choice_label(m.row, choice);
                    let summary = if m.selected == *choice {
                        self.strings
                            .get(&format!(
                                "enum__{prefix}__{}__description",
                                choice.to_ascii_lowercase()
                            ))
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    let rh = (height(text, &label, 16.0 * d * f, width - 80.0 * d)
                        + height(text, &summary, 14.0 * d * f, width - 80.0 * d)
                        + 24.0 * d)
                        .max(56.0 * d);
                    (label, summary, rh)
                })
                .collect(),
            Kind::Slider(_, unit) => m
                .values
                .iter()
                .enumerate()
                .map(|(slot, value)| {
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
                    let rh = height(text, &label, 16.0 * d * f, width - 48.0 * d) + 88.0 * d;
                    (label, String::new(), rh)
                })
                .collect(),
            _ => Vec::new(),
        }
    }
}
