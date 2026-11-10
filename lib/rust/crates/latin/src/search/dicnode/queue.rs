//! `suggest/core/dicnode/dic_node_priority_queue`: the bounded queue of nodes, worst first.
//!
//! The C++ keeps pointers in a `std::priority_queue`. The order of two nodes is not a strict weak
//! order (scores are equal within an epsilon), so the heap's exact layout shows in which nodes
//! are dropped and in what order they pop; [`crate::search::heap`] replays libc++'s algorithms.

use super::node::DicNode;
use super::pool::DicNodePool;
use crate::search::heap;

/// `DicNodePriorityQueue`: holds at most `max_size` nodes; the top is the worst.
#[derive(Debug)]
pub struct DicNodePriorityQueue {
    /// `mMaxSize`.
    max_size: i32,
    /// `mDicNodesQueue`: the heap of pool slots, worst on top.
    heap: Vec<u32>,
    /// `mDicNodePool`.
    pool: DicNodePool,
}

/// `DicNodePriorityQueue::compareDicNode` over two slots of a pool, the slot order standing for
/// the address order.
fn less(nodes: &[DicNode], left: u32, right: u32) -> bool {
    nodes[left as usize].compare(&nodes[right as usize], left > right)
}

impl DicNodePriorityQueue {
    /// `DicNodePriorityQueue(capacity)`.
    pub fn new(capacity: i32) -> Self {
        let mut queue = Self {
            max_size: capacity,
            heap: Vec::new(),
            pool: DicNodePool::new(capacity),
        };
        queue.clear();
        queue
    }

    /// `getSize`.
    pub fn size(&self) -> i32 {
        self.heap.len() as i32
    }

    /// `getMaxSize`.
    pub fn max_size(&self) -> i32 {
        self.max_size
    }

    /// `setMaxSize`.
    pub fn set_max_size(&mut self, max_size: i32) {
        self.max_size = max_size;
    }

    /// `clear`.
    pub fn clear(&mut self) {
        self.clear_and_resize(self.max_size);
    }

    /// `clearAndResize`: empties the queue without returning its nodes one by one; the pool
    /// is refilled unless it was already full.
    pub fn clear_and_resize(&mut self, max_size: i32) {
        self.max_size = max_size;
        self.heap.clear();
        self.pool.reset(self.max_size.saturating_add(1));
    }

    /// `copyPush`: a copy of `dic_node` joins the queue; at the maximum size the worst node goes
    /// when `dic_node` is better, otherwise the copy is dropped. A pool with no free node drops
    /// it too.
    pub fn copy_push(&mut self, dic_node: &DicNode) {
        let Some(slot) = self.pool.get_instance() else {
            return;
        };
        self.pool.node_mut(slot).init_by_copy(dic_node);
        if self.size() < self.max_size {
            self.push(slot);
            return;
        }
        // The C++ reads the top of an empty queue here (a maximum size of 0 or less, which no
        // caller sets): the copy is dropped.
        let Some(&worst) = self.heap.first() else {
            self.pool.place_back_instance(slot);
            return;
        };
        if less(self.pool.nodes(), slot, worst) {
            self.pool.place_back_instance(worst);
            self.pop_top();
            self.push(slot);
            return;
        }
        self.pool.place_back_instance(slot);
    }

    /// `copyPop`: the top (worst) node into `dest`, and out of the queue.
    pub fn copy_pop(&mut self, dest: Option<&mut DicNode>) {
        let Some(&slot) = self.heap.first() else {
            return;
        };
        if let Some(dest) = dest {
            dest.init_by_copy(self.pool.node(slot));
        }
        self.pool.place_back_instance(slot);
        self.pop_top();
    }

    /// `std::priority_queue::push`.
    fn push(&mut self, slot: u32) {
        self.heap.push(slot);
        let nodes = self.pool.nodes();
        heap::push_heap(&mut self.heap, |&l, &r| less(nodes, l, r));
    }

    /// `std::priority_queue::pop`.
    fn pop_top(&mut self) {
        let nodes = self.pool.nodes();
        heap::pop_heap(&mut self.heap, |&l, &r| less(nodes, l, r));
        self.heap.pop();
    }
}
