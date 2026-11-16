//! The bundled n-gram model (`crates/ngram`) in the session: what it predicts after a space, how
//! it re-weighs the decoder's candidates for the word being typed, and what it tells the swipe
//! ranker. Without a model the session is the dictionary alone, exactly as before.
//!
//! The decoder scores a candidate partly by the dictionary's 0-255 value for it after the
//! previous word: the pair's value if the dictionary has the pair, else the word's own. The model
//! knows far more pairs, and the word before those too, so for each candidate that language term
//! is redone with the model: the word's own dictionary value, raised (or lowered) by how much
//! likelier the model finds it after these words than anywhere ([`ngram::Model::lift`]), in the
//! dictionary's units. The score changes by that difference, weighted as the decoder weighs its
//! language cost (`calculate_final_score`), and the candidates are sorted again.

use crate::letter_odds::LetterOdds;
use crate::previous_words::PreviousWords;
use crate::raw::{KIND_CORRECTION, KIND_PREDICTION, NOT_A_PROBABILITY, RawSuggestion};
use latin::defines::{MAX_RESULTS, SUGGEST_INTERFACE_OUTPUT_SCALE};
use latin::search::scoring_params::{DISTANCE_WEIGHT_LANGUAGE, TYPING_MAX_OUTPUT_SCORE_PER_INPUT};
use ngram::{Context, Model};
use std::cmp::Reverse;
use std::sync::Arc;

/// The decoder's most probable value.
const MAX_VALUE: f32 = 255.0;
const SPACE: u16 = b' ' as u16;

/// How much the session leans on the model. The defaults are the scoreboard's choice
/// (docs/language-model-results.md).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tuning {
    /// How much of the model's lift goes into a typed candidate's value: 0 leaves the decoder
    /// alone, 1 takes all of it.
    pub typing: f32,
    /// The least lift, in nats, that the swipe ranker counts as an n-gram for a guess: 0 counts
    /// every word the model finds likelier here than anywhere.
    pub swipe: f32,
}

impl Tuning {
    pub const DEFAULT: Tuning = Tuning {
        typing: 0.75,
        swipe: 0.0,
    };
}

impl Default for Tuning {
    fn default() -> Self {
        Tuning::DEFAULT
    }
}

/// A model and how the session uses it.
#[derive(Debug, Clone)]
pub struct LanguageModel {
    model: Arc<Model>,
    tuning: Tuning,
}

impl LanguageModel {
    pub fn new(model: Arc<Model>, tuning: Tuning) -> Self {
        LanguageModel { model, tuning }
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    fn context(&self, previous: &PreviousWords) -> Context {
        (self.model).context(&previous.words, previous.is_beginning_of_sentence)
    }

    /// A probability as a 0-255 dictionary value.
    fn value(&self, ln: f32) -> f32 {
        let header = self.model.header();
        (header.scale_a + header.scale_b * ln).clamp(0.0, MAX_VALUE)
    }

    /// The likeliest next words after `previous`, best first, as the decoder's predictions. After
    /// a word the model doesn't know, the words likeliest after any word.
    pub fn predictions(&self, previous: &PreviousWords) -> Vec<RawSuggestion> {
        if previous.slot_count() == 0 {
            return Vec::new();
        }
        let predicted = self.model.predict(self.context(previous), MAX_RESULTS);
        (predicted.into_iter())
            .filter_map(|(id, ln)| {
                Some(RawSuggestion {
                    word: self.model.spelling(id)?,
                    score: (self.value(ln) * 1000.0) as i32,
                    kind_and_flags: KIND_PREDICTION,
                })
            })
            .collect()
    }

    /// How likely each letter is to come next in `typed` after `previous` (see [`LetterOdds`]).
    pub fn letter_odds(&self, previous: &PreviousWords, typed: &[u16]) -> Option<LetterOdds> {
        LetterOdds::of(&self.model, self.context(previous), typed)
    }

    /// Redoes the language term of each of the decoder's single-word candidates for `input_size`
    /// typed code points after `previous`, then sorts them again, keeping the order of ties.
    /// `dictionary` gives a word's own value and its value after `previous` (or
    /// [`NOT_A_PROBABILITY`]), as the decoder used them.
    pub fn rescore(
        &self,
        previous: &PreviousWords,
        input_size: usize,
        results: &mut [RawSuggestion],
        mut dictionary: impl FnMut(&[u16]) -> (i32, i32),
    ) {
        let context = self.context(previous);
        if context.is_empty() || self.tuning.typing == 0.0 {
            return;
        }
        let max_distance = (input_size as f32)
            .mul_add(TYPING_MAX_OUTPUT_SCORE_PER_INPUT, DISTANCE_WEIGHT_LANGUAGE);
        let per_value = DISTANCE_WEIGHT_LANGUAGE / MAX_VALUE / max_distance;
        let rise = self.tuning.typing * self.model.header().scale_b;
        for result in results.iter_mut() {
            if result.kind() != KIND_CORRECTION || result.word.contains(&SPACE) {
                continue;
            }
            let Some(lift) =
                (self.model.word(&result.word)).and_then(|id| self.model.lift(context, id))
            else {
                continue;
            };
            let (unigram, ngram) = dictionary(&result.word);
            if unigram == NOT_A_PROBABILITY {
                continue;
            }
            let used = if ngram == NOT_A_PROBABILITY {
                unigram
            } else {
                ngram
            };
            let value = (unigram as f32 + rise * lift).clamp(0.0, MAX_VALUE);
            let change = (value - used as f32) * per_value * SUGGEST_INTERFACE_OUTPUT_SCALE;
            result.score = result.score.saturating_add(change as i32);
        }
        results.sort_by_key(|r| Reverse(r.score));
    }

    /// How much likelier, in nats, the model finds `word` than `other` after `previous`; none if
    /// it lacks either.
    pub fn ln_odds(&self, previous: &PreviousWords, word: &[u16], other: &[u16]) -> Option<f32> {
        let context = self.context(previous);
        let ln =
            |word| (self.model.word(word)).and_then(|id| self.model.ln_probability(context, id));
        Some(ln(word)? - ln(other)?)
    }

    /// The swipe ranker's n-gram for `word` after `previous` (see `glide_ranker`): the word's
    /// own dictionary value `unigram` raised by the model's lift, if the lift is at least
    /// [`Tuning::swipe`]; else none.
    pub fn swipe_ngram(&self, previous: &PreviousWords, word: &[u16], unigram: i32) -> i32 {
        if unigram == NOT_A_PROBABILITY {
            return NOT_A_PROBABILITY;
        }
        let lift =
            (self.model.word(word)).and_then(|id| self.model.lift(self.context(previous), id));
        match lift {
            Some(lift) if lift >= self.tuning.swipe => {
                let rise = self.model.header().scale_b * lift;
                (unigram as f32 + rise).round().min(MAX_VALUE) as i32
            }
            _ => NOT_A_PROBABILITY,
        }
    }
}
