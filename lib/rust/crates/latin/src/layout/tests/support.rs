//! Shared by the differential tests: keyboards as plain arrays (the app's QWERTY and random
//! ones), and the comparison of a Rust [`ProximityInfo`] and state with the C++'s.

use latin_oracle::hooks::layout::{RawKeyboard, State};
use latin_oracle::queries::Random;

use crate::defines::{MAX_PROXIMITY_CHARS_SIZE, MAX_WORD_LENGTH, ProximityType};
use crate::layout::{KeyboardGeometry, ProximityInfo, ProximityInfoState};

/// A keyboard's arguments to `setProximityInfoNative`, owned.
#[derive(Clone, Debug)]
pub struct Keyboard {
    pub keyboard_width: i32,
    pub keyboard_height: i32,
    pub grid_width: i32,
    pub grid_height: i32,
    pub most_common_key_width: i32,
    pub most_common_key_height: i32,
    pub proximity_chars: Vec<i32>,
    pub key_count: i32,
    pub key_xs: Vec<i32>,
    pub key_ys: Vec<i32>,
    pub key_widths: Vec<i32>,
    pub key_heights: Vec<i32>,
    pub key_codes: Vec<i32>,
}

impl Keyboard {
    pub fn geometry(&self) -> KeyboardGeometry<'_> {
        KeyboardGeometry {
            keyboard_width: self.keyboard_width,
            keyboard_height: self.keyboard_height,
            grid_width: self.grid_width,
            grid_height: self.grid_height,
            most_common_key_width: self.most_common_key_width,
            most_common_key_height: self.most_common_key_height,
            proximity_chars: &self.proximity_chars,
            key_count: self.key_count,
            key_x_coordinates: &self.key_xs,
            key_y_coordinates: &self.key_ys,
            key_widths: &self.key_widths,
            key_heights: &self.key_heights,
            key_char_codes: &self.key_codes,
        }
    }

    pub fn raw(&self) -> RawKeyboard<'_> {
        RawKeyboard {
            keyboard_width: self.keyboard_width,
            keyboard_height: self.keyboard_height,
            grid_width: self.grid_width,
            grid_height: self.grid_height,
            most_common_key_width: self.most_common_key_width,
            most_common_key_height: self.most_common_key_height,
            proximity_chars: &self.proximity_chars,
            key_count: self.key_count,
            key_xs: &self.key_xs,
            key_ys: &self.key_ys,
            key_widths: &self.key_widths,
            key_heights: &self.key_heights,
            key_codes: &self.key_codes,
        }
    }

    pub fn cell_width(&self) -> i32 {
        (self.keyboard_width + self.grid_width - 1) / self.grid_width
    }

    pub fn cell_height(&self) -> i32 {
        (self.keyboard_height + self.grid_height - 1) / self.grid_height
    }

    /// The points whose grid cell is inside the grid: from `-margin` to the end of the last cell.
    pub fn x_range(&self, margin: i32) -> std::ops::Range<i32> {
        -margin..self.cell_width() * self.grid_width
    }

    pub fn y_range(&self, margin: i32) -> std::ops::Range<i32> {
        -margin..self.cell_height() * self.grid_height
    }
}

/// The app's QWERTY model (`suggest::qwerty`): three staggered rows of 100 by 150 keys on a
/// 1000 by 450 keyboard, a 32 by 16 grid listing the keys whose edge is within 1.2 key widths of
/// each cell's centre (AOSP's `computeNearestNeighbors`).
pub fn qwerty() -> Keyboard {
    const KEY_WIDTH: i32 = 100;
    const KEY_HEIGHT: i32 = 150;
    const GRID_WIDTH: i32 = 32;
    const GRID_HEIGHT: i32 = 16;
    const WIDTH: i32 = 10 * KEY_WIDTH;
    const HEIGHT: i32 = 3 * KEY_HEIGHT;
    let rows = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];
    let offsets = [0, KEY_WIDTH / 2, KEY_WIDTH * 3 / 2];
    let mut keyboard = Keyboard {
        keyboard_width: WIDTH,
        keyboard_height: HEIGHT,
        grid_width: GRID_WIDTH,
        grid_height: GRID_HEIGHT,
        most_common_key_width: KEY_WIDTH,
        most_common_key_height: KEY_HEIGHT,
        proximity_chars: Vec::new(),
        key_count: 0,
        key_xs: Vec::new(),
        key_ys: Vec::new(),
        key_widths: Vec::new(),
        key_heights: Vec::new(),
        key_codes: Vec::new(),
    };
    for (row, letters) in rows.iter().enumerate() {
        for (column, letter) in letters.chars().enumerate() {
            keyboard
                .key_xs
                .push(offsets[row] + column as i32 * KEY_WIDTH);
            keyboard.key_ys.push(row as i32 * KEY_HEIGHT);
            keyboard.key_codes.push(letter as i32);
        }
    }
    keyboard.key_count = keyboard.key_codes.len() as i32;
    keyboard.key_widths = vec![KEY_WIDTH; keyboard.key_codes.len()];
    keyboard.key_heights = vec![KEY_HEIGHT; keyboard.key_codes.len()];
    let (cell_width, cell_height) = (keyboard.cell_width(), keyboard.cell_height());
    let threshold = (KEY_WIDTH as f32 * 1.2) as i32;
    for cell in 0..GRID_WIDTH * GRID_HEIGHT {
        let center_x = (cell % GRID_WIDTH) * cell_width + cell_width / 2;
        let center_y = (cell / GRID_WIDTH) * cell_height + cell_height / 2;
        let mut slots = Vec::new();
        for key in 0..keyboard.key_codes.len() {
            let (x, y) = (keyboard.key_xs[key], keyboard.key_ys[key]);
            let dx = 0.max(x - center_x).max(center_x - (x + KEY_WIDTH));
            let dy = 0.max(y - center_y).max(center_y - (y + KEY_HEIGHT));
            if dx * dx + dy * dy < threshold * threshold {
                slots.push(keyboard.key_codes[key]);
            }
        }
        slots.resize(MAX_PROXIMITY_CHARS_SIZE, -1);
        keyboard.proximity_chars.extend(slots);
    }
    keyboard
}

/// Code points that exercise case, accents, the delimiters and the odd ones.
const CODE_POOL: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZéÉßñÑàâèïöüçÿ1 '-.,\u{0}\u{2}\u{1f}";

pub fn pool_code(random: &mut Random) -> i32 {
    let pool: Vec<char> = CODE_POOL.chars().collect();
    match random.below(40) {
        0 => 0x11_0000,
        1 => 0x1F600,
        2 => 0x3A9,
        _ => *random.pick(&pool) as i32,
    }
}

/// A random valid keyboard: any size and grid, up to 80 keys (the C++ keeps 64), keys of any
/// size and position (some wider than the most common), codes with duplicates and odd ones, and
/// a grid with any contents.
pub fn random_keyboard(random: &mut Random) -> Keyboard {
    let keyboard_width = 1 + random.below(1500) as i32;
    let keyboard_height = 1 + random.below(800) as i32;
    let grid_width = 1 + random.below(36) as i32;
    let grid_height = 1 + random.below(24) as i32;
    let most_common_key_width = random.below(260) as i32 - 3;
    let key_count = match random.below(10) {
        0 => 0,
        1 => 60 + random.below(21),
        _ => 1 + random.below(40),
    };
    let key_codes: Vec<i32> = (0..key_count)
        .map(|_| match random.below(30) {
            0 => -1,
            _ => pool_code(random),
        })
        .collect();
    let key = |random: &mut Random, limit: i32| random.below(limit as usize + 100) as i32 - 50;
    let key_xs = (0..key_count)
        .map(|_| key(random, keyboard_width))
        .collect();
    let key_ys = (0..key_count)
        .map(|_| key(random, keyboard_height))
        .collect();
    let size = |random: &mut Random| -> Vec<i32> {
        (0..key_count)
            .map(|_| match random.below(8) {
                0 => random.below(400) as i32,
                1 => 0,
                _ => (most_common_key_width + random.below(20) as i32 - 10).max(0),
            })
            .collect()
    };
    let key_widths = size(random);
    let key_heights = size(random);
    let cells = (grid_width * grid_height) as usize;
    let mut proximity_chars = Vec::with_capacity(cells * MAX_PROXIMITY_CHARS_SIZE);
    for _ in 0..cells {
        let listed = match random.below(6) {
            0 => MAX_PROXIMITY_CHARS_SIZE,
            1 => 0,
            _ => random.below(7),
        };
        for slot in 0..MAX_PROXIMITY_CHARS_SIZE {
            proximity_chars.push(if slot >= listed {
                if random.chance(10) { 0 } else { -1 }
            } else if key_count > 0 && random.chance(85) {
                key_codes[random.below(key_count)]
            } else {
                pool_code(random)
            });
        }
    }
    Keyboard {
        keyboard_width,
        keyboard_height,
        grid_width,
        grid_height,
        most_common_key_width,
        most_common_key_height: random.below(300) as i32,
        proximity_chars,
        key_count: key_count as i32,
        key_xs,
        key_ys,
        key_widths,
        key_heights,
        key_codes,
    }
}

/// Floats equal to the bit.
#[track_caller]
pub fn assert_bits(ours: f32, theirs: f32, what: &dyn std::fmt::Debug) {
    assert_eq!(
        ours.to_bits(),
        theirs.to_bits(),
        "{ours} vs {theirs} for {what:?}"
    );
}

/// The code point types to ask the states about: the typed ones, their cases and accents, and
/// some from the pool.
pub fn candidates(random: &mut Random, typed: &[i32]) -> Vec<i32> {
    let mut codes = vec![-1, 0, 0x2, 0x20];
    for &code in typed {
        codes.push(code);
        codes.push(crate::chars::to_lower_case(code));
        codes.push(crate::chars::to_base_lower_case(code));
        if (b'a'.into()..=b'z'.into()).contains(&code) {
            codes.push(code - 32);
        }
    }
    codes.extend((0..10).map(|_| pool_code(random)));
    codes.retain(|&it| it >= -1);
    codes.sort_unstable();
    codes.dedup();
    codes
}

/// A word to test `sameAsTyped` with: what was typed, or a variation of it.
fn word_like(random: &mut Random, typed: &[i32]) -> Vec<i32> {
    let mut word = typed.to_vec();
    match random.below(6) {
        0 | 1 => {}
        2 if !word.is_empty() => {
            word.pop();
        }
        3 if !word.is_empty() => {
            let at = random.below(word.len());
            word[at] = pool_code(random);
        }
        4 if word.len() < MAX_WORD_LENGTH => word.push(pool_code(random)),
        _ => word = (0..random.below(8)).map(|_| pool_code(random)).collect(),
    }
    word
}

/// Everything the typing search reads from the state, the same in both.
pub fn assert_states_match(
    random: &mut Random,
    ours: &ProximityInfoState,
    info: &ProximityInfo,
    theirs: &State,
    typed: &[i32],
    context: &dyn std::fmt::Debug,
) {
    let summary = theirs.summary();
    assert_eq!(ours.size(), summary.size, "size for {context:?}");
    assert_eq!(ours.is_used(), summary.is_used, "is_used for {context:?}");
    assert_eq!(
        ours.touch_position_correction_enabled(),
        summary.touch_position_correction_enabled,
        "touch position correction for {context:?}"
    );
    assert_eq!(
        ours.is_continuous_suggestion_possible(),
        summary.is_continuous_suggestion_possible,
        "continuous suggestion for {context:?}"
    );
    let size = summary.size;
    let proximities = theirs.proximities();
    for index in 0..MAX_WORD_LENGTH as i32 {
        let row = index as usize * MAX_PROXIMITY_CHARS_SIZE;
        assert_eq!(
            ours.proximity_code_points_at(index),
            &proximities[row..row + MAX_PROXIMITY_CHARS_SIZE],
            "proximity code points at {index} for {context:?}"
        );
        assert_eq!(
            ours.get_primary_code_point_at(index),
            theirs.primary(index),
            "primary at {index} for {context:?}"
        );
        assert_eq!(
            ours.get_primary_original_code_point_at(info, index),
            theirs.primary_original(index),
            "primary original at {index} for {context:?}"
        );
    }
    for index in 0..size {
        let (x, y, from) = theirs.point(index);
        assert_eq!(
            (
                ours.get_input_x(index),
                ours.get_input_y(index),
                ours.get_input_index_of_sampled_point(index)
            ),
            (x, y, from),
            "point {index} for {context:?}"
        );
        // The C++ reads past its grid for a tap beyond the keyboard: no value to compare.
        let start = crate::layout::proximity_info_utils::get_start_index_from_coordinates(
            x,
            y,
            info.get_cell_height(),
            info.get_cell_width(),
            info.get_grid_width(),
        );
        if x < 0 || y < 0 || info.proximity_chars_at(start).is_some() {
            assert_eq!(
                ours.has_space_proximity(info, index),
                theirs.has_space_proximity(index),
                "space proximity at {index} for {context:?}"
            );
        }
        for key_id in -1..info.get_key_count() {
            assert_bits(
                ours.get_point_to_key_by_id_length(info, index, key_id),
                theirs.point_to_key_by_id_length(index, key_id, info.get_key_count()),
                &(index, key_id, context),
            );
        }
    }
    for index in -1..=size + 1 {
        assert_eq!(
            ours.exists_adjacent_proximity_chars(index),
            theirs.exists_adjacent(index),
            "adjacent at {index} for {context:?}"
        );
    }
    let codes = candidates(random, typed);
    for index in 0..=size.min(MAX_WORD_LENGTH as i32 - 1) {
        for &c in &codes {
            assert_eq!(
                ours.exists_code_point_in_proximity_at(index, c),
                theirs.exists_in_proximity(index, c),
                "exists {c:#x} at {index} for {context:?}"
            );
            if index < size {
                assert_bits(
                    ours.get_point_to_key_length(info, index, c),
                    theirs.point_to_key_length(index, c),
                    &(index, c, context),
                );
            }
            if c < 0 {
                continue;
            }
            for check in [true, false] {
                let mut found = None;
                let mut slot = i32::MIN;
                let ours_kind = ours.get_proximity_type(index, c, check, Some(&mut slot));
                if slot != i32::MIN {
                    found = Some(slot);
                }
                let (kind, theirs_found) = theirs.proximity_type(index, c, check);
                assert_eq!(
                    (kind_number(ours_kind), found),
                    (kind, theirs_found),
                    "proximity type of {c:#x} at {index} (check {check}) for {context:?}"
                );
                assert_eq!(ours_kind, ours.get_proximity_type(index, c, check, None));
            }
        }
    }
    for _ in 0..4 {
        let word = word_like(random, typed);
        assert_eq!(
            ours.same_as_typed(&word),
            theirs.same_as_typed(&word),
            "same as typed {word:?} for {context:?}"
        );
    }
    let primaries: Vec<i32> = (0..size)
        .map(|it| ours.get_primary_code_point_at(it))
        .collect();
    assert_eq!(
        theirs.same_as_typed(&primaries),
        ours.same_as_typed(&primaries)
    );
}

/// The C++ enum's value of a [`ProximityType`].
pub fn kind_number(kind: ProximityType) -> i32 {
    match kind {
        ProximityType::MatchChar => 0,
        ProximityType::ProximityChar => 1,
        ProximityType::AdditionalProximityChar => 2,
        ProximityType::SubstitutionChar => 3,
        ProximityType::UnrelatedChar => 4,
    }
}
