//! The component meta editor, ComponentMetaEditorDialog as a screen: the label, the authors and
//! the night theme switch. The component's id, its stylesheet path and Material You's flags are
//! not offered here. The authors are one comma-separated line where Compose took one per line,
//! as the settings renderer's fields are single-line. Apply checks the label and the authors
//! with `ExtensionValidation`'s rules (the `extensions` crate) and shows why under each field.
use super::{META, Meta, Tap};
use crate::settings_ui::{Action, App, cards::plain, catalogue::theme_editor::Part};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use extensions::validation::validate;
use keys::Rect;
use wire::extensions::{
    COMPONENT_AUTHORS, COMPONENT_LABEL, INVALID,
    validation::{RESOURCE, STATUS},
};

/// The night switch's control id.
pub const NIGHT_ID: i32 = 400_050;

/// The meta editor's state: the fields as typed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MetaForm {
    pub label: String,
    pub authors: String,
    pub night: bool,
    /// Apply was tried: the fields' messages show.
    pub checked: bool,
}

impl MetaForm {
    pub fn of(meta: &Meta) -> Self {
        Self {
            label: meta.label.clone(),
            authors: meta.authors.join(", "),
            night: meta.night,
            checked: false,
        }
    }
    /// The authors as the component keeps them: trimmed, the blank ones dropped.
    pub fn authors(&self) -> Vec<String> {
        let all = self.authors.split(',').map(str::trim);
        all.filter(|a| !a.is_empty()).map(Into::into).collect()
    }
    /// Field `k`'s text: 0 the label, 1 the authors.
    pub fn field(&self, k: usize) -> &str {
        if k == 0 { &self.label } else { &self.authors }
    }
    /// Field `k` as the extension editor checks it: the authors one per line, as Compose took
    /// them.
    pub fn checked_text(&self, k: usize) -> String {
        if k == 0 {
            self.label.clone()
        } else {
            self.authors().join("\n")
        }
    }
}

/// Field `k`'s check: its message's string key (empty for none), and whether it refuses Apply.
pub fn check(k: usize, value: &str) -> (String, bool) {
    let rule = if k == 0 {
        COMPONENT_LABEL
    } else {
        COMPONENT_AUTHORS
    };
    let row = validate(rule, &value.encode_utf16().collect::<Vec<_>>());
    let status = String::from_utf16_lossy(&row[STATUS]);
    (String::from_utf16_lossy(&row[RESOURCE]), status == INVALID)
}

impl App {
    /// The component box's Edit: the meta editor showing the component's meta.
    pub(super) fn open_meta(&mut self) {
        self.theme_editor.meta_form = Some(MetaForm::of(&self.theme_editor.meta));
        self.theme_dialog_opened();
        self.navigate(META);
    }
    /// Field `k`'s message once Apply was tried, translated.
    pub fn meta_error(&self, k: usize) -> String {
        let Some(f) = self.theme_editor.meta_form.as_ref().filter(|f| f.checked) else {
            return String::new();
        };
        let (key, _) = check(k, &f.checked_text(k));
        if key.is_empty() {
            key
        } else {
            self.label(&key)
        }
    }
    /// Field `k` changed.
    pub(in crate::settings_ui) fn meta_typed(&mut self, k: usize, value: String) {
        if let Some(f) = self.theme_editor.meta_form.as_mut() {
            if k == 0 {
                f.label = value;
            } else {
                f.authors = value;
            }
        }
    }
    /// Apply: the meta changes unless a field refuses, and the editor closes.
    pub(super) fn save_meta(&mut self) {
        let Some(f) = self.theme_editor.meta_form.as_mut() else {
            return;
        };
        if check(0, &f.checked_text(0)).1 || check(1, &f.checked_text(1)).1 {
            f.checked = true;
            return;
        }
        let meta = Meta {
            label: f.label.trim().into(),
            authors: f.authors(),
            night: f.night,
        };
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
        match part {
            Part::MetaField(k) => {
                let title = self.label(self.rows()[i].label);
                self.line_box(p, (i, title, self.meta_error(k)), at, clip, text)
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
