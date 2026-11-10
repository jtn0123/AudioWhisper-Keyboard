//! `suggest/core/dicnode/dic_nodes_cache`: the queues that drive the search and keep its state
//! between keystrokes.
//!
//! The C++ has four queues and four pointers to them (active, next active, cached for continuous
//! suggestion, terminal) which it swaps. Here the queues are in an array and each role is an
//! index into it.

use super::node::DicNode;
use super::queue::DicNodePriorityQueue;
use crate::defines::MAX_RESULTS;

/// `DicNodesCache::LARGE_PRIORITY_QUEUE_CAPACITY`.
const LARGE_PRIORITY_QUEUE_CAPACITY: i32 = 310;
/// `DicNodesCache::SMALL_PRIORITY_QUEUE_CAPACITY`.
const SMALL_PRIORITY_QUEUE_CAPACITY: i32 = 100;
/// `CACHE_BACK_LENGTH` of `isCacheBorderForTyping`.
const CACHE_BACK_LENGTH: i32 = 3;

/// `DicNodesCache`.
#[derive(Debug)]
pub struct DicNodesCache {
    /// `mUsesLargeCapacityCache`.
    uses_large_capacity_cache: bool,
    /// `mDicNodePriorityQueue0`, `1`, `2` and `mDicNodePriorityQueueForTerminal`.
    queues: [DicNodePriorityQueue; 4],
    /// The queue `mActiveDicNodes` points to: the nodes being expanded.
    active: usize,
    /// The queue `mNextActiveDicNodes` points to: the nodes to expand next.
    next_active: usize,
    /// The queue `mCachedDicNodesForContinuousSuggestion` points to: the nodes kept for the next
    /// call.
    cached: usize,
    /// The queue `mTerminalDicNodes` points to: the best terminals so far.
    terminal: usize,
    /// `mInputIndex`.
    input_index: i32,
    /// `mLastCachedInputIndex`.
    last_cached_input_index: i32,
    /// How many times [`Self::continue_search`] ran, for the tests to check that typing letter by
    /// letter does continue from the cache.
    #[cfg(test)]
    pub continued_searches: usize,
}

impl DicNodesCache {
    /// `DicNodesCache(usesLargeCapacityCache)`.
    pub fn new(uses_large_capacity_cache: bool) -> Self {
        let capacity = if uses_large_capacity_cache {
            LARGE_PRIORITY_QUEUE_CAPACITY
        } else {
            SMALL_PRIORITY_QUEUE_CAPACITY
        };
        Self {
            uses_large_capacity_cache,
            queues: [
                DicNodePriorityQueue::new(capacity),
                DicNodePriorityQueue::new(capacity),
                DicNodePriorityQueue::new(capacity),
                DicNodePriorityQueue::new(MAX_RESULTS as i32),
            ],
            active: 0,
            next_active: 1,
            cached: 2,
            terminal: 3,
            input_index: 0,
            last_cached_input_index: 0,
            #[cfg(test)]
            continued_searches: 0,
        }
    }

    /// `getCacheCapacity`.
    fn cache_capacity(&self) -> i32 {
        if self.uses_large_capacity_cache {
            LARGE_PRIORITY_QUEUE_CAPACITY
        } else {
            SMALL_PRIORITY_QUEUE_CAPACITY
        }
    }

    /// `DicNodesCache::reset`.
    pub fn reset(&mut self, next_active_size: i32, terminal_size: i32) {
        self.input_index = 0;
        self.last_cached_input_index = 0;
        // The size of current active DicNode queue doesn't have to be changed.
        self.queues[self.active].clear();
        // nextActiveSize is used to limit the next iteration's active DicNode size.
        let next_active_size_fitting_to_the_capacity = next_active_size.min(self.cache_capacity());
        self.queues[self.next_active].clear_and_resize(next_active_size_fitting_to_the_capacity);
        self.queues[self.terminal].clear_and_resize(terminal_size);
        // The size of cached DicNode queue doesn't have to be changed.
        self.queues[self.cached].clear();
    }

    /// `continueSearch`.
    pub fn continue_search(&mut self) {
        #[cfg(test)]
        {
            self.continued_searches += 1;
        }
        self.reset_temporary_caches();
        self.restore_active_dic_nodes_from_cache();
    }

    /// `advanceActiveDicNodes`: the next active nodes become the active ones.
    pub fn advance_active_dic_nodes(&mut self) {
        let reusable =
            self.move_nodes_into_active_and_return_reusable_empty_queue(self.next_active);
        self.next_active = reusable;
    }

    /// `activeSize`.
    pub fn active_size(&self) -> i32 {
        self.queues[self.active].size()
    }

    /// `terminalSize`.
    pub fn terminal_size(&self) -> i32 {
        self.queues[self.terminal].size()
    }

    /// `isLookAheadCorrectionInputIndex`.
    pub fn is_look_ahead_correction_input_index(&self, input_index: i32) -> bool {
        input_index == self.input_index - 1
    }

    /// `advanceInputIndex`.
    pub fn advance_input_index(&mut self, input_size: i32) {
        if self.input_index < input_size {
            self.input_index += 1;
        }
    }

    /// `copyPushTerminal`.
    pub fn copy_push_terminal(&mut self, dic_node: &DicNode) {
        self.queues[self.terminal].copy_push(dic_node);
    }

    /// `copyPushActive`.
    pub fn copy_push_active(&mut self, dic_node: &DicNode) {
        self.queues[self.active].copy_push(dic_node);
    }

    /// `copyPushContinue`.
    pub fn copy_push_continue(&mut self, dic_node: &DicNode) {
        self.queues[self.cached].copy_push(dic_node);
    }

    /// `copyPushNextActive`.
    pub fn copy_push_next_active(&mut self, dic_node: &DicNode) {
        self.queues[self.next_active].copy_push(dic_node);
    }

    /// `popTerminal`.
    pub fn pop_terminal(&mut self, dest: &mut DicNode) {
        self.queues[self.terminal].copy_pop(Some(dest));
    }

    /// `popActive`.
    pub fn pop_active(&mut self, dest: &mut DicNode) {
        self.queues[self.active].copy_pop(Some(dest));
    }

    /// `hasCachedDicNodesForContinuousSuggestion`.
    pub fn has_cached_dic_nodes_for_continuous_suggestion(&self) -> bool {
        self.queues[self.cached].size() > 0
    }

    /// `isCacheBorderForTyping`.
    pub fn is_cache_border_for_typing(&self, input_size: i32) -> bool {
        let cache_input_index = input_size - CACHE_BACK_LENGTH;
        cache_input_index == self.input_index && cache_input_index != self.last_cached_input_index
    }

    /// `updateLastCachedInputIndex`.
    pub fn update_last_cached_input_index(&mut self) {
        self.last_cached_input_index = self.input_index;
    }

    /// `restoreActiveDicNodesFromCache`.
    fn restore_active_dic_nodes_from_cache(&mut self) {
        self.input_index = self.last_cached_input_index;
        let reusable = self.move_nodes_into_active_and_return_reusable_empty_queue(self.cached);
        self.cached = reusable;
    }

    /// `moveNodesAndReturnReusableEmptyQueue(src, &mActiveDicNodes)`: the active role goes to the
    /// queue `src` (with the old active queue's maximum size); the old active queue is emptied,
    /// given `src`'s old maximum size, and is returned to take `src`'s role.
    fn move_nodes_into_active_and_return_reusable_empty_queue(&mut self, src: usize) -> usize {
        let src_max_size = self.queues[src].max_size();
        let tmp = self.active;
        let dest_max_size = self.queues[tmp].max_size();
        self.active = src;
        self.queues[src].set_max_size(dest_max_size);
        self.queues[tmp].clear_and_resize(src_max_size);
        tmp
    }

    /// `resetTemporaryCaches`.
    fn reset_temporary_caches(&mut self) {
        self.queues[self.active].clear();
        self.queues[self.next_active].clear();
        self.queues[self.terminal].clear();
    }
}
