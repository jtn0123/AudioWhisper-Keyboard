//! `suggest/policyimpl/typing/typing_weighting` and `suggest/core/policy/weighting`: what each
//! kind of correction costs, and the one routine that applies it to a node.
//!
//! The C++ `Weighting` is virtual with a typing and a gesture implementation; only typing is
//! reachable, so [`add_cost_and_forward_input_index`] calls the typing costs directly. The
//! gesture-only `DicNode_InputStateG` is not ported (typing never sets `mNeedsToUpdateInputStateG`),
//! so the `else` branch of `addCostAndForwardInputIndex` is the only one.
//!
//! Every expression here is a single multiply or a single add, or is split over statements the
//! way the C++ is, so clang has nothing to fuse: no `mul_add` is needed (the compound distance in
//! `dicnode::state_scoring` and `calculateFinalScore` are the two places that are fused).

use crate::chars;
use crate::defines::{CorrectionType, KEYCODE_SPACE, NOT_AN_INDEX};
use crate::layout::touch_position_correction_utils::get_sweet_spot_factor;
use crate::search::dicnode::node::DicNode;
use crate::search::dicnode::utils::get_bigram_node_improbability;
use crate::search::error_type::{self, ErrorType};
use crate::search::scoring_params as params;
use crate::search::session::SessionView;
use crate::trie::multi_bigram::MultiBigramMap;

/// `Weighting::addCostAndForwardInputIndex`: adds the cost of `correction_type` to `dic_node`
/// and moves it forward in the input. `parent_dic_node` is the node the correction started from,
/// for the corrections that need it (omission, insertion, transposition and the new word ones);
/// the C++ passes null otherwise.
pub fn add_cost_and_forward_input_index(
    v: &SessionView<'_>,
    correction_type: CorrectionType,
    parent_dic_node: Option<&DicNode>,
    dic_node: &mut DicNode,
    multi_bigram_map: Option<&mut MultiBigramMap>,
) {
    let spatial_cost = spatial_cost(v, correction_type, parent_dic_node, dic_node);
    let language_cost = language_cost(
        v,
        correction_type,
        parent_dic_node,
        dic_node,
        multi_bigram_map,
    );
    let error_type = error_type(v, correction_type, parent_dic_node, dic_node);
    dic_node.forward_input_index(0, forward_input_count(correction_type));
    // `needsToNormalizeCompoundDistance` is false for typing: `add_cost` does not normalize.
    dic_node.add_cost(spatial_cost, language_cost, error_type);
    if correction_type == CorrectionType::NewWordSpaceOmission {
        // When we are on a terminal, we save the current distance for evaluating
        // when to auto-commit partial suggestions.
        dic_node.save_normalized_compound_distance_after_first_word_if_none_yet();
    }
}

/// The node a correction started from. Every caller that needs one gives one (the C++ would read
/// through a null pointer otherwise); without one, the node itself stands in.
fn parent_or_self<'a>(parent_dic_node: Option<&'a DicNode>, dic_node: &'a DicNode) -> &'a DicNode {
    parent_dic_node.unwrap_or(dic_node)
}

/// `Weighting::getSpatialCost`.
fn spatial_cost(
    v: &SessionView<'_>,
    correction_type: CorrectionType,
    parent_dic_node: Option<&DicNode>,
    dic_node: &DicNode,
) -> f32 {
    let parent = parent_or_self(parent_dic_node, dic_node);
    match correction_type {
        CorrectionType::Omission => omission_cost(parent, dic_node),
        // TODO (C++): Quit calling getMatchedCost().
        CorrectionType::AdditionalProximity => {
            params::ADDITIONAL_PROXIMITY_COST + matched_cost(v, dic_node)
        }
        CorrectionType::Substitution => params::SUBSTITUTION_COST + matched_cost(v, dic_node),
        CorrectionType::NewWordSpaceOmission => space_omission_cost(v),
        CorrectionType::Match => matched_cost(v, dic_node),
        CorrectionType::Completion => completion_cost(v, dic_node),
        CorrectionType::Terminal => terminal_spatial_cost(dic_node),
        CorrectionType::TerminalInsertion => terminal_insertion_cost(v, dic_node),
        CorrectionType::NewWordSpaceSubstitution => space_substitution_cost(v, dic_node),
        CorrectionType::Insertion => insertion_cost(v, parent, dic_node),
        CorrectionType::Transposition => transposition_cost(v, parent, dic_node),
        CorrectionType::Proximity => 0.0,
    }
}

/// `Weighting::getLanguageCost`.
fn language_cost(
    v: &SessionView<'_>,
    correction_type: CorrectionType,
    parent_dic_node: Option<&DicNode>,
    dic_node: &DicNode,
    multi_bigram_map: Option<&mut MultiBigramMap>,
) -> f32 {
    let parent = parent_or_self(parent_dic_node, dic_node);
    match correction_type {
        CorrectionType::NewWordSpaceOmission | CorrectionType::NewWordSpaceSubstitution => {
            new_word_bigram_language_cost(v, parent, multi_bigram_map)
        }
        CorrectionType::Terminal => {
            let language_improbability =
                get_bigram_node_improbability(v.policy, dic_node, multi_bigram_map);
            terminal_language_cost(language_improbability)
        }
        _ => 0.0,
    }
}

/// `Weighting::getForwardInputCount`.
fn forward_input_count(correction_type: CorrectionType) -> i32 {
    match correction_type {
        CorrectionType::Omission => 0,
        CorrectionType::AdditionalProximity => 1,
        CorrectionType::Substitution => 1,
        CorrectionType::NewWordSpaceOmission => 0,
        CorrectionType::Match => 1,
        CorrectionType::Completion => 1,
        CorrectionType::Terminal => 0,
        CorrectionType::TerminalInsertion => 1,
        CorrectionType::NewWordSpaceSubstitution => 1,
        CorrectionType::Insertion => 2, // look ahead + skip the current char
        CorrectionType::Transposition => 2, // look ahead + skip the current char
        CorrectionType::Proximity => 0,
    }
}

/// `TypingWeighting::getTerminalSpatialCost`.
fn terminal_spatial_cost(dic_node: &DicNode) -> f32 {
    let mut cost = 0.0_f32;
    if dic_node.has_multiple_words() {
        cost += params::HAS_MULTI_WORD_TERMINAL_COST;
    }
    if dic_node.proximity_correction_count() > 0 {
        cost += params::HAS_PROXIMITY_TERMINAL_COST;
    }
    if dic_node.edit_correction_count() > 0 {
        cost += params::HAS_EDIT_CORRECTION_TERMINAL_COST;
    }
    cost
}

/// `TypingWeighting::getOmissionCost`.
fn omission_cost(parent_dic_node: &DicNode, dic_node: &DicNode) -> f32 {
    let is_zero_cost_omission = parent_dic_node.is_zero_cost_omission();
    let is_intentional_omission = parent_dic_node.can_be_intentional_omission();
    let same_code_point = dic_node.is_same_node_code_point(parent_dic_node);
    // If the traversal omitted the first letter then the dicNode should now be on the second.
    let is_first_letter_omission = dic_node.node_code_point_count() == 2;
    if is_zero_cost_omission {
        0.0
    } else if is_intentional_omission {
        params::INTENTIONAL_OMISSION_COST
    } else if is_first_letter_omission {
        params::OMISSION_COST_FIRST_CHAR
    } else if same_code_point {
        params::OMISSION_COST_SAME_CHAR
    } else {
        params::OMISSION_COST
    }
}

/// `TypingWeighting::getMatchedCost`.
fn matched_cost(v: &SessionView<'_>, dic_node: &DicNode) -> f32 {
    let point_index = i32::from(dic_node.input_index(0));
    let normalized_squared_length = v.state.get_point_to_key_length(
        v.info,
        point_index,
        chars::to_base_lower_case(dic_node.node_code_point()),
    );
    let normalized_distance = get_sweet_spot_factor(
        v.state.touch_position_correction_enabled(),
        normalized_squared_length,
    );
    let weighted_distance = params::DISTANCE_WEIGHT_LENGTH * normalized_distance;

    let is_first_char = point_index == 0;
    let is_proximity = is_proximity_dic_node(v, dic_node);
    let mut cost = if is_proximity {
        if is_first_char {
            params::FIRST_CHAR_PROXIMITY_COST
        } else {
            params::PROXIMITY_COST
        }
    } else {
        0.0
    };
    if is_proximity && dic_node.proximity_correction_count() == 0 {
        cost += params::FIRST_PROXIMITY_COST;
    }
    if dic_node.node_code_point_count() == 2 {
        // At the second character of the current word, a first char that is uppercase in the
        // second or later word of a multiple word suggestion is demoted.
        let is_second_or_later_word_first_char_uppercase =
            dic_node.has_multiple_words() && dic_node.is_first_char_uppercase();
        if is_second_or_later_word_first_char_uppercase {
            cost += params::COST_SECOND_OR_LATER_WORD_FIRST_CHAR_UPPERCASE;
        }
    }
    weighted_distance + cost
}

/// `TypingWeighting::isProximityDicNode`.
fn is_proximity_dic_node(v: &SessionView<'_>, dic_node: &DicNode) -> bool {
    let point_index = i32::from(dic_node.input_index(0));
    let primary_code_point =
        chars::to_base_lower_case(v.state.get_primary_code_point_at(point_index));
    let dic_node_char = chars::to_base_lower_case(dic_node.node_code_point());
    primary_code_point != dic_node_char
}

/// `TypingWeighting::getTranspositionCost`.
fn transposition_cost(v: &SessionView<'_>, parent_dic_node: &DicNode, dic_node: &DicNode) -> f32 {
    let parent_point_index = i32::from(parent_dic_node.input_index(0));
    let prev_code_point = parent_dic_node.node_code_point();
    let distance1 = v.state.get_point_to_key_length(
        v.info,
        parent_point_index + 1,
        chars::to_base_lower_case(prev_code_point),
    );
    let code_point = dic_node.node_code_point();
    let distance2 = v.state.get_point_to_key_length(
        v.info,
        parent_point_index,
        chars::to_base_lower_case(code_point),
    );
    let distance = distance1 + distance2;
    let weighted_length_distance = distance * params::DISTANCE_WEIGHT_LENGTH;
    params::TRANSPOSITION_COST + weighted_length_distance
}

/// `TypingWeighting::getInsertionCost`.
fn insertion_cost(v: &SessionView<'_>, parent_dic_node: &DicNode, dic_node: &DicNode) -> f32 {
    let inserted_point_index = i32::from(parent_dic_node.input_index(0));
    let prev_code_point = v.state.get_primary_code_point_at(inserted_point_index);
    let current_code_point = dic_node.node_code_point();
    let same_code_point = prev_code_point == current_code_point;
    let exists_adjacent_proximity_chars = v
        .state
        .exists_adjacent_proximity_chars(inserted_point_index);
    let dist = v.state.get_point_to_key_length(
        v.info,
        inserted_point_index + 1,
        chars::to_base_lower_case(dic_node.node_code_point()),
    );
    let weighted_distance = dist * params::DISTANCE_WEIGHT_LENGTH;
    let single_char = dic_node.node_code_point_count() == 1;
    let mut cost = if single_char {
        params::INSERTION_COST_FIRST_CHAR
    } else {
        0.0
    };
    if same_code_point {
        cost += params::INSERTION_COST_SAME_CHAR;
    } else if exists_adjacent_proximity_chars {
        cost += params::INSERTION_COST_PROXIMITY_CHAR;
    } else {
        cost += params::INSERTION_COST;
    }
    cost + weighted_distance
}

/// `TypingWeighting::getSpaceOmissionCost`.
fn space_omission_cost(v: &SessionView<'_>) -> f32 {
    let cost = params::SPACE_OMISSION_COST;
    cost * v.multi_word_cost_multiplier
}

/// `TypingWeighting::getNewWordBigramLanguageCost`.
fn new_word_bigram_language_cost(
    v: &SessionView<'_>,
    dic_node: &DicNode,
    multi_bigram_map: Option<&mut MultiBigramMap>,
) -> f32 {
    get_bigram_node_improbability(v.policy, dic_node, multi_bigram_map)
        * params::DISTANCE_WEIGHT_LANGUAGE
}

/// `TypingWeighting::getCompletionCost`.
fn completion_cost(v: &SessionView<'_>, dic_node: &DicNode) -> f32 {
    // The auto completion starts when the input index is same as the input size
    let first_completion = i32::from(dic_node.input_index(0)) == v.input_size;
    if first_completion {
        params::COST_FIRST_COMPLETION
    } else {
        params::COST_COMPLETION
    }
}

/// `TypingWeighting::getTerminalLanguageCost`.
fn terminal_language_cost(dic_node_language_improbability: f32) -> f32 {
    dic_node_language_improbability * params::DISTANCE_WEIGHT_LANGUAGE
}

/// `TypingWeighting::getTerminalInsertionCost`.
fn terminal_insertion_cost(v: &SessionView<'_>, dic_node: &DicNode) -> f32 {
    let input_index = i32::from(dic_node.input_index(0));
    params::TERMINAL_INSERTION_COST * (v.input_size.wrapping_sub(input_index) as f32)
}

/// `TypingWeighting::getSpaceSubstitutionCost`.
fn space_substitution_cost(v: &SessionView<'_>, dic_node: &DicNode) -> f32 {
    let input_index = i32::from(dic_node.input_index(0));
    let distance_to_space_key = v
        .state
        .get_point_to_key_length(v.info, input_index, KEYCODE_SPACE);
    let cost = params::SPACE_SUBSTITUTION_COST * distance_to_space_key;
    cost * v.multi_word_cost_multiplier
}

/// `TypingWeighting::getErrorType`.
fn error_type(
    v: &SessionView<'_>,
    correction_type: CorrectionType,
    parent_dic_node: Option<&DicNode>,
    dic_node: &DicNode,
) -> ErrorType {
    let parent = parent_or_self(parent_dic_node, dic_node);
    match correction_type {
        CorrectionType::Match => match_error_type(v, dic_node),
        // TODO (C++): Change to EDIT_CORRECTION.
        CorrectionType::AdditionalProximity => error_type::PROXIMITY_CORRECTION,
        CorrectionType::Omission => {
            if parent.can_be_intentional_omission() {
                error_type::INTENTIONAL_OMISSION
            } else {
                error_type::EDIT_CORRECTION
            }
        }
        // TODO (C++): Quit setting PROXIMITY_CORRECTION.
        CorrectionType::Substitution => {
            error_type::EDIT_CORRECTION | error_type::PROXIMITY_CORRECTION
        }
        CorrectionType::Insertion
        | CorrectionType::TerminalInsertion
        | CorrectionType::Transposition => error_type::EDIT_CORRECTION,
        CorrectionType::NewWordSpaceOmission | CorrectionType::NewWordSpaceSubstitution => {
            error_type::NEW_WORD
        }
        CorrectionType::Terminal => error_type::NOT_AN_ERROR,
        CorrectionType::Completion => error_type::COMPLETION,
        CorrectionType::Proximity => error_type::NOT_AN_ERROR,
    }
}

/// The `CT_MATCH` case of `TypingWeighting::getErrorType`: how the node's letter differs from
/// the typed one.
fn match_error_type(v: &SessionView<'_>, dic_node: &DicNode) -> ErrorType {
    if is_proximity_dic_node(v, dic_node) {
        return error_type::PROXIMITY_CORRECTION;
    }
    if dic_node.is_in_digraph() {
        return error_type::MATCH_WITH_DIGRAPH;
    }
    // Compare the node code point with original primary code point on the keyboard.
    let primary_code_point = v
        .state
        .get_primary_code_point_at(i32::from(dic_node.input_index(0)));
    let node_code_point = dic_node.node_code_point();
    let key_index = v.info.get_key_index_of(primary_code_point);
    if primary_code_point == node_code_point {
        // Node code point is same as original code point on the keyboard.
        error_type::NOT_AN_ERROR
    } else if chars::to_lower_case(primary_code_point) == chars::to_lower_case(node_code_point) {
        // Only cases of the code points are different.
        error_type::MATCH_WITH_WRONG_CASE
    } else if primary_code_point == chars::to_base_code_point(node_code_point) {
        // Node code point is a variant of original code point.
        error_type::MATCH_WITH_MISSING_ACCENT
    } else if chars::to_base_code_point(primary_code_point)
        == chars::to_base_code_point(node_code_point)
    {
        // Base code points are the same but the code point is intentionally input.
        if key_index == NOT_AN_INDEX {
            return error_type::MATCH_WITH_MISSING_EXPLICIT_ACCENT;
        }
        error_type::MATCH_WITH_WRONG_ACCENT
    } else if chars::to_lower_case(primary_code_point) == chars::to_base_lower_case(node_code_point)
    {
        // Node code point is a variant of original code point and the cases are also different.
        error_type::MATCH_WITH_MISSING_ACCENT | error_type::MATCH_WITH_WRONG_CASE
    } else {
        if key_index == NOT_AN_INDEX {
            return error_type::MATCH_WITH_MISSING_EXPLICIT_ACCENT
                | error_type::MATCH_WITH_WRONG_CASE;
        }
        // Base code points are the same and the cases are different.
        error_type::MATCH_WITH_WRONG_ACCENT | error_type::MATCH_WITH_WRONG_CASE
    }
}
