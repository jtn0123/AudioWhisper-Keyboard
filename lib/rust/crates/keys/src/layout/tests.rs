use super::*;

fn rect(left: f32, top: f32, right: f32, bottom: f32) -> Rect {
    Rect {
        left,
        top,
        right,
        bottom,
    }
}

fn key(width_factor: f32, shrink: f32, grow: f32) -> KeyFlex {
    KeyFlex {
        width_factor,
        shrink,
        grow,
    }
}

/// A desired key 100 by 50 whose visible bounds are 5 in from each side.
fn desired() -> (Rect, Rect) {
    (rect(0.0, 0.0, 100.0, 50.0), rect(5.0, 5.0, 95.0, 45.0))
}

#[test]
fn a_full_row_fills_the_keyboard_exactly() {
    let row = [key(1.0, 1.0, 0.0); 10];
    let (touch, visible) = desired();
    let bounds = layout(&[&row], touch, visible, 1010.0, 50.0, false).unwrap();
    assert_eq!(bounds.len(), 10);
    // The row margin is 10, half each side; the end keys' touch bounds reach the edges.
    assert_eq!(bounds[0].touch, rect(0.0, 0.0, 105.0, 50.0));
    assert_eq!(bounds[0].visible, rect(10.0, 5.0, 100.0, 45.0));
    assert_eq!(bounds[1].touch, rect(105.0, 0.0, 205.0, 50.0));
    assert_eq!(bounds[9].touch, rect(905.0, 0.0, 1010.0, 50.0));
    assert_eq!(bounds[9].visible, rect(910.0, 5.0, 1000.0, 45.0));
}

#[test]
fn a_growing_key_takes_the_spare_width() {
    let row = [key(1.0, 1.0, 0.0), key(1.0, 1.0, 1.0), key(1.0, 1.0, 0.0)];
    let (touch, visible) = desired();
    let bounds = layout(&[&row], touch, visible, 510.0, 50.0, false).unwrap();
    assert_eq!(bounds[1].touch, rect(105.0, 0.0, 405.0, 50.0));
    assert_eq!(bounds[2].touch, rect(405.0, 0.0, 510.0, 50.0));
}

#[test]
fn without_growing_keys_the_end_keys_touch_bounds_take_the_spare_width() {
    let row = [key(1.0, 1.0, 0.0), key(1.0, 1.0, 0.0), key(1.0, 1.0, 0.0)];
    let (touch, visible) = desired();
    let bounds = layout(&[&row], touch, visible, 510.0, 50.0, false).unwrap();
    assert_eq!(bounds[0].touch, rect(0.0, 0.0, 205.0, 50.0));
    // Only the touch bounds: the visible ones stay a key wide, at the inner edge.
    assert_eq!(bounds[0].visible, rect(110.0, 5.0, 200.0, 45.0));
    assert_eq!(bounds[1].touch, rect(205.0, 0.0, 305.0, 50.0));
    assert_eq!(bounds[2].touch, rect(305.0, 0.0, 510.0, 50.0));
    assert_eq!(bounds[2].visible, rect(310.0, 5.0, 400.0, 45.0));
}

#[test]
fn an_overfull_row_shrinks_its_shrinking_keys() {
    let row = [key(2.0, 1.0, 0.0), key(2.0, 0.0, 0.0), key(2.0, 3.0, 0.0)];
    let (touch, visible) = desired();
    let bounds = layout(&[&row], touch, visible, 410.0, 50.0, false).unwrap();
    // 2 too wide: the first gives up a quarter of it and the last three quarters.
    assert_eq!(bounds[0].touch, rect(0.0, 0.0, 155.0, 50.0));
    assert_eq!(bounds[1].touch, rect(155.0, 0.0, 355.0, 50.0));
    assert_eq!(bounds[2].touch, rect(355.0, 0.0, 410.0, 50.0));
    assert_eq!(bounds[2].visible, rect(360.0, 5.0, 400.0, 45.0));
}

#[test]
fn rows_are_spread_down_the_keyboard() {
    let row = [key(10.0, 1.0, 0.0)];
    let (touch, visible) = desired();
    let bounds = layout(&[&row, &row, &row], touch, visible, 1010.0, 250.0, true).unwrap();
    // 100 to spare between three rows of 50: 50 between each.
    assert_eq!(bounds[1].touch.top, 100.0);
    assert_eq!(bounds[2].touch.top, 200.0);
    // A single key reaches the left edge but not the right; the last row reaches down a row.
    assert_eq!(bounds[2].touch, rect(0.0, 200.0, 1005.0, 300.0));
    assert_eq!(bounds[1].touch.bottom, 150.0);
}

#[test]
fn nothing_is_laid_out_without_a_desired_key_or_a_size() {
    let row = [key(1.0, 1.0, 0.0)];
    let (touch, visible) = desired();
    assert_eq!(
        layout(&[], touch, visible, 100.0, 100.0, false),
        Some(vec![])
    );
    let empty = rect(0.0, 0.0, 0.0, 50.0);
    assert_eq!(layout(&[&row], empty, visible, 100.0, 100.0, false), None);
    assert_eq!(layout(&[&row], touch, empty, 100.0, 100.0, false), None);
    assert_eq!(
        layout(&[&row], touch, visible, f32::NAN, 100.0, false),
        None
    );
    assert_eq!(
        layout(&[&row], touch, visible, 100.0, f32::NAN, false),
        None
    );
    // A NaN edge isn't empty, so that lays out.
    let nan = rect(0.0, 0.0, f32::NAN, 50.0);
    assert!(layout(&[&row], nan, visible, 100.0, 100.0, false).is_some());
    assert_eq!(
        layout(&[&[]], touch, visible, 100.0, 100.0, false),
        Some(vec![])
    );
}

#[test]
fn a_touch_finds_the_first_key_holding_it() {
    let a = [rect(0.0, 0.0, 10.0, 10.0), rect(10.0, 0.0, 20.0, 10.0)];
    let b = [rect(0.0, 5.0, 20.0, 20.0)];
    let rows: [&[Rect]; 2] = [&a, &b];
    assert_eq!(key_at(&rows, 0.0, 0.0), Some(0));
    assert_eq!(key_at(&rows, 10.0, 9.0), Some(1));
    assert_eq!(key_at(&rows, 5.0, 7.0), Some(0));
    assert_eq!(key_at(&rows, 5.0, 10.0), Some(2));
    assert_eq!(key_at(&rows, 20.0, 5.0), None);
    assert_eq!(key_at(&rows, 5.0, 20.0), None);
    assert_eq!(key_at(&rows, f32::NAN, 5.0), None);
}

#[test]
fn the_search_stops_at_a_row_without_keys() {
    let a = [rect(0.0, 0.0, 10.0, 10.0)];
    let rows: [&[Rect]; 3] = [&a, &[], &a];
    assert_eq!(key_at(&rows, 5.0, 5.0), Some(0));
    let rows: [&[Rect]; 2] = [&[], &a];
    assert_eq!(key_at(&rows, 5.0, 5.0), None);
}
