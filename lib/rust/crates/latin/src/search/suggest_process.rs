//! The `process*` steps of `suggest/core/suggest.cpp`: what the search does with one node after
//! it has expanded it, one function per kind of correction. They are the other half of
//! [`Search`]; the loop that calls them is in `suggest`.

use crate::defines::{CorrectionType, MAX_VALUE_FOR_WEIGHTING};
use crate::layout::proximity_info_utils::is_match_or_proximity_char;
use crate::search::dicnode::node::DicNode;
use crate::search::dicnode::utils::{get_all_child_dic_nodes, init_as_root_with_previous_word};
use crate::search::result::output_utils::should_block_word;
use crate::search::suggest::Search;
use crate::search::typing::traversal;
use crate::search::typing::weighting::add_cost_and_forward_input_index;

impl Search<'_> {
    /// `Suggest::processTerminalDicNode`: a terminal node is a word to output; it goes to the
    /// terminal queue with the cost of ending there.
    fn process_terminal_dic_node(&mut self, dic_node: &DicNode) {
        if dic_node.compound_distance(1.0) >= MAX_VALUE_FOR_WEIGHTING as f32 {
            return;
        }
        if !dic_node.is_terminal_dic_node() {
            return;
        }
        if dic_node.should_be_filtered_by_safety_net_for_bigram() {
            return;
        }
        if !dic_node.has_matched_or_proximity_code_points() {
            return;
        }
        // Create a non-cached node here.
        let mut terminal_dic_node = *dic_node;
        // `needsToTraverseAllUserInput` is true for typing.
        if i32::from(dic_node.input_index(0)) < self.v.input_size {
            add_cost_and_forward_input_index(
                &self.v,
                CorrectionType::TerminalInsertion,
                None,
                &mut terminal_dic_node,
                Some(&mut *self.multi_bigram_map),
            );
        }
        add_cost_and_forward_input_index(
            &self.v,
            CorrectionType::Terminal,
            None,
            &mut terminal_dic_node,
            Some(&mut *self.multi_bigram_map),
        );
        self.cache.copy_push_terminal(&terminal_dic_node);
    }

    /// `Suggest::processExpandedDicNode`: adds the expanded node to the next search queue, and a
    /// next-word path (the space omission correction) if it is on a terminal.
    pub(super) fn process_expanded_dic_node(&mut self, dic_node: &DicNode) {
        self.process_terminal_dic_node(dic_node);
        if dic_node.compound_distance(1.0) < MAX_VALUE_FOR_WEIGHTING as f32 {
            if traversal::is_space_omission_terminal(&self.v, dic_node) {
                self.create_next_word_dic_node(dic_node, false);
            }
            let allows_look_ahead =
                !(dic_node.has_multiple_words() && dic_node.is_completion(self.v.input_size));
            if dic_node.has_children() && allows_look_ahead {
                self.cache.copy_push_next_active(dic_node);
            }
        }
    }

    /// `Suggest::processDicNodeAsMatch`.
    pub(super) fn process_dic_node_as_match(&mut self, child_dic_node: &mut DicNode) {
        self.weight_child_node(child_dic_node);
        self.process_expanded_dic_node(child_dic_node);
    }

    /// `Suggest::processDicNodeAsAdditionalProximityChar`. Most corrections don't look the
    /// bigram up, since they don't treat the node as a terminal: no bigram map is passed.
    pub(super) fn process_dic_node_as_additional_proximity_char(
        &mut self,
        dic_node: &DicNode,
        child_dic_node: &mut DicNode,
    ) {
        add_cost_and_forward_input_index(
            &self.v,
            CorrectionType::AdditionalProximity,
            Some(dic_node),
            child_dic_node,
            None,
        );
        self.process_expanded_dic_node(child_dic_node);
    }

    /// `Suggest::processDicNodeAsSubstitution`.
    pub(super) fn process_dic_node_as_substitution(
        &mut self,
        dic_node: &DicNode,
        child_dic_node: &mut DicNode,
    ) {
        add_cost_and_forward_input_index(
            &self.v,
            CorrectionType::Substitution,
            Some(dic_node),
            child_dic_node,
            None,
        );
        self.process_expanded_dic_node(child_dic_node);
    }

    /// `Suggest::processDicNodeAsDigraph`: the node's composite letter, like the German u-umlaut,
    /// is expanded to its transliteration ("ue"). It runs in parallel with the normal traversal,
    /// so both "uber" and "ueber" can be corrected to the u-umlaut word.
    pub(super) fn process_dic_node_as_digraph(&mut self, child_dic_node: &mut DicNode) {
        self.weight_child_node(child_dic_node);
        child_dic_node.advance_digraph_index();
        self.process_expanded_dic_node(child_dic_node);
    }

    /// `Suggest::processDicNodeAsOmission` (ths => this): skips the current letter and considers
    /// matches for all possible next letters. Apostrophes are handled as omissions.
    pub(super) fn process_dic_node_as_omission(&mut self, dic_node: &DicNode) {
        let mut child_dic_nodes = std::mem::take(&mut self.scratch.grand_children);
        child_dic_nodes.clear();
        get_all_child_dic_nodes(dic_node, self.v.policy, &mut child_dic_nodes);
        for i in 0..child_dic_nodes.len() {
            let child_dic_node = child_dic_nodes.get_mut(i);
            // Treat this word as omission
            add_cost_and_forward_input_index(
                &self.v,
                CorrectionType::Omission,
                Some(dic_node),
                child_dic_node,
                None,
            );
            self.weight_child_node(child_dic_node);
            if !traversal::is_possible_omission_child_node(&self.v, dic_node, child_dic_node) {
                continue;
            }
            self.process_expanded_dic_node(child_dic_node);
        }
        self.scratch.grand_children = child_dic_nodes;
    }

    /// `Suggest::processDicNodeAsInsertion` (thiis => this): skips the current touch point and
    /// considers matches for the next.
    pub(super) fn process_dic_node_as_insertion(&mut self, dic_node: &DicNode) {
        let point_index = i32::from(dic_node.input_index(0));
        let mut child_dic_nodes = std::mem::take(&mut self.scratch.grand_children);
        child_dic_nodes.clear();
        get_all_child_dic_nodes(dic_node, self.v.policy, &mut child_dic_nodes);
        for i in 0..child_dic_nodes.len() {
            if self.v.state.get_primary_code_point_at(point_index + 1)
                != child_dic_nodes.nodes()[i].node_code_point()
            {
                continue;
            }
            let child_dic_node = child_dic_nodes.get_mut(i);
            add_cost_and_forward_input_index(
                &self.v,
                CorrectionType::Insertion,
                Some(dic_node),
                child_dic_node,
                None,
            );
            self.process_expanded_dic_node(child_dic_node);
        }
        self.scratch.grand_children = child_dic_nodes;
    }

    /// `Suggest::processDicNodeAsTransposition` (thsi => this): swaps the next two touch points.
    pub(super) fn process_dic_node_as_transposition(&mut self, dic_node: &DicNode) {
        let point_index = i32::from(dic_node.input_index(0));
        let mut child_dic_nodes1 = std::mem::take(&mut self.scratch.grand_children);
        let mut child_dic_nodes2 = std::mem::take(&mut self.scratch.second_grand_children);
        child_dic_nodes1.clear();
        get_all_child_dic_nodes(dic_node, self.v.policy, &mut child_dic_nodes1);
        for i in 0..child_dic_nodes1.len() {
            let child1 = child_dic_nodes1.nodes()[i];
            let matched_id1 = self.v.state.get_proximity_type(
                point_index + 1,
                child1.node_code_point(),
                true,
                None,
            );
            if !is_match_or_proximity_char(matched_id1) {
                continue;
            }
            if child1.has_children() {
                child_dic_nodes2.clear();
                get_all_child_dic_nodes(&child1, self.v.policy, &mut child_dic_nodes2);
                for j in 0..child_dic_nodes2.len() {
                    let child_dic_node2 = child_dic_nodes2.get_mut(j);
                    let matched_id2 = self.v.state.get_proximity_type(
                        point_index,
                        child_dic_node2.node_code_point(),
                        true,
                        None,
                    );
                    if !is_match_or_proximity_char(matched_id2) {
                        continue;
                    }
                    add_cost_and_forward_input_index(
                        &self.v,
                        CorrectionType::Transposition,
                        Some(&child1),
                        child_dic_node2,
                        None,
                    );
                    self.process_expanded_dic_node(child_dic_node2);
                }
            }
        }
        self.scratch.grand_children = child_dic_nodes1;
        self.scratch.second_grand_children = child_dic_nodes2;
    }

    /// `Suggest::weightChildNode`: weights a child node by aligning it to the key.
    fn weight_child_node(&mut self, dic_node: &mut DicNode) {
        let correction_type = if dic_node.is_completion(self.v.input_size) {
            CorrectionType::Completion
        } else {
            CorrectionType::Match
        };
        add_cost_and_forward_input_index(&self.v, correction_type, None, dic_node, None);
    }

    /// `Suggest::createNextWordDicNode`: a new node for a space after `dic_node`, the unigram or
    /// bigram score of the word it ends going into the new node.
    pub(super) fn create_next_word_dic_node(
        &mut self,
        dic_node: &DicNode,
        space_substitution: bool,
    ) {
        let word_attributes = self.v.policy.get_word_attributes_in_context(
            dic_node.prev_word_ids(),
            dic_node.word_id(),
            Some(&mut *self.multi_bigram_map),
        );
        if should_block_word(&self.v.options, dic_node, &word_attributes, false) {
            return;
        }
        if !traversal::is_good_to_traverse_next_word(dic_node, word_attributes.probability()) {
            return;
        }
        // Create a non-cached node here.
        let mut new_dic_node = DicNode::default();
        init_as_root_with_previous_word(self.v.policy, dic_node, &mut new_dic_node);
        let correction_type = if space_substitution {
            CorrectionType::NewWordSpaceSubstitution
        } else {
            CorrectionType::NewWordSpaceOmission
        };
        add_cost_and_forward_input_index(
            &self.v,
            correction_type,
            Some(dic_node),
            &mut new_dic_node,
            Some(&mut *self.multi_bigram_map),
        );
        if new_dic_node.compound_distance(1.0) < MAX_VALUE_FOR_WEIGHTING as f32 {
            // newDicNode is worth continuing to traverse. CAVEAT (C++): this pruning is important
            // for speed; it should take place only in the priority queue.
            self.cache.copy_push_next_active(&new_dic_node);
        }
    }
}
