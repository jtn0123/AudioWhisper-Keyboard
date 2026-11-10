//! `suggest/core/dictionary/dictionary`: the probability queries of a dictionary, over its structure
//! policy. `Dictionary::getProbability` and `getNgramProbability` are here, and `iterateWords`.
//! The suggestion entry points (`getSuggestions`, `getPredictions`) need the search's DicNodes and
//! results, and are not ported (see the TODO(search) list in `trie/mod.rs`).

use std::sync::Arc;

use super::factory::new_policy_for_file;
use super::interface::WordListener;
use super::ngram::NgramContext;
use super::patricia::PatriciaTriePolicy;
use crate::defines::{MAX_PREV_WORD_COUNT_FOR_N_GRAM, NOT_A_PROBABILITY, NOT_A_WORD_ID};
use crate::header::HeaderPolicy;

/// `Dictionary`: a dictionary, to answer probability questions about its words.
#[derive(Debug)]
pub struct Dictionary {
    header: HeaderPolicy,
    policy: PatriciaTriePolicy,
}

impl Dictionary {
    /// `Dictionary(env, dictionaryStructureWithBufferPolicy)`, without the logging: the policy and
    /// the header it was read with (the C++ policy owns its header).
    pub fn new(header: HeaderPolicy, policy: PatriciaTriePolicy) -> Self {
        Self { header, policy }
    }

    /// `BinaryDictionary.openNative` for a whole dictionary file: its header, then its structure.
    /// `None` when the file is not a dictionary of a ported format version.
    pub fn open(file: Arc<[u8]>) -> Option<Self> {
        let (header, policy) = new_policy_for_file(file)?;
        Some(Self::new(header, policy))
    }

    /// `getHeaderStructurePolicy`: the dictionary's header.
    pub fn header(&self) -> &HeaderPolicy {
        &self.header
    }

    /// `Dictionary::getDictionaryStructurePolicy`.
    pub fn policy(&self) -> &PatriciaTriePolicy {
        &self.policy
    }

    /// `Dictionary::getProbability`: the probability of `code_points` with no previous word.
    pub fn probability(&self, code_points: &[i32]) -> i32 {
        self.ngram_probability(None, code_points)
    }

    /// `Dictionary::getNgramProbability`: the probability of `code_points` after the previous words
    /// of `ngram` (none when it is `None`). NOT_A_PROBABILITY for a word that is not in the
    /// dictionary.
    pub fn ngram_probability(&self, ngram: Option<&NgramContext>, code_points: &[i32]) -> i32 {
        let word_id = self.policy.get_word_id(code_points, false);
        if word_id == NOT_A_WORD_ID {
            return NOT_A_PROBABILITY;
        }
        let Some(ngram) = ngram else {
            return self.policy.get_probability_of_word(&[], word_id);
        };
        let mut prev_word_ids = [NOT_A_WORD_ID; MAX_PREV_WORD_COUNT_FOR_N_GRAM];
        let prev_word_ids = ngram.prev_word_ids(&self.policy, &mut prev_word_ids, true);
        self.policy.get_probability_of_word(prev_word_ids, word_id)
    }

    /// `Dictionary::iterateWords`: every word of the dictionary, to `listener`.
    pub fn iterate_words<L: WordListener>(&self, listener: &mut L) {
        self.policy.iterate_words(listener);
    }
}
