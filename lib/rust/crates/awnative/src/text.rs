//! Kotlin's character rules as the `chars` crate has them, for `JavaTextNative`, whose generated
//! entry points call these or the crate directly. Nothing in the keyboard calls these; they are
//! there so tests can hold `chars` up against `java.lang.String` and `Character`, on a computer and
//! on a phone.

// The bits of `unitFacts`, which `JavaText.Unit` reads: the predicates, then the cases and the
// first code point of the NFD.
const LETTER: i64 = 1;
const DIGIT: i64 = 2;
const LETTER_OR_DIGIT: i64 = 4;
const UPPER_CASE: i64 = 8;
const LOWER_CASE: i64 = 16;
const TITLE_CASE: i64 = 32;
const WHITESPACE: i64 = 64;
const REGEX_SPACE: i64 = 128;
const UPPERCASE_SHIFT: u32 = 8;
const LOWERCASE_SHIFT: u32 = 24;
const DECOMPOSED_SHIFT: u32 = 40;

/// What `toIntOrNull` returns for null: no `Int` is `Long.MIN_VALUE`.
const NO_INT: i64 = i64::MIN;

/// `unit`'s `Char` predicates, as bits, then its `uppercaseChar()` from bit 8, its
/// `lowercaseChar()` from bit 24 and the first code point of its NFD from bit 40.
pub fn unit_facts(unit: u16) -> i64 {
    let predicates = [
        (chars::is_letter as fn(u16) -> bool, LETTER),
        (chars::is_digit, DIGIT),
        (chars::is_letter_or_digit, LETTER_OR_DIGIT),
        (chars::is_upper_case, UPPER_CASE),
        (chars::is_lower_case, LOWER_CASE),
        (chars::is_title_case, TITLE_CASE),
        (chars::is_whitespace, WHITESPACE),
        (chars::is_regex_space, REGEX_SPACE),
    ];
    let bits: i64 = predicates
        .iter()
        .filter(|(test, _)| test(unit))
        .map(|(_, bit)| bit)
        .sum();
    // A code point is at most 0x10FFFF: 21 bits, which fit above the cases.
    let decomposed = i64::from(chars::code_point::decomposed_first(unit));
    bits | i64::from(chars::to_upper(unit)) << UPPERCASE_SHIFT
        | i64::from(chars::to_lower(unit)) << LOWERCASE_SHIFT
        | decomposed << DECOMPOSED_SHIFT
}

/// `Character.isDigit(codePoint)` as 0 or 1, then `Character.digit(codePoint, 10)`,
/// `Character.toUpperCase(codePoint)` and `Character.toLowerCase(codePoint)`.
pub fn code_point_facts(code_point: i32) -> Vec<i32> {
    use chars::code_point::{digit, is_digit, to_lower, to_upper};
    // Java answers a negative Int as no character at all.
    match u32::try_from(code_point) {
        // A code point is at most 0x10FFFF, so it fits in an Int.
        Ok(cp) => vec![
            i32::from(is_digit(cp)),
            digit(cp).map_or(-1, i32::from),
            to_upper(cp) as i32,
            to_lower(cp) as i32,
        ],
        Err(_) => vec![0, -1, code_point, code_point],
    }
}

/// Kotlin's `text.trim()`.
pub fn trim(text: &[u16]) -> Vec<u16> {
    chars::trim(text).to_vec()
}

/// Kotlin's `text.codePointCount(0, text.length)`.
pub fn code_point_count(text: &[u16]) -> i32 {
    // There are no more code points than units, whose count is an Int.
    chars::text::code_point_count(text) as i32
}

/// Kotlin's `text.toIntOrNull()`, with `Long.MIN_VALUE` for null.
pub fn to_int_or_null(text: &[u16]) -> i64 {
    chars::text::to_int_or_null(text).map_or(NO_INT, i64::from)
}
