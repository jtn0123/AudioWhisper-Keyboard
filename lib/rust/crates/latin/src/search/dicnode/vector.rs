//! `suggest/core/dicnode/dic_node_vector`: the children of one node, gathered before they are
//! scored. The vector is reused: [`DicNodeVector::clear`] keeps its storage.

use super::node::DicNode;

/// `DicNodeVector`. The C++'s lock flag only guards against pushing after the size was read, in
/// debug builds, and is not ported.
#[derive(Debug, Default)]
pub struct DicNodeVector {
    /// `mDicNodes`.
    dic_nodes: Vec<DicNode>,
}

impl DicNodeVector {
    /// `DicNodeVector::DEFAULT_NODES_SIZE_FOR_OPTIMIZATION`.
    const DEFAULT_NODES_SIZE_FOR_OPTIMIZATION: usize = 60;

    /// `DicNodeVector(size)`, with the default capacity.
    pub fn new() -> Self {
        Self {
            dic_nodes: Vec::with_capacity(Self::DEFAULT_NODES_SIZE_FOR_OPTIMIZATION),
        }
    }

    /// The storage held, for tests that check a warm search does not grow it.
    #[cfg(all(test, not(target_os = "android")))]
    pub fn capacity(&self) -> usize {
        self.dic_nodes.capacity()
    }

    /// `clear`.
    pub fn clear(&mut self) {
        self.dic_nodes.clear();
    }

    /// `getSizeAndLock`: the number of nodes.
    pub fn len(&self) -> usize {
        self.dic_nodes.len()
    }

    /// `pushPassingChild`.
    pub fn push_passing_child(&mut self, dic_node: &DicNode) {
        let mut child = DicNode::default();
        child.init_as_passing_child(dic_node);
        self.dic_nodes.push(child);
    }

    /// `pushLeavingChild`.
    pub fn push_leaving_child(
        &mut self,
        dic_node: &DicNode,
        children_pt_node_array_pos: i32,
        word_id: i32,
        merged_code_points: &[i32],
    ) {
        let mut child = DicNode::default();
        child.init_as_child(
            dic_node,
            children_pt_node_array_pos,
            word_id,
            merged_code_points,
        );
        self.dic_nodes.push(child);
    }

    /// `operator[]`.
    pub fn get_mut(&mut self, id: usize) -> &mut DicNode {
        &mut self.dic_nodes[id]
    }

    /// The nodes in order.
    pub fn nodes(&self) -> &[DicNode] {
        &self.dic_nodes
    }
}
