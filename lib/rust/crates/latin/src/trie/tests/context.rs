//! Differential tests of the answers that depend on previous words: the probability of a word after
//! up to three words (with the JNI's rules for null, empty and over-long slots), the bigram entries of
//! each word, and the multi-bigram cache across long sequences.

use std::collections::HashSet;

use super::{Fixture, Random, check_attributes, code_points, fixtures, ngram_entries, random_word};
use crate::defines::NOT_A_WORD_ID;
use crate::golden::{Tape, record};
use crate::trie::multi_bigram::MultiBigramMap;
use crate::trie::ngram::NgramContext;

/// One previous word as the JNI sees it: `None` for a null Java array.
type Slot = Option<Vec<i32>>;

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
fn bigram_target(fixture: &Fixture, word: &[i32], random: &mut Random) -> Option<Vec<i32>> {
    let id = fixture.dictionary.policy().get_word_id(word, false);
    if id == NOT_A_WORD_ID {
        return None;
    }
    let entries = ngram_entries(fixture, &[id]);
    if entries.is_empty() {
        return None;
    }
    let (_, target) = entries[random.below(entries.len())];
    if target == NOT_A_WORD_ID {
        return None;
    }
    Some(code_points(fixture, target))
}

/// The next word: half the time a real bigram follower of the last previous word, otherwise any word.
fn next_word(random: &mut Random, fixture: &Fixture, last: Option<&Slot>) -> Vec<i32> {
    let follower = match last {
        Some(Some(word)) if !word.is_empty() && random.chance(2) => {
            bigram_target(fixture, word, random)
        }
        _ => None,
    };
    follower.unwrap_or_else(|| random_word(random, &fixture.words))
}

/// A real bigram follower of the first previous word, when it has one.
fn follower_id(fixture: &Fixture, prev: &[i32], random: &mut Random) -> Option<i32> {
    let first = *prev.first()?;
    if first == NOT_A_WORD_ID {
        return None;
    }
    let entries = ngram_entries(fixture, &[first]);
    if entries.is_empty() {
        return None;
    }
    let (_, target) = entries[random.below(entries.len())];
    (target != NOT_A_WORD_ID).then_some(target)
}

fn check_ngram(
    tape: &mut Tape,
    fixture: &Fixture,
    (slots, beginning): (&[Slot], &[bool]),
    next: &[i32],
) {
    let refs: Vec<Option<&[i32]>> = slots.iter().map(|slot| slot.as_deref()).collect();
    let context = NgramContext::new(&refs, beginning);
    record!(
        tape,
        fixture.dictionary.ngram_probability(Some(&context), next)
    );
}

#[test]
fn probability_after_previous_words_matches_the_goldens() {
    for fixture in fixtures() {
        let mut tape = fixture.tape("probability_after_previous_words");
        let mut random = Random(0xC0FF_EE01);
        for _ in 0..40_000 {
            let count = random.below(4);
            let slots: Vec<Slot> = (0..count)
                .map(|_| random_slot(&mut random, &fixture.words))
                .collect();
            let beginning: Vec<bool> = (0..count).map(|_| random.chance(3)).collect();
            let next = next_word(&mut random, fixture, slots.last());
            check_ngram(&mut tape, fixture, (&slots, &beginning), &next);
        }
        tape.finish();
    }
}

#[test]
fn real_bigram_pairs_match_the_goldens() {
    for fixture in fixtures() {
        let mut tape = fixture.tape("real_bigram_pairs");
        let mut random = Random(0x00B1_6AAA);
        let mut checked = 0_usize;
        for _ in 0..20_000 {
            let first = fixture.words[random.below(fixture.words.len())].clone();
            let Some(second) = bigram_target(fixture, &first, &mut random) else {
                continue;
            };
            checked += 1;
            let beginning = [random.chance(4)];
            let slots = [Some(first)];
            check_ngram(&mut tape, fixture, (&slots, &beginning), &second);
        }
        tape.finish();
        assert!(
            checked > 1_000,
            "the dictionary has bigrams to check ({checked})"
        );
    }
}

#[test]
fn bigram_entries_of_every_word_match_the_goldens() {
    for fixture in fixtures() {
        let mut tape = fixture.tape("bigram_entries_of_every_word");
        for word in &fixture.words {
            let id = fixture.dictionary.policy().get_word_id(word, false);
            record!(tape, ngram_entries(fixture, &[id]));
        }
        tape.finish();
    }
}

#[test]
fn word_attributes_in_context_match_the_goldens() {
    for fixture in fixtures() {
        let mut tape = fixture.tape("word_attributes_in_context");
        let mut random = Random(0x00A7_7B0B);
        for _ in 0..15_000 {
            let prev = fixture.words[random.below(fixture.words.len())].clone();
            let prev_id = fixture.dictionary.policy().get_word_id(&prev, false);
            let prev_ids: Vec<i32> = if random.chance(3) {
                Vec::new()
            } else {
                vec![prev_id]
            };
            let follower = if random.chance(2) {
                follower_id(fixture, &prev_ids, &mut random)
            } else {
                None
            };
            let id = follower.unwrap_or_else(|| {
                let word = &fixture.words[random.below(fixture.words.len())];
                fixture.dictionary.policy().get_word_id(word, false)
            });
            check_attributes(&mut tape, fixture, id, &prev_ids);
        }
        tape.finish();
    }
}

/// How many previous words the C++ `MultiBigramMap` caches: the first 25 distinct ones seen.
const CACHED_PREV_WORDS: usize = 25;

#[test]
fn multi_bigram_cache_answers_as_the_goldens_across_long_sequences() {
    for fixture in fixtures() {
        let policy = fixture.dictionary.policy();
        let mut tape = fixture.tape("multi_bigram_cache");
        let mut random = Random(0x003A_B16A);
        for sequence in 0..60 {
            let mut ours = MultiBigramMap::new();
            let mut first_words = HashSet::new();
            for step in 0..60 {
                let label = format!("sequence {sequence} step {step}");
                let prev = random_prev_ids(fixture, &mut random);
                let follower = if random.chance(2) {
                    follower_id(fixture, &prev, &mut random)
                } else {
                    None
                };
                let next = follower.unwrap_or_else(|| {
                    policy.get_word_id(&fixture.words[random.below(fixture.words.len())], false)
                });
                let unigram = random.below(257) as i32 - 1;
                record!(
                    tape,
                    ours.get_bigram_probability(policy, &prev, next, unigram)
                );
                record!(
                    tape,
                    policy
                        .get_word_attributes_in_context(&prev, next, Some(&mut ours))
                        .probability()
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
        tape.finish();
    }
}

/// Previous word ids for the cache test: none, one, or two, with some of them not words.
fn random_prev_ids(fixture: &Fixture, random: &mut Random) -> Vec<i32> {
    let count = random.below(3);
    (0..count)
        .map(|_| {
            if random.chance(8) {
                NOT_A_WORD_ID
            } else {
                let word = &fixture.words[random.below(fixture.words.len())];
                fixture.dictionary.policy().get_word_id(word, false)
            }
        })
        .collect()
}
