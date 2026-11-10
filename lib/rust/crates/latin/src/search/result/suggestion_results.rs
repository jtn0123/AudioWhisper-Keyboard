//! `suggest/core/result/suggestion_results` and `suggested_word`: the best suggestions found, in
//! the heap the C++ keeps them in.
//!
//! The C++ `std::priority_queue` pops the worst first, so index 0 of the output is the worst of
//! the suggestions and the last is the best. Two suggestions of the same score and length are not
//! ordered by the comparator, so which one is dropped or popped first depends on the heap's moves;
//! [`crate::search::heap`] replays libc++'s.

use crate::defines::{
    MAX_WORD_LENGTH, NOT_A_FIRST_WORD_CONFIDENCE, NOT_A_PROBABILITY,
    NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL, NOT_AN_INDEX,
};
use crate::search::dictionary::KIND_PREDICTION;
use crate::search::heap;

/// `SuggestedWord`. The code points are in a fixed array, so a suggestion needs no allocation.
#[derive(Clone, Copy, Debug)]
pub struct SuggestedWord {
    /// `mCodePoints` (only the first `code_point_count` are the word).
    code_points: [i32; MAX_WORD_LENGTH],
    /// The word's length.
    code_point_count: usize,
    /// `mScore`.
    score: i32,
    /// `mType`.
    kind: i32,
    /// `mIndexToPartialCommit`.
    index_to_partial_commit: i32,
    /// `mAutoCommitFirstWordConfidence`.
    auto_commit_first_word_confidence: i32,
}

impl SuggestedWord {
    /// `SuggestedWord(codePoints, codePointCount, score, type, indexToPartialCommit,
    /// autoCommitFirstWordConfidence)`; `code_points` is 1 to `MAX_WORD_LENGTH` long.
    fn new(
        code_points: &[i32],
        score: i32,
        kind: i32,
        index_to_partial_commit: i32,
        auto_commit_first_word_confidence: i32,
    ) -> Self {
        let mut word = [0; MAX_WORD_LENGTH];
        word[..code_points.len()].copy_from_slice(code_points);
        Self {
            code_points: word,
            code_point_count: code_points.len(),
            score,
            kind,
            index_to_partial_commit,
            auto_commit_first_word_confidence,
        }
    }

    /// `getCodePoint`, up to `getCodePointCount`.
    pub fn code_points(&self) -> &[i32] {
        &self.code_points[..self.code_point_count]
    }

    /// `getScore`.
    pub fn score(&self) -> i32 {
        self.score
    }

    /// `getType`.
    pub fn kind(&self) -> i32 {
        self.kind
    }

    /// `getIndexToPartialCommit`.
    pub fn index_to_partial_commit(&self) -> i32 {
        self.index_to_partial_commit
    }

    /// `getAutoCommitFirstWordConfidence`.
    pub fn auto_commit_first_word_confidence(&self) -> i32 {
        self.auto_commit_first_word_confidence
    }
}

/// `SuggestedWord::Comparator`: `left` goes below `right` when its score is higher, or when the
/// scores are equal and it is shorter. The worst suggestion is the top of the heap.
fn less(left: &SuggestedWord, right: &SuggestedWord) -> bool {
    if left.score != right.score {
        return left.score > right.score;
    }
    left.code_point_count < right.code_point_count
}

/// `SuggestionResults`.
#[derive(Debug)]
pub struct SuggestionResults {
    /// `mMaxSuggestionCount`.
    max_suggestion_count: usize,
    /// `mWeightOfLangModelVsSpatialModel`.
    weight_of_lang_model_vs_spatial_model: f32,
    /// `mSuggestedWords`: the heap of words, worst on top.
    suggested_words: Vec<SuggestedWord>,
}

impl SuggestionResults {
    /// `SuggestionResults(maxSuggestionCount)`.
    pub fn new(max_suggestion_count: usize) -> Self {
        Self {
            max_suggestion_count,
            weight_of_lang_model_vs_spatial_model: NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL,
            suggested_words: Vec::with_capacity(max_suggestion_count + 1),
        }
    }

    /// Back to the state of `SuggestionResults(maxSuggestionCount)`, keeping the storage.
    pub fn clear(&mut self) {
        self.weight_of_lang_model_vs_spatial_model = NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL;
        self.suggested_words.clear();
    }

    /// `addPrediction`.
    pub fn add_prediction(&mut self, code_points: &[i32], probability: i32) {
        if probability == NOT_A_PROBABILITY {
            // Invalid word.
            return;
        }
        self.add_suggestion(
            code_points,
            probability,
            KIND_PREDICTION,
            NOT_AN_INDEX,
            NOT_A_FIRST_WORD_CONFIDENCE,
        );
    }

    /// `addSuggestion`.
    pub fn add_suggestion(
        &mut self,
        code_points: &[i32],
        score: i32,
        kind: i32,
        index_to_partial_commit: i32,
        auto_commit_first_word_confidence: i32,
    ) {
        if code_points.is_empty() || code_points.len() > MAX_WORD_LENGTH {
            // Invalid word.
            return;
        }
        if self.suggestion_count() >= self.max_suggestion_count {
            // The C++ reads the top of an empty queue here for a maximum of 0, which nothing sets.
            let Some(worst) = self.suggested_words.first() else {
                return;
            };
            if score > worst.score
                || (score == worst.score && code_points.len() < worst.code_point_count)
            {
                self.pop();
            } else {
                return;
            }
        }
        self.suggested_words.push(SuggestedWord::new(
            code_points,
            score,
            kind,
            index_to_partial_commit,
            auto_commit_first_word_confidence,
        ));
        heap::push_heap(&mut self.suggested_words, less);
    }

    /// `getSuggestionCount`.
    pub fn suggestion_count(&self) -> usize {
        self.suggested_words.len()
    }

    /// `setWeightOfLangModelVsSpatialModel`.
    pub fn set_weight_of_lang_model_vs_spatial_model(&mut self, weight: f32) {
        self.weight_of_lang_model_vs_spatial_model = weight;
    }

    /// The weight `outputSuggestions` writes to `inOutWeightOfLangModelVsSpatialModel`.
    pub fn weight_of_lang_model_vs_spatial_model(&self) -> f32 {
        self.weight_of_lang_model_vs_spatial_model
    }

    /// `mSuggestedWords.top()`: the worst suggestion.
    pub fn top(&self) -> Option<&SuggestedWord> {
        self.suggested_words.first()
    }

    /// `mSuggestedWords.pop()`.
    pub fn pop(&mut self) {
        heap::pop_heap(&mut self.suggested_words, less);
        self.suggested_words.pop();
    }
}
