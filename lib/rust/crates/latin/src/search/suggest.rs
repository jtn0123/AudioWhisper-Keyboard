//! `suggest/core/suggest`: the search. [`get_suggestions`] (`Dictionary::getSuggestions` and
//! `Suggest::getSuggestions`) starts it at the trie's root, or where the last call left off, and
//! expands the active nodes one input point at a time until none are left; the expansions (match,
//! omission, insertion, transposition, digraph, space) are in `suggest_process`.
//!
//! The C++ `Suggest` is built over a `SuggestPolicy`, typing or gesture; typing is the only one
//! reachable here, so the search calls the typing traversal, weighting and scoring directly. The
//! C++'s profiler timers and logging are not ported.

use crate::defines::ProximityType;
use crate::layout::proximity_info::ProximityInfo;
use crate::layout::proximity_info_state::TypingInput;
use crate::search::dicnode::cache::DicNodesCache;
use crate::search::dicnode::node::DicNode;
use crate::search::dicnode::utils::{get_all_child_dic_nodes, init_as_root};
use crate::search::digraph;
use crate::search::result::output_utils::output_suggestions;
use crate::search::session::{self, DicTraverseSession, Scratch, SessionView};
use crate::search::suggest_options::SuggestOptions;
use crate::search::typing::traversal;
use crate::trie::dictionary::Dictionary;
use crate::trie::multi_bigram::MultiBigramMap;
use crate::trie::ngram::NgramContext;

/// `Suggest::MIN_CONTINUOUS_SUGGESTION_INPUT_SIZE`.
const MIN_CONTINUOUS_SUGGESTION_INPUT_SIZE: i32 = 2;

/// The search of one call: the session's read-only view, and the parts of the session it changes.
pub(super) struct Search<'a> {
    /// What the search reads.
    pub(super) v: SessionView<'a>,
    /// The session's `mDicNodesCache`.
    pub(super) cache: &'a mut DicNodesCache,
    /// The session's `mMultiBigramMap`.
    pub(super) multi_bigram_map: &'a mut MultiBigramMap,
    /// The session's reused storage, moved out for the call.
    pub(super) scratch: Scratch,
}

/// `Dictionary::getSuggestions` and `Suggest::getSuggestions` for typing: the suggestions for
/// the taps `input` into `session.results` (which `jni` writes out), after the words before them
/// in `ngram_context`. `weight_of_lang_model_vs_spatial_model` is
/// NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL (-1) for the app, which has the scoring choose.
pub fn get_suggestions(
    dictionary: &Dictionary,
    info: &ProximityInfo,
    session: &mut DicTraverseSession,
    input: &TypingInput<'_>,
    ngram_context: &NgramContext,
    options: SuggestOptions<'_>,
    weight_of_lang_model_vs_spatial_model: f32,
) {
    session.init(dictionary, ngram_context);
    session.setup_for_get_suggestions(dictionary.header(), info, input);
    session.results.clear();
    let DicTraverseSession {
        prev_word_id_array,
        prev_word_id_count,
        dic_nodes_cache,
        multi_bigram_map,
        proximity_info_state,
        input_size,
        multi_word_cost_multiplier,
        scratch,
        results,
    } = session;
    let mut search = Search {
        v: SessionView {
            policy: dictionary.policy(),
            header: dictionary.header(),
            info,
            state: proximity_info_state,
            options,
            input_size: *input_size,
            multi_word_cost_multiplier: *multi_word_cost_multiplier,
            prev_word_ids: &prev_word_id_array[..*prev_word_id_count],
        },
        cache: dic_nodes_cache,
        multi_bigram_map,
        scratch: std::mem::take(scratch),
    };
    // The C++ passes the caller's `inputSize` to `advanceInputIndex`, not the session's.
    let given_input_size = input.size() as i32;
    search.initialize_search();
    // keep expanding search dicNodes until all have terminated.
    while search.cache.active_size() > 0 {
        search.expand_current_dic_nodes();
        search.cache.advance_active_dic_nodes();
        search.cache.advance_input_index(given_input_size);
    }
    output_suggestions(
        &search.v,
        search.cache,
        &mut search.scratch.terminals,
        weight_of_lang_model_vs_spatial_model,
        results,
    );
    *scratch = search.scratch;
}

impl Search<'_> {
    /// `Suggest::initializeSearch`: starts the search at the root of the trie, or continues from
    /// where the last call left off.
    fn initialize_search(&mut self) {
        if !self.v.state.is_used() {
            return;
        }
        if self.v.input_size > MIN_CONTINUOUS_SUGGESTION_INPUT_SIZE
            && session::is_continuous_suggestion_possible(self.cache, self.v.state)
        {
            // Continue suggestion
            self.cache.continue_search();
        } else {
            // Restart recognition at the root.
            let max_cache_size =
                traversal::max_cache_size(self.v.input_size, self.v.options.weight_for_locale());
            self.cache
                .reset(max_cache_size, traversal::terminal_cache_size());
            self.multi_bigram_map.clear();
            // Create a new dic node here
            let mut root_node = DicNode::default();
            init_as_root(self.v.policy, self.v.prev_word_ids, &mut root_node);
            self.cache.copy_push_active(&root_node);
        }
    }

    /// `Suggest::expandCurrentDicNodes`: advances every active node to its children, by the next
    /// touch point or by none (look-ahead).
    fn expand_current_dic_nodes(&mut self) {
        let input_size = self.v.input_size;
        let mut child_dic_nodes = std::mem::take(&mut self.scratch.children);
        let mut correction_dic_node = DicNode::default();

        // TODO (C++): Find more efficient caching
        let should_depth_level_cache = traversal::should_depth_level_cache(&self.v, self.cache);
        if should_depth_level_cache {
            self.cache.update_last_cached_input_index();
        }
        while self.cache.active_size() > 0 {
            let mut dic_node = DicNode::default();
            self.cache.pop_active(&mut dic_node);
            if dic_node.is_total_input_size_exceeding_limit() {
                break;
            }
            child_dic_nodes.clear();
            let point0_index = i32::from(dic_node.input_index(0));
            let can_do_look_ahead_correction =
                traversal::can_do_look_ahead_correction(&self.v, &dic_node);
            let is_look_ahead_correction = can_do_look_ahead_correction
                && self
                    .cache
                    .is_look_ahead_correction_input_index(point0_index);
            let is_completion = dic_node.is_completion(input_size);

            let should_node_level_cache = traversal::should_node_level_cache();
            if should_depth_level_cache || should_node_level_cache {
                self.cache.copy_push_continue(&dic_node);
                dic_node.set_cached();
            }

            if dic_node.is_in_digraph() {
                // Finish digraph handling if the node is in the middle of a digraph expansion.
                self.process_dic_node_as_digraph(&mut dic_node);
            } else if is_look_ahead_correction {
                // The algorithm maintains a small set of "deferred" nodes that have not consumed
                // the latest touch point yet, for the look-ahead corrections (insertions such as
                // "thiis" -> "this" must not consume it at all).
                self.process_dic_node_as_transposition(&dic_node);
                self.process_dic_node_as_insertion(&dic_node);
            } else {
                self.expand_without_look_ahead(
                    &dic_node,
                    &mut child_dic_nodes,
                    &mut correction_dic_node,
                    is_completion,
                    can_do_look_ahead_correction,
                );
            }
        }
        self.scratch.children = child_dic_nodes;
    }

    /// The `!isLookAheadCorrection` branch of `expandCurrentDicNodes`.
    fn expand_without_look_ahead(
        &mut self,
        dic_node: &DicNode,
        child_dic_nodes: &mut crate::search::dicnode::vector::DicNodeVector,
        correction_dic_node: &mut DicNode,
        is_completion: bool,
        can_do_look_ahead_correction: bool,
    ) {
        // Only consider typing error corrections if the normalized compound distance is below a
        // spatial distance threshold.
        // TODO (C++): Remove. Do not prune node here.
        let allows_error_corrections = traversal::allows_error_corrections(dic_node);
        // Process for handling space substitution (e.g., hevis => he is)
        if traversal::is_space_substitution_terminal(&self.v, dic_node) {
            self.create_next_word_dic_node(dic_node, true);
        }

        get_all_child_dic_nodes(dic_node, self.v.policy, child_dic_nodes);

        for i in 0..child_dic_nodes.len() {
            let child_dic_node = child_dic_nodes.get_mut(i);
            if is_completion {
                // Handle forward lookahead when the lexicon letter exceeds the input size.
                self.process_dic_node_as_match(child_dic_node);
                continue;
            }
            if digraph::has_digraph_for_code_point(self.v.header, child_dic_node.node_code_point())
            {
                correction_dic_node.init_by_copy(child_dic_node);
                correction_dic_node.advance_digraph_index();
                self.process_dic_node_as_digraph(correction_dic_node);
            }
            if traversal::is_omission(&self.v, dic_node, child_dic_node, allows_error_corrections) {
                // TODO (C++): (Gesture) Change weight between omission and substitution errors
                correction_dic_node.init_by_copy(child_dic_node);
                self.process_dic_node_as_omission(correction_dic_node);
            }
            let proximity_type = traversal::get_proximity_type(&self.v, dic_node, child_dic_node);
            match proximity_type {
                ProximityType::MatchChar | ProximityType::ProximityChar => {
                    self.process_dic_node_as_match(child_dic_node);
                }
                ProximityType::AdditionalProximityChar => {
                    if allows_error_corrections {
                        self.process_dic_node_as_additional_proximity_char(
                            dic_node,
                            child_dic_node,
                        );
                    }
                }
                ProximityType::SubstitutionChar => {
                    if allows_error_corrections {
                        self.process_dic_node_as_substitution(dic_node, child_dic_node);
                    }
                }
                // Just drop this dicNode and do nothing.
                ProximityType::UnrelatedChar => {}
            }
        }

        // Push the dicNode for look-ahead correction
        if allows_error_corrections && can_do_look_ahead_correction {
            self.cache.copy_push_next_active(dic_node);
        }
    }
}
