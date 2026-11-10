//! `dictionary/interface/word_listener`, `ngram_listener` and `dictionary/property/word_attributes`:
//! what a walk of the dictionary reports to its caller, and the attributes of a word in context.

use crate::defines::NOT_A_PROBABILITY;

/// `WordListener`: receives each word a walk visits, as code points.
pub trait WordListener {
    /// `WordListener::onVisitWord`.
    fn on_visit_word(&mut self, code_points: &[i32]);
}

/// `NgramListener`: receives each bigram entry of a word, its probability and the target word id.
pub trait NgramListener {
    /// `NgramListener::onVisitEntry`.
    fn on_visit_entry(&mut self, ngram_probability: i32, target_word_id: i32);
}

/// `WordAttributes`: a word's probability in context, and its flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordAttributes {
    probability: i32,
    is_blacklisted: bool,
    is_not_a_word: bool,
    is_possibly_offensive: bool,
}

impl Default for WordAttributes {
    /// `WordAttributes()`: no probability, no flags.
    fn default() -> Self {
        Self::new(NOT_A_PROBABILITY, false, false, false)
    }
}

impl WordAttributes {
    /// `WordAttributes(probability, isBlacklisted, isNotAWord, isPossiblyOffensive)`.
    pub fn new(
        probability: i32,
        is_blacklisted: bool,
        is_not_a_word: bool,
        is_possibly_offensive: bool,
    ) -> Self {
        Self {
            probability,
            is_blacklisted,
            is_not_a_word,
            is_possibly_offensive,
        }
    }

    /// `WordAttributes::getProbability`.
    pub fn probability(&self) -> i32 {
        self.probability
    }

    /// `WordAttributes::isBlacklisted`.
    pub fn is_blacklisted(&self) -> bool {
        self.is_blacklisted
    }

    /// `WordAttributes::isNotAWord`.
    pub fn is_not_a_word(&self) -> bool {
        self.is_not_a_word
    }

    /// `WordAttributes::isPossiblyOffensive`.
    pub fn is_possibly_offensive(&self) -> bool {
        self.is_possibly_offensive
    }
}
