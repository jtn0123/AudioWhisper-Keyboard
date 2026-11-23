//! The Material 3 text styles `m3VariantTextStyles` gives Traditional rows, and one-line text
//! the way BasicText (`maxLines = 1`, Ellipsis) shows it.
use super::super::App;
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// Size in sp, line height in em of the size (0: none), tracking in sp, weight, monospace.
#[derive(Clone, Copy, Debug)]
pub(super) struct Style {
    pub size: usize,
    pub line_em: f32,
    pub line_sp: f32,
    pub tracking: f32,
    pub weight: i32,
    pub mono: bool,
}
/// titleSmall (0.1 sp tracking) at 14 sp, Medium, 1.2 em lines.
pub(super) const NAME: Style = Style {
    size: 14,
    line_em: 1.2,
    line_sp: 0.0,
    tracking: 0.1,
    weight: 500,
    mono: false,
};
/// bodySmall (0.4 sp tracking) at 11 sp, 1.25 em lines.
pub(super) const AUTHOR: Style = Style {
    size: 11,
    line_em: 1.25,
    line_sp: 0.0,
    tracking: 0.4,
    weight: 400,
    mono: false,
};
/// labelSmall (16 sp lines, 0.5 sp tracking, Medium) at 11 sp, in the monospace family.
pub(super) const VERSION: Style = Style {
    size: 11,
    line_em: 0.0,
    line_sp: 16.0,
    tracking: 0.5,
    weight: 500,
    mono: true,
};
/// labelSmall at 11 sp, Medium: the license badge.
pub(super) const BADGE: Style = Style {
    mono: false,
    ..VERSION
};
/// The app's bodyLarge: 16 sp, font-metric lines, no tracking (the dialog's text).
pub(super) const BODY: Style = Style {
    size: 16,
    line_em: 0.0,
    line_sp: 0.0,
    tracking: 0.0,
    weight: 400,
    mono: false,
};
/// One line of text: its width and height in pixels.
pub(super) type Extent = [f32; 2];
impl App {
    /// Line height and letter spacing in pixels; an sp line height keeps its ratio to the
    /// nonlinearly scaled size, as `resolveLineHeightInPx` does.
    pub(super) fn style_metrics(&self, s: Style) -> [f32; 2] {
        let size = self.sp(s.size);
        let line = if s.line_em > 0.0 {
            size * s.line_em
        } else if s.line_sp > 0.0 {
            size * s.line_sp / s.size as f32
        } else {
            0.0
        };
        [line, s.tracking * self.config[2] * self.config[3]]
    }
    /// Unwrapped text's width and height, `ceil`'d as Compose sizes a Text.
    pub(super) fn extent(&self, text: &mut Text, label: &str, s: Style) -> Extent {
        let metrics = self.style_metrics(s);
        if s.mono {
            let mut layout =
                text.layout_typography(label, self.sp(s.size), true, s.weight, false, metrics);
            layout.break_all_lines(None);
            return [layout.width().ceil(), layout.height().ceil()];
        }
        let layout = text.android(label, self.sp(s.size), s.weight, metrics, None);
        [layout.width.ceil(), layout.height.ceil()]
    }
    /// StaticLayout's one-line END ellipsis: whole clusters while they and "…" fit `width`.
    pub(super) fn one_line(&self, text: &mut Text, label: &str, s: Style, width: f32) -> String {
        let (size, metrics) = (self.sp(s.size), self.style_metrics(s));
        let fits = |text: &mut Text, candidate: &str| {
            text.android(candidate, size, s.weight, metrics, Some(width.max(1.0)))
                .lines
                .len()
                <= 1
        };
        if label.is_empty() || fits(text, label) {
            return label.into();
        }
        let spacing = metrics[1] * crate::android_text::layout_size(size) / size;
        let ellipsis = text.android("…", size, s.weight, metrics, None).width + spacing;
        let whole = text.android(label, size, s.weight, metrics, None);
        let cut = whole.lines[0]
            .carets
            .windows(2)
            .find(|pair| pair[1].1 + spacing / 2.0 > width - ellipsis)
            .map_or(label.len(), |pair| pair[0].0);
        let mut fitted = label[..cut].to_owned();
        loop {
            let candidate = format!("{fitted}…");
            if fits(text, &candidate) || fitted.is_empty() {
                return candidate;
            }
            fitted.pop();
        }
    }
    /// Paints `label` in style `s` with its top-left at `at`, clipped to `clip`.
    pub(super) fn paint_style(
        &self,
        p: &mut Panel,
        label: &str,
        at: (f32, f32, f32, f32),
        s: Style,
        color: u32,
        clip: Rect,
    ) {
        let (x, y, w, h) = at;
        self.paint_text(p, label, rect(x, y, w, h), self.sp(s.size), color, clip);
        let [line, tracking] = self.style_metrics(s);
        let style = &mut p.items.last_mut().expect("paint_text pushed an item").text;
        style.line_height = line;
        style.letter_spacing = tracking;
        style.weight = s.weight;
        style.font = s.mono.into();
    }
}
