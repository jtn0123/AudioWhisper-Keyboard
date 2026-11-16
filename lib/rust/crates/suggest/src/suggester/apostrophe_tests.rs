//! The apostrophe rule ([`crate::apostrophes`]) in the suggester, over a dictionary whose language
//! model's odds for the apostrophe are set by the previous word.

use super::*;
use crate::auto_correction::Strength;
use crate::dictionary::Lookup;
use crate::raw::KIND_WHITELIST;
use crate::raw::{FLAG_APPROPRIATE_FOR_AUTO_CORRECTION, FLAG_EXACT_MATCH, KIND_CORRECTION};

const CORRECTION: i32 = KIND_CORRECTION | FLAG_APPROPRIATE_FOR_AUTO_CORRECTION;
const EXACT: i32 = CORRECTION | FLAG_EXACT_MATCH;

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

/// "ill" (whitelisted to "I'll") and "its" (a word, as is "it's"); "dont" is no word.
struct Pairs {
    previous: PreviousWords,
    /// The model's odds asked for: the word with the apostrophe, then the typed one.
    asked: Vec<(String, String)>,
}

impl Lookup for Pairs {
    type Error = NegativeCount;

    fn probability(&mut self, word: &[u16]) -> Result<i32, NegativeCount> {
        let known = ["ill", "I'll", "will", "its", "it's", "don't"];
        Ok(
            if known.contains(&String::from_utf16_lossy(word).as_str()) {
                150
            } else {
                NOT_A_PROBABILITY
            },
        )
    }
}

impl Dictionary for Pairs {
    fn ngram_probability(&mut self, _word: &[u16]) -> Result<i32, NegativeCount> {
        Ok(NOT_A_PROBABILITY)
    }

    /// Clearly the plain word after "is" and "wagged", the apostrophe after "think", unclear after
    /// "maybe"; nothing known after anything else.
    fn ln_odds(&mut self, word: &[u16], other: &[u16]) -> Result<Option<f32>, NegativeCount> {
        let asked = (
            String::from_utf16_lossy(word),
            String::from_utf16_lossy(other),
        );
        self.asked.push(asked);
        let previous = self
            .previous
            .words
            .first()
            .map(|word| String::from_utf16_lossy(word));
        Ok(match previous.as_deref() {
            Some("is" | "wagged") => Some(-6.0),
            Some("think") => Some(3.0),
            Some("maybe") => Some(apostrophes::MARGIN / 2.0),
            _ => None,
        })
    }

    fn decode(&mut self, input: &[u16], _: Purpose) -> Result<Vec<RawSuggestion>, NegativeCount> {
        let decoded: &[(&str, i32, i32)] = match String::from_utf16_lossy(input).as_str() {
            "ill" => &[
                ("I'll", i32::MAX - 1, KIND_WHITELIST),
                ("ill", 1_500_000, EXACT),
                ("will", 500_000, CORRECTION),
            ],
            "its" => &[("its", 1_600_000, EXACT), ("it's", 900_000, CORRECTION)],
            "dont" => &[("don't", 1_400_000, CORRECTION)],
            _ => &[],
        };
        let raw = |&(word, score, kind_and_flags): &(&str, i32, i32)| RawSuggestion {
            word: utf16(word),
            score,
            kind_and_flags,
        };
        Ok(decoded.iter().map(raw).collect())
    }
}

/// What `typed` after `before` (and before `after`) is auto-corrected to at `strength`, and the
/// odds asked for.
fn corrected(
    before: &str,
    typed: &str,
    after: &str,
    strength: Strength,
) -> (Option<String>, Vec<(String, String)>) {
    let previous_words = PreviousWords::from(&utf16(before));
    let state = TypingState {
        typed: utf16(typed),
        previous_words: previous_words.clone(),
        is_cursor_mid_word: !after.is_empty(),
    };
    let mut pairs = Pairs {
        previous: previous_words,
        asked: Vec::new(),
    };
    let personal = PersonalWords::new(Vec::new());
    let mut suggester = Suggester {
        dictionary: &mut pairs,
        personal: &personal,
        learned: None,
        threshold: strength.threshold(),
    };
    let strip = suggester.suggest(&state, 8).unwrap();
    let correction = strip.into_iter().find(|it| it.is_auto_correction);
    (
        correction.map(|it| String::from_utf16_lossy(&it.text)),
        pairs.asked,
    )
}

fn correction(before: &str, typed: &str) -> Option<String> {
    corrected(before, typed, "", Strength::DEFAULT).0
}

#[test]
fn a_whitelisted_word_meant_as_typed_is_kept() {
    assert_eq!(correction("he is ", "ill"), None);
    assert_eq!(correction("I think ", "ill").as_deref(), Some("I'll"));
}

#[test]
fn a_valid_word_gets_its_apostrophe_where_the_model_clearly_wants_it() {
    assert_eq!(correction("I think ", "its").as_deref(), Some("it's"));
    assert_eq!(correction("the dog wagged ", "its"), None);
}

#[test]
fn where_the_model_is_unsure_or_lacks_a_word_aosp_decides() {
    for before in ["maybe ", "on "] {
        assert_eq!(
            correction(before, "ill").as_deref(),
            Some("I'll"),
            "{before}"
        );
        assert_eq!(correction(before, "its"), None, "{before}");
    }
}

#[test]
fn the_model_is_asked_about_both_words_as_shown() {
    let (correction, asked) = corrected("I think ", "Its", "", Strength::DEFAULT);
    assert_eq!(correction.as_deref(), Some("It's"));
    assert_eq!(asked, [("It's".to_string(), "Its".to_string())]);
}

#[test]
fn a_word_that_is_no_word_is_left_to_aosp() {
    let (correction, asked) = corrected("he is ", "dont", "", Strength::DEFAULT);
    assert_eq!(correction.as_deref(), Some("don't"));
    assert!(asked.is_empty());
}

#[test]
fn nothing_mid_word_or_with_auto_correction_off() {
    assert_eq!(corrected("I think ", "its", "x", Strength::DEFAULT).0, None);
    assert_eq!(corrected("I think ", "its", "", Strength::Off).0, None);
    for strength in [Strength::Modest, Strength::Aggressive] {
        assert_eq!(
            corrected("I think ", "its", "", strength).0.as_deref(),
            Some("it's")
        );
    }
}
