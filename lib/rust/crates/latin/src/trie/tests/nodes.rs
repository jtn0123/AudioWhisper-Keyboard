//! The node-level reading against LatinIME's C++ answers (golden files): every node of both
//! dictionaries as the C++'s own reader read it, node arrays and their forward links, and the
//! bigram and shortcut lists each node points at. Random bodies go through the same readers, so a
//! flag the dictionary never sets is still checked.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use super::{Fixture, Random, fixtures};
use crate::golden::{Tape, record};
use crate::trie::bigram::BigramIterator;
use crate::trie::node::PtNodeParams;
use crate::trie::patricia::PatriciaTriePolicy;
use crate::trie::shortcut::ShortcutIterator;

/// The positions of every node of the trie: the root array's nodes, and those of each array reached
/// through a node's children, in breadth-first order.
fn all_node_positions(fixture: &Fixture) -> Vec<i32> {
    let policy = fixture.dictionary.policy();
    let body_len = policy.body().len();
    let mut arrays = VecDeque::from([policy.root_position()]);
    let mut seen_arrays = HashSet::new();
    let mut nodes = Vec::new();
    while let Some(array_pos) = arrays.pop_front() {
        if !seen_arrays.insert(array_pos) {
            continue;
        }
        let Some((count, first)) = policy.array_reader().read_info(array_pos) else {
            continue;
        };
        let mut pos = first;
        for _ in 0..count {
            let node = policy.node_reader().read_info(pos);
            if !node.is_valid() {
                break;
            }
            nodes.push(pos);
            assert!(
                nodes.len() <= body_len,
                "the node walk visits more nodes than the body holds"
            );
            if node.has_children() {
                arrays.push_back(node.children_pos());
            }
            pos = node.sibling_pos();
        }
    }
    nodes
}

/// What a read node holds: `None` when it is not valid, otherwise its head position, code points,
/// probability, children, shortcut, bigram and sibling positions, and its flags as booleans.
type Node = Option<(
    i32,
    Vec<i32>,
    i32,
    (i32, i32, i32, i32),
    (bool, bool, bool, bool, bool),
)>;

fn node(ours: &PtNodeParams) -> Node {
    ours.is_valid().then(|| {
        (
            ours.head_pos(),
            ours.code_points().to_vec(),
            ours.probability(),
            (
                ours.children_pos(),
                ours.shortcut_pos(),
                ours.bigram_pos(),
                ours.sibling_pos(),
            ),
            (
                ours.is_terminal(),
                ours.is_not_a_word(),
                ours.is_possibly_offensive(),
                ours.has_children(),
                ours.has_bigrams(),
            ),
        )
    })
}

/// The node at `pos` read (with its flags) and fetched.
fn check_node(tape: &mut Tape, policy: &PatriciaTriePolicy, pos: i32) {
    let read = policy.node_reader().read_info(pos);
    record!(tape, (node(&read), i32::from(read.flags())));
    record!(tape, node(&policy.node_reader().fetch(pos)));
}

/// Every bigram visit of the list at `list`, as (probability, bigram position).
fn rust_bigrams(policy: &PatriciaTriePolicy, list: i32) -> Vec<(i32, i32)> {
    let mut iterator = BigramIterator::new(policy.bigram_policy(), list);
    let mut visits = Vec::new();
    while iterator.has_next() {
        iterator.next_entry();
        visits.push((iterator.probability(), iterator.bigram_pos()));
    }
    visits
}

/// Every entry of the shortcut list at `list`, as (probability, whitelist, has next, code points).
fn rust_shortcuts(policy: &PatriciaTriePolicy, list: i32, max_depth: usize) -> Vec<Shortcut> {
    let mut iterator = ShortcutIterator::new(policy.shortcut_policy(), list);
    let mut out = vec![0_i32; max_depth];
    let mut entries = Vec::new();
    while iterator.has_next_shortcut_target() {
        let entry = iterator.next_shortcut_target(max_depth, &mut out);
        let count = entry.code_point_count.min(max_depth);
        entries.push((
            entry.probability,
            entry.is_whitelist,
            entry.has_next,
            out[..count].to_vec(),
        ));
    }
    entries
}

/// A shortcut entry: probability, whitelist, has next, and its target's code points.
type Shortcut = (i32, bool, bool, Vec<i32>);

#[test]
fn every_node_reads_the_same_as_the_goldens() {
    for fixture in fixtures() {
        let policy = fixture.dictionary.policy();
        let positions = all_node_positions(fixture);
        assert!(
            positions.len() > 10_000,
            "the trie has nodes to compare ({})",
            positions.len()
        );
        let mut tape = fixture.tape("every_node");
        for pos in positions {
            check_node(&mut tape, policy, pos);
        }
        tape.finish();
    }
}

/// The array and the forward link at `pos`, checked against the probe's.
fn check_array(tape: &mut Tape, policy: &PatriciaTriePolicy, pos: i32) {
    record!(
        tape,
        (
            policy.array_reader().read_info(pos),
            policy.array_reader().read_forward_link(pos)
        )
    );
}

#[test]
fn arrays_and_forward_links_match_the_goldens_at_random_positions() {
    for fixture in fixtures() {
        let policy = fixture.dictionary.policy();
        let body_len = policy.body().len();
        let mut random = Random(0x00A2_2A77);
        let mut positions = all_node_positions(fixture);
        positions.extend((0..20_000).map(|_| random.below(body_len) as i32));
        positions.push(body_len as i32);
        let mut tape = fixture.tape("arrays_and_forward_links");
        for pos in positions {
            check_array(&mut tape, policy, pos);
        }
        tape.finish();
    }
}

/// The bigram visits and the shortcut entries of the lists at `pos`, checked against the probe's.
fn check_lists(tape: &mut Tape, policy: &PatriciaTriePolicy, pos: i32) {
    record!(tape, rust_bigrams(policy, pos));
    record!(tape, rust_shortcuts(policy, pos, 48));
}

#[test]
fn bigram_lists_of_every_node_match_the_goldens() {
    for fixture in fixtures() {
        let policy = fixture.dictionary.policy();
        let mut tape = fixture.tape("bigram_lists_of_every_node");
        let mut checked = 0_usize;
        for pos in all_node_positions(fixture) {
            let node = policy.node_reader().fetch(pos);
            if !node.has_bigrams() {
                continue;
            }
            checked += 1;
            let list = node.bigram_pos();
            record!(tape, rust_bigrams(policy, list));
        }
        tape.finish();
        assert!(
            checked > 1_000,
            "the trie has bigram lists to compare ({checked})"
        );
    }
}

#[test]
fn shortcut_lists_of_every_node_match_the_goldens() {
    for fixture in fixtures() {
        let policy = fixture.dictionary.policy();
        let mut tape = fixture.tape("shortcut_lists_of_every_node");
        let mut checked = 0_usize;
        for pos in all_node_positions(fixture) {
            let node = policy.node_reader().fetch(pos);
            if !node.has_shortcut_targets() {
                continue;
            }
            checked += 1;
            let list = node.shortcut_pos();
            record!(tape, rust_shortcuts(policy, list, 48));
        }
        tape.finish();
        assert!(checked > 0, "the trie has shortcut lists to compare");
    }
}

/// A byte of a random body: the terminator and small values, which the readings branch on, and
/// otherwise any byte.
fn random_byte(random: &mut Random) -> u8 {
    match random.below(8) {
        0 => 0x1F,
        1 => random.below(0x40) as u8,
        _ => random.draw() as u8,
    }
}

#[test]
fn random_bodies_read_their_nodes_arrays_and_lists_as_the_goldens_say() {
    let mut tape = crate::golden::tape("trie.random_bodies");
    let mut random = Random(0x00B0_D1E5);
    for _ in 0..2_000 {
        let length = 8 + random.below(200);
        let body: Vec<u8> = (0..length).map(|_| random_byte(&mut random)).collect();
        // A header of size 0, so the whole buffer is the body.
        let policy = PatriciaTriePolicy::new(Arc::from(body.as_slice()), 0, None);
        for pos in 0..length as i32 {
            check_node(&mut tape, &policy, pos);
            check_array(&mut tape, &policy, pos);
            check_lists(&mut tape, &policy, pos);
        }
    }
    tape.finish();
}
