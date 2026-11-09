//! Where the C++ has undefined behaviour the Rust picks a safe result; these tests pin them.
//! They need no oracle, so they run on Android too.

use super::super::*;
use crate::defines::MAX_PROXIMITY_CHARS_SIZE;

const A: i32 = 'a' as i32;
const B: i32 = 'b' as i32;

/// Two keys side by side, `a` over x 0..50 and `b` over 50..100, on a 2 by 1 grid.
fn two_keys(proximity_chars: &[i32]) -> ProximityInfo {
    ProximityInfo::new(&KeyboardGeometry {
        keyboard_width: 100,
        keyboard_height: 50,
        grid_width: 2,
        grid_height: 1,
        most_common_key_width: 50,
        most_common_key_height: 50,
        proximity_chars,
        key_count: 2,
        key_x_coordinates: &[0, 50],
        key_y_coordinates: &[0, 0],
        key_widths: &[50, 50],
        key_heights: &[50, 50],
        key_char_codes: &[A, B],
    })
    .expect("a valid keyboard")
}

fn grid() -> Vec<i32> {
    let mut grid = vec![0; 2 * MAX_PROXIMITY_CHARS_SIZE];
    grid[0] = A;
    grid[MAX_PROXIMITY_CHARS_SIZE] = B;
    grid
}

fn init(
    state: &mut ProximityInfoState,
    info: &ProximityInfo,
    codes: &[i32],
    xs: &[i32],
    ys: &[i32],
) {
    let input = TypingInput {
        codes,
        xs,
        ys,
        times: None,
        pointer_ids: None,
    };
    state.init_input_params(1.0, info, &input, &[]);
}

#[test]
fn a_keyboard_without_a_grid_or_cells_is_an_error() {
    let grid = grid();
    // (grid width, grid height, keyboard width): the last has none.
    for (grid_width, grid_height, keyboard_width) in
        [(0, 1, 100), (2, 0, 100), (-1, 1, 100), (2, 1, 0)]
    {
        let geometry = KeyboardGeometry {
            keyboard_width,
            keyboard_height: 50,
            grid_width,
            grid_height,
            most_common_key_width: 50,
            most_common_key_height: 50,
            proximity_chars: &grid,
            key_count: 0,
            key_x_coordinates: &[],
            key_y_coordinates: &[],
            key_widths: &[],
            key_heights: &[],
            key_char_codes: &[],
        };
        let result = ProximityInfo::new(&geometry);
        assert!(
            result.is_err(),
            "{grid_width} x {grid_height}, keyboard width {keyboard_width}"
        );
    }
}

#[test]
fn a_short_grid_or_key_list_is_an_error() {
    let mut geometry = KeyboardGeometry {
        keyboard_width: 100,
        keyboard_height: 50,
        grid_width: 2,
        grid_height: 1,
        most_common_key_width: 50,
        most_common_key_height: 50,
        proximity_chars: &[A; 3],
        key_count: 0,
        key_x_coordinates: &[],
        key_y_coordinates: &[],
        key_widths: &[],
        key_heights: &[],
        key_char_codes: &[],
    };
    assert!(ProximityInfo::new(&geometry).is_err());
    let grid = grid();
    geometry.proximity_chars = &grid;
    geometry.key_count = 2;
    assert!(ProximityInfo::new(&geometry).is_err());
    geometry.key_count = -1;
    assert!(ProximityInfo::new(&geometry).is_err());
}

#[test]
fn lookups_outside_the_keyboard_give_harmless_values() {
    let info = two_keys(&grid());
    for key in [-1, -5, 2, 3, i32::MAX, i32::MIN] {
        assert_eq!(
            info.get_code_point_of(key),
            crate::defines::NOT_A_CODE_POINT
        );
        assert_eq!(
            info.get_original_code_point_of(key),
            crate::defines::NOT_A_CODE_POINT
        );
        assert_eq!(info.get_key_center_x_of_key_id_g(key, 7), 0);
        assert_eq!(info.get_key_center_y_of_key_id_g(key, 9), 0);
    }
    assert_eq!(info.get_key_center_x_of_key_id_g(1, 0), 75);
    assert_eq!(info.get_key_index_of(B), 1);
    assert_eq!(info.get_key_index_of('B' as i32), 1);
    assert_eq!(
        info.get_key_index_of(crate::defines::NOT_A_CODE_POINT),
        crate::defines::NOT_AN_INDEX
    );
    assert!(!info.has_space_proximity(-1, 5));
    assert!(!info.has_space_proximity(5, -1));
    assert!(!info.has_space_proximity(100_000, 5));
    assert!(!info.has_space_proximity(5, 100_000));
    assert!(!info.has_space_proximity(i32::MAX, i32::MAX));
}

#[test]
fn an_empty_input_leaves_an_empty_state() {
    let info = two_keys(&grid());
    let mut state = ProximityInfoState::new();
    init(&mut state, &info, &[], &[], &[]);
    assert_eq!(state.size(), 0);
    assert!(!state.is_used());
    assert_eq!(state.get_primary_code_point_at(0), 0);
    assert!(!state.has_space_proximity(&info, 0));
}

#[test]
fn indexes_outside_the_state_give_harmless_values() {
    let info = two_keys(&grid());
    let mut state = ProximityInfoState::new();
    init(&mut state, &info, &[A, B], &[10, 60], &[10, 10]);
    assert_eq!(state.size(), 2);
    for index in [-1, 2, 47, 48, 49, 1000, i32::MAX, i32::MIN] {
        let _ = state.proximity_code_points_at(index);
        let _ = state.get_primary_code_point_at(index);
        let _ = state.get_primary_original_code_point_at(&info, index);
        let _ = state.exists_code_point_in_proximity_at(index, A);
        let _ = state.exists_adjacent_proximity_chars(index);
        let _ = state.get_input_x(index);
        let _ = state.get_input_y(index);
        let _ = state.get_input_index_of_sampled_point(index);
        assert!(!state.has_space_proximity(&info, index) || index < 2);
        for key in [-1, 0, 1, 2, i32::MAX, i32::MIN] {
            let _ = state.get_point_to_key_by_id_length(&info, index, key);
        }
    }
}

#[test]
fn short_coordinate_slices_do_not_panic() {
    let info = two_keys(&grid());
    let mut state = ProximityInfoState::new();
    init(&mut state, &info, &[A, B, A], &[10], &[]);
    let input = TypingInput {
        codes: &[A, B],
        xs: &[10, 60],
        ys: &[10, 10],
        times: Some(&[1]),
        pointer_ids: Some(&[]),
    };
    state.init_input_params(1.0, &info, &input, &[]);
    assert!(state.size() <= 2);
}

#[test]
fn extreme_coordinates_do_not_panic() {
    let info = two_keys(&grid());
    let mut state = ProximityInfoState::new();
    let extremes = [i32::MIN, i32::MIN + 1, -1, 0, i32::MAX - 1, i32::MAX];
    for &x in &extremes {
        for &y in &extremes {
            init(
                &mut state,
                &info,
                &[A, B, 'q' as i32],
                &[x, y, x],
                &[y, x, y],
            );
            for key in -1..2 {
                let _ = info.get_normalized_squared_distance_from_center_float_g(key, x, y);
            }
        }
    }
}

#[test]
fn a_word_longer_than_the_state_is_cut_to_it() {
    let info = two_keys(&grid());
    let mut state = ProximityInfoState::new();
    let codes = vec![A; 80];
    let (xs, ys) = (vec![10; 80], vec![10; 80]);
    init(&mut state, &info, &codes, &xs, &ys);
    assert!(state.size() as usize <= crate::defines::MAX_WORD_LENGTH);
}
