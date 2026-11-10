//! `dictionary/structure/pt_common/bigram/bigram_list_read_write_utils` and
//! `dictionary/structure/v2/bigram/bigram_list_policy`: the list of bigrams a node has, each one
//! a link from the node to the word that may follow it.

use crate::bytes::{read_u8_advance, read_u16_advance, read_u24_advance};
use crate::defines::{NOT_A_DICT_POS, NOT_A_PROBABILITY};

/// `BigramListReadWriteUtils::BigramFlags`: one byte at the start of each bigram entry.
pub type BigramFlags = u8;

const MASK_ATTRIBUTE_ADDRESS_TYPE: BigramFlags = 0x30;
const FLAG_ATTRIBUTE_ADDRESS_TYPE_ONEBYTE: BigramFlags = 0x10;
const FLAG_ATTRIBUTE_ADDRESS_TYPE_TWOBYTES: BigramFlags = 0x20;
const FLAG_ATTRIBUTE_ADDRESS_TYPE_THREEBYTES: BigramFlags = 0x30;
const FLAG_ATTRIBUTE_OFFSET_NEGATIVE: BigramFlags = 0x40;
const FLAG_ATTRIBUTE_HAS_NEXT: BigramFlags = 0x80;
const MASK_ATTRIBUTE_PROBABILITY: BigramFlags = 0x0F;

/// One bigram entry as read: where its target node is, the bigram's probability, and whether
/// another entry follows it in the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BigramEntry {
    pub target_pos: i32,
    pub probability: i32,
    pub has_next: bool,
}

/// `BigramListReadWriteUtils::getProbabilityFromFlags`.
pub fn probability_from_flags(flags: BigramFlags) -> i32 {
    i32::from(flags & MASK_ATTRIBUTE_PROBABILITY)
}

/// `BigramListReadWriteUtils::hasNext`.
pub fn has_next(flags: BigramFlags) -> bool {
    flags & FLAG_ATTRIBUTE_HAS_NEXT != 0
}

/// `BigramListReadWriteUtils::isOffsetNegative`.
pub fn is_offset_negative(flags: BigramFlags) -> bool {
    flags & FLAG_ATTRIBUTE_OFFSET_NEGATIVE != 0
}

/// `BigramListReadWriteUtils::getBigramAddressAndAdvancePosition`: the target's position. The
/// offset is read from the bytes after the flags, and is counted back from the offset's own
/// position when negative. A flags value with no address type reads no offset, so the target is
/// the offset's position. The C++ adds and subtracts in `int`; this wraps instead of overflowing.
pub fn bigram_address_advance(buffer: &[u8], flags: BigramFlags, pos: &mut i32) -> i32 {
    let origin = *pos;
    let offset = match flags & MASK_ATTRIBUTE_ADDRESS_TYPE {
        FLAG_ATTRIBUTE_ADDRESS_TYPE_ONEBYTE => i32::from(read_u8_advance(buffer, pos)),
        FLAG_ATTRIBUTE_ADDRESS_TYPE_TWOBYTES => i32::from(read_u16_advance(buffer, pos)),
        FLAG_ATTRIBUTE_ADDRESS_TYPE_THREEBYTES => read_u24_advance(buffer, pos) as i32,
        _ => 0,
    };
    if is_offset_negative(flags) {
        origin.wrapping_sub(offset)
    } else {
        origin.wrapping_add(offset)
    }
}

/// `BigramListReadWriteUtils::getBigramEntryPropertiesAndAdvancePosition`: the entry's flags and
/// target, with `pos` moved past the entry. `None` when `pos` is at or past the end of the buffer
/// (the C++ returns false and leaves `pos` alone).
pub fn read_bigram_entry(buffer: &[u8], pos: &mut i32) -> Option<(BigramFlags, i32)> {
    let end = i32::try_from(buffer.len()).unwrap_or(i32::MAX);
    if end <= *pos {
        return None;
    }
    let flags = read_u8_advance(buffer, pos);
    let target = bigram_address_advance(buffer, flags, pos);
    Some((flags, target))
}

/// `BigramListReadWriteUtils::skipExistingBigrams`: moves `pos` past every entry of the list.
/// Returns false, with `pos` where it stopped, when an entry is past the end of the buffer.
pub fn skip_existing_bigrams(buffer: &[u8], pos: &mut i32) -> bool {
    loop {
        let Some((flags, _)) = read_bigram_entry(buffer, pos) else {
            return false;
        };
        if !has_next(flags) {
            return true;
        }
    }
}

/// `BigramListPolicy`: reads the bigram lists of a version 2 dictionary's body.
#[derive(Clone, Copy, Debug)]
pub struct BigramListPolicy<'a> {
    buffer: &'a [u8],
}

impl<'a> BigramListPolicy<'a> {
    /// The policy over the dictionary body `buffer`.
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer }
    }

    /// `BigramListPolicy::getNextBigram`. `None` where the C++ reports NOT_A_PROBABILITY and no
    /// next entry, when the entry is past the end of the buffer.
    pub fn next_bigram(&self, pos: &mut i32) -> Option<BigramEntry> {
        let (flags, target_pos) = read_bigram_entry(self.buffer, pos)?;
        Some(BigramEntry {
            target_pos,
            probability: probability_from_flags(flags),
            has_next: has_next(flags),
        })
    }

    /// `BigramListPolicy::skipAllBigrams`. The C++ returns the result of `skipExistingBigrams` and
    /// the reading code ignores it; so does this.
    pub fn skip_all(&self, pos: &mut i32) {
        let _ = skip_existing_bigrams(self.buffer, pos);
    }
}

/// `BinaryDictionaryBigramsIterator`: walks a node's bigram list. As in the C++, a failed read
/// leaves the bigram position where it was, with NOT_A_PROBABILITY and no next entry, and that
/// stale position is still reported once.
#[derive(Clone, Copy, Debug)]
pub struct BigramIterator<'a> {
    policy: BigramListPolicy<'a>,
    pos: i32,
    bigram_pos: i32,
    probability: i32,
    has_next: bool,
}

impl<'a> BigramIterator<'a> {
    /// The iterator over the list at `pos`, which is NOT_A_DICT_POS for a node with no bigrams.
    pub fn new(policy: BigramListPolicy<'a>, pos: i32) -> Self {
        Self {
            policy,
            pos,
            bigram_pos: NOT_A_DICT_POS,
            probability: NOT_A_PROBABILITY,
            has_next: pos != NOT_A_DICT_POS,
        }
    }

    /// `BinaryDictionaryBigramsIterator::hasNext`.
    pub fn has_next(&self) -> bool {
        self.has_next
    }

    /// `BinaryDictionaryBigramsIterator::next`.
    pub fn next_entry(&mut self) {
        match self.policy.next_bigram(&mut self.pos) {
            Some(entry) => {
                self.bigram_pos = entry.target_pos;
                self.probability = entry.probability;
                self.has_next = entry.has_next;
            }
            None => {
                self.probability = NOT_A_PROBABILITY;
                self.has_next = false;
            }
        }
    }

    /// `BinaryDictionaryBigramsIterator::getProbability`.
    pub fn probability(&self) -> i32 {
        self.probability
    }

    /// `BinaryDictionaryBigramsIterator::getBigramPos`.
    pub fn bigram_pos(&self) -> i32 {
        self.bigram_pos
    }
}
