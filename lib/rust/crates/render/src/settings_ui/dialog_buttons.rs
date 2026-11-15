//! Jetpref's neutral/spacer/dismiss+confirm FlowRow, using Material TextButton metrics.
//!
//! Compose measures in whole pixels: each dp padding and minimum is `roundToPx`'d on its own,
//! the label is `ceil`'d text width wide, and centring offsets round. A 12 dp padding is
//! 32 px at 2.625x, not 31.5.
use super::{Action, App};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;
impl App {
    /// The neutral, dismiss and confirm labels: ListPreference and DialogSliderPreference's
    /// default/cancel/ok, or the emoji history confirmation's no/yes with no neutral button.
    pub(super) fn button_keys(&self) -> [&'static str; 3] {
        let confirm = self.modal.as_ref().is_some_and(|m| {
            matches!(self.rows()[m.row].kind, super::catalogue::Kind::Confirm(..))
        });
        if confirm {
            ["", "action__no", "action__yes"]
        } else {
            ["action__default", "action__cancel", "action__ok"]
        }
    }
    pub(super) fn button_metrics(&self, width: f32, text: &mut Text) -> (f32, [f32; 3], f32) {
        let d = self.config[2];
        let widths = self.button_keys().map(|key| {
            if key.is_empty() {
                return 0.0;
            }
            let label = self.button_label(text, &self.label(key));
            (label.width.ceil() + 2.0 * (12.0 * d).round()).max((58.0 * d).round())
        });
        let height = self.button_box(self.line_height(self.sp(14)).ceil())[0];
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
        let px = |dp: f32| (dp * self.config[2]).round();
        let y = bottom - px(8.0) - height;
        let [neutral, dismiss, confirm] = self.button_keys();
        let mut buttons = vec![
            (
                5,
                neutral,
                Action::Default,
                left + px(16.0),
                bottom - footer + px(16.0),
                widths[0],
            ),
            (
                6,
                dismiss,
                Action::Cancel,
                left + width - px(16.0) - widths[2] - px(8.0) - widths[1],
                y,
                widths[1],
            ),
            (
                7,
                confirm,
                Action::Confirm,
                left + width - px(16.0) - widths[2],
                y,
                widths[2],
            ),
        ];
        buttons.retain(|b| !b.1.is_empty());
        buttons
    }
    /// labelLarge: 14/20 sp, medium weight, 0.1 sp tracking.
    fn button_label(
        &self,
        text: &mut Text,
        label: &str,
    ) -> std::sync::Arc<crate::android_text::Layout> {
        let spacing = 0.1 * self.config[2] * self.config[3];
        text.android(
            label,
            self.sp(14),
            500,
            [self.line_height(self.sp(14)), spacing],
            None,
        )
    }
    /// The 48 dp touch box around a 40 dp TextButton (8 dp vertical padding) whose label is
    /// `label_height` tall: the box height and the label's offset from the box top.
    fn button_box(&self, label_height: f32) -> [f32; 2] {
        let d = self.config[2];
        let padding = (8.0 * d).round();
        let button = (40.0 * d).round().max(label_height + 2.0 * padding);
        let touch = (48.0 * d).round().max(button);
        let inner = button - 2.0 * padding;
        [
            touch,
            ((touch - button) / 2.0).round() + padding + ((inner - label_height) / 2.0).round(),
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
        let layout = self.button_label(text, label);
        let width = layout.width.ceil();
        let padding = (12.0 * self.config[2]).round();
        let [_, top] = self.button_box(layout.height);
        self.paint_text(
            p,
            label,
            rect(
                b.left + padding + ((b.width() - 2.0 * padding - width) / 2.0).round(),
                b.top + top,
                width + 1.0,
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
