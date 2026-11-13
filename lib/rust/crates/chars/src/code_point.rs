//! Java's `Character` functions that take a code point (an `Int`), as Kotlin code calls them
//! directly (`Character.isDigit(codePoint)`), and UTF-16's surrogates.

use crate::tables::TABLES;
use crate::{CASE_IGNORABLE, CASED, DIGIT, LETTER};

/// The highest code point, U+10FFFF.
pub const MAX: u32 = 0x10_FFFF;

const HANGUL_FIRST: u32 = 0xAC00;
const HANGUL_LAST: u32 = 0xD7A3;
const HANGUL_LEADING_FIRST: u32 = 0x1100;
/// How many Hangul syllables share a leading consonant: 21 vowels times 28 trailing consonants.
const HANGUL_PER_LEADING: u32 = 588;

/// `Character.isDigit(codePoint)`: general category Nd.
pub fn is_digit(code_point: u32) -> bool {
    TABLES.flags(code_point) & DIGIT != 0
}

/// `Character.isLetter(codePoint)`: Unicode general category L, including supplementary letters.
pub fn is_letter(code_point: u32) -> bool {
    TABLES.flags(code_point) & LETTER != 0
}

/// `Character.toUpperCase(codePoint)`: one code point to one.
pub fn to_upper(code_point: u32) -> u32 {
    TABLES.upper(code_point)
}

/// `Character.toLowerCase(codePoint)`: one code point to one.
pub fn to_lower(code_point: u32) -> u32 {
    TABLES.lower(code_point)
}

/// `Character.digit(codePoint, 10)`: a decimal digit's value (as for ٣ or ９), or `None`.
pub fn digit(code_point: u32) -> Option<u8> {
    TABLES.digit(code_point)
}

/// The first code point of `Normalizer.normalize(unit, NFD)`: the base letter of an accented
/// one (e for é), or the unit itself when it doesn't decompose.
pub fn decomposed_first(unit: u16) -> u32 {
    let code_point = u32::from(unit);
    if (HANGUL_FIRST..=HANGUL_LAST).contains(&code_point) {
        HANGUL_LEADING_FIRST + (code_point - HANGUL_FIRST) / HANGUL_PER_LEADING
    } else {
        TABLES.decomposed(unit)
    }
}

pub(crate) fn is_cased(tables: &crate::tables::Tables, code_point: u32) -> bool {
    tables.flags(code_point) & CASED != 0
}

pub(crate) fn is_case_ignorable(tables: &crate::tables::Tables, code_point: u32) -> bool {
    tables.flags(code_point) & CASE_IGNORABLE != 0
}

/// Whether the unit is a high (leading) surrogate, U+D800 to U+DBFF.
pub fn is_high_surrogate(unit: u16) -> bool {
    (0xD800..=0xDBFF).contains(&unit)
}

/// Whether the unit is a low (trailing) surrogate, U+DC00 to U+DFFF.
pub fn is_low_surrogate(unit: u16) -> bool {
    (0xDC00..=0xDFFF).contains(&unit)
}

/// Whether the unit is either kind of surrogate.
pub fn is_surrogate(unit: u16) -> bool {
    (0xD800..=0xDFFF).contains(&unit)
}

/// `Character.toCodePoint(high, low)`.
pub fn from_surrogates(high: u16, low: u16) -> u32 {
    0x10000 + ((u32::from(high) - 0xD800) << 10) + (u32::from(low) - 0xDC00)
}

/// Appends `Character.toChars(codePoint)`: one unit, or a surrogate pair.
pub fn push_utf16(out: &mut Vec<u16>, code_point: u32) {
    match u16::try_from(code_point) {
        Ok(unit) => out.push(unit),
        Err(_) => {
            let offset = code_point - 0x10000;
            out.push(0xD800 + (offset >> 10) as u16);
            out.push(0xDC00 + (offset & 0x3FF) as u16);
        }
    }
}
