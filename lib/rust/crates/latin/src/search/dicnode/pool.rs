//! `suggest/core/dicnode/dic_node_pool`: the storage of a queue's nodes, handed out and taken
//! back from the end of a free list.
//!
//! A node is a slot index here; the C++ uses its address, and the order of two addresses in the
//! one pool is the order of their slots.

use super::node::DicNode;

/// `DicNodePool`.
#[derive(Debug)]
pub struct DicNodePool {
    /// `mDicNodes`.
    dic_nodes: Vec<DicNode>,
    /// `mPooledDicNodes`: the free slots (a deque of pointers in the C++, used as a stack).
    pooled_dic_nodes: Vec<u32>,
}

impl DicNodePool {
    /// `DicNodePool(capacity)`.
    pub fn new(capacity: i32) -> Self {
        let mut pool = Self {
            dic_nodes: Vec::new(),
            pooled_dic_nodes: Vec::new(),
        };
        pool.reset(capacity);
        pool
    }

    /// `DicNodePool::reset`: nothing when the pool already has `capacity` nodes all free,
    /// otherwise `capacity` nodes, all free, in slot order. A negative capacity is 0 (the C++
    /// `vector::resize` would throw).
    pub fn reset(&mut self, capacity: i32) {
        let capacity = usize::try_from(capacity).unwrap_or(0);
        if capacity == self.dic_nodes.len() && capacity == self.pooled_dic_nodes.len() {
            return;
        }
        self.dic_nodes.resize(capacity, DicNode::default());
        self.pooled_dic_nodes.clear();
        self.pooled_dic_nodes.extend(0..capacity as u32);
    }

    /// `DicNodePool::getInstance`: the slot at the end of the free list, if any.
    pub fn get_instance(&mut self) -> Option<u32> {
        self.pooled_dic_nodes.pop()
    }

    /// `DicNodePool::placeBackInstance`.
    pub fn place_back_instance(&mut self, slot: u32) {
        self.pooled_dic_nodes.push(slot);
    }

    /// The node in `slot`.
    pub fn node(&self, slot: u32) -> &DicNode {
        &self.dic_nodes[slot as usize]
    }

    /// The node in `slot`, to write.
    pub fn node_mut(&mut self, slot: u32) -> &mut DicNode {
        &mut self.dic_nodes[slot as usize]
    }

    /// All the slots, to order two nodes of the pool.
    pub fn nodes(&self) -> &[DicNode] {
        &self.dic_nodes
    }
}
