//! `suggest/policyimpl/typing/typing_traversal`: when the typing search may correct, cache and
//! start a new word. The C++ reaches it through the virtual `Traversal`; typing is the only
//! implementation that can be reached (`isGesture` is 0), so these are plain functions.

use crate::chars;
use crate::defines::{MAX_RESULTS, ProximityType};
use crate::layout::proximity_info_utils::is_match_or_proximity_char;
use crate::search::dicnode::cache::DicNodesCache;
use crate::search::dicnode::node::DicNode;
use crate::search::scoring_params as params;
use crate::search::session::SessionView;

/// `TypingTraversal::CORRECT_OMISSION`.
const CORRECT_OMISSION: bool = true;
/// `TypingTraversal::CORRECT_NEW_WORD_SPACE_SUBSTITUTION`.
const CORRECT_NEW_WORD_SPACE_SUBSTITUTION: bool = true;
/// `TypingTraversal::CORRECT_NEW_WORD_SPACE_OMISSION`.
const CORRECT_NEW_WORD_SPACE_OMISSION: bool = true;

/// `TypingTraversal::allowsErrorCorrections`.
pub fn allows_error_corrections(dic_node: &DicNode) -> bool {
    dic_node.normalized_spatial_distance() < params::NORMALIZED_SPATIAL_DISTANCE_THRESHOLD_FOR_EDIT
}

/// `TypingTraversal::isOmission`.
pub fn is_omission(
    v: &SessionView<'_>,
    dic_node: &DicNode,
    child_dic_node: &DicNode,
    allows_error_corrections: bool,
) -> bool {
    if !CORRECT_OMISSION {
        return false;
    }
    // Note: Always consider intentional omissions (like apostrophes) since they are common.
    let can_consider_omission =
        allows_error_corrections || child_dic_node.can_be_intentional_omission();
    if !can_consider_omission {
        return false;
    }
    // TODO: Don't refer to isCompletion?
    if dic_node.is_completion(v.input_size) {
        return false;
    }
    if dic_node.can_be_intentional_omission() {
        return true;
    }
    let point0_index = i32::from(dic_node.input_index(0));
    let current_base_lower_code_point = chars::to_base_lower_case(child_dic_node.node_code_point());
    let typed_base_lower_code_point =
        chars::to_base_lower_case(v.state.get_primary_code_point_at(point0_index));
    current_base_lower_code_point != typed_base_lower_code_point
}

/// `TypingTraversal::isSpaceSubstitutionTerminal`.
pub fn is_space_substitution_terminal(v: &SessionView<'_>, dic_node: &DicNode) -> bool {
    if !CORRECT_NEW_WORD_SPACE_SUBSTITUTION {
        return false;
    }
    if v.options.weight_for_locale() < params::LOCALE_WEIGHT_THRESHOLD_FOR_SPACE_SUBSTITUTION {
        // Space substitution is heavy, so it is skipped when the weight for this language is low.
        return false;
    }
    if !can_do_look_ahead_correction(v, dic_node) {
        return false;
    }
    let point0_index = i32::from(dic_node.input_index(0));
    dic_node.is_terminal_dic_node() && v.state.has_space_proximity(v.info, point0_index)
}

/// `TypingTraversal::isSpaceOmissionTerminal`.
pub fn is_space_omission_terminal(v: &SessionView<'_>, dic_node: &DicNode) -> bool {
    if !CORRECT_NEW_WORD_SPACE_OMISSION {
        return false;
    }
    if v.options.weight_for_locale() < params::LOCALE_WEIGHT_THRESHOLD_FOR_SPACE_OMISSION {
        return false;
    }
    // TODO: Don't refer to isCompletion?
    if dic_node.is_completion(v.input_size) {
        return false;
    }
    if !dic_node.is_terminal_dic_node() {
        return false;
    }
    let point_index = i32::from(dic_node.input_index(0));
    point_index <= v.input_size
        && !dic_node.is_total_input_size_exceeding_limit()
        && !dic_node.should_be_filtered_by_safety_net_for_bigram()
}

/// `TypingTraversal::shouldDepthLevelCache`.
pub fn should_depth_level_cache(v: &SessionView<'_>, cache: &DicNodesCache) -> bool {
    cache.is_cache_border_for_typing(v.input_size)
}

/// `TypingTraversal::shouldNodeLevelCache`: never.
pub fn should_node_level_cache() -> bool {
    false
}

/// `TypingTraversal::canDoLookAheadCorrection`.
pub fn can_do_look_ahead_correction(v: &SessionView<'_>, dic_node: &DicNode) -> bool {
    dic_node.can_do_look_ahead_correction(v.input_size)
}

/// `TypingTraversal::getProximityType`.
pub fn get_proximity_type(
    v: &SessionView<'_>,
    dic_node: &DicNode,
    child_dic_node: &DicNode,
) -> ProximityType {
    v.state.get_proximity_type(
        i32::from(dic_node.input_index(0)),
        child_dic_node.node_code_point(),
        true,
        None,
    )
}

/// `TypingTraversal::getMaxSpatialDistance`.
pub fn max_spatial_distance() -> f32 {
    params::MAX_SPATIAL_DISTANCE
}

/// `TypingTraversal::getMaxCacheSize`.
pub fn max_cache_size(input_size: i32, weight_for_locale: f32) -> i32 {
    if input_size <= 1 {
        return params::MAX_CACHE_DIC_NODE_SIZE_FOR_SINGLE_POINT;
    }
    if weight_for_locale < params::LOCALE_WEIGHT_THRESHOLD_FOR_SMALL_CACHE_SIZE {
        return params::MAX_CACHE_DIC_NODE_SIZE_FOR_LOW_PROBABILITY_LOCALE;
    }
    params::MAX_CACHE_DIC_NODE_SIZE
}

/// `TypingTraversal::getTerminalCacheSize`.
pub fn terminal_cache_size() -> i32 {
    MAX_RESULTS as i32
}

/// `TypingTraversal::isPossibleOmissionChildNode`.
pub fn is_possible_omission_child_node(
    v: &SessionView<'_>,
    parent_dic_node: &DicNode,
    dic_node: &DicNode,
) -> bool {
    let proximity_type = get_proximity_type(v, parent_dic_node, dic_node);
    is_match_or_proximity_char(proximity_type)
}

/// `TypingTraversal::isGoodToTraverseNextWord`.
pub fn is_good_to_traverse_next_word(dic_node: &DicNode, probability: i32) -> bool {
    if probability < params::THRESHOLD_NEXT_WORD_PROBABILITY {
        return false;
    }
    let short_capped_word = i32::from(dic_node.node_code_point_count())
        < params::THRESHOLD_SHORT_WORD_LENGTH
        && dic_node.is_first_char_uppercase();
    !short_capped_word || probability >= params::THRESHOLD_NEXT_WORD_PROBABILITY_FOR_CAPPED
}
