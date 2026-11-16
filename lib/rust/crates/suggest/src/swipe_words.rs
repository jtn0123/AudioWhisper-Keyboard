//! The words swipe typing can produce and how strongly the glide classifier favours each: the
//! dictionary's words, plus the user's personal and learned words so their names and slang can be
//! swiped too. Was the rules of Kotlin's `SwipeWords`, which still holds the dictionary's word list
//! and looks each word's probability up in it.

use std::collections::HashMap;

use crate::raw::NOT_A_PROBABILITY;
use crate::word_characters::is_word;

pub const MAX_PROBABILITY: i32 = 255;

/// Words this common or more count the same; see [`frequency_of`].
pub const COMMON: i32 = 170;

/// What each step of probability below [`COMMON`] costs a word, in nats; see [`frequency_of`].
pub const COST_PER_STEP: f64 = 0.007;

/// A word typed [`crate::learned_words::KNOWN_AFTER`] times counts as fairly common (the
/// dictionary has "hello" at 120), whatever the dictionary says. One value for every learned word,
/// so using a word again doesn't change the list and re-index the classifier.
pub const LEARNED_PROBABILITY: i32 = 120;

/// Whether a swipe can spell `word` on a letters layout: whether it [`is_word`], letters with
/// apostrophes or hyphens inside, which a swipe skips over. Not "MP3" or "US$".
pub fn is_swipeable(word: &[u16]) -> bool {
    is_word(word)
}

/// The classifier's frequency for a word of `probability` (0-255): 1 from [`COMMON`] up, then
/// e^-([`COST_PER_STEP`] x steps below it), down to 0.30 for the rarest word.
///
/// Calibrated by replaying the classifier's scoring on the host over the whole dictionary, with
/// swipes that miss key centres by 22 and 40 px (it reproduces the emulator's results swipe for
/// swipe). Weighted by how often each of the 1,500 commonest words is used, it gets 98.1% and 86.1%
/// of them, against 97.5% and 80.5% with FlorisBoard's word list, and far more of the rarer words
/// that list lacked. Common words count the same because words sharing a path differ only by
/// frequency ("to", "too": 0.07 nats apart), and context then settles them (see
/// [`crate::glide_ranker`]).
///
/// The price: a clean swipe separates neighbouring keys by little, so a commoner neighbour can win
/// ("an" beats "am" by 0.14), until the user's habit tips it.
pub fn frequency_of(probability: i32) -> f64 {
    let steps = (COMMON - probability.clamp(0, MAX_PROBABILITY)).max(0);
    (-COST_PER_STEP * f64::from(steps)).exp()
}

/// How a 0-255 probability becomes the classifier's frequency: `below` nats a step below
/// `common`, and `above` nats a step from there up to [`MAX_PROBABILITY`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve {
    pub common: i32,
    pub below: f64,
    pub above: f64,
}

impl Curve {
    /// [`frequency_of`]: common words count the same.
    pub const FLAT: Curve = Curve {
        common: COMMON,
        below: COST_PER_STEP,
        above: 0.0,
    };

    /// What the keyboard uses: [`COST_PER_STEP`] all the way up, so a commoner word always
    /// counts for more. The flat top tied words that share a path ("the" and "three", "to" and
    /// "too"); one slope swipes better on both of the scoreboard's simulators
    /// (docs/swipe-accuracy.md).
    pub const DEFAULT: Curve = Curve {
        common: COMMON,
        below: COST_PER_STEP,
        above: COST_PER_STEP,
    };

    /// The classifier's frequency for a word of `probability`.
    pub fn frequency_of(&self, probability: i32) -> f64 {
        let probability = probability.clamp(0, MAX_PROBABILITY);
        let below = (self.common - probability).max(0);
        let above = MAX_PROBABILITY - probability.max(self.common);
        (-(self.below * f64::from(below) + self.above * f64::from(above))).exp()
    }

    /// [`frequency`] on this curve.
    pub fn frequency(&self, probability: i32, user_probability: i32) -> f64 {
        let best = probability.max(user_probability);
        if best == NOT_A_PROBABILITY {
            0.0
        } else {
            self.frequency_of(best)
        }
    }
}

/// How strongly the classifier favours a word the dictionary gives `probability` and the user's
/// words `user_probability` (either [`NOT_A_PROBABILITY`]): the higher one's [`frequency_of`], or
/// 0 for a word neither knows.
pub fn frequency(probability: i32, user_probability: i32) -> f64 {
    let best = probability.max(user_probability);
    if best == NOT_A_PROBABILITY {
        0.0
    } else {
        frequency_of(best)
    }
}

/// [`frequency`] on the curve the keyboard uses, [`Curve::DEFAULT`]. [`frequency`] stays the
/// flat curve, which the Kotlin reference holds it to.
pub fn tuned_frequency(probability: i32, user_probability: i32) -> f64 {
    Curve::DEFAULT.frequency(probability, user_probability)
}

/// The user's words as probabilities: `learned` spellings at [`LEARNED_PROBABILITY`], `personal`
/// words at their frequency (taken into 0-255), and the higher one where a word is in both. Words
/// that aren't [`is_swipeable`] are left out. Each word comes once, as where it first appears among
/// `learned` followed by `personal`, with its probability; in the order Kotlin's map first had
/// them, the learned ones and then the personal ones that weren't learned.
pub fn user_words(personal: &[(&[u16], i32)], learned: &[&[u16]]) -> Vec<(usize, i32)> {
    let mut words: Vec<(usize, i32)> = Vec::new();
    let mut places: HashMap<&[u16], usize> = HashMap::new();
    for (index, &word) in learned.iter().enumerate() {
        if is_swipeable(word) {
            places.entry(word).or_insert_with(|| {
                words.push((index, LEARNED_PROBABILITY));
                words.len() - 1
            });
        }
    }
    for (index, &(word, frequency)) in personal.iter().enumerate() {
        if !is_swipeable(word) {
            continue;
        }
        let place = *places.entry(word).or_insert_with(|| {
            words.push((learned.len() + index, 0));
            words.len() - 1
        });
        let probability = &mut words[place].1;
        *probability = (*probability).max(frequency.clamp(0, MAX_PROBABILITY));
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    #[test]
    fn common_words_count_the_same_and_rarer_ones_less() {
        assert_eq!(frequency_of(255), 1.0);
        assert_eq!(frequency_of(COMMON), 1.0);
        assert_eq!(frequency_of(1000), 1.0);
        assert!((frequency_of(100) - (-0.007_f64 * 70.0).exp()).abs() < 1e-12);
        assert!((frequency_of(0) - 0.304).abs() < 0.001);
        assert_eq!(frequency_of(-20), frequency_of(0));
    }

    #[test]
    fn a_word_is_as_frequent_as_its_likelier_source() {
        assert_eq!(frequency(NOT_A_PROBABILITY, NOT_A_PROBABILITY), 0.0);
        assert_eq!(frequency(100, NOT_A_PROBABILITY), frequency_of(100));
        assert_eq!(frequency(NOT_A_PROBABILITY, 120), frequency_of(120));
        assert_eq!(frequency(10, LEARNED_PROBABILITY), frequency_of(120));
        // The higher of the two is "none", whatever the other's other negative value.
        assert_eq!(frequency(-5, NOT_A_PROBABILITY), 0.0);
        assert_eq!(frequency(-5, -7), frequency_of(0));
    }

    #[test]
    fn user_words_take_the_higher_probability_in_first_seen_order() {
        let (kaito, bae, mp3, yeet) = (utf16("Kaito"), utf16("bae"), utf16("mp3"), utf16("yeet"));
        let personal = [
            (&kaito[..], 300),
            (&bae[..], 50),
            (&mp3[..], 10),
            (&yeet[..], 130),
        ];
        let learned = [&bae[..], &mp3[..], &bae[..]];
        // Kaito is the fourth of these, yeet the seventh.
        assert_eq!(
            user_words(&personal, &learned),
            [(0, 120), (3, 255), (6, 130)]
        );
        assert_eq!(user_words(&[(&kaito[..], -4)], &[]), [(0, 0)]);
        let louder = [(&bae[..], 200)];
        assert_eq!(user_words(&louder, &learned), [(0, 200)]);
    }

    #[test]
    fn the_flat_curve_is_frequency_of() {
        for probability in -3..=258 {
            assert_eq!(
                Curve::FLAT.frequency_of(probability),
                frequency_of(probability)
            );
            assert_eq!(
                Curve::FLAT.frequency(probability, 120),
                frequency(probability, 120)
            );
        }
    }

    #[test]
    fn the_keyboards_curve_has_one_slope_all_the_way_up() {
        let curve = Curve::DEFAULT;
        assert_eq!(curve.frequency_of(MAX_PROBABILITY), 1.0);
        for probability in 0..MAX_PROBABILITY {
            let step = curve.frequency_of(probability) / curve.frequency_of(probability + 1);
            assert!(
                (step - (-COST_PER_STEP).exp()).abs() < 1e-12,
                "{probability}"
            );
        }
        assert_eq!(tuned_frequency(NOT_A_PROBABILITY, NOT_A_PROBABILITY), 0.0);
        assert_eq!(
            tuned_frequency(10, LEARNED_PROBABILITY),
            curve.frequency_of(LEARNED_PROBABILITY)
        );
        assert_eq!(tuned_frequency(300, -1), 1.0);
    }

    #[test]
    fn swipes_letters_and_connectors_only() {
        for word in ["don't", "rock’n’roll", "café", "e-mail"] {
            assert!(is_swipeable(&utf16(word)), "{word}");
        }
        for word in ["US$", "'", "", "mp3", "a b"] {
            assert!(!is_swipeable(&utf16(word)), "{word}");
        }
    }
}
