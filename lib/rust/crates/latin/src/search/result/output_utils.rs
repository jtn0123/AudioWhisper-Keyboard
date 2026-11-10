//! `suggest/core/result/suggestions_output_utils`: turns the search's terminal nodes into
//! suggestions, with their scores, kinds, partial-commit indices and shortcuts.
//!
//! `scoringPolicy->getMostProbableString` is a no-op for typing and is not ported.

use super::suggestion_results::SuggestionResults;
use crate::defines::{
    DISTANCE_WEIGHT_FOR_AUTO_COMMIT, LENGTH_WEIGHT_FOR_AUTO_COMMIT, MAX_WORD_LENGTH,
    NOT_A_FIRST_WORD_CONFIDENCE, NOT_AN_INDEX, S_INT_MAX, S_INT_MIN,
    SPACE_COUNT_WEIGHT_FOR_AUTO_COMMIT,
};
use crate::search::dicnode::cache::DicNodesCache;
use crate::search::dicnode::node::DicNode;
use crate::search::dictionary::{
    KIND_CORRECTION, KIND_FLAG_APPROPRIATE_FOR_AUTOCORRECTION, KIND_FLAG_EXACT_MATCH,
    KIND_FLAG_EXACT_MATCH_WITH_INTENTIONAL_OMISSION, KIND_FLAG_POSSIBLY_OFFENSIVE, KIND_SHORTCUT,
    KIND_WHITELIST,
};
use crate::search::error_type;
use crate::search::session::SessionView;
use crate::search::suggest_options::SuggestOptions;
use crate::search::typing::scoring;
use crate::trie::interface::WordAttributes;
use crate::trie::shortcut::ShortcutIterator;

/// `SuggestionsOutputUtils::MIN_LEN_FOR_MULTI_WORD_AUTOCORRECT`.
const MIN_LEN_FOR_MULTI_WORD_AUTOCORRECT: i32 = 16;

/// `SuggestionsOutputUtils::outputSuggestions`: pops the terminal nodes of `cache` (into
/// `terminals`, best first) and adds a suggestion for each to `out_suggestion_results`.
pub fn output_suggestions(
    v: &SessionView<'_>,
    cache: &mut DicNodesCache,
    terminals: &mut Vec<DicNode>,
    weight_of_lang_model_vs_spatial_model: f32,
    out_suggestion_results: &mut SuggestionResults,
) {
    let terminal_size = usize::try_from(cache.terminal_size()).unwrap_or(0);
    terminals.clear();
    terminals.resize(terminal_size, DicNode::default());
    for index in (0..terminal_size).rev() {
        cache.pop_terminal(&mut terminals[index]);
    }
    // Compute a weight of language model when an invalid weight is passed.
    // NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL (-1) is taken as an invalid value.
    let weight_to_output = if weight_of_lang_model_vs_spatial_model < 0.0 {
        scoring::adjusted_weight_of_lang_model_vs_spatial_model()
    } else {
        weight_of_lang_model_vs_spatial_model
    };
    out_suggestion_results.set_weight_of_lang_model_vs_spatial_model(weight_to_output);
    // Force autocorrection for obvious long multi-word suggestions when the top suggestion is a
    // long multiple words suggestion.
    let force_commit_multi_words = scoring::auto_corrects_to_multi_word_suggestion_if_top()
        && (v.input_size >= MIN_LEN_FOR_MULTI_WORD_AUTOCORRECT
            && terminals.first().is_some_and(DicNode::has_multiple_words));
    // `isOnlyOnePointerUsed(0)`: typing has one pointer.
    let output_second_word_first_letter_input_index = v.state.is_used();
    let boost_exact_matches = v.header.should_boost_exact_matches();

    for terminal_dic_node in terminals.iter() {
        output_suggestions_of_dic_node(
            v,
            terminal_dic_node,
            weight_to_output,
            boost_exact_matches,
            force_commit_multi_words,
            output_second_word_first_letter_input_index,
            out_suggestion_results,
        );
    }
}

/// `SuggestionsOutputUtils::shouldBlockWord`.
pub fn should_block_word(
    suggest_options: &SuggestOptions<'_>,
    terminal_dic_node: &DicNode,
    word_attributes: &WordAttributes,
    is_last_word: bool,
) -> bool {
    let current_word_exact_match =
        error_type::is_exact_match(terminal_dic_node.contained_error_types());
    // When we have to block offensive words, non-exact matched offensive words should not be
    // output.
    let should_block_offensive_words = suggest_options.block_offensive_words();
    let is_blocked_offensive_word =
        should_block_offensive_words && word_attributes.is_possibly_offensive();
    // At the end of a search (`is_last_word`) an exact match is what the user typed, so it goes
    // through. In the middle of a search it must not: correcting "fuckthis" to "fuck this" with
    // offensive words blocked would be a bad idea.
    if is_last_word && current_word_exact_match {
        false
    } else {
        is_blocked_offensive_word
    }
}

/// `SuggestionsOutputUtils::outputSuggestionsOfDicNode`.
fn output_suggestions_of_dic_node(
    v: &SessionView<'_>,
    terminal_dic_node: &DicNode,
    weight_of_lang_model_vs_spatial_model: f32,
    boost_exact_matches: bool,
    force_commit_multi_words: bool,
    output_second_word_first_letter_input_index: bool,
    out_suggestion_results: &mut SuggestionResults,
) {
    let double_letter_cost = scoring::double_letter_demotion_distance_cost();
    let compound_distance = terminal_dic_node
        .compound_distance(weight_of_lang_model_vs_spatial_model)
        + double_letter_cost;
    let word_attributes = v.policy.get_word_attributes_in_context(
        terminal_dic_node.prev_word_ids(),
        terminal_dic_node.word_id(),
        None,
    );
    let contained = terminal_dic_node.contained_error_types();
    let is_exact_match = error_type::is_exact_match(contained);
    let is_exact_match_with_intentional_omission =
        error_type::is_exact_match_with_intentional_omission(contained);
    // TODO (C++): Decide whether the word should be auto-corrected or not here.
    let is_appropriate_for_auto_correction = !error_type::is_missing_explicit_accent(contained);
    let output_type_flags = (if word_attributes.is_possibly_offensive() {
        KIND_FLAG_POSSIBLY_OFFENSIVE
    } else {
        0
    }) | (if is_exact_match && boost_exact_matches {
        KIND_FLAG_EXACT_MATCH
    } else {
        0
    }) | (if is_exact_match_with_intentional_omission {
        KIND_FLAG_EXACT_MATCH_WITH_INTENTIONAL_OMISSION
    } else {
        0
    }) | (if is_appropriate_for_auto_correction {
        KIND_FLAG_APPROPRIATE_FOR_AUTOCORRECTION
    } else {
        0
    });
    // Entries that are blacklisted or do not represent a word should not be output.
    let is_valid_word = !(word_attributes.is_blacklisted() || word_attributes.is_not_a_word());
    let should_block_this_word =
        should_block_word(&v.options, terminal_dic_node, &word_attributes, true);

    // Increase output score of top typing suggestion to ensure autocorrection.
    let final_score = scoring::calculate_final_score(
        compound_distance,
        v.input_size,
        contained,
        force_commit_multi_words && terminal_dic_node.has_multiple_words(),
        boost_exact_matches,
        word_attributes.probability() == 0,
    );

    // Don't output invalid or blocked offensive words. However, we still need to submit their
    // shortcuts if any.
    if is_valid_word && !should_block_this_word {
        let mut code_points = [0; MAX_WORD_LENGTH];
        terminal_dic_node.output_result(&mut code_points);
        let index_to_partial_commit = if output_second_word_first_letter_input_index {
            terminal_dic_node.second_word_first_input_index(v.state)
        } else {
            NOT_AN_INDEX
        };
        let total = usize::from(terminal_dic_node.total_node_code_point_count());
        // `addSuggestion` refuses a word longer than MAX_WORD_LENGTH (and one of no letters).
        if total <= MAX_WORD_LENGTH {
            out_suggestion_results.add_suggestion(
                &code_points[..total],
                final_score,
                KIND_CORRECTION | output_type_flags,
                index_to_partial_commit,
                compute_first_word_confidence(terminal_dic_node),
            );
        }
    }

    // Output shortcuts. Shortcut is not supported for multiple words suggestions.
    if !terminal_dic_node.has_multiple_words() {
        let shortcut_it = v.policy.shortcut_iterator(terminal_dic_node.word_id());
        let same_as_typed = scoring::same_as_typed(v.state, terminal_dic_node);
        output_shortcuts(
            shortcut_it,
            final_score,
            same_as_typed,
            out_suggestion_results,
        );
    }
}

/// `SuggestionsOutputUtils::computeFirstWordConfidence`.
fn compute_first_word_confidence(terminal_dic_node: &DicNode) -> i32 {
    // Get the number of spaces in the first suggestion
    let space_count = terminal_dic_node.total_node_space_count();
    // Get the number of characters in the first suggestion
    let length = i32::from(terminal_dic_node.total_node_code_point_count());
    // Get the distance for the first word of the suggestion
    let distance = terminal_dic_node.normalized_compound_distance_after_first_word();

    // Expected space count is 1 ~ 5
    const MIN_EXPECTED_SPACE_COUNT: i32 = 1;
    const MAX_EXPECTED_SPACE_COUNT: i32 = 5;
    // Expected length is about 4 ~ 30
    const MIN_EXPECTED_LENGTH: i32 = 4;
    const MAX_EXPECTED_LENGTH: i32 = 30;
    // Expected distance is about 0.2 ~ 2.0, but consider 0.0 ~ 2.0
    const MIN_EXPECTED_DISTANCE: f32 = 0.0;
    const MAX_EXPECTED_DISTANCE: f32 = 2.0;

    // We need at least a space.
    if space_count < 1 {
        return NOT_A_FIRST_WORD_CONFIDENCE;
    }

    // The smaller the edit distance, the higher the contribution. Clamp to avoid overflows.
    // `clamp` agrees with the C++'s two comparisons, NaN included (it stays NaN).
    let clamped_distance = distance.clamp(MIN_EXPECTED_DISTANCE, MAX_EXPECTED_DISTANCE);
    // `int * float` is a float product, then a float quotient, truncated to an int.
    let distance_contribution = (DISTANCE_WEIGHT_FOR_AUTO_COMMIT as f32
        * (MAX_EXPECTED_DISTANCE - clamped_distance)
        / (MAX_EXPECTED_DISTANCE - MIN_EXPECTED_DISTANCE)) as i32;
    // The larger the suggestion length, the larger the contribution. Length is between 1 and 48.
    let length_contribution = LENGTH_WEIGHT_FOR_AUTO_COMMIT * (length - MIN_EXPECTED_LENGTH)
        / (MAX_EXPECTED_LENGTH - MIN_EXPECTED_LENGTH);
    // The more spaces, the larger the contribution.
    let space_contribution = SPACE_COUNT_WEIGHT_FOR_AUTO_COMMIT
        * (space_count - MIN_EXPECTED_SPACE_COUNT)
        / (MAX_EXPECTED_SPACE_COUNT - MIN_EXPECTED_SPACE_COUNT);

    distance_contribution
        .wrapping_add(length_contribution)
        .wrapping_add(space_contribution)
}

/// `SuggestionsOutputUtils::outputShortcuts`.
fn output_shortcuts(
    mut shortcut_it: ShortcutIterator<'_>,
    final_score: i32,
    same_as_typed: bool,
    out_suggestion_results: &mut SuggestionResults,
) {
    let mut shortcut_target = [0; MAX_WORD_LENGTH];
    while shortcut_it.has_next_shortcut_target() {
        let entry = shortcut_it.next_shortcut_target(MAX_WORD_LENGTH, &mut shortcut_target);
        let (shortcut_score, kind) = if entry.is_whitelist && same_as_typed {
            (S_INT_MAX, KIND_WHITELIST)
        } else {
            // shortcut entry's score == its base entry's score - 1
            // Protection against int underflow
            (final_score.max(S_INT_MIN + 1) - 1, KIND_SHORTCUT)
        };
        out_suggestion_results.add_suggestion(
            &shortcut_target[..entry.code_point_count.min(MAX_WORD_LENGTH)],
            shortcut_score.max(S_INT_MIN + 1) - 1,
            kind,
            NOT_AN_INDEX,
            NOT_A_FIRST_WORD_CONFIDENCE,
        );
    }
}
