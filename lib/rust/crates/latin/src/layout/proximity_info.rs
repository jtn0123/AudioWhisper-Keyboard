//! `ProximityInfo` (`suggest/core/layout/proximity_info.{h,cpp}`) and the JNI glue that builds
//! one from Java's arrays (`com_android_inputmethod_keyboard_ProximityInfo.cpp`): the keyboard as
//! the decoder sees it, a grid of cells each listing the keys near it, and the keys' boxes.
//!
//! The app never gives the keyboard sweet spots (touch position calibration), so the C++'s
//! `HAS_TOUCH_POSITION_CORRECTION_DATA` is always false and every sweet spot branch is skipped:
//! `hasSweetSpotData`, `getSweetSpotRadiiAt`, `getSweetSpotCenterXAt`, `getSweetSpotCenterYAt`,
//! `mSweetSpotCenterYsG`, and the sweet spot arms of `getKeyCenterXOfKeyIdG` and
//! `getKeyCenterYOfKeyIdG`. Also skipped, because only gesture input reads them:
//! `getKeyKeyDistanceG` and `mKeyKeyDistancesG` (`hypotf`), `KEYBOARD_HYPOTENUSE` (`hypotf`),
//! `NORMALIZED_SQUARED_MOST_COMMON_KEY_HYPOTENUSE`, `MOST_COMMON_KEY_WIDTH_SQUARE` (the state
//! only stores it), the `getKeyboardWidth`, `getKeyboardHypotenuse`, `getGridHeight` and
//! `getMostCommonKeyWidthSquare` getters nobody calls, and the `isGeometric` argument (the sweet
//! spot arm of `getKeyCenterYOfKeyIdG` is its only use).

use crate::chars;
use crate::defines::{
    KEYCODE_SPACE, MAX_KEY_COUNT_IN_A_KEYBOARD, MAX_PROXIMITY_CHARS_SIZE, NOT_A_CODE_POINT,
    NOT_A_COORDINATE, NOT_AN_INDEX,
};

use super::geometry_utils::square_float;
use super::proximity_info_state::TypingInput;
use super::proximity_info_utils;

/// The arguments of `setProximityInfoNative`, as Java passes them. The key arrays are the ones
/// the Java side sizes to `key_count`; the C++ also accepts null for them (and fills zeros),
/// which a slice can't say, so pass empty slices with a `key_count` of 0 instead.
#[derive(Clone, Copy, Debug)]
pub struct KeyboardGeometry<'a> {
    pub keyboard_width: i32,
    pub keyboard_height: i32,
    pub grid_width: i32,
    pub grid_height: i32,
    pub most_common_key_width: i32,
    /// Only feeds `NORMALIZED_SQUARED_MOST_COMMON_KEY_HYPOTENUSE`, which typing never reads.
    pub most_common_key_height: i32,
    /// `grid_width * grid_height * MAX_PROXIMITY_CHARS_SIZE` code points: for each cell, the keys
    /// near it, padded with `NOT_A_CODE_POINT`.
    pub proximity_chars: &'a [i32],
    pub key_count: i32,
    pub key_x_coordinates: &'a [i32],
    pub key_y_coordinates: &'a [i32],
    pub key_widths: &'a [i32],
    pub key_heights: &'a [i32],
    pub key_char_codes: &'a [i32],
}

/// Why a [`ProximityInfo`] can't be built. The C++ returns half way through its constructor for
/// the first one (leaving the object unusable) and crashes or throws for the others.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutError {
    /// The grid isn't at least 1 by 1, or the keyboard has no width or height to divide into it.
    InvalidGrid,
    /// `proximity_chars` isn't `grid_width * grid_height * MAX_PROXIMITY_CHARS_SIZE` long.
    InvalidProximityCharsLength,
    /// `key_count` is negative, or a key array is shorter than the keys it should describe.
    InvalidKeys,
}

/// `ProximityInfo`.
#[derive(Clone, Debug)]
pub struct ProximityInfo {
    most_common_key_width: i32,
    cell_width: i32,
    cell_height: i32,
    grid_width: i32,
    key_count: i32,
    keyboard_height: i32,
    /// `mProximityCharsArray`.
    proximity_chars: Vec<i32>,
    /// `mKeyXCoordinates`, `mKeyYCoordinates`, `mKeyWidths`, `mKeyHeights`, one per key.
    key_x_coordinates: Vec<i32>,
    key_y_coordinates: Vec<i32>,
    key_widths: Vec<i32>,
    key_heights: Vec<i32>,
    /// `mCenterXsG`, `mCenterYsG`.
    center_xs: Vec<i32>,
    center_ys: Vec<i32>,
    /// `mLowerCodePointToKeyMap`, as a list sorted by code point: the C++'s `unordered_map` has one
    /// entry per lower case code point, the last key with it.
    lower_code_point_to_key: Vec<(i32, i32)>,
    /// `mKeyIndexToOriginalCodePoint`, `mKeyIndexToLowerCodePointG`.
    key_index_to_original_code_point: Vec<i32>,
    key_index_to_lower_code_point: Vec<i32>,
}

impl ProximityInfo {
    /// The `ProximityInfo` constructor, with the checks of `latinime_Keyboard_setProximityInfo`'s
    /// arguments the C++ makes (or should have).
    pub fn new(geometry: &KeyboardGeometry<'_>) -> Result<Self, LayoutError> {
        let cell_width = cell_size(geometry.keyboard_width, geometry.grid_width)?;
        let cell_height = cell_size(geometry.keyboard_height, geometry.grid_height)?;
        let expected = i64::from(geometry.grid_width)
            * i64::from(geometry.grid_height)
            * MAX_PROXIMITY_CHARS_SIZE as i64;
        if geometry.proximity_chars.len() as i64 != expected {
            return Err(LayoutError::InvalidProximityCharsLength);
        }
        // KEY_COUNT(std::min(keyCount, MAX_KEY_COUNT_IN_A_KEYBOARD))
        let key_count = usize::try_from(geometry.key_count)
            .map_err(|_| LayoutError::InvalidKeys)?
            .min(MAX_KEY_COUNT_IN_A_KEYBOARD);
        let first_keys = |array: &[i32]| -> Result<Vec<i32>, LayoutError> {
            Ok(array
                .get(..key_count)
                .ok_or(LayoutError::InvalidKeys)?
                .to_vec())
        };
        let mut info = Self {
            most_common_key_width: geometry.most_common_key_width,
            cell_width,
            cell_height,
            grid_width: geometry.grid_width,
            key_count: key_count as i32,
            keyboard_height: geometry.keyboard_height,
            proximity_chars: geometry.proximity_chars.to_vec(),
            key_x_coordinates: first_keys(geometry.key_x_coordinates)?,
            key_y_coordinates: first_keys(geometry.key_y_coordinates)?,
            key_widths: first_keys(geometry.key_widths)?,
            key_heights: first_keys(geometry.key_heights)?,
            center_xs: Vec::with_capacity(key_count),
            center_ys: Vec::with_capacity(key_count),
            lower_code_point_to_key: Vec::with_capacity(key_count),
            key_index_to_original_code_point: first_keys(geometry.key_char_codes)?,
            key_index_to_lower_code_point: Vec::with_capacity(key_count),
        };
        info.initialize_g();
        Ok(info)
    }

    /// `ProximityInfo::initializeG`, without the sweet spots and the key-to-key distances.
    fn initialize_g(&mut self) {
        for index in 0..self.key_count as usize {
            let code = self.key_index_to_original_code_point[index];
            let lower_code = chars::to_lower_case(code);
            self.center_xs
                .push(self.key_x_coordinates[index].wrapping_add(self.key_widths[index] / 2));
            self.center_ys
                .push(self.key_y_coordinates[index].wrapping_add(self.key_heights[index] / 2));
            // mLowerCodePointToKeyMap[lowerCode] = i;
            match self
                .lower_code_point_to_key
                .binary_search_by_key(&lower_code, |&(code, _)| code)
            {
                Ok(at) => self.lower_code_point_to_key[at].1 = index as i32,
                Err(at) => self
                    .lower_code_point_to_key
                    .insert(at, (lower_code, index as i32)),
            }
            self.key_index_to_lower_code_point.push(lower_code);
        }
    }

    /// The keys' count (`getKeyCount`), at most `MAX_KEY_COUNT_IN_A_KEYBOARD`.
    pub fn get_key_count(&self) -> i32 {
        self.key_count
    }

    /// `getMostCommonKeyWidth`.
    pub fn get_most_common_key_width(&self) -> i32 {
        self.most_common_key_width
    }

    /// `getCellWidth`.
    pub fn get_cell_width(&self) -> i32 {
        self.cell_width
    }

    /// `getCellHeight`.
    pub fn get_cell_height(&self) -> i32 {
        self.cell_height
    }

    /// `getGridWidth`.
    pub fn get_grid_width(&self) -> i32 {
        self.grid_width
    }

    /// The keyboard's height (`getKeyboardHeight`).
    pub fn get_keyboard_height(&self) -> i32 {
        self.keyboard_height
    }

    /// The key at `key_index`, as an index into the key arrays, or `None` if there is no such key.
    /// (The C++ reads its fixed size arrays there; its tail is never written.)
    fn key(&self, key_index: i32) -> Option<usize> {
        usize::try_from(key_index)
            .ok()
            .filter(|&it| it < self.key_count as usize)
    }

    /// The `MAX_PROXIMITY_CHARS_SIZE` code points the grid lists from `start_index`
    /// (`getStartIndexFromCoordinates`'s result), or `None` if that is outside the grid, where the
    /// C++ reads past `mProximityCharsArray`.
    pub(super) fn proximity_chars_at(&self, start_index: i32) -> Option<&[i32]> {
        let start = usize::try_from(start_index).ok()?;
        self.proximity_chars
            .get(start..start.checked_add(MAX_PROXIMITY_CHARS_SIZE)?)
    }

    /// `ProximityInfo::initializeProximities`: for each of the first `input.size()` inputs, the
    /// `MAX_PROXIMITY_CHARS_SIZE` code points `calculateProximities` lists, into
    /// `all_input_codes` (at least that many times the input size long).
    pub fn initialize_proximities(
        &self,
        input: &TypingInput<'_>,
        locale: &[i32],
        all_input_codes: &mut [i32],
    ) {
        proximity_info_utils::initialize_proximities(
            self,
            input,
            input.size(),
            locale,
            all_input_codes,
        );
    }

    /// `ProximityInfo::hasSpaceProximity`: whether the grid cell holding `(x, y)` lists the space
    /// bar. False for a negative coordinate, and (the C++ reads outside its grid) for a point
    /// beyond it.
    pub fn has_space_proximity(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 {
            return false;
        }
        let start_index = proximity_info_utils::get_start_index_from_coordinates(
            x,
            y,
            self.cell_height,
            self.cell_width,
            self.grid_width,
        );
        self.proximity_chars_at(start_index)
            .is_some_and(|cell| cell.contains(&KEYCODE_SPACE))
    }

    /// `ProximityInfo::getNormalizedSquaredDistanceFromCenterFloatG`, for typing
    /// (`isGeometric` false): the squared distance from the touch to the key's centre, in units
    /// of the squared most common key width.
    pub fn get_normalized_squared_distance_from_center_float_g(
        &self,
        key_id: i32,
        x: i32,
        y: i32,
    ) -> f32 {
        let center_x = self.get_key_center_x_of_key_id_g(key_id, x) as f32;
        let center_y = self.get_key_center_y_of_key_id_g(key_id, y) as f32;
        let touch_x = x as f32;
        let touch_y = y as f32;
        proximity_info_utils::get_squared_distance_float(center_x, center_y, touch_x, touch_y)
            / square_float(self.most_common_key_width as f32)
    }

    /// `ProximityInfo::getCodePointOf`: the key's lower case code point, `NOT_A_CODE_POINT` if
    /// there is no such key.
    pub fn get_code_point_of(&self, key_index: i32) -> i32 {
        self.key(key_index).map_or(NOT_A_CODE_POINT, |it| {
            self.key_index_to_lower_code_point[it]
        })
    }

    /// `ProximityInfo::getOriginalCodePointOf`: the key's code point as given, `NOT_A_CODE_POINT`
    /// if there is no such key.
    pub fn get_original_code_point_of(&self, key_index: i32) -> i32 {
        self.key(key_index).map_or(NOT_A_CODE_POINT, |it| {
            self.key_index_to_original_code_point[it]
        })
    }

    /// `ProximityInfo::getKeyCenterXOfKeyIdG`: the key's centre, or for a key wider than the most
    /// common one and a `reference_point_x` that isn't `NOT_A_COORDINATE`, the point of its
    /// middle segment nearest `reference_point_x`. 0 for no key.
    pub fn get_key_center_x_of_key_id_g(&self, key_id: i32, reference_point_x: i32) -> i32 {
        let Some(key) = self.key(key_id) else {
            return 0;
        };
        let mut center_x = self.center_xs[key];
        let key_width = self.key_widths[key];
        if reference_point_x != NOT_A_COORDINATE && key_width > self.most_common_key_width {
            let key_width_half_diff = key_width.wrapping_sub(self.most_common_key_width) / 2;
            if reference_point_x < center_x.wrapping_sub(key_width_half_diff) {
                center_x = center_x.wrapping_sub(key_width_half_diff);
            } else if reference_point_x > center_x.wrapping_add(key_width_half_diff) {
                center_x = center_x.wrapping_add(key_width_half_diff);
            } else {
                center_x = reference_point_x;
            }
        }
        center_x
    }

    /// `ProximityInfo::getKeyCenterYOfKeyIdG` for typing: the key's centre, or `reference_point_y`
    /// for a key on the bottom row that the point is below. 0 for no key.
    pub fn get_key_center_y_of_key_id_g(&self, key_id: i32, reference_point_y: i32) -> i32 {
        let Some(key) = self.key(key_id) else {
            return 0;
        };
        let center_y = self.center_ys[key];
        if reference_point_y != NOT_A_COORDINATE
            && center_y.wrapping_add(self.key_heights[key]) > self.keyboard_height
            && center_y < reference_point_y
        {
            return reference_point_y;
        }
        center_y
    }

    /// `ProximityInfo::getKeyIndexOf`: the index of the key that types `c` in either case, or
    /// `NOT_AN_INDEX`.
    pub fn get_key_index_of(&self, c: i32) -> i32 {
        proximity_info_utils::get_key_index_of(self, c)
    }

    /// `ProximityInfo::isCodePointOnKeyboard`.
    pub fn is_code_point_on_keyboard(&self, code_point: i32) -> bool {
        self.get_key_index_of(code_point) != NOT_AN_INDEX
    }

    /// The key index `lower_code` (already lower case) types, from `mLowerCodePointToKeyMap`.
    pub(super) fn key_of_lower_code_point(&self, lower_code: i32) -> Option<i32> {
        self.lower_code_point_to_key
            .binary_search_by_key(&lower_code, |&(code, _)| code)
            .ok()
            .map(|at| self.lower_code_point_to_key[at].1)
    }

    /// `isOnKey`'s inputs for a key: its left and top edges and its width and height.
    pub(super) fn key_box(&self, key_id: i32) -> Option<[i32; 4]> {
        let key = self.key(key_id)?;
        Some([
            self.key_x_coordinates[key],
            self.key_y_coordinates[key],
            self.key_widths[key],
            self.key_heights[key],
        ])
    }
}

/// `CELL_WIDTH` and `CELL_HEIGHT`: `(size + cells - 1) / cells`, which the C++ divides by with no
/// check (a 0 would crash the first lookup).
fn cell_size(size: i32, cells: i32) -> Result<i32, LayoutError> {
    if cells < 1 {
        return Err(LayoutError::InvalidGrid);
    }
    let cell = size.wrapping_add(cells).wrapping_sub(1) / cells;
    if cell < 1 {
        return Err(LayoutError::InvalidGrid);
    }
    Ok(cell)
}
