//! The property editor's painting: why Apply refused, the search line, the name, the encoder,
//! the value as its encoder edits it, the four sides or corners with the chip copying one to
//! all, and Show keyboard. The screen has the item editor's bottom bar and no preview field, so
//! Show keyboard focuses its hidden in-app editor, as the rules do when `DisplayKbdAfterDialogs`
//! brings the keyboard back: the IME then draws the live preview over the bottom of the
//! screen, and Back hides it again.
use super::{
    Tap,
    boxes::required,
    paint::DELETE_ID,
    property::{CORNERS, PropTap, PropertyForm, SIDES},
    property_values::{self as values, Editing, editing},
    ui::{Chip, Dropdown},
};
use crate::settings_ui::{Action, App, catalogue::theme_editor::Part, items::Entry};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;
use snygg::{
    text,
    value::{Encoder, VARIABLE_NAME_PATTERN},
};

/// Why Apply refused; the chip copying a side to all four; Show keyboard; the no files note;
/// a corner's hint (`HINT_ID + k`).
pub const PROP_ERROR_ID: i32 = 400_200;
pub const ALL_ID: i32 = 400_201;
pub const KEYBOARD_ID: i32 = 400_202;
pub const NO_FILES_ID: i32 = 400_203;
pub const HINT_ID: i32 = 400_210;

impl App {
    /// Paints the property editor's part `part` at row `i`; its height.
    pub(super) fn property_part(
        &mut self,
        p: &mut Panel,
        (i, part): (usize, Part),
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let Some(f) = self.theme_editor.prop.clone() else {
            return 0.0;
        };
        let title = self.label(self.rows()[i].label);
        let sides = f.editing_kind() == Editing::Sides;
        match part {
            Part::PropError => {
                let (error, ink) = (self.message(f.error), self.colors[18]);
                let note = (PROP_ERROR_ID, error, ink, self.px(8.0));
                self.theme_note(p, note, at, clip, text)
            }
            Part::PropSearch if f.searching() => {
                let field = (i, String::new(), String::new(), title);
                self.line_box(p, field, at, clip, text)
            }
            Part::PropName if self.property_line(part).is_some() => {
                let field = (i, title, self.name_error(&f), String::new());
                self.line_box(p, field, at, clip, text)
            }
            Part::PropName | Part::PropEncoder => {
                let d = self.property_dropdown(i, part, &f, title);
                self.dropdown(p, d, at, clip, text)
            }
            Part::PropValue => self.value_part(p, (i, &f), at, clip, text),
            Part::PropSide(k) if sides => self.side_part(p, (i, k, &f), at, clip, text),
            Part::PropAll if sides => {
                let key = match f.encoder {
                    Some(Encoder::Padding) => "settings__theme_editor__padding_apply_for_all",
                    _ => "settings__theme_editor__property_value_shape_apply_for_all_corners",
                };
                let chip = Chip {
                    id: ALL_ID,
                    label: self.label(key),
                    selected: false,
                    action: Action::ThemeEdit(Tap::Prop(PropTap::All)),
                };
                self.chips(p, vec![chip], at, clip, text) - at.1
            }
            Part::PropKeyboard => self.keyboard_entry(p, title, at, clip, text),
            _ => 0.0,
        }
    }
    /// A string key's text, or nothing for none.
    fn message(&self, key: &str) -> String {
        if key.is_empty() {
            String::new()
        } else {
            self.label(key)
        }
    }
    /// A variable's name's message once Apply was tried, its pattern filled in.
    fn name_error(&self, f: &PropertyForm) -> String {
        let key = if f.checked {
            values::variable_check(&f.name)
        } else {
            ""
        };
        let message = self.message(key);
        message.replace("{variable_name_regex}", VARIABLE_NAME_PATTERN)
    }
    /// The name's or the encoder's dropdown.
    fn property_dropdown(&self, i: usize, part: Part, f: &PropertyForm, title: String) -> Dropdown {
        let placeholder = self.label("general__select_dropdown_value_placeholder");
        let (value, error) = match part {
            Part::PropName if f.name.is_empty() => (placeholder, f.checked),
            Part::PropName => (self.property_label(&f.key()), false),
            _ => match f.encoder {
                Some(e) => (self.label(values::encoder_key(e)), false),
                None => (placeholder, f.checked),
            },
        };
        Dropdown {
            id: 1000 + i as i32,
            title,
            value,
            error,
            enabled: self.property_selected(i).is_some(),
            action: Action::ThemeEdit(Tap::Open(i)),
        }
    }
    /// The value as its encoder edits it: a dropdown (with the note when the workspace has no
    /// files), a field with its unit, the colour opening the picker, a shape's outline, or
    /// nothing.
    fn value_part(
        &mut self,
        p: &mut Panel,
        (i, f): (usize, &PropertyForm),
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let Some(e) = f.encoder else {
            return 0.0;
        };
        match editing(e) {
            Editing::Line => {
                let error = self.message(values::field_check(e, 0, &f.texts).0);
                let field = (i, String::new(), error, String::new());
                self.unit_box(p, field, values::unit(e), at, clip, text)
            }
            Editing::Choice => {
                let value = match f.choice.as_str() {
                    "" => self.label("general__select_dropdown_value_placeholder"),
                    choice => self.encoded_label(choice),
                };
                let d = Dropdown {
                    id: 1000 + i as i32,
                    title: String::new(),
                    value,
                    error: f.value().is_none(),
                    enabled: self.property_selected(i).is_some(),
                    action: Action::ThemeEdit(Tap::Open(i)),
                };
                let h = self.dropdown(p, d, at, clip, text);
                h + self.no_files_note(p, e, (at.0, at.1 + h), clip, text)
            }
            Editing::Picker => self.color_value(p, i, at, clip, text),
            Editing::Nothing => self.shape_outline(p, e, at, clip),
            Editing::Sides => 0.0,
        }
    }
    /// The file selector's note when the workspace has no fonts or images; its height.
    fn no_files_note(
        &mut self,
        p: &mut Panel,
        e: Encoder,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        if e != Encoder::Uri || !self.workspace_files().is_empty() {
            return 0.0;
        }
        let action = self.label("ext__editor__files__title");
        let note = self
            .label("settings__theme_editor__file_selector_no_files_text")
            .replace("{action_title}", &action);
        let ink = self.colors[2];
        self.theme_note(p, (NO_FILES_ID, note, ink, 0.0), at, clip, text)
    }
    /// A rectangle's or a circle's outline, 48 dp; nothing for a keyword.
    fn shape_outline(&mut self, p: &mut Panel, e: Encoder, (x, y): (f32, f32), clip: Rect) -> f32 {
        let radius = match e {
            Encoder::Rectangle => 0.0,
            Encoder::Circle => self.px(24.0),
            _ => return 0.0,
        };
        let line = self.px(2.0);
        let b = rect(
            x + self.px(16.0),
            y + self.px(8.0),
            self.px(48.0),
            self.px(48.0),
        );
        self.paint_clipped_box(p, b, self.colors[2], radius, clip);
        let inside = rect(
            b.left + line,
            b.top + line,
            b.width() - 2.0 * line,
            b.height() - 2.0 * line,
        );
        let inner = (radius - line).max(0.0);
        self.paint_clipped_box(p, inside, self.colors[0], inner, clip);
        self.px(64.0)
    }
    /// Side or corner `k`'s field: its name, its unit, its message and its hint.
    fn side_part(
        &mut self,
        p: &mut Panel,
        (i, k, f): (usize, usize, &PropertyForm),
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let Some(e) = f.encoder else {
            return 0.0;
        };
        let names = if e == Encoder::Padding {
            SIDES
        } else {
            CORNERS
        };
        let (error, hint) = values::field_check(e, k, &f.texts);
        let field = (i, self.label(names[k]), self.message(error), String::new());
        let h = self.unit_box(p, field, values::unit(e), at, clip, text);
        let ink = self.colors[2];
        let note = (HINT_ID + k as i32, self.message(hint), ink, 0.0);
        h + self.theme_note(p, note, (at.0, at.1 + h), clip, text)
    }
    /// Show keyboard: a ListItem focusing the screen's hidden editor; its height.
    fn keyboard_entry(
        &mut self,
        p: &mut Panel,
        title: String,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let entry = Entry {
            id: KEYBOARD_ID,
            title,
            summary: String::new(),
            icon: "keyboard_keys",
            ink: [None, None],
            buttons: 0,
            right: 0.0,
            enabled: true,
            action: Action::Editor,
        };
        self.paint_entry(p, entry, at, clip, text).0
    }
    /// The property editor's focus order, row by row: Delete in the top bar first when editing.
    pub(super) fn property_focus(&self, i: usize, part: Part) -> Vec<i32> {
        let Some(f) = self.theme_editor.prop.as_ref() else {
            return vec![];
        };
        let sides = f.editing_kind() == Editing::Sides;
        let picker = part == Part::PropValue && f.editing_kind() == Editing::Picker;
        let field =
            picker || self.property_line(part).is_some() || self.property_selected(i).is_some();
        match part {
            Part::PropError if self.property_header().is_some() => vec![DELETE_ID],
            Part::PropAll if sides => vec![ALL_ID],
            Part::PropKeyboard => vec![KEYBOARD_ID],
            _ if field => vec![1000 + i as i32],
            _ => vec![],
        }
    }
    /// The top bar's Delete when editing a property that may go: a font set's `src` may not.
    pub fn property_header(&self) -> Option<(i32, &'static str, &'static str, String, Action)> {
        let f = self.theme_editor.prop.as_ref()?;
        let name = f.original.as_ref().filter(|n| !required(&f.rule, n))?;
        let delete = Action::ThemeEdit(Tap::Prop(PropTap::Delete));
        let label = self.property_label(&text(name));
        Some((DELETE_ID, "delete", "action__delete", label, delete))
    }
}
