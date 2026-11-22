//! The component meta editor, ComponentMetaEditorDialog as a screen: the id, the label, the
//! authors, the night theme switch, the stylesheet path, and Material You's palette style,
//! contrast level and spec version as dropdowns of their enums' names, as Compose's were. The
//! authors are one comma-separated line where Compose took one per line, as the settings
//! renderer's fields are single-line. Apply checks the id, the label, the authors and the path
//! with `ExtensionValidation`'s rules (the `extensions` crate), and the id against the
//! extension's other components, and shows why under each field; Compose said a repeated id in
//! a toast.
use super::{META, Meta, Tap, ui::Dropdown};
use crate::settings_ui::{Action, App, cards::plain, catalogue::theme_editor::Part};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use extensions::validation::validate;
use keys::Rect;
use wire::extensions::{
    COMPONENT_AUTHORS, COMPONENT_ID, COMPONENT_LABEL, INVALID, STYLESHEET_PATH,
    validation::{ARGUMENT, PATTERN, RESOURCE, STATUS},
};

/// The night switch's control id.
pub const NIGHT_ID: i32 = 400_050;
/// Why Apply refused an id another component of the extension has.
pub const ID_TAKEN: &str = "settings__theme_editor__component_id_exists";

/// The meta editor's state: the fields as typed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MetaForm {
    pub id: String,
    pub label: String,
    pub authors: String,
    pub night: bool,
    pub path: String,
    /// The Material You palette style, contrast level and spec version chosen.
    pub flags: [String; 3],
    /// Apply was tried: the fields' messages show.
    pub checked: bool,
}

impl MetaForm {
    pub fn of(meta: &Meta) -> Self {
        Self {
            id: meta.id.clone(),
            label: meta.label.clone(),
            authors: meta.authors.join(", "),
            night: meta.night,
            path: meta.path.clone(),
            flags: meta.flags.clone(),
            checked: false,
        }
    }
    /// The authors as the component keeps them: trimmed, the blank ones dropped.
    pub fn authors(&self) -> Vec<String> {
        let all = self.authors.split(',').map(str::trim);
        all.filter(|a| !a.is_empty()).map(Into::into).collect()
    }
    /// Field `k`'s text: 0 the label, 1 the authors, 2 the id, 3 the stylesheet path.
    pub fn field(&self, k: usize) -> &str {
        match k {
            0 => &self.label,
            1 => &self.authors,
            2 => &self.id,
            _ => &self.path,
        }
    }
    fn field_mut(&mut self, k: usize) -> &mut String {
        match k {
            0 => &mut self.label,
            1 => &mut self.authors,
            2 => &mut self.id,
            _ => &mut self.path,
        }
    }
    /// Field `k` as the extension editor checks it: the authors one per line, as Compose took
    /// them.
    pub fn checked_text(&self, k: usize) -> String {
        if k == 1 {
            self.authors().join("\n")
        } else {
            self.field(k).to_owned()
        }
    }
    /// The meta Apply gives: the id, the label and the path trimmed, as Compose kept them.
    pub fn meta(&self) -> Meta {
        Meta {
            id: self.id.trim().into(),
            label: self.label.trim().into(),
            authors: self.authors(),
            night: self.night,
            path: self.path.trim().into(),
            flags: self.flags.clone(),
        }
    }
}

/// Field `k`'s check: its message's string key (empty for none), whether it refuses Apply, and
/// the message's `{name}` with the pattern it stands for.
pub fn check(k: usize, value: &str) -> (String, bool, [String; 2]) {
    let rule = [
        COMPONENT_LABEL,
        COMPONENT_AUTHORS,
        COMPONENT_ID,
        STYLESHEET_PATH,
    ][k.min(3)];
    let row = validate(rule, &value.encode_utf16().collect::<Vec<_>>());
    let field = |f: usize| String::from_utf16_lossy(&row[f]);
    let refused = field(STATUS) == INVALID;
    (field(RESOURCE), refused, [field(ARGUMENT), field(PATTERN)])
}

impl App {
    /// The component box's Edit: the meta editor showing the component's meta.
    pub(super) fn open_meta(&mut self) {
        self.theme_editor.meta_form = Some(MetaForm::of(&self.theme_editor.meta));
        self.theme_dialog_opened();
        self.navigate(META);
    }
    /// Field `k` of `f` as Apply checks it: an id that passes must also not be another
    /// component's, as Compose's Apply asked.
    fn meta_check(&self, f: &MetaForm, k: usize) -> (String, bool, [String; 2]) {
        let checked = check(k, &f.checked_text(k));
        let others = &self.theme_editor.component.other_ids;
        if k == 2 && !checked.1 && others.iter().any(|o| o == f.id.trim()) {
            return (ID_TAKEN.into(), true, Default::default());
        }
        checked
    }
    /// Field `k`'s message once Apply was tried, translated, its pattern filled in as
    /// `rememberValidationResult` did.
    pub fn meta_error(&self, k: usize) -> String {
        let Some(f) = self.theme_editor.meta_form.as_ref().filter(|f| f.checked) else {
            return String::new();
        };
        let (key, _, [name, pattern]) = self.meta_check(f, k);
        if key.is_empty() || name.is_empty() {
            return if key.is_empty() {
                key
            } else {
                self.label(&key)
            };
        }
        self.label(&key).replace(&format!("{{{name}}}"), &pattern)
    }
    /// Field `k` changed.
    pub(in crate::settings_ui) fn meta_typed(&mut self, k: usize, value: String) {
        if let Some(f) = self.theme_editor.meta_form.as_mut() {
            *f.field_mut(k) = value;
        }
    }
    /// Apply: the meta changes unless a field refuses, and the editor closes.
    pub(super) fn save_meta(&mut self) {
        let Some(f) = self.theme_editor.meta_form.as_ref() else {
            return;
        };
        let (refused, meta) = ((0..4).any(|k| self.meta_check(f, k).1), f.meta());
        if refused {
            if let Some(f) = self.theme_editor.meta_form.as_mut() {
                f.checked = true;
            }
            return;
        }
        self.theme_editor.meta = meta;
        self.theme_changed();
        self.close_editor();
    }
    /// Paints the meta editor's part `part` at row `i`; its height.
    pub(super) fn meta_part(
        &mut self,
        p: &mut Panel,
        (i, part): (usize, Part),
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let title = self.label(self.rows()[i].label);
        match part {
            Part::MetaField(k) => self.line_box(p, (i, title, self.meta_error(k)), at, clip, text),
            Part::MaterialYou => self.field_title(p, &title, at, clip, text) - at.1,
            Part::MetaChoice(k) => {
                let form = self.theme_editor.meta_form.as_ref();
                let d = Dropdown {
                    id: 1000 + i as i32,
                    title,
                    value: form.map_or(String::new(), |f| f.flags[k].clone()),
                    error: false,
                    enabled: true,
                    action: Action::ThemeEdit(Tap::Open(i)),
                };
                self.dropdown(p, d, at, clip, text)
            }
            _ => self.night_row(p, i, at, clip, text),
        }
    }
    /// The night theme switch: a 56 dp row, its label 16 dp in and the switch 16 dp from the end.
    fn night_row(
        &mut self,
        p: &mut Panel,
        i: usize,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let on = self
            .theme_editor
            .meta_form
            .as_ref()
            .is_some_and(|f| f.night);
        let label = self.label(self.rows()[i].label);
        let w = self.config[0] - x;
        let room = (w - self.px(16.0) - self.px(80.0)).max(1.0);
        let lh = self.measure(text, &label, self.sp(16), room).ceil();
        let rh = (lh + self.px(16.0)).max(self.px(56.0));
        let b = rect(x, y, w, rh);
        if self.focus == NIGHT_ID {
            self.paint_clipped_box(p, b, self.colors[4], self.px(8.0), clip);
        }
        let at = rect(x + self.px(16.0), y + ((rh - lh) / 2.0).round(), room, lh);
        self.paint_text(p, &label, at, self.sp(16), self.colors[1], clip);
        self.paint_switch(p, b, on, true, false, clip);
        let mut c = plain(NIGHT_ID, b, label, "switch");
        (c.checked, c.action) = (Some(on), Action::ThemeEdit(Tap::Night));
        self.push_control(c, clip);
        rh
    }
}
