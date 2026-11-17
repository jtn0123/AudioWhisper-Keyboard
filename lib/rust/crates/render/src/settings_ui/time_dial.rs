//! The time dialog's clock dial and TimeInput's fields. The dial is ClockFace: its numbers on a
//! circle (12 hours, a 24-hour clock's afternoon on an inner ring, or every fifth minute), the
//! hand's 48 dp circle with the numbers under it in onPrimary, its 2 dp line and 8 dp centre dot.
//! Panels clip only to rectangles, so a number the circle only partly covers is drawn again in
//! one-pixel rows, each clipped to the circle's chord.
use super::{
    Action, App, Control,
    picker::{Surface, Tap},
    time::{self, TimePicker},
    time_dialog::{FIELD, LABEL, SUFFIX, kround},
    time_events::{HOUR_FIELD, TimeTap},
};
use crate::{
    panel_layout::intersection,
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;
use std::f32::consts::{FRAC_PI_2, TAU};

impl App {
    /// ClockFace in `b`: the numbers as buttons, then the dial over them taking the touches.
    pub(super) fn dial(
        &mut self,
        p: &mut Panel,
        t: &mut Text,
        s: &TimePicker,
        b: Rect,
        clip: Rect,
    ) {
        let is24 = self.is24();
        let d = b.width();
        self.paint_clipped_box(p, b, self.colors[5], d / 2.0, clip);
        let a = s.angle() - FRAC_PI_2;
        let reach = d * if s.inner(is24) { 69.0 } else { 101.0 } / 256.0;
        let centre = (b.left + d / 2.0, b.top + d / 2.0);
        let hand = (centre.0 + reach * a.cos(), centre.1 + reach * a.sin());
        let radius = 24.0 * self.config[2] * d / self.px(256.0);
        let glyphs = self.numbers(p, t, s, b, hand, clip);
        let circle = rect(hand.0 - radius, hand.1 - radius, 2.0 * radius, 2.0 * radius);
        self.paint_clipped_box(p, circle, self.colors[3], radius, clip);
        for (label, glyph) in glyphs {
            self.under_hand(p, &label, glyph, (hand, radius), clip);
        }
        let end = (hand.0 - radius * a.cos(), hand.1 - radius * a.sin());
        self.hand_line(p, centre, end, clip);
        let dot = 4.0 * self.config[2];
        let at = rect(centre.0 - dot, centre.1 - dot, 2.0 * dot, 2.0 * dot);
        self.paint_clipped_box(p, at, self.colors[3], dot, clip);
        self.controls.push(Control {
            id: 7132,
            bounds: b,
            label: String::new(),
            summary: String::new(),
            role: "none",
            enabled: true,
            checked: None,
            selected: false,
            action: Action::Drag(Surface::Dial),
            range: None,
        });
    }
    /// The dial's numbers in bodyLarge, each centred in a 48 dp box on its ring and a button
    /// setting its value; their labels and glyph boxes. The one under the hand is selected.
    fn numbers(
        &mut self,
        p: &mut Panel,
        t: &mut Text,
        s: &TimePicker,
        b: Rect,
        hand: (f32, f32),
        clip: Rect,
    ) -> Vec<(String, Rect)> {
        let is24 = self.is24();
        let d = b.width();
        let values: Vec<(u8, bool)> = if s.minutes {
            (0..12).map(|i| (i * 5, false)).collect()
        } else if is24 {
            (0..24).map(|v| (v, v >= 12)).collect()
        } else {
            (0..12)
                .map(|i| (if i == 0 { 12 } else { i }, false))
                .collect()
        };
        let suffix = self.label(
            SUFFIX[match (s.minutes, is24) {
                (true, _) => 2,
                (false, true) => 1,
                (false, false) => 0,
            }],
        );
        let cell = self.px(48.0);
        let offset = ((d as i32) / 2 - (cell as i32) / 2) as f32;
        let mut glyphs = vec![];
        for (k, &(value, inner)) in values.iter().enumerate() {
            let ring = d * if inner { 69.0 } else { 101.0 } / 256.0;
            let theta = TAU / 12.0 * (k % 12) as f32 - FRAC_PI_2;
            let at = rect(
                b.left + kround(ring * theta.cos() + offset),
                b.top + kround(ring * theta.sin() + offset),
                cell,
                cell,
            );
            let id = 7160 + k as i32;
            if self.focus == id {
                self.paint_clipped_box(p, at, self.colors[4], cell / 2.0, clip);
            }
            let label = value.to_string();
            let glyph = self.centred(p, t, &label, (self.font(16), self.colors[1]), at, clip);
            let description = suffix.replace("%1$d", &label);
            let selected = at.contains(hand.0, hand.1);
            self.tap_control(
                id,
                at,
                description,
                selected,
                Tap::Time(TimeTap::Number(value)),
            );
            glyphs.push((label, glyph));
        }
        glyphs
    }
    /// The part of a number inside the hand's circle, again in onPrimary: whole when the circle
    /// covers it, else row by row within the circle's chord.
    fn under_hand(
        &self,
        p: &mut Panel,
        label: &str,
        g: Rect,
        ((cx, cy), r): ((f32, f32), f32),
        clip: Rect,
    ) {
        let (f, color) = (self.font(16), self.colors[6]);
        let corners = [
            (g.left, g.top),
            (g.right, g.top),
            (g.left, g.bottom),
            (g.right, g.bottom),
        ];
        if corners.iter().all(|&(x, y)| (x - cx).hypot(y - cy) <= r) {
            return self.paint_font(p, label, g, f, color, clip);
        }
        if g.right < cx - r || g.left > cx + r {
            return;
        }
        let mut y = g.top.max(cy - r).floor();
        while y < g.bottom.min(cy + r) {
            let dy = (y + 0.5 - cy).abs();
            if dy < r {
                let half = (r * r - dy * dy).sqrt();
                let row = intersection(rect(cx - half, y, 2.0 * half, 1.0), g);
                if row.width() > 0.0 && row.height() > 0.0 {
                    self.paint_font(p, label, g, f, color, intersection(row, clip));
                }
            }
            y += 1.0;
        }
    }
    /// The hand's 2 dp line from the centre to the circle, drawn into a picture as panels have
    /// no slanted line.
    fn hand_line(&mut self, p: &mut Panel, from: (f32, f32), to: (f32, f32), clip: Rect) {
        let half = self.config[2];
        let pad = 2.0 * half + 1.0;
        let left = (from.0.min(to.0) - pad).floor();
        let top = (from.1.min(to.1) - pad).floor();
        let w = ((from.0.max(to.0) + pad).ceil() - left).max(1.0) as usize;
        let h = ((from.1.max(to.1) + pad).ceil() - top).max(1.0) as usize;
        let (a, b) = ((from.0 - left, from.1 - top), (to.0 - left, to.1 - top));
        let color = self.colors[3];
        let name = format!(
            "@hand:{w}x{h}:{:.2}:{:.2}:{:.2}:{:.2}:{color:08x}",
            a.0, a.1, b.0, b.1
        );
        self.cached(&name, w, h, "@hand:", |w, h| {
            time::segment(w, h, a, b, half, color)
        });
        self.paint_image(p, &name, rect(left, top, w as f32, h as f32), clip);
    }
    /// TimeInput: the selected value as a field and the other as a TimeSelector, each with its
    /// bodySmall label 7 dp under it, the separator between, a 12-hour clock's period toggle.
    pub(super) fn time_input(
        &mut self,
        p: &mut Panel,
        t: &mut Text,
        s: &TimePicker,
        (x, y): (f32, f32),
        clip: Rect,
    ) -> f32 {
        let (w, h, gap) = (self.px(96.0), self.px(72.0), self.px(24.0));
        let f = self.font(12);
        let support = self.support_height(t);
        for minutes in [false, true] {
            let left = x + if minutes { w + gap } else { 0.0 };
            let b = rect(left, y, w, h);
            if s.minutes == minutes {
                self.paint_time_field(p, t, minutes, b, clip);
            } else {
                self.selector(p, t, s, minutes, b, clip);
            }
            let label = self.label(LABEL[usize::from(minutes)]);
            let at = rect(left, y + h + self.px(7.0), w + 1.0, support);
            self.paint_font(p, &label, at, f, self.colors[2], clip);
        }
        self.separator(p, t, rect(x + w, y, gap, h), clip);
        if !self.is24() {
            let toggle = rect(x + 2.0 * w + gap + self.px(12.0), y, self.px(52.0), h);
            self.period(p, t, s, toggle, true, clip);
        }
        y + h + support + self.px(24.0)
    }
    /// An OutlinedTextField decoration round a BasicTextField: primaryContainer, 8 dp corners, a
    /// 2 dp primary border focused and 1 dp outline otherwise, the two digits in displayMedium
    /// centred, the caret over the middle 80 % of the line.
    fn paint_time_field(
        &mut self,
        p: &mut Panel,
        t: &mut Text,
        minutes: bool,
        b: Rect,
        clip: Rect,
    ) {
        let index = HOUR_FIELD + usize::from(minutes);
        let id = 1000 + index as i32;
        let (e, active) = self.field_editor(id);
        let editing = active && e.focused;
        let (stroke, edge) = if editing || self.focus == id {
            (self.px(2.0), self.colors[3])
        } else {
            (self.px(1.0), self.colors[7])
        };
        let r = self.px(8.0);
        self.paint_clipped_box(p, b, edge, r, clip);
        let inner = rect(
            b.left + stroke,
            b.top + stroke,
            b.width() - 2.0 * stroke,
            b.height() - 2.0 * stroke,
        );
        self.paint_clipped_box(p, inner, self.colors[4], r - stroke, clip);
        let f = self.font(45);
        let value = String::from_utf16_lossy(&e.text);
        let layout = t.android(&value, f.size, f.weight, [f.line, f.tracking], None);
        let height = layout.height.max(f.line).ceil();
        let (ox, oy) = (
            b.left + (b.width() - layout.width) / 2.0,
            b.top + kround((b.height() - height) / 2.0),
        );
        let field_clip = intersection(clip, inner);
        let byte = |at: usize| String::from_utf16_lossy(&e.text[..at.min(e.text.len())]).len();
        if active {
            // LocalTextSelectionColors' background: primary at 40 %.
            let tint = self.colors[3] & 0x00ff_ffff | 0x6600_0000;
            for [l, top, right, bottom] in layout.selection(byte(e.anchor), byte(e.focus)) {
                let at = rect(ox + l, oy + top, right - l, bottom - top);
                self.paint_clipped_box(p, at, tint, 0.0, field_clip);
            }
        }
        let at = rect(ox, oy, layout.width.ceil() + 1.0, height);
        self.paint_font(p, &value, at, f, self.colors[14], field_clip);
        if editing {
            let (_, x) = layout.caret(byte(e.focus));
            let caret = rect(ox + x, oy + 0.1 * height, self.px(2.0), 0.8 * height);
            self.paint_clipped_box(p, caret, self.colors[3], 0.0, field_clip);
        }
        self.field_layouts.push((id, layout.clone(), [ox, oy]));
        if active {
            self.editor_layout = Some(layout);
            self.editor_origin = [ox, oy];
        }
        self.controls.push(Control {
            id,
            bounds: b,
            label: self.label(FIELD[usize::from(minutes)]),
            summary: String::new(),
            role: "edit",
            enabled: true,
            checked: None,
            selected: false,
            action: Action::Field(index),
            range: None,
        });
    }
}
