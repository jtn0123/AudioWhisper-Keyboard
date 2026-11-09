//! `EntryCounts` and `MutableEntryCounters` (`dictionary/utils/entry_counters.h`), and the
//! `NgramType` they are indexed by (`utils/ngram_utils.h`).

use crate::defines::MAX_PREV_WORD_COUNT_FOR_N_GRAM;

/// The counts the header holds, from the unigram up to the `MAX_PREV_WORD_COUNT_FOR_N_GRAM + 1`-gram.
const COUNT_SLOTS: usize = MAX_PREV_WORD_COUNT_FOR_N_GRAM + 1;

/// `NgramType` (`utils/ngram_utils.h`), without `NotANgramType`, which the header never reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NgramType {
    Unigram = 0,
    Bigram = 1,
    Trigram = 2,
    Quadgram = 3,
}

impl NgramType {
    /// `AllNgramTypes::ASCENDING`. The C++ list stops at the trigram, so the header never reads a
    /// quadgram count or maximum: both stay 0 whatever the file says.
    pub const ASCENDING: [NgramType; 3] =
        [NgramType::Unigram, NgramType::Bigram, NgramType::Trigram];
}

/// `EntryCounts`: a copy of the counts, indexed by [`NgramType`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EntryCounts {
    counts: [i32; COUNT_SLOTS],
}

impl EntryCounts {
    /// `EntryCounts(counters)`.
    pub fn new(counts: [i32; COUNT_SLOTS]) -> Self {
        Self { counts }
    }

    /// `EntryCounts::getNgramCount`.
    pub fn ngram_count(&self, ngram_type: NgramType) -> i32 {
        self.counts[ngram_type as usize]
    }

    /// `EntryCounts::getCountArray`.
    pub fn count_array(&self) -> &[i32; COUNT_SLOTS] {
        &self.counts
    }
}

/// `MutableEntryCounters`: counts set one at a time, all 0 to begin with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MutableEntryCounters {
    counters: [i32; COUNT_SLOTS],
}

impl MutableEntryCounters {
    /// `MutableEntryCounters::setNgramCount`.
    pub fn set_ngram_count(&mut self, ngram_type: NgramType, count: i32) {
        self.counters[ngram_type as usize] = count;
    }

    /// `MutableEntryCounters::getEntryCounts`.
    pub fn entry_counts(&self) -> EntryCounts {
        EntryCounts::new(self.counters)
    }
}
