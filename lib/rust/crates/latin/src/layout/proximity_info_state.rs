//! `ProximityInfoState` (`suggest/core/layout/proximity_info_state.{h,cpp}`) for typing: the taps
//! of one decode, each with the letters near it and its distance to every key.
//!
//! The C++ keeps a pointer to its `ProximityInfo`; here the methods that need the keyboard take
//! it as an argument, and it must be the one [`ProximityInfoState::init_input_params`] was given.
//! Typing has one pointer and is never a gesture, so `pointerId` is 0 and `isGeometric` false
//! throughout, and `mHasBeenUpdatedByGeometricInput` is always false (it is not kept).
//!
//! Skipped, because only gesture input reaches them (the app's `isGesture` is 0, so
//! `isGeometric` is false): `getProximityTypeG`, `isKeyInSerchKeysAfterIndex`, both
//! `getDirection`s, `getMostProbableString`, `getProbability`, `getSpeedRate`,
//! `getBeelineSpeedPercentile`, `getDoubleLetterLevel`, and the members behind them
//! (`mSpeedRates`, `mDirections`, `mCharProbabilities`, `mSampledSearchKeySets`, ...).
//!
//! Skipped because nothing in typing reads them: `getLengthCache` and the length cache,
//! `mPrimaryInputWord`, `mAverageSpeed`, and the copies of the keyboard's size the C++ stores
//! (`mHasTouchPositionCorrectionData`, `mMostCommonKeyWidthSquare`, `mKeyCount`, `mCellHeight`,
//! `mCellWidth`, `mGridHeight` and `mGridWidth`, the last two swapped there).

use crate::chars;
use crate::defines::{
    ADDITIONAL_PROXIMITY_CHAR_DELIMITER_CODE, MAX_PROXIMITY_CHARS_SIZE, MAX_VALUE_FOR_WEIGHTING,
    MAX_WORD_LENGTH, NOT_A_COORDINATE, NOT_A_TIMESTAMP, NOT_AN_INDEX, ProximityType,
};

use super::proximity_info::ProximityInfo;
use super::proximity_info_state_utils::{
    SampledPoints, check_and_return_is_continuous_suggestion_possible,
    get_point_to_key_by_id_length, init_geometric_distance_infos, trim_last_two_touch_points,
    update_touch_points,
};
use super::proximity_info_utils::initialize_proximities;

/// The row `getProximityCodePointsAt` reads outside `mInputProximities`: the C++ reads whatever
/// is next to the array; this is the zeros it holds past the input.
static NO_PROXIMITY_CODE_POINTS: [i32; MAX_PROXIMITY_CHARS_SIZE] = [0; MAX_PROXIMITY_CHARS_SIZE];

/// What `initInputParams` is given about the taps: their code points and where they were, as the
/// arrays `BinaryDictionary.getSuggestions` passes, each as long as the input or longer.
#[derive(Clone, Copy, Debug)]
pub struct TypingInput<'a> {
    pub codes: &'a [i32],
    pub xs: &'a [i32],
    pub ys: &'a [i32],
    /// `times`, or `None` for the C++'s null (then no point has a time, and they are not
    /// compared to the last call's).
    pub times: Option<&'a [i32]>,
    /// `pointerIds`, or `None` for the C++'s null (every input is pointer 0).
    pub pointer_ids: Option<&'a [i32]>,
}

impl TypingInput<'_> {
    /// `inputSize`: the code points given, at most `MAX_WORD_LENGTH` (the C++ overruns its arrays
    /// beyond that).
    pub fn size(&self) -> usize {
        self.codes.len().min(MAX_WORD_LENGTH)
    }

    /// `xCoordinates[i]`; `NOT_A_COORDINATE` past the array.
    pub fn x(&self, i: usize) -> i32 {
        self.xs.get(i).copied().unwrap_or(NOT_A_COORDINATE)
    }

    /// `yCoordinates[i]`; `NOT_A_COORDINATE` past the array.
    pub fn y(&self, i: usize) -> i32 {
        self.ys.get(i).copied().unwrap_or(NOT_A_COORDINATE)
    }

    /// `times ? times[i] : -1`; `NOT_A_TIMESTAMP` past the array.
    pub fn time(&self, i: usize) -> i32 {
        self.times
            .and_then(|it| it.get(i).copied())
            .unwrap_or(NOT_A_TIMESTAMP)
    }

    /// `pointerIds ? pointerIds[i] : 0`; 0 past the array.
    pub fn pointer_id(&self, i: usize) -> i32 {
        self.pointer_ids
            .and_then(|it| it.get(i).copied())
            .unwrap_or(0)
    }
}

/// `ProximityInfoState`.
#[derive(Clone, Debug)]
pub struct ProximityInfoState {
    max_point_to_key_length: f32,
    is_continuous_suggestion_possible: bool,
    sampled: SampledPoints,
    /// `mSampledNormalizedSquaredLengthCache`: `size` points by the keys.
    sampled_normalized_squared_length_cache: Vec<f32>,
    /// `mInputProximities`: `MAX_PROXIMITY_CHARS_SIZE` code points for each input.
    input_proximities: [i32; MAX_PROXIMITY_CHARS_SIZE * MAX_WORD_LENGTH],
    sampled_input_size: usize,
}

impl Default for ProximityInfoState {
    fn default() -> Self {
        Self::new()
    }
}

impl ProximityInfoState {
    /// The `ProximityInfoState` constructor: no input.
    pub fn new() -> Self {
        Self {
            max_point_to_key_length: 0.0,
            is_continuous_suggestion_possible: false,
            sampled: SampledPoints::default(),
            sampled_normalized_squared_length_cache: Vec::new(),
            input_proximities: [0; MAX_PROXIMITY_CHARS_SIZE * MAX_WORD_LENGTH],
            sampled_input_size: 0,
        }
    }

    /// `ProximityInfoState::initInputParams` for typing: takes the new input (`locale` is the
    /// dictionary's, as code points). The points kept from the last call are reused when it was
    /// the start of this one (`isContinuousSuggestionPossible`); the result is the same either
    /// way.
    pub fn init_input_params(
        &mut self,
        max_point_to_key_length: f32,
        info: &ProximityInfo,
        input: &TypingInput<'_>,
        locale: &[i32],
    ) {
        let input_size = input.size();
        self.is_continuous_suggestion_possible = check_and_return_is_continuous_suggestion_possible(
            input,
            input_size,
            self.sampled_input_size,
            &self.sampled,
        );
        self.input_proximities.fill(0);
        initialize_proximities(info, input, input_size, locale, &mut self.input_proximities);

        // Setup touch points
        self.max_point_to_key_length = max_point_to_key_length;
        self.sampled_input_size = 0;
        let mut push_touch_point_start_index = 0;
        let mut last_saved_input_size = 0;
        let reuse = self.is_continuous_suggestion_possible && self.sampled.indice.len() > 1;
        match reuse
            .then(|| trim_last_two_touch_points(&mut self.sampled))
            .flatten()
        {
            Some(next_start_index) => {
                // Previous two points are never skipped. Thus, we pop 2 input point data here.
                push_touch_point_start_index = usize::try_from(next_start_index).unwrap_or(0);
                last_saved_input_size = self.sampled.len();
            }
            None => {
                // Clear all data.
                self.sampled.clear();
                self.sampled_normalized_squared_length_cache.clear();
            }
        }
        self.sampled_input_size = update_touch_points(
            info,
            &self.input_proximities,
            input,
            input_size,
            push_touch_point_start_index,
            &mut self.sampled,
        );
        if self.sampled_input_size > 0 {
            init_geometric_distance_infos(
                info,
                self.sampled_input_size,
                last_saved_input_size,
                &self.sampled,
                &mut self.sampled_normalized_squared_length_cache,
            );
        }
    }

    /// `ProximityInfoState::getProximityCodePointsAt` (private in the C++): the typed code point
    /// and the ones near it at `index`, then `NOT_A_CODE_POINT` padding. All 0 for an index past
    /// the input or outside `MAX_WORD_LENGTH`.
    pub fn proximity_code_points_at(&self, index: i32) -> &[i32] {
        usize::try_from(index)
            .ok()
            .filter(|&it| it < MAX_WORD_LENGTH)
            .map_or(&NO_PROXIMITY_CODE_POINTS[..], |it| {
                &self.input_proximities
                    [it * MAX_PROXIMITY_CHARS_SIZE..(it + 1) * MAX_PROXIMITY_CHARS_SIZE]
            })
    }

    /// `ProximityInfoState::getPrimaryCodePointAt`: the code point typed at `index`.
    pub fn get_primary_code_point_at(&self, index: i32) -> i32 {
        self.proximity_code_points_at(index)[0]
    }

    /// `ProximityInfoState::getPrimaryOriginalCodePointAt`: the code point the key for what was
    /// typed at `index` was given, `NOT_A_CODE_POINT` if no key types it.
    pub fn get_primary_original_code_point_at(&self, info: &ProximityInfo, index: i32) -> i32 {
        let primary_code_point = self.get_primary_code_point_at(index);
        let key_index = info.get_key_index_of(primary_code_point);
        info.get_original_code_point_of(key_index)
    }

    /// `ProximityInfoState::sameAsTyped`: whether `word` is exactly what was typed.
    pub fn same_as_typed(&self, word: &[i32]) -> bool {
        if word.len() != self.sampled_input_size {
            return false;
        }
        let typed = self
            .input_proximities
            .iter()
            .step_by(MAX_PROXIMITY_CHARS_SIZE);
        typed.zip(word).all(|(typed, word)| typed == word)
    }

    /// `ProximityInfoState::existsCodePointInProximityAt`: whether `c` is the typed code point or
    /// one listed near it at `index` (before the delimiter and padding).
    pub fn exists_code_point_in_proximity_at(&self, index: i32, c: i32) -> bool {
        self.proximity_code_points_at(index)
            .iter()
            .take_while(|&&it| it > 0)
            .any(|&it| it == c)
    }

    /// `ProximityInfoState::existsAdjacentProximityChars`: whether the code point typed at
    /// `index` is near the one typed just before or just after it.
    pub fn exists_adjacent_proximity_chars(&self, index: i32) -> bool {
        if index < 0 || index as usize >= self.sampled_input_size {
            return false;
        }
        let current_code_point = self.get_primary_code_point_at(index);
        let left_index = index - 1;
        if left_index >= 0 && self.exists_code_point_in_proximity_at(left_index, current_code_point)
        {
            return true;
        }
        let right_index = index + 1;
        (right_index as usize) < self.sampled_input_size
            && self.exists_code_point_in_proximity_at(right_index, current_code_point)
    }

    /// `ProximityInfoState::touchPositionCorrectionEnabled`: always false, the keyboard having no
    /// sweet spots.
    pub fn touch_position_correction_enabled(&self) -> bool {
        false
    }

    /// `ProximityInfoState::isUsed`: whether there is any input.
    pub fn is_used(&self) -> bool {
        self.sampled_input_size > 0
    }

    /// `ProximityInfoState::size`: how many points there are.
    pub fn size(&self) -> i32 {
        self.sampled_input_size as i32
    }

    /// `ProximityInfoState::getInputX`: the x of the point, `NOT_A_COORDINATE` for none.
    pub fn get_input_x(&self, index: i32) -> i32 {
        point(&self.sampled.xs, index).unwrap_or(NOT_A_COORDINATE)
    }

    /// `ProximityInfoState::getInputY`: the y of the point, `NOT_A_COORDINATE` for none.
    pub fn get_input_y(&self, index: i32) -> i32 {
        point(&self.sampled.ys, index).unwrap_or(NOT_A_COORDINATE)
    }

    /// `ProximityInfoState::getInputIndexOfSampledPoint`: which input the point came from,
    /// `NOT_AN_INDEX` for no point.
    pub fn get_input_index_of_sampled_point(&self, sampled_index: i32) -> i32 {
        point(&self.sampled.indice, sampled_index).unwrap_or(NOT_AN_INDEX)
    }

    /// `ProximityInfoState::hasSpaceProximity`: whether the point at `index` is near the space
    /// bar. False for no point.
    pub fn has_space_proximity(&self, info: &ProximityInfo, index: i32) -> bool {
        match (
            point(&self.sampled.xs, index),
            point(&self.sampled.ys, index),
        ) {
            (Some(x), Some(y)) => info.has_space_proximity(x, y),
            _ => false,
        }
    }

    /// `ProximityInfoState::isContinuousSuggestionPossible`.
    pub fn is_continuous_suggestion_possible(&self) -> bool {
        self.is_continuous_suggestion_possible
    }

    /// `ProximityInfoState::getPointToKeyByIdLength`.
    pub fn get_point_to_key_by_id_length(
        &self,
        info: &ProximityInfo,
        input_index: i32,
        key_id: i32,
    ) -> f32 {
        get_point_to_key_by_id_length(
            self.max_point_to_key_length,
            &self.sampled_normalized_squared_length_cache,
            info.get_key_count(),
            input_index,
            key_id,
        )
    }

    /// `ProximityInfoState::getPointToKeyLength`: how far the tap at `input_index` is from the key
    /// for `code_point`, in squared key widths and at most `max_point_to_key_length`; 0 for an
    /// apostrophe or hyphen with no key (which may be left out), else `MAX_VALUE_FOR_WEIGHTING`.
    /// (Also that, for a tap outside the cache, where the C++ reads outside its vector.)
    pub fn get_point_to_key_length(
        &self,
        info: &ProximityInfo,
        input_index: i32,
        code_point: i32,
    ) -> f32 {
        let key_id = info.get_key_index_of(code_point);
        if key_id != NOT_AN_INDEX {
            return self.get_point_to_key_by_id_length(info, input_index, key_id);
        }
        if chars::is_intentional_omission_code_point(code_point) {
            return 0.0;
        }
        // If the char is not a key on the keyboard then return the max length.
        MAX_VALUE_FOR_WEIGHTING as f32
    }

    /// `ProximityInfoState::getProximityType`: how `code_point` relates to what was typed at
    /// `index`: the same letter (`MatchChar`), a near key (`ProximityChar`), a letter the language
    /// adds (`AdditionalProximityChar`), or none of them (`SubstitutionChar`). With
    /// `check_proximity_chars` false only a match counts. `proximity_index` receives where in the
    /// list the code point was found.
    pub fn get_proximity_type(
        &self,
        index: i32,
        code_point: i32,
        check_proximity_chars: bool,
        mut proximity_index: Option<&mut i32>,
    ) -> ProximityType {
        let current_code_points = self.proximity_code_points_at(index);
        let first_code_point = current_code_points[0];
        let base_lower_c = chars::to_base_lower_case(code_point);

        // The first char in the array is what user typed. If it matches right away, that means the
        // user typed that same char for this pos.
        if first_code_point == base_lower_c || first_code_point == code_point {
            return ProximityType::MatchChar;
        }
        if !check_proximity_chars {
            return ProximityType::SubstitutionChar;
        }
        // If the non-accented, lowercased version of that first character matches c, then we have
        // a non-accented version of the accented character the user typed. Treat it as a close
        // char.
        if chars::to_base_lower_case(first_code_point) == base_lower_c {
            return ProximityType::ProximityChar;
        }

        // Not an exact nor an accent-alike match: search the list of close keys
        let matches = |it: i32| it == base_lower_c || it == code_point;
        let mut j = 1;
        while j < MAX_PROXIMITY_CHARS_SIZE
            && current_code_points[j] > ADDITIONAL_PROXIMITY_CHAR_DELIMITER_CODE
        {
            if matches(current_code_points[j]) {
                if let Some(found) = proximity_index.as_deref_mut() {
                    *found = j as i32;
                }
                return ProximityType::ProximityChar;
            }
            j += 1;
        }
        if j < MAX_PROXIMITY_CHARS_SIZE
            && current_code_points[j] == ADDITIONAL_PROXIMITY_CHAR_DELIMITER_CODE
        {
            j += 1;
            while j < MAX_PROXIMITY_CHARS_SIZE
                && current_code_points[j] > ADDITIONAL_PROXIMITY_CHAR_DELIMITER_CODE
            {
                if matches(current_code_points[j]) {
                    if let Some(found) = proximity_index.as_deref_mut() {
                        *found = j as i32;
                    }
                    return ProximityType::AdditionalProximityChar;
                }
                j += 1;
            }
        }
        // Was not included, signal this as a substitution character.
        ProximityType::SubstitutionChar
    }
}

/// `vector[index]`, or `None` outside it (where the C++ reads outside the vector).
fn point(values: &[i32], index: i32) -> Option<i32> {
    usize::try_from(index)
        .ok()
        .and_then(|it| values.get(it).copied())
}
