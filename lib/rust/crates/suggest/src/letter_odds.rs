//! How likely each letter is to come next in the word being typed: the n-gram model's
//! probability, after the words before it, of its words (the dictionary's) that start with what
//! was typed, split by the letter that follows (`ngram::Model::next_units`). The keyboard's hit
//! test leans on it near a key's border (`keys::resize`).
//!
//! What was typed is looked up in lower case and capitalised, so "th" continues "the" and "Thomas"
//! alike, and a letter's odds are the same in either case.

use crate::tap::TapTrail;
use ngram::{Context, Model};

/// The letters a share of the odds is spread over evenly: a to z.
const LETTERS: f32 = 26.0;

/// Each letter's chance of coming next.
#[derive(Clone, Debug, PartialEq)]
pub struct LetterOdds {
    /// Each lower-case letter the model's words continue with, and its share, adding up to one.
    letters: Vec<(i32, f32)>,
}

impl LetterOdds {
    /// The share of the odds spread evenly over every letter, so that a letter no known word
    /// continues with is unlikely, never impossible: the user may be typing a word the dictionary
    /// doesn't have.
    pub const FLOOR: f32 = 0.1;

    /// The odds after `context` with `typed` typed so far; none when no word the model knows
    /// starts with it, as soon as a word leaves the dictionary.
    pub fn of(model: &Model, context: Context, typed: &[u16]) -> Option<LetterOdds> {
        let lower = chars::lowercase(typed);
        let mut prefixes = vec![lower.clone()];
        if let Some(&first) = lower.first() {
            let upper = chars::to_upper(first);
            if upper != first {
                let mut capitalised = lower;
                capitalised[0] = upper;
                prefixes.push(capitalised);
            }
        }
        let mut letters: Vec<(i32, f64)> = Vec::new();
        for prefix in &prefixes {
            for (unit, mass) in model.next_units(context, prefix) {
                let letter = i32::from(chars::to_lower(unit));
                match letters.iter_mut().find(|(l, _)| *l == letter) {
                    Some((_, sum)) => *sum += mass,
                    None => letters.push((letter, mass)),
                }
            }
        }
        let total: f64 = letters.iter().map(|&(_, mass)| mass).sum();
        if total.is_nan() || total <= 0.0 {
            return None;
        }
        let letters = letters
            .into_iter()
            .map(|(letter, mass)| (letter, (mass / total) as f32))
            .collect();
        Some(LetterOdds { letters })
    }

    /// How likely `letter` (a code point, in any case) is to come next, by the model alone.
    pub fn share(&self, letter: i32) -> f32 {
        let letter = u16::try_from(letter).map_or(letter, |unit| i32::from(chars::to_lower(unit)));
        self.letters
            .iter()
            .find(|&&(l, _)| l == letter)
            .map_or(0.0, |&(_, share)| share)
    }

    /// The natural log of how likely `letter` is to come next with `floor` of the odds spread
    /// over every letter: a key's prior for `keys::resize`.
    pub fn prior(&self, letter: i32, floor: f32) -> f32 {
        ((1.0 - floor) * self.share(letter) + floor / LETTERS).ln()
    }

    /// Each of `codes`' prior, with [`LetterOdds::FLOOR`]; none for a key that isn't a letter.
    pub fn priors(&self, codes: impl IntoIterator<Item = i32>) -> Vec<Option<f32>> {
        codes
            .into_iter()
            .map(|code| {
                let letter = u16::try_from(code).is_ok_and(chars::is_letter);
                letter.then(|| self.prior(code, Self::FLOOR))
            })
            .collect()
    }
}

/// The odds the suggester last worked out, and the word being typed it worked them out for.
#[derive(Clone, Debug, PartialEq)]
pub struct Latest {
    word: Vec<u16>,
    odds: LetterOdds,
}

impl Latest {
    pub fn new(word: Vec<u16>, odds: LetterOdds) -> Latest {
        Latest { word, odds }
    }

    /// The odds, if they still hold for the key about to go down on `trail`
    /// ([`TapTrail::at_word`]).
    pub fn at(&self, trail: &TapTrail) -> Option<&LetterOdds> {
        trail.at_word(&self.word).then_some(&self.odds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    fn model() -> Model {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../app/src/main/assets/dicts/main_en-US.lm");
        Model::open(std::fs::read(path).unwrap()).unwrap()
    }

    fn odds(model: &Model, before: &[&str], typed: &str) -> Option<LetterOdds> {
        let before: Vec<Vec<u16>> = before.iter().rev().map(|w| utf16(w)).collect();
        LetterOdds::of(model, model.context(&before, true), &utf16(typed))
    }

    #[test]
    fn after_th_an_e_is_far_likelier_than_an_r() {
        let model = model();
        let th = odds(&model, &["I", "think"], "th").unwrap();
        let (e, r) = (th.share('e' as i32), th.share('r' as i32));
        assert!(e > 5.0 * r, "{e} {r}");
        assert!(r > 0.0);
        let total: f32 = th.letters.iter().map(|&(_, s)| s).sum();
        assert!((total - 1.0).abs() < 1e-4, "{total}");
        // Any case typed, any case asked.
        assert_eq!(odds(&model, &["I", "think"], "TH"), Some(th.clone()));
        assert_eq!(th.share('E' as i32), e);
    }

    #[test]
    fn the_words_before_change_the_odds() {
        let model = model();
        // After "the", "th" goes on to "three" and "throne" more than to "them" or "then".
        let the_th = odds(&model, &["the"], "th").unwrap();
        assert!(the_th.share('r' as i32) > the_th.share('e' as i32));
        // "New Y..." is mostly York; "a y..." is mostly "year", "you"...
        let new = odds(&model, &["in", "New"], "").unwrap();
        let plain = odds(&model, &["in", "a"], "").unwrap();
        assert!(new.share('y' as i32) > 4.0 * plain.share('y' as i32));
    }

    #[test]
    fn the_odds_hold_while_the_trail_ends_in_their_word_and_only_letters_get_priors() {
        let model = model();
        let th = odds(&model, &["I", "think"], "th").unwrap();
        let latest = Latest::new(utf16("th"), th.clone());
        let mut trail = TapTrail::new(TapTrail::CAPACITY);
        for letter in ["t", "h"] {
            trail.pressed(i32::from(utf16(letter)[0]), None);
            assert_eq!(latest.at(&trail), None);
            trail.typed(&utf16(letter));
        }
        assert_eq!(latest.at(&trail), Some(&th));
        trail.pressed('e' as i32, None);
        assert_eq!(latest.at(&trail), None);
        let priors = th.priors(
            ['e', 'r', 'E', ' ', '\'']
                .map(|c| c as i32)
                .into_iter()
                .chain([-5]),
        );
        assert!(priors[3..].iter().all(Option::is_none), "{priors:?}");
        let [Some(e), Some(r), Some(upper)] = priors[..3] else {
            panic!("{priors:?}");
        };
        assert!(e > r && e == upper && e < 0.0);
    }

    #[test]
    fn a_word_outside_the_dictionary_has_no_odds_and_none_are_impossible() {
        let model = model();
        assert_eq!(odds(&model, &["the"], "xqzv"), None);
        let th = odds(&model, &["I", "think"], "th").unwrap();
        assert_eq!(th.share('q' as i32), 0.0);
        let floor = LetterOdds::FLOOR / LETTERS;
        assert!((th.prior('q' as i32, LetterOdds::FLOOR) - floor.ln()).abs() < 1e-6);
        assert!(th.prior('e' as i32, LetterOdds::FLOOR) > th.prior('r' as i32, LetterOdds::FLOOR));
    }
}
