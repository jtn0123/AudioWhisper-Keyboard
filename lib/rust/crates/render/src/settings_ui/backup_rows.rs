//! Back up's and Restore's rows: Material ListItems that lead with a radio button, a checkbox or
//! an icon, the backup's metadata warning, and the bottom bar with the screen's note and its
//! Cancel and Back up or Restore buttons. Compose's screens put those buttons in the same place.
use super::{
    Action, App, Control,
    cards::plain,
    catalogue::{Kind, Part, languages::Lang},
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// The control id of the bottom bar's note.
pub const NOTE_ID: i32 = 30;

/// What a row shows at its start.
#[derive(Clone, Copy)]
pub(super) enum Lead {
    Radio(bool),
    /// On, off, or None when only some of it is on.
    Check(Option<bool>),
    Icon(&'static str),
}

/// A row to paint.
struct Item {
    lead: Lead,
    label: String,
    summary: String,
    role: &'static str,
    /// The clipboard's kinds sit under its checkbox.
    indent: f32,
    /// The label's colour; onSurface when None.
    ink: Option<u32>,
}

impl App {
    /// Paints row `i`, one of these screens' parts, and returns its height.
    pub(super) fn backup_row(
        &mut self,
        p: &mut Panel,
        i: usize,
        part: Part,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let r = self.rows()[i];
        let (label, summary) = self.part_text(part, self.label(r.label));
        let lead = match part {
            Part::Destination(_) | Part::Mode(_) => Lead::Radio(self.part_selected(part)),
            Part::Group(_) | Part::Clipboard => Lead::Check(self.part_checked(part)),
            Part::Choose | Part::About(_) => Lead::Icon(r.icon),
            Part::Warning => return self.warning_row(p, i, at, clip, text),
            Part::Confirm | Part::Cancel | Part::Start => return 0.0,
        };
        let role = match lead {
            Lead::Radio(_) => "radio",
            Lead::Check(_) => "checkbox",
            Lead::Icon(_) if part == Part::Choose => "button",
            Lead::Icon(_) => "text",
        };
        let indent = match part {
            Part::Group(g) if g >= wire::backup::layout::CLIPBOARD_TEXT => self.px(40.0),
            _ => 0.0,
        };
        let item = Item {
            lead,
            label,
            summary,
            role,
            indent,
            ink: None,
        };
        self.paint_item(p, i, item, at, clip, text)
    }
    /// The backup's metadata warning, or its error in the error colour; nothing when it has none.
    fn warning_row(
        &mut self,
        p: &mut Panel,
        i: usize,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let (message, error) = self.backup_warning();
        if message.is_empty() {
            return 0.0;
        }
        let item = Item {
            lead: Lead::Icon(if error { "error_outline" } else { "warning" }),
            label: message,
            summary: String::new(),
            role: "text",
            indent: 0.0,
            ink: Some(if error {
                self.colors[18]
            } else {
                self.colors[2]
            }),
        };
        self.paint_item(p, i, item, at, clip, text)
    }
    fn paint_item(
        &mut self,
        p: &mut Panel,
        i: usize,
        item: Item,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let w = self.config[0];
        let enabled = self.enabled(i);
        let left = x + item.indent + self.px(56.0);
        let width = (w - item.indent - self.px(72.0)).max(1.0);
        let lh = self.measure(text, &item.label, self.sp(16), width);
        let sh = self.measure(text, &item.summary, self.sp(14), width);
        let three = self.multiline(sh);
        let pad = self.px(if three { 12.0 } else { 8.0 });
        let least = match (item.summary.is_empty(), three) {
            (true, _) => 56.0,
            (false, false) => 72.0,
            (false, true) => 88.0,
        };
        let rh = (lh + sh + 2.0 * pad).max(self.px(least));
        let b = rect(x, y, w, rh);
        let id = 1000 + i as i32;
        if id == self.focus {
            self.paint_clipped_box(p, b, self.colors[4], self.px(8.0), clip);
        }
        let top = y + ((rh - lh - sh) / 2.0).round();
        let glyph = (x + item.indent + self.px(28.0), y + (rh / 2.0).round());
        self.paint_lead(p, item.lead, glyph, enabled, clip);
        let ink = self.enabled_color(item.ink.unwrap_or(self.colors[1]), enabled);
        self.paint_text(
            p,
            &item.label,
            rect(left, top, width, lh),
            self.sp(16),
            ink,
            clip,
        );
        if sh > 0.0 {
            let faded = self.enabled_color(self.colors[2], enabled);
            let at = rect(left, top + lh, width, sh);
            self.paint_text(p, &item.summary, at, self.sp(14), faded, clip);
        }
        let (checked, selected) = match item.lead {
            Lead::Check(state) => (state, false),
            Lead::Radio(on) => (None, on),
            Lead::Icon(_) => (None, false),
        };
        let action = if item.role == "text" {
            Action::None
        } else {
            Action::Row(i)
        };
        let control = Control {
            id,
            bounds: b,
            label: item.label,
            summary: item.summary,
            role: item.role,
            enabled,
            checked,
            selected,
            action,
            range: None,
        };
        self.push_control(control, clip);
        rh
    }
    /// A radio button, checkbox or icon centred on `(cx, cy)`.
    pub(super) fn paint_lead(
        &self,
        p: &mut Panel,
        lead: Lead,
        (cx, cy): (f32, f32),
        on: bool,
        clip: Rect,
    ) {
        let d = self.config[2];
        let circle = |size: f32| rect(cx - size * d / 2.0, cy - size * d / 2.0, size * d, size * d);
        match lead {
            Lead::Radio(selected) => {
                let ink = self.colors[if selected { 3 } else { 2 }];
                let ink = self.enabled_color(ink, on);
                self.paint_clipped_box(p, circle(20.0), ink, 10.0 * d, clip);
                self.paint_clipped_box(p, circle(16.0), self.colors[0], 8.0 * d, clip);
                if selected {
                    self.paint_clipped_box(p, circle(10.0), ink, 5.0 * d, clip);
                }
            }
            Lead::Check(state) => {
                let filled = state != Some(false);
                let ink = self.enabled_color(self.colors[if filled { 3 } else { 2 }], on);
                self.paint_clipped_box(p, circle(18.0), ink, 2.0 * d, clip);
                match state {
                    Some(true) => self.paint_icon(p, "done", circle(16.0), self.colors[6], clip),
                    Some(false) => {
                        self.paint_clipped_box(p, circle(14.0), self.colors[0], 0.0, clip)
                    }
                    None => {
                        let dash = rect(cx - 5.0 * d, cy - d, 10.0 * d, 2.0 * d);
                        self.paint_clipped_box(p, dash, self.colors[6], 0.0, clip);
                    }
                }
            }
            Lead::Icon(name) => {
                let ink = self.enabled_color(self.colors[2], on);
                self.paint_icon(p, name, circle(24.0), ink, clip);
            }
        }
    }
    /// Whether the screen on top keeps its buttons in a bottom bar: Back up, Restore and the
    /// subtype editor. A bar whose buttons are hidden, as the extension editor's are until it
    /// opens, is not drawn; every screen shows its Cancel exactly when its Start or Save.
    fn backup_screen(&self) -> bool {
        let mut rows = self.rows().iter().enumerate();
        rows.any(|(i, r)| {
            matches!(r.kind, Kind::Backup(Part::Start) | Kind::Lang(Lang::Save)) && self.visible(i)
        })
    }
    /// The bottom bar's height: the note, then a row of 40 dp buttons with 16 dp around them.
    /// Zero on other screens.
    pub(super) fn backup_footer(&self, text: &mut Text) -> f32 {
        if !self.backup_screen() {
            return 0.0;
        }
        let w = self.config[0] - self.px(32.0);
        let note = self.backup_note().map_or(0.0, |n| {
            self.measure(text, &n.text, self.sp(14), w).ceil() + self.px(12.0)
        });
        note + self.px(72.0)
    }
    pub(super) fn paint_backup_footer(&mut self, p: &mut Panel, footer: f32, text: &mut Text) {
        if footer <= 0.0 {
            return;
        }
        let [w, h, ..] = self.config;
        let area = rect(0.0, h - footer, w, footer);
        self.paint_box(p, area, self.colors[0], 0.0);
        self.paint_box(
            p,
            rect(0.0, area.top, w, self.px(1.0)),
            self.colors[10],
            0.0,
        );
        if let Some(note) = self.backup_note().cloned() {
            let width = w - self.px(32.0);
            let nh = self.measure(text, &note.text, self.sp(14), width).ceil();
            let b = rect(self.px(16.0), area.top + self.px(12.0), width, nh);
            let ink = self.colors[if note.error { 18 } else { 2 }];
            self.paint_text(p, &note.text, b, self.sp(14), ink, area);
            self.push_control(plain(NOTE_ID, b, note.text, "status"), area);
        }
        let mut right = w - self.px(16.0);
        let top = h - self.px(56.0);
        for part in [Part::Start, Part::Cancel] {
            // The subtype editor's Save and Cancel stand where Back up's Start and Cancel do.
            let Some(i) = self.rows().iter().position(|r| match r.kind {
                Kind::Backup(q) => q == part,
                Kind::Lang(l) => {
                    l == if part == Part::Start {
                        Lang::Save
                    } else {
                        Lang::Cancel
                    }
                }
                _ => false,
            }) else {
                continue;
            };
            let label = self.label(self.rows()[i].label);
            let padding = self.px(if part == Part::Start { 24.0 } else { 12.0 });
            let width =
                (self.button_label(text, &label).width.ceil() + 2.0 * padding).max(self.px(58.0));
            let b = rect(right - width, top, width, self.px(40.0));
            right = b.left - self.px(8.0);
            self.paint_footer_button(p, i, part == Part::Start, b, text, area);
            let mut c = plain(1000 + i as i32, b, label, "button");
            c.enabled = self.enabled(i);
            c.action = Action::Row(i);
            self.push_control(c, area);
        }
    }
    /// Material's filled Button for Start, its TextButton for Cancel.
    fn paint_footer_button(
        &self,
        p: &mut Panel,
        i: usize,
        filled: bool,
        b: Rect,
        text: &mut Text,
        clip: Rect,
    ) {
        let on = self.enabled(i);
        let radius = b.height() / 2.0;
        if 1000 + i as i32 == self.focus {
            let ring = self.px(3.0);
            let around = rect(
                b.left - ring,
                b.top - ring,
                b.width() + 2.0 * ring,
                b.height() + 2.0 * ring,
            );
            self.paint_clipped_box(p, around, self.colors[4], radius + ring, clip);
        }
        let label = self.label(self.rows()[i].label);
        let layout = self.button_label(text, &label);
        let (lw, lh) = (layout.width.ceil(), layout.height.ceil());
        let ink = if filled && on {
            self.paint_clipped_box(p, b, self.colors[3], radius, clip);
            self.colors[6]
        } else if filled {
            let faint = (self.colors[1] & 0x00ff_ffff) | 0x1f00_0000;
            self.paint_clipped_box(p, b, faint, radius, clip);
            self.enabled_color(self.colors[1], false)
        } else {
            self.enabled_color(self.colors[3], on)
        };
        let at = rect(
            b.left + ((b.width() - lw) / 2.0).round(),
            b.top + ((b.height() - lh) / 2.0).round(),
            lw + 1.0,
            lh,
        );
        self.paint_styled(p, &label, at, super::cards::LABEL_LARGE, ink, clip);
    }
}
