//! `ProximityInfoStateUtils` (`suggest/core/layout/proximity_info_state_utils.{h,cpp}`) for
//! typing: turning the taps into the points the search measures against.
//!
//! Typing never samples (`doSampling` is `isGeometric`, false), so every input is one point.
//! Skipped as gesture only (`isGeometric`), with their helpers: `refreshSpeedRates`,
//! `refreshBeelineSpeedRates`, `calculateBeelineSpeedRate`, `getDirection`, `getPointAngle`,
//! `getPointsAngle`, `updateNearKeysDistances`, `isPrevLocalMin`, `getPointScore`, the sampling
//! half of `pushTouchPoint`, `updateAlignPointProbabilities`, `suppressCharProbabilities`,
//! `updateSampledSearchKeySets`, `getMostProbableString`, `dump`, and
//! `calculateNormalizedSquaredDistance` with `calculateSquaredDistanceFromSweetSpotCenter` (they
//! need sweet spots, which the app gives none; nothing calls them anyway).
//!
//! Skipped because nothing in typing reads what they produce: the sampled length cache (every
//! `popInputData` and `pushTouchPoint` use of `sampledLengthCache`, which is `hypotf`), the angle
//! sum `updateTouchPoints` accumulates (`atan2f`, only read when sampling) and
//! `initPrimaryInputWord` (`mPrimaryInputWord` is never read).

use crate::defines::{
    MAX_PROXIMITY_CHARS_SIZE, MAX_VALUE_FOR_WEIGHTING, NOT_A_COORDINATE, NOT_AN_INDEX,
};

use super::proximity_info::ProximityInfo;
use super::proximity_info_state::TypingInput;

/// `mSampledInputXs`, `mSampledInputYs`, `mSampledTimes` and `mSampledInputIndice`: for each
/// point, where it is, when, and which input it came from. (Without the length cache.)
#[derive(Clone, Debug, Default)]
pub struct SampledPoints {
    pub xs: Vec<i32>,
    pub ys: Vec<i32>,
    pub times: Vec<i32>,
    pub indice: Vec<i32>,
}

impl SampledPoints {
    pub fn len(&self) -> usize {
        self.xs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.xs.is_empty()
    }

    pub fn clear(&mut self) {
        self.xs.clear();
        self.ys.clear();
        self.times.clear();
        self.indice.clear();
    }

    /// `ProximityInfoStateUtils::popInputData`.
    fn pop_input_data(&mut self) {
        self.xs.pop();
        self.ys.pop();
        self.times.pop();
        self.indice.pop();
    }
}

/// `ProximityInfoStateUtils::trimLastTwoTouchPoints`: drops the last two points and returns the
/// input the next one starts from (the second last's), or `None` with fewer than two points
/// (the C++ reads before the vector's start).
pub fn trim_last_two_touch_points(sampled: &mut SampledPoints) -> Option<i32> {
    let next_start_index = *sampled.indice.iter().rev().nth(1)?;
    sampled.pop_input_data();
    sampled.pop_input_data();
    Some(next_start_index)
}

/// `ProximityInfoStateUtils::checkAndReturnIsContinuousSuggestionPossible`: whether the points
/// kept from the last call are the start of this call's input, taps and times.
pub fn check_and_return_is_continuous_suggestion_possible(
    input: &TypingInput<'_>,
    input_size: usize,
    sampled_input_size: usize,
    sampled: &SampledPoints,
) -> bool {
    if input_size < sampled_input_size {
        return false;
    }
    let points = sampled.xs.iter().zip(&sampled.ys).zip(&sampled.indice);
    for (i, ((&sampled_x, &sampled_y), &index)) in points.take(sampled_input_size).enumerate() {
        let Ok(index) = usize::try_from(index) else {
            return false;
        };
        if index >= input_size {
            return false;
        }
        if input.x(index) != sampled_x || input.y(index) != sampled_y {
            return false;
        }
        if input.times.is_some() && input.time(index) != sampled.times[i] {
            return false;
        }
    }
    true
}

/// `ProximityInfoStateUtils::getProximityCodePointsAt`'s row for `index`, within
/// `input_proximities`; `None` outside it.
fn proximity_row(input_proximities: &[i32], index: usize) -> Option<&[i32]> {
    let start = index.checked_mul(MAX_PROXIMITY_CHARS_SIZE)?;
    input_proximities.get(start..start.checked_add(MAX_PROXIMITY_CHARS_SIZE)?)
}

/// `ProximityInfoStateUtils::updateTouchPoints` for typing (`isGeometric` false, `pointerId` 0):
/// pushes the point of every input from `push_touch_point_start_index` to the last one of the
/// pointer, and returns how many points there are. With no coordinate on the first input
/// (`proximityOnly`) every point is its key's centre.
pub fn update_touch_points(
    info: &ProximityInfo,
    input_proximities: &[i32],
    input: &TypingInput<'_>,
    input_size: usize,
    push_touch_point_start_index: usize,
    sampled: &mut SampledPoints,
) -> usize {
    // With no input, or past it, the C++ pushes a point from outside the arrays.
    if push_touch_point_start_index >= input_size {
        return sampled.len();
    }
    let proximity_only = input.x(0) < 0 || input.y(0) < 0;
    let last_input_index = (push_touch_point_start_index..input_size)
        .rev()
        .find(|&i| input.pointer_id(i) == 0)
        .unwrap_or(push_touch_point_start_index);
    for i in push_touch_point_start_index..=last_input_index {
        if input.pointer_id(i) == 0 {
            let c = proximity_row(input_proximities, i).map_or(0, |row| row[0]);
            let (x, y) = if proximity_only {
                (NOT_A_COORDINATE, NOT_A_COORDINATE)
            } else {
                (input.x(i), input.y(i))
            };
            push_touch_point(info, i, c, x, y, input.time(i), sampled);
        }
    }
    sampled.len()
}

/// `ProximityInfoStateUtils::pushTouchPoint` without sampling: a tap missing a coordinate takes
/// the centre of the key its code point types (if it types one), and the point is added.
fn push_touch_point(
    info: &ProximityInfo,
    input_index: usize,
    node_code_point: i32,
    mut x: i32,
    mut y: i32,
    time: i32,
    sampled: &mut SampledPoints,
) {
    if node_code_point >= 0 && (x < 0 || y < 0) {
        let key_id = info.get_key_index_of(node_code_point);
        if key_id >= 0 {
            x = info.get_key_center_x_of_key_id_g(key_id, NOT_AN_INDEX);
            y = info.get_key_center_y_of_key_id_g(key_id, NOT_AN_INDEX);
        }
    }
    sampled.xs.push(x);
    sampled.ys.push(y);
    sampled.times.push(time);
    sampled.indice.push(input_index as i32);
}

/// `ProximityInfoStateUtils::initGeometricDistanceInfos` for typing: the normalised squared
/// distance from every point to every key, from point `last_saved_input_size` on, in
/// `sampled_normalized_squared_length_cache` (`sampled_input_size` times the key count long).
pub fn init_geometric_distance_infos(
    info: &ProximityInfo,
    sampled_input_size: usize,
    last_saved_input_size: usize,
    sampled: &SampledPoints,
    sampled_normalized_squared_length_cache: &mut Vec<f32>,
) {
    let key_count = info.get_key_count();
    sampled_normalized_squared_length_cache
        .resize(sampled_input_size.wrapping_mul(key_count as usize), 0.0);
    let points = sampled.xs.iter().zip(&sampled.ys);
    for (i, (&x, &y)) in points
        .enumerate()
        .take(sampled_input_size)
        .skip(last_saved_input_size)
    {
        for k in 0..key_count {
            let index = i * key_count as usize + k as usize;
            sampled_normalized_squared_length_cache[index] =
                info.get_normalized_squared_distance_from_center_float_g(k, x, y);
        }
    }
}

/// `ProximityInfoStateUtils::getPointToKeyByIdLength`: the distance cached for the input and key,
/// at most `max_point_to_key_length`; `MAX_VALUE_FOR_WEIGHTING` for no key. (And for an input
/// outside the cache, where the C++ reads outside its vector.)
pub fn get_point_to_key_by_id_length(
    max_point_to_key_length: f32,
    sampled_normalized_squared_length_cache: &[f32],
    key_count: i32,
    input_index: i32,
    key_id: i32,
) -> f32 {
    if key_id != NOT_AN_INDEX {
        let cached = input_index
            .checked_mul(key_count)
            .and_then(|it| it.checked_add(key_id))
            .and_then(|it| usize::try_from(it).ok())
            .and_then(|it| sampled_normalized_squared_length_cache.get(it));
        if let Some(&cached) = cached {
            // std::min(cached, max) is `(max < cached) ? max : cached`.
            return if max_point_to_key_length < cached {
                max_point_to_key_length
            } else {
                cached
            };
        }
    }
    // If the char is not a key on the keyboard then return the max length.
    MAX_VALUE_FOR_WEIGHTING as f32
}
