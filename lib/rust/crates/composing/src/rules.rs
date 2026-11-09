//! A table of rules, each an ending and what it becomes, as FlorisBoard's `WithRules` applied
//! them (Telex, for Vietnamese, is one: `aa` becomes `â`, `as` becomes `á`). The keystroke is
//! added to the text before it, and the first rule whose ending that text ends with, lower-cased,
//! replaces the ending: all of it but the keystroke is deleted, and the rule's text typed in its
//! place, upper-cased if the ending began with a letter typed upper-case.
//!
//! Kotlin orders the rules (the longest endings first) and hands them over once; [`Rules`] keeps
//! them. Case mapping is Kotlin's `lowercase()` and `uppercase()`, from `chars`, which can change
//! a string's length (ß upper-cases to SS, İ lower-cases to i and a combining dot).

use crate::Action;

/// The rules, in the order they are tried.
#[derive(Debug)]
pub struct Rules {
    rules: Vec<(Vec<u16>, Vec<u16>)>,
}

impl Rules {
    /// Rules to try in this order: each an ending and what replaces it.
    pub fn new(rules: Vec<(Vec<u16>, Vec<u16>)>) -> Self {
        Rules { rules }
    }

    /// `WithRules.getActions`: what typing `to_insert` after `preceding` does.
    pub fn actions(&self, preceding: &[u16], to_insert: &[u16]) -> Option<Action> {
        let text = [preceding, to_insert].concat();
        let lower = chars::lowercase(&text);
        let (key, value) = self.rules.iter().find(|(key, _)| lower.ends_with(key))?;
        // Kotlin's takeLast(key.length).take(1): the lower-cased text can be longer than the text.
        let ending = &text[text.len().saturating_sub(key.len())..];
        let first = &ending[..ending.len().min(1)];
        let text = if chars::uppercase(first) == first {
            chars::uppercase(value)
        } else {
            value.clone()
        };
        // A key's length is a Kotlin String's, an Int.
        Some(Action::new(key.len() as i32 - 1, text))
    }
}

#[cfg(test)]
mod tests;
