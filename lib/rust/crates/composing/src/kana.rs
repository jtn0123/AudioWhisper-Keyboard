//! Japanese kana's marks, as FlorisBoard's `KanaUnicode` applied them: a dakuten (voicing mark)
//! after a kana voices it, a handakuten half-voices it, and the small-kana key (〓) makes it
//! small; the same again on a kana that already has it takes it back off. A mark with nothing to
//! apply to is typed as it is, and the small-kana key then types nothing. Two combining marks in
//! a row become the second, or nothing if they are the same.
//!
//! *Sticky* (a setting of the composer, off in FlorisBoard's) keeps a mark on rather than taking
//! it off, and keeps a repeated combining mark.
//!
//! Only the last unit before the cursor is looked at.

use crate::Action;

mod tables;

use tables::{DAKU, HANDAKU, REVERSE_DAKU, REVERSE_HANDAKU, REVERSE_SMALL, SMALL};

/// The small-kana key's character.
const SMALL_SENTINEL: u16 = 0x3013;
/// The combining dakuten and handakuten.
const COMBINING_DAKUTEN: u16 = 0x3099;
const COMBINING_HANDAKUTEN: u16 = 0x309A;

fn is_dakuten(unit: u16) -> bool {
    matches!(unit, COMBINING_DAKUTEN | 0x309B | 0xFF9E)
}

fn is_handakuten(unit: u16) -> bool {
    matches!(unit, COMBINING_HANDAKUTEN | 0x309C | 0xFF9F)
}

fn is_composing(unit: u16) -> bool {
    matches!(unit, COMBINING_DAKUTEN | COMBINING_HANDAKUTEN)
}

fn find<V: Copy>(table: &[(char, V)], unit: u16) -> Option<V> {
    table
        .iter()
        .find(|(key, _)| *key as u32 == u32::from(unit))
        .map(|&(_, value)| value)
}

/// Kotlin's `Char.toString()` of a kana from the tables, all in the Basic Multilingual Plane.
fn units_of(c: char) -> Vec<u16> {
    vec![c as u16]
}

/// `unit` without its mark or small form.
fn base(unit: u16) -> u16 {
    find(&REVERSE_DAKU, unit)
        .or_else(|| find(&REVERSE_HANDAKU, unit))
        .or_else(|| find(&REVERSE_SMALL, unit))
        .map_or(unit, |c| c as u16)
}

/// One of the marks: what it makes of a kana, what takes it off, and whether a mark with nothing
/// to apply to is typed.
struct Mark {
    apply: fn(u16) -> Option<Vec<u16>>,
    remove: fn(u16) -> Option<u16>,
    typed_alone: bool,
}

const VOICING: Mark = Mark {
    apply: |unit| find(&DAKU, unit).map(units_of),
    remove: |unit| find(&REVERSE_DAKU, unit).map(|c| c as u16),
    typed_alone: true,
};

const HALF_VOICING: Mark = Mark {
    apply: |unit| find(&HANDAKU, unit).map(units_of),
    remove: |unit| find(&REVERSE_HANDAKU, unit).map(|c| c as u16),
    typed_alone: true,
};

const SMALL_FORM: Mark = Mark {
    apply: |unit| find(&SMALL, unit).map(|s: &str| s.encode_utf16().collect()),
    remove: |unit| find(&REVERSE_SMALL, unit).map(|c| c as u16),
    typed_alone: false,
};

/// `KanaUnicode.handleTransform`: the mark `c` typed after `last`.
fn transform(last: u16, c: u16, mark: &Mark, sticky: bool) -> Action {
    let applied = || (mark.apply)(base(last));
    let made = if sticky {
        applied()
    } else {
        (mark.remove)(last).map(|unit| vec![unit]).or_else(applied)
    };
    if let Some(made) = made {
        Action::new(1, made)
    } else if is_composing(last) && is_composing(c) {
        Action::new(
            1,
            if last == c && !sticky {
                vec![]
            } else {
                vec![c]
            },
        )
    } else {
        Action::new(0, if mark.typed_alone { vec![c] } else { vec![] })
    }
}

/// `KanaUnicode.getActions`: what typing `to_insert` after `preceding` does.
pub fn actions(preceding: &[u16], to_insert: &[u16], sticky: bool) -> Option<Action> {
    let &c = to_insert.first()?;
    let Some(&last) = preceding.last() else {
        return (c == SMALL_SENTINEL || is_composing(c)).then(|| Action::new(0, vec![]));
    };
    let mark = if is_dakuten(c) {
        VOICING
    } else if is_handakuten(c) {
        HALF_VOICING
    } else if c == SMALL_SENTINEL {
        SMALL_FORM
    } else {
        return None;
    };
    Some(transform(last, c, &mark, sticky))
}

#[cfg(test)]
mod tests;
