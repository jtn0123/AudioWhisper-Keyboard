//! The time dialog drawn as Material 3 (1.4.0) lays out TimePicker and TimeInput in whole pixels
//! inside Jetpref's dialog: the picker or fields in 24 dp of their own padding, then the icon
//! button switching them. The picker stacks the time over the dial in a tall window and sets
//! them side by side in a wide one; the dial's numbers under the hand turn onPrimary.
use super::{
    Action, App, Control,
    picker::Tap,
    time::{self, TimePicker},
    time_events::TimeTap,
};
use crate::{
    panel_layout::intersection,
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

const AM: &str = "settings__time_picker__am";
const PM: &str = "settings__time_picker__pm";
const SELECT: [&str; 2] = [
    "settings__time_picker__select_hour",
    "settings__time_picker__select_minutes",
];
/// A clock number's description: o'clock, a 24-hour clock's hours, minutes.
pub(super) const SUFFIX: [&str; 3] = [
    "settings__time_picker__hour_suffix",
    "settings__time_picker__hour_24h_suffix",
    "settings__time_picker__minute_suffix",
];
pub(super) const LABEL: [&str; 2] = [
    "settings__time_picker__hour",
    "settings__time_picker__minute",
];
pub(super) const FIELD: [&str; 2] = [
    "settings__time_picker__hour_field",
    "settings__time_picker__minute_field",
];
/// Jetpref's toggle descriptions, literal English there.
const MODE: [&str; 2] = [
    "settings__time_picker__to_keyboard",
    "settings__time_picker__to_picker",
];
pub const KEYS: [&str; 13] = [
    AM, PM, SELECT[0], SELECT[1], SUFFIX[0], SUFFIX[1], SUFFIX[2], LABEL[0], LABEL[1], FIELD[0],
    FIELD[1], MODE[0], MODE[1],
];

/// A text style of Material's type scale in pixels.
#[derive(Clone, Copy)]
pub(super) struct Font {
    pub(super) size: f32,
    pub(super) line: f32,
    pub(super) tracking: f32,
    pub(super) weight: i32,
}

/// Kotlin's `roundToInt`: halves round up.
pub(super) fn kround(v: f32) -> f32 {
    (v + 0.5).floor()
}

impl App {
    /// The type scale's sizes the picker uses; sp beyond 32 come from Kotlin as `display_pixels`.
    pub(super) fn font(&self, style: usize) -> Font {
        let [d, scale] = [self.config[2], self.config[3]];
        let sp = |n: f32| n * d * scale;
        let display = |i: usize, n: f32| match self.display_pixels[i] {
            v if v > 0.0 => v,
            _ => sp(n),
        };
        // (size, line height in sp of the size, tracking in sp, weight)
        let (size, line, tracking, weight) = match style {
            57 => (display(1, 57.0), 64.0 / 57.0, -0.2, 400),
            45 => (display(0, 45.0), 52.0 / 45.0, 0.0, 400),
            14 => (self.sp(14), 20.0 / 14.0, 0.1, 500),
            12 => (self.sp(12), 16.0 / 12.0, 0.4, 400),
            // The app's bodyLarge: no line height, no tracking.
            _ => (self.sp(16), 0.0, 0.0, 400),
        };
        Font {
            size,
            line: size * line,
            tracking: sp(tracking),
            weight,
        }
    }
    /// `label` in `f` at `at`, as wide as `at` before it wraps.
    pub(super) fn paint_font(
        &self,
        p: &mut Panel,
        label: &str,
        at: Rect,
        f: Font,
        color: u32,
        clip: Rect,
    ) {
        self.paint_text(p, label, at, f.size, color, clip);
        if let Some(item) = p.items.last_mut() {
            (item.text.line_height, item.text.letter_spacing) = (f.line, f.tracking);
            item.text.weight = f.weight;
        }
    }
    /// `label` centred in `b` as Box(contentAlignment = Center) places a Text, in font `f` and
    /// `color`; its box.
    pub(super) fn centred(
        &self,
        p: &mut Panel,
        t: &mut Text,
        label: &str,
        (f, color): (Font, u32),
        b: Rect,
        clip: Rect,
    ) -> Rect {
        let layout = t.android(label, f.size, f.weight, [f.line, f.tracking], None);
        let (w, h) = (layout.width.ceil(), layout.height.ceil());
        let at = rect(
            b.left + kround((b.width() - w) / 2.0),
            b.top + kround((b.height() - h) / 2.0),
            w + 1.0,
            h,
        );
        self.paint_font(p, label, at, f, color, clip);
        at
    }
    /// A 3 dp primary ring round a focused control.
    fn focus_outline(&self, p: &mut Panel, id: i32, b: Rect, clip: Rect) {
        if self.focus == id {
            let g = self.px(3.0);
            let r = rect(
                b.left - g,
                b.top - g,
                b.width() + 2.0 * g,
                b.height() + 2.0 * g,
            );
            self.paint_clipped_box(p, r, self.colors[3], self.px(8.0) + g, clip);
        }
    }
    /// Whether the picker sets the time beside the dial (`TimePickerLayoutType.Horizontal`).
    fn horizontal(&self) -> bool {
        self.config[1] < self.config[0]
    }
    /// The time's row: two TimeSelectors round the separator, and a 12-hour clock's period
    /// toggle 12 dp after them.
    fn numbers_width(&self) -> f32 {
        let w = 2.0 * self.px(96.0) + self.px(24.0);
        if self.is24() {
            w
        } else {
            w + self.px(12.0) + self.px(52.0)
        }
    }
    /// The picker or fields in their 24 dp padding: the dialog content's intrinsic width.
    fn time_inner(&self, p: &TimePicker) -> f32 {
        let inner = if p.input {
            self.numbers_width()
        } else if self.horizontal() {
            2.0 * self.px(96.0) + self.px(24.0) + self.px(36.0) + self.px(256.0)
        } else {
            self.numbers_width().max(self.px(256.0))
        };
        inner + 2.0 * self.px(24.0)
    }
    /// Jetpref's `width(IntrinsicSize.Min)` dialog: its content in 24 dp padding, 280 to 560 dp,
    /// and 16 dp inside the window.
    pub(super) fn time_width(&self) -> f32 {
        let inner = self.time_picker().map_or(0.0, |p| self.time_inner(p));
        (inner + 2.0 * self.px(24.0))
            .clamp(self.px(280.0), self.px(560.0))
            .min(self.config[0] - 32.0 * self.config[2])
    }
    /// The content's height: the picker or fields, then the 48 dp icon button.
    pub(super) fn time_height(&self, t: &mut Text) -> f32 {
        let Some(p) = self.time_picker() else {
            return 0.0;
        };
        let body = if p.input {
            self.px(72.0) + self.support_height(t) + self.px(24.0)
        } else if self.horizontal() {
            self.px(256.0)
        } else {
            self.px(80.0) + self.px(36.0) + self.px(256.0) + self.px(24.0)
        };
        body + self.px(48.0)
    }
    pub(super) fn support_height(&self, t: &mut Text) -> f32 {
        let f = self.font(12);
        let label = self.label(LABEL[0]);
        t.android(&label, f.size, f.weight, [f.line, f.tracking], None)
            .height
            .ceil()
    }
    /// Paints the dialog's content from `y`; returns the y under it.
    pub(super) fn time_content(
        &mut self,
        p: &mut Panel,
        t: &mut Text,
        (left, y): (f32, f32),
        width: f32,
        clip: Rect,
    ) -> f32 {
        let Some(s) = self.time_picker().cloned() else {
            return y;
        };
        let x = left + self.px(24.0);
        let inner_x = x + self.px(24.0);
        let inner_w = width - 4.0 * self.px(24.0);
        let bottom = if s.input {
            self.time_input(p, t, &s, (inner_x, y), clip)
        } else if self.horizontal() {
            self.side_by_side(p, t, &s, (inner_x, y, inner_w), clip)
        } else {
            self.stacked(p, t, &s, (inner_x, y), clip)
        };
        let b = rect(x, bottom, self.px(48.0), self.px(48.0));
        self.focus_ring(p, 7154, b, clip);
        let s24 = self.px(24.0);
        let glyph = rect(b.left + self.px(12.0), b.top + self.px(12.0), s24, s24);
        let icon = if s.input { "clock" } else { "picker" };
        self.paint_icon(p, icon, glyph, self.colors[2], clip);
        let label = self.label(MODE[usize::from(s.input)]);
        self.tap_control(7154, b, label, false, Tap::Time(TimeTap::Mode));
        bottom + self.px(48.0)
    }
    /// VerticalTimePicker: the time centred over the dial, 36 dp between, 24 dp under it.
    fn stacked(
        &mut self,
        p: &mut Panel,
        t: &mut Text,
        s: &TimePicker,
        (x, y): (f32, f32),
        clip: Rect,
    ) -> f32 {
        let column = self.numbers_width().max(self.px(256.0));
        let nx = x + kround((column - self.numbers_width()) / 2.0);
        self.time_row(p, t, s, (nx, y), self.px(80.0), clip);
        let top = y + self.px(80.0) + self.px(36.0);
        let dial = rect(
            x + kround((column - self.px(256.0)) / 2.0),
            top,
            self.px(256.0),
            self.px(256.0),
        );
        self.dial(p, t, s, dial, clip);
        dial.bottom + self.px(24.0)
    }
    /// HorizontalTimePicker: the time (and the wide period toggle under it) centred beside the
    /// dial. A slot narrower than the dial centres the dial on it, as Compose places a child
    /// that measured larger than its constraints.
    fn side_by_side(
        &mut self,
        p: &mut Panel,
        t: &mut Text,
        s: &TimePicker,
        (x, y, w): (f32, f32, f32),
        clip: Rect,
    ) -> f32 {
        let (row, dial) = (2.0 * self.px(96.0) + self.px(24.0), self.px(256.0));
        let column = self.px(80.0)
            + if self.is24() {
                0.0
            } else {
                self.px(12.0) + self.px(38.0)
            };
        let top = y + kround((dial - column) / 2.0);
        self.time_row(p, t, s, (x, top), self.px(80.0), clip);
        if !self.is24() {
            let toggle = rect(x, top + self.px(80.0) + self.px(12.0), row, self.px(38.0));
            self.period(p, t, s, toggle, false, clip);
        }
        let slot = (w - row - self.px(36.0)).clamp(0.0, dial);
        let left = x + row + self.px(36.0) + ((slot - dial) / 2.0).trunc();
        self.dial(p, t, s, rect(left, y, dial, dial), clip);
        y + dial
    }
    /// The hour and minute TimeSelectors round the separator, `h` tall; the stacked picker's
    /// period toggle after them.
    fn time_row(
        &mut self,
        p: &mut Panel,
        t: &mut Text,
        s: &TimePicker,
        (x, y): (f32, f32),
        h: f32,
        clip: Rect,
    ) {
        let (w, gap) = (self.px(96.0), self.px(24.0));
        for minutes in [false, true] {
            let left = x + if minutes { w + gap } else { 0.0 };
            self.selector(p, t, s, minutes, rect(left, y, w, h), clip);
        }
        self.separator(p, t, rect(x + w, y, gap, h), clip);
        if !self.is24() && !self.horizontal() {
            let toggle = rect(x + 2.0 * w + gap + self.px(12.0), y, self.px(52.0), h);
            self.period(p, t, s, toggle, true, clip);
        }
    }
    /// TimeSelector: the value in two digits on primaryContainer when selected, a radio button.
    pub(super) fn selector(
        &mut self,
        p: &mut Panel,
        t: &mut Text,
        s: &TimePicker,
        minutes: bool,
        b: Rect,
        clip: Rect,
    ) {
        let selected = s.minutes == minutes;
        let value = if minutes {
            s.minute
        } else {
            time::display_hour(s.hour, self.is24())
        };
        let id = 7150 + i32::from(minutes);
        self.focus_outline(p, id, b, clip);
        let (back, fore) = if selected {
            (self.colors[4], self.colors[14])
        } else {
            (self.colors[5], self.colors[1])
        };
        self.paint_clipped_box(p, b, back, self.px(8.0), clip);
        let f = self.font(if b.height() > self.px(72.0) { 57 } else { 45 });
        self.centred(p, t, &format!("{value:02}"), (f, fore), b, clip);
        self.controls.push(Control {
            id,
            bounds: b,
            label: self.label(SELECT[usize::from(minutes)]),
            summary: String::new(),
            role: "radio",
            enabled: true,
            checked: None,
            selected,
            action: Action::Tap(Tap::Time(TimeTap::Select(minutes))),
            range: None,
        });
    }
    /// The ":" between them, trimmed to its glyph's line and centred.
    pub(super) fn separator(&self, p: &mut Panel, t: &mut Text, b: Rect, clip: Rect) {
        let f = self.font(if b.height() > self.px(72.0) { 57 } else { 45 });
        self.centred(
            p,
            t,
            ":",
            (Font { line: 0.0, ..f }, self.colors[1]),
            b,
            clip,
        );
    }
    /// The period toggle: AM over PM (or beside it), the selected half on tertiaryContainer, a
    /// 1 dp outline border drawn over both and a 1 dp divider between them.
    pub(super) fn period(
        &mut self,
        p: &mut Panel,
        t: &mut Text,
        s: &TimePicker,
        b: Rect,
        vertical: bool,
        clip: Rect,
    ) {
        let (line, r) = (self.px(1.0), self.px(8.0));
        self.paint_clipped_box(p, b, self.colors[7], r, clip);
        let inner = rect(
            b.left + line,
            b.top + line,
            b.width() - 2.0 * line,
            b.height() - 2.0 * line,
        );
        self.paint_clipped_box(p, inner, self.colors[11], r - line, clip);
        let half = if vertical {
            [b.width(), (b.height() / 2.0).floor()]
        } else {
            [(b.width() / 2.0).floor(), b.height()]
        };
        for pm in [false, true] {
            let at = match (pm, vertical) {
                (false, _) => rect(b.left, b.top, half[0], half[1]),
                (true, true) => rect(b.left, b.top + half[1], half[0], half[1]),
                (true, false) => rect(b.left + half[0], b.top, half[0], half[1]),
            };
            let id = 7152 + i32::from(pm);
            let checked = s.pm() == pm;
            self.focus_outline(p, id, at, clip);
            if checked {
                let c = r - line;
                self.paint_clipped_box(p, intersection(at, inner), self.colors[16], c, clip);
                if let Some(item) = p.items.last_mut() {
                    let corners = match (pm, vertical) {
                        (false, true) => [c, c, 0.0, 0.0],
                        (true, true) => [0.0, 0.0, c, c],
                        (false, false) => [c, 0.0, 0.0, c],
                        (true, false) => [0.0, c, c, 0.0],
                    };
                    item.styles.iter_mut().for_each(|st| st.corners = corners);
                }
            }
            let label = self.label(if pm { PM } else { AM });
            let color = self.colors[if checked { 17 } else { 2 }];
            self.centred(p, t, &label, (self.font(14), color), at, clip);
            self.tap_control(id, at, label, checked, Tap::Time(TimeTap::Period(pm)));
        }
        let divider = if vertical {
            rect(
                b.left,
                b.top + half[1] - (line / 2.0).floor(),
                b.width(),
                line,
            )
        } else {
            rect(
                b.left + half[0] - (line / 2.0).floor(),
                b.top,
                line,
                b.height(),
            )
        };
        self.paint_clipped_box(p, divider, self.colors[7], 0.0, clip);
    }
}
