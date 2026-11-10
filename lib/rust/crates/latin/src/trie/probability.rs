//! `dictionary/utils/probability_utils`: how a unigram probability and a bigram's 4-bit probability
//! combine into the probability of a word after a previous word.

use crate::defines::{MAX_BIGRAM_ENCODED_PROBABILITY, MAX_PROBABILITY};

/// `ProbabilityUtils::backoff`: with no bigram, the unigram probability is the answer.
pub fn backoff(unigram_probability: i32) -> i32 {
    unigram_probability
}

/// `ProbabilityUtils::computeProbabilityForBigram`: the unigram probability plus the bigram's
/// share of what is left above it, in f32 as the C++ does it. The C++ divides the range
/// `[unigram..255]` into 16.5 steps, so the bigram's 0 to 15 lands in the middle of its step.
pub fn compute_probability_for_bigram(unigram_probability: i32, bigram_probability: i32) -> i32 {
    let step_size = (MAX_PROBABILITY - unigram_probability) as f32
        / (1.5_f32 + MAX_BIGRAM_ENCODED_PROBABILITY as f32);
    unigram_probability + ((bigram_probability + 1) as f32 * step_size) as i32
}
