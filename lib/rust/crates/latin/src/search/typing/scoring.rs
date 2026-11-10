//! `suggest/policyimpl/typing/typing_scoring`: turns a compound distance into the integer score
//! the app reads. The C++ reaches it through the virtual `Scoring`; typing is the only
//! implementation reachable, so these are plain functions.
//!
//! `getMostProbableString` is a no-op for typing and is not ported.

use crate::defines::SUGGEST_INTERFACE_OUTPUT_SCALE;
use crate::layout::proximity_info_state::ProximityInfoState;
use crate::search::dicnode::node::DicNode;
use crate::search::error_type::{self, ErrorType};
use crate::search::scoring_params as params;

/// `TypingScoring::calculateFinalScore`.
///
/// `maxDistance = DISTANCE_WEIGHT_LANGUAGE + (float)inputSize * TYPING_MAX_OUTPUT_SCORE_PER_INPUT`
/// is one expression, so clang fuses it into a multiply-add on arm64 (both constants live in
/// another translation unit, so nothing is folded): it is a `mul_add` here. This is one of the two
/// places the search is fused; the other is `DicNodeStateScoring::getCompoundDistance`.
pub fn calculate_final_score(
    compound_distance: f32,
    input_size: i32,
    contained_error_types: ErrorType,
    force_commit: bool,
    boost_exact_matches: bool,
    has_probability_zero: bool,
) -> i32 {
    let max_distance = (input_size as f32).mul_add(
        params::TYPING_MAX_OUTPUT_SCORE_PER_INPUT,
        params::DISTANCE_WEIGHT_LANGUAGE,
    );
    let mut score = params::TYPING_BASE_OUTPUT_SCORE - compound_distance / max_distance;
    if force_commit {
        score += params::AUTOCORRECT_OUTPUT_THRESHOLD;
    }
    if has_probability_zero {
        // A word of probability zero (a distracter) is only boosted for a perfect match, so that
        // typing "mt" gives "my" and not "Mt".
        if boost_exact_matches && error_type::is_perfect_match(contained_error_types) {
            score += params::PERFECT_MATCH_PROMOTION;
        }
    } else if boost_exact_matches && error_type::is_exact_match(contained_error_types) {
        score += params::EXACT_MATCH_PROMOTION;
        if (error_type::MATCH_WITH_WRONG_CASE & contained_error_types) != 0 {
            score -= params::CASE_ERROR_PENALTY_FOR_EXACT_MATCH;
        }
        if (error_type::MATCH_WITH_MISSING_ACCENT & contained_error_types) != 0 {
            score -= params::ACCENT_ERROR_PENALTY_FOR_EXACT_MATCH;
        }
        if (error_type::MATCH_WITH_DIGRAPH & contained_error_types) != 0 {
            score -= params::DIGRAPH_PENALTY_FOR_EXACT_MATCH;
        }
    }
    // static_cast<int>: truncates toward zero. Out of range the C++ is undefined; Rust saturates,
    // and a NaN is 0 (what arm64 gives too).
    (score * SUGGEST_INTERFACE_OUTPUT_SCALE) as i32
}

/// `TypingScoring::getAdjustedWeightOfLangModelVsSpatialModel`.
pub fn adjusted_weight_of_lang_model_vs_spatial_model() -> f32 {
    1.0
}

/// `TypingScoring::getDoubleLetterDemotionDistanceCost`.
pub fn double_letter_demotion_distance_cost() -> f32 {
    0.0
}

/// `TypingScoring::autoCorrectsToMultiWordSuggestionIfTop`.
pub fn auto_corrects_to_multi_word_suggestion_if_top() -> bool {
    true
}

/// `TypingScoring::sameAsTyped`: whether the node's word, from the start of the output buffer,
/// is what was typed.
pub fn same_as_typed(state: &ProximityInfoState, dic_node: &DicNode) -> bool {
    let buf = dic_node.output_word_buf();
    let count = usize::from(dic_node.node_code_point_count()).min(buf.len());
    state.same_as_typed(&buf[..count])
}
