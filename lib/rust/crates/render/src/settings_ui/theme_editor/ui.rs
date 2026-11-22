//! The theme editor's own controls: a title in titleSmall over an outlined dropdown or a 56 dp
//! single-line field (the subtype editor's), Material's 32 dp InputChip, and a message in the
//! error colour. EditRuleDialog's and ComponentMetaEditorDialog's DialogProperty put a title over
//! each control the same way.
use crate::settings_ui::{
    Action, App, Control,
    cards::{LABEL_LARGE, TITLE_SMALL, plain},
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// A dropdown: its control id, title, value, whether it is in error and whether it opens.
#[derive(Clone, Debug)]
pub struct Dropdown {
    pub id: i32,
    pub title: String,
    pub value: String,
    pub error: bool,
    pub enabled: bool,
    pub action: Action,
}
/// A chip: its control id, label, whether it is selected and what a tap does.
#[derive(Clone, Debug)]
pub struct Chip {
    pub id: i32,
    pub label: String,
    pub selected: bool,
    pub action: Action,
}

impl App {
    /// A control's title 16 dp in and 8 dp down; where the control under it starts (`y` for
    /// none).
    pub(super) fn field_title(
        &mut self,
        p: &mut Panel,
        title: &str,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        if title.is_empty() {
            return y;
        }
        let left = x + self.px(16.0);
        let width = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
        let th = self
            .styled(text, title, TITLE_SMALL, Some(width))
            .height
            .ceil();
        let at = rect(left, y + self.px(8.0), width, th);
        self.paint_styled(p, title, at, TITLE_SMALL, self.colors[1], clip);
        at.bottom + self.px(8.0)
    }
    /// The outlined box of a 56 dp control: 2 dp in the primary or error colour when focused
    /// or in error, else 1 dp outline.
    pub(super) fn outlined_field(
        &self,
        p: &mut Panel,
        b: Rect,
        (focused, error): (bool, bool),
        clip: Rect,
    ) {
        let (stroke, edge) = match (focused, error) {
            (true, false) => (self.px(2.0), self.colors[3]),
            (_, true) => (self.px(2.0), self.colors[18]),
            _ => (self.px(1.0), self.colors[7]),
        };
        let radius = self.px(4.0);
        self.paint_clipped_box(p, b, edge, radius, clip);
        let inside = rect(
            b.left + stroke,
            b.top + stroke,
            b.width() - 2.0 * stroke,
            b.height() - 2.0 * stroke,
        );
        self.paint_clipped_box(p, inside, self.colors[0], radius - stroke, clip);
    }
    /// A title over a dropdown showing `d.value` and a drop-down arrow; its height.
    pub(super) fn dropdown(
        &mut self,
        p: &mut Panel,
        d: Dropdown,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        self.dropdown_in(p, (d, 0.0), at, clip, text)
    }
    /// [`dropdown`](Self::dropdown) leaving `trail` free at its end, for what follows it.
    pub(super) fn dropdown_in(
        &mut self,
        p: &mut Panel,
        (d, trail): (Dropdown, f32),
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let top = self.field_title(p, &d.title, (x, y), clip, text);
        let left = x + self.px(16.0);
        let width = (self.config[0] - x - 2.0 * self.px(16.0) - trail).max(1.0);
        let b = rect(left, top, width, self.px(56.0));
        self.outlined_field(p, b, (self.focus == d.id, d.error), clip);
        let size = self.px(24.0);
        let room = (b.width() - self.px(36.0) - size).max(1.0);
        let shown = self.row_text(text, &d.value, self.sp(16), room, 1);
        let vh = self.measure(text, &shown, self.sp(16), room).ceil();
        let at = rect(
            b.left + self.px(16.0),
            b.top + ((b.height() - vh) / 2.0).round(),
            room,
            vh,
        );
        let ink = self.enabled_color(self.colors[1], d.enabled);
        self.paint_text(p, &shown, at, self.sp(16), ink, clip);
        let icon = rect(
            b.right - self.px(12.0) - size,
            b.top + ((b.height() - size) / 2.0).round(),
            size,
            size,
        );
        let ink = self.colors[if d.error { 18 } else { 2 }];
        let ink = self.enabled_color(ink, d.enabled);
        self.paint_icon(p, "arrow_drop_down", icon, ink, clip);
        let mut c = plain(d.id, b, d.title, "button");
        (c.summary, c.enabled, c.action) = (d.value, d.enabled, d.action);
        self.push_control(c, clip);
        b.bottom + self.px(8.0) - y
    }
    /// A title over row `i`'s single-line field showing `hint` while empty, and `error` under
    /// it; its height.
    pub(super) fn line_box(
        &mut self,
        p: &mut Panel,
        field: (usize, String, String, String),
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        self.unit_box(p, field, "", at, clip, text)
    }
    /// [`line_box`](Self::line_box) with `unit` after the text, as Compose put
    /// `dp`, `sp` or `%` beside a size's field.
    pub(super) fn unit_box(
        &mut self,
        p: &mut Panel,
        (i, title, error, hint): (usize, String, String, String),
        unit: &str,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let top = self.field_title(p, &title, (x, y), clip, text);
        let left = x + self.px(16.0);
        let width = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
        let b = rect(left, top, width, self.px(56.0));
        let id = 1000 + i as i32;
        let (_, active) = self.field_editor(id);
        let focused = self.focus == id || (active && self.editor.focused);
        self.outlined_field(p, b, (focused, !error.is_empty()), clip);
        let field = Control {
            id,
            bounds: b,
            label: title,
            summary: error.clone(),
            role: "edit",
            enabled: true,
            checked: None,
            selected: false,
            action: Action::Field(i),
            range: None,
        };
        // The title is above the field, so an empty field shows only a hint it was given.
        let mut drawn = field.clone();
        drawn.label = hint;
        if !unit.is_empty() {
            let uw = self.measure(text, unit, self.sp(16), width).ceil() + 1.0;
            let uh = self.measure(text, unit, self.sp(16), uw).ceil();
            let right = b.right - self.px(16.0);
            let at = rect(
                right - uw,
                b.top + ((b.height() - uh) / 2.0).round(),
                uw,
                uh,
            );
            self.paint_text(p, unit, at, self.sp(16), self.colors[2], clip);
            drawn.bounds = rect(b.left, b.top, (at.left - b.left).max(1.0), b.height());
        }
        self.paint_editor(p, &drawn, text, clip);
        self.push_control(field, clip);
        let mut bottom = b.bottom;
        if !error.is_empty() {
            let eh = self.measure(text, &error, self.sp(14), width).ceil();
            let at = rect(left, b.bottom + self.px(4.0), width, eh);
            self.paint_text(p, &error, at, self.sp(14), self.colors[18], clip);
            bottom = at.bottom;
        }
        bottom + self.px(8.0) - y
    }
    /// How wide chip `label` is: labelLarge in 12 dp padding.
    pub(super) fn chip_width(&self, text: &mut Text, label: &str) -> f32 {
        self.styled(text, label, LABEL_LARGE, None).width.ceil() + 2.0 * self.px(12.0)
    }
    /// An InputChip of width `w` in the 48 dp row at `(x, y)`: filled secondaryContainer when
    /// selected, else a 1 dp outline; a focus ring in the primary colour.
    pub(super) fn chip(
        &mut self,
        p: &mut Panel,
        c: Chip,
        (x, y, w): (f32, f32, f32),
        clip: Rect,
        text: &mut Text,
    ) {
        let b = rect(x, y + self.px(8.0), w, self.px(32.0));
        let radius = self.px(8.0);
        if self.focus == c.id {
            let ring = self.px(2.0);
            let around = rect(
                b.left - ring,
                b.top - ring,
                b.width() + 2.0 * ring,
                b.height() + 2.0 * ring,
            );
            self.paint_clipped_box(p, around, self.colors[3], radius + ring, clip);
        }
        let line = self.px(1.0);
        if c.selected {
            self.paint_clipped_box(p, b, self.colors[4], radius, clip);
        } else {
            self.paint_clipped_box(p, b, self.colors[7], radius, clip);
            let inside = rect(
                b.left + line,
                b.top + line,
                b.width() - 2.0 * line,
                b.height() - 2.0 * line,
            );
            self.paint_clipped_box(p, inside, self.colors[0], radius - line, clip);
        }
        let layout = self.styled(text, &c.label, LABEL_LARGE, None);
        let (lw, lh) = (layout.width.ceil(), layout.height.ceil());
        let at = rect(
            b.left + ((w - lw) / 2.0).round(),
            b.top + ((b.height() - lh) / 2.0).round(),
            lw + 1.0,
            lh,
        );
        let ink = self.colors[if c.selected { 1 } else { 2 }];
        self.paint_styled(p, &c.label, at, LABEL_LARGE, ink, clip);
        let touch = rect(x, y, w, self.px(48.0));
        let mut control = plain(c.id, touch, c.label, "button");
        (control.selected, control.action) = (c.selected, c.action);
        control.checked = Some(c.selected);
        self.push_control(control, clip);
    }
    /// Chips flowing in rows from `(x, y)` 16 dp in, 8 dp apart; where the last row ends.
    pub(super) fn chips(
        &mut self,
        p: &mut Panel,
        chips: Vec<Chip>,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let left = x + self.px(16.0);
        let right = self.config[0] - self.px(16.0);
        let (mut cx, mut cy) = (left, y);
        for c in chips {
            let w = self.chip_width(text, &c.label).min(right - left);
            if cx > left && cx + w > right {
                (cx, cy) = (left, cy + self.px(48.0));
            }
            self.chip(p, c, (cx, cy, w), clip, text);
            cx += w + self.px(8.0);
        }
        cy + self.px(48.0)
    }
    /// `message` in bodyMedium and `ink`, 16 dp in from `x` and as far from the other edge as
    /// `x` is plus 16 dp, so a note in a rule box keeps the box's padding; `pad` above and below;
    /// its height.
    pub(super) fn theme_note(
        &mut self,
        p: &mut Panel,
        (id, message, ink, pad): (i32, String, u32, f32),
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        if message.is_empty() {
            return 0.0;
        }
        let width = (self.config[0] - 2.0 * (x + self.px(16.0))).max(1.0);
        let h = self.measure(text, &message, self.sp(14), width).ceil();
        let at = rect(x + self.px(16.0), y + pad, width, h);
        self.paint_text(p, &message, at, self.sp(14), ink, clip);
        self.push_control(plain(id, at, message, "text"), clip);
        h + 2.0 * pad
    }
}
