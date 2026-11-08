//! The JVM's case mapping: `StringUTF16` and `StringLatin1`, which agree.

use super::lowercase_by_code_point;
use crate::code_point::{from_surrogates, is_high_surrogate, is_low_surrogate, is_surrogate};
use crate::tables::Tables;

/// `String.toLowerCase`, code point by code point. Σ follows ICU's rule, not the JVM's (see
/// the module above).
pub(crate) fn lowercase(tables: &Tables, text: &[u16]) -> Vec<u16> {
    lowercase_by_code_point(tables, text)
}

/// `String.regionMatches(true, 0, other, 0, len)`, bounds already checked: `StringUTF16`'s
/// `compareToCIImpl`. Units that differ are compared again as the code points they are part of.
pub(crate) fn region_matches(tables: &Tables, text: &[u16], other: &[u16], len: usize) -> bool {
    let (mut k1, mut k2) = (0, 0);
    while k1 < len && k2 < len {
        let (c1, c2) = (text[k1], other[k2]);
        if c1 != c2 && !same_ignoring_case(tables, u32::from(c1), u32::from(c2)) {
            let (cp1, pair1) = code_point_including(text, k1, len);
            let (cp2, pair2) = code_point_including(other, k2, len);
            k1 += usize::from(pair1);
            k2 += usize::from(pair2);
            if !same_ignoring_case(tables, cp1, cp2) {
                return false;
            }
        }
        k1 += 1;
        k2 += 1;
    }
    true
}

/// `compareCodePointCI(...) == 0`: equal in upper case, or in the lower case of that.
fn same_ignoring_case(tables: &Tables, cp1: u32, cp2: u32) -> bool {
    let (u1, u2) = (tables.upper(cp1), tables.upper(cp2));
    u1 == u2 || tables.lower(u1) == tables.lower(u2)
}

/// `codePointIncluding`: the code point the unit at `index` is part of, with a low surrogate
/// joined to the high one before it and a high one to the low one after it (within `end`), and
/// whether it was the latter, which moves the walk on a unit.
fn code_point_including(text: &[u16], index: usize, end: usize) -> (u32, bool) {
    let unit = text[index];
    if !is_surrogate(unit) {
        return (u32::from(unit), false);
    }
    if is_low_surrogate(unit) {
        if index > 0 && is_high_surrogate(text[index - 1]) {
            return (from_surrogates(text[index - 1], unit), false);
        }
    } else if index + 1 < end && is_low_surrogate(text[index + 1]) {
        return (from_surrogates(unit, text[index + 1]), true);
    }
    (u32::from(unit), false)
}
