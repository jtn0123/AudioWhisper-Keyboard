//! Korean typed jamo by jamo, as FlorisBoard's `HangulUnicode` composed it: an initial consonant
//! and a vowel become a syllable, a consonant after a syllable becomes its final, two finals or
//! two vowels that combine do, and a vowel after a final takes that consonant to start the next
//! syllable. Only the last unit before the cursor is looked at.
//!
//! The jamo are Hangul Compatibility Jamo (U+3131 to U+318E), the syllables U+AC00 to U+D7A3,
//! numbered `initial * 588 + medial * 28 + final` from U+AC00.

use crate::Action;

/// Initial consonants, in syllable order.
const INITIALS: [char; 19] = [
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ',
    'ㅌ', 'ㅍ', 'ㅎ',
];

/// Medial vowels, in syllable order.
const MEDIALS: [char; 21] = [
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ',
    'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
];

/// Final consonants in syllable order, after `_`, which stands for none (and which, typed after
/// a syllable, is typed as it is).
const FINALS: [char; 28] = [
    '_', 'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ',
    'ㅁ', 'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
];

/// A vowel, the vowels it combines with, and what each combination makes.
const MEDIAL_COMPOSITIONS: [(char, &[char], &[char]); 3] = [
    ('ㅗ', &['ㅏ', 'ㅐ', 'ㅣ'], &['ㅘ', 'ㅙ', 'ㅚ']),
    ('ㅜ', &['ㅓ', 'ㅔ', 'ㅣ'], &['ㅝ', 'ㅞ', 'ㅟ']),
    ('ㅡ', &['ㅣ'], &['ㅢ']),
];

/// A final, the consonants it combines with, and what each combination makes.
const FINAL_COMPOSITIONS: [(char, &[char], &[char]); 4] = [
    ('ㄱ', &['ㅅ'], &['ㄳ']),
    ('ㄴ', &['ㅈ', 'ㅎ'], &['ㄵ', 'ㄶ']),
    (
        'ㄹ',
        &['ㄱ', 'ㅁ', 'ㅂ', 'ㅅ', 'ㅌ', 'ㅍ', 'ㅎ'],
        &['ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ'],
    ),
    ('ㅂ', &['ㅅ'], &['ㅄ']),
];

const FIRST_SYLLABLE: u16 = 0xAC00;
const LAST_SYLLABLE: u16 = 0xD7A3;
const PER_INITIAL: u16 = 588;
const PER_MEDIAL: u16 = 28;

/// Where `unit` is in `list`, as Kotlin's `String.indexOf(Char)`.
fn index_of(list: &[char], unit: u16) -> Option<usize> {
    list.iter().position(|&c| c as u32 == u32::from(unit))
}

/// The jamo `list` holds at `index`, which a syllable's numbering keeps in range.
fn at(list: &[char], index: usize) -> u16 {
    // Every jamo is in the Basic Multilingual Plane, so its code is one unit.
    list[index] as u16
}

/// What `unit` combines with and into, if it combines at all.
fn compositions(
    table: &[(char, &'static [char], &'static [char])],
    unit: u16,
) -> Option<(&'static [char], &'static [char])> {
    table
        .iter()
        .find(|(first, _, _)| *first as u32 == u32::from(unit))
        .map(|&(_, seconds, made)| (seconds, made))
}

/// What `first` and `second` combine into in `table`, if they do.
fn compose(
    table: &[(char, &'static [char], &'static [char])],
    first: u16,
    second: u16,
) -> Option<u16> {
    let (seconds, made) = compositions(table, first)?;
    index_of(seconds, second).map(|i| at(made, i))
}

/// The two jamo a combined jamo of `table` was made of.
fn decompose(table: &[(char, &'static [char], &'static [char])], made: u16) -> Option<(u16, u16)> {
    table.iter().find_map(|&(first, seconds, mades)| {
        index_of(mades, made).map(|i| (first as u16, at(seconds, i)))
    })
}

fn syllable(initial: usize, medial: usize, last: usize) -> u16 {
    let number = initial * usize::from(PER_INITIAL) + medial * usize::from(PER_MEDIAL) + last;
    // The indices come from the tables, so the number stays within the syllables.
    FIRST_SYLLABLE + number as u16
}

/// A syllable's initial, medial and final.
fn blocks(syllable: u16) -> (usize, usize, usize) {
    let offset = syllable - FIRST_SYLLABLE;
    let initial = offset / PER_INITIAL;
    let medial = (offset - initial * PER_INITIAL) / PER_MEDIAL;
    let last = offset % PER_MEDIAL;
    (usize::from(initial), usize::from(medial), usize::from(last))
}

fn replace(units: &[u16]) -> Option<Action> {
    Some(Action::new(1, units.to_vec()))
}

/// `HangulUnicode.getActions`: what typing `to_insert` after `preceding` does.
pub fn actions(preceding: &[u16], to_insert: &[u16]) -> Option<Action> {
    let (&last, &c) = (preceding.last()?, to_insert.first()?);
    if let (Some(initial), Some(medial)) = (index_of(&INITIALS, last), index_of(&MEDIALS, c)) {
        replace(&[syllable(initial, medial, 0)])
    } else if (FIRST_SYLLABLE..=LAST_SYLLABLE).contains(&last) {
        after_syllable(last, c)
    } else if let Some(vowel) = compose(&MEDIAL_COMPOSITIONS, last, c) {
        replace(&[vowel])
    } else if let Some(consonant) = compose(&FINAL_COMPOSITIONS, last, c) {
        replace(&[consonant])
    } else {
        None
    }
}

/// Typing `c` after the syllable `last`.
fn after_syllable(last: u16, c: u16) -> Option<Action> {
    let (initial, medial, fin) = blocks(last);
    if c == at(&FINALS, 0) {
        return None;
    }
    if fin == 0
        && let Some(new_final) = index_of(&FINALS, c)
    {
        return replace(&[syllable(initial, medial, new_final)]);
    }
    let old_final = at(&FINALS, fin);
    if let Some(composed) = compose(&FINAL_COMPOSITIONS, old_final, c) {
        let composed = index_of(&FINALS, composed)?;
        return replace(&[syllable(initial, medial, composed)]);
    }
    let parts = decompose(&FINAL_COMPOSITIONS, old_final);
    if let Some(next_medial) = index_of(&MEDIALS, c) {
        match parts {
            None if fin != 0 => {
                let next_initial = index_of(&INITIALS, old_final)?;
                return replace(&[
                    syllable(initial, medial, 0),
                    syllable(next_initial, next_medial, 0),
                ]);
            }
            Some((kept, moved)) => {
                let kept = index_of(&FINALS, kept)?;
                let next_initial = index_of(&INITIALS, moved)?;
                return replace(&[
                    syllable(initial, medial, kept),
                    syllable(next_initial, next_medial, 0),
                ]);
            }
            None => {}
        }
    }
    // Kotlin also asked that the syllable have no final, but a vowel after a final has already
    // split it above.
    if let Some(vowel) = compose(&MEDIAL_COMPOSITIONS, at(&MEDIALS, medial), c) {
        let vowel = index_of(&MEDIALS, vowel)?;
        return replace(&[syllable(initial, vowel, 0)]);
    }
    None
}

#[cfg(test)]
mod tests;
