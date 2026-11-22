//! The rule editor, EditRuleDialog as a screen: the element (a dropdown while adding, fixed
//! when editing, as Compose disabled it), the font's name for `@font`, the selector chips, the
//! codes as chips that open the code editor (`codes.rs`), and the mode and shift state chips with
//! a list to add one. Add and Apply refuse a rule that exists already, as `updateRule` did, and adding with no
//! element marks the dropdown, and a font name the stylesheet could not read back is refused.
//! Duplicate starts from a rule's element, selector and attributes, and the rule it adds takes
//! that rule's properties, unless its element was changed to one taking the other kind of set.
use super::{RULE_ADD, RULE_EDIT, rules::ordered};
use crate::settings_ui::{
    App,
    catalogue::{Kind, theme_editor::Part},
};
use snygg::{attributes::Attributes, grammar::Rule, resolve::PropertySets, text};
use std::mem::discriminant;

/// The dropdown's entry for `@font`; the empty entry is "--select--".
pub const FONT: &str = "@font";
/// The attributes the editor changes: FlorisImeUi.Attr's code, mode and shift state.
pub const ATTRIBUTES: [&str; 3] = ["code", "mode", "shiftstate"];
/// KeyboardMode's entries in EnumDisplayEntries, as `toString` writes them, with their labels.
pub const MODES: [(&str, &str); 7] = [
    ("characters", "enum__keyboard_mode__characters"),
    ("symbols", "enum__keyboard_mode__symbols"),
    ("symbols2", "enum__keyboard_mode__symbols2"),
    ("numeric", "enum__keyboard_mode__numeric"),
    ("numeric_advanced", "enum__keyboard_mode__numeric_advanced"),
    ("phone", "enum__keyboard_mode__phone"),
    ("phone2", "enum__keyboard_mode__phone2"),
];
/// InputShiftState's entries, likewise.
pub const SHIFT_STATES: [(&str, &str); 4] = [
    ("unshifted", "enum__input_shift_state__unshifted"),
    ("shifted_manual", "enum__input_shift_state__shifted_manual"),
    (
        "shifted_automatic",
        "enum__input_shift_state__shifted_automatic",
    ),
    ("caps_lock", "enum__input_shift_state__caps_lock"),
];
/// Why Add or Apply refused.
pub const EXISTS: &str = "settings__theme_editor__rule_already_exists";
/// A font name the stylesheet's grammar cannot read back: Compose wrote it, and the theme then
/// failed to open.
pub const INVALID_FONT: &str = "settings__theme_editor__font_name_invalid";

fn lossy(t: &[u16]) -> String {
    String::from_utf16_lossy(t)
}

/// The sets a new rule starts with: none, as `updateRule` added every rule, so a font's empty
/// list or an element's one empty set. A duplicate takes its source's sets when they are that
/// kind; a key duplicated as `@font` does not make the key's properties a font's one set.
fn new_sets(rule: &Rule, copied: Option<&PropertySets>) -> PropertySets {
    let empty = match rule {
        Rule::Font(_) => PropertySets::Multiple(vec![]),
        _ => PropertySets::Single(vec![]),
    };
    match copied {
        Some(sets) if discriminant(sets) == discriminant(&empty) => sets.clone(),
        _ => empty,
    }
}

/// The rule editor's state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuleForm {
    /// The rule being changed; None while adding.
    pub original: Option<Rule>,
    /// The rule a duplicate copies its properties from.
    pub source: Option<Rule>,
    /// Empty for none chosen, [`FONT`], or an element's name.
    pub element: String,
    pub font: String,
    /// A `grammar::SELECTORS` index; 0 for none.
    pub selector: usize,
    /// The rule's attributes, its codes among them.
    pub attributes: Attributes,
    /// Why Add or Apply refused: a string key, or empty.
    pub error: &'static str,
    /// Add was tried with no element chosen.
    pub checked: bool,
}

impl RuleForm {
    /// A form showing `rule`.
    fn of(rule: &Rule) -> Self {
        let mut f = Self::default();
        match rule {
            Rule::Defines => {}
            Rule::Font(name) => (f.element, f.font) = (FONT.into(), lossy(name)),
            Rule::Element {
                name,
                attributes,
                selector,
            } => {
                f.element = lossy(name);
                f.selector = *selector;
                f.attributes = attributes.clone();
            }
        }
        f
    }
    pub fn editing(rule: &Rule) -> Self {
        Self {
            original: Some(rule.clone()),
            ..Self::of(rule)
        }
    }
    pub fn copying(rule: &Rule) -> Self {
        Self {
            source: Some(rule.clone()),
            ..Self::of(rule)
        }
    }
    /// The element changed: a new rule of it, as EditRuleDialog started from its template.
    pub fn choose(&mut self, element: &str) {
        *self = Self {
            original: self.original.take(),
            source: self.source.take(),
            element: element.into(),
            ..Self::default()
        };
    }
    /// Whether the element is one, so selectors and attributes apply.
    pub fn is_element(&self) -> bool {
        !self.element.is_empty() && self.element != FONT
    }
    /// The values of attribute `k` (0 codes, 1 modes, 2 shift states) the rule has.
    pub fn values(&self, k: usize) -> Vec<String> {
        let key = text(ATTRIBUTES[k]);
        self.attributes
            .0
            .iter()
            .find(|(name, _)| *name == key)
            .map_or(vec![], |(_, v)| v.iter().map(|t| lossy(t)).collect())
    }
    /// Adds or removes `value` of attribute `k`, as `including` and `excluding` did.
    pub fn set_value(&mut self, k: usize, value: &str, on: bool) {
        let pair = (text(ATTRIBUTES[k]), text(value));
        self.attributes.edit(&[pair], u8::from(!on));
    }
    /// The rule the form describes, or the string key of why there is none (empty: no element).
    pub fn build(&self) -> Result<Rule, &'static str> {
        match self.element.as_str() {
            "" => Err(""),
            FONT => {
                let rule = Rule::Font(text(self.font.trim()));
                let back = snygg::grammar::parse(&rule.format()).ok().flatten();
                back.filter(|b| *b == rule).ok_or(INVALID_FONT)
            }
            name => Ok(Rule::Element {
                name: text(name),
                attributes: self.attributes.clone(),
                selector: self.selector,
            }),
        }
    }
}

impl App {
    /// Add rule: the rule editor with nothing chosen.
    pub(super) fn add_rule(&mut self) {
        self.theme_editor.form = Some(RuleForm::default());
        self.theme_dialog_opened();
        self.navigate(RULE_ADD);
    }
    /// A rule's Edit: the rule editor showing rule `r`. `@defines` has none.
    pub(super) fn edit_rule(&mut self, r: usize) {
        let Some((rule, _)) = self.theme_editor.rules.get(r) else {
            return;
        };
        if *rule == Rule::Defines {
            return;
        }
        self.theme_editor.form = Some(RuleForm::editing(rule));
        self.theme_dialog_opened();
        self.navigate(RULE_EDIT);
    }
    /// Add or Apply: the rule goes in, renamed in place or added with no properties (a
    /// duplicate with its source's), unless such a rule exists; the editor closes.
    pub(super) fn save_rule(&mut self) {
        let Some(form) = self.theme_editor.form.as_mut() else {
            return;
        };
        let rule = match form.build() {
            Ok(rule) => rule,
            Err(key) => {
                (form.error, form.checked) = (key, key.is_empty());
                return;
            }
        };
        let (original, source) = (form.original.clone(), form.source.clone());
        let e = &mut self.theme_editor;
        // Another rule like it: the one being changed does not count.
        let exists = e.rules.iter().any(|(r, _)| {
            r.compare(&rule) == 0 && original.as_ref().is_none_or(|o| r.compare(o) != 0)
        });
        if exists {
            if let Some(form) = e.form.as_mut() {
                form.error = EXISTS;
            }
            return;
        }
        match original {
            Some(old) => {
                if let Some(at) = e.rules.iter().position(|(r, _)| r.compare(&old) == 0) {
                    e.rules[at].0 = rule;
                }
            }
            None => {
                let copied = source.and_then(|s| e.rules.iter().find(|(r, _)| *r == s));
                let sets = new_sets(&rule, copied.map(|(_, sets)| sets));
                e.rules.push((rule, sets));
            }
        }
        e.rules = ordered(std::mem::take(&mut e.rules));
        self.theme_changed();
        self.close_editor();
    }
    /// The delete question's Delete: the rule being edited goes, to undo from the rules.
    pub(in crate::settings_ui) fn delete_form_rule(&mut self) {
        let original = self
            .theme_editor
            .form
            .as_ref()
            .and_then(|f| f.original.clone());
        let at = original.and_then(|o| self.theme_editor.rules.iter().position(|(r, _)| *r == o));
        if let Some(r) = at {
            self.delete_rule(r);
        }
        self.close_editor();
    }
    /// What row `i`'s list offers: the elements, attribute values the rule lacks, or what a
    /// Material You flag can be.
    pub(in crate::settings_ui) fn theme_choices(&self, i: usize) -> Option<Vec<String>> {
        if let Kind::ThemeEdit(Part::MetaChoice(k)) = self.rows().get(i)?.kind {
            return Some(self.theme_editor.component.choices[k].clone());
        }
        if let Some(choices) = self.property_choices(i) {
            return Some(choices);
        }
        let form = self.theme_editor.form.as_ref()?;
        match self.rows().get(i)?.kind {
            Kind::ThemeEdit(Part::Element) => {
                let mut all = vec![String::new(), FONT.to_owned()];
                all.extend(super::names::ELEMENTS.iter().map(|e| (*e).to_owned()));
                Some(all)
            }
            Kind::ThemeEdit(Part::Attribute(k @ 1..=2)) => {
                let had = form.values(k);
                let all = if k == 1 {
                    &MODES[..]
                } else {
                    &SHIFT_STATES[..]
                };
                let missing = all.iter().filter(|(v, _)| !had.iter().any(|h| h == v));
                Some(missing.map(|(v, _)| (*v).to_owned()).collect())
            }
            _ => None,
        }
    }
    /// A list entry's label: an element as `translateElementName` names it, or a value's.
    pub(in crate::settings_ui) fn theme_choice_label(
        &self,
        i: usize,
        choice: &str,
    ) -> Option<String> {
        match self.rows().get(i)?.kind {
            Kind::ThemeEdit(Part::Element) => Some(self.element_choice(choice)),
            Kind::ThemeEdit(Part::Attribute(k)) => Some(self.attr_value_label(k, choice)),
            // As Compose's dropdowns named them: the enum's name.
            Kind::ThemeEdit(Part::MetaChoice(_)) => Some(choice.to_owned()),
            Kind::ThemeEdit(Part::PropName | Part::PropEncoder | Part::PropValue) => {
                Some(self.property_choice_label(i, choice))
            }
            _ => None,
        }
    }
    /// How the dropdown names `element`.
    pub fn element_choice(&self, element: &str) -> String {
        match element {
            "" => self.label("general__select_dropdown_value_placeholder"),
            FONT => self.element_label("font"),
            name => self.element_label(name),
        }
    }
    /// Value `v` of attribute `k`: its label, or as written at the developer level.
    pub fn attr_value_label(&self, k: usize, v: &str) -> String {
        let all = if k == 1 {
            &MODES[..]
        } else {
            &SHIFT_STATES[..]
        };
        match all.iter().find(|(value, _)| *value == v) {
            Some((_, key)) if !self.theme_developer() => self.label(key),
            _ => v.to_owned(),
        }
    }
    /// A list entry tapped on row `i`: the element or a Material You flag chosen, or the value
    /// added.
    pub(in crate::settings_ui) fn theme_choose(&mut self, i: usize, choice: &str) {
        if self.property_choose(i, choice) {
            return;
        }
        let kind = self.rows().get(i).map(|r| r.kind);
        if let (Some(Kind::ThemeEdit(Part::MetaChoice(k))), Some(f)) =
            (kind, self.theme_editor.meta_form.as_mut())
        {
            return f.flags[k] = choice.into();
        }
        let Some(form) = self.theme_editor.form.as_mut() else {
            return;
        };
        match kind {
            Some(Kind::ThemeEdit(Part::Element)) => form.choose(choice),
            Some(Kind::ThemeEdit(Part::Attribute(k))) => form.set_value(k, choice, true),
            _ => {}
        }
    }
    /// The rule editor's message: why Add or Apply refused.
    pub fn rule_error(&self) -> String {
        let key = self.theme_editor.form.as_ref().map_or("", |f| f.error);
        if key.is_empty() {
            return String::new();
        }
        self.label(key)
    }
}
