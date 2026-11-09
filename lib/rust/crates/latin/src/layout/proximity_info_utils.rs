//! `ProximityInfoUtils` (`suggest/core/layout/proximity_info_utils.h`): looking a code point up
//! on the keyboard and listing, for a tap, the letters near it.
//!
//! The C++ takes the keyboard's arrays one by one; here they are read from the [`ProximityInfo`]
//! that owns them. Skipped: `pointToLineSegSquaredDistanceFloat` (nothing calls it).
//!
//! The C++'s `int` arithmetic on coordinates wraps on the hardware where it overflows (undefined
//! in C++); here it wraps explicitly, so no input panics.

use crate::chars;
use crate::defines::{
    ADDITIONAL_PROXIMITY_CHAR_DELIMITER_CODE, KEYCODE_SPACE, MAX_PROXIMITY_CHARS_SIZE,
    MAX_VALUE_FOR_WEIGHTING, NOT_A_CODE_POINT, NOT_A_COORDINATE, NOT_AN_INDEX, ProximityType,
};

use super::additional_proximity_chars::get_additional_chars;
use super::geometry_utils::square_float;
use super::proximity_info::ProximityInfo;
use super::proximity_info_state::TypingInput;

/// `ProximityInfoUtils::getKeyIndexOf`: the index of the key typing `c` in either case, or
/// `NOT_AN_INDEX` (also for a keyboard with no keys, and for `NOT_A_CODE_POINT`).
pub fn get_key_index_of(info: &ProximityInfo, c: i32) -> i32 {
    if info.get_key_count() == 0 {
        // We do not have the coordinate data
        return NOT_AN_INDEX;
    }
    if c == NOT_A_CODE_POINT {
        return NOT_AN_INDEX;
    }
    let lower_code = chars::to_lower_case(c);
    info.key_of_lower_code_point(lower_code)
        .unwrap_or(NOT_AN_INDEX)
}

/// `ProximityInfoUtils::initializeProximities`: for each of the first `input_size` inputs, the
/// `MAX_PROXIMITY_CHARS_SIZE` code points `calculateProximities` lists, into `input_proximities`
/// (at least that many times `input_size` long).
pub fn initialize_proximities(
    info: &ProximityInfo,
    input: &TypingInput<'_>,
    input_size: usize,
    locale: &[i32],
    input_proximities: &mut [i32],
) {
    let (rows, _) = input_proximities.as_chunks_mut::<MAX_PROXIMITY_CHARS_SIZE>();
    for (i, proximities) in rows.iter_mut().take(input_size).enumerate() {
        let primary_key = input.codes[i];
        let x = input.x(i);
        let y = input.y(i);
        calculate_proximities(info, x, y, primary_key, locale, &mut proximities[..]);
    }
}

/// `ProximityInfoUtils::getStartIndexFromCoordinates`: where in the grid the cell holding
/// `(x, y)` starts. Outside the keyboard this is outside the grid (or in another row of it), as
/// in the C++.
pub fn get_start_index_from_coordinates(
    x: i32,
    y: i32,
    cell_height: i32,
    cell_width: i32,
    grid_width: i32,
) -> i32 {
    (y / cell_height)
        .wrapping_mul(grid_width)
        .wrapping_add(x / cell_width)
        .wrapping_mul(MAX_PROXIMITY_CHARS_SIZE as i32)
}

/// `ProximityInfoUtils::getSquaredDistanceFloat`.
pub fn get_squared_distance_float(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    square_float(x1 - x2) + square_float(y1 - y2)
}

/// `ProximityInfoUtils::isMatchOrProximityChar`.
pub fn is_match_or_proximity_char(proximity_type: ProximityType) -> bool {
    matches!(
        proximity_type,
        ProximityType::MatchChar
            | ProximityType::ProximityChar
            | ProximityType::AdditionalProximityChar
    )
}

/// `ProximityInfoUtils::isOnKey`: whether `(x, y)` is in the key's box, which extends a pixel
/// further right than the key is wide. True for no key.
fn is_on_key(info: &ProximityInfo, key_id: i32, x: i32, y: i32) -> bool {
    let Some([left, top, width, height]) = info.key_box(key_id) else {
        // NOT_A_ID is -1, but return whenever < 0 just in case
        return true;
    };
    let right = left.wrapping_add(width).wrapping_add(1);
    let bottom = top.wrapping_add(height);
    left < right && top < bottom && x >= left && x < right && y >= top && y < bottom
}

/// `ProximityInfoUtils::squaredLengthToEdge`: the squared distance from `(x, y)` to the key's
/// box, 0 inside it, `MAX_VALUE_FOR_WEIGHTING` for no key.
fn squared_length_to_edge(info: &ProximityInfo, key_id: i32, x: i32, y: i32) -> i32 {
    let Some([left, top, width, height]) = info.key_box(key_id) else {
        return MAX_VALUE_FOR_WEIGHTING;
    };
    let right = left.wrapping_add(width);
    let bottom = top.wrapping_add(height);
    let edge_x = if x < left {
        left
    } else if x > right {
        right
    } else {
        x
    };
    let edge_y = if y < top {
        top
    } else if y > bottom {
        bottom
    } else {
        y
    };
    let dx = x.wrapping_sub(edge_x);
    let dy = y.wrapping_sub(edge_y);
    dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy))
}

/// `ProximityInfoUtils::calculateProximities`: the typed code point, then the keys near the tap
/// that the grid lists and are on it or within a key width of it, then (for English vowels) the
/// delimiter and the letters the language adds, padded with `NOT_A_CODE_POINT`. `proximities` is
/// `MAX_PROXIMITY_CHARS_SIZE` long. A tap outside the grid lists no keys (the C++ reads outside
/// its array there).
fn calculate_proximities(
    info: &ProximityInfo,
    x: i32,
    y: i32,
    primary_key: i32,
    locale: &[i32],
    proximities: &mut [i32],
) {
    let most_common_key_width = info.get_most_common_key_width();
    let most_common_key_width_square = most_common_key_width.wrapping_mul(most_common_key_width);
    let mut insert_pos = 0;
    proximities[insert_pos] = primary_key;
    insert_pos += 1;
    if x == NOT_A_COORDINATE || y == NOT_A_COORDINATE {
        proximities[insert_pos..].fill(NOT_A_CODE_POINT);
        return;
    }
    let start_index = get_start_index_from_coordinates(
        x,
        y,
        info.get_cell_height(),
        info.get_cell_width(),
        info.get_grid_width(),
    );
    if start_index >= 0 {
        let cell = info.proximity_chars_at(start_index).unwrap_or(&[]);
        for &c in cell {
            if c < KEYCODE_SPACE || c == primary_key {
                continue;
            }
            let key_index = get_key_index_of(info, c);
            let on_key = is_on_key(info, key_index, x, y);
            let distance = squared_length_to_edge(info, key_index, x, y);
            if on_key || distance < most_common_key_width_square {
                proximities[insert_pos] = c;
                insert_pos += 1;
                if insert_pos >= MAX_PROXIMITY_CHARS_SIZE {
                    return;
                }
            }
        }
        let additional_proximity_chars = get_additional_chars(locale, primary_key);
        if !additional_proximity_chars.is_empty() {
            proximities[insert_pos] = ADDITIONAL_PROXIMITY_CHAR_DELIMITER_CODE;
            insert_pos += 1;
            if insert_pos >= MAX_PROXIMITY_CHARS_SIZE {
                return;
            }
            for &ac in additional_proximity_chars {
                if proximities[..insert_pos].contains(&ac) {
                    continue;
                }
                proximities[insert_pos] = ac;
                insert_pos += 1;
                if insert_pos >= MAX_PROXIMITY_CHARS_SIZE {
                    return;
                }
            }
        }
    }
    // Add a delimiter for the proximity characters
    proximities[insert_pos..].fill(NOT_A_CODE_POINT);
}
