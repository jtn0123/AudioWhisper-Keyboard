//! Swipe typing's rules (`suggest::glide_ranker` and `suggest::swipe_words`), for `SwipeNative`,
//! whose generated entry points call these or the crate directly. Kotlin asks the dictionary and
//! the learned words what they know and passes it in; these turn it into frequencies, bonuses and
//! orders.

use suggest::glide_ranker::{self, Context};
use suggest::swipe_words;

use crate::jni::{self, Result, illegal};

/// The most a guess's context can take off its cost (`glide_ranker::MAX_BONUS`).
pub fn max_glide_bonus() -> f32 {
    glide_ranker::MAX_BONUS
}

/// How much a guess's context favours it (`glide_ranker::bonus`).
pub fn glide_bonus(pair_count: i32, ngram: i32, probability: i32) -> f32 {
    glide_ranker::bonus(Context {
        pair_count,
        ngram,
        probability,
    })
}

/// The guesses' order, best first, as their indexes (`glide_ranker::rank`): each guess has a cost
/// and, at the same index, its context's pair count, n-gram and probability.
pub fn rank_glide(
    costs: &[f32],
    pair_counts: &[i32],
    ngrams: &[i32],
    probabilities: &[i32],
) -> Result<Vec<i32>> {
    if ngrams.len() != pair_counts.len() || probabilities.len() != pair_counts.len() {
        return Err(illegal("costs and contexts differ in length"));
    }
    let contexts: Vec<Context> = (pair_counts.iter().zip(ngrams).zip(probabilities))
        .map(|((&pair_count, &ngram), &probability)| Context {
            pair_count,
            ngram,
            probability,
        })
        .collect();
    let order = glide_ranker::rank(costs, &contexts)
        .ok_or_else(|| illegal("costs and contexts differ in length"))?;
    // There are no more guesses than a Java array holds.
    Ok(order.into_iter().map(|i| i as i32).collect())
}

pub fn max_probability() -> i32 {
    swipe_words::MAX_PROBABILITY
}

pub fn common() -> i32 {
    swipe_words::COMMON
}

pub fn cost_per_step() -> f64 {
    swipe_words::COST_PER_STEP
}

pub fn learned_probability() -> i32 {
    swipe_words::LEARNED_PROBABILITY
}

/// The indexes of the words a swipe can spell, in order. The words are `text` cut at `lengths`
/// ([`jni::cut`]), so a whole dictionary crosses in one string.
pub fn swipeable(text: &[u16], lengths: &[i32]) -> Result<Vec<i32>> {
    let words = jni::cut(text, lengths)?;
    // There are no more words than a Java array holds.
    Ok((0..words.len())
        .filter(|&i| swipe_words::is_swipeable(words[i]))
        .map(|i| i as i32)
        .collect())
}

/// The user's words (`swipe_words::user_words`), packed: for each, where it first appears among
/// `learned` followed by `personal_words`, then its probability.
pub fn user_words(
    learned: &[Vec<u16>],
    personal_words: &[Vec<u16>],
    personal_frequencies: &[i32],
) -> Result<Vec<i32>> {
    if personal_frequencies.len() != personal_words.len() {
        return Err(illegal("personal words and frequencies differ in length"));
    }
    let learned: Vec<&[u16]> = learned.iter().map(Vec::as_slice).collect();
    let personal: Vec<(&[u16], i32)> = (personal_words.iter().map(Vec::as_slice))
        .zip(personal_frequencies.iter().copied())
        .collect();
    let mut packed = Vec::new();
    for (place, probability) in swipe_words::user_words(&personal, &learned) {
        let place = i32::try_from(place).map_err(|_| illegal("too many words"))?;
        packed.extend([place, probability]);
    }
    Ok(packed)
}
