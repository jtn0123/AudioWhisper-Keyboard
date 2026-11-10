//! `dictionary/structure/pt_common/patricia_trie_reading_utils`: the flags of a trie node, and how
//! its fields are read from the dictionary's bytes. Positions are into the dictionary after its
//! header, as the C++ has them.

use super::bigram::BigramListPolicy;
use super::node::PtNodeParams;
use super::shortcut::ShortcutListPolicy;
use crate::bytes::{
    advance_past_string, read_code_point_advance, read_string_advance, read_u8_advance,
    read_u16_advance, read_u24_advance,
};
use crate::defines::{MAX_WORD_LENGTH, NOT_A_CODE_POINT, NOT_A_DICT_POS, NOT_A_PROBABILITY};

/// `PatriciaTrieReadingUtils::NodeFlags`: one byte of flags for each node.
pub type NodeFlags = u8;

const MASK_CHILDREN_POSITION_TYPE: NodeFlags = 0xC0;
const FLAG_CHILDREN_POSITION_TYPE_NOPOSITION: NodeFlags = 0x00;
const FLAG_CHILDREN_POSITION_TYPE_ONEBYTE: NodeFlags = 0x40;
const FLAG_CHILDREN_POSITION_TYPE_TWOBYTES: NodeFlags = 0x80;
const FLAG_CHILDREN_POSITION_TYPE_THREEBYTES: NodeFlags = 0xC0;
const FLAG_HAS_MULTIPLE_CHARS: NodeFlags = 0x20;
const FLAG_IS_TERMINAL: NodeFlags = 0x10;
const FLAG_HAS_SHORTCUT_TARGETS: NodeFlags = 0x08;
const FLAG_HAS_BIGRAMS: NodeFlags = 0x04;
const FLAG_IS_NOT_A_WORD: NodeFlags = 0x02;
const FLAG_IS_POSSIBLY_OFFENSIVE: NodeFlags = 0x01;

/// `PatriciaTrieReadingUtils::getPtNodeArraySizeAndAdvancePosition`: a node array's length, in one
/// byte, or two when the first byte's top bit is set.
pub fn pt_node_array_size(buffer: &[u8], pos: &mut i32) -> i32 {
    let first = read_u8_advance(buffer, pos);
    if first < 0x80 {
        i32::from(first)
    } else {
        (i32::from(first & 0x7F) << 8) ^ i32::from(read_u8_advance(buffer, pos))
    }
}

/// `PatriciaTrieReadingUtils::getFlagsAndAdvancePosition`.
pub fn flags_advance(buffer: &[u8], pos: &mut i32) -> NodeFlags {
    read_u8_advance(buffer, pos)
}

/// `PatriciaTrieReadingUtils::getCodePointAndAdvancePosition`.
pub fn code_point_advance(buffer: &[u8], table: Option<&[i32]>, pos: &mut i32) -> i32 {
    read_code_point_advance(buffer, table, pos)
}

/// `PatriciaTrieReadingUtils::getCharsAndAdvancePosition`: the node's code points into `out`, at
/// most `max_length`; returns how many. A one-character node whose code point is the terminator
/// has none (the C++ asserts there).
pub fn chars_advance(
    buffer: &[u8],
    flags: NodeFlags,
    max_length: usize,
    table: Option<&[i32]>,
    out: &mut [i32],
    pos: &mut i32,
) -> usize {
    if has_multiple_chars(flags) {
        return read_string_advance(buffer, max_length, table, out, pos);
    }
    let code_point = code_point_advance(buffer, table, pos);
    if code_point == NOT_A_CODE_POINT {
        0
    } else if max_length > 0 {
        if let Some(first) = out.first_mut() {
            *first = code_point;
        }
        1
    } else {
        0
    }
}

/// `PatriciaTrieReadingUtils::skipCharacters`: skips the node's code points, past the first; returns
/// how many were skipped.
pub fn skip_characters(
    buffer: &[u8],
    flags: NodeFlags,
    max_length: usize,
    table: Option<&[i32]>,
    pos: &mut i32,
) -> usize {
    if has_multiple_chars(flags) {
        advance_past_string(buffer, max_length, pos)
    } else if max_length > 0 {
        code_point_advance(buffer, table, pos);
        1
    } else {
        0
    }
}

/// `PatriciaTrieReadingUtils::readProbabilityAndAdvancePosition`: one byte.
pub fn read_probability(buffer: &[u8], pos: &mut i32) -> i32 {
    i32::from(read_u8_advance(buffer, pos))
}

/// `PatriciaTrieReadingUtils::readChildrenPositionAndAdvancePosition`: the children's position,
/// which is the offset's own position plus the offset. NOT_A_DICT_POS for a node without
/// children.
pub fn read_children_position(buffer: &[u8], flags: NodeFlags, pos: &mut i32) -> i32 {
    let base = *pos;
    let offset = match flags & MASK_CHILDREN_POSITION_TYPE {
        FLAG_CHILDREN_POSITION_TYPE_ONEBYTE => i32::from(read_u8_advance(buffer, pos)),
        FLAG_CHILDREN_POSITION_TYPE_TWOBYTES => i32::from(read_u16_advance(buffer, pos)),
        FLAG_CHILDREN_POSITION_TYPE_THREEBYTES => read_u24_advance(buffer, pos) as i32,
        _ => return NOT_A_DICT_POS,
    };
    base + offset
}

/// `PatriciaTrieReadingUtils::isPossiblyOffensive`.
pub fn is_possibly_offensive(flags: NodeFlags) -> bool {
    flags & FLAG_IS_POSSIBLY_OFFENSIVE != 0
}

/// `PatriciaTrieReadingUtils::isNotAWord`.
pub fn is_not_a_word(flags: NodeFlags) -> bool {
    flags & FLAG_IS_NOT_A_WORD != 0
}

/// `PatriciaTrieReadingUtils::isTerminal`.
pub fn is_terminal(flags: NodeFlags) -> bool {
    flags & FLAG_IS_TERMINAL != 0
}

/// `PatriciaTrieReadingUtils::hasShortcutTargets`.
pub fn has_shortcut_targets(flags: NodeFlags) -> bool {
    flags & FLAG_HAS_SHORTCUT_TARGETS != 0
}

/// `PatriciaTrieReadingUtils::hasBigrams`.
pub fn has_bigrams(flags: NodeFlags) -> bool {
    flags & FLAG_HAS_BIGRAMS != 0
}

/// `PatriciaTrieReadingUtils::hasMultipleChars`.
pub fn has_multiple_chars(flags: NodeFlags) -> bool {
    flags & FLAG_HAS_MULTIPLE_CHARS != 0
}

/// `PatriciaTrieReadingUtils::hasChildrenInFlags`: whether the flags say a children offset follows.
pub fn has_children_in_flags(flags: NodeFlags) -> bool {
    FLAG_CHILDREN_POSITION_TYPE_NOPOSITION != (MASK_CHILDREN_POSITION_TYPE & flags)
}

/// `PatriciaTrieReadingUtils::readPtNodeInfo`: the node at `pos`, with every field read, and its
/// sibling's position after it. The node is returned as read, even one with no code points
/// (the C++ reads those too).
pub fn read_pt_node_info(
    buffer: &[u8],
    pos: i32,
    shortcuts: &ShortcutListPolicy<'_>,
    bigrams: &BigramListPolicy<'_>,
    table: Option<&[i32]>,
) -> PtNodeParams {
    let mut reading = pos;
    let flags = flags_advance(buffer, &mut reading);
    let mut code_points = [0; MAX_WORD_LENGTH];
    let count = chars_advance(
        buffer,
        flags,
        MAX_WORD_LENGTH,
        table,
        &mut code_points,
        &mut reading,
    );
    let probability = if is_terminal(flags) {
        read_probability(buffer, &mut reading)
    } else {
        NOT_A_PROBABILITY
    };
    let children_pos = if has_children_in_flags(flags) {
        read_children_position(buffer, flags, &mut reading)
    } else {
        NOT_A_DICT_POS
    };
    let mut shortcut_pos = NOT_A_DICT_POS;
    if has_shortcut_targets(flags) {
        shortcut_pos = reading;
        shortcuts.skip_all(&mut reading);
    }
    let mut bigram_pos = NOT_A_DICT_POS;
    if has_bigrams(flags) {
        bigram_pos = reading;
        bigrams.skip_all(&mut reading);
    }
    PtNodeParams {
        head_pos: pos,
        flags,
        code_point_count: count as u8,
        code_points,
        probability,
        children_pos,
        shortcut_pos,
        bigram_pos,
        sibling_pos: reading,
    }
}
