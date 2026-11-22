//! The component editor's painting, as ThemeEditorScreen laid it out: the loading message, the
//! stylesheet error (Compose's dialog, here a page with its Yes and No) or the missing workspace;
//! ExtensionComponentView's box with its Edit; Fine-tune and Add rule as ListItems where Compose
//! had a top bar button and a floating button; then each rule in an outlined box, SnyggRuleRow's
//! lines over its properties. A rule row has Duplicate and Edit; a property has Delete, and the
//! undo bar stands where the deleted rule or property was.
use super::{Gone, Load, RULE_EDIT, Tap, rules::ordered, target};
use crate::settings_ui::{
    Action, App,
    cards::{TITLE_MEDIUM, plain},
    catalogue::theme_editor::Part,
    items::Entry,
    languages::form::Button,
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

pub const STATUS_ID: i32 = 400_000;
pub const YES_ID: i32 = 400_001;
pub const NO_ID: i32 = 400_002;
/// The component box: its title, its name, its text and its Edit.
pub const META_ID: i32 = 400_010;
pub const TUNE_ID: i32 = 400_020;
pub const ADD_ID: i32 = 400_030;
pub const EMPTY_ID: i32 = 400_031;
/// The top bar's Save, and in the rule editor its Delete.
pub const SAVE_ID: i32 = 400_040;
pub const DELETE_ID: i32 = 400_041;
/// Rule `r`'s controls start at `RULE_ID + 4096 * r`.
pub const RULE_ID: i32 = 500_000;

/// Rule `r`'s control `part`: 0 its row, 1 Duplicate, 2 Edit, 3 its note, 4 onwards a set's
/// heading, 64 + 2k its `k`th property and 65 + 2k that property's Delete.
pub fn rule_id(r: usize, part: usize) -> i32 {
    RULE_ID + 4096 * r as i32 + part.min(4095) as i32
}

impl App {
    /// Paints row `i`'s part `part`; its height.
    pub(in crate::settings_ui) fn theme_part(
        &mut self,
        p: &mut Panel,
        (i, part): (usize, Part),
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let ready = self.theme_editor.load == Load::Ready;
        match part {
            Part::Status => self.theme_status(p, at, clip, text),
            Part::Meta if ready => self.component_box(p, at, clip, text),
            Part::Tune if ready => {
                let title = self.label(self.rows()[i].label);
                self.theme_entry(p, (TUNE_ID, title, "tune", Tap::Tune), at, clip, text)
            }
            Part::Rules if ready => self.theme_rules(p, at, clip, text),
            Part::MetaField(_) | Part::Night => self.meta_part(p, (i, part), at, clip, text),
            Part::Meta | Part::Tune | Part::Rules => 0.0,
            _ => self.rule_part(p, (i, part), at, clip, text),
        }
    }
    /// A ListItem with a leading icon that does `tap`; its height.
    fn theme_entry(
        &mut self,
        p: &mut Panel,
        (id, title, icon, tap): (i32, String, &'static str, Tap),
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let entry = Entry {
            id,
            title,
            summary: String::new(),
            icon,
            ink: [None, None],
            buttons: 0,
            right: 0.0,
            enabled: true,
            action: Action::ThemeEdit(tap),
        };
        self.paint_entry(p, entry, at, clip, text).0
    }
    /// Loading, the workspace gone, or the stylesheet error and its question; else nothing.
    fn theme_status(&mut self, p: &mut Panel, at: (f32, f32), clip: Rect, text: &mut Text) -> f32 {
        let pad = self.px(16.0);
        let (key, ink) = match self.theme_editor.load.clone() {
            Load::Ready => return 0.0,
            Load::Failed(error) => return self.stylesheet_error(p, error, at, clip, text),
            Load::Waiting => ("settings__theme_editor__loading", self.colors[2]),
            Load::Missing => ("settings__theme_editor__missing_workspace", self.colors[18]),
        };
        self.theme_note(p, (STATUS_ID, self.label(key), ink, pad), at, clip, text)
    }
    /// The stylesheet error: its title, the error, the question, and Yes and No.
    fn stylesheet_error(
        &mut self,
        p: &mut Panel,
        error: String,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let width = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
        let title = self.label("settings__theme_editor__stylesheet_error_title");
        let th = self
            .styled(text, &title, TITLE_MEDIUM, Some(width))
            .height
            .ceil();
        let b = rect(x + self.px(16.0), y + self.px(16.0), width, th);
        self.paint_styled(p, &title, b, TITLE_MEDIUM, self.colors[1], clip);
        self.push_control(plain(STATUS_ID, b, title, "heading"), clip);
        let mut at = b.bottom + self.px(8.0);
        let ink = self.colors[2];
        at += self.theme_note(p, (STATUS_ID + 3, error, ink, 0.0), (x, at), clip, text);
        let ask = self.label("settings__theme_editor__stylesheet_error_description");
        let ink = self.colors[1];
        at += self.theme_note(
            p,
            (STATUS_ID + 4, ask, ink, self.px(8.0)),
            (x, at),
            clip,
            text,
        );
        let mut right = self.config[0] - self.px(16.0);
        for (id, key, yes) in [(YES_ID, "action__yes", true), (NO_ID, "action__no", false)] {
            let button = Button {
                id,
                label: self.label(key),
                icon: "",
                filled: yes,
                ink: self.colors[if yes { 6 } else { 3 }],
                action: Action::ThemeEdit(Tap::Lenient(yes)),
            };
            let bw = self.button_width(text, &button);
            let touch = rect(right - bw, at, bw, self.px(48.0));
            right = touch.left - self.px(8.0);
            self.material_button(p, button, touch, clip, text);
        }
        at + self.px(56.0) - y
    }
    /// ExtensionComponentView: the label over the component's name, its authors, night flag
    /// and stylesheet path, and Edit at the end; its height with the 8 dp round it.
    fn component_box(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let left = x + self.px(16.0);
        let width = self.config[0] - x - 2.0 * self.px(16.0);
        let border_top = y + self.px(8.0) + self.px(11.0);
        let outline = p.items.len();
        self.outlined_box(p, rect(left, border_top, width, 0.0), clip);
        let (c, meta) = (&self.theme_editor.component, &self.theme_editor.meta);
        let name = format!("{}:{}", c.extension, c.id);
        let body = format!(
            "authors = [{}]\nisNightTheme = {}\nstylesheetPath = {}",
            meta.authors.join(", "),
            meta.night,
            c.path
        );
        let label = meta.label.clone();
        let edit = Action::ThemeEdit(Tap::Meta);
        let id = (META_ID + 1, name.as_str(), edit.clone());
        let mut at = self.box_id(p, id, (left, border_top + self.px(11.0)), clip, text);
        let ink = self.colors[1];
        at += self.theme_note(p, (META_ID + 2, body, ink, 0.0), (left, at), clip, text);
        let button = Button {
            id: META_ID + 3,
            label: self.label("action__edit"),
            icon: "edit",
            filled: false,
            ink: self.colors[3],
            action: edit.clone(),
        };
        let bw = self.button_width(text, &button);
        let touch = rect(left + width - self.px(6.0) - bw, at, bw, self.px(48.0));
        self.material_button(p, button, touch, clip, text);
        at += self.px(48.0);
        self.fit_box(p, outline, at - border_top);
        let border = rect(left, border_top, width, 0.0);
        self.box_title(p, (META_ID, &label), edit, border, clip, text);
        at + self.px(8.0) - y
    }
    /// Add rule, the note when no rule is defined, the reason a save failed, then the rules.
    fn theme_rules(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let add = self.label("settings__theme_editor__add_rule");
        let mut at = y + self.theme_entry(p, (ADD_ID, add, "add", Tap::Add), (x, y), clip, text);
        let rules = self.theme_editor.rules.clone();
        if rules
            .iter()
            .all(|(r, _)| *r == snygg::grammar::Rule::Defines)
        {
            let note = self.label("settings__theme_editor__no_rules_defined");
            let ink = self.colors[2];
            at += self.theme_note(p, (EMPTY_ID, note, ink, self.px(8.0)), (x, at), clip, text);
        }
        if let Some(error) = self.theme_editor.error.clone() {
            let ink = self.colors[18];
            at += self.theme_note(
                p,
                (EMPTY_ID + 1, error, ink, self.px(8.0)),
                (x, at),
                clip,
                text,
            );
        }
        let gone_at = self.gone_rule_place();
        for r in 0..rules.len() {
            if gone_at == Some(r) {
                at += self.theme_undo(p, (x, at), clip, text);
            }
            at += self.rule_box(p, r, (x, at), clip, text);
        }
        if gone_at == Some(rules.len()) {
            at += self.theme_undo(p, (x, at), clip, text);
        }
        at + self.px(72.0) - y
    }
    /// Where the deleted rule would stand in the list, if a rule was the last thing deleted.
    fn gone_rule_place(&self) -> Option<usize> {
        let Some(Gone::Rule(rule, sets)) = self.theme_editor.deleted.clone() else {
            return None;
        };
        let mut all = self.theme_editor.rules.clone();
        all.push((rule.clone(), sets));
        ordered(all).iter().position(|(r, _)| *r == rule)
    }
    /// The undo bar for what was deleted; its height.
    pub(super) fn theme_undo(
        &mut self,
        p: &mut Panel,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let Some(name) = self.theme_deleted_name() else {
            return 0.0;
        };
        self.undo_bar(p, (&name, Action::ThemeEdit(Tap::Undo)), at, clip, text)
    }
    /// The top bar's action: Save once the stylesheet loaded, Delete when editing a rule.
    pub fn theme_header(&self) -> Option<(i32, &'static str, &'static str, String, Action)> {
        let top = self.stack.last()?;
        if target(top).is_some() {
            let ready = self.theme_editor.load == Load::Ready;
            let save = Action::ThemeEdit(Tap::Save);
            return ready.then(|| (SAVE_ID, "done", "action__save", String::new(), save));
        }
        let original = self.theme_editor.form.as_ref()?.original.as_ref()?;
        let title = self.rule_lines(original).0;
        let delete = Action::ThemeEdit(Tap::AskDelete);
        (top == RULE_EDIT).then_some((DELETE_ID, "delete", "action__delete", title, delete))
    }
}
