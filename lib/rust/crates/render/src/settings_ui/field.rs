//! Single-line preview field; Parley supplies caret and selection geometry and hit testing.
use super::{Action, App, Control};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;
use parley::{Cursor, Selection, layout::Affinity};
impl App {
    pub(super) fn paint_editor(&mut self, p: &mut Panel, c: &Control, text: &mut Text, clip: Rect) {
        let d = self.config[2];
        let size = self.sp(16);
        let b = c.bounds;
        let value = String::from_utf16_lossy(&self.editor.text);
        let layout = text.layout_line(&value, size, false, 400, false, self.line_height(size));
        let byte = |at: usize| String::from_utf16_lossy(&self.editor.text[..at]).len();
        let cursor = |at: usize| Cursor::from_byte_index(&layout, byte(at), Affinity::Downstream);
        let caret = cursor(self.editor.focus).geometry(&layout, 2.0 * d);
        let available = (b.width() - 32.0 * d).max(1.0);
        let shift = (caret.x0 as f32 - available + 2.0 * d).max(0.0);
        let x = b.left + 16.0 * d - shift;
        let y = b.top
            + if self.editor.focused && self.editor.anchor != self.editor.focus {
                2.0 * d
            } else {
                (b.height() - layout.height()) / 2.0
            };
        let field_clip = crate::panel_layout::intersection(
            clip,
            rect(b.left + 8.0 * d, b.top, b.width() - 16.0 * d, b.height()),
        );
        let selection = Selection::new(cursor(self.editor.anchor), cursor(self.editor.focus));
        for (r, _) in selection.geometry(&layout) {
            self.field_box(
                p,
                rect(
                    x + r.x0 as f32,
                    y + r.y0 as f32,
                    r.width() as f32,
                    r.height() as f32,
                ),
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
                layout.width().max(available) + 2.0 * d,
                layout.height().max(size * 1.3),
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
                    x + caret.x0 as f32,
                    y + caret.y0 as f32,
                    2.0 * d,
                    (caret.height() as f32).max(size),
                ),
                self.colors[3],
                field_clip,
            );
        }
        if let Some((a, b)) = self.editor.composing {
            for (r, _) in Selection::new(cursor(a), cursor(b)).geometry(&layout) {
                self.field_box(
                    p,
                    rect(
                        x + r.x0 as f32,
                        y + r.y1 as f32 - 2.0 * d,
                        r.width() as f32,
                        2.0 * d,
                    ),
                    self.colors[3],
                    field_clip,
                );
            }
        }
        if self.editor.focused && self.editor.anchor != self.editor.focus {
            for (start, at) in [(true, self.editor.anchor), (false, self.editor.focus)] {
                let caret = cursor(at).geometry(&layout, 1.0);
                let other = cursor(if start {
                    self.editor.focus
                } else {
                    self.editor.anchor
                })
                .geometry(&layout, 1.0);
                let leading = caret.x0 <= other.x0;
                let diameter = 25.0 * d;
                let left = x + caret.x0 as f32 - if leading { diameter } else { 0.0 };
                let bounds = rect(left, y + caret.y1 as f32, diameter, diameter);
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
        self.paint_box(p, b, color, 0.0);
        if let Some(last) = p.items.last_mut() {
            last.clip = clip;
        }
    }
    pub(super) fn place_editor_caret(&mut self, x: f32, y: f32) {
        if let Some(layout) = &self.editor_layout {
            let cursor =
                Cursor::from_point(layout, x - self.editor_origin[0], y - self.editor_origin[1]);
            let value = String::from_utf16_lossy(&self.editor.text);
            let at = value[..cursor.index()].encode_utf16().count() as i32;
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
        Some(
            self.editor_origin[0]
                + Cursor::from_byte_index(layout, byte, Affinity::Downstream)
                    .geometry(layout, 1.0)
                    .x0 as f32,
        )
    }
    pub(super) fn drag_editor_handle(&mut self, x: f32) {
        if let Some(layout) = &self.editor_layout {
            let cursor = Cursor::from_point(layout, x - self.editor_origin[0], 0.0);
            let value = String::from_utf16_lossy(&self.editor.text);
            let at = value[..cursor.index()].encode_utf16().count() as i32;
            self.editor.select(at, self.handle_drag_other as i32);
        }
    }
}
