//! Reading and writing a version 202 dictionary: LatinIME's read-only patricia trie, the format of
//! the app's `main_en-US.dict`. The keyboard only reads it (`crates/latin`); this writes it, so the
//! dictionary can be rebuilt with new statistics, and reads it back whole (every word, flag,
//! shortcut and next word) to test the writer and to carry the old dictionary's words over.
//!
//! The layout is AOSP's `BinaryDictEncoderUtils` for version 2: the header, then the node arrays
//! in depth-first order, each array followed by its nodes' children, siblings in code point order.
//! The keyboard finds a word from its node's position by relying on that order.

const MAGIC: u32 = 0x9BC1_3AFE;
const VERSION: u16 = 202;

const CHILDREN_ADDRESS_MASK: u8 = 0xC0;
const MULTIPLE_CHARS: u8 = 0x20;
const TERMINAL: u8 = 0x10;
const HAS_SHORTCUTS: u8 = 0x08;
const HAS_BIGRAMS: u8 = 0x04;
const NOT_A_WORD: u8 = 0x02;
const POSSIBLY_OFFENSIVE: u8 = 0x01;

const ATTRIBUTE_HAS_NEXT: u8 = 0x80;
const BIGRAM_OFFSET_NEGATIVE: u8 = 0x40;
const BIGRAM_ADDRESS_MASK: u8 = 0x30;
const PROBABILITY_MASK: u8 = 0x0F;

const TERMINATOR: u8 = 0x1F;
const MIN_ONE_BYTE: u32 = 0x20;
const MAX_ONE_BYTE: u32 = 0xFF;
/// Longest word the keyboard reads.
pub const MAX_WORD_LENGTH: usize = 48;

/// A word of the dictionary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub word: Vec<u32>,
    /// 0-255.
    pub probability: u8,
    pub not_a_word: bool,
    pub possibly_offensive: bool,
    /// Shortcut targets with their 4-bit probability (15 marks a whitelist entry).
    pub shortcuts: Vec<(Vec<u32>, u8)>,
    /// The words that may follow, as indexes into the dictionary's entries, with their 4-bit
    /// probability.
    pub bigrams: Vec<(usize, u8)>,
}

/// A whole dictionary: its header and its words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dictionary {
    /// The header's flags field.
    pub flags: u16,
    /// The header's attributes, in order.
    pub attributes: Vec<(String, String)>,
    pub entries: Vec<Entry>,
}

impl Dictionary {
    pub fn attribute(&self, key: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

pub fn code_points(word: &str) -> Vec<u32> {
    word.chars().map(u32::from).collect()
}

pub fn string(code_points: &[u32]) -> String {
    code_points
        .iter()
        .filter_map(|&c| char::from_u32(c))
        .collect()
}

mod read;
#[cfg(test)]
mod tests;
mod write;

pub use read::read;
pub use write::write;
