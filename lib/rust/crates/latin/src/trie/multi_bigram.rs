//! `dictionary/utils/multi_bigram_map` and `bloom_filter`: a cache of the bigrams of previous
//! words, so that a walk that asks about many words after the same one reads each bigram list once.

use std::collections::HashMap;

use super::interface::NgramListener;
use super::patricia::PatriciaTriePolicy;
use crate::defines::{NOT_A_PROBABILITY, NOT_A_WORD_ID};

/// `MultiBigramMap::MAX_CACHED_PREV_WORDS_IN_BIGRAM_MAP`: how many previous words are cached.
pub const MAX_CACHED_PREV_WORDS_IN_BIGRAM_MAP: usize = 25;
/// `MultiBigramMap::BigramMap::DEFAULT_HASH_MAP_SIZE_FOR_EACH_BIGRAM_MAP`.
const DEFAULT_HASH_MAP_SIZE_FOR_EACH_BIGRAM_MAP: usize = 100;
/// `BloomFilter::BIGRAM_FILTER_MODULO`.
const BIGRAM_FILTER_MODULO: usize = 1021;
/// The filter's bits, as words of 64.
const FILTER_WORDS: usize = BIGRAM_FILTER_MODULO.div_ceil(64);

/// `BloomFilter`: a set of word ids, with false positives. Ids are taken modulo 1021, as the C++ does.
#[derive(Clone, Copy, Debug)]
struct BloomFilter {
    bits: [u64; FILTER_WORDS],
}

impl BloomFilter {
    /// `BloomFilter()`: empty.
    fn new() -> Self {
        Self {
            bits: [0; FILTER_WORDS],
        }
    }

    /// `BloomFilter::getIndex`. A negative id reads as its unsigned value, as the C++'s
    /// `static_cast<size_t>` does.
    fn index(position: i32) -> usize {
        (position as usize) % BIGRAM_FILTER_MODULO
    }

    /// `BloomFilter::setInFilter`.
    fn set_in_filter(&mut self, position: i32) {
        let index = Self::index(position);
        self.bits[index / 64] |= 1_u64 << (index % 64);
    }

    /// `BloomFilter::isInFilter`.
    fn is_in_filter(&self, position: i32) -> bool {
        let index = Self::index(position);
        self.bits[index / 64] & (1_u64 << (index % 64)) != 0
    }
}

/// `MultiBigramMap::BigramMap`: the bigrams of one previous word, by target word id.
#[derive(Clone, Debug)]
struct BigramMap {
    bigrams: HashMap<i32, i32>,
    filter: BloomFilter,
}

impl BigramMap {
    /// `BigramMap()`: empty.
    fn new() -> Self {
        Self {
            bigrams: HashMap::with_capacity(DEFAULT_HASH_MAP_SIZE_FOR_EACH_BIGRAM_MAP),
            filter: BloomFilter::new(),
        }
    }

    /// `BigramMap::getBigramProbability`: the bigram's combined probability for `next_word_id`,
    /// from the unigram `unigram_probability`.
    fn bigram_probability(
        &self,
        policy: &PatriciaTriePolicy,
        next_word_id: i32,
        unigram_probability: i32,
    ) -> i32 {
        let bigram_probability = if self.filter.is_in_filter(next_word_id) {
            self.bigrams
                .get(&next_word_id)
                .copied()
                .unwrap_or(NOT_A_PROBABILITY)
        } else {
            NOT_A_PROBABILITY
        };
        policy.get_probability(unigram_probability, bigram_probability)
    }
}

impl NgramListener for BigramMap {
    /// `BigramMap::onVisitEntry`: a later entry for the same target replaces the earlier one.
    fn on_visit_entry(&mut self, ngram_probability: i32, target_word_id: i32) {
        if target_word_id == NOT_A_WORD_ID {
            return;
        }
        self.bigrams.insert(target_word_id, ngram_probability);
        self.filter.set_in_filter(target_word_id);
    }
}

/// `MultiBigramMap`: up to `MAX_CACHED_PREV_WORDS_IN_BIGRAM_MAP` previous words' bigram maps.
#[derive(Clone, Debug, Default)]
pub struct MultiBigramMap {
    bigram_maps: HashMap<i32, BigramMap>,
}

impl MultiBigramMap {
    /// `MultiBigramMap()`: nothing cached.
    pub fn new() -> Self {
        Self::default()
    }

    /// `MultiBigramMap::clear`: forgets every cached previous word.
    pub fn clear(&mut self) {
        self.bigram_maps.clear();
    }

    /// The number of previous words cached.
    pub fn cached_prev_word_count(&self) -> usize {
        self.bigram_maps.len()
    }

    /// `MultiBigramMap::getBigramProbability`: the probability of `next_word_id` after the previous
    /// words, from the cache when the first previous word is cached, caching it when there is room,
    /// and otherwise from the dictionary.
    pub fn get_bigram_probability(
        &mut self,
        policy: &PatriciaTriePolicy,
        prev_word_ids: &[i32],
        next_word_id: i32,
        unigram_probability: i32,
    ) -> i32 {
        let Some(&first_prev) = prev_word_ids.first() else {
            return policy.get_probability(unigram_probability, NOT_A_PROBABILITY);
        };
        if first_prev == NOT_A_WORD_ID {
            return policy.get_probability(unigram_probability, NOT_A_PROBABILITY);
        }
        if let Some(map) = self.bigram_maps.get(&first_prev) {
            return map.bigram_probability(policy, next_word_id, unigram_probability);
        }
        if self.bigram_maps.len() < MAX_CACHED_PREV_WORDS_IN_BIGRAM_MAP {
            let mut map = BigramMap::new();
            policy.iterate_ngram_entries(prev_word_ids, &mut map);
            let probability = map.bigram_probability(policy, next_word_id, unigram_probability);
            self.bigram_maps.insert(first_prev, map);
            return probability;
        }
        read_bigram_probability_from_binary_dictionary(
            policy,
            prev_word_ids,
            next_word_id,
            unigram_probability,
        )
    }
}

/// `MultiBigramMap::readBigramProbabilityFromBinaryDictionary`: the bigram read from the dictionary
/// when the cache is full.
fn read_bigram_probability_from_binary_dictionary(
    policy: &PatriciaTriePolicy,
    prev_word_ids: &[i32],
    next_word_id: i32,
    unigram_probability: i32,
) -> i32 {
    let bigram_probability = policy.get_probability_of_word(prev_word_ids, next_word_id);
    if bigram_probability != NOT_A_PROBABILITY {
        return bigram_probability;
    }
    policy.get_probability(unigram_probability, NOT_A_PROBABILITY)
}
