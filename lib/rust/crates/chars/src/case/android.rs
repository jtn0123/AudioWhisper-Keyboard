//! Android's case mapping: libcore's `CaseMapper` and `String.regionMatches`.

use super::{CAPITAL_SIGMA, FINAL_SIGMA, SMALL_SIGMA, lowercase_by_code_point};
use crate::code_point::is_high_surrogate;
use crate::tables::Tables;
use crate::{LOWER, TITLE, UPPER};

const DOTTED_CAPITAL_I: u16 = 0x0130;

/// `CaseMapper.toLowerCase`: ICU's way for a string with İ or a high surrogate, else char by char.
pub(crate) fn lowercase(tables: &Tables, text: &[u16]) -> Vec<u16> {
    if text
        .iter()
        .any(|&unit| unit == DOTTED_CAPITAL_I || is_high_surrogate(unit))
    {
        return lowercase_by_code_point(tables, text);
    }
    (0..text.len())
        .map(|i| match text[i] {
            CAPITAL_SIGMA if is_final_sigma(tables, text, i) => FINAL_SIGMA,
            CAPITAL_SIGMA => SMALL_SIGMA,
            unit => tables.lower(u32::from(unit)) as u16,
        })
        .collect()
}

/// `CaseMapper.isFinalSigma`: the char before is cased and the one after, if any, isn't. Cased is
/// `Character.isLowerCase`, `isUpperCase` or `isTitleCase`, which on Android are Ll, Lu and Lt.
fn is_final_sigma(tables: &Tables, text: &[u16], index: usize) -> bool {
    let cased = |unit: u16| tables.flags(u32::from(unit)) & (LOWER | UPPER | TITLE) != 0;
    index > 0 && cased(text[index - 1]) && text.get(index + 1).is_none_or(|&next| !cased(next))
}

/// `String.regionMatches(true, 0, other, 0, len)`, bounds already checked: char by char, each
/// pair equal, or equal in upper case, or in the lower case of that.
pub(crate) fn region_matches(tables: &Tables, text: &[u16], other: &[u16], len: usize) -> bool {
    let upper = |unit: u16| tables.upper(u32::from(unit)) as u16;
    let lower = |unit: u16| tables.lower(u32::from(unit)) as u16;
    text[..len].iter().zip(&other[..len]).all(|(&c1, &c2)| {
        let (u1, u2) = (upper(c1), upper(c2));
        c1 == c2 || u1 == u2 || lower(u1) == lower(u2)
    })
}
