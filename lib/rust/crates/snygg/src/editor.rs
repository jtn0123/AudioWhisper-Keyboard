//! Ordered editor operations. Managed adapters keep public mutable collection identities;
//! this model is independently usable by Rust consumers, including draft previews.
use crate::{
    Text,
    grammar::Rule,
    resolve::{Properties, PropertySets, Stylesheet, set},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleEdit {
    Unchanged,
    Duplicate,
    Missing,
    Add,
    Replace(usize),
}
/// Rename/add validates before removing the old entry. Equal rules are a successful no-op.
pub fn rule_edit(rules: &[Rule], old: &Rule, new: &Rule, allow_add: bool) -> RuleEdit {
    if old == new {
        return RuleEdit::Unchanged;
    }
    if rules.iter().any(|r| r.compare(new) == 0) {
        return RuleEdit::Duplicate;
    }
    match rules.iter().position(|r| r.compare(old) == 0) {
        Some(i) => RuleEdit::Replace(i),
        None if allow_add => RuleEdit::Add,
        None => RuleEdit::Missing,
    }
}
/// A property's replacement retains its insertion position. Unsetting then setting appends.
pub fn property_slot(keys: &[Text], key: &[u16]) -> Option<usize> {
    keys.iter().position(|k| k == key)
}
#[derive(Clone, Debug)]
pub struct Editor<V> {
    pub sheet: Stylesheet<V>,
}
impl<V: Clone> Editor<V> {
    pub fn new(mut sheet: Stylesheet<V>) -> Self {
        sheet.rules.sort_by(|(a, _), (b, _)| a.compare(b).cmp(&0));
        // Like TreeMap, comparator-equivalent keys keep their first key and last value.
        let mut rules: Vec<(Rule, PropertySets<V>)> = vec![];
        for (r, v) in sheet.rules {
            if let Some((_, old)) = rules.iter_mut().find(|(k, _)| k.compare(&r) == 0) {
                *old = v;
            } else {
                rules.push((r, v));
            }
        }
        Self {
            sheet: Stylesheet { rules },
        }
    }
    pub fn rename(&mut self, old: &Rule, new: Rule, allow_add: bool) -> RuleEdit {
        let keys: Vec<_> = self.sheet.rules.iter().map(|(r, _)| r.clone()).collect();
        let plan = rule_edit(&keys, old, &new, allow_add);
        match plan {
            RuleEdit::Replace(i) => self.sheet.rules[i].0 = new,
            RuleEdit::Add => {
                let sets = if matches!(new, Rule::Font(_)) {
                    PropertySets::Multiple(vec![])
                } else {
                    PropertySets::Single(vec![])
                };
                self.sheet.rules.push((new, sets));
            }
            _ => {}
        }
        self.sheet
            .rules
            .sort_by(|(a, _), (b, _)| a.compare(b).cmp(&0));
        plan
    }
    pub fn remove(&mut self, rule: &Rule) -> Option<PropertySets<V>> {
        let i = self
            .sheet
            .rules
            .iter()
            .position(|(r, _)| r.compare(rule) == 0)?;
        Some(self.sheet.rules.remove(i).1)
    }
    pub fn copy_rule(&mut self, source: &Rule, target: Rule) -> bool {
        if self
            .sheet
            .rules
            .iter()
            .any(|(r, _)| r.compare(&target) == 0)
        {
            return false;
        }
        let Some((_, v)) = self
            .sheet
            .rules
            .iter()
            .find(|(r, _)| r.compare(source) == 0)
        else {
            return false;
        };
        self.sheet.rules.push((target, v.clone()));
        self.sheet
            .rules
            .sort_by(|(a, _), (b, _)| a.compare(b).cmp(&0));
        true
    }
    pub fn set_property(properties: &mut Properties<V>, name: Text, value: Option<V>) -> Option<V> {
        set(properties, name, value)
    }
    pub fn build(&self) -> Stylesheet<V> {
        self.sheet.clone()
    }
}

/// The theme editor's display order is supplied as data, never a callback. Unknown
/// elements follow known ones; specificity and annotation ordering use the grammar.
pub fn compare_order(a: &Rule, b: &Rule, elements: &[Text]) -> i32 {
    if let (Rule::Element { name: a_name, .. }, Rule::Element { name: b_name, .. }) = (a, b)
        && a_name != b_name
    {
        let ai = elements.iter().position(|n| n == a_name);
        let bi = elements.iter().position(|n| n == b_name);
        return match (ai, bi) {
            (Some(a), Some(b)) => a.cmp(&b) as i32,
            (Some(_), None) => -1,
            (None, Some(_)) => 1,
            (None, None) => crate::compare(a_name, b_name),
        };
    }
    a.compare(b)
}
