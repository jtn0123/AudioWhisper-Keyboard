//! What the suggester and learner ask a dictionary. The native session implements these traits
//! directly over `latin`; fallible test/platform adapters may implement them too.

use crate::raw::RawSuggestion;

/// A word's probability in the dictionary.
pub trait Lookup {
    /// Why an answer couldn't be had.
    type Error;

    /// `word`'s probability on the dictionary's 0-255 log scale, or
    /// [`crate::raw::NOT_A_PROBABILITY`] if the dictionary lacks it.
    fn probability(&mut self, word: &[u16]) -> Result<i32, Self::Error>;
}

/// The decoder and the dictionary around one request for suggestions, which knows the words
/// before the one typed and where its letters were tapped.
pub trait Dictionary: Lookup {
    /// `word`'s probability right after the request's previous words, or
    /// [`crate::raw::NOT_A_PROBABILITY`] when the dictionary has no such n-gram or there is no
    /// previous word.
    fn ngram_probability(&mut self, word: &[u16]) -> Result<i32, Self::Error>;

    /// How much likelier, in nats, the language model finds `word` than `other` after the
    /// request's previous words; none without a model or if it lacks either word.
    fn ln_odds(&mut self, _word: &[u16], _other: &[u16]) -> Result<Option<f32>, Self::Error> {
        Ok(None)
    }

    /// The decoder's corrections and completions of `input`, or its predictions when `input` is
    /// empty, highest score first, decoded for `purpose`.
    fn decode(
        &mut self,
        input: &[u16],
        purpose: Purpose,
    ) -> Result<Vec<RawSuggestion>, Self::Error>;
}

/// What a [`Dictionary::decode`] is for, which says what it decodes with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    /// The word being typed: after the request's previous words, where its letters were tapped.
    Typing,
    /// The next word, nothing typed yet: after the request's previous words.
    Predicting,
    /// A word on its own, for the spell checker: no previous words, no taps.
    Spelling,
}

impl Purpose {
    /// Whether the decoder reads the request's previous words.
    pub fn in_context(self) -> bool {
        self != Purpose::Spelling
    }

    /// Whether the decoder reads where the letters were tapped.
    pub fn with_taps(self) -> bool {
        self == Purpose::Typing
    }
}
