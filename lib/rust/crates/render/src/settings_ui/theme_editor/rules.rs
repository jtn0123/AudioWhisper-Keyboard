//! The rules list's model: `compare_order`, what each rule and property shows, Delete with Undo
//! and Duplicate. A rule shows as SnyggRuleRow did: its element translated (as written at the
//! developer level), its selector, then each attribute as `key = [values]`; an annotation as it
//! is written. A property shows its translated name and its value as the stylesheet writes it.
use super::{Gone, Rules, form::RuleForm, names};
use crate::settings_ui::App;
use snygg::{
    Text,
    editor::compare_order,
    grammar::{Rule, SELECTORS},
    resolve::PropertySets,
    text,
};

/// The selectors' string keys, by `grammar::SELECTORS` index.
pub const SELECTOR_KEYS: [&str; 5] = [
    "",
    "snygg__rule_selector__pressed",
    "snygg__rule_selector__focus",
    "snygg__rule_selector__hover",
    "snygg__rule_selector__disabled",
];

/// A property as the list shows it: where it is, its name as written and translated, and its
/// value as the stylesheet writes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prop {
    pub set: usize,
    pub place: usize,
    pub raw: String,
    pub name: String,
    pub value: String,
}

/// `rules` in the order ThemeEditorScreen's `CustomRuleComparator` keeps them.
pub fn ordered(mut rules: Rules) -> Rules {
    let order: Vec<Text> = names::ELEMENTS.iter().map(|e| text(e)).collect();
    rules.sort_by(|a, b| compare_order(&a.0, &b.0, &order).cmp(&0));
    rules
}
fn lossy(t: &[u16]) -> String {
    String::from_utf16_lossy(t)
}

impl App {
    /// Whether the editor level is DEVELOPER, which shows names as written.
    pub fn theme_developer(&self) -> bool {
        self.value("theme__editor_level") == "DEVELOPER"
    }
    /// An element's translated name: `translateElementName`, which also names the annotations.
    pub fn element_label(&self, name: &str) -> String {
        if self.theme_developer() {
            return name.into();
        }
        let key = match name {
            "defines" => Some("snygg__rule_annotation__defines"),
            "font" => Some("snygg__rule_annotation__font"),
            _ => names::ELEMENTS
                .iter()
                .position(|e| *e == name)
                .map(|i| names::ELEMENT_KEYS[i]),
        };
        key.map_or_else(|| name.into(), |k| self.label(k))
    }
    /// A rule's lines: its title, its selector chip (or empty) and its attributes.
    pub fn rule_lines(&self, rule: &Rule) -> (String, String, Vec<String>) {
        let Rule::Element {
            name,
            attributes,
            selector,
        } = rule
        else {
            return (lossy(&rule.format()), String::new(), vec![]);
        };
        let chip = match *selector {
            0 => String::new(),
            s if self.theme_developer() => SELECTORS[s][1..].into(),
            s => self.label(SELECTOR_KEYS[s]),
        };
        let attrs = attributes
            .0
            .iter()
            .map(|(key, values)| {
                let list: Vec<String> = values.iter().map(|v| lossy(v)).collect();
                format!("{} = [{}]", lossy(key), list.join(", "))
            })
            .collect();
        (self.element_label(&lossy(name)), chip, attrs)
    }
    /// A property's name as the list shows it: `translatePropertyName`.
    pub fn property_label(&self, name: &[u16]) -> String {
        let name = lossy(name);
        let key = names::PROPERTIES.iter().find(|(n, _)| *n == name);
        match key {
            Some((_, k)) if !self.theme_developer() => self.label(k),
            _ => name,
        }
    }
    /// The properties of rule `r` in order.
    pub fn rule_properties(&self, r: usize) -> Vec<Prop> {
        let Some((_, sets)) = self.theme_editor.rules.get(r) else {
            return vec![];
        };
        let all = match sets {
            PropertySets::Single(props) => vec![props],
            PropertySets::Multiple(sets) => sets.iter().collect(),
        };
        let mut out = vec![];
        for (set, props) in all.into_iter().enumerate() {
            for (place, (name, value)) in props.iter().enumerate() {
                let shown = value.encoder().serialize(value);
                out.push(Prop {
                    set,
                    place,
                    raw: lossy(name),
                    name: self.property_label(name),
                    value: shown.map_or_else(|_| String::from("?"), |t| lossy(&t)),
                });
            }
        }
        out
    }
    /// Deletes property `k` of set `s` of rule `r`; Undo puts it back where it was.
    pub(super) fn delete_property(&mut self, r: usize, s: usize, k: usize) {
        let e = &mut self.theme_editor;
        let Some((rule, sets)) = e.rules.get_mut(r) else {
            return;
        };
        let props = match sets {
            PropertySets::Single(props) => (s == 0).then_some(props),
            PropertySets::Multiple(sets) => sets.get_mut(s),
        };
        let Some(props) = props.filter(|p| k < p.len()) else {
            return;
        };
        if super::boxes::required(rule, &lossy(&props[k].0)) {
            return;
        }
        let (name, value) = props.remove(k);
        e.deleted = Some(Gone::Property(rule.clone(), s, k, name, value));
        self.theme_changed();
    }
    /// Deletes rule `r`; Undo puts it back with its properties.
    pub(super) fn delete_rule(&mut self, r: usize) {
        let e = &mut self.theme_editor;
        if r >= e.rules.len() || e.rules[r].0 == Rule::Defines {
            return;
        }
        let (rule, sets) = e.rules.remove(r);
        e.deleted = Some(Gone::Rule(rule, sets));
        self.theme_changed();
    }
    /// Undo: the last deleted rule or property back where it was. A rule added since in its
    /// place, or a property whose rule is gone, cannot come back.
    pub(super) fn undo_theme_delete(&mut self) {
        let e = &mut self.theme_editor;
        match e.deleted.take() {
            Some(Gone::Rule(rule, sets)) => {
                if e.rules.iter().all(|(r, _)| r.compare(&rule) != 0) {
                    e.rules.push((rule, sets));
                    e.rules = ordered(std::mem::take(&mut e.rules));
                }
            }
            Some(Gone::Property(rule, s, k, name, value)) => {
                let Some((_, sets)) = e.rules.iter_mut().find(|(r, _)| *r == rule) else {
                    return;
                };
                let props = match sets {
                    PropertySets::Single(props) => Some(props),
                    PropertySets::Multiple(sets) => sets.get_mut(s),
                };
                if let Some(props) = props {
                    props.insert(k.min(props.len()), (name, value));
                }
            }
            None => return,
        }
        self.theme_changed();
    }
    /// The undo bar's message names what went: a rule by its title, a property by its name.
    pub fn theme_deleted_name(&self) -> Option<String> {
        Some(match self.theme_editor.deleted.as_ref()? {
            Gone::Rule(rule, _) => self.rule_lines(rule).0,
            Gone::Property(_, _, _, name, _) => self.property_label(name),
        })
    }
    /// Duplicate: the rule editor adding a rule like rule `r`, which takes `r`'s properties.
    pub(super) fn duplicate_rule(&mut self, r: usize) {
        let Some((rule, _)) = self.theme_editor.rules.get(r) else {
            return;
        };
        self.theme_editor.form = Some(RuleForm::copying(rule));
        self.theme_dialog_opened();
        self.navigate(super::RULE_ADD);
    }
}
