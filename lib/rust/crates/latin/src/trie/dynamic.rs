//! `dictionary/structure/pt_common/dynamic_pt_reading_helper`: walks the trie node by node, to find
//! the terminal node of a word from the root. Its loops are counted, so a broken dictionary ends
//! the walk instead of looping.

use super::node::{PtNodeParams, Ver2PtNodeArrayReader, Ver2PtNodeReader};
use crate::chars::to_lower_case;
use crate::defines::NOT_A_DICT_POS;

/// `DynamicPtReadingHelper::MAX_CHILD_COUNT_TO_AVOID_INFINITE_LOOP`.
const MAX_CHILD_COUNT_TO_AVOID_INFINITE_LOOP: i32 = 100_000;
/// `DynamicPtReadingHelper::MAX_PT_NODE_ARRAY_COUNT_TO_AVOID_INFINITE_LOOP`.
const MAX_PT_NODE_ARRAY_COUNT_TO_AVOID_INFINITE_LOOP: i32 = 100_000;

/// `DynamicPtReadingHelper::PtNodeReadingState`.
#[derive(Clone, Copy, Debug)]
struct ReadingState {
    pos: i32,
    remaining_pt_node_count_in_this_array: i32,
    total_code_point_count_since_initialization: usize,
    total_pt_node_index_in_this_array_chain: i32,
    pt_node_array_index_in_this_array_chain: i32,
}

impl Default for ReadingState {
    /// `PtNodeReadingState()`.
    fn default() -> Self {
        Self {
            pos: NOT_A_DICT_POS,
            remaining_pt_node_count_in_this_array: 0,
            total_code_point_count_since_initialization: 0,
            total_pt_node_index_in_this_array_chain: 0,
            pt_node_array_index_in_this_array_chain: 0,
        }
    }
}

/// `DynamicPtReadingHelper`: traverses the trie from its root, one node at a time.
#[derive(Clone, Copy, Debug)]
pub struct DynamicPtReadingHelper<'a> {
    is_error: bool,
    state: ReadingState,
    nodes: Ver2PtNodeReader<'a>,
    arrays: Ver2PtNodeArrayReader<'a>,
}

impl<'a> DynamicPtReadingHelper<'a> {
    /// `DynamicPtReadingHelper(ptNodeReader, ptNodeArrayReader)`.
    pub fn new(nodes: Ver2PtNodeReader<'a>, arrays: Ver2PtNodeArrayReader<'a>) -> Self {
        Self {
            is_error: false,
            state: ReadingState::default(),
            nodes,
            arrays,
        }
    }

    /// `DynamicPtReadingHelper::isError`: a read went outside the dictionary or past a loop bound.
    /// It never changes the word id a search returns, as in the C++.
    pub fn is_error(&self) -> bool {
        self.is_error
    }

    /// `DynamicPtReadingHelper::isEnd`.
    pub fn is_end(&self) -> bool {
        self.state.pos == NOT_A_DICT_POS
    }

    /// `DynamicPtReadingHelper::initWithPtNodeArrayPos`: starts at the array at `pos`.
    pub fn init_with_pt_node_array_pos(&mut self, pt_node_array_pos: i32) {
        if pt_node_array_pos == NOT_A_DICT_POS {
            self.state.pos = NOT_A_DICT_POS;
        } else {
            self.is_error = false;
            self.state.pos = pt_node_array_pos;
            self.state.total_code_point_count_since_initialization = 0;
            self.state.total_pt_node_index_in_this_array_chain = 0;
            self.state.pt_node_array_index_in_this_array_chain = 0;
            self.next_pt_node_array();
        }
    }

    /// `DynamicPtReadingHelper::getPtNodeParams`: the node at the current position, or the invalid
    /// node at the end.
    fn pt_node_params(&self) -> PtNodeParams {
        if self.is_end() {
            return PtNodeParams::default();
        }
        self.nodes.fetch(self.state.pos)
    }

    /// `DynamicPtReadingHelper::getTerminalPtNodePositionOfWord`: the position of the terminal node
    /// of `word`, or NOT_A_DICT_POS when the word is not in the dictionary. With
    /// `force_lower_case_search`, each code point is lower-cased before it is compared, as the C++
    /// lower-cases the whole word into a scratch buffer first; here that happens on the fly.
    pub fn terminal_pt_node_position_of_word(
        &mut self,
        word: &[i32],
        force_lower_case_search: bool,
    ) -> i32 {
        let length = word.len();
        while !self.is_end() {
            let node = self.pt_node_params();
            let matched = self.state.total_code_point_count_since_initialization;
            if total_code_point_count(&self.state, &node) > length
                || !self.is_matched_at(&node, 0, word, matched, force_lower_case_search)
            {
                self.read_next_sibling_node(&node);
                continue;
            }
            let node_code_point_count = node.code_point_count();
            for j in 1..node_code_point_count {
                if !self.is_matched_at(&node, j, word, matched + j, force_lower_case_search) {
                    return NOT_A_DICT_POS;
                }
            }
            if length == total_code_point_count(&self.state, &node) {
                return if node.is_terminal() {
                    node.head_pos()
                } else {
                    NOT_A_DICT_POS
                };
            }
            if !node.has_children() {
                return NOT_A_DICT_POS;
            }
            self.read_child_node(&node);
        }
        NOT_A_DICT_POS
    }

    /// `DynamicPtReadingHelper::isMatchedCodePoint`, with the search code point taken from `word`
    /// (lower-cased when asked). A search index past the word's end can't match; the C++ reads past
    /// the end of its buffer there, which only a node with no code points can reach.
    fn is_matched_at(
        &self,
        node: &PtNodeParams,
        index: usize,
        word: &[i32],
        search_index: usize,
        force_lower_case_search: bool,
    ) -> bool {
        let Some(&raw) = word.get(search_index) else {
            return false;
        };
        let search = if force_lower_case_search {
            to_lower_case(raw)
        } else {
            raw
        };
        node.code_point_at(index) == search
    }

    /// `DynamicPtReadingHelper::nextPtNodeArray`: reads the array at the current position, and
    /// follows forward links past empty arrays.
    fn next_pt_node_array(&mut self) {
        let Some((count, first_pos)) = self.arrays.read_info(self.state.pos) else {
            self.enter_error();
            return;
        };
        self.state.remaining_pt_node_count_in_this_array = count;
        self.state.pos = first_pos;
        self.state.total_pt_node_index_in_this_array_chain += count;
        self.state.pt_node_array_index_in_this_array_chain += 1;
        if count < 0
            || self.state.total_pt_node_index_in_this_array_chain
                > MAX_CHILD_COUNT_TO_AVOID_INFINITE_LOOP
            || self.state.pt_node_array_index_in_this_array_chain
                > MAX_PT_NODE_ARRAY_COUNT_TO_AVOID_INFINITE_LOOP
        {
            self.enter_error();
            return;
        }
        if count == 0 {
            self.follow_forward_link();
        }
    }

    /// `DynamicPtReadingHelper::followForwardLink`.
    fn follow_forward_link(&mut self) {
        let Some(next_pos) = self.arrays.read_forward_link(self.state.pos) else {
            self.enter_error();
            return;
        };
        if next_pos != NOT_A_DICT_POS {
            self.state.pos = next_pos;
            self.next_pt_node_array();
        } else {
            self.state.pos = NOT_A_DICT_POS;
        }
    }

    /// `DynamicPtReadingHelper::readNextSiblingNode`.
    fn read_next_sibling_node(&mut self, node: &PtNodeParams) {
        self.state.remaining_pt_node_count_in_this_array -= 1;
        self.state.pos = node.sibling_pos();
        if self.state.remaining_pt_node_count_in_this_array <= 0 {
            self.follow_forward_link();
        }
    }

    /// `DynamicPtReadingHelper::readChildNode`.
    fn read_child_node(&mut self, node: &PtNodeParams) {
        if node.has_children() {
            self.state.total_code_point_count_since_initialization += node.code_point_count();
            self.state.total_pt_node_index_in_this_array_chain = 0;
            self.state.pt_node_array_index_in_this_array_chain = 0;
            self.state.pos = node.children_pos();
            self.next_pt_node_array();
        } else {
            self.state.pos = NOT_A_DICT_POS;
        }
    }

    /// The error the C++ records before it ends the walk.
    fn enter_error(&mut self) {
        self.is_error = true;
        self.state.pos = NOT_A_DICT_POS;
    }
}

/// `DynamicPtReadingHelper::getTotalCodePointCount`: the code points on the path so far, with this
/// node's.
fn total_code_point_count(state: &ReadingState, node: &PtNodeParams) -> usize {
    state.total_code_point_count_since_initialization + node.code_point_count()
}
