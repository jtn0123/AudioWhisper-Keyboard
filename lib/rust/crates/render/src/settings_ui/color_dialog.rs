//! The colour dialog's two pages as Jetpref lays them out in whole pixels inside the dialog's
//! 24 dp content padding. Presets: a `GridCells.Adaptive(48.dp)` grid of 50 dp ColorBoxes (4 dp
//! padding, the circle with a 1 dp outline ring and a 1 dp ring of the colour itself, the row's
//! icon as the tick), 16 dp, then the Default row. Custom: a square saturation/value plane, a
//! 64 dp preview beside the hue bar (12 dp padding round a 12 dp track), and the filled text
//! field in 12 dp vertical padding. In the editor the plane and that row draw at half alpha with
//! no thumbs and take no touches.
use super::{
    Action, App, Control,
    color::{self, PRESETS},
    picker::{ColorPicker, HEX_FIELD, Surface, Tap},
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// Jetpref's literal labels, as English strings the app ships untranslated.
pub const PRESETS_KEY: &str = "settings__color_picker__presets";
pub const CUSTOM_KEY: &str = "settings__color_picker__custom";
const FIELD_KEY: &str = "settings__color_picker__field";
const EDIT_KEYS: [&str; 3] = [
    "settings__color_picker__edit",
    "settings__color_picker__apply",
    "settings__color_picker__discard",
];
pub const KEYS: [&str; 6] = [
    PRESETS_KEY,
    CUSTOM_KEY,
    FIELD_KEY,
    EDIT_KEYS[0],
    EDIT_KEYS[1],
    EDIT_KEYS[2],
];

impl App {
    /// The grid's column widths: Adaptive(48 dp) over the content width, the remainder pixels
    /// going one each to the first columns.
    fn columns(&self, w: f32) -> Vec<f32> {
        let w = w.floor();
        let count = (w / self.px(48.0)).floor().max(1.0);
        let base = (w / count).floor();
        let extra = w - base * count;
        (0..count as usize)
            .map(|i| base + f32::from(u8::from((i as f32) < extra)))
            .collect()
    }
    /// The content's height in a dialog `width` wide.
    pub(super) fn color_height(&self, width: f32, text: &mut Text) -> f32 {
        let Some(p) = self.color_picker() else {
            return 0.0;
        };
        let w = (width - 2.0 * self.px(24.0)).floor();
        if !p.custom {
            let rows = PRESETS.len().div_ceil(self.columns(w).len()) as f32;
            return (rows + 1.0) * self.px(50.0) + self.px(16.0);
        }
        w + self.px(12.0) + self.px(64.0) + 2.0 * self.px(12.0) + self.hex_height(w, text)
    }
    /// The filled field's height: 8 dp, the 16 sp label line, the text, 8 dp; 56 dp at least.
    fn hex_height(&self, w: f32, text: &mut Text) -> f32 {
        let Some(p) = self.color_picker() else {
            return 0.0;
        };
        let shown = self.hex_text(p);
        let width = (w - self.px(16.0) - self.trailing(p)).max(1.0);
        let lines = self.android(text, &shown, self.sp(16), Some(width)).height;
        (self.px(16.0) + self.sp(16).max(lines) + self.line_height(self.sp(12)))
            .max(self.px(56.0))
            .ceil()
    }
    fn trailing(&self, p: &ColorPicker) -> f32 {
        self.px(48.0) * if p.editing { 2.0 } else { 1.0 }
    }
    /// What the field shows: the colour in its format, or the text being edited.
    fn hex_text(&self, p: &ColorPicker) -> String {
        match (p.editing, self.field == Some(HEX_FIELD)) {
            (true, true) => String::from_utf16_lossy(&self.editor.text),
            (true, false) => p.text.clone(),
            _ => color::format(p.format, p.color()),
        }
    }
    /// Paints the open page from `y`; returns the y under it.
    pub(super) fn color_content(
        &mut self,
        p: &mut Panel,
        text: &mut Text,
        (left, y): (f32, f32),
        width: f32,
        clip: Rect,
    ) -> f32 {
        let Some(state) = self.color_picker().cloned() else {
            return y;
        };
        let x = left + self.px(24.0);
        let w = (width - 2.0 * self.px(24.0)).floor();
        if state.custom {
            self.custom_page(p, text, &state, (x, y, w), clip)
        } else {
            self.presets_page(p, text, &state, (x, y, w), clip)
        }
    }
    fn presets_page(
        &mut self,
        p: &mut Panel,
        text: &mut Text,
        s: &ColorPicker,
        (x, y, w): (f32, f32, f32),
        clip: Rect,
    ) -> f32 {
        let columns = self.columns(w);
        let size = self.px(50.0);
        let alpha = s.value & 0xff00_0000;
        for (j, preset) in PRESETS.iter().enumerate() {
            let (row, column) = (j / columns.len(), j % columns.len());
            let cell_x = x + columns[..column].iter().sum::<f32>();
            let cell_w = columns[column];
            // A requiredSize box wider than its cell is centred on it, as Compose places it.
            let bx = cell_x + ((cell_w - size) / 2.0).trunc();
            let b = rect(bx, y + row as f32 * size, size, size);
            let selected = s.preset == Some(j);
            let c = preset & 0x00ff_ffff | alpha;
            self.focus_ring(p, 7100 + j as i32, b, clip);
            self.color_box(p, b, c, selected, clip);
            self.tap_control(
                7100 + j as i32,
                b,
                color::format(color::Format::Hex, c),
                selected,
                Tap::Swatch(j),
            );
        }
        let rows = PRESETS.len().div_ceil(columns.len()) as f32;
        let top = y + rows * size + self.px(16.0);
        let b = rect(x, top, size, size);
        let ticked = s.value | 0xff00_0000 == s.accent;
        self.focus_ring(p, 7120, rect(x, top, w, size), clip);
        self.color_box(p, b, s.accent, ticked, clip);
        let label = self.label("action__default");
        let height = self.android(text, &label, self.sp(16), None).height;
        let tx = x + size + self.px(4.0);
        let tb = rect(
            tx,
            top + ((size - height) / 2.0).round(),
            w - (tx - x),
            height,
        );
        self.paint_text(p, &label, tb, self.sp(16), self.colors[2], clip);
        self.tap_control(7120, rect(x, top, w, size), label, ticked, Tap::Accent);
        top + size
    }
    /// ColorBox: the circle inside 4 dp padding, its outline and colour rings, the colour, and
    /// the row's icon as the tick, white on a dark colour and black on a light one.
    fn color_box(&mut self, p: &mut Panel, b: Rect, c: u32, selected: bool, clip: Rect) {
        let pad = self.px(4.0);
        let circle = rect(
            b.left + pad,
            b.top + pad,
            b.width() - 2.0 * pad,
            b.height() - 2.0 * pad,
        );
        self.swatch_circle(p, circle, c, clip);
        if selected {
            let icon = self.modal.as_ref().map_or("", |m| self.rows()[m.row].icon);
            let tint = if color::luminance(c) < 0.5 {
                0xffffffff
            } else {
                0xff000000
            };
            let s = self.px(24.0);
            let at = rect(
                b.left + (b.width() - s) / 2.0,
                b.top + (b.height() - s) / 2.0,
                s,
                s,
            );
            self.paint_icon(p, icon, at, tint, clip);
        }
    }
    /// A colour circle with Jetpref's 1 dp outline ring and 1 dp ring of the opaque colour; a
    /// translucent colour shows over the checkered backing inside them.
    pub(super) fn swatch_circle(&mut self, p: &mut Panel, b: Rect, c: u32, clip: Rect) {
        let d = self.config[2];
        let ring = |n: f32| {
            rect(
                b.left + n * d,
                b.top + n * d,
                b.width() - 2.0 * n * d,
                b.height() - 2.0 * n * d,
            )
        };
        self.paint_clipped_box(p, b, self.colors[7], b.width() / 2.0, clip);
        self.paint_clipped_box(p, ring(1.0), c | 0xff00_0000, ring(1.0).width() / 2.0, clip);
        if c >> 24 != 0xff {
            let inner = ring(2.0);
            self.paint_clipped_box(p, inner, self.colors[11], inner.width() / 2.0, clip);
            self.checkered(p, inner, clip);
            self.paint_clipped_box(p, inner, c, inner.width() / 2.0, clip);
        }
    }
    /// `checkeredBackground`: 6 dp squares of onBackground at 16 % in a circle.
    fn checkered(&mut self, p: &mut Panel, b: Rect, clip: Rect) {
        let n = b.width().round().max(1.0) as usize;
        let even = 0x2900_0000 | self.colors[15] & 0x00ff_ffff;
        let name = format!("@checkered:{n}:{even:08x}");
        if !self.images.contains_key(&name) {
            let frame = crate::media_cache::Frame::argb(
                n as u16,
                n as u16,
                &color::checkered(n, 6.0 * self.config[2], even),
                0,
            );
            self.images.retain(|k, _| !k.starts_with("@checkered:"));
            self.images.extend(frame.map(|f| (name.clone(), f)));
        }
        self.paint_image(p, &name, b, clip);
    }
    pub(super) fn tap_control(
        &mut self,
        id: i32,
        b: Rect,
        label: String,
        selected: bool,
        tap: Tap,
    ) {
        self.controls.push(Control {
            id,
            bounds: b,
            label,
            summary: String::new(),
            role: "button",
            enabled: true,
            checked: None,
            selected,
            action: Action::Tap(tap),
            range: None,
        });
    }
    fn custom_page(
        &mut self,
        p: &mut Panel,
        text: &mut Text,
        s: &ColorPicker,
        (x, y, w): (f32, f32, f32),
        clip: Rect,
    ) -> f32 {
        let d = self.config[2];
        let [h, sat, v] = s.hsv;
        let plane = rect(x, y, w, w);
        let hue = color::hsv(h, 1.0, 1.0, 1.0);
        let n = w.round().clamp(1.0, 700.0) as usize;
        let name = format!("@plane:{n}:{hue:08x}:{:.0}", 4.0 * d * n as f32 / w);
        self.cached(&name, n, n, "@plane:", |n, _| {
            color::plane(n, n, hue, 4.0 * d * n as f32 / w)
        });
        self.paint_image(p, &name, plane, clip);
        let row = y + w + self.px(12.0);
        let preview = rect(x + self.px(2.0), row, self.px(64.0), self.px(64.0));
        self.swatch_circle(p, preview, s.color(), clip);
        let track_left = preview.right + self.px(16.0) + self.px(12.0);
        let track_w = (x + w - self.px(12.0) - track_left).max(1.0);
        let track_top = row + ((self.px(64.0) - 3.0 * self.px(12.0)) / 2.0).round() + self.px(12.0);
        let track = rect(track_left, track_top, track_w, self.px(12.0));
        let (tw, th) = (
            track_w.round().max(1.0) as usize,
            track.height().round().max(1.0) as usize,
        );
        let bar = format!("@hue:{tw}x{th}");
        self.cached(&bar, tw, th, "@hue:", color::hue_bar);
        self.paint_image(p, &bar, track, clip);
        let centre = track.top + track.height() / 2.0;
        if s.editing {
            // Modifier.alpha(0.5) over the dialog surface.
            let veil = self.colors[11] & 0x00ff_ffff | 0x8000_0000;
            self.paint_clipped_box(p, plane, veil, 4.0 * d, clip);
            self.paint_clipped_box(p, rect(x, row, w, self.px(64.0)), veil, 0.0, clip);
        } else {
            self.thumb(p, (x + w * sat, y + w * (1.0 - v)), s.color(), clip);
            self.thumb(p, (track_left + track_w * h / 360.0, centre), hue, clip);
            self.drag_control(7130, plane, Surface::Plane);
            let touch = rect(track_left, centre - self.px(24.0), track_w, self.px(48.0));
            self.drag_control(7131, touch, Surface::Hue);
        }
        let top = row + self.px(64.0) + self.px(12.0);
        let height = self.hex_height(w, text);
        self.hex_field(p, text, s, rect(x, top, w, height), clip);
        top + height + self.px(12.0)
    }
    /// An image named `name` drawn at `w` x `h` pixels, made once; others of its kind dropped.
    pub(super) fn cached(
        &mut self,
        name: &str,
        w: usize,
        h: usize,
        kind: &str,
        make: impl Fn(usize, usize) -> Vec<i32>,
    ) {
        if self.images.contains_key(name) {
            return;
        }
        let frame = crate::media_cache::Frame::argb(w as u16, h as u16, &make(w, h), 0);
        self.images.retain(|k, _| !k.starts_with(kind));
        self.images.extend(frame.map(|f| (name.to_owned(), f)));
    }
    /// A 24 dp thumb centred at `at`: a 4 dp white ring at 4 dp elevation round the colour.
    fn thumb(&self, p: &mut Panel, (cx, cy): (f32, f32), fill: u32, clip: Rect) {
        let d = self.config[2];
        let size = 24.0 * d;
        let b = rect(
            (cx - size / 2.0).round(),
            (cy - size / 2.0).round(),
            size,
            size,
        );
        let reach = 4.0 * d * 3.0;
        let mut outer = rect(
            b.left - reach,
            b.top - reach,
            size + 2.0 * reach,
            size + 2.0 * reach,
        );
        self.paint_clipped_box(p, outer, 0xffffffff, size / 2.0, clip);
        let item = p.items.last_mut().unwrap();
        for style in &mut item.styles {
            style.margin = [reach; 4];
            style.shadow = 4.0 * d;
        }
        outer = rect(
            b.left + 4.0 * d,
            b.top + 4.0 * d,
            size - 8.0 * d,
            size - 8.0 * d,
        );
        self.paint_clipped_box(p, outer, fill | 0xff00_0000, outer.width() / 2.0, clip);
    }
    fn drag_control(&mut self, id: i32, b: Rect, surface: Surface) {
        self.controls.push(Control {
            id,
            bounds: b,
            label: String::new(),
            summary: String::new(),
            role: "none",
            enabled: true,
            checked: None,
            selected: false,
            action: Action::Drag(surface),
            range: None,
        });
    }
}

impl App {
    /// A focused control's highlight, as the other dialogs draw keyboard focus.
    pub(super) fn focus_ring(&self, p: &mut Panel, id: i32, b: Rect, clip: Rect) {
        if self.focus == id {
            self.paint_clipped_box(p, b, self.colors[4], 8.0 * self.config[2], clip);
        }
    }
    /// JetPrefTextField's filled field: surfaceContainerHighest with 4 dp top corners, a 1 dp
    /// onSurfaceVariant indicator (2 dp primary focused, error after a failed Apply), the
    /// bodySmall "Color" label over bodyLarge text, and the Edit, or Apply and Discard, icons.
    fn hex_field(&mut self, p: &mut Panel, text: &mut Text, s: &ColorPicker, b: Rect, clip: Rect) {
        let d = self.config[2];
        let owned = self.field == Some(HEX_FIELD);
        let focused = (owned && self.editor.focused) || self.focus == 7000;
        let tone = if s.error {
            self.colors[18]
        } else if focused {
            self.colors[3]
        } else {
            self.colors[2]
        };
        self.paint_clipped_box(p, b, self.colors[5], 4.0 * d, clip);
        if let Some(item) = p.items.last_mut() {
            item.styles
                .iter_mut()
                .for_each(|style| style.corners = [4.0 * d, 4.0 * d, 0.0, 0.0]);
        }
        let stroke = if focused { 2.0 } else { 1.0 } * d;
        let line = if s.error || focused {
            tone
        } else {
            self.colors[2]
        };
        self.paint_clipped_box(
            p,
            rect(b.left, b.bottom - stroke, b.width(), stroke),
            line,
            0.0,
            clip,
        );
        let label = self.label(FIELD_KEY);
        let shown = self.hex_text(s);
        let label_line = self.line_height(self.sp(12)).ceil();
        let (pad, top) = (self.px(16.0), b.top + self.px(8.0));
        if !shown.is_empty() || focused {
            self.paint_text(
                p,
                &label,
                rect(b.left + pad, top, b.width() - 2.0 * pad, label_line),
                self.sp(12),
                tone,
                clip,
            );
        }
        let text_w = (b.width() - pad - self.trailing(s)).max(1.0);
        let field = Control {
            id: 1000 + HEX_FIELD as i32,
            bounds: b,
            label: label.clone(),
            summary: String::new(),
            role: "edit",
            enabled: true,
            checked: None,
            selected: false,
            action: Action::Field(HEX_FIELD),
            range: None,
        };
        if s.editing {
            // The shared painter puts the text 16 dp inside its bounds.
            let drawn = Control {
                bounds: rect(
                    b.left,
                    top + label_line - pad,
                    text_w + 2.0 * pad,
                    b.height(),
                ),
                label: if focused { String::new() } else { label },
                ..field.clone()
            };
            self.paint_editor(p, &drawn, text, clip);
            self.controls.push(field);
        } else {
            let height = self.android(text, &shown, self.sp(16), Some(text_w)).height;
            self.paint_text(
                p,
                &shown,
                rect(b.left + pad, top + label_line, text_w, height),
                self.sp(16),
                self.colors[1],
                clip,
            );
            self.controls.push(Control {
                summary: shown,
                role: "text",
                action: Action::None,
                ..field
            });
        }
        let icons: &[(&str, Tap, i32)] = if s.editing {
            &[("done", Tap::Apply, 1), ("clear", Tap::Discard, 2)]
        } else {
            &[("edit", Tap::Edit, 0)]
        };
        let icon_tone = if s.error {
            self.colors[18]
        } else {
            self.colors[2]
        };
        for (k, (icon, tap, n)) in icons.iter().enumerate() {
            let box_left = b.right - self.px(48.0) * (icons.len() - k) as f32;
            let at = rect(
                box_left,
                b.top + ((b.height() - self.px(48.0)) / 2.0).round(),
                self.px(48.0),
                self.px(48.0),
            );
            self.focus_ring(p, 7140 + n, at, clip);
            let glyph = rect(
                at.left + self.px(12.0),
                at.top + self.px(12.0),
                self.px(24.0),
                self.px(24.0),
            );
            self.paint_icon(p, icon, glyph, icon_tone, clip);
            self.tap_control(
                7140 + n,
                at,
                self.label(EDIT_KEYS[*n as usize]),
                false,
                *tap,
            );
        }
    }
}
