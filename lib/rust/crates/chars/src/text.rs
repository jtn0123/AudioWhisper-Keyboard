//! Kotlin `String` functions that read code points or numbers, as Java does.

use crate::code_point::{self, from_surrogates, is_high_surrogate, is_low_surrogate};

/// Java's `codePointAt(index)` and how many units it spans: a high surrogate followed by a low
/// one is one code point; any other unit, a lone surrogate included, is one by itself.
pub fn code_point_at(text: &[u16], index: usize) -> (u32, usize) {
    let unit = text[index];
    match text.get(index + 1) {
        Some(&low) if is_high_surrogate(unit) && is_low_surrogate(low) => {
            (from_surrogates(unit, low), 2)
        }
        _ => (u32::from(unit), 1),
    }
}

/// Java's `codePointBefore(index)` and how many units it spans.
pub fn code_point_before(text: &[u16], index: usize) -> (u32, usize) {
    let unit = text[index - 1];
    match index.checked_sub(2).map(|i| text[i]) {
        Some(high) if is_low_surrogate(unit) && is_high_surrogate(high) => {
            (from_surrogates(high, unit), 2)
        }
        _ => (u32::from(unit), 1),
    }
}

/// Java's `codePoints()`: each code point with the index of its first unit.
pub fn code_points(text: &[u16]) -> impl Iterator<Item = (usize, u32)> + '_ {
    let mut index = 0;
    std::iter::from_fn(move || {
        let start = index;
        let (code_point, len) = (start < text.len()).then(|| code_point_at(text, start))?;
        index += len;
        Some((start, code_point))
    })
}

/// Java's `codePointCount(0, length)`.
pub fn code_point_count(text: &[u16]) -> usize {
    code_points(text).count()
}

/// How many units the first code point spans (`offsetByCodePoints(0, 1)`); 0 for empty text.
pub fn first_code_point_len(text: &[u16]) -> usize {
    if text.is_empty() {
        0
    } else {
        code_point_at(text, 0).1
    }
}

/// Kotlin's `String.toIntOrNull()`: an optional `-` or `+`, then decimal digits of any script
/// (`Character.digit`, so ٣ counts), and nothing else; `None` past `Int`'s range.
pub fn to_int_or_null(text: &[u16]) -> Option<i32> {
    let (&first, rest) = text.split_first()?;
    let (negative, digits) = match first {
        // Kotlin only looks for a sign below '0', and wants a digit after it.
        _ if first >= u16::from(b'0') => (false, text),
        _ if rest.is_empty() => return None,
        0x2D => (true, rest),
        0x2B => (false, rest),
        _ => return None,
    };
    let mut value: i64 = 0;
    for &unit in digits {
        let digit = code_point::digit(u32::from(unit))?;
        value = value * 10 + i64::from(digit);
        if value > i64::from(i32::MAX) + 1 {
            return None;
        }
    }
    let value = if negative { -value } else { value };
    i32::try_from(value).ok()
}
