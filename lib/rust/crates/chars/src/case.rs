//! Java's `String` case mapping, which Kotlin's `lowercase()`, `uppercase()` and the
//! `ignoreCase` comparisons (`equals`, `startsWith`) call. Locale.US and Locale.ROOT, the only
//! ones the keyboard asks for, map alike.
//!
//! Each platform does it its own way, so each has its own module here, and the library takes the
//! one it is built for:
//!
//! - **Android** (libcore's `CaseMapper` and `String.regionMatches`) maps a string char by char,
//!   unless it holds İ or a high surrogate, which it hands to ICU to map code point by code point.
//!   The two differ only for Σ, whose small form is ς at the end of a word: char by char, Σ ends a
//!   word when the char before it is cased and the one after isn't; ICU looks past case-ignorable
//!   characters (apostrophes, accents) on either side. Its `ignoreCase` comparisons compare char
//!   by char, so a surrogate pair never matches another case of itself.
//! - **The JVM** maps code point by code point, and its comparisons compare code points.
//!   It decides whether Σ ends a word with its own word-break iterator, which this doesn't copy:
//!   the JVM build follows ICU's rule instead, so the JVM tests leave Σ out of lower-casing.
//!
//! Every other character maps the same way on both, from the tables: one code point to its
//! simple case (`Character.toUpperCase`), except the few whose full case is longer (ß to SS, İ to
//! i and a combining dot). tools/JavaChars.java checks that Android's two ways map every BMP
//! character alike, Σ aside.

use crate::code_point::{is_case_ignorable, is_cased, push_utf16};
use crate::tables::Tables;
use crate::text::{code_point_at, code_point_before, code_points};

#[cfg(any(test, target_os = "android"))]
pub(crate) mod android;
#[cfg(any(test, not(target_os = "android")))]
pub(crate) mod jvm;

#[cfg(target_os = "android")]
use android as platform;
#[cfg(not(target_os = "android"))]
use jvm as platform;

const CAPITAL_SIGMA: u16 = 0x03A3;
const SMALL_SIGMA: u16 = 0x03C3;
const FINAL_SIGMA: u16 = 0x03C2;

/// Kotlin's `String.lowercase()`.
pub fn lowercase(text: &[u16]) -> Vec<u16> {
    platform::lowercase(&crate::TABLES, text)
}

/// Kotlin's `String.uppercase()`. Both platforms map code point by code point.
pub fn uppercase(text: &[u16]) -> Vec<u16> {
    uppercase_with(&crate::TABLES, text)
}

/// Kotlin's `a.equals(b, ignoreCase = true)`.
pub fn equals_ignore_case(a: &[u16], b: &[u16]) -> bool {
    a.len() == b.len() && platform::region_matches(&crate::TABLES, a, b, a.len())
}

/// Kotlin's `text.startsWith(prefix, ignoreCase = true)`.
pub fn starts_with_ignore_case(text: &[u16], prefix: &[u16]) -> bool {
    prefix.len() <= text.len()
        && platform::region_matches(&crate::TABLES, text, prefix, prefix.len())
}

fn uppercase_with(tables: &Tables, text: &[u16]) -> Vec<u16> {
    let mut out = Vec::with_capacity(text.len());
    for (_, code_point) in code_points(text) {
        match tables.full_upper(code_point) {
            Some(units) => out.extend_from_slice(units),
            None => push_utf16(&mut out, tables.upper(code_point)),
        }
    }
    out
}

/// Lower-cases code point by code point, as ICU and the JVM do; [`is_final`] decides Σ.
fn lowercase_by_code_point(tables: &Tables, text: &[u16]) -> Vec<u16> {
    let mut out = Vec::with_capacity(text.len());
    for (index, code_point) in code_points(text) {
        if code_point == u32::from(CAPITAL_SIGMA) {
            out.push(if is_final(tables, text, index) {
                FINAL_SIGMA
            } else {
                SMALL_SIGMA
            });
        } else if let Some(units) = tables.full_lower(code_point) {
            out.extend_from_slice(units);
        } else {
            push_utf16(&mut out, tables.lower(code_point));
        }
    }
    out
}

/// ICU's Final_Sigma (ucase.cpp): the Σ at `index` is preceded by a cased letter, and not
/// followed by one, looking past case-ignorable characters either way.
fn is_final(tables: &Tables, text: &[u16], index: usize) -> bool {
    let after = index + 1;
    let followed = first_uncased_ignorable(tables, after..text.len(), |i| code_point_at(text, i));
    let preceded =
        first_uncased_ignorable(tables, (0..index).rev(), |i| code_point_before(text, i + 1));
    !followed && preceded
}

/// Whether, walking `indices` a code point at a time with `read`, the first code point that isn't
/// case-ignorable is cased. `read` gives the code point at (or before) an index and its length.
fn first_uncased_ignorable(
    tables: &Tables,
    mut indices: impl Iterator<Item = usize>,
    read: impl Fn(usize) -> (u32, usize),
) -> bool {
    while let Some(index) = indices.next() {
        let (code_point, len) = read(index);
        if !is_case_ignorable(tables, code_point) {
            return is_cased(tables, code_point);
        }
        // Skip the rest of a surrogate pair.
        for _ in 1..len {
            indices.next();
        }
    }
    false
}

#[cfg(test)]
mod tests;
