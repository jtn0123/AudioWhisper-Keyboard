//! What the suggester and the learner ask the dictionary. The LatinIME decoder and its dictionary
//! stay C++ behind Kotlin, so each question goes back to Kotlin, whose answer can fail (an exception
//! there): hence the `Result`s.

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
