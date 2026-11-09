//! LatinIME's character rules (`utils/char_utils`): lower case and base letters by its own tables,
//! not Unicode's, so the decoder matches letters exactly as the C++ did.
//!
//! Code points are `i32`, as the C++ has them: [`NOT_A_CODE_POINT`](crate::defines) and the
//! beginning-of-sentence marker (U+110000) travel in the same arrays as real characters.

mod tables;

use crate::defines::{
    CODE_POINT_BEGINNING_OF_SENTENCE, KEYCODE_HYPHEN_MINUS, KEYCODE_SINGLE_QUOTE, KEYCODE_SPACE,
};

const MIN_UNICODE_CODE_POINT: i32 = 0;
const MAX_UNICODE_CODE_POINT: i32 = 0x10FFFF;

/// `CharUtils::isAsciiUpper`: A to Z only.
pub fn is_ascii_upper(c: i32) -> bool {
    (i32::from(b'A')..=i32::from(b'Z')).contains(&c)
}

/// `isascii`: 0 to 0x7F.
fn is_ascii(c: i32) -> bool {
    c & !0x7F == 0
}

/// `CharUtils::latin_tolower`. The C++ searched its table with the code point's low 16 bits (it
/// read an `int` through an `unsigned short` pointer), so a code point above U+FFFF whose low bits
/// are a listed capital comes back as that capital's small letter. Kept, as the decoder did it.
fn latin_tolower(c: i32) -> i32 {
    let key = c as u16;
    match tables::LOWER_CASE.binary_search_by_key(&key, |&(capital, _)| capital) {
        Ok(at) => i32::from(tables::LOWER_CASE[at].1),
        Err(_) => c,
    }
}

/// `CharUtils::toLowerCase`.
pub fn to_lower_case(c: i32) -> i32 {
    if is_ascii_upper(c) {
        return c - i32::from(b'A') + i32::from(b'a');
    }
    if is_ascii(c) {
        return c;
    }
    latin_tolower(c)
}

/// `CharUtils::toBaseCodePoint`: the letter without its accent, below U+0500. The C++ indexed its
/// table with a negative code point too, reading outside it; here a negative one is itself.
pub fn to_base_code_point(c: i32) -> i32 {
    match usize::try_from(c) {
        Ok(at) if at < tables::BASE_CHARS.len() => i32::from(tables::BASE_CHARS[at]),
        _ => c,
    }
}

/// `CharUtils::toBaseLowerCase`.
pub fn to_base_lower_case(c: i32) -> i32 {
    to_lower_case(to_base_code_point(c))
}

/// `CharUtils::isIntentionalOmissionCodePoint`: an apostrophe or a hyphen, which a typist may
/// leave out.
pub fn is_intentional_omission_code_point(c: i32) -> bool {
    c == KEYCODE_SINGLE_QUOTE || c == KEYCODE_HYPHEN_MINUS
}

/// `CharUtils::getCodePointCount`: the length up to the first 0, within `code_points`.
pub fn code_point_count(code_points: &[i32]) -> usize {
    code_points
        .iter()
        .position(|&it| it == 0)
        .unwrap_or(code_points.len())
}

/// `CharUtils::getSpaceCount`.
pub fn space_count(code_points: &[i32]) -> i32 {
    code_points
        .iter()
        .filter(|&&it| it == KEYCODE_SPACE)
        .count() as i32
}

/// `CharUtils::isInUnicodeSpace`.
pub fn is_in_unicode_space(c: i32) -> bool {
    (MIN_UNICODE_CODE_POINT..=MAX_UNICODE_CODE_POINT).contains(&c)
}

/// `CharUtils::attachBeginningOfSentenceMarker`: puts the marker before the first `count` code
/// points unless it is there already, and returns the new count; 0 if `max` code points have no
/// room for it. `code_points` holds at least `count + 1` when the marker is added.
pub fn attach_beginning_of_sentence_marker(
    code_points: &mut [i32],
    count: usize,
    max: usize,
) -> usize {
    if count > 0 && code_points[0] == CODE_POINT_BEGINNING_OF_SENTENCE {
        return count;
    }
    if count >= max {
        return 0;
    }
    code_points.copy_within(0..count, 1);
    code_points[0] = CODE_POINT_BEGINNING_OF_SENTENCE;
    count + 1
}

/// `CharUtils::removeBeginningOfSentenceMarker`: takes a leading marker off the first `count`
/// code points and returns the new count.
pub fn remove_beginning_of_sentence_marker(code_points: &mut [i32], count: usize) -> usize {
    if count == 0 || code_points[0] != CODE_POINT_BEGINNING_OF_SENTENCE {
        return count;
    }
    code_points.copy_within(1..count, 0);
    count - 1
}

#[cfg(test)]
mod tests;
