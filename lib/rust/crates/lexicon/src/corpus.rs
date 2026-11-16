//! The corpus as sentences, one a line, split once and for all into training text and held-out
//! test text.
//!
//! A sentence goes to the test side when the hash of its *key* (its letters and digits, lower
//! case, single-spaced) falls in one bucket of [`TEST_SHARE`]. So the split needs no randomness
//! and no state: the same corpus always splits the same way, and two copies of a sentence that
//! differ only in case or punctuation land on the same side. Only the first copy of each key is
//! kept on either side, so a repeated line counts once and a test sentence never reappears in
//! training. (Near-duplicates are another matter: Tatoeba has many sentences that differ by one
//! word, "Tom is tall." and "Tom is tired.", and those can fall on both sides.)
//!
//! Test sentences are written in hash order, so the first N of them are a uniform sample.

use std::collections::HashSet;

/// One sentence in this many is held out for testing.
pub const TEST_SHARE: u64 = 100;
/// Longer lines are not sentences anyone types on a phone (tables, run-together paragraphs).
const MAX_CHARS: usize = 300;

/// A sentence as typed on a phone: curly quotes made straight, as the keyboard types them, and
/// surrounding space trimmed.
pub fn clean(line: &str) -> String {
    line.trim()
        .chars()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' | '\u{02BC}' => '\'',
            '\u{201C}' | '\u{201D}' => '"',
            '\u{00A0}' | '\t' => ' ',
            c => c,
        })
        .collect()
}

/// What makes two sentences the same: their letters and digits, lower case, single-spaced.
pub fn key(sentence: &str) -> String {
    let mut key = String::with_capacity(sentence.len());
    let mut space = false;
    for c in sentence.chars() {
        if c.is_alphanumeric() {
            if space && !key.is_empty() {
                key.push(' ');
            }
            space = false;
            key.extend(c.to_lowercase());
        } else {
            space = true;
        }
    }
    key
}

/// A stable 64-bit hash: FNV-1a, then SplitMix64's finaliser so every bit depends on every byte.
pub fn hash(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in text.as_bytes() {
        h ^= u64::from(byte);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h = (h ^ (h >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    h ^ (h >> 31)
}

/// One source's sentences, split.
#[derive(Debug, Default)]
pub struct Split {
    pub train: Vec<String>,
    /// Held-out sentences in hash order.
    pub test: Vec<String>,
    /// Lines dropped as empty, too long or repeated.
    pub dropped: usize,
}

/// Splits `lines`, dropping any whose key is in `seen` (which they are then added to), so a
/// sentence in two sources is kept once.
pub fn split<'a>(lines: impl Iterator<Item = &'a str>, seen: &mut HashSet<u64>) -> Split {
    let mut out = Split::default();
    let mut test = Vec::new();
    for line in lines {
        let sentence = clean(line);
        let key = key(&sentence);
        if key.is_empty() || sentence.chars().count() > MAX_CHARS {
            out.dropped += 1;
            continue;
        }
        let h = hash(&key);
        if !seen.insert(h) {
            out.dropped += 1;
            continue;
        }
        if h.is_multiple_of(TEST_SHARE) {
            test.push((h, sentence));
        } else {
            out.train.push(sentence);
        }
    }
    test.sort_unstable_by_key(|&(h, _)| h);
    out.test = test.into_iter().map(|(_, sentence)| sentence).collect();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentences_that_differ_only_in_case_or_punctuation_are_the_same() {
        assert_eq!(key("  Don't go -- \"Tom\"!"), "don t go tom");
        assert_eq!(key("DON'T GO, TOM."), key("Don't go -- \"Tom\"!"));
        assert_ne!(hash("a"), hash("b"));
        assert_eq!(
            clean("\u{2018}Don\u{2019}t\u{2019} \u{201C}go\u{201D} "),
            "'Don't' \"go\""
        );
    }

    #[test]
    fn holds_out_one_in_a_hundred_and_keeps_each_sentence_once() {
        let lines: Vec<String> = (0..20_000)
            .map(|i| format!("Sentence number {i}."))
            .collect();
        let mut seen = HashSet::new();
        let split = split(lines.iter().map(String::as_str), &mut seen);
        assert!(
            (150..250).contains(&split.test.len()),
            "{}",
            split.test.len()
        );
        assert_eq!(split.train.len() + split.test.len(), 20_000);
        let again = super::split(
            ["SENTENCE NUMBER 7", "", "x".repeat(400).as_str()].into_iter(),
            &mut seen,
        );
        assert_eq!(
            (again.train.len(), again.test.len(), again.dropped),
            (0, 0, 3)
        );
    }

    #[test]
    fn test_sentences_come_in_hash_order() {
        let lines: Vec<String> = (0..20_000).map(|i| format!("Line {i}")).collect();
        let split = split(lines.iter().map(String::as_str), &mut HashSet::new());
        let hashes: Vec<u64> = split.test.iter().map(|s| hash(&key(s))).collect();
        assert!(hashes.windows(2).all(|w| w[0] <= w[1]));
    }
}
