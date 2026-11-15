//! Single-line preview field; caret, selection and hit testing use the same Android layout
//! positions the text is drawn from.
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
    pub(super) fn paint_editor(&mut self, p: &mut Panel, c: &Control, text: &mut Text, clip: Rect) {
        let d = self.config[2];
        let size = self.sp(16);
        let b = c.bounds;
        let value = String::from_utf16_lossy(&self.editor.text);
        let layout = self.android(text, &value, size, None);
        let byte = |at: usize| String::from_utf16_lossy(&self.editor.text[..at]).len();
        let focus = caret(&layout, byte(self.editor.focus));
        let available = (b.width() - 32.0 * d).max(1.0);
        let shift = (focus.0 - available + 2.0 * d).max(0.0);
        let x = b.left + 16.0 * d - shift;
        let y = b.top
            + if self.editor.focused && self.editor.anchor != self.editor.focus {
                2.0 * d
            } else {
                (b.height() - layout.height) / 2.0
            };
        let field_clip = crate::panel_layout::intersection(
            clip,
            rect(b.left + 8.0 * d, b.top, b.width() - 16.0 * d, b.height()),
        );
        for [l, t, r, bottom] in layout.selection(byte(self.editor.anchor), byte(self.editor.focus))
        {
            self.field_box(
                p,
                rect(x + l, y + t, r - l, bottom - t),
                self.colors[4],
                field_clip,
            );
        }
        let display = if value.is_empty() {
            c.label.clone()
        } else {
            value
        };
        self.paint_text(
            p,
            &display,
            rect(
                x,
                y,
                layout.width.max(available) + 2.0 * d,
                layout.height.max(size * 1.3),
            ),
            size,
            if self.editor.text.is_empty() {
                self.colors[2]
            } else {
                self.colors[1]
            },
            field_clip,
        );
        if self.editor.focused {
            self.field_box(
                p,
                rect(
                    x + focus.0,
                    y + focus.1,
                    2.0 * d,
                    (focus.2 - focus.1).max(size),
                ),
                self.colors[3],
                field_clip,
            );
        }
        if let Some((a, b)) = self.editor.composing {
            for [l, _, r, bottom] in layout.selection(byte(a), byte(b)) {
                let underline = rect(x + l, y + bottom - 2.0 * d, r - l, 2.0 * d);
                self.field_box(p, underline, self.colors[3], field_clip);
            }
        }
        if self.editor.focused && self.editor.anchor != self.editor.focus {
            for (start, at) in [(true, self.editor.anchor), (false, self.editor.focus)] {
                let here = caret(&layout, byte(at));
                let other = caret(
                    &layout,
                    byte(if start {
                        self.editor.focus
                    } else {
                        self.editor.anchor
                    }),
                );
                let leading = here.0 <= other.0;
                let diameter = 25.0 * d;
                let left = x + here.0 - if leading { diameter } else { 0.0 };
                let bounds = rect(left, y + here.2, diameter, diameter);
                self.paint_box(p, bounds, self.colors[3], diameter / 2.0);
                let item = p.items.last_mut().unwrap();
                // M3 teardrop: circle with the corner at the selection endpoint squared.
                for style in &mut item.styles {
                    style.corners[if leading { 1 } else { 0 }] = 0.0;
                }
                item.clip = clip;
                self.controls.push(Control {
                    id: if start { 20 } else { 21 },
                    bounds,
                    label: self.label(if start {
                        "settings__selection_start"
                    } else {
                        "settings__selection_end"
                    }),
                    summary: at.to_string(),
                    role: "button",
                    enabled: true,
                    checked: None,
                    selected: false,
                    action: Action::SelectionHandle(start),
                    range: None,
                });
            }
        }
        self.editor_layout = Some(layout);
        self.editor_origin = [x, y];
    }
    fn field_box(&self, p: &mut Panel, b: Rect, color: u32, clip: Rect) {
        self.paint_clipped_box(p, b, color, 0.0, clip);
    }
    pub(super) fn place_editor_caret(&mut self, x: f32, y: f32) {
        if let Some(layout) = &self.editor_layout {
            let index = layout.index_at(x - self.editor_origin[0], y - self.editor_origin[1]);
            let value = String::from_utf16_lossy(&self.editor.text);
            let at = value[..index].encode_utf16().count() as i32;
            self.editor.select(at, at);
        }
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
    pub(super) fn drag_editor_handle(&mut self, x: f32) {
        if let Some(layout) = &self.editor_layout {
            let index = layout.index_at(x - self.editor_origin[0], 0.0);
            let value = String::from_utf16_lossy(&self.editor.text);
            let at = value[..index].encode_utf16().count() as i32;
            self.editor.select(at, self.handle_drag_other as i32);
        }
    }
}
