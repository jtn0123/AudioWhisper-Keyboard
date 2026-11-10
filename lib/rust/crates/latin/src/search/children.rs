//! The two trie policy methods the search calls to expand a node:
//! `PatriciaTriePolicy::createAndGetAllChildDicNodes` and `createAndGetLeavingChildNode`
//! (`dictionary/structure/v2/patricia_trie_policy.cpp`).

use crate::chars;
use crate::defines::NOT_A_WORD_ID;
use crate::search::dicnode::node::DicNode;
use crate::search::dicnode::vector::DicNodeVector;
use crate::trie::patricia::PatriciaTriePolicy;

/// `createAndGetAllChildDicNodes`: a node for every PtNode in the children array of `dic_node`.
pub fn create_and_get_all_child_dic_nodes(
    policy: &PatriciaTriePolicy,
    dic_node: &DicNode,
    child_dic_nodes: &mut DicNodeVector,
) {
    if !dic_node.has_children() {
        return;
    }
    let Some((child_count, mut next_pos)) = policy
        .array_reader()
        .read_info(dic_node.children_pt_node_array_pos())
    else {
        return;
    };
    for _ in 0..child_count {
        if !policy.is_valid_pos(next_pos) {
            return;
        }
        next_pos = create_and_get_leaving_child_node(policy, dic_node, next_pos, child_dic_nodes);
    }
}

/// `createAndGetLeavingChildNode`: the node for the PtNode at `pt_node_pos`, unless it starts
/// with something that is not a Unicode code point (non-word information); returns the position
/// of the next sibling. A PtNode with no code points (its first one is read from an unset array in
/// the C++) gets no node.
fn create_and_get_leaving_child_node(
    policy: &PatriciaTriePolicy,
    dic_node: &DicNode,
    pt_node_pos: i32,
    child_dic_nodes: &mut DicNodeVector,
) -> i32 {
    let node = policy.node_reader().read_info(pt_node_pos);
    let merged_code_points = node.code_points();
    if merged_code_points
        .first()
        .is_some_and(|&c| chars::is_in_unicode_space(c))
    {
        let word_id = if node.is_terminal() {
            pt_node_pos
        } else {
            NOT_A_WORD_ID
        };
        child_dic_nodes.push_leaving_child(
            dic_node,
            node.children_pos(),
            word_id,
            merged_code_points,
        );
    }
    node.sibling_pos()
}
