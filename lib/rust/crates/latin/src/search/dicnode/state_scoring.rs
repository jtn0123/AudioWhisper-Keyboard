//! `suggest/core/dicnode/internal/dic_node_state_scoring`: what a DicNode has cost so far.

use crate::defines::MAX_VALUE_FOR_WEIGHTING;
use crate::search::digraph::DigraphCodePointIndex;
use crate::search::error_type::{self, ErrorType};

/// `DicNodeStateScoring`. Every distance is an `f32`, added in the C++'s order.
/// `mDoubleLetterLevel` and `mRawLength` are not ported: only the gesture weighting sets or reads
/// them.
#[derive(Clone, Copy, Debug)]
pub struct DicNodeStateScoring {
    /// `mDigraphIndex`.
    digraph_index: DigraphCodePointIndex,
    /// `mEditCorrectionCount`.
    edit_correction_count: i16,
    /// `mProximityCorrectionCount`.
    proximity_correction_count: i16,
    /// `mCompletionCount`.
    completion_count: i16,
    /// `mNormalizedCompoundDistance`.
    normalized_compound_distance: f32,
    /// `mSpatialDistance`.
    spatial_distance: f32,
    /// `mLanguageDistance`.
    language_distance: f32,
    /// `mContainedErrorTypes`.
    contained_error_types: ErrorType,
    /// `mNormalizedCompoundDistanceAfterFirstWord`.
    normalized_compound_distance_after_first_word: f32,
}

impl Default for DicNodeStateScoring {
    /// `DicNodeStateScoring()`.
    fn default() -> Self {
        Self {
            digraph_index: DigraphCodePointIndex::NotADigraphIndex,
            edit_correction_count: 0,
            proximity_correction_count: 0,
            completion_count: 0,
            normalized_compound_distance: 0.0,
            spatial_distance: 0.0,
            language_distance: 0.0,
            contained_error_types: error_type::NOT_AN_ERROR,
            normalized_compound_distance_after_first_word: MAX_VALUE_FOR_WEIGHTING as f32,
        }
    }
}

impl DicNodeStateScoring {
    /// `DicNodeStateScoring::init`.
    pub fn init(&mut self) {
        *self = Self::default();
    }

    /// `DicNodeStateScoring::initByCopy`.
    pub fn init_by_copy(&mut self, scoring: &Self) {
        *self = *scoring;
    }

    /// `DicNodeStateScoring::addCost`, with `doNormalization` false: the typing weighting's
    /// `needsToNormalizeCompoundDistance` is false, so `inputSize` and `totalInputIndex` are
    /// unused.
    pub fn add_cost(&mut self, spatial_cost: f32, language_cost: f32, error_type: ErrorType) {
        self.add_distance(spatial_cost, language_cost);
        self.contained_error_types |= error_type;
        if error_type::is_edit_correction_error(error_type) {
            self.edit_correction_count = self.edit_correction_count.wrapping_add(1);
        }
        if error_type::is_proximity_correction_error(error_type) {
            self.proximity_correction_count = self.proximity_correction_count.wrapping_add(1);
        }
        if error_type::is_completion(error_type) {
            self.completion_count = self.completion_count.wrapping_add(1);
        }
    }

    /// `DicNodeStateScoring::saveNormalizedCompoundDistanceAfterFirstWordIfNoneYet`.
    pub fn save_normalized_compound_distance_after_first_word_if_none_yet(&mut self) {
        if self.normalized_compound_distance_after_first_word >= MAX_VALUE_FOR_WEIGHTING as f32 {
            self.normalized_compound_distance_after_first_word = self.normalized_compound_distance;
        }
    }

    /// `DicNodeStateScoring::getCompoundDistance(weightOfLangModelVsSpatialModel)`:
    /// `mSpatialDistance + mLanguageDistance * weight`. clang's default `-ffp-contract=on` fuses
    /// the product and the sum of one expression on arm64, host and Android alike, so this is
    /// one `mul_add`, as in `touch_position_correction_utils`. With weight 1.0 the product is
    /// exact and the fusing changes nothing.
    pub fn compound_distance(&self, weight_of_lang_model_vs_spatial_model: f32) -> f32 {
        self.language_distance
            .mul_add(weight_of_lang_model_vs_spatial_model, self.spatial_distance)
    }

    /// `getNormalizedCompoundDistance`.
    pub fn normalized_compound_distance(&self) -> f32 {
        self.normalized_compound_distance
    }

    /// `getNormalizedCompoundDistanceAfterFirstWord`.
    pub fn normalized_compound_distance_after_first_word(&self) -> f32 {
        self.normalized_compound_distance_after_first_word
    }

    /// `getSpatialDistance`.
    pub fn spatial_distance(&self) -> f32 {
        self.spatial_distance
    }

    /// `getEditCorrectionCount`.
    pub fn edit_correction_count(&self) -> i16 {
        self.edit_correction_count
    }

    /// `getProximityCorrectionCount`.
    pub fn proximity_correction_count(&self) -> i16 {
        self.proximity_correction_count
    }

    /// `getCompletionCount`.
    pub fn completion_count(&self) -> i16 {
        self.completion_count
    }

    /// `getDigraphIndex`.
    pub fn digraph_index(&self) -> DigraphCodePointIndex {
        self.digraph_index
    }

    /// `advanceDigraphIndex`.
    pub fn advance_digraph_index(&mut self) {
        self.digraph_index = match self.digraph_index {
            DigraphCodePointIndex::NotADigraphIndex => DigraphCodePointIndex::FirstDigraphCodepoint,
            DigraphCodePointIndex::FirstDigraphCodepoint => {
                DigraphCodePointIndex::SecondDigraphCodepoint
            }
            DigraphCodePointIndex::SecondDigraphCodepoint => {
                DigraphCodePointIndex::NotADigraphIndex
            }
        };
    }

    /// `getContainedErrorTypes`.
    pub fn contained_error_types(&self) -> ErrorType {
        self.contained_error_types
    }

    /// `DicNodeStateScoring::addDistance` without normalization (see [`Self::add_cost`]).
    fn add_distance(&mut self, spatial_distance: f32, language_distance: f32) {
        self.spatial_distance += spatial_distance;
        self.language_distance += language_distance;
        self.normalized_compound_distance = self.spatial_distance + self.language_distance;
    }
}
