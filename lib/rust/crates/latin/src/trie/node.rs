//! `dictionary/structure/pt_common/pt_node_params`, `pt_node_reader` and `pt_node_array_reader`,
//! and their version 2 readings (`dictionary/structure/v2/ver2_patricia_trie_node_reader`,
//! `ver2_pt_node_array_reader`): a node of the trie, and the arrays the nodes are in.

use super::bigram::BigramListPolicy;
use super::reading::{NodeFlags, is_not_a_word, is_possibly_offensive, is_terminal};
use super::reading::{has_bigrams, has_shortcut_targets, pt_node_array_size, read_pt_node_info};
use super::shortcut::ShortcutListPolicy;
use crate::defines::{MAX_WORD_LENGTH, NOT_A_DICT_POS, NOT_A_PROBABILITY};

/// `PtNodeParams`: one node of the trie, as read. A node with no code points is invalid, and
/// so is one read from outside the dictionary; those have the defaults `PtNodeParams()` has.
#[derive(Clone, Copy, Debug)]
pub struct PtNodeParams {
    pub(super) head_pos: i32,
    pub(super) flags: NodeFlags,
    pub(super) code_point_count: u8,
    pub(super) code_points: [i32; MAX_WORD_LENGTH],
    pub(super) probability: i32,
    pub(super) children_pos: i32,
    pub(super) shortcut_pos: i32,
    pub(super) bigram_pos: i32,
    pub(super) sibling_pos: i32,
}

impl Default for PtNodeParams {
    /// `PtNodeParams()`: the invalid node.
    fn default() -> Self {
        Self {
            head_pos: NOT_A_DICT_POS,
            flags: 0,
            code_point_count: 0,
            code_points: [0; MAX_WORD_LENGTH],
            probability: NOT_A_PROBABILITY,
            children_pos: NOT_A_DICT_POS,
            shortcut_pos: NOT_A_DICT_POS,
            bigram_pos: NOT_A_DICT_POS,
            sibling_pos: NOT_A_DICT_POS,
        }
    }
}

impl PtNodeParams {
    /// `PtNodeParams::isValid`: the node has code points.
    pub fn is_valid(&self) -> bool {
        self.code_point_count > 0
    }

    /// `PtNodeParams::getHeadPos`: where the node starts, its word id if it is a word.
    pub fn head_pos(&self) -> i32 {
        self.head_pos
    }

    /// `PtNodeParams::getCodePointCount`.
    pub fn code_point_count(&self) -> usize {
        usize::from(self.code_point_count)
    }

    /// `PtNodeParams::getCodePoints`: the node's code points, as many as it has.
    pub fn code_points(&self) -> &[i32] {
        &self.code_points[..self.code_point_count()]
    }

    /// `PtNodeParams::getCodePoints()[index]`, for any index below the array's length: a node's
    /// code points past its count are zero.
    pub fn code_point_at(&self, index: usize) -> i32 {
        self.code_points[index]
    }

    /// `PtNodeParams::hasChildren`: the node has a children array.
    pub fn has_children(&self) -> bool {
        self.children_pos != NOT_A_DICT_POS
    }

    /// `PtNodeParams::isTerminal`.
    pub fn is_terminal(&self) -> bool {
        is_terminal(self.flags)
    }

    /// `PtNodeParams::isNotAWord`.
    pub fn is_not_a_word(&self) -> bool {
        is_not_a_word(self.flags)
    }

    /// `PtNodeParams::isPossiblyOffensive`.
    pub fn is_possibly_offensive(&self) -> bool {
        is_possibly_offensive(self.flags)
    }

    /// `PtNodeParams::hasBigrams`.
    pub fn has_bigrams(&self) -> bool {
        has_bigrams(self.flags)
    }

    /// `PtNodeParams::hasShortcutTargets`.
    pub fn has_shortcut_targets(&self) -> bool {
        has_shortcut_targets(self.flags)
    }

    /// `PtNodeParams::getProbability`: the unigram probability, or NOT_A_PROBABILITY.
    pub fn probability(&self) -> i32 {
        self.probability
    }

    /// `PtNodeParams::getChildrenPos`: the children array's position.
    pub fn children_pos(&self) -> i32 {
        self.children_pos
    }

    /// `PtNodeParams::getShortcutPos`.
    pub fn shortcut_pos(&self) -> i32 {
        self.shortcut_pos
    }

    /// `PtNodeParams::getBigramsPos`.
    pub fn bigram_pos(&self) -> i32 {
        self.bigram_pos
    }

    /// `PtNodeParams::getSiblingNodePos`: the next node in the array, or what follows the array.
    pub fn sibling_pos(&self) -> i32 {
        self.sibling_pos
    }

    /// The node's raw flags (`PatriciaTrieReadingUtils::readPtNodeInfo`'s `outFlags`).
    pub fn flags(&self) -> NodeFlags {
        self.flags
    }
}

/// `isValidPos` of `PatriciaTriePolicy`: whether `pos` is inside `buffer`.
pub(super) fn is_valid_pos(buffer: &[u8], pos: i32) -> bool {
    pos >= 0 && (pos as usize) < buffer.len()
}

/// `Ver2ParticiaTrieNodeReader`: reads the nodes of a version 2 dictionary's body.
#[derive(Clone, Copy, Debug)]
pub struct Ver2PtNodeReader<'a> {
    buffer: &'a [u8],
    table: Option<&'a [i32]>,
}

impl<'a> Ver2PtNodeReader<'a> {
    /// The reader over the dictionary body `buffer`; `table` is the header's code point table.
    pub fn new(buffer: &'a [u8], table: Option<&'a [i32]>) -> Self {
        Self { buffer, table }
    }

    /// `PatriciaTriePolicy::isValidPos`: whether `pos` is in the body.
    pub fn is_valid_pos(&self, pos: i32) -> bool {
        is_valid_pos(self.buffer, pos)
    }

    /// `fetchPtNodeParamsInBufferFromPtNodePos`: the node at `pos`, or the invalid node when `pos` is
    /// outside the body or the node has no code points. The C++ asserts on both.
    pub fn fetch(&self, pos: i32) -> PtNodeParams {
        if !self.is_valid_pos(pos) {
            return PtNodeParams::default();
        }
        let node = self.read_info(pos);
        if node.is_valid() {
            node
        } else {
            PtNodeParams::default()
        }
    }

    /// `PatriciaTrieReadingUtils::readPtNodeInfo` over this body, with no validity check: what a
    /// node holds, even when it has no code points. A position outside the body has no node, so it
    /// gives the default node without reading: the C++ reads past its buffer there (undefined
    /// behaviour), and the byte reads would overflow for positions near `i32::MAX`.
    pub fn read_info(&self, pos: i32) -> PtNodeParams {
        if !is_valid_pos(self.buffer, pos) {
            return PtNodeParams::default();
        }
        read_pt_node_info(
            self.buffer,
            pos,
            &ShortcutListPolicy::new(self.buffer),
            &BigramListPolicy::new(self.buffer),
            self.table,
        )
    }
}

/// `Ver2PtNodeArrayReader`: reads the arrays of nodes of a version 2 dictionary's body.
#[derive(Clone, Copy, Debug)]
pub struct Ver2PtNodeArrayReader<'a> {
    buffer: &'a [u8],
}

impl<'a> Ver2PtNodeArrayReader<'a> {
    /// The reader over the dictionary body `buffer`.
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer }
    }

    /// `readPtNodeArrayInfoAndReturnIfValid`: the array's node count and the position of its first
    /// node, or `None` when `pos` is outside the body.
    pub fn read_info(&self, pos: i32) -> Option<(i32, i32)> {
        if !is_valid_pos(self.buffer, pos) {
            return None;
        }
        let mut reading = pos;
        let count = pt_node_array_size(self.buffer, &mut reading);
        Some((count, reading))
    }

    /// `readForwardLinkAndReturnIfValid`: the next array, which version 2 dictionaries never have,
    /// so NOT_A_DICT_POS; `None` when `pos` is outside the body.
    pub fn read_forward_link(&self, pos: i32) -> Option<i32> {
        is_valid_pos(self.buffer, pos).then_some(NOT_A_DICT_POS)
    }
}
