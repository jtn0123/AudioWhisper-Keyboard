//! The editor's own screen, as ExtensionEditScreen laid it out: a box of Metadata, Dependencies
//! and Files, then ExtensionComponentListView's header with Add, and an ExtensionComponentView
//! per theme with Delete in the error colour at the start and Edit at the end. Before the
//! workspace opens it says it is opening; when it cannot open, why, where Compose showed
//! ExtensionNotFoundScreen or a stack trace. The sheets are `sheets.rs`.
use super::{Load, Tap as EditorTap, Target};
use crate::settings_ui::{
    Action, App,
    cards::plain,
    catalogue::ext_editor::Edit,
    extensions::{Component, THEME, Tap, component_text},
    items::Entry,
    languages::form::Button,
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

pub const STATUS_ID: i32 = 400_000;
/// Metadata, Dependencies and Files: `MANAGE_ID + k`.
pub const MANAGE_ID: i32 = 400_010;
pub const HEADER_ID: i32 = 400_020;
pub const ADD_ID: i32 = 400_021;
pub const NONE_ID: i32 = 400_022;
/// Component `c`'s box: `COMP_ID + 8 * c`, then 1 its name, 2 its text, 3 Delete and 4 Edit.
pub const COMP_ID: i32 = 410_000;

/// The Manage box's rows: their titles and icons.
const MANAGE: [(&str, &str); 3] = [
    ("ext__editor__metadata__title", "code"),
    ("ext__editor__dependencies__title", "library_books"),
    ("ext__editor__files__title", "file_blank"),
];

/// A tap on the editor's control.
fn tap(t: EditorTap) -> Action {
    Action::Ext(Tap::Editor(t))
}

impl App {
    /// Paints the editor's part `e` at row `i`; its height.
    pub(in crate::settings_ui) fn ext_editor_part(
        &mut self,
        p: &mut Panel,
        (i, e): (usize, Edit),
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        match e {
            Edit::Status => self.editor_status(p, at, clip, text),
            Edit::Manage if self.editing() => self.manage_box(p, at, clip, text),
            Edit::Components if self.editing() => self.editor_components(p, at, clip, text),
            Edit::Manage | Edit::Components => 0.0,
            _ => self.sheet_part(p, (i, e), at, clip, text),
        }
    }
    /// That the extension is opening, or why it cannot be edited; nothing once it is open.
    fn editor_status(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        // `ext/edit` alone names no extension.
        let load = match self.editor_target() {
            None => Load::Missing,
            Some(_) => self.extensions.editor.load,
        };
        let key = match load {
            Load::Ready => return 0.0,
            Load::Waiting => "settings__ext_editor__opening",
            Load::Missing => "ext__error__not_found_description",
            Load::Failed => "settings__ext_editor__unpack_failed",
            Load::Gone => "settings__ext_editor__gone",
        };
        let id = match self.editor_target() {
            Some(Target::Edit(id)) => id,
            _ => "null".into(),
        };
        let message = self.label(key).replace("{id}", &id);
        let width = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
        let frame = (x + self.px(16.0), y + self.px(16.0), width);
        self.body(p, (STATUS_ID, &message), frame, clip, text) + 2.0 * self.px(16.0)
    }
    /// The box of Metadata, Dependencies and Files, each a Preference with its icon; its height
    /// with the 8 dp round it.
    fn manage_box(
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
        for (k, (key, icon)) in MANAGE.into_iter().enumerate() {
            let entry = Entry {
                id: MANAGE_ID + k as i32,
                title: self.label(key),
                summary: String::new(),
                icon,
                ink: [None, None],
                buttons: 0,
                right: self.px(16.0),
                enabled: true,
                action: tap(EditorTap::Manage(k)),
            };
            at += self.paint_entry(p, entry, (left, at), clip, text).0;
        }
        self.fit_box(p, outline, at - top);
        at - top + 2.0 * self.px(8.0)
    }
    /// The themes' header with Add at its end, then a box per theme, or that there are none.
    fn editor_components(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let header = (HEADER_ID, "ext__meta__components_theme", self.px(52.0));
        let h = self.components_header(p, header, (x, y), clip, text);
        let b = self.end_button(rect(x, y, self.config[0] - x, h), 1);
        let label = self.label("action__add");
        self.paint_button_icon(p, ADD_ID, "add", b, self.colors[9], clip);
        let mut c = plain(ADD_ID, b, label, "button");
        c.action = tap(EditorTap::AddComponent);
        self.push_control(c, clip);
        let mut at = y + h;
        let themes = self.extensions.editor.draft.themes.clone();
        if themes.is_empty() {
            return at + self.none_found(p, NONE_ID, (x, at), clip, text) - y;
        }
        for (c, comp) in themes.iter().enumerate() {
            at += self.comp_box(p, (c, comp), (x, at), clip, text);
        }
        at - y
    }
    /// ExtensionComponentView: a box titled with the theme's label over its full name, what it
    /// is in bodyMedium 16 dp in, then Delete and Edit 6 dp in; its height with the 8 dp round it.
    fn comp_box(
        &mut self,
        p: &mut Panel,
        (c, comp): (usize, &super::Comp),
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let id = COMP_ID + 8 * c as i32;
        let left = x + self.px(16.0);
        let width = self.config[0] - x - 2.0 * self.px(16.0);
        let border_top = y + self.px(8.0) + self.px(11.0);
        let outline = p.items.len();
        self.outlined_box(p, rect(left, border_top, width, 0.0), clip);
        let name = format!("{}:{}", self.extensions.editor.draft.meta.id, comp.id);
        let named = (id + 1, name.as_str(), Action::None);
        let at = self.box_id(p, named, (left, border_top + self.px(11.0)), clip, text);
        self.as_text(id + 1);
        let shown = Component {
            id: comp.id.clone(),
            label: comp.label.clone(),
            detail: comp.stylesheet(),
            night: comp.night,
            authors: comp.authors.clone(),
        };
        let body = component_text(THEME, &shown);
        let inner = (width - 2.0 * self.px(16.0)).max(1.0);
        let h = self.measure(text, &body, self.sp(14), inner).ceil();
        let b = rect(left + self.px(16.0), at, inner, h);
        self.paint_text(p, &body, b, self.sp(14), self.colors[15], clip);
        self.push_control(plain(id + 2, b, body, "text"), clip);
        let buttons = [
            self.comp_button(
                (id + 3, "action__delete", "delete"),
                18,
                EditorTap::DeleteComponent(c),
            ),
            self.comp_button(
                (id + 4, "action__edit", "edit"),
                3,
                EditorTap::EditComponent(c),
            ),
        ];
        self.button_row(p, buttons.map(Some), (left, b.bottom, width), clip, text);
        let bottom = b.bottom + self.px(48.0);
        self.fit_box(p, outline, bottom - border_top);
        let border = rect(left, border_top, width, 0.0);
        self.box_title(p, (id, &comp.label), Action::None, border, clip, text);
        self.as_text(id);
        bottom + self.px(8.0) - y
    }
    /// A text button in colour `ink`.
    fn comp_button(
        &self,
        (id, key, icon): (i32, &str, &'static str),
        ink: usize,
        t: EditorTap,
    ) -> Button {
        Button {
            id,
            label: self.label(key),
            icon,
            filled: false,
            ink: self.colors[ink],
            action: tap(t),
        }
    }
    /// The focus order of the editor's part `e`.
    pub(in crate::settings_ui) fn ext_editor_focus(&self, (i, e): (usize, Edit)) -> Vec<i32> {
        if !self.editing() {
            return vec![];
        }
        match e {
            Edit::Status | Edit::Note | Edit::Dependencies => vec![],
            Edit::Manage => (0..3).map(|k| MANAGE_ID + k).collect(),
            Edit::Components => {
                let n = self.extensions.editor.draft.themes.len() as i32;
                let boxes = (0..n).flat_map(|c| [COMP_ID + 8 * c + 3, COMP_ID + 8 * c + 4]);
                std::iter::once(ADD_ID).chain(boxes).collect()
            }
            _ => self.sheet_focus((i, e)),
        }
    }
}
