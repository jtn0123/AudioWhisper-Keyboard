//! The text composers (the `composing` crate) for `ComposingNative`, whose generated entry points
//! call these.
//!
//! A rule table crosses once, as its endings and their texts, and is kept behind a handle. An
//! action comes back as a row of two: the number of units to delete as a decimal (it can be -1),
//! then the text to type. With nothing to do, that is 0 and the keystroke itself.

use composing::Action;
use composing::rules::Rules;

use crate::handle::{free, into_handle, object};
use crate::jni::{Result, illegal};

/// Rules to try in order: ending `i` is `keys[i]`, and its text `values[i]`.
pub fn new_rules(keys: &[Vec<u16>], values: &[Vec<u16>]) -> Result<i64> {
    if keys.len() != values.len() {
        return Err(illegal("as many texts as endings"));
    }
    let rules = keys.iter().cloned().zip(values.iter().cloned());
    Ok(into_handle(Rules::new(rules.collect())))
}

pub fn free_rules(rules: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each rule table once, when nothing can reach it.
    unsafe { free::<Rules>(rules) };
    Ok(())
}

/// Typing `to_insert` after `preceding` with the rules behind `rules` (`Rules::actions`). The
/// rules never change, so calls from several threads may share them unlocked.
pub fn rules(rules: i64, preceding: &[u16], to_insert: &[u16]) -> Result<Vec<Vec<u16>>> {
    // SAFETY: Kotlin holds the rules while it calls (see `object`).
    let rules = unsafe { object::<Rules>(rules, "rules")? };
    let action = rules.actions(preceding, to_insert).unwrap_or(Action {
        delete: 0,
        text: to_insert.to_vec(),
    });
    Ok(vec![
        action.delete.to_string().encode_utf16().collect(),
        action.text,
    ])
}
