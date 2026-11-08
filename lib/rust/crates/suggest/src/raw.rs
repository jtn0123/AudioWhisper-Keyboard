//! What the native decoder gives: a word, its score, and the AOSP kind plus flag bits
//! (`Dictionary::KIND_*` in the native `dictionary.h`). Was Kotlin's `RawSuggestion`.

/// The native NOT_A_PROBABILITY: the dictionary has no such word (or no such n-gram).
pub const NOT_A_PROBABILITY: i32 = -1;

pub const KIND_MASK: i32 = 0xFF;
pub const KIND_CORRECTION: i32 = 1;
pub const KIND_WHITELIST: i32 = 3;
pub const KIND_SHORTCUT: i32 = 7;
pub const KIND_PREDICTION: i32 = 8;
pub const FLAG_APPROPRIATE_FOR_AUTO_CORRECTION: i32 = 0x1000_0000;
pub const FLAG_EXACT_MATCH: i32 = 0x4000_0000;

/// One result of the decoder, as the dictionary spells it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawSuggestion {
    pub word: Vec<u16>,
    pub score: i32,
    pub kind_and_flags: i32,
}

impl RawSuggestion {
    pub fn kind(&self) -> i32 {
        self.kind_and_flags & KIND_MASK
    }

    /// A dictionary-curated replacement such as "im" for "I'm"; AOSP always auto-corrects these.
    pub fn is_whitelist(&self) -> bool {
        self.kind() == KIND_WHITELIST
    }

    /// A next-word prediction (the decoder was called with nothing typed).
    pub fn is_prediction(&self) -> bool {
        self.kind() == KIND_PREDICTION
    }

    /// A shortcut expansion; AOSP never auto-corrects to one.
    pub fn is_shortcut(&self) -> bool {
        self.kind() == KIND_SHORTCUT
    }

    /// The decoder judged this word a plausible auto-correction target.
    pub fn is_appropriate_for_auto_correction(&self) -> bool {
        self.kind_and_flags & FLAG_APPROPRIATE_FOR_AUTO_CORRECTION != 0
    }
}
