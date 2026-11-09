//! `ProximityInfoState` hooks: a state the tests can drive and read.

use super::*;

/// A `ProximityInfoState` the hooks can drive and read.
#[derive(Debug)]
pub struct State {
    handle: i64,
    /// Whether this made the object (and frees it) or borrows a session's.
    owned: bool,
}

/// What `ProximityInfoState`'s simple getters say.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateSummary {
    pub size: i32,
    pub is_used: bool,
    pub touch_position_correction_enabled: bool,
    pub is_continuous_suggestion_possible: bool,
}

/// The arguments of `initInputParams` for typing.
#[derive(Clone, Copy, Debug)]
pub struct StateInput<'a> {
    pub max_point_to_key_length: f32,
    pub codes: &'a [i32],
    pub xs: &'a [i32],
    pub ys: &'a [i32],
    pub times: Option<&'a [i32]>,
    pub pointer_ids: Option<&'a [i32]>,
    /// The dictionary's locale as code points.
    pub locale: &'a [i32],
}

impl State {
    /// A new state with no input.
    pub fn new() -> Self {
        let _lock = lock();
        // SAFETY: allocates an object, freed in drop.
        let handle = unsafe { aw_hook_state_new() };
        Self {
            handle,
            owned: true,
        }
    }

    /// The state the session's last `suggest` left for pointer 0. It lives as long as the session,
    /// which must outlive this.
    pub fn of_session(session: &Session) -> Self {
        let _lock = lock();
        // SAFETY: the session is a live DicTraverseSession.
        let handle = unsafe { aw_hook_session_state(session.address()) };
        Self {
            handle,
            owned: false,
        }
    }

    /// `initInputParams` for typing. The arrays are padded to `MAX_WORD_LENGTH` as the app's are;
    /// the points must be inside the grid (or above and left of it).
    pub fn init(&mut self, info: Info, input: &StateInput<'_>) {
        let size = input.codes.len();
        assert!(size <= MAX_WORD_LENGTH && input.xs.len() == size && input.ys.len() == size);
        for (&x, &y) in input.xs.iter().zip(input.ys) {
            if x != NOT_A_COORDINATE && y != NOT_A_COORDINATE {
                info.assert_cell_readable(x, y);
            }
        }
        let (codes, xs, ys) = (
            padded(input.codes, 0),
            padded(input.xs, -1),
            padded(input.ys, -1),
        );
        let times = input.times.map(|it| padded(it, 0));
        let pointer_ids = input.pointer_ids.map(|it| padded(it, 0));
        let pointer =
            |it: &Option<Vec<i32>>| it.as_ref().map_or(std::ptr::null(), |it| it.as_ptr());
        let _lock = lock();
        // SAFETY: the arrays hold MAX_WORD_LENGTH ints (times and pointer ids may be null, as in
        // the app's JNI call), the grid cells read are inside the array, the info is alive.
        unsafe {
            aw_hook_state_init(
                self.handle,
                input.max_point_to_key_length,
                info.0,
                codes.as_ptr(),
                size as i32,
                xs.as_ptr(),
                ys.as_ptr(),
                pointer(&times),
                pointer(&pointer_ids),
                input.locale.as_ptr(),
                input.locale.len() as i32,
            );
        }
    }

    /// `size`, `isUsed`, `touchPositionCorrectionEnabled` and `isContinuousSuggestionPossible`.
    pub fn summary(&self) -> StateSummary {
        let mut out = [0; 4];
        let _lock = lock();
        // SAFETY: out holds 4 ints.
        unsafe { aw_hook_state_summary(self.handle, out.as_mut_ptr()) };
        StateSummary {
            size: out[0],
            is_used: out[1] != 0,
            touch_position_correction_enabled: out[2] != 0,
            is_continuous_suggestion_possible: out[3] != 0,
        }
    }

    /// `mInputProximities`: 16 code points for each of `MAX_WORD_LENGTH` inputs.
    pub fn proximities(&self) -> Vec<i32> {
        let mut out = vec![0; MAX_PROXIMITY_CHARS_SIZE * MAX_WORD_LENGTH];
        let _lock = lock();
        // SAFETY: out holds the whole array.
        unsafe { aw_hook_state_proximities(self.handle, out.as_mut_ptr()) };
        out
    }

    fn assert_point(&self, index: i32) {
        assert!(
            (0..self.summary().size).contains(&index),
            "the C++ reads outside its vectors for point {index}"
        );
    }

    /// `getInputX`, `getInputY` and `getInputIndexOfSampledPoint` of a point.
    pub fn point(&self, index: i32) -> (i32, i32, i32) {
        self.assert_point(index);
        let mut out = [0; 3];
        let _lock = lock();
        // SAFETY: the point exists, asserted above; out holds 3 ints.
        unsafe { aw_hook_state_point(self.handle, index, out.as_mut_ptr()) };
        (out[0], out[1], out[2])
    }

    /// `getPointToKeyLength`, for a point the state has.
    pub fn point_to_key_length(&self, index: i32, code_point: i32) -> f32 {
        self.assert_point(index);
        let _lock = lock();
        // SAFETY: the point exists, asserted above.
        unsafe { aw_hook_state_point_to_key_length(self.handle, index, code_point) }
    }

    /// `getPointToKeyByIdLength`, for a point the state has and a key the keyboard has.
    pub fn point_to_key_by_id_length(&self, index: i32, key_id: i32, key_count: i32) -> f32 {
        self.assert_point(index);
        assert!(key_id < key_count, "the C++ reads outside its cache");
        let _lock = lock();
        // SAFETY: the point and the key exist (asserted above), or the key is -1, which the C++
        // answers without reading.
        unsafe { aw_hook_state_point_to_key_by_id_length(self.handle, index, key_id) }
    }

    /// `getProximityType` as its enum's value, and the proximity index it wrote (`None` if it
    /// didn't).
    pub fn proximity_type(
        &self,
        index: i32,
        code_point: i32,
        check_proximity_chars: bool,
    ) -> (i32, Option<i32>) {
        assert!((0..MAX_WORD_LENGTH as i32).contains(&index));
        assert!(
            code_point >= 0,
            "the C++ reads outside its table for {code_point}"
        );
        let mut proximity_index = i32::MIN;
        let _lock = lock();
        // SAFETY: the row is inside mInputProximities; proximity_index is a valid int.
        let kind = unsafe {
            aw_hook_state_proximity_type(
                self.handle,
                index,
                code_point,
                i32::from(check_proximity_chars),
                &raw mut proximity_index,
            )
        };
        (
            kind,
            (proximity_index != i32::MIN).then_some(proximity_index),
        )
    }

    /// `sameAsTyped`.
    pub fn same_as_typed(&self, word: &[i32]) -> bool {
        let _lock = lock();
        // SAFETY: the C++ reads at most `word.len()` ints, and compares only when it equals the
        // size.
        unsafe { aw_hook_state_same_as_typed(self.handle, word.as_ptr(), word.len() as i32) != 0 }
    }

    /// `existsCodePointInProximityAt`.
    pub fn exists_in_proximity(&self, index: i32, c: i32) -> bool {
        assert!((0..MAX_WORD_LENGTH as i32).contains(&index));
        let _lock = lock();
        // SAFETY: the row is inside mInputProximities.
        unsafe { aw_hook_state_exists_in_proximity(self.handle, index, c) != 0 }
    }

    /// `existsAdjacentProximityChars`.
    pub fn exists_adjacent(&self, index: i32) -> bool {
        let _lock = lock();
        // SAFETY: the C++ checks the index against the size.
        unsafe { aw_hook_state_exists_adjacent(self.handle, index) != 0 }
    }

    /// `hasSpaceProximity`, for a point the state has.
    pub fn has_space_proximity(&self, index: i32) -> bool {
        self.assert_point(index);
        let _lock = lock();
        // SAFETY: the point exists, asserted above, and so does its grid cell if it is inside the
        // keyboard (the tests only use such points).
        unsafe { aw_hook_state_has_space_proximity(self.handle, index) != 0 }
    }

    /// `getPrimaryCodePointAt`.
    pub fn primary(&self, index: i32) -> i32 {
        assert!((0..MAX_WORD_LENGTH as i32).contains(&index));
        let _lock = lock();
        // SAFETY: the row is inside mInputProximities.
        unsafe { aw_hook_state_primary(self.handle, index) }
    }

    /// `getPrimaryOriginalCodePointAt`.
    pub fn primary_original(&self, index: i32) -> i32 {
        assert!((0..MAX_WORD_LENGTH as i32).contains(&index));
        let _lock = lock();
        // SAFETY: as primary; the key lookups are bounds checked.
        unsafe { aw_hook_state_primary_original(self.handle, index) }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for State {
    fn drop(&mut self) {
        if self.owned {
            let _lock = lock();
            // SAFETY: the handle came from aw_hook_state_new and is freed only here.
            unsafe { aw_hook_state_free(self.handle) }
        }
    }
}
