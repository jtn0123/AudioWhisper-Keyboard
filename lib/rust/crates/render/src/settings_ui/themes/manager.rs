//! The theme manager's painting, as ThemeManagerScreen laid it out: one FlorisOutlinedBox per
//! theme extension (the language pack manager's box), titled with its name over its id, listing
//! its themes as JetPrefListItems: a radio button, the label, and an 18 dp day or night icon in
//! onSurfaceVariant at 56%. Under them the extension list's View details, and Edit for an
//! extension made or imported here. Above the boxes, an outlined box with Create and Import.
use super::{Extension, Tap, Theme};
use crate::settings_ui::{
    Action, App, backup_rows::Lead, cards::plain, catalogue::themes::Part, items::Entry,
    languages::form::Button,
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// The Create and Import rows' control ids.
pub const CREATE_ID: i32 = 90000;
pub const IMPORT_ID: i32 = 90001;
/// Extension `e`'s controls start at `BOX_ID + 256 * e`.
pub const BOX_ID: i32 = 100000;

/// Extension `e`'s control `part`: 0 the title, 1 the id, 2 View details, 3 Edit, 4 onwards its
/// themes.
pub fn theme_id(e: usize, part: usize) -> i32 {
    BOX_ID + 256 * e as i32 + part.min(255) as i32
}

impl App {
    /// Paints part `part` of the manager; its height.
    pub(in crate::settings_ui) fn themes_part(
        &mut self,
        p: &mut Panel,
        part: Part,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        match part {
            Part::Actions => self.theme_actions(p, at, clip, text),
            Part::Extensions => {
                let extensions = self.theme_extensions().to_vec();
                let mut y = at.1;
                for (e, ext) in extensions.iter().enumerate() {
                    y += self.theme_box(p, (e, ext), (at.0, y), clip, text);
                }
                y - at.1
            }
        }
    }
    /// The box round Create and Import: 16 dp in, 8 dp down; its height with that padding.
    fn theme_actions(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let left = x + self.px(16.0);
        let width = self.config[0] - x - 2.0 * self.px(16.0);
        let top = y + self.px(8.0);
        let outline = p.items.len();
        self.outlined_box(p, rect(left, top, width, 0.0), clip);
        let mut at = top;
        let rows = [
            (
                CREATE_ID,
                "ext__editor__title_create_theme",
                "add",
                Tap::Create,
            ),
            (IMPORT_ID, "action__import", "input", Tap::Import),
        ];
        for (id, label, icon, tap) in rows {
            let entry = Entry {
                id,
                title: self.label(label),
                summary: String::new(),
                icon,
                ink: [None, None],
                buttons: 0,
                right: self.px(16.0),
                enabled: true,
                action: Action::Themes(tap),
            };
            at += self.paint_entry(p, entry, (left, at), clip, text).0;
        }
        self.fit_box(p, outline, at - top);
        at - top + 2.0 * self.px(8.0)
    }
    /// Extension `ext`'s box: the 11 dp title gap above the border and inside it, the id, the
    /// themes and the buttons; its height with the 8 dp round it.
    fn theme_box(
        &mut self,
        p: &mut Panel,
        (e, ext): (usize, &Extension),
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let left = x + self.px(16.0);
        let width = self.config[0] - x - 2.0 * self.px(16.0);
        let border_top = y + self.px(8.0) + self.px(11.0);
        let outline = p.items.len();
        self.outlined_box(p, rect(left, border_top, width, 0.0), clip);
        let view = Action::Themes(Tap::View(ext.id.clone()));
        let id = (theme_id(e, 1), ext.id.as_str(), view.clone());
        let mut at = self.box_id(p, id, (left, border_top + self.px(11.0)), clip, text);
        for (t, theme) in ext.themes.iter().enumerate() {
            at += self.theme_row(p, (e, t, theme), (left, at, width), clip, text);
        }
        at = self.theme_buttons(p, (e, ext), (left, at, width), clip, text);
        self.fit_box(p, outline, at - border_top);
        let border = rect(left, border_top, width, 0.0);
        self.box_title(p, (theme_id(e, 0), &ext.title), view, border, clip, text);
        at + self.px(8.0) - y
    }
    /// A theme's ListItem: 56 dp, its radio button centred in the 56 dp lead, the label in 16 sp
    /// and the day or night icon 16 dp from the end. TalkBack hears which it is, and whether it
    /// is chosen.
    fn theme_row(
        &mut self,
        p: &mut Panel,
        (e, t, theme): (usize, usize, &Theme),
        (left, y, width): (f32, f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let chosen = self.theme_chosen(e, t);
        let id = theme_id(e, 4 + t);
        let icon = self.px(18.0);
        let room = (width - self.px(56.0) - 2.0 * self.px(16.0) - icon).max(1.0);
        let label = self.row_text(text, &theme.label, self.sp(16), room, 1);
        let lh = self.measure(text, &label, self.sp(16), room);
        let rh = (lh + 2.0 * self.px(8.0)).max(self.px(56.0));
        let b = rect(left, y, width, rh);
        if id == self.focus {
            self.paint_clipped_box(p, b, self.colors[4], self.px(8.0), clip);
        }
        let centre = (left + self.px(28.0), y + (rh / 2.0).round());
        self.paint_lead(p, Lead::Radio(chosen), centre, true, clip);
        let at = rect(
            left + self.px(56.0),
            y + ((rh - lh) / 2.0).round(),
            room,
            lh,
        );
        self.paint_text(p, &label, at, self.sp(16), self.colors[1], clip);
        let faded = (self.colors[2] & 0x00ff_ffff) | 0x8f00_0000;
        let glyph = rect(
            left + width - self.px(16.0) - icon,
            y + ((rh - icon) / 2.0).round(),
            icon,
            icon,
        );
        let (name, kind) = match theme.night {
            true => ("dark_mode", "pref__theme__night"),
            false => ("light_mode", "pref__theme__day"),
        };
        self.paint_icon(p, name, glyph, faded, clip);
        let mut c = plain(id, b, theme.label.clone(), "radio");
        c.summary = self.label(kind);
        c.selected = chosen;
        c.action = Action::Themes(Tap::Pick(e, t));
        self.push_control(c, clip);
        rh
    }
    /// The extension list's row of text buttons 6 dp in: View details, and Edit at the end for
    /// an extension made or imported here; where the box's content ends.
    fn theme_buttons(
        &mut self,
        p: &mut Panel,
        (e, ext): (usize, &Extension),
        (left, at, width): (f32, f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let view = Button {
            id: theme_id(e, 2),
            label: self.label("ext__list__view_details"),
            icon: "info",
            filled: false,
            ink: self.colors[3],
            action: Action::Themes(Tap::View(ext.id.clone())),
        };
        let vw = self.button_width(text, &view);
        let touch = rect(left + self.px(6.0), at, vw, self.px(48.0));
        self.material_button(p, view, touch, clip, text);
        if ext.deletable {
            let edit = Button {
                id: theme_id(e, 3),
                label: self.label("action__edit"),
                icon: "edit",
                filled: false,
                ink: self.colors[3],
                action: Action::Themes(Tap::Edit(ext.id.clone())),
            };
            let ew = self.button_width(text, &edit);
            let touch = rect(left + width - self.px(6.0) - ew, at, ew, self.px(48.0));
            self.material_button(p, edit, touch, clip, text);
        }
        at + self.px(48.0)
    }
    /// The focus order: Create, Import, then each box's title, id, themes, View details and Edit.
    pub(in crate::settings_ui) fn themes_focus(&self, part: Part) -> Vec<i32> {
        if part == Part::Actions {
            return vec![CREATE_ID, IMPORT_ID];
        }
        let mut out = vec![];
        for (e, ext) in self.theme_extensions().iter().enumerate() {
            out.extend([theme_id(e, 0), theme_id(e, 1)]);
            out.extend((0..ext.themes.len()).map(|t| theme_id(e, 4 + t)));
            out.push(theme_id(e, 2));
            if ext.deletable {
                out.push(theme_id(e, 3));
            }
        }
        out
    }
}
