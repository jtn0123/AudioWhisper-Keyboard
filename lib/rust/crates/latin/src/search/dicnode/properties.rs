//! `suggest/core/dicnode/internal/dic_node_properties`: what a DicNode knows of its PtNode in the
//! trie.

use crate::defines::{
    MAX_PREV_WORD_COUNT_FOR_N_GRAM, NOT_A_CODE_POINT, NOT_A_DICT_POS, NOT_A_WORD_ID,
};

/// `DicNodeProperties`: the trie position, code point, word and depth of a node, and the ids of
/// the words before it.
#[derive(Clone, Copy, Debug)]
pub struct DicNodeProperties {
    /// `mChildrenPtNodeArrayPos`.
    children_pt_node_array_pos: i32,
    /// `mDicNodeCodePoint`.
    dic_node_code_point: i32,
    /// `mWordId`.
    word_id: i32,
    /// `mDepth`.
    depth: u16,
    /// `mLeavingDepth`.
    leaving_depth: u16,
    /// `mPrevWordIds`.
    prev_word_ids: [i32; MAX_PREV_WORD_COUNT_FOR_N_GRAM],
    /// How many of `prev_word_ids` are in use (the size of the C++ array view).
    prev_word_count: usize,
}

impl Default for DicNodeProperties {
    /// `DicNodeProperties()`. The C++ leaves the previous word ids unset; they start as
    /// `NOT_A_WORD_ID` here, and only the first `prev_word_count` are ever read.
    fn default() -> Self {
        Self {
            children_pt_node_array_pos: NOT_A_DICT_POS,
            dic_node_code_point: NOT_A_CODE_POINT,
            word_id: NOT_A_WORD_ID,
            depth: 0,
            leaving_depth: 0,
            prev_word_ids: [NOT_A_WORD_ID; MAX_PREV_WORD_COUNT_FOR_N_GRAM],
            prev_word_count: 0,
        }
    }
}

impl DicNodeProperties {
    /// `DicNodeProperties::init(childrenPos, nodeCodePoint, wordId, depth, leavingDepth, prevWordIds)`.
    pub fn init_child(
        &mut self,
        children_pos: i32,
        node_code_point: i32,
        word_id: i32,
        depth: u16,
        leaving_depth: u16,
        prev_word_ids: &[i32],
    ) {
        self.children_pt_node_array_pos = children_pos;
        self.dic_node_code_point = node_code_point;
        self.word_id = word_id;
        self.depth = depth;
        self.leaving_depth = leaving_depth;
        self.set_prev_word_ids(prev_word_ids);
    }

    /// `DicNodeProperties::init(rootPtNodeArrayPos, prevWordIds)`: a root.
    pub fn init_root(&mut self, root_pt_node_array_pos: i32, prev_word_ids: &[i32]) {
        self.children_pt_node_array_pos = root_pt_node_array_pos;
        self.dic_node_code_point = NOT_A_CODE_POINT;
        self.word_id = NOT_A_WORD_ID;
        self.depth = 0;
        self.leaving_depth = 0;
        self.set_prev_word_ids(prev_word_ids);
    }

    /// `DicNodeProperties::initByCopy`.
    pub fn init_by_copy(&mut self, other: &Self) {
        self.children_pt_node_array_pos = other.children_pt_node_array_pos;
        self.dic_node_code_point = other.dic_node_code_point;
        self.word_id = other.word_id;
        self.depth = other.depth;
        self.leaving_depth = other.leaving_depth;
        self.set_prev_word_ids(other.prev_word_ids());
    }

    /// `DicNodeProperties::init(dicNodeProp, codePoint)`: a passing child, which is the node one
    /// code point deeper in the same PtNode.
    pub fn init_passing_child(&mut self, other: &Self, code_point: i32) {
        self.children_pt_node_array_pos = other.children_pt_node_array_pos;
        self.dic_node_code_point = code_point; // Overwrite the node char of a passing child
        self.word_id = other.word_id;
        self.depth = other.depth.wrapping_add(1); // Increment the depth of a passing child
        self.leaving_depth = other.leaving_depth;
        self.set_prev_word_ids(other.prev_word_ids());
    }

    /// `prevWordIds.copyToArray(&mPrevWordIds, 0)` and `mPrevWordCount = prevWordIds.size()`: only
    /// the given ids are written; the slots after them are not touched.
    fn set_prev_word_ids(&mut self, prev_word_ids: &[i32]) {
        let count = prev_word_ids.len().min(MAX_PREV_WORD_COUNT_FOR_N_GRAM);
        self.prev_word_ids[..count].copy_from_slice(&prev_word_ids[..count]);
        self.prev_word_count = count;
    }

    /// `getChildrenPtNodeArrayPos`.
    pub fn children_pt_node_array_pos(&self) -> i32 {
        self.children_pt_node_array_pos
    }

    /// `getDicNodeCodePoint`.
    pub fn dic_node_code_point(&self) -> i32 {
        self.dic_node_code_point
    }

    /// `getDepth`.
    pub fn depth(&self) -> u16 {
        self.depth
    }

    /// `getLeavingDepth`.
    pub fn leaving_depth(&self) -> u16 {
        self.leaving_depth
    }

    /// `isTerminal`.
    pub fn is_terminal(&self) -> bool {
        self.word_id != NOT_A_WORD_ID
    }

    /// `hasChildren`.
    pub fn has_children(&self) -> bool {
        self.children_pt_node_array_pos != NOT_A_DICT_POS || self.depth != self.leaving_depth
    }

    /// `getPrevWordIds`: the ids of the words before this node's.
    pub fn prev_word_ids(&self) -> &[i32] {
        &self.prev_word_ids[..self.prev_word_count]
    }

    /// `getWordId`.
    pub fn word_id(&self) -> i32 {
        self.word_id
    }
}
