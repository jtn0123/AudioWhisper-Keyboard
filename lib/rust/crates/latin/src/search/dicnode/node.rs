//! `suggest/core/dicnode/dic_node` and `internal/dic_node_state`: one candidate of the search, a
//! position in the trie with the word spelt so far and its cost.
//!
//! `addCost` and `forwardInputIndex` are the C++'s `friend class Weighting`; here they are
//! visible to the whole of `search`. The gesture-only `DicNode_InputStateG` and `updateInputIndexG`
//! are not ported: typing never asks for them (`mNeedsToUpdateInputStateG` stays false).

use super::properties::DicNodeProperties;
use super::state_input::DicNodeStateInput;
use super::state_output::DicNodeStateOutput;
use super::state_scoring::DicNodeStateScoring;
use crate::chars;
use crate::defines::{
    KEYCODE_SINGLE_QUOTE, MAX_PREV_WORD_COUNT_FOR_N_GRAM, MAX_WORD_LENGTH, NOT_A_WORD_ID,
    NOT_AN_INDEX,
};
use crate::layout::proximity_info_state::ProximityInfoState;
use crate::search::digraph::{self, DigraphCodePointIndex};
use crate::search::error_type::{self, ErrorType};

/// `MIN_DIFF` of `DicNode::compare`.
const MIN_DIFF: f32 = 0.000001;

/// `DicNode`. It is `Copy`: the C++ copies nodes by value between the pool and its queues.
#[derive(Clone, Copy, Debug, Default)]
pub struct DicNode {
    /// `mDicNodeProperties`.
    properties: DicNodeProperties,
    /// `mDicNodeState.mDicNodeStateInput`.
    input: DicNodeStateInput,
    /// `mDicNodeState.mDicNodeStateOutput`.
    output: DicNodeStateOutput,
    /// `mDicNodeState.mDicNodeStateScoring`.
    scoring: DicNodeStateScoring,
    /// `mIsCachedForNextSuggestion`: set at the cache border, never read (as in the C++).
    is_cached_for_next_suggestion: bool,
}

impl DicNode {
    /// `DicNode::initByCopy`.
    pub fn init_by_copy(&mut self, dic_node: &Self) {
        self.is_cached_for_next_suggestion = dic_node.is_cached_for_next_suggestion;
        self.properties.init_by_copy(&dic_node.properties);
        self.input.init_by_copy(&dic_node.input);
        self.output.init_by_copy(&dic_node.output);
        self.scoring.init_by_copy(&dic_node.scoring);
    }

    /// `DicNode::initAsRoot`.
    pub fn init_as_root(&mut self, root_pt_node_array_pos: i32, prev_word_ids: &[i32]) {
        self.is_cached_for_next_suggestion = false;
        self.properties
            .init_root(root_pt_node_array_pos, prev_word_ids);
        self.input.init();
        self.output.init();
        self.scoring.init();
    }

    /// `DicNode::initAsRootWithPreviousWord`: the root of the next word, after `dic_node`.
    ///
    /// The new previous word ids are the parent's word id then its first previous ids, always
    /// `MAX_PREV_WORD_COUNT_FOR_N_GRAM` of them. The C++ leaves the slots after those it copies
    /// uninitialised and no consumer reads them; here they are `NOT_A_WORD_ID`.
    pub fn init_as_root_with_previous_word(
        &mut self,
        dic_node: &Self,
        root_pt_node_array_pos: i32,
    ) {
        self.is_cached_for_next_suggestion = dic_node.is_cached_for_next_suggestion;
        let mut new_prev_word_ids = [NOT_A_WORD_ID; MAX_PREV_WORD_COUNT_FOR_N_GRAM];
        new_prev_word_ids[0] = dic_node.properties.word_id();
        let kept = dic_node
            .properties
            .prev_word_ids()
            .len()
            .min(MAX_PREV_WORD_COUNT_FOR_N_GRAM - 1);
        new_prev_word_ids[1..=kept].copy_from_slice(&dic_node.properties.prev_word_ids()[..kept]);
        self.properties
            .init_root(root_pt_node_array_pos, &new_prev_word_ids);
        self.output.init_for_next_word(&dic_node.output);
        // `init(src, true)`: resets the terminal difference cost, which is not ported.
        self.input.init_by_copy(&dic_node.input);
        self.scoring.init_by_copy(&dic_node.scoring);
    }

    /// `DicNode::initAsPassingChild`: the node one code point further into the same PtNode.
    pub fn init_as_passing_child(&mut self, parent: &Self) {
        self.is_cached_for_next_suggestion = parent.is_cached_for_next_suggestion;
        let code_point = parent
            .output
            .current_word_code_point_at(usize::from(parent.node_code_point_count()));
        self.properties
            .init_passing_child(&parent.properties, code_point);
        self.input.init_by_copy(&parent.input);
        self.output.init_by_copy(&parent.output);
        self.scoring.init_by_copy(&parent.scoring);
    }

    /// `DicNode::initAsChild`: the node for a child PtNode, with its merged code points.
    /// `merged_code_points` is not empty (the caller skips an empty PtNode).
    pub fn init_as_child(
        &mut self,
        dic_node: &Self,
        children_pt_node_array_pos: i32,
        word_id: i32,
        merged_code_points: &[i32],
    ) {
        let new_depth = dic_node.node_code_point_count().wrapping_add(1);
        self.is_cached_for_next_suggestion = dic_node.is_cached_for_next_suggestion;
        let new_leaving_depth = (dic_node.properties.leaving_depth() as usize)
            .wrapping_add(merged_code_points.len()) as u16;
        self.properties.init_child(
            children_pt_node_array_pos,
            merged_code_points[0],
            word_id,
            new_depth,
            new_leaving_depth,
            dic_node.properties.prev_word_ids(),
        );
        self.input.init_by_copy(&dic_node.input);
        self.output.init_by_copy(&dic_node.output);
        self.output.add_merged_node_code_points(merged_code_points);
        self.scoring.init_by_copy(&dic_node.scoring);
    }

    /// `DicNode::hasChildren`.
    pub fn has_children(&self) -> bool {
        self.properties.has_children()
    }

    /// `DicNode::isLeavingNode`.
    pub fn is_leaving_node(&self) -> bool {
        self.node_code_point_count() == self.properties.leaving_depth()
    }

    /// `DicNode::setCached`.
    pub fn set_cached(&mut self) {
        self.is_cached_for_next_suggestion = true;
    }

    /// `DicNode::isValidMultipleWordSuggestion`.
    pub fn is_valid_multiple_word_suggestion(&self) -> bool {
        let prev_word_len = i32::from(self.output.prev_words_length())
            - i32::from(self.output.prev_word_start())
            - 1;
        let current_word_len = i32::from(self.node_code_point_count());
        prev_word_len != 1 || current_word_len != 1
    }

    /// `DicNode::isFirstCharUppercase`.
    pub fn is_first_char_uppercase(&self) -> bool {
        chars::is_ascii_upper(self.output.current_word_code_point_at(0))
    }

    /// `DicNode::isCompletion`.
    pub fn is_completion(&self, input_size: i32) -> bool {
        self.input.input_index(0) >= input_size
    }

    /// `DicNode::canDoLookAheadCorrection`.
    pub fn can_do_look_ahead_correction(&self, input_size: i32) -> bool {
        self.input.input_index(0) < input_size - 1
    }

    /// `DicNode::getWordId`.
    pub fn word_id(&self) -> i32 {
        self.properties.word_id()
    }

    /// `DicNode::getPrevWordIds`.
    pub fn prev_word_ids(&self) -> &[i32] {
        self.properties.prev_word_ids()
    }

    /// `DicNode::getChildrenPtNodeArrayPos`.
    pub fn children_pt_node_array_pos(&self) -> i32 {
        self.properties.children_pt_node_array_pos()
    }

    /// `DicNode::isTerminalDicNode`.
    pub fn is_terminal_dic_node(&self) -> bool {
        let current_depth = self.node_code_point_count();
        self.properties.is_terminal()
            && current_depth > 0
            && current_depth == self.properties.leaving_depth()
    }

    /// `DicNode::shouldBeFilteredBySafetyNetForBigram`.
    pub fn should_be_filtered_by_safety_net_for_bigram(&self) -> bool {
        let current_depth = self.node_code_point_count();
        let prev_word_len = i32::from(self.output.prev_words_length())
            - i32::from(self.output.prev_word_start())
            - 1;
        !(current_depth > 0 && (current_depth != 1 || prev_word_len != 1))
    }

    /// `DicNode::hasMatchedOrProximityCodePoints`.
    pub fn has_matched_or_proximity_code_points(&self) -> bool {
        let edit_correction_count = i32::from(self.scoring.edit_correction_count());
        let completion_count = i32::from(self.scoring.completion_count());
        edit_correction_count + completion_count < i32::from(self.node_code_point_count())
    }

    /// `DicNode::isTotalInputSizeExceedingLimit`.
    pub fn is_total_input_size_exceeding_limit(&self) -> bool {
        i32::from(self.total_node_code_point_count()) > MAX_WORD_LENGTH as i32 - 3
    }

    /// `DicNode::outputResult`: the word's code points into `dest`. The C++ `memmove`s the total
    /// count from a buffer of `MAX_WORD_LENGTH`; a total above that is clamped here.
    pub fn output_result(&self, dest: &mut [i32]) {
        let count = usize::from(self.total_node_code_point_count())
            .min(MAX_WORD_LENGTH)
            .min(dest.len());
        dest[..count].copy_from_slice(&self.output.code_point_buf()[..count]);
    }

    /// `DicNode::getTotalNodeSpaceCount`.
    pub fn total_node_space_count(&self) -> i32 {
        if !self.has_multiple_words() {
            return 0;
        }
        let len = usize::try_from(self.output.prev_words_length())
            .unwrap_or(0)
            .min(MAX_WORD_LENGTH);
        chars::space_count(&self.output.code_point_buf()[..len])
    }

    /// `DicNode::getSecondWordFirstInputIndex`.
    pub fn second_word_first_input_index(&self, p_info_state: &ProximityInfoState) -> i32 {
        let input_index = self.output.second_word_first_input_index();
        if input_index == NOT_AN_INDEX {
            NOT_AN_INDEX
        } else {
            p_info_state.get_input_index_of_sampled_point(input_index)
        }
    }

    /// `DicNode::hasMultipleWords`.
    pub fn has_multiple_words(&self) -> bool {
        self.output.prev_word_count() > 0
    }

    /// `DicNode::getProximityCorrectionCount`.
    pub fn proximity_correction_count(&self) -> i32 {
        i32::from(self.scoring.proximity_correction_count())
    }

    /// `DicNode::getEditCorrectionCount`.
    pub fn edit_correction_count(&self) -> i32 {
        i32::from(self.scoring.edit_correction_count())
    }

    /// `DicNode::getNormalizedCompoundDistance`.
    pub fn normalized_compound_distance(&self) -> f32 {
        self.scoring.normalized_compound_distance()
    }

    /// `DicNode::getNormalizedSpatialDistance`.
    pub fn normalized_spatial_distance(&self) -> f32 {
        self.scoring.spatial_distance() / ((i32::from(self.input_index(0)) + 1) as f32)
    }

    /// `DicNode::getCompoundDistance(weightOfLangModelVsSpatialModel)`.
    pub fn compound_distance(&self, weight_of_lang_model_vs_spatial_model: f32) -> f32 {
        self.scoring
            .compound_distance(weight_of_lang_model_vs_spatial_model)
    }

    /// `DicNode::getOutputWordBuf`.
    pub fn output_word_buf(&self) -> &[i32; MAX_WORD_LENGTH] {
        self.output.code_point_buf()
    }

    /// `DicNode::canBeIntentionalOmission`.
    pub fn can_be_intentional_omission(&self) -> bool {
        chars::is_intentional_omission_code_point(self.node_code_point())
    }

    /// `DicNode::isZeroCostOmission`.
    pub fn is_zero_cost_omission(&self) -> bool {
        self.node_code_point() == KEYCODE_SINGLE_QUOTE
    }

    /// `DicNode::getNodeCodePoint`: the PtNode's code point, or the digraph's current one.
    pub fn node_code_point(&self) -> i32 {
        let code_point = self.properties.dic_node_code_point();
        let digraph_index = self.scoring.digraph_index();
        if digraph_index == DigraphCodePointIndex::NotADigraphIndex {
            return code_point;
        }
        digraph::digraph_code_point_for_index(code_point, digraph_index)
    }

    /// `DicNode::isSameNodeCodePoint`.
    pub fn is_same_node_code_point(&self, dic_node: &Self) -> bool {
        self.properties.dic_node_code_point() == dic_node.properties.dic_node_code_point()
    }

    /// `DicNode::getInputIndex`.
    pub fn input_index(&self, pointer_id: usize) -> i16 {
        self.input.input_index(pointer_id) as i16
    }

    /// `DicNode::getNormalizedCompoundDistanceAfterFirstWord`.
    pub fn normalized_compound_distance_after_first_word(&self) -> f32 {
        self.scoring.normalized_compound_distance_after_first_word()
    }

    /// `DicNode::isInDigraph`.
    pub fn is_in_digraph(&self) -> bool {
        self.scoring.digraph_index() != DigraphCodePointIndex::NotADigraphIndex
    }

    /// `DicNode::advanceDigraphIndex`.
    pub fn advance_digraph_index(&mut self) {
        self.scoring.advance_digraph_index();
    }

    /// `DicNode::getContainedErrorTypes`.
    pub fn contained_error_types(&self) -> ErrorType {
        self.scoring.contained_error_types()
    }

    /// `DicNode::getNodeCodePointCount`: the depth.
    pub fn node_code_point_count(&self) -> u16 {
        self.properties.depth()
    }

    /// `DicNode::getTotalNodeCodePointCount`: the depth plus the length of the words before.
    pub fn total_node_code_point_count(&self) -> u16 {
        self.node_code_point_count()
            .wrapping_add(self.output.prev_words_length() as u16)
    }

    /// `DicNode::compare`: whether this node is worse than `right` (it sorts as "less"). The C++
    /// last resort `this > right` compares the nodes' addresses; every node lives in one pool, so
    /// the caller gives `self_after_right`, whether this node's pool slot is after `right`'s.
    pub fn compare(&self, right: &Self, self_after_right: bool) -> bool {
        let left_exact_match = error_type::is_exact_match(self.contained_error_types());
        let right_exact_match = error_type::is_exact_match(right.contained_error_types());
        if left_exact_match != right_exact_match {
            return left_exact_match;
        }
        let diff = right.normalized_compound_distance() - self.normalized_compound_distance();
        if diff > MIN_DIFF {
            return true;
        } else if diff < -MIN_DIFF {
            return false;
        }
        let depth = i32::from(self.node_code_point_count());
        let depth_diff = i32::from(right.node_code_point_count()) - depth;
        if depth_diff != 0 {
            return depth_diff > 0;
        }
        for i in 0..depth as usize {
            let code_point = self.output.current_word_code_point_at(i);
            let right_code_point = right.output.current_word_code_point_at(i);
            if code_point != right_code_point {
                return right_code_point > code_point;
            }
        }
        self_after_right
    }

    /// `DicNode::addCost` (the C++'s `friend Weighting`), with `doNormalization` false: see
    /// [`DicNodeStateScoring::add_cost`]. `getTotalInputIndex` only feeds the normalization, so it
    /// is not ported.
    pub(in crate::search) fn add_cost(
        &mut self,
        spatial_cost: f32,
        language_cost: f32,
        error_type: ErrorType,
    ) {
        self.scoring
            .add_cost(spatial_cost, language_cost, error_type);
    }

    /// `DicNode::saveNormalizedCompoundDistanceAfterFirstWordIfNoneYet`.
    pub(in crate::search) fn save_normalized_compound_distance_after_first_word_if_none_yet(
        &mut self,
    ) {
        self.scoring
            .save_normalized_compound_distance_after_first_word_if_none_yet();
    }

    /// `DicNode::forwardInputIndex`. `overwritesPrevCodePointByNodeCodePoint` (true for a
    /// transposition) only sets the previous code point, which typing never reads and which is
    /// not ported.
    pub(in crate::search) fn forward_input_index(&mut self, pointer_id: usize, count: i32) {
        if count == 0 {
            return;
        }
        self.input.forward_input_index(pointer_id, count);
    }
}
