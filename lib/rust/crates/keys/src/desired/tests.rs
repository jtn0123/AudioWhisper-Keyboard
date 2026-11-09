use super::*;

#[test]
fn a_tenth_of_the_keyboard_wide_and_a_row_high() {
    let (touch, visible) = desired_key(mode::CHARACTERS, 1080.0, 800.0, 4, 300.0, 4.0, 6.0);
    assert_eq!(
        touch,
        Rect {
            left: 0.0,
            top: 0.0,
            right: 108.0,
            bottom: 200.0
        }
    );
    assert_eq!(
        visible,
        Rect {
            left: 4.0,
            top: 6.0,
            right: 104.0,
            bottom: 194.0
        }
    );
}

#[test]
fn character_rows_are_at_most_a_little_taller_than_the_base() {
    let (touch, _) = desired_key(mode::SYMBOLS2, 1000.0, 1000.0, 4, 100.0, 0.0, 0.0);
    assert_eq!(touch.bottom, 100.0 * 1.12);
    let (touch, _) = desired_key(mode::NUMERIC_ADVANCED, 1000.0, 300.0, 4, 100.0, 0.0, 0.0);
    assert_eq!(touch.bottom, 75.0);
}

#[test]
fn other_modes_take_the_base_row_height() {
    for mode in [mode::NUMERIC, mode::PHONE, mode::PHONE2, -1, 1, 8, 10] {
        let (touch, _) = desired_key(mode, 1000.0, 10.0, 4, 100.0, 0.0, 0.0);
        assert_eq!(touch.bottom, 100.0, "mode {mode}");
    }
}

#[test]
fn no_rows_divide_by_zero_as_floats_do() {
    let (touch, _) = desired_key(mode::CHARACTERS, 1000.0, 300.0, 0, 100.0, 0.0, 0.0);
    assert_eq!(touch.bottom, 112.0);
    let (touch, _) = desired_key(mode::CHARACTERS, 1000.0, 0.0, 0, 100.0, 0.0, 0.0);
    assert!(touch.bottom.is_nan());
}
