//! `PatriciaTriePolicy::getCodePointsAndProbabilityAndReturnCodePointCount`: finds a word by its id,
//! and reads its code points and unigram probability back. Words are ordered in the dictionary in
//! strict breadth-first order, so the walk checks one node array at a time, and descends only the
//! node whose children hold the word's position. It cannot loop forever: each level adds a code
//! point, and the walk stops after `max_code_point_count` levels.

use super::bigram::skip_existing_bigrams;
use super::patricia::PatriciaTriePolicy;
use super::reading::{
    NodeFlags, code_point_advance, flags_advance, has_bigrams, has_children_in_flags,
    has_multiple_chars, has_shortcut_targets, is_terminal, pt_node_array_size,
    read_children_position, read_probability, skip_characters,
};
use crate::defines::{MAX_WORD_LENGTH, NOT_A_CODE_POINT, NOT_A_PROBABILITY};

/// Where a step of the walk leaves things.
enum Step {
    /// The word was found: its code point count, and its unigram probability.
    Found(usize, i32),
    /// The walk goes on with the next level, from the position `pos` is at.
    Next,
    /// The dictionary is broken, so the walk ends with nothing found.
    Failed,
}

/// The walk's state: the body position, the output, and what the C++ keeps in its locals.
struct Cursor<'o> {
    pos: i32,
    word_pos: usize,
    last_candidate: i32,
    max: usize,
    out: &'o mut [i32],
}

impl Cursor<'_> {
    /// `outCodePoints[index] = value`, where the buffer has room. The C++ writes past a buffer that is
    /// too small; here such a write is dropped.
    fn put(&mut self, index: usize, value: i32) {
        if let Some(slot) = self.out.get_mut(index) {
            *slot = value;
        }
    }
}

/// The walk over one word's path, from the root to its node.
#[derive(Clone, Copy, Debug)]
pub struct PatriciaWalk<'a> {
    policy: &'a PatriciaTriePolicy,
}

impl<'a> PatriciaWalk<'a> {
    /// The walk over `policy`.
    pub fn new(policy: &'a PatriciaTriePolicy) -> Self {
        Self { policy }
    }

    /// The word's code points and unigram probability, with the count (0 when not found).
    pub fn code_points_and_probability(
        &self,
        word_id: i32,
        max_code_point_count: usize,
        out: &mut [i32],
    ) -> (usize, i32) {
        let target = self.policy.terminal_pt_node_pos_from_word_id(word_id);
        let mut cursor = Cursor {
            pos: self.policy.root_position(),
            word_pos: 0,
            last_candidate: 0,
            max: max_code_point_count,
            out,
        };
        for _ in 0..max_code_point_count {
            match self.scan_array(target, &mut cursor) {
                Step::Found(count, probability) => return (count, probability),
                Step::Next => {}
                Step::Failed => return (0, NOT_A_PROBABILITY),
            }
        }
        (0, NOT_A_PROBABILITY)
    }

    /// One level of the walk: the node array at the cursor's position.
    fn scan_array(&self, target: i32, cursor: &mut Cursor<'_>) -> Step {
        if !self.policy.is_valid_pos(cursor.pos) {
            return Step::Failed;
        }
        let mut remaining = pt_node_array_size(self.policy.body(), &mut cursor.pos);
        cursor.last_candidate = 0;
        while remaining > 0 {
            if !self.policy.is_valid_pos(cursor.pos) {
                return Step::Failed;
            }
            if let Some(step) = self.scan_node(target, remaining, cursor) {
                return step;
            }
            remaining -= 1;
        }
        Step::Next
    }

    /// One node of the array, at the cursor's position: `None` to go on to its next sibling.
    /// `remaining` is how many nodes the array has left, this one included.
    fn scan_node(&self, target: i32, remaining: i32, cursor: &mut Cursor<'_>) -> Option<Step> {
        let body = self.policy.body();
        let table = self.policy.code_point_table();
        let start_pos = cursor.pos;
        let flags = flags_advance(body, &mut cursor.pos);
        let character = code_point_advance(body, table, &mut cursor.pos);
        if target == start_pos {
            cursor.put(cursor.word_pos, character);
            if has_multiple_chars(flags) {
                copy_rest(
                    body,
                    table,
                    cursor.max,
                    cursor.out,
                    &mut cursor.word_pos,
                    &mut cursor.pos,
                );
            }
            let probability = read_probability(body, &mut cursor.pos);
            return Some(Step::Found(cursor.word_pos + 1, probability));
        }
        if has_multiple_chars(flags) {
            skip_characters(body, flags, MAX_WORD_LENGTH, table, &mut cursor.pos);
        }
        if is_terminal(flags) {
            read_probability(body, &mut cursor.pos);
        }
        let found = if has_children_in_flags(flags) {
            let mut children = cursor.pos;
            let children_pos = read_children_position(body, flags, &mut children);
            if children_pos > target {
                true
            } else if remaining <= 1 {
                cursor.last_candidate = start_pos;
                true
            } else {
                false
            }
        } else {
            remaining <= 1
        };
        if found {
            if cursor.last_candidate != 0 {
                return Some(self.descend(cursor));
            }
        } else if has_children_in_flags(flags) {
            cursor.last_candidate = start_pos;
        }
        if self.skip_node_tail(flags, &mut cursor.pos) {
            None
        } else {
            Some(Step::Failed)
        }
    }

    /// The descent: re-reads the remembered node, writes its code points, and moves the cursor to
    /// its children, which the next level scans.
    fn descend(&self, cursor: &mut Cursor<'_>) -> Step {
        let body = self.policy.body();
        let table = self.policy.code_point_table();
        let mut at = cursor.last_candidate;
        let flags = flags_advance(body, &mut at);
        let character = code_point_advance(body, table, &mut at);
        cursor.put(cursor.word_pos, character);
        if has_multiple_chars(flags) {
            copy_rest(
                body,
                table,
                cursor.max,
                cursor.out,
                &mut cursor.word_pos,
                &mut at,
            );
        }
        cursor.word_pos += 1;
        if is_terminal(flags) {
            read_probability(body, &mut at);
        }
        cursor.pos = read_children_position(body, flags, &mut at);
        Step::Next
    }

    /// Passes a node's tail, its children offset, shortcuts and bigrams, when the walk doesn't
    /// descend it. Returns false when the bigrams are broken.
    fn skip_node_tail(&self, flags: NodeFlags, pos: &mut i32) -> bool {
        let body = self.policy.body();
        if has_children_in_flags(flags) {
            read_children_position(body, flags, pos);
        }
        if has_shortcut_targets(flags) {
            self.policy.shortcut_policy().skip_all(pos);
        }
        !has_bigrams(flags) || skip_existing_bigrams(body, pos)
    }
}

/// Writes the rest of a multi-character node's code points, after the first one already written,
/// at most `max` in all, and moves `pos` past their terminator.
fn copy_rest(
    body: &[u8],
    table: Option<&[i32]>,
    max: usize,
    out: &mut [i32],
    word_pos: &mut usize,
    pos: &mut i32,
) {
    let mut next = code_point_advance(body, table, pos);
    let mut char_count = max as isize;
    while next != NOT_A_CODE_POINT && {
        char_count -= 1;
        char_count > 0
    } {
        *word_pos += 1;
        if let Some(slot) = out.get_mut(*word_pos) {
            *slot = next;
        }
        next = code_point_advance(body, table, pos);
    }
}
