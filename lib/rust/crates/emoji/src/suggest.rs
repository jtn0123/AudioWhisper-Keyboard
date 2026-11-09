//! Emoji suggestions, as `EmojiSuggestionProvider` made them: which typed text is a query
//! ([`validate_query`]), how well each emoji matches it ([`score`]), and the best few
//! ([`suggest`]).

use std::cmp::Ordering;

use crate::data::{EmojiData, Entry};

/// The query in the composing text, `validateInputQuery`: the text after `prefix` (`:` for
/// suggestions after a leading colon, or nothing), if the text starts with the prefix, is at least
/// `min_length` plus the prefix long (Kotlin `Int`s, so the sum wraps), and is only ASCII letters
/// after it (`^[A-Za-z]*$`, so no line end either).
pub fn validate_query<'a>(
    composing: &'a [u16],
    prefix: &[u16],
    min_length: i32,
) -> Option<&'a [u16]> {
    // Lengths of Kotlin strings fit in an Int.
    let query_min_length = min_length.wrapping_add(prefix.len() as i32);
    if !prefix.is_empty() && !composing.starts_with(prefix) {
        return None;
    }
    if (composing.len() as i32) < query_min_length {
        return None;
    }
    let partial = &composing[prefix.len()..];
    partial
        .iter()
        .all(|&u| u8::try_from(u).is_ok_and(|b| b.is_ascii_alphabetic()))
        .then_some(partial)
}

/// Kotlin's `text.contains(other, ignoreCase = true)`: `other.regionMatches(0, text, index, ...)`
/// at some index from 0 to `text`'s length, each Java's `regionMatches(true, ...)` (`chars::case`:
/// char by char on Android, code point by code point on the JVM).
pub fn contains_ignore_case(text: &[u16], other: &[u16]) -> bool {
    (0..=text.len()).any(|index| chars::starts_with_ignore_case(&text[index..], other))
}

/// How well an emoji matches `query`: 0.7 times the share of its name the query is, if the name
/// contains it, plus 0.3 if a keyword contains it (`containsWeighted`). An empty query in an
/// empty name is 0 / 0, NaN, as in Kotlin.
pub fn score(data: &EmojiData, entry: &Entry, query: &[u16]) -> f64 {
    let name = data.units(entry.name);
    let name_weight = if contains_ignore_case(name, query) {
        query.len() as f64 / name.len() as f64
    } else {
        0.0
    };
    let keyword_weight = if entry
        .keywords
        .iter()
        .any(|&keyword| contains_ignore_case(data.units(keyword), query))
    {
        1.0
    } else {
        0.0
    };
    name_weight * 0.7 + keyword_weight * 0.3
}

/// Java's `Double.compare`: NaN above everything (all NaNs alike), -0 below +0. Rust's
/// `total_cmp` differs: it puts a NaN with its sign bit set below everything, and 0.0 / 0.0 has
/// it set on x86.
fn java_compare(a: f64, b: f64) -> Ordering {
    let bits = |d: f64| {
        if d.is_nan() {
            0x7ff8_0000_0000_0000_i64
        } else {
            d.to_bits() as i64
        }
    };
    if a < b {
        Ordering::Less
    } else if a > b {
        Ordering::Greater
    } else {
        bits(a).cmp(&bits(b))
    }
}

/// The emoji for `query` among those shown for `skin_tone`, as positions in
/// [`EmojiData::by_skin_tone`]'s list for it: sorted by [`score`], highest first (stably, so
/// ties keep the list's order), the first `max` taken, and of those the ones scoring above 0.
///
/// Kotlin sorted every emoji; this sorts only those not scoring 0, which is the same, as none
/// scores below 0: a 0 sorts after every other score, so it can only be taken where the ones above
/// it run out, and is then dropped. A NaN sorts first, takes a place, and is dropped.
pub fn suggest(data: &EmojiData, skin_tone: usize, query: &[u16], max: usize) -> Vec<usize> {
    let mut scored: Vec<(usize, f64)> = data.by_skin_tone[skin_tone]
        .iter()
        .enumerate()
        .map(|(position, &entry)| (position, score(data, &data.entries[entry], query)))
        .filter(|&(_, score)| score != 0.0)
        .collect();
    scored.sort_by(|&(_, a), &(_, b)| java_compare(b, a));
    scored
        .into_iter()
        .take(max)
        .filter(|&(_, score)| score > 0.0)
        .map(|(position, _)| position)
        .collect()
}

#[cfg(test)]
mod tests;
