//! The suggester's small rules about words and the strip, apart from the suggester itself.

use std::collections::HashSet;
use std::hash::Hash;

use chars::{equals_ignore_case, is_digit, is_letter, is_lower_case, is_upper_case};

use super::{Arranged, Ranked, TypingState};
use crate::word_case::is_mostly_caps;

/// The strip: the auto-correction, the typed word, then the `others`, each text once.
pub(super) fn arrange(
    auto_correction: Option<Vec<u16>>,
    typed: &[u16],
    others: impl Iterator<Item = Vec<u16>>,
) -> Vec<Arranged> {
    let auto_correction = auto_correction.map(|text| Arranged {
        text,
        is_auto_correction: true,
        is_typed_word: false,
    });
    let typed = Arranged {
        text: typed.to_vec(),
        is_auto_correction: false,
        is_typed_word: true,
    };
    let strip = auto_correction
        .into_iter()
        .chain([typed])
        .chain(others.map(Arranged::plain));
    distinct_by(strip, |it| it.text.clone())
}

/// Of one word's spellings, the first with no capital, else the first.
pub(super) fn lower_case_first(mut spellings: Vec<Ranked>) -> Ranked {
    let has_no_capital = |it: &Ranked| !it.display.iter().any(|&unit| is_upper_case(unit));
    let index = spellings.iter().position(has_no_capital).unwrap_or(0);
    spellings.swap_remove(index)
}

pub(super) fn is_acronym(word: &[u16]) -> bool {
    let letters = word.iter().filter(|&&unit| is_letter(unit));
    letters.clone().count() > 1 && !letters.clone().any(|&unit| is_lower_case(unit))
}

/// AOSP never auto-corrects mid-word, a word with digits, or a mostly-caps word.
pub(super) fn is_correctable(state: &TypingState) -> bool {
    !state.is_cursor_mid_word
        && !state.typed.iter().any(|&unit| is_digit(unit))
        && !is_mostly_caps(&state.typed)
}

pub(super) fn is_case_only_acronym(typed: &[u16], display: &[u16]) -> bool {
    equals_ignore_case(display, typed)
        && !display.iter().any(|&unit| is_lower_case(unit))
        && typed.iter().any(|&unit| is_lower_case(unit))
}

/// `items` without those whose `key` an earlier one had, as Kotlin's `distinctBy`.
pub(super) fn distinct_by<T, K: Hash + Eq>(
    items: impl IntoIterator<Item = T>,
    key: impl Fn(&T) -> K,
) -> Vec<T> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(key(item)))
        .collect()
}
