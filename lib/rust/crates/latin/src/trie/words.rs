//! `PatriciaTriePolicy::iterateWords` and `iterateWordsInPtNodeArray`: every word of the dictionary,
//! each to a listener. Walks the trie once, carrying each word's prefix down, so no word is looked
//! up again from the root. A node's children are visited before the node itself, and a broken
//! dictionary ends the walk early.

use super::interface::WordListener;
use super::patricia::PatriciaTriePolicy;
use crate::defines::{MAX_WORD_LENGTH, NOT_A_PROBABILITY};

impl PatriciaTriePolicy {
    /// `iterateWords`: every word with a probability, to `listener`. Words longer than
    /// `MAX_WORD_LENGTH` are skipped, with everything below them.
    pub fn iterate_words<L: WordListener>(&self, listener: &mut L) {
        let mut code_points = [0_i32; MAX_WORD_LENGTH];
        self.iterate_words_in_pt_node_array(self.root_position(), &mut code_points, 0, listener);
    }

    /// `iterateWordsInPtNodeArray`: the words under the array at `pt_node_array_pos`, whose prefix
    /// is the first `prefix_length` code points of `code_points`. Returns false when the dictionary
    /// turned out to be broken, which ends the walk.
    fn iterate_words_in_pt_node_array<L: WordListener>(
        &self,
        pt_node_array_pos: i32,
        code_points: &mut [i32; MAX_WORD_LENGTH],
        prefix_length: usize,
        listener: &mut L,
    ) -> bool {
        let Some((node_count, mut pt_node_pos)) = self.array_reader().read_info(pt_node_array_pos)
        else {
            return false;
        };
        let nodes = self.node_reader();
        for _ in 0..node_count {
            let node = nodes.fetch(pt_node_pos);
            if !node.is_valid() {
                return false;
            }
            let length = prefix_length + node.code_point_count();
            // A longer word couldn't be looked up, nor could anything below it.
            if length <= MAX_WORD_LENGTH {
                code_points[prefix_length..length].copy_from_slice(node.code_points());
                if node.has_children()
                    && !self.iterate_words_in_pt_node_array(
                        node.children_pos(),
                        code_points,
                        length,
                        listener,
                    )
                {
                    return false;
                }
                // As getProbabilityOfWord has it, with no previous words.
                if node.is_terminal()
                    && !node.is_not_a_word()
                    && self.get_probability(node.probability(), NOT_A_PROBABILITY)
                        != NOT_A_PROBABILITY
                {
                    listener.on_visit_word(&code_points[..length]);
                }
            }
            pt_node_pos = node.sibling_pos();
        }
        true
    }
}
