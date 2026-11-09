//! `ProximityInfo`, `ProximityInfoState` and the layout helpers, from `cpp/hooks/layout.cpp`.
//!
//! A [`Info`] is a `latinime::ProximityInfo` (the oracle's QWERTY one, or a random keyboard built
//! through the same JNI method), a [`State`] a `latinime::ProximityInfoState` (one made here, or
//! the one a decode session holds). Every call takes the oracle lock. The C++ reads vectors and
//! arrays without bounds checks, so the readers assert the indexes the C++ keeps inside them.

use crate::{MAX_WORD_LENGTH, NOT_A_COORDINATE, Proximity, Session, lock};

/// The native MAX_PROXIMITY_CHARS_SIZE.
pub const MAX_PROXIMITY_CHARS_SIZE: usize = 16;

// SAFETY: cpp/hooks/layout.cpp defines these with the same C signatures, and cpp/fake_jni.cpp
// the last three.
unsafe extern "C" {
    fn aw_hook_info_params(info: i64, out: *mut i32);
    fn aw_hook_info_grid(info: i64, out: *mut i32, count: i32);
    fn aw_hook_info_key(info: i64, key: i32, out: *mut i32);
    fn aw_hook_info_key_index_of(info: i64, c: i32) -> i32;
    fn aw_hook_info_on_keyboard(info: i64, c: i32) -> i32;
    fn aw_hook_info_has_space_proximity(info: i64, x: i32, y: i32) -> i32;
    fn aw_hook_info_normalized_squared_distance(info: i64, key_id: i32, x: i32, y: i32) -> f32;
    fn aw_hook_info_key_center_x(info: i64, key_id: i32, reference_x: i32) -> i32;
    fn aw_hook_info_key_center_y(info: i64, key_id: i32, reference_y: i32) -> i32;
    fn aw_hook_info_initialize_proximities(
        info: i64,
        codes: *const i32,
        xs: *const i32,
        ys: *const i32,
        size: i32,
        locale: *const i32,
        locale_length: i32,
        out: *mut i32,
    );
    fn aw_hook_sweet_spot_factor(enabled: i32, x: f32) -> f32;
    fn aw_hook_state_new() -> i64;
    fn aw_hook_state_free(state: i64);
    fn aw_hook_session_state(session: i64) -> i64;
    fn aw_hook_state_init(
        state: i64,
        max_point_to_key_length: f32,
        info: i64,
        codes: *const i32,
        size: i32,
        xs: *const i32,
        ys: *const i32,
        times: *const i32,
        pointer_ids: *const i32,
        locale: *const i32,
        locale_length: i32,
    );
    fn aw_hook_state_summary(state: i64, out: *mut i32);
    fn aw_hook_state_proximities(state: i64, out: *mut i32);
    fn aw_hook_state_point(state: i64, index: i32, out: *mut i32);
    fn aw_hook_state_point_to_key_length(state: i64, index: i32, code_point: i32) -> f32;
    fn aw_hook_state_point_to_key_by_id_length(state: i64, index: i32, key_id: i32) -> f32;
    fn aw_hook_state_proximity_type(
        state: i64,
        index: i32,
        code_point: i32,
        check: i32,
        proximity_index: *mut i32,
    ) -> i32;
    fn aw_hook_state_same_as_typed(state: i64, word: *const i32, length: i32) -> i32;
    fn aw_hook_state_exists_in_proximity(state: i64, index: i32, c: i32) -> i32;
    fn aw_hook_state_exists_adjacent(state: i64, index: i32) -> i32;
    fn aw_hook_state_has_space_proximity(state: i64, index: i32) -> i32;
    fn aw_hook_state_primary(state: i64, index: i32) -> i32;
    fn aw_hook_state_primary_original(state: i64, index: i32) -> i32;

    fn aw_oracle_proximity(
        width: i32,
        height: i32,
        grid_width: i32,
        grid_height: i32,
        key_width: i32,
        key_height: i32,
        grid: *const i32,
        grid_length: i32,
        key_count: i32,
        xs: *const i32,
        ys: *const i32,
        widths: *const i32,
        heights: *const i32,
        codes: *const i32,
    ) -> i64;
    fn aw_oracle_proximity_release(proximity: i64);
}

/// `TouchPositionCorrectionUtils::getSweetSpotFactor`.
pub fn sweet_spot_factor(enabled: bool, x: f32) -> f32 {
    let _lock = lock();
    // SAFETY: a pure function of its arguments.
    unsafe { aw_hook_sweet_spot_factor(i32::from(enabled), x) }
}

/// A `ProximityInfo` the hooks can read: the C++ object's address.
#[derive(Clone, Copy, Debug)]
pub struct Info(i64);

impl Info {
    /// The oracle's QWERTY keyboard, which must outlive this.
    pub fn of(proximity: &Proximity) -> Self {
        Self(proximity.address())
    }
}

/// What `ProximityInfo`'s getters say of the keyboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InfoParams {
    pub key_count: i32,
    pub cell_width: i32,
    pub cell_height: i32,
    pub grid_width: i32,
    pub grid_height: i32,
    pub keyboard_height: i32,
    pub most_common_key_width: i32,
}

impl Info {
    /// The keyboard's size getters.
    pub fn params(self) -> InfoParams {
        let mut out = [0; 7];
        let _lock = lock();
        // SAFETY: the object is alive (the owner of this handle outlives it); out holds 7 ints.
        unsafe { aw_hook_info_params(self.0, out.as_mut_ptr()) };
        InfoParams {
            key_count: out[0],
            cell_width: out[1],
            cell_height: out[2],
            grid_width: out[3],
            grid_height: out[4],
            keyboard_height: out[5],
            most_common_key_width: out[6],
        }
    }

    /// `mProximityCharsArray`: `grid_width * grid_height * 16` code points.
    pub fn grid(self) -> Vec<i32> {
        let params = self.params();
        let count = params.grid_width * params.grid_height * MAX_PROXIMITY_CHARS_SIZE as i32;
        let mut out = vec![0; count as usize];
        let _lock = lock();
        // SAFETY: the array has exactly `count` ints (the constructor checked its length), and
        // out holds that many.
        unsafe { aw_hook_info_grid(self.0, out.as_mut_ptr(), count) };
        out
    }

    /// `getCodePointOf` and `getOriginalCodePointOf` of a key index.
    pub fn key(self, key: i32) -> (i32, i32) {
        let mut out = [0; 2];
        let _lock = lock();
        // SAFETY: bounds-checked in the C++; out holds 2 ints.
        unsafe { aw_hook_info_key(self.0, key, out.as_mut_ptr()) };
        (out[0], out[1])
    }

    /// `getKeyIndexOf`.
    pub fn key_index_of(self, c: i32) -> i32 {
        let _lock = lock();
        // SAFETY: a lookup in the object's map.
        unsafe { aw_hook_info_key_index_of(self.0, c) }
    }

    /// `isCodePointOnKeyboard`.
    pub fn on_keyboard(self, c: i32) -> bool {
        let _lock = lock();
        // SAFETY: as key_index_of.
        unsafe { aw_hook_info_on_keyboard(self.0, c) != 0 }
    }

    /// `hasSpaceProximity`; the point must be where the grid has a cell (the C++ reads outside
    /// its array otherwise).
    pub fn has_space_proximity(self, x: i32, y: i32) -> bool {
        self.assert_in_grid(x, y);
        let _lock = lock();
        // SAFETY: the cell is inside the grid, asserted above.
        unsafe { aw_hook_info_has_space_proximity(self.0, x, y) != 0 }
    }

    /// `getNormalizedSquaredDistanceFromCenterFloatG` for typing, for a key index the keyboard has
    /// (or none, which the C++ checks).
    pub fn normalized_squared_distance(self, key_id: i32, x: i32, y: i32) -> f32 {
        assert!(
            key_id < self.params().key_count,
            "the C++ reads outside its keys"
        );
        let _lock = lock();
        // SAFETY: the key is inside the arrays, or negative, which the C++ turns into 0.
        unsafe { aw_hook_info_normalized_squared_distance(self.0, key_id, x, y) }
    }

    /// `getKeyCenterXOfKeyIdG`.
    pub fn key_center_x(self, key_id: i32, reference_x: i32) -> i32 {
        assert!(
            key_id < self.params().key_count,
            "the C++ reads outside its keys"
        );
        let _lock = lock();
        // SAFETY: as normalized_squared_distance.
        unsafe { aw_hook_info_key_center_x(self.0, key_id, reference_x) }
    }

    /// `getKeyCenterYOfKeyIdG` for typing.
    pub fn key_center_y(self, key_id: i32, reference_y: i32) -> i32 {
        assert!(
            key_id < self.params().key_count,
            "the C++ reads outside its keys"
        );
        let _lock = lock();
        // SAFETY: as normalized_squared_distance.
        unsafe { aw_hook_info_key_center_y(self.0, key_id, reference_y) }
    }

    /// `initializeProximities`: 16 code points for each of `codes`. The arrays are padded to
    /// `MAX_WORD_LENGTH` as the app's are; each point must be inside the grid or above and left
    /// of it.
    pub fn initialize_proximities(
        self,
        codes: &[i32],
        xs: &[i32],
        ys: &[i32],
        locale: &[i32],
    ) -> Vec<i32> {
        let size = codes.len();
        assert!(size <= MAX_WORD_LENGTH && xs.len() == size && ys.len() == size);
        for (&x, &y) in xs.iter().zip(ys) {
            if x != NOT_A_COORDINATE && y != NOT_A_COORDINATE {
                self.assert_cell_readable(x, y);
            }
        }
        let (codes, xs, ys) = (padded(codes, 0), padded(xs, -1), padded(ys, -1));
        let mut out = vec![0; MAX_PROXIMITY_CHARS_SIZE * MAX_WORD_LENGTH];
        let _lock = lock();
        // SAFETY: the arrays hold MAX_WORD_LENGTH ints, the cells read are inside the grid
        // (asserted), and out holds 16 ints for each of the size inputs.
        unsafe {
            aw_hook_info_initialize_proximities(
                self.0,
                codes.as_ptr(),
                xs.as_ptr(),
                ys.as_ptr(),
                size as i32,
                locale.as_ptr(),
                locale.len() as i32,
                out.as_mut_ptr(),
            );
        }
        out.truncate(size * MAX_PROXIMITY_CHARS_SIZE);
        out
    }

    /// Whether the C++ reads `(x, y)`'s grid cell inside its array: `calculateProximities`
    /// skips negative start indexes, and the array ends after the last cell.
    fn assert_cell_readable(self, x: i32, y: i32) {
        let params = self.params();
        let row = i64::from(y / params.cell_height);
        let start = row * i64::from(params.grid_width) + i64::from(x / params.cell_width);
        let cells = i64::from(params.grid_width) * i64::from(params.grid_height);
        assert!(
            start < cells,
            "({x}, {y}) is past the grid, where the C++ reads outside it"
        );
    }

    fn assert_in_grid(self, x: i32, y: i32) {
        if x >= 0 && y >= 0 {
            self.assert_cell_readable(x, y);
        }
    }
}

/// `values` as an array of `MAX_WORD_LENGTH`, filled with `fill`.
fn padded(values: &[i32], fill: i32) -> Vec<i32> {
    let mut array = vec![fill; MAX_WORD_LENGTH];
    array[..values.len()].copy_from_slice(values);
    array
}

/// What `setProximityInfoNative` takes (no sweet spots).
#[derive(Clone, Copy, Debug)]
pub struct RawKeyboard<'a> {
    pub keyboard_width: i32,
    pub keyboard_height: i32,
    pub grid_width: i32,
    pub grid_height: i32,
    pub most_common_key_width: i32,
    pub most_common_key_height: i32,
    pub proximity_chars: &'a [i32],
    pub key_count: i32,
    pub key_xs: &'a [i32],
    pub key_ys: &'a [i32],
    pub key_widths: &'a [i32],
    pub key_heights: &'a [i32],
    pub key_codes: &'a [i32],
}

/// A `ProximityInfo` built from any keyboard, as `Proximity::qwerty` builds the app's.
#[derive(Debug)]
pub struct Keyboard {
    handle: i64,
}

impl Keyboard {
    /// The keyboard `setProximityInfoNative` makes from `raw`, which must be valid (the grid
    /// array `grid_width * grid_height * 16` long, each key array `key_count` long, a grid and
    /// cells that aren't empty): the C++ builds half an object for one that isn't.
    pub fn new(raw: &RawKeyboard<'_>) -> Self {
        let grid_cells = i64::from(raw.grid_width) * i64::from(raw.grid_height);
        assert_eq!(
            raw.proximity_chars.len() as i64,
            grid_cells * MAX_PROXIMITY_CHARS_SIZE as i64
        );
        for array in [
            raw.key_xs,
            raw.key_ys,
            raw.key_widths,
            raw.key_heights,
            raw.key_codes,
        ] {
            assert_eq!(array.len() as i64, i64::from(raw.key_count));
        }
        let _lock = lock();
        // SAFETY: every array is valid for the length given and only read during the call.
        let handle = unsafe {
            aw_oracle_proximity(
                raw.keyboard_width,
                raw.keyboard_height,
                raw.grid_width,
                raw.grid_height,
                raw.most_common_key_width,
                raw.most_common_key_height,
                raw.proximity_chars.as_ptr(),
                raw.proximity_chars.len() as i32,
                raw.key_count,
                raw.key_xs.as_ptr(),
                raw.key_ys.as_ptr(),
                raw.key_widths.as_ptr(),
                raw.key_heights.as_ptr(),
                raw.key_codes.as_ptr(),
            )
        };
        Self { handle }
    }

    pub fn info(&self) -> Info {
        Info(self.handle)
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        let _lock = lock();
        // SAFETY: the handle came from aw_oracle_proximity and is released only here.
        unsafe { aw_oracle_proximity_release(self.handle) }
    }
}

mod state;
pub use state::{State, StateInput, StateSummary};
