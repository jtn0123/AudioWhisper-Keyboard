//! Unit tests with fixed values worked out from the C++: the probability arithmetic, a bigram list that
//! cannot be read, the previous-word rules of `NgramContext`, the multi-bigram cache's size, and the
//! word collector's filtering and surrogate pairs.

use super::pair;
use crate::defines::{MAX_WORD_LENGTH, NOT_A_DICT_POS, NOT_A_PROBABILITY};
use crate::trie::bigram::{BigramIterator, BigramListPolicy};
use crate::trie::interface::WordListener;
use crate::trie::jni::WordCollector;
use crate::trie::multi_bigram::MAX_CACHED_PREV_WORDS_IN_BIGRAM_MAP;
use crate::trie::ngram::NgramContext;
use crate::trie::probability::{backoff, compute_probability_for_bigram};

#[test]
fn bigram_probability_steps_the_unigram_towards_255() {
    // ProbabilityUtils::computeProbabilityForBigram: unigram + (bigram + 1) * ((255 - unigram) / 16.5).
    assert_eq!(compute_probability_for_bigram(0, 0), 15);
    assert_eq!(compute_probability_for_bigram(100, 15), 250);
    assert_eq!(compute_probability_for_bigram(255, 0), 255);
    assert_eq!(
        compute_probability_for_bigram(200, 7),
        200 + (8.0_f32 * 55.0 / 16.5) as i32
    );
}

#[test]
fn the_policy_backs_off_to_the_unigram_without_a_bigram() {
    let policy = pair().ours.policy();
    for unigram in 0..=255 {
        assert_eq!(backoff(unigram), unigram);
        assert_eq!(policy.get_probability(unigram, NOT_A_PROBABILITY), unigram);
        assert_eq!(
            policy.get_probability(unigram, 9),
            compute_probability_for_bigram(unigram, 9)
        );
    }
}

#[test]
fn a_bigram_list_that_cannot_be_read_is_reported_once_then_ends() {
    let mut iterator = BigramIterator::new(BigramListPolicy::new(&[]), 0);
    assert!(iterator.has_next(), "a list at position 0 starts");
    iterator.next_entry();
    assert_eq!(iterator.probability(), NOT_A_PROBABILITY);
    assert_eq!(
        iterator.bigram_pos(),
        NOT_A_DICT_POS,
        "the position is left where it was"
    );
    assert!(!iterator.has_next(), "the failed read ends the list");
}

#[test]
fn an_absent_bigram_list_has_no_entries() {
    let iterator = BigramIterator::new(BigramListPolicy::new(&[]), NOT_A_DICT_POS);
    assert!(!iterator.has_next());
}

#[test]
fn ngram_context_keeps_three_previous_words_and_empties_null_and_long_ones() {
    let long = vec![7; MAX_WORD_LENGTH + 1];
    let first: &[i32] = &[1, 2];
    let context = NgramContext::new(
        &[Some(first), None, Some(long.as_slice()), Some(first)],
        &[true, true, true, true],
    );
    assert_eq!(context.prev_word_count(), 3, "at most three previous words");
    assert_eq!(context.nth_prev_word_code_points(1), &[1, 2]);
    assert!(context.is_nth_prev_word_beginning_of_sentence(1));
    assert!(
        context.nth_prev_word_code_points(2).is_empty(),
        "a null slot is empty"
    );
    assert!(!context.is_nth_prev_word_beginning_of_sentence(2));
    assert!(
        context.nth_prev_word_code_points(3).is_empty(),
        "an over-long slot is empty"
    );
    assert!(!context.is_nth_prev_word_beginning_of_sentence(3));
    assert!(
        context.nth_prev_word_code_points(4).is_empty(),
        "no fourth word"
    );
}

#[test]
fn ngram_context_keeps_a_previous_word_of_exactly_the_longest_length() {
    // JniDataUtils::constructNgramContext drops only a word longer than MAX_WORD_LENGTH.
    let longest = vec![7; MAX_WORD_LENGTH];
    let context = NgramContext::new(&[Some(longest.as_slice())], &[true]);
    assert_eq!(context.nth_prev_word_code_points(1), longest.as_slice());
    assert!(context.is_nth_prev_word_beginning_of_sentence(1));
}

#[test]
fn the_multi_bigram_cache_holds_25_previous_words() {
    assert_eq!(MAX_CACHED_PREV_WORDS_IN_BIGRAM_MAP, 25);
}

#[test]
fn the_word_collector_drops_control_words_and_splits_supplementary_code_points() {
    let mut collector = WordCollector::new();
    collector.on_visit_word(&[i32::from(b'a'), 0x1F]);
    collector.on_visit_word(&[0x1F600]);
    collector.on_visit_word(&[i32::from(b'b'), 0x110000]);
    assert_eq!(collector.utf16(), &[0xD83D, 0xDE00, 0x0A][..]);
}

#[test]
fn the_word_collector_keeps_a_word_with_a_space() {
    // WordCollector::onVisitWord drops only code points below the space.
    let mut collector = WordCollector::new();
    collector.on_visit_word(&[i32::from(b'a'), i32::from(b' '), i32::from(b'b')]);
    assert_eq!(collector.utf16(), &[0x61, 0x20, 0x62, 0x0A][..]);
}
