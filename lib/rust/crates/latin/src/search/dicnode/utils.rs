//! `suggest/core/dicnode/dic_node_utils`: starting nodes, expanding a node into its children, and
//! the cost of a word in its context.

use super::node::DicNode;
use super::vector::DicNodeVector;
use crate::defines::{MAX_PROBABILITY, MAX_VALUE_FOR_WEIGHTING, NOT_A_PROBABILITY};
use crate::search::children::create_and_get_all_child_dic_nodes;
use crate::trie::multi_bigram::MultiBigramMap;
use crate::trie::patricia::PatriciaTriePolicy;

/// `DicNodeUtils::initAsRoot`.
pub fn init_as_root(policy: &PatriciaTriePolicy, prev_word_ids: &[i32], new_root: &mut DicNode) {
    new_root.init_as_root(policy.root_position(), prev_word_ids);
}

/// `DicNodeUtils::initAsRootWithPreviousWord`.
pub fn init_as_root_with_previous_word(
    policy: &PatriciaTriePolicy,
    prev_word_last_dic_node: &DicNode,
    new_root: &mut DicNode,
) {
    new_root.init_as_root_with_previous_word(prev_word_last_dic_node, policy.root_position());
}

/// `DicNodeUtils::getAllChildDicNodes`: the node one code point further, or the children of the
/// PtNode when the node is at its end.
pub fn get_all_child_dic_nodes(
    dic_node: &DicNode,
    policy: &PatriciaTriePolicy,
    child_dic_nodes: &mut DicNodeVector,
) {
    if dic_node.is_total_input_size_exceeding_limit() {
        return;
    }
    if !dic_node.is_leaving_node() {
        child_dic_nodes.push_passing_child(dic_node);
    } else {
        create_and_get_all_child_dic_nodes(policy, dic_node, child_dic_nodes);
    }
}

/// `DicNodeUtils::getBigramNodeImprobability`: the combined bigram and unigram cost of the node's
/// word.
pub fn get_bigram_node_improbability(
    policy: &PatriciaTriePolicy,
    dic_node: &DicNode,
    multi_bigram_map: Option<&mut MultiBigramMap>,
) -> f32 {
    if dic_node.has_multiple_words() && !dic_node.is_valid_multiple_word_suggestion() {
        return MAX_VALUE_FOR_WEIGHTING as f32;
    }
    let word_attributes = policy.get_word_attributes_in_context(
        dic_node.prev_word_ids(),
        dic_node.word_id(),
        multi_bigram_map,
    );
    if word_attributes.probability() == NOT_A_PROBABILITY
        || (dic_node.has_multiple_words()
            && (word_attributes.is_blacklisted() || word_attributes.is_not_a_word()))
    {
        return MAX_VALUE_FOR_WEIGHTING as f32;
    }
    (MAX_PROBABILITY - word_attributes.probability()) as f32 / MAX_PROBABILITY as f32
}
