//! `dictionary/structure/pt_common/shortcut/shortcut_list_reading_utils` and
//! `dictionary/structure/v2/shortcut/shortcut_list_policy`: the shortcut list a node has, each
//! entry a shortcut's target string with its probability, and whether it is a whitelist entry.

use crate::bytes::{read_string_advance, read_u8_advance, read_u16_advance};
use crate::defines::NOT_A_DICT_POS;

/// `ShortcutListReadingUtils::ShortcutFlags`: one byte at the start of each shortcut entry.
pub type ShortcutFlags = u8;

const FLAG_ATTRIBUTE_HAS_NEXT: ShortcutFlags = 0x80;
const MASK_ATTRIBUTE_PROBABILITY: ShortcutFlags = 0x0F;
/// `ShortcutListReadingUtils::SHORTCUT_LIST_SIZE_FIELD_SIZE`: the list's size field counts itself.
const SHORTCUT_LIST_SIZE_FIELD_SIZE: i32 = 2;
/// `ShortcutListReadingUtils::WHITELIST_SHORTCUT_PROBABILITY`.
const WHITELIST_SHORTCUT_PROBABILITY: i32 = 15;

/// One shortcut entry as read: how many code points its target has, and its flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShortcutEntry {
    pub code_point_count: usize,
    pub probability: i32,
    pub is_whitelist: bool,
    pub has_next: bool,
}

/// `ShortcutListReadingUtils::getProbabilityFromFlags`.
pub fn probability_from_flags(flags: ShortcutFlags) -> i32 {
    i32::from(flags & MASK_ATTRIBUTE_PROBABILITY)
}

/// `ShortcutListReadingUtils::hasNext`.
pub fn has_next(flags: ShortcutFlags) -> bool {
    flags & FLAG_ATTRIBUTE_HAS_NEXT != 0
}

/// `ShortcutListReadingUtils::isWhitelist`: the probability is the whitelist marker.
pub fn is_whitelist(flags: ShortcutFlags) -> bool {
    probability_from_flags(flags) == WHITELIST_SHORTCUT_PROBABILITY
}

/// `ShortcutListReadingUtils::getShortcutListSizeAndForwardPointer`: the list's size, not counting
/// the size field, with `pos` moved past the field. The stored size counts the field too, so a
/// stored size below 2 reads as a negative size.
pub fn list_size_advance(buffer: &[u8], pos: &mut i32) -> i32 {
    i32::from(read_u16_advance(buffer, pos)) - SHORTCUT_LIST_SIZE_FIELD_SIZE
}

/// `ShortcutListPolicy`: reads the shortcut lists of a version 2 dictionary's body.
#[derive(Clone, Copy, Debug)]
pub struct ShortcutListPolicy<'a> {
    buffer: &'a [u8],
}

impl<'a> ShortcutListPolicy<'a> {
    /// The policy over the dictionary body `buffer`.
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer }
    }

    /// `ShortcutListPolicy::getStartPos`: where the first entry of the list at `pos` is, past the
    /// list's size field; NOT_A_DICT_POS stays NOT_A_DICT_POS.
    pub fn start_pos(&self, pos: i32) -> i32 {
        if pos == NOT_A_DICT_POS {
            return NOT_A_DICT_POS;
        }
        let mut list_pos = pos;
        list_size_advance(self.buffer, &mut list_pos);
        list_pos
    }

    /// `ShortcutListPolicy::getNextShortcut`: reads the entry at `pos`, its target into `out` (at
    /// most `max_code_point_count` code points, and at most `out.len()`), and moves `pos` past it.
    /// The C++ reads the target without a code point table, as the TODO there says.
    pub fn next_shortcut(
        &self,
        max_code_point_count: usize,
        out: &mut [i32],
        pos: &mut i32,
    ) -> ShortcutEntry {
        let flags = read_u8_advance(self.buffer, pos);
        let code_point_count =
            read_string_advance(self.buffer, max_code_point_count, None, out, pos);
        ShortcutEntry {
            code_point_count,
            probability: probability_from_flags(flags),
            is_whitelist: is_whitelist(flags),
            has_next: has_next(flags),
        }
    }

    /// `ShortcutListPolicy::skipAllShortcuts`: moves `pos` past the whole list.
    pub fn skip_all(&self, pos: &mut i32) {
        let list_size = list_size_advance(self.buffer, pos);
        *pos = pos.wrapping_add(list_size);
    }
}

/// `BinaryDictionaryShortcutIterator`: walks a word's shortcut list, entry by entry.
#[derive(Clone, Copy, Debug)]
pub struct ShortcutIterator<'a> {
    policy: ShortcutListPolicy<'a>,
    pos: i32,
    has_next_target: bool,
}

impl<'a> ShortcutIterator<'a> {
    /// The iterator over the list at `shortcut_pos`, which is NOT_A_DICT_POS for a word with none.
    pub fn new(policy: ShortcutListPolicy<'a>, shortcut_pos: i32) -> Self {
        Self {
            policy,
            pos: policy.start_pos(shortcut_pos),
            has_next_target: shortcut_pos != NOT_A_DICT_POS,
        }
    }

    /// `BinaryDictionaryShortcutIterator::hasNextShortcutTarget`.
    pub fn has_next_shortcut_target(&self) -> bool {
        self.has_next_target
    }

    /// `BinaryDictionaryShortcutIterator::nextShortcutTarget`: the next entry, its target into `out`.
    pub fn next_shortcut_target(&mut self, max_depth: usize, out: &mut [i32]) -> ShortcutEntry {
        let entry = self.policy.next_shortcut(max_depth, out, &mut self.pos);
        self.has_next_target = entry.has_next;
        entry
    }
}
