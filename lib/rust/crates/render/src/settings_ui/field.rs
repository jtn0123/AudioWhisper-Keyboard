//! The editor family's painting: the single-line preview field and a row's multi-line field.
//! Caret, selection and hit testing use the same Android layout positions the text is drawn from.
use super::{Action, App, Control};
use crate::{
    android_text::Layout,
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// Caret x and its line's top and bottom.
fn caret(layout: &Layout, byte: usize) -> (f32, f32, f32) {
    let (line, x) = layout.caret(byte);
    layout
        .lines
        .get(line)
        .map_or((x, 0.0, layout.height), |l| (x, l.top, l.top + l.height))
}
impl App {
    /// PreviewKeyboardField: the bottom bar with the field and the keyboard picker.
    pub(super) fn preview_field(&mut self, p: &mut Panel, footer: f32, text: &mut Text) {
        let [w, h, d, _] = self.config;
        self.paint_box(p, rect(0.0, h - footer, w, footer), self.colors[5], 0.0);
        let window = rect(0.0, 0.0, w, h);
        let field = Control {
            id: 2,
            bounds: rect(0.0, h - footer, w - 48.0 * d, footer),
            label: self.label("settings__preview_keyboard"),
            summary: String::new(),
            role: "edit",
            enabled: true,
            checked: None,
            selected: false,
            action: Action::Editor,
            range: None,
        };
        self.add(p, field, window, text);
        let picker = Control {
            id: 3,
            bounds: rect(w - 48.0 * d, h - footer, 48.0 * d, footer),
            label: self.label("setup__select_ime__switch_keyboard_btn"),
            summary: String::new(),
            role: "button",
            enabled: true,
            checked: None,
            selected: false,
            action: Action::Picker,
            range: None,
        };
        self.add(p, picker, window, text);
    }
    /// Paints a field from its editor: the preview field's one line, shifted to keep the caret
    /// in view, or a row's OutlinedTextField wrapped inside its 16 dp padding.
    pub(super) fn paint_editor(&mut self, p: &mut Panel, c: &Control, text: &mut Text, clip: Rect) {
        let d = self.config[2];
        let size = self.sp(16);
        let b = c.bounds;
        let (e, active) = self.field_editor(c.id);
        let multi = c.id != 2 && !self.search_field(c.id);
        let pad = self.px(16.0);
        let value = String::from_utf16_lossy(&e.text);
        let available = if multi {
            b.width() - 2.0 * pad
        } else {
            (b.width() - 32.0 * d).max(1.0)
        };
        let layout = self.android(text, &value, size, multi.then_some(available));
        let byte = |at: usize| String::from_utf16_lossy(&e.text[..at]).len();
        let focus = caret(&layout, byte(e.focus));
        let editing = active && e.focused;
        let (x, y, field_clip) = if multi {
            let inside = rect(
                b.left + d,
                b.top + d,
                b.width() - 2.0 * d,
                b.height() - 2.0 * d,
            );
            let field_clip = crate::panel_layout::intersection(clip, inside);
            (b.left + pad, b.top + pad, field_clip)
        } else {
            let shift = (focus.0 - available + 2.0 * d).max(0.0);
            let y = b.top
                + if editing && e.anchor != e.focus {
                    2.0 * d
                } else {
                    (b.height() - layout.height) / 2.0
                };
            let inside = rect(b.left + 8.0 * d, b.top, b.width() - 16.0 * d, b.height());
            let field_clip = crate::panel_layout::intersection(clip, inside);
            (b.left + 16.0 * d - shift, y, field_clip)
        };
        if active {
            for [l, t, r, bottom] in layout.selection(byte(e.anchor), byte(e.focus)) {
                let at = rect(x + l, y + t, r - l, bottom - t);
                self.field_box(p, at, self.colors[4], field_clip);
            }
        }
        let display = if value.is_empty() {
            c.label.clone()
        } else {
            value
        };
        let bounds = if multi {
            let shown = self.android(text, &display, size, Some(available));
            rect(x, y, available, shown.height.max(size * 1.3))
        } else {
            let width = layout.width.max(available) + 2.0 * d;
            rect(x, y, width, layout.height.max(size * 1.3))
        };
        let color = if e.text.is_empty() {
            self.colors[2]
        } else {
            self.colors[1]
        };
        self.paint_text(p, &display, bounds, size, color, field_clip);
        if editing {
            let height = (focus.2 - focus.1).max(size);
            let at = rect(x + focus.0, y + focus.1, 2.0 * d, height);
            self.field_box(p, at, self.colors[3], field_clip);
        }
        if let Some((a, b)) = e.composing.filter(|_| active) {
            for [l, _, r, bottom] in layout.selection(byte(a), byte(b)) {
                let underline = rect(x + l, y + bottom - 2.0 * d, r - l, 2.0 * d);
                self.field_box(p, underline, self.colors[3], field_clip);
            }
        }
        if editing && e.anchor != e.focus {
            for (start, at) in [(true, e.anchor), (false, e.focus)] {
                let here = caret(&layout, byte(at));
                let other = caret(&layout, byte(if start { e.focus } else { e.anchor }));
                let leading = here.0 <= other.0;
                let diameter = 25.0 * d;
                let left = x + here.0 - if leading { diameter } else { 0.0 };
                let bounds = rect(left, y + here.2, diameter, diameter);
                self.paint_handle(p, (start, at), bounds, leading, clip);
            }
        }
        self.field_layouts.push((c.id, layout.clone(), [x, y]));
        if active {
            self.editor_layout = Some(layout);
            self.editor_origin = [x, y];
        }
    }
    /// A selection handle at `bounds`, its corner at the selection endpoint squared.
    fn paint_handle(
        &mut self,
        p: &mut Panel,
        (start, at): (bool, usize),
        bounds: Rect,
        leading: bool,
        clip: Rect,
    ) {
        let diameter = bounds.width();
        self.paint_box(p, bounds, self.colors[3], diameter / 2.0);
        let item = p.items.last_mut().expect("paint_box pushed an item");
        // M3 teardrop: circle with the corner at the selection endpoint squared.
        for style in &mut item.styles {
            style.corners[if leading { 1 } else { 0 }] = 0.0;
        }
        item.clip = clip;
        let label = if start {
            "settings__selection_start"
        } else {
            "settings__selection_end"
        };
        self.controls.push(Control {
            id: if start { 20 } else { 21 },
            bounds,
            label: self.label(label),
            summary: at.to_string(),
            role: "button",
            enabled: true,
            checked: None,
            selected: false,
            action: Action::SelectionHandle(start),
            range: None,
        });
    }
    fn field_box(&self, p: &mut Panel, b: Rect, color: u32, clip: Rect) {
        self.paint_clipped_box(p, b, color, 0.0, clip);
    }
    /// Puts the caret of field `id`, which owns the editor, where a tap at (`x`, `y`) landed.
    pub(super) fn place_editor_caret(&mut self, id: i32, x: f32, y: f32) {
        let Some((_, layout, [ox, oy])) = self.field_layouts.iter().find(|f| f.0 == id).cloned()
        else {
            return;
        };
        let index = layout.index_at(x - ox, y - oy);
        let value = String::from_utf16_lossy(&self.editor.text);
        let at = value[..index].encode_utf16().count() as i32;
        self.editor.select(at, at);
    }
    pub(super) fn handle_caret_x(&self, anchor: bool) -> Option<f32> {
        let layout = self.editor_layout.as_ref()?;
        let at = if anchor {
            self.editor.anchor
        } else {
            self.editor.focus
        };
        let byte = String::from_utf16_lossy(&self.editor.text[..at]).len();
        Some(self.editor_origin[0] + layout.caret(byte).1)
    }
    /// Moves the dragged handle's endpoint under the finger. The handle hangs below its line,
    /// so the line is the one 25 dp above the touch; a one-line field has only one.
    pub(super) fn drag_editor_handle(&mut self, x: f32, y: f32) {
        if let Some(layout) = &self.editor_layout {
            let [ox, oy] = self.editor_origin;
            let index = layout.index_at(x - ox, y - oy - 25.0 * self.config[2]);
            let value = String::from_utf16_lossy(&self.editor.text);
            let at = value[..index].encode_utf16().count() as i32;
            self.editor.select(at, self.handle_drag_other as i32);
        }
    }
}
