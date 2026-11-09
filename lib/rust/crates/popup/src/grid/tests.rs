use super::*;

/// A rectangle from its left, top, width and height.
fn rect(left: f32, top: f32, width: f32, height: f32) -> Bounds {
    Bounds {
        left,
        top,
        right: left + width,
        bottom: top + height,
    }
}

/// A key 100 wide at `left`, top 200, whose preview popup is 100 by 120.
fn key(left: f32) -> (Bounds, Bounds) {
    (
        rect(left, 200.0, 100.0, 80.0),
        rect(left, 80.0, 100.0, 120.0),
    )
}

#[test]
fn no_keys_no_popup() {
    let (key, base) = key(0.0);
    assert_eq!(extend(0, 0, key, base, 1000.0), None);
    assert_eq!(extend(-3, 0, key, base, 1000.0), None);
}

#[test]
fn up_to_five_keys_take_one_row() {
    let (key, base) = key(400.0);
    for n in 1..=5 {
        let ext = extend(n, 0, key, base, 1000.0).unwrap();
        assert_eq!((ext.grid.row0_count, ext.grid.row1_count), (n, 0));
        assert_eq!(ext.popup_indices, (0..n).collect::<Vec<_>>());
    }
}

#[test]
fn more_keys_take_two_rows_row0_holding_the_odd_one() {
    let (key, base) = key(400.0);
    let odd = extend(7, 0, key, base, 1000.0).unwrap();
    assert_eq!((odd.grid.row0_count, odd.grid.row1_count), (4, 3));
    let even = extend(8, 0, key, base, 1000.0).unwrap();
    assert_eq!((even.grid.row0_count, even.grid.row1_count), (4, 4));
    assert_eq!((odd.row(2), odd.row(3)), (1, 0));
}

#[test]
fn a_key_in_the_left_half_anchors_left_and_reaches_right_by_what_fits() {
    let (key, base) = key(100.0);
    let ext = extend(5, 0, key, base, 1000.0).unwrap();
    assert!(ext.grid.anchor_left);
    // Half of five is two, but only one key fits to the left.
    assert_eq!(ext.grid.anchor_offset, 1);
    assert_eq!(ext.initial_index, 1);
    assert_eq!(ext.bounds, rect(0.0, 160.0, 500.0, 48.0));
    let (edge, base) = self::key(0.0);
    assert_eq!(
        extend(5, 0, edge, base, 1000.0).unwrap().grid.anchor_offset,
        0
    );
}

#[test]
fn a_key_in_the_right_half_anchors_right() {
    let (key, base) = key(800.0);
    let ext = extend(5, 0, key, base, 1000.0).unwrap();
    assert!(!ext.grid.anchor_left);
    assert_eq!(ext.grid.anchor_offset, 1);
    assert_eq!(ext.initial_index, 3);
    assert_eq!(ext.bounds.left, 500.0);
    assert_eq!(ext.bounds.right, 1000.0);
}

#[test]
fn two_rows_rise_by_row_1s_height_truncated() {
    let (key, base) = (
        rect(400.0, 200.0, 100.0, 80.0),
        rect(400.0, 80.0, 100.0, 121.0),
    );
    let ext = extend(6, 0, key, base, 1000.0).unwrap();
    // 121 * 0.4 is 48.4, truncated to 48.
    assert_eq!(ext.bounds.top, -121.0 - 48.0 + 280.0);
    assert_eq!(ext.bounds.bottom, ext.bounds.top + 121.0 * 0.4 * 2.0);
}

#[test]
fn an_offset_of_0_gives_plus_zero() {
    let (key, base) = (rect(0.0, 0.0, 0.0, 0.0), rect(0.0, 0.0, -0.0, 0.0));
    let ext = extend(1, 0, key, base, 1.0).unwrap();
    assert_eq!(ext.bounds.left.to_bits(), 0.0f32.to_bits());
}

#[test]
fn nan_bounds_still_lay_out() {
    let (key, base) = (rect(f32::NAN, 0.0, 10.0, 10.0), rect(0.0, 0.0, 10.0, 10.0));
    let ext = extend(9, 1, key, base, 100.0).unwrap();
    assert!(!ext.grid.anchor_left);
    assert_eq!(ext.grid.anchor_offset, 0);
    assert!(ext.bounds.left.is_nan());
}

#[test]
fn the_prioritized_keys_sit_at_and_beside_the_initial_place() {
    let (key, base) = key(400.0);
    let one = extend(5, 1, key, base, 1000.0).unwrap();
    assert_eq!(one.popup_indices, [0, 1, FIRST_PRIORITIZED, 2, 3]);
    let two = extend(5, 2, key, base, 1000.0).unwrap();
    assert_eq!(two.popup_indices, [0, 1, -1, -2, 2]);
    let three = extend(5, 3, key, base, 1000.0).unwrap();
    assert_eq!(three.popup_indices, [0, -3, -1, -2, 1]);
    let more = extend(5, 4, key, base, 1000.0).unwrap();
    assert_eq!(more.popup_indices, [0, 1, 2, 3, 4]);
}

#[test]
fn prioritized_keys_go_where_there_is_room() {
    let (left, base) = key(0.0);
    assert_eq!(
        extend(3, 3, left, base, 1000.0).unwrap().popup_indices,
        [-1, -2, -3]
    );
    assert_eq!(
        extend(1, 2, left, base, 1000.0).unwrap().popup_indices,
        [-1]
    );
    assert_eq!(
        extend(2, 3, left, base, 1000.0).unwrap().popup_indices,
        [-1, 0]
    );
    let (right, base) = key(900.0);
    assert_eq!(
        extend(3, 3, right, base, 1000.0).unwrap().popup_indices,
        [-3, -2, -1]
    );
    assert_eq!(
        extend(2, 2, right, base, 1000.0).unwrap().popup_indices,
        [-2, -1]
    );
}

#[test]
fn the_finger_moves_along_row_0() {
    let (key, base) = key(400.0);
    let grid = extend(5, 0, key, base, 1000.0).unwrap().grid;
    assert_eq!(pointer(key, base, grid, 450.0, 230.0), Some(2));
    assert_eq!(pointer(key, base, grid, 350.0, 230.0), Some(1));
    assert_eq!(pointer(key, base, grid, 250.0, 230.0), Some(0));
    assert_eq!(pointer(key, base, grid, 650.0, 230.0), Some(4));
    assert_eq!(pointer(key, base, grid, 450.0, 200.0 + 108.1), None);
    assert_eq!(pointer(key, base, grid, 450.0, 200.0 - 120.1), None);
    assert_eq!(pointer(key, base, grid, 199.0, 230.0), Some(0));
    assert_eq!(pointer(key, base, grid, 99.0, 230.0), None);
    assert_eq!(pointer(key, base, grid, 801.0, 230.0), None);
}

#[test]
fn the_finger_moves_to_row_1_above() {
    let (key, base) = key(400.0);
    let grid = extend(7, 0, key, base, 1000.0).unwrap().grid;
    assert_eq!(pointer(key, base, grid, 450.0, 190.0), Some(1));
    assert_eq!(pointer(key, base, grid, 450.0, 210.0), Some(4));
    let (right, base) = self::key(800.0);
    let grid = extend(7, 0, right, base, 1000.0).unwrap().grid;
    assert!(!grid.anchor_left);
    assert_eq!(pointer(right, base, grid, 850.0, 190.0), Some(1));
    assert_eq!(pointer(right, base, grid, 750.0, 190.0), Some(0));
    assert_eq!(pointer(right, base, grid, 850.0, 210.0), Some(5));
    assert_eq!(pointer(right, base, grid, 650.0, 210.0), Some(3));
}

#[test]
fn odd_grids_wrap_as_kotlins_int_does() {
    let (key, base) = key(400.0);
    let grid = Grid {
        anchor_left: true,
        anchor_offset: i32::MAX - 1,
        row0_count: i32::MAX,
        row1_count: 5,
    };
    assert_eq!(pointer(key, base, grid, 450.0, 230.0), Some(i32::MIN + 3));
}

#[test]
fn elements_count_from_the_last_row() {
    assert_eq!(element_at(&[4, 3], 0), Some((1, 0)));
    assert_eq!(element_at(&[4, 3], 3), Some((0, 0)));
    assert_eq!(element_at(&[4, 3], 6), Some((0, 3)));
    assert_eq!(element_at(&[4, 3], 7), None);
    assert_eq!(element_at(&[4, 3], -1), None);
    assert_eq!(element_at(&[], 0), None);
}
