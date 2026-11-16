//! The AOSP auto-correction threshold check, ported (through Kotlin's `AutoCorrection`) from
//! `AutoCorrectionUtils` and the native `AutocorrectionThresholdUtils::calcNormalizedScore` and
//! Damerau-Levenshtein edit distance (Apache-2.0, The Android Open Source Project).

use chars::code_point;

use crate::java;
use crate::raw::RawSuggestion;

/// AOSP's "modest" auto-correction threshold, the HeliBoard default, and this keyboard's
/// [`Strength::Normal`].
pub const MODEST_THRESHOLD: f32 = 0.185;

/// How readily typing is auto-corrected: the setting `correction__auto_correct` (Settings,
/// Typing, Corrections). Each strength is a threshold for [`suggestion_exceeds_threshold`]; the
/// numbers and why are in docs/autocorrect-results.md and docs/decisions.md.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strength {
    /// Never auto-correct; suggestions still show.
    Off,
    /// Fewer corrections: fewer correct words changed, more typos left.
    Modest,
    /// The default, AOSP's modest threshold: what the keyboard always did.
    Normal,
    /// More corrections: fewer typos left, more correct words changed.
    Aggressive,
}

impl Strength {
    /// In the order Kotlin's `AutoCorrectStrength` declares them, which is how a strength crosses
    /// JNI (its ordinal), and the stored names the settings catalogue lists.
    pub const ALL: [Strength; 4] = [
        Strength::Off,
        Strength::Modest,
        Strength::Normal,
        Strength::Aggressive,
    ];
    pub const DEFAULT: Strength = Strength::Normal;

    /// The strength with Kotlin ordinal `ordinal`; the default for anything else.
    pub fn from_ordinal(ordinal: i32) -> Strength {
        usize::try_from(ordinal)
            .ok()
            .and_then(|at| Self::ALL.get(at).copied())
            .unwrap_or(Self::DEFAULT)
    }

    /// The name it is stored and listed under.
    pub fn name(self) -> &'static str {
        match self {
            Strength::Off => "OFF",
            Strength::Modest => "MODEST",
            Strength::Normal => "NORMAL",
            Strength::Aggressive => "AGGRESSIVE",
        }
    }

    /// The threshold a correction's normalized score must reach, or `None` for never.
    pub fn threshold(self) -> Option<f32> {
        match self {
            Strength::Off => None,
            Strength::Modest => Some(MODEST_LEVEL),
            Strength::Normal => Some(MODEST_THRESHOLD),
            Strength::Aggressive => Some(AGGRESSIVE_LEVEL),
        }
    }
}

/// [`Strength::Modest`]'s threshold, chosen on the scoreboard (docs/autocorrect-results.md): the
/// lowest that halves harmful corrections for both typists.
const MODEST_LEVEL: f32 = 0.25;
/// [`Strength::Aggressive`]'s threshold, chosen the same way: the highest at which a left thumb's
/// word accuracy peaks. AOSP's own aggressive 0.067 changes more correct words and fixes no more.
const AGGRESSIVE_LEVEL: f32 = 0.12;

/// The native SUGGEST_INTERFACE_OUTPUT_SCALE: decoder scores are fixed-point with this scale.
const OUTPUT_SCALE: f32 = 1_000_000.0;

/// The native BASE_CHARS covers code points below this; above it characters compare as they are.
const BASE_CHARS_SIZE: u32 = 0x500;

/// `AutoCorrectionUtils.suggestionExceedsThreshold`: whitelisted words always pass, words the
/// decoder didn't mark as appropriate never do, and the rest need a normalized score of at least
/// `threshold`.
pub fn suggestion_exceeds_threshold(
    suggestion: &RawSuggestion,
    typed: &[u16],
    threshold: f32,
) -> bool {
    if suggestion.is_whitelist() {
        return true;
    }
    if !suggestion.is_appropriate_for_auto_correction() {
        return false;
    }
    normalized_score(typed, &suggestion.word, suggestion.score) >= threshold
}

/// `calcNormalizedScore`: the decoder's score scaled back to a float and weighted down by how much
/// of `after` had to be edited to get there from `before`.
pub fn normalized_score(before: &[u16], after: &[u16], score: i32) -> f32 {
    let before = java::code_points(before);
    let after = java::code_points(after);
    if before.is_empty() || after.is_empty() || after.iter().all(|&cp| cp == ' ' as i32) {
        return 0.0;
    }
    // Code points from a string are never negative.
    let distance = edit_distance(&unsigned(&before), &unsigned(&after));
    // A string's length is an Int, and so is its code point count.
    let length = after.len() as i32;
    if score <= 0 || distance >= length {
        return 0.0;
    }
    let weight = 1.0 - distance as f32 / length as f32;
    score as f32 / OUTPUT_SCALE * weight
}

fn unsigned(code_points: &[i32]) -> Vec<u32> {
    code_points.iter().map(|&cp| cp as u32).collect()
}

/// The native `EditDistance` with `DamerauLevenshteinEditDistancePolicy`: inserting, deleting,
/// substituting and swapping two neighbours each cost one, comparing base lower-case letters
/// ([`base_lower_case`]).
pub fn edit_distance(before: &[u32], after: &[u32]) -> i32 {
    let a: Vec<u32> = before.iter().map(|&cp| base_lower_case(cp)).collect();
    let b: Vec<u32> = after.iter().map(|&cp| base_lower_case(cp)).collect();
    let width = b.len() + 1;
    let mut dp = vec![0i32; (a.len() + 1) * width];
    for (i, cost) in (0..=a.len()).zip(0..) {
        dp[width * i] = cost;
    }
    for (j, cost) in (0..=b.len()).zip(0..) {
        dp[j] = cost;
    }
    for i in 0..a.len() {
        for j in 0..b.len() {
            let substitution = i32::from(a[i] != b[j]);
            let mut best = (dp[width * i + j + 1] + 1)
                .min(dp[width * (i + 1) + j] + 1)
                .min(dp[width * i + j] + substitution);
            if i > 0 && j > 0 && a[i] == b[j - 1] && b[j] == a[i - 1] {
                best = best.min(dp[width * (i - 1) + j - 1] + substitution);
            }
            dp[width * (i + 1) + j + 1] = best;
        }
    }
    dp[dp.len() - 1]
}

/// The native `CharUtils::toBaseLowerCase`: without its accent (below U+0500 only), in lower case.
pub fn base_lower_case(code_point: u32) -> u32 {
    let base = match u16::try_from(code_point) {
        Ok(unit) if code_point < BASE_CHARS_SIZE => code_point::decomposed_first(unit),
        _ => code_point,
    };
    code_point::to_lower(base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::{FLAG_APPROPRIATE_FOR_AUTO_CORRECTION, KIND_CORRECTION, KIND_WHITELIST};

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    fn code_points(text: &str) -> Vec<u32> {
        text.chars().map(u32::from).collect()
    }

    fn distance(a: &str, b: &str) -> i32 {
        edit_distance(&code_points(a), &code_points(b))
    }

    #[test]
    fn counts_edits_with_swaps_as_one() {
        assert_eq!(distance("teh", "the"), 1);
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("", "abc"), 3);
        assert_eq!(distance("abc", ""), 3);
        assert_eq!(distance("ca", "abc"), 3);
        assert_eq!(distance("Café", "cafe"), 0);
        assert_eq!(distance("ǅ", "ǆ"), 0);
        assert_eq!(distance("ώ", "ω"), 1); // above U+0500's base characters, the accent counts
    }

    #[test]
    fn strips_accents_below_u0500_only() {
        assert_eq!(base_lower_case('É' as u32), 'e' as u32);
        assert_eq!(base_lower_case('ǖ' as u32), 'u' as u32);
        assert_eq!(base_lower_case('ḕ' as u32), 'ḕ' as u32);
        assert_eq!(base_lower_case('Ḕ' as u32), 'ḕ' as u32);
    }

    #[test]
    fn weights_the_score_by_the_edits() {
        let score = normalized_score(&utf16("teh"), &utf16("the"), 1_000_000);
        assert_eq!(score, 1.0 - 1.0 / 3.0);
        assert_eq!(normalized_score(&utf16(""), &utf16("the"), 1_000_000), 0.0);
        assert_eq!(normalized_score(&utf16("a"), &utf16("  "), 1_000_000), 0.0);
        assert_eq!(normalized_score(&utf16("teh"), &utf16("the"), 0), 0.0);
        assert_eq!(
            normalized_score(&utf16("xyz"), &utf16("the"), 1_000_000),
            0.0
        );
    }

    #[test]
    fn whitelisted_words_always_pass() {
        let suggestion = |kind_and_flags, score| RawSuggestion {
            word: utf16("the"),
            score,
            kind_and_flags,
        };
        let appropriate = KIND_CORRECTION | FLAG_APPROPRIATE_FOR_AUTO_CORRECTION;
        assert!(suggestion_exceeds_threshold(
            &suggestion(KIND_WHITELIST, 0),
            &utf16("x"),
            1.0
        ));
        assert!(!suggestion_exceeds_threshold(
            &suggestion(KIND_CORRECTION, 1_000_000),
            &utf16("teh"),
            0.1
        ));
        assert!(suggestion_exceeds_threshold(
            &suggestion(appropriate, 1_000_000),
            &utf16("teh"),
            MODEST_THRESHOLD
        ));
        assert!(!suggestion_exceeds_threshold(
            &suggestion(appropriate, 100_000),
            &utf16("teh"),
            MODEST_THRESHOLD
        ));
    }

    #[test]
    fn normal_is_the_default_and_what_the_keyboard_always_did() {
        assert_eq!(Strength::DEFAULT, Strength::Normal);
        assert_eq!(Strength::Normal.threshold(), Some(0.185));
        assert_eq!(Strength::Off.threshold(), None);
        let names: Vec<&str> = Strength::ALL.iter().map(|it| it.name()).collect();
        assert_eq!(names, ["OFF", "MODEST", "NORMAL", "AGGRESSIVE"]);
        assert_eq!(Strength::from_ordinal(0), Strength::Off);
        assert_eq!(Strength::from_ordinal(3), Strength::Aggressive);
        assert_eq!(Strength::from_ordinal(4), Strength::Normal);
        assert_eq!(Strength::from_ordinal(-1), Strength::Normal);
    }

    #[test]
    fn a_stronger_setting_corrects_on_less_evidence() {
        let appropriate = KIND_CORRECTION | FLAG_APPROPRIATE_FOR_AUTO_CORRECTION;
        // "teh" to "the" is one edit in three: these score 0.22, 0.17 and 0.10.
        let corrects = |score, strength: Strength| {
            let suggestion = RawSuggestion {
                word: utf16("the"),
                score,
                kind_and_flags: appropriate,
            };
            let threshold = strength.threshold();
            threshold.is_some_and(|it| suggestion_exceeds_threshold(&suggestion, &utf16("teh"), it))
        };
        let levels = |score| Strength::ALL.map(|strength| corrects(score, strength));
        assert_eq!(levels(330_000), [false, false, true, true]);
        assert_eq!(levels(255_000), [false, false, false, true]);
        assert_eq!(levels(150_000), [false, false, false, false]);
        assert_eq!(levels(1_000_000), [false, true, true, true]);
    }

    #[test]
    fn strips_accents_up_to_u_0500() {
        assert_eq!(base_lower_case(0x04F1), 0x0443);
        assert_eq!(base_lower_case(0x1E01), 0x1E01);
    }

    #[test]
    fn a_word_of_spaces_scores_nothing() {
        assert_eq!(normalized_score(&utf16(" "), &utf16(" "), 1_000_000), 0.0);
    }
}
