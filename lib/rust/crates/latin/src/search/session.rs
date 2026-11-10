//! `suggest/core/session/dic_traverse_session`: the state a typing session keeps between calls
//! (the node cache, the bigram map, the taps), and [`SessionView`], the read-only part of it that
//! the search reads while it changes the rest.
//!
//! The C++ session holds pointers to the dictionary, the keyboard and the options, set on each
//! call by `init` and `setupForGetSuggestions`; here they are arguments, gathered in the
//! [`SessionView`] a call builds. `getProximityTypeG`, `isOnlyOnePointerUsed` for more than one
//! pointer and the second `ProximityInfoState` are gesture-only and not ported: typing has one
//! pointer (`MAX_POINTER_COUNT` is 1).

use crate::defines::{MAX_PREV_WORD_COUNT_FOR_N_GRAM, NOT_A_WORD_ID};
use crate::header::HeaderPolicy;
use crate::layout::proximity_info::ProximityInfo;
use crate::layout::proximity_info_state::{ProximityInfoState, TypingInput};
use crate::search::dicnode::cache::DicNodesCache;
use crate::search::dicnode::node::DicNode;
use crate::search::dicnode::vector::DicNodeVector;
use crate::search::result::suggestion_results::SuggestionResults;
use crate::search::suggest_options::SuggestOptions;
use crate::search::typing::traversal;
use crate::trie::dictionary::Dictionary;
use crate::trie::multi_bigram::MultiBigramMap;
use crate::trie::ngram::NgramContext;
use crate::trie::patricia::PatriciaTriePolicy;

/// `DICTIONARY_SIZE_THRESHOLD_TO_USE_LARGE_CACHE_FOR_SUGGESTION`: 256K bytes tells a dictionary of
/// many unigrams (the main one) from a small one (contacts).
const DICTIONARY_SIZE_THRESHOLD_TO_USE_LARGE_CACHE_FOR_SUGGESTION: usize = 256 * 1024;

/// The storage the search reuses between nodes and calls, so that a call allocates nothing once
/// it has run: the C++'s stack `DicNodeVector`s and `std::vector<DicNode> terminals`.
#[derive(Debug, Default)]
pub struct Scratch {
    /// `childDicNodes` of `expandCurrentDicNodes`.
    pub children: DicNodeVector,
    /// `childDicNodes` of `processDicNodeAsOmission`, `Insertion`, and `childDicNodes1` of
    /// `Transposition`.
    pub grand_children: DicNodeVector,
    /// `childDicNodes2` of `processDicNodeAsTransposition`.
    pub second_grand_children: DicNodeVector,
    /// `terminals` of `outputSuggestions`.
    pub terminals: Vec<DicNode>,
}

/// `DicTraverseSession`.
#[derive(Debug)]
pub struct DicTraverseSession {
    /// `mPrevWordIdArray`.
    pub(super) prev_word_id_array: [i32; MAX_PREV_WORD_COUNT_FOR_N_GRAM],
    /// `mPrevWordIdCount`.
    pub(super) prev_word_id_count: usize,
    /// `mDicNodesCache`.
    pub(super) dic_nodes_cache: DicNodesCache,
    /// `mMultiBigramMap`.
    pub(super) multi_bigram_map: MultiBigramMap,
    /// `mProximityInfoStates[0]` (typing has one pointer).
    pub(super) proximity_info_state: ProximityInfoState,
    /// `getInputSize()`.
    pub(super) input_size: i32,
    /// `getMultiWordCostMultiplier()`.
    pub(super) multi_word_cost_multiplier: f32,
    /// Storage the search reuses (see [`Scratch`]).
    pub(super) scratch: Scratch,
    /// The `SuggestionResults` of a call (a local of the JNI method in the C++), kept so that a
    /// call does not allocate it.
    pub(super) results: SuggestionResults,
}

impl DicTraverseSession {
    /// `DicTraverseSession::getSessionInstance(env, localeStr, dictSize)`: a session for a
    /// dictionary of `dictionary_size` bytes. A large dictionary gets the large node cache.
    pub fn new(dictionary_size: usize) -> Self {
        let uses_large_cache =
            dictionary_size >= DICTIONARY_SIZE_THRESHOLD_TO_USE_LARGE_CACHE_FOR_SUGGESTION;
        Self {
            prev_word_id_array: [NOT_A_WORD_ID; MAX_PREV_WORD_COUNT_FOR_N_GRAM],
            prev_word_id_count: 0,
            dic_nodes_cache: DicNodesCache::new(uses_large_cache),
            multi_bigram_map: MultiBigramMap::new(),
            proximity_info_state: ProximityInfoState::new(),
            input_size: 0,
            multi_word_cost_multiplier: 1.0,
            scratch: Scratch {
                children: DicNodeVector::new(),
                grand_children: DicNodeVector::new(),
                second_grand_children: DicNodeVector::new(),
                terminals: Vec::new(),
            },
            results: SuggestionResults::new(crate::defines::MAX_RESULTS),
        }
    }

    /// `DicTraverseSession::init`: takes the dictionary's cost multiplier and the ids of the
    /// previous words, for one call.
    pub(super) fn init(&mut self, dictionary: &Dictionary, ngram_context: &NgramContext) {
        self.multi_word_cost_multiplier = dictionary.header().multi_word_cost_multiplier();
        self.prev_word_id_count = ngram_context
            .prev_word_ids(dictionary.policy(), &mut self.prev_word_id_array, true)
            .len();
    }

    /// `DicTraverseSession::setupForGetSuggestions` and `initializeProximityInfoStates`: reads
    /// the taps of this call. `input` has exactly the taps' arrays.
    pub(super) fn setup_for_get_suggestions(
        &mut self,
        header: &HeaderPolicy,
        info: &ProximityInfo,
        input: &TypingInput<'_>,
    ) {
        self.proximity_info_state.init_input_params(
            traversal::max_spatial_distance(),
            info,
            input,
            header.locale(),
        );
        self.input_size = self.proximity_info_state.size();
    }
}

/// `DicTraverseSession::isContinuousSuggestionPossible`: whether the cache holds nodes of the
/// last call and this call's taps start with that call's. It takes the session's cache and taps
/// rather than the session, which the search holds apart.
pub(super) fn is_continuous_suggestion_possible(
    cache: &DicNodesCache,
    state: &ProximityInfoState,
) -> bool {
    if !cache.has_cached_dic_nodes_for_continuous_suggestion() {
        return false;
    }
    !(state.is_used() && !state.is_continuous_suggestion_possible())
}

/// What a search reads from the session, the dictionary, the keyboard and the options: the C++
/// reads each through the session's getters.
#[derive(Clone, Copy, Debug)]
pub struct SessionView<'a> {
    /// `getDictionaryStructurePolicy()`.
    pub policy: &'a PatriciaTriePolicy,
    /// `getDictionaryStructurePolicy()->getHeaderStructurePolicy()`.
    pub header: &'a HeaderPolicy,
    /// `getProximityInfo()`.
    pub info: &'a ProximityInfo,
    /// `getProximityInfoState(0)`.
    pub state: &'a ProximityInfoState,
    /// `getSuggestOptions()`.
    pub options: SuggestOptions<'a>,
    /// `getInputSize()`.
    pub input_size: i32,
    /// `getMultiWordCostMultiplier()`.
    pub multi_word_cost_multiplier: f32,
    /// `getPrevWordIds()`.
    pub prev_word_ids: &'a [i32],
}
