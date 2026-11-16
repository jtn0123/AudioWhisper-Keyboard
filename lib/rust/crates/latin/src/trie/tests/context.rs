//! Differential tests of the answers that depend on previous words: the probability of a word after
//! up to three words (with the JNI's rules for null, empty and over-long slots), the bigram entries of
//! each word, and the multi-bigram cache across long sequences.

use std::collections::HashSet;

use latin_oracle::hooks::trie as cpp;

use super::{Pair, Random, check_attributes, pairs, random_word};
use crate::defines::{MAX_WORD_LENGTH, NOT_A_WORD_ID};
use crate::trie::interface::NgramListener;
use crate::trie::multi_bigram::MultiBigramMap;
use crate::trie::ngram::NgramContext;

/// One previous word as the JNI sees it: `None` for a null Java array.
type Slot = Option<Vec<i32>>;

/// Collects the bigram entries a walk visits.
#[derive(Default)]
struct Entries(Vec<(i32, i32)>);

impl NgramListener for Entries {
    fn on_visit_entry(&mut self, ngram_probability: i32, target_word_id: i32) {
        self.0.push((ngram_probability, target_word_id));
    }
}

/// A previous-word slot: null, empty, a dictionary word, or a word of any other kind.
fn random_slot(random: &mut Random, real: &[Vec<i32>]) -> Slot {
    match random.below(12) {
        0 => None,
        1 => Some(Vec::new()),
        2 => Some(random_word(random, real)),
        _ => Some(real[random.below(real.len())].clone()),
    }
}

/// The target of a real bigram of `word`, when it has one; otherwise `None`.
fn bigram_target(pair: &Pair, word: &[i32], random: &mut Random) -> Option<Vec<i32>> {
    let id = cpp::word_id(&pair.theirs, word, false);
    if id == NOT_A_WORD_ID {
        return None;
    }
    let entries = cpp::ngram_entries(&pair.theirs, &[id]);
    if entries.is_empty() {
        return None;
    }
    let (_, target) = entries[random.below(entries.len())];
    if target == NOT_A_WORD_ID {
        return None;
    }
    Some(cpp::code_points(&pair.theirs, target, MAX_WORD_LENGTH))
}

/// The next word: half the time a real bigram follower of the last previous word, otherwise any word.
fn next_word(random: &mut Random, pair: &Pair, last: Option<&Slot>) -> Vec<i32> {
    let follower = match last {
        Some(Some(word)) if !word.is_empty() && random.chance(2) => {
            bigram_target(pair, word, random)
        }
        _ => None,
    };
    follower.unwrap_or_else(|| random_word(random, &pair.words))
}

/// A real bigram follower of the first previous word, when it has one.
fn follower_id(pair: &Pair, prev: &[i32], random: &mut Random) -> Option<i32> {
    let first = *prev.first()?;
    if first == NOT_A_WORD_ID {
        return None;
    }
    let entries = cpp::ngram_entries(&pair.theirs, &[first]);
    if entries.is_empty() {
        return None;
    }
    let (_, target) = entries[random.below(entries.len())];
    (target != NOT_A_WORD_ID).then_some(target)
}

fn check_ngram(pair: &Pair, slots: &[Slot], beginning: &[bool], next: &[i32], label: &str) {
    let refs: Vec<Option<&[i32]>> = slots.iter().map(|slot| slot.as_deref()).collect();
    let context = NgramContext::new(&refs, beginning);
    let ours = pair.ours.ngram_probability(Some(&context), next);
    let theirs = cpp::ngram_probability_jni(&pair.theirs, slots, beginning, next);
    assert_eq!(
        ours,
        theirs,
        "{label}: ngram probability ({} previous)",
        slots.len()
    );
}

#[test]
fn probability_after_previous_words_matches_the_cpp() {
    for pair in pairs() {
        let mut random = Random(0xC0FF_EE01);
        for round in 0..40_000 {
            let count = random.below(4);
            let slots: Vec<Slot> = (0..count)
                .map(|_| random_slot(&mut random, &pair.words))
                .collect();
            let beginning: Vec<bool> = (0..count).map(|_| random.chance(3)).collect();
            let next = next_word(&mut random, pair, slots.last());
            check_ngram(pair, &slots, &beginning, &next, &format!("round {round}"));
        }
    }
}

#[test]
fn real_bigram_pairs_match_the_cpp() {
    for pair in pairs() {
        let mut random = Random(0x00B1_6AAA);
        let mut checked = 0_usize;
        for round in 0..20_000 {
            let first = pair.words[random.below(pair.words.len())].clone();
            let Some(second) = bigram_target(pair, &first, &mut random) else {
                continue;
            };
            checked += 1;
            let beginning = [random.chance(4)];
            let slots = [Some(first)];
            check_ngram(pair, &slots, &beginning, &second, &format!("pair {round}"));
        }
        assert!(
            checked > 1_000,
            "the dictionary has bigrams to check ({checked})"
        );
    }
}

#[test]
fn bigram_entries_of_every_word_match_the_cpp() {
    for pair in pairs() {
        for (index, word) in pair.words.iter().enumerate() {
            let id = pair.ours.policy().get_word_id(word, false);
            let mut ours = Entries::default();
            pair.ours.policy().iterate_ngram_entries(&[id], &mut ours);
            let theirs = cpp::ngram_entries(&pair.theirs, &[id]);
            assert!(ours.0 == theirs, "word {index}: bigram entries differ");
        }
    }
}

#[test]
fn word_attributes_in_context_match_the_cpp() {
    for pair in pairs() {
        let mut random = Random(0x00A7_7B0B);
        for round in 0..15_000 {
            let prev = pair.words[random.below(pair.words.len())].clone();
            let prev_id = pair.ours.policy().get_word_id(&prev, false);
            let prev_ids: Vec<i32> = if random.chance(3) {
                Vec::new()
            } else {
                vec![prev_id]
            };
            let follower = if random.chance(2) {
                follower_id(pair, &prev_ids, &mut random)
            } else {
                None
            };
            let id = follower.unwrap_or_else(|| {
                let word = &pair.words[random.below(pair.words.len())];
                pair.ours.policy().get_word_id(word, false)
            });
            check_attributes(pair, id, &prev_ids, &format!("round {round}"));
        }
    }
}

/// How many previous words the C++ `MultiBigramMap` caches: the first 25 distinct ones seen.
const CACHED_PREV_WORDS: usize = 25;

#[test]
fn multi_bigram_cache_answers_as_the_cpp_across_long_sequences() {
    for pair in pairs() {
        let policy = pair.ours.policy();
        let mut random = Random(0x003A_B16A);
        for sequence in 0..60 {
            let mut ours = MultiBigramMap::new();
            let mut theirs = cpp::MultiBigramMap::new();
            let mut first_words = HashSet::new();
            for step in 0..60 {
                let label = format!("sequence {sequence} step {step}");
                let prev = random_prev_ids(pair, &mut random);
                let follower = if random.chance(2) {
                    follower_id(pair, &prev, &mut random)
                } else {
                    None
                };
                let next = follower.unwrap_or_else(|| {
                    policy.get_word_id(&pair.words[random.below(pair.words.len())], false)
                });
                let unigram = random.below(257) as i32 - 1;
                let ours_probability = ours.get_bigram_probability(policy, &prev, next, unigram);
                let theirs_probability = theirs.probability(&pair.theirs, &prev, next, unigram);
                assert_eq!(ours_probability, theirs_probability, "{label}: probability");
                let ours_attributes =
                    policy.get_word_attributes_in_context(&prev, next, Some(&mut ours));
                let theirs_attributes =
                    cpp::word_attributes(&pair.theirs, Some(&theirs), &prev, next);
                assert_eq!(
                    ours_attributes.probability(),
                    theirs_attributes.probability,
                    "{label}: attributes"
                );
                if let Some(&first) = prev.first().filter(|&&it| it != NOT_A_WORD_ID) {
                    first_words.insert(first);
                }
                assert_eq!(
                    ours.cached_prev_word_count(),
                    first_words.len().min(CACHED_PREV_WORDS),
                    "{label}: cached previous words"
                );
            }
        }
    }
}

/// Previous word ids for the cache test: none, one, or two, with some of them not words.
fn random_prev_ids(pair: &Pair, random: &mut Random) -> Vec<i32> {
    let count = random.below(3);
    (0..count)
        .map(|_| {
            if random.chance(8) {
                NOT_A_WORD_ID
            } else {
                let word = &pair.words[random.below(pair.words.len())];
                pair.ours.policy().get_word_id(word, false)
            }
        })
        .collect()
}
