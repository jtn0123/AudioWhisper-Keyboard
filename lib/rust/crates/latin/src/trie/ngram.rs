//! `dictionary/property/ngram_context`: up to three previous words, as the caller gives them, and
//! their word ids in the dictionary. The JNI rules for building one (`JniDataUtils::constructNgramContext`)
//! are in [`NgramContext::new`].

use super::patricia::PatriciaTriePolicy;
use crate::chars::attach_beginning_of_sentence_marker;
use crate::defines::{MAX_PREV_WORD_COUNT_FOR_N_GRAM, MAX_WORD_LENGTH, NOT_A_WORD_ID};

/// `NgramContext`: the previous words of a prediction, at most `MAX_PREV_WORD_COUNT_FOR_N_GRAM`.
#[derive(Clone, Debug)]
pub struct NgramContext {
    prev_word_count: usize,
    prev_word_code_points: [[i32; MAX_WORD_LENGTH]; MAX_PREV_WORD_COUNT_FOR_N_GRAM],
    prev_word_code_point_count: [usize; MAX_PREV_WORD_COUNT_FOR_N_GRAM],
    is_beginning_of_sentence: [bool; MAX_PREV_WORD_COUNT_FOR_N_GRAM],
}

impl Default for NgramContext {
    /// `NgramContext()`: no previous words.
    fn default() -> Self {
        Self::new(&[], &[])
    }
}

impl NgramContext {
    /// `NgramContext(prevWordCodePoints, prevWordCodePointCount, isBeginningOfSentence, prevWordCount)`.
    /// `prev_words` has a slot per previous word, `None` for one the caller didn't give. Only the
    /// first `MAX_PREV_WORD_COUNT_FOR_N_GRAM` slots are kept. A slot with more than `MAX_WORD_LENGTH`
    /// code points, or none, is empty, and so has no beginning-of-sentence flag. This is the rule
    /// `JniDataUtils::constructNgramContext` applies before the C++ constructor.
    pub fn new(prev_words: &[Option<&[i32]>], is_beginning_of_sentence: &[bool]) -> Self {
        let mut context = Self {
            prev_word_count: prev_words.len().min(MAX_PREV_WORD_COUNT_FOR_N_GRAM),
            prev_word_code_points: [[0; MAX_WORD_LENGTH]; MAX_PREV_WORD_COUNT_FOR_N_GRAM],
            prev_word_code_point_count: [0; MAX_PREV_WORD_COUNT_FOR_N_GRAM],
            is_beginning_of_sentence: [false; MAX_PREV_WORD_COUNT_FOR_N_GRAM],
        };
        for (i, slot) in prev_words.iter().take(context.prev_word_count).enumerate() {
            let Some(code_points) = slot else {
                continue;
            };
            if code_points.len() > MAX_WORD_LENGTH {
                continue;
            }
            context.prev_word_code_points[i][..code_points.len()].copy_from_slice(code_points);
            context.prev_word_code_point_count[i] = code_points.len();
            context.is_beginning_of_sentence[i] =
                is_beginning_of_sentence.get(i).copied().unwrap_or(false);
        }
        context
    }

    /// `NgramContext::getPrevWordCount`.
    pub fn prev_word_count(&self) -> usize {
        self.prev_word_count
    }

    /// `NgramContext::getNthPrevWordCodePoints`: `n` counts from 1; empty when there is no such word.
    pub fn nth_prev_word_code_points(&self, n: usize) -> &[i32] {
        match n.checked_sub(1) {
            Some(index) if index < self.prev_word_count => {
                &self.prev_word_code_points[index][..self.prev_word_code_point_count[index]]
            }
            _ => &[],
        }
    }

    /// `NgramContext::isNthPrevWordBeginningOfSentence`: `n` counts from 1.
    pub fn is_nth_prev_word_beginning_of_sentence(&self, n: usize) -> bool {
        match n.checked_sub(1) {
            Some(index) if index < self.prev_word_count => self.is_beginning_of_sentence[index],
            _ => false,
        }
    }

    /// `NgramContext::getPrevWordIds`: the word id of each previous word, into `out`, and the
    /// slice of those ids. A word that is not found is retried lower-cased when `try_lower_case`.
    pub fn prev_word_ids<'b>(
        &self,
        policy: &PatriciaTriePolicy,
        out: &'b mut [i32; MAX_PREV_WORD_COUNT_FOR_N_GRAM],
        try_lower_case: bool,
    ) -> &'b [i32] {
        for (i, slot) in out.iter_mut().enumerate().take(self.prev_word_count) {
            *slot = word_id(
                policy,
                &self.prev_word_code_points[i][..self.prev_word_code_point_count[i]],
                self.is_beginning_of_sentence[i],
                try_lower_case,
            );
        }
        &out[..self.prev_word_count]
    }
}

/// `NgramContext::getWordId`: the id of `word`, with the beginning-of-sentence marker attached when
/// `is_beginning_of_sentence`. When `try_lower_case` and the word is not found as it is, its
/// lower-cased form is looked up instead (for auto-capitalised words like "The").
pub fn word_id(
    policy: &PatriciaTriePolicy,
    word: &[i32],
    is_beginning_of_sentence: bool,
    try_lower_case: bool,
) -> i32 {
    if word.len() > MAX_WORD_LENGTH {
        return NOT_A_WORD_ID;
    }
    let mut code_points = [0_i32; MAX_WORD_LENGTH];
    code_points[..word.len()].copy_from_slice(word);
    let mut count = word.len();
    if is_beginning_of_sentence {
        count = attach_beginning_of_sentence_marker(&mut code_points, count, MAX_WORD_LENGTH);
        if count == 0 {
            return NOT_A_WORD_ID;
        }
    }
    let code_points = &code_points[..count];
    let word_id = policy.get_word_id(code_points, false);
    if word_id != NOT_A_WORD_ID || !try_lower_case {
        return word_id;
    }
    policy.get_word_id(code_points, true)
}
