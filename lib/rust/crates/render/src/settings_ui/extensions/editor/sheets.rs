//! The editor's sheets, as Compose drew them: ManageMetaDataScreen's fields, each a title over a
//! single-line field with its message under it, the id read-only once installed; the
//! dependencies' note and list; ExtensionEditFilesScreen's two lists, a header with Add and a
//! row per file; a file's name; and Create component's From existing and Empty, the components
//! to copy, and the empty one's fields, as RadioListItems in outlined boxes.
use super::{COMPONENT_ADD, FILE, Tap as EditorTap};
use crate::settings_ui::{
    Action, App, Control, backup_rows::Lead, cards::plain, catalogue::ext_editor::Edit,
    extensions::Tap, items::Entry,
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

pub const NOTE_ID: i32 = 420_000;
pub const DEPENDENCY_ID: i32 = 420_100;
pub const NO_DEPENDENCIES_ID: i32 = 420_099;
/// Folder `f`'s header is `FOLDER_ID + 1000 * f`, its Add 1 after, and file `k` 10 + `k` after.
pub const FOLDER_ID: i32 = 421_000;
/// From existing and Empty.
pub const FROM_ID: i32 = 430_000;
/// The component to copy `k`.
pub const SOURCE_ID: i32 = 431_000;
/// The installed extension's id, which cannot change.
pub const FIXED_ID: i32 = 432_000;
/// The file properties' Delete, in the top bar.
pub const FILE_DELETE_ID: i32 = 432_001;
/// The icons the sheets and the editor's screen draw through a variable (`SettingsResourcesTest`
/// reads this list).
pub const EDITOR_ICONS: [&str; 9] = [
    "code",
    "library_books",
    "file_blank",
    "text_fields",
    "photo",
    "add",
    "delete",
    "edit",
    "info",
];
/// Each folder's rows' icon.
const FILE_ICONS: [&str; 2] = ["text_fields", "photo"];

/// A RadioListItem's control id, label, second line, whether it is selected and its tap.
type Radio = (i32, String, String, bool, EditorTap);

fn tap(t: EditorTap) -> Action {
    Action::Ext(Tap::Editor(t))
}
fn folder_id(f: usize) -> i32 {
    FOLDER_ID + 1000 * f as i32
}

impl App {
    /// Paints the sheets' part `e` at row `i`; its height.
    pub(super) fn sheet_part(
        &mut self,
        p: &mut Panel,
        (i, e): (usize, Edit),
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        if !self.editing() {
            return 0.0;
        }
        let title = self.label(self.rows()[i].label);
        match e {
            Edit::Meta(0) if self.ext_line(e).is_none() => self.fixed_box(p, title, at, clip, text),
            Edit::Meta(_) | Edit::New(_) | Edit::FileName => {
                let field = (i, title, self.ext_error(e), String::new());
                self.line_box(p, field, at, clip, text)
            }
            Edit::Note => {
                let inset = if self.on(COMPONENT_ADD) { 16.0 } else { 8.0 };
                let card = ("info", [self.colors[12], self.colors[1]], inset);
                self.icon_card(p, (NOTE_ID + i as i32, title), card, at, clip, text)
            }
            Edit::Dependencies => self.dependencies(p, at, clip, text),
            Edit::Files(f) => self.folder(p, (f.min(1), title), at, clip, text),
            Edit::From => {
                let empty = self.from_empty();
                let radios = [
                    (
                        FROM_ID,
                        "ext__editor__create_component__from_existing",
                        !empty,
                        true,
                    ),
                    (
                        FROM_ID + 1,
                        "ext__editor__create_component__from_empty",
                        empty,
                        false,
                    ),
                ]
                .map(|(id, key, on, existing)| {
                    (
                        id,
                        self.label(key),
                        String::new(),
                        on,
                        EditorTap::From(existing),
                    )
                });
                self.radio_box(p, &radios, at, clip, text)
            }
            Edit::Sources if !self.from_empty() => {
                let chosen = self
                    .extensions
                    .editor
                    .comp_form
                    .as_ref()
                    .and_then(|f| f.selected);
                let radios: Vec<Radio> = self
                    .sources()
                    .into_iter()
                    .enumerate()
                    .map(|(k, s)| {
                        let name = format!("{}:{}", s.extension, s.comp.id);
                        (
                            SOURCE_ID + k as i32,
                            s.comp.label,
                            name,
                            chosen == Some(k),
                            EditorTap::Source(k),
                        )
                    })
                    .collect();
                self.radio_box(p, &radios, at, clip, text)
            }
            _ => 0.0,
        }
    }
    /// The installed extension's id, as ManageMetaDataScreen showed it disabled: its title over
    /// a field that does not take focus.
    fn fixed_box(
        &mut self,
        p: &mut Panel,
        title: String,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let top = self.field_title(p, &title, (x, y), clip, text);
        let left = x + self.px(16.0);
        let width = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
        let b = rect(left, top, width, self.px(56.0));
        self.outlined_field(p, b, (false, false), clip);
        let value = self.extensions.editor.draft.meta.id.clone();
        let room = (width - 2.0 * self.px(16.0)).max(1.0);
        let shown = self.row_text(text, &value, self.sp(16), room, 1);
        let vh = self.measure(text, &shown, self.sp(16), room).ceil();
        let at = rect(
            b.left + self.px(16.0),
            b.top + ((b.height() - vh) / 2.0).round(),
            room,
            vh,
        );
        self.paint_text(
            p,
            &shown,
            at,
            self.sp(16),
            self.enabled_color(self.colors[1], false),
            clip,
        );
        let mut c = plain(FIXED_ID, b, title, "edit");
        (c.summary, c.enabled) = (value, false);
        self.push_control(c, clip);
        b.bottom + self.px(8.0) - y
    }
    /// What the extension depends on, a line each, or that it depends on nothing.
    fn dependencies(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let width = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
        let list = self.extensions.editor.draft.dependencies.clone();
        let left = x + self.px(16.0);
        if list.is_empty() {
            let none = self.label("settings__ext_editor__no_dependencies");
            return self.body(p, (NO_DEPENDENCIES_ID, &none), (left, y, width), clip, text);
        }
        let mut at = y;
        for (k, d) in list.iter().enumerate() {
            at += self.body(
                p,
                (DEPENDENCY_ID + k as i32, d),
                (left, at, width),
                clip,
                text,
            );
        }
        at - y
    }
    /// A folder's list: its header 56 dp in, after a 24 dp spacer, with Add at the end, then a
    /// Preference per file; its height.
    fn folder(
        &mut self,
        p: &mut Panel,
        (f, title): (usize, String),
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let id = folder_id(f);
        let header = (id, title.as_str(), self.px(52.0));
        let h = self.components_header(p, header, (x + self.px(40.0), y), clip, text);
        let b = self.end_button(rect(x, y, self.config[0] - x, h), 1);
        self.paint_button_icon(p, id + 1, "add", b, self.colors[2], clip);
        let mut c = plain(id + 1, b, self.label("action__add"), "button");
        c.summary = title;
        c.action = tap(EditorTap::AddFile(f));
        self.push_control(c, clip);
        let mut at = y + h;
        let names: Vec<String> = self.folder_files(f);
        for (k, name) in names.into_iter().enumerate() {
            let entry = Entry {
                id: id + 10 + k as i32,
                title: name,
                summary: String::new(),
                icon: FILE_ICONS[f],
                ink: [None, None],
                buttons: 0,
                right: 0.0,
                enabled: true,
                action: tap(EditorTap::OpenFile(f, k)),
            };
            at += self.paint_entry(p, entry, (x, at), clip, text).0;
        }
        at - y
    }
    /// Folder `f`'s files' names, in name order.
    fn folder_files(&self, f: usize) -> Vec<String> {
        let files = self.extensions.editor.files.iter();
        files
            .filter(|(d, _)| *d == f)
            .map(|(_, n)| n.clone())
            .collect()
    }
    /// An outlined box of RadioListItems; its height with the 8 dp round it.
    fn radio_box(
        &mut self,
        p: &mut Panel,
        radios: &[Radio],
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
        for r in radios {
            at += self.radio_item(p, r.clone(), (left, at, width), clip, text);
        }
        self.fit_box(p, outline, at - top);
        at - top + 2.0 * self.px(8.0)
    }
    /// A RadioListItem: the radio button 16 dp in, the label and its second line 56 dp in; its
    /// height.
    fn radio_item(
        &mut self,
        p: &mut Panel,
        (id, label, second, on, t): Radio,
        (left, y, width): (f32, f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let room = (width - self.px(56.0) - self.px(16.0)).max(1.0);
        let lh = self.measure(text, &label, self.sp(16), room).ceil();
        let sh = match second.is_empty() {
            true => 0.0,
            false => self.measure(text, &second, self.sp(14), room).ceil(),
        };
        let least = self.px(if second.is_empty() { 56.0 } else { 72.0 });
        let rh = (lh + sh + 2.0 * self.px(8.0)).max(least);
        let b = rect(left, y, width, rh);
        if self.focus == id {
            self.paint_clipped_box(p, b, self.colors[4], self.px(8.0), clip);
        }
        self.paint_lead(
            p,
            Lead::Radio(on),
            (left + self.px(28.0), y + (rh / 2.0).round()),
            true,
            clip,
        );
        let top = y + ((rh - lh - sh) / 2.0).round();
        let at = rect(left + self.px(56.0), top, room, lh);
        self.paint_text(p, &label, at, self.sp(16), self.colors[1], clip);
        if !second.is_empty() {
            let s = rect(at.left, at.bottom, room, sh);
            self.paint_text(p, &second, s, self.sp(14), self.colors[2], clip);
        }
        let c = Control {
            summary: second,
            selected: on,
            action: tap(t),
            ..plain(id, b, label, "radio")
        };
        self.push_control(c, clip);
        rh
    }
    /// The file properties' Delete, in the top bar where the subtype editor has its own: its
    /// id, icon, label, what it deletes and its tap.
    pub(in crate::settings_ui) fn ext_header(
        &self,
    ) -> Option<(i32, &'static str, &'static str, String, Action)> {
        let f = self.extensions.editor.file_form.as_ref();
        let name = f
            .filter(|_| self.on(FILE) && self.editing())?
            .original
            .clone()?;
        let delete = tap(EditorTap::DeleteFile);
        Some((FILE_DELETE_ID, "delete", "action__delete", name, delete))
    }
    /// The focus order of the sheets' part `e` at row `i`.
    pub(super) fn sheet_focus(&self, (i, e): (usize, Edit)) -> Vec<i32> {
        match e {
            Edit::Meta(0) if self.ext_line(e).is_none() => vec![],
            Edit::FileName if self.ext_header().is_some() => vec![FILE_DELETE_ID, 1000 + i as i32],
            Edit::Meta(_) | Edit::New(_) | Edit::FileName => vec![1000 + i as i32],
            Edit::Files(f) => {
                let n = self.folder_files(f.min(1)).len() as i32;
                let id = folder_id(f.min(1));
                std::iter::once(id + 1)
                    .chain((0..n).map(|k| id + 10 + k))
                    .collect()
            }
            Edit::From => vec![FROM_ID, FROM_ID + 1],
            Edit::Sources if !self.from_empty() => (0..self.sources().len() as i32)
                .map(|k| SOURCE_ID + k)
                .collect(),
            _ => vec![],
        }
    }
}
