//! Editing the GIF search field, as Kotlin's `SearchText` had it.

use chars::code_point::{is_high_surrogate, is_low_surrogate};
use chars::{is_letter, is_whitespace};

/// The longest query, in UTF-16 units: GIPHY's limit.
pub const MAX_LENGTH: usize = 50;

/// The shortest and longest word offered as a search, in UTF-16 units.
const MIN_WORD: usize = 2;
const MAX_WORD: usize = 30;

const APOSTROPHE: u16 = b'\'' as u16;
const SPACE: u16 = b' ' as u16;

/// `query` with `text` typed after it, cut at [`MAX_LENGTH`] units (which may split an emoji, as
/// Kotlin's `take` does).
pub fn type_text(query: &[u16], text: &[u16]) -> Vec<u16> {
    let mut typed = [query, text].concat();
    typed.truncate(MAX_LENGTH);
    typed
}

/// A glide-typed `word`, spaced from the one before unless the query is empty or ends in a space.
pub fn type_word(query: &[u16], word: &[u16]) -> Vec<u16> {
    let spaced = query.is_empty() || query.last() == Some(&SPACE);
    let separator: &[u16] = if spaced { &[] } else { &[SPACE] };
    type_text(query, &[separator, word].concat())
}

/// `query` without its last `count` characters, an emoji (a surrogate pair) counting as one.
/// Nothing is deleted for a `count` below 1.
pub fn delete_last(query: &[u16], count: i32) -> &[u16] {
    let mut left = query;
    for _ in 0..count.max(0) {
        let end = match left {
            [] => break,
            [.., high, low] if is_high_surrogate(*high) && is_low_surrogate(*low) => 2,
            _ => 1,
        };
        left = &left[..left.len() - end];
    }
    left
}

/// `query` without its last `count` words and the spaces after them; the space before stays.
pub fn delete_words(query: &[u16], count: i32) -> &[u16] {
    let mut left = query;
    for _ in 0..count.max(0) {
        let trimmed = trim_end(left, is_whitespace);
        let kept = trimmed.iter().rposition(|&it| is_whitespace(it));
        left = &trimmed[..kept.map_or(0, |it| it + 1)];
    }
    left
}

/// The word just typed before the cursor, to offer as a search: "congrats" from "Big congrats! ".
/// `None` when `text` doesn't end in a word of 2 to 30 letters (apostrophes inside count).
pub fn last_word(text: &[u16]) -> Option<&[u16]> {
    let end = trim_end(text, |it| is_whitespace(it) || is_closing(it));
    let start = end
        .iter()
        .rposition(|&it| !(is_letter(it) || it == APOSTROPHE))
        .map_or(0, |it| it + 1);
    let word = trim_end(&end[start..], |it| it == APOSTROPHE);
    let first = word
        .iter()
        .position(|&it| it != APOSTROPHE)
        .unwrap_or(word.len());
    let word = &word[first..];
    (MIN_WORD..=MAX_WORD).contains(&word.len()).then_some(word)
}

/// One of the marks a sentence ends a word with: `.,!?;:`.
fn is_closing(unit: u16) -> bool {
    u8::try_from(unit).is_ok_and(|it| b".,!?;:".contains(&it))
}

fn trim_end(text: &[u16], drop: impl Fn(u16) -> bool) -> &[u16] {
    let end = text
        .iter()
        .rposition(|&it| !drop(it))
        .map_or(0, |it| it + 1);
    &text[..end]
}

#[cfg(test)]
mod tests;
