//! Differential tests of the node-level reading: every node of the bundled trie read through the
//! C++'s own reader (the probe), node arrays and their forward links, and the bigram and shortcut lists
//! each node points at. Random bodies go through the same readers, so a flag the dictionary never sets
//! is still compared.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use latin_oracle::hooks::trie as cpp;

use super::{Pair, Random, pairs};
use crate::trie::bigram::BigramIterator;
use crate::trie::node::PtNodeParams;
use crate::trie::patricia::PatriciaTriePolicy;
use crate::trie::shortcut::ShortcutIterator;

/// The positions of every node of the trie: the root array's nodes, and those of each array reached
/// through a node's children, in breadth-first order.
fn all_node_positions(pair: &Pair) -> Vec<i32> {
    let policy = pair.ours.policy();
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

/// Asserts that one node read by the Rust agrees with the C++ probe's. The word is not printed.
fn same_node(ours: &PtNodeParams, theirs: &cpp::ProbedNode, label: &str) {
    assert_eq!(ours.is_valid(), theirs.valid, "{label}: valid");
    if !theirs.valid {
        return;
    }
    assert_eq!(ours.head_pos(), theirs.head_pos, "{label}: head position");
    assert!(
        ours.code_points() == theirs.code_points.as_slice(),
        "{label}: code points"
    );
    assert_eq!(
        ours.probability(),
        theirs.probability,
        "{label}: probability"
    );
    assert_eq!(
        ours.children_pos(),
        theirs.children_pos,
        "{label}: children position"
    );
    assert_eq!(
        ours.shortcut_pos(),
        theirs.shortcut_pos,
        "{label}: shortcut position"
    );
    assert_eq!(
        ours.bigram_pos(),
        theirs.bigram_pos,
        "{label}: bigram position"
    );
    assert_eq!(
        ours.sibling_pos(),
        theirs.sibling_pos,
        "{label}: sibling position"
    );
    assert_eq!(ours.is_terminal(), theirs.is_terminal, "{label}: terminal");
    assert_eq!(
        ours.is_not_a_word(),
        theirs.is_not_a_word,
        "{label}: not a word"
    );
    assert_eq!(
        ours.is_possibly_offensive(),
        theirs.is_possibly_offensive,
        "{label}: possibly offensive"
    );
    assert_eq!(
        ours.has_children(),
        theirs.has_children,
        "{label}: has children"
    );
    assert_eq!(
        ours.has_bigrams(),
        theirs.has_bigrams,
        "{label}: has bigrams"
    );
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

/// The probe's shortcut entries at `list`, in the same shape as [`rust_shortcuts`].
fn probe_shortcuts(probe: &cpp::Probe, list: i32, max_depth: usize) -> Vec<Shortcut> {
    probe
        .shortcut_entries(list, max_depth)
        .into_iter()
        .map(|entry| {
            (
                entry.probability,
                entry.is_whitelist,
                entry.has_next,
                entry.code_points,
            )
        })
        .collect()
}

#[test]
fn every_node_reads_the_same_as_the_cpp_reader() {
    for pair in pairs() {
        let probe = cpp::Probe::open(pair.ours.policy().body(), None);
        let positions = all_node_positions(pair);
        assert!(
            positions.len() > 10_000,
            "the trie has nodes to compare ({})",
            positions.len()
        );
        let ours = pair.ours.policy().node_reader();
        for pos in positions {
            let label = format!("node {pos}");
            let read = ours.read_info(pos);
            same_node(&read, &probe.read_info(pos), &format!("{label} read"));
            assert_eq!(
                i32::from(read.flags()),
                probe.read_info(pos).flags,
                "{label}: flags"
            );
            same_node(
                &ours.fetch(pos),
                &probe.fetch(pos),
                &format!("{label} fetch"),
            );
        }
    }
}

#[test]
fn arrays_and_forward_links_match_the_cpp_at_random_positions() {
    for pair in pairs() {
        let probe = cpp::Probe::open(pair.ours.policy().body(), None);
        let body_len = pair.ours.policy().body().len();
        let mut random = Random(0x00A2_2A77);
        let mut positions = all_node_positions(pair);
        positions.extend((0..20_000).map(|_| random.below(body_len) as i32));
        positions.push(body_len as i32);
        for pos in positions {
            let ours_array = pair.ours.policy().array_reader().read_info(pos);
            assert_eq!(probe.array(pos), ours_array, "array at {pos}");
            let ours_link = pair.ours.policy().array_reader().read_forward_link(pos);
            assert_eq!(probe.forward_link(pos), ours_link, "forward link at {pos}");
        }
    }
}

#[test]
fn bigram_lists_of_every_node_match_the_cpp_visits() {
    for pair in pairs() {
        let probe = cpp::Probe::open(pair.ours.policy().body(), None);
        let mut checked = 0_usize;
        for pos in all_node_positions(pair) {
            let node = pair.ours.policy().node_reader().fetch(pos);
            if !node.has_bigrams() {
                continue;
            }
            checked += 1;
            let list = node.bigram_pos();
            assert!(
                rust_bigrams(pair.ours.policy(), list) == probe.bigram_visits(list),
                "bigram list at {list} differs"
            );
        }
        assert!(
            checked > 1_000,
            "the trie has bigram lists to compare ({checked})"
        );
    }
}

#[test]
fn shortcut_lists_of_every_node_match_the_cpp_entries() {
    for pair in pairs() {
        let probe = cpp::Probe::open(pair.ours.policy().body(), None);
        let mut checked = 0_usize;
        for pos in all_node_positions(pair) {
            let node = pair.ours.policy().node_reader().fetch(pos);
            if !node.has_shortcut_targets() {
                continue;
            }
            checked += 1;
            let list = node.shortcut_pos();
            assert!(
                rust_shortcuts(pair.ours.policy(), list, 48) == probe_shortcuts(&probe, list, 48),
                "shortcut list at {list} differs"
            );
        }
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
fn random_bodies_read_their_nodes_arrays_and_lists_as_the_cpp_does() {
    let mut random = Random(0x00B0_D1E5);
    for round in 0..2_000 {
        let length = 8 + random.below(200);
        let body: Vec<u8> = (0..length).map(|_| random_byte(&mut random)).collect();
        // A header of size 0, so the whole buffer is the body.
        let policy = PatriciaTriePolicy::new(Arc::from(body.as_slice()), 0, None);
        let probe = cpp::Probe::open(&body, None);
        for pos in 0..length as i32 {
            let label = format!("round {round} position {pos}");
            let read = policy.node_reader().read_info(pos);
            same_node(&read, &probe.read_info(pos), &format!("{label} read"));
            assert_eq!(
                i32::from(read.flags()),
                probe.read_info(pos).flags,
                "{label}: flags"
            );
            same_node(
                &policy.node_reader().fetch(pos),
                &probe.fetch(pos),
                &format!("{label} fetch"),
            );
            assert_eq!(
                policy.array_reader().read_info(pos),
                probe.array(pos),
                "{label}: array"
            );
            assert_eq!(
                policy.array_reader().read_forward_link(pos),
                probe.forward_link(pos),
                "{label}: forward link"
            );
            assert!(
                rust_bigrams(&policy, pos) == probe.bigram_visits(pos),
                "{label}: bigram list"
            );
            assert!(
                rust_shortcuts(&policy, pos, 48) == probe_shortcuts(&probe, pos, 48),
                "{label}: shortcut list"
            );
        }
    }
}
