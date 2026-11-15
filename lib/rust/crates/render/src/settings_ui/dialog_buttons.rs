//! Jetpref's neutral/spacer/dismiss+confirm FlowRow, using Material TextButton metrics.
use super::{Action, App};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;
impl App {
    pub(super) fn button_metrics(&self, width: f32, text: &mut Text) -> (f32, [f32; 3], f32) {
        let d = self.config[2];
        let widths = ["action__default", "action__cancel", "action__ok"].map(|key| {
            (text
                .android(
                    &self.label(key),
                    self.sp(14),
                    500,
                    [self.sp(20), 0.1 * d * self.config[3]],
                    None,
                )
                .width
                + 24.0 * d)
                .max(58.0 * d)
        });
        let height = (self.sp(20) + 16.0 * d).max(48.0 * d);
        let lines = if widths.iter().sum::<f32>() + 16.0 * d > width - 32.0 * d {
            2.0
        } else {
            1.0
        };
        (height, widths, height * lines + 24.0 * d)
    }
    pub(super) fn button_positions(
        &self,
        area: [f32; 4],
        widths: [f32; 3],
        footer: f32,
    ) -> Vec<(i32, &'static str, Action, f32, f32, f32)> {
        let [left, bottom, width, height] = area;
        let d = self.config[2];
        let y = bottom - 8.0 * d - height;
        vec![
            (
                5,
                "action__default",
                Action::Default,
                left + 16.0 * d,
                bottom - footer + 16.0 * d,
                widths[0],
            ),
            (
                6,
                "action__cancel",
                Action::Cancel,
                left + width - 16.0 * d - widths[2] - 8.0 * d - widths[1],
                y,
                widths[1],
            ),
            (
                7,
                "action__ok",
                Action::Confirm,
                left + width - 16.0 * d - widths[2],
                y,
                widths[2],
            ),
        ]
    }
    pub(super) fn paint_button(
        &self,
        p: &mut Panel,
        label: &str,
        b: Rect,
        text: &mut Text,
        clip: Rect,
    ) {
        let spacing = 0.1 * self.config[2] * self.config[3];
        let layout = text.android(label, self.sp(14), 500, [self.sp(20), spacing], None);
        self.paint_text(
            p,
            label,
            rect(
                b.left + (b.width() - layout.width) / 2.0,
                b.top + (b.height() - layout.height) / 2.0,
                layout.width + 1.0,
                layout.height,
            ),
            self.sp(14),
            self.colors[3],
            clip,
        );
        let style = &mut p.items.last_mut().unwrap().text;
        style.weight = 500;
        style.letter_spacing = spacing;
    }
}
