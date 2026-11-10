//! `suggest/core/dictionary/dictionary`: the kinds of suggestion, and `getPredictions` with its
//! `NgramListenerForPrediction`. `getSuggestions` is [`crate::search::suggest::get_suggestions`].
//! The dictionary's probability queries are in `trie::dictionary`.

use crate::defines::{MAX_WORD_LENGTH, NOT_A_PROBABILITY, NOT_A_WORD_ID};
use crate::search::result::suggestion_results::SuggestionResults;
use crate::trie::interface::NgramListener;
use crate::trie::multi_bigram::MultiBigramMap;
use crate::trie::ngram::NgramContext;
use crate::trie::patricia::PatriciaTriePolicy;

/// `Dictionary::KIND_CORRECTION`: a simple correction or suggestion.
pub const KIND_CORRECTION: i32 = 1;
/// `Dictionary::KIND_WHITELIST`.
pub const KIND_WHITELIST: i32 = 3;
/// `Dictionary::KIND_SHORTCUT`.
pub const KIND_SHORTCUT: i32 = 7;
/// `Dictionary::KIND_PREDICTION`: a suggestion with no input.
pub const KIND_PREDICTION: i32 = 8;

/// `Dictionary::KIND_FLAG_POSSIBLY_OFFENSIVE` (0x80000000 as the C++ `int`).
pub const KIND_FLAG_POSSIBLY_OFFENSIVE: i32 = 0x8000_0000_u32 as i32;
/// `Dictionary::KIND_FLAG_EXACT_MATCH`.
pub const KIND_FLAG_EXACT_MATCH: i32 = 0x4000_0000;
/// `Dictionary::KIND_FLAG_EXACT_MATCH_WITH_INTENTIONAL_OMISSION`.
pub const KIND_FLAG_EXACT_MATCH_WITH_INTENTIONAL_OMISSION: i32 = 0x2000_0000;
/// `Dictionary::KIND_FLAG_APPROPRIATE_FOR_AUTOCORRECTION`.
pub const KIND_FLAG_APPROPRIATE_FOR_AUTOCORRECTION: i32 = 0x1000_0000;

/// `Dictionary::NgramListenerForPrediction`: adds each word that follows the previous word as a
/// prediction.
struct NgramListenerForPrediction<'a> {
    /// `mNgramContext`.
    ngram_context: &'a NgramContext,
    /// `mPrevWordIds`.
    prev_word_ids: &'a [i32],
    /// `mSuggestionResults`.
    suggestion_results: &'a mut SuggestionResults,
    /// `mDictStructurePolicy`.
    policy: &'a PatriciaTriePolicy,
}

impl NgramListener for NgramListenerForPrediction<'_> {
    /// `NgramListenerForPrediction::onVisitEntry`.
    fn on_visit_entry(&mut self, ngram_probability: i32, target_word_id: i32) {
        if target_word_id == NOT_A_WORD_ID {
            return;
        }
        if self.ngram_context.is_nth_prev_word_beginning_of_sentence(1)
            && ngram_probability == NOT_A_PROBABILITY
        {
            return;
        }
        let mut target_word_code_points = [0; MAX_WORD_LENGTH];
        let code_point_count = self.policy.get_code_points_and_return_code_point_count(
            target_word_id,
            MAX_WORD_LENGTH,
            &mut target_word_code_points,
        );
        if code_point_count == 0 {
            return;
        }
        let word_attributes = self.policy.get_word_attributes_in_context(
            self.prev_word_ids,
            target_word_id,
            None::<&mut MultiBigramMap>,
        );
        if word_attributes.probability() == NOT_A_PROBABILITY {
            return;
        }
        self.suggestion_results.add_prediction(
            &target_word_code_points[..code_point_count.min(MAX_WORD_LENGTH)],
            word_attributes.probability(),
        );
    }
}

/// `Dictionary::getPredictions`: the words that follow the previous words of `ngram_context`, as
/// suggestions with no input.
pub fn get_predictions(
    policy: &PatriciaTriePolicy,
    ngram_context: &NgramContext,
    out_suggestion_results: &mut SuggestionResults,
) {
    let mut prev_word_id_array = [NOT_A_WORD_ID; crate::defines::MAX_PREV_WORD_COUNT_FOR_N_GRAM];
    let prev_word_ids = ngram_context.prev_word_ids(policy, &mut prev_word_id_array, true);
    let mut listener = NgramListenerForPrediction {
        ngram_context,
        prev_word_ids,
        suggestion_results: out_suggestion_results,
        policy,
    };
    policy.iterate_ngram_entries(prev_word_ids, &mut listener);
}
