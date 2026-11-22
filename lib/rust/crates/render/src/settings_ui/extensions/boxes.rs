//! The home's and the lists' painting, as AddonBox.kt and ExtensionListScreen laid them out:
//! FlorisOutlinedBoxes 16 dp in and 8 dp apart. ImportExtensionBox and UpdateBox are a bodySmall
//! note in 16 dp by 8 dp padding over a row of TextButtons 6 dp in. An extension's box is titled
//! with its name over its id, then its description and View details, and Edit at the end for an
//! extension the extension manager can delete. The theme list's Create extension, a floating
//! button in Compose, is an outlined box above the extensions, as the theme manager has it.
use super::{Extension, Tap};
use crate::settings_ui::{
    Action, App,
    cards::{BODY_SMALL, plain},
    catalogue::extensions::Part,
    items::Entry,
    languages::form::Button,
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// Visit Addons Store, Import, Search for Updates and Create extension.
pub const STORE_ID: i32 = 200_000;
pub const IMPORT_ID: i32 = 200_001;
pub const UPDATE_ID: i32 = 200_002;
pub const CREATE_ID: i32 = 200_003;
/// The two boxes' notes.
pub const STORE_NOTE_ID: i32 = 200_004;
pub const UPDATE_NOTE_ID: i32 = 200_005;
/// Extension `e`'s controls start at `BOX_ID + 8 * e`.
pub const BOX_ID: i32 = 210_000;

/// Extension `e`'s control `part`: 0 the title, 1 the id, 2 the description, 3 View details,
/// 4 Edit.
pub fn ext_id(e: usize, part: usize) -> i32 {
    BOX_ID + 8 * e as i32 + part.min(7) as i32
}

impl App {
    /// Whether the screen on top shows UpdateBox: Check for Updates, and wherever ImportExtensionBox
    /// shows.
    pub fn shows_updates(&self) -> bool {
        self.stack.last().is_some_and(|r| r == super::UPDATES) || self.shows_store()
    }
    /// Paints part `part`; its height.
    pub(in crate::settings_ui) fn ext_part(
        &mut self,
        p: &mut Panel,
        part: Part,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        match part {
            Part::Store if self.shows_store() => {
                let visit =
                    self.text_button(STORE_ID, "ext__home__visit_store", "shop", Tap::Store);
                let import = self.text_button(IMPORT_ID, "action__import", "input", Tap::Import);
                let note = (STORE_NOTE_ID, "ext__home__info");
                self.note_box(p, note, [Some(visit), Some(import)], at, clip, text)
            }
            Part::Updates if self.shows_updates() => {
                let key = "ext__update_box__search_for_updates";
                let update = self.text_button(UPDATE_ID, key, "file_download", Tap::Update);
                let note = (UPDATE_NOTE_ID, "ext__update_box__manual_updates_hint");
                self.note_box(p, note, [Some(update), None], at, clip, text)
            }
            Part::Create => self.create_box(p, at, clip, text),
            Part::Extensions => {
                let listed: Vec<Extension> =
                    self.listed_extensions().into_iter().cloned().collect();
                let mut y = at.1;
                for (e, ext) in listed.iter().enumerate() {
                    y += self.ext_box(p, (e, ext), (at.0, y), clip, text);
                }
                y - at.1
            }
            Part::About => self.page_about(p, at, clip, text),
            Part::Components => self.page_components(p, at, clip, text),
            Part::Store | Part::Updates => 0.0,
        }
    }
    /// A FlorisTextButton in the primary colour.
    fn text_button(&self, id: i32, key: &str, icon: &'static str, tap: Tap) -> Button {
        Button {
            id,
            label: self.label(key),
            icon,
            filled: false,
            ink: self.colors[3],
            action: Action::Ext(tap),
        }
    }
    /// An untitled box round a note and a row of up to two text buttons, one at each end; its
    /// height with the 8 dp round it.
    fn note_box(
        &mut self,
        p: &mut Panel,
        (id, key): (i32, &str),
        buttons: [Option<Button>; 2],
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let left = x + self.px(16.0);
        let width = self.config[0] - x - 2.0 * self.px(16.0);
        let top = y + self.px(8.0);
        let outline = p.items.len();
        self.outlined_box(p, rect(left, top, width, 0.0), clip);
        let inner = (width - 2.0 * self.px(16.0)).max(1.0);
        let note = self.label(key);
        let nh = self
            .styled(text, &note, BODY_SMALL, Some(inner))
            .height
            .ceil();
        let at = rect(left + self.px(16.0), top + self.px(8.0), inner, nh);
        self.paint_styled(p, &note, at, BODY_SMALL, self.colors[15], clip);
        self.push_control(plain(id, at, note, "text"), clip);
        let row = at.bottom + self.px(4.0);
        self.button_row(p, buttons, (left, row, width), clip, text);
        let h = row + self.px(48.0) - top;
        self.fit_box(p, outline, h);
        h + 2.0 * self.px(8.0)
    }
    /// A row of text buttons 6 dp in from each end of `width`, the first at the start.
    fn button_row(
        &mut self,
        p: &mut Panel,
        [start, end]: [Option<Button>; 2],
        (left, row, width): (f32, f32, f32),
        clip: Rect,
        text: &mut Text,
    ) {
        if let Some(b) = start {
            let w = self.button_width(text, &b);
            let touch = rect(left + self.px(6.0), row, w, self.px(48.0));
            self.material_button(p, b, touch, clip, text);
        }
        if let Some(b) = end {
            let w = self.button_width(text, &b);
            let touch = rect(left + width - self.px(6.0) - w, row, w, self.px(48.0));
            self.material_button(p, b, touch, clip, text);
        }
    }
    /// The theme list's box round Create extension; its height with the 8 dp round it.
    fn create_box(
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
        let entry = Entry {
            id: CREATE_ID,
            title: self.label("ext__editor__title_create_any"),
            summary: String::new(),
            icon: "add",
            ink: [None, None],
            buttons: 0,
            right: self.px(16.0),
            enabled: true,
            action: Action::Ext(Tap::Create),
        };
        let h = self.paint_entry(p, entry, (left, top), clip, text).0;
        self.fit_box(p, outline, h);
        h + 2.0 * self.px(8.0)
    }
    /// Extension `ext`'s box: the 11 dp title gap above the border and inside it, the id, the
    /// description in bodySmall 16 dp by 4 dp in, and the buttons; its height with the 8 dp
    /// round it. Its title and id open its page, as the theme manager's do.
    fn ext_box(
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
        let view = Action::Ext(Tap::View(ext.id.clone()));
        let id = (ext_id(e, 1), ext.id.as_str(), view.clone());
        let mut at = self.box_id(p, id, (left, border_top + self.px(11.0)), clip, text);
        if let Some(d) = ext.description.as_ref().filter(|d| !d.is_empty()) {
            let inner = (width - 2.0 * self.px(16.0)).max(1.0);
            let h = self.styled(text, d, BODY_SMALL, Some(inner)).height.ceil();
            let b = rect(left + self.px(16.0), at + self.px(4.0), inner, h);
            self.paint_styled(p, d, b, BODY_SMALL, self.colors[15], clip);
            self.push_control(plain(ext_id(e, 2), b, d.clone(), "text"), clip);
            at = b.bottom + self.px(4.0);
        }
        let details = Tap::View(ext.id.clone());
        let details = self.text_button(ext_id(e, 3), "ext__list__view_details", "info", details);
        let edit = Tap::Edit(ext.id.clone());
        let edit = ext
            .deletable
            .then(|| self.text_button(ext_id(e, 4), "action__edit", "edit", edit));
        self.button_row(p, [Some(details), edit], (left, at, width), clip, text);
        at += self.px(48.0);
        self.fit_box(p, outline, at - border_top);
        let border = rect(left, border_top, width, 0.0);
        self.box_title(p, (ext_id(e, 0), &ext.title), view, border, clip, text);
        at + self.px(8.0) - y
    }
    /// The focus order of part `part`.
    pub(in crate::settings_ui) fn ext_focus(&self, part: Part) -> Vec<i32> {
        match part {
            Part::Store if self.shows_store() => vec![STORE_ID, IMPORT_ID],
            Part::Updates if self.shows_updates() => vec![UPDATE_ID],
            Part::Create => vec![CREATE_ID],
            Part::Extensions => {
                let mut out = vec![];
                for (e, ext) in self.listed_extensions().iter().enumerate() {
                    out.extend([ext_id(e, 0), ext_id(e, 1), ext_id(e, 3)]);
                    if ext.deletable {
                        out.push(ext_id(e, 4));
                    }
                }
                out
            }
            Part::About | Part::Components => self.page_focus(part),
            Part::Store | Part::Updates => vec![],
        }
    }
}
