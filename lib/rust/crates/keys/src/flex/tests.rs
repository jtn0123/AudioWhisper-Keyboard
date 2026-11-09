use super::*;

fn factors(mode: i32, code: i32, numeric: bool) -> (f32, f32, f32) {
    let flex = flex(mode, code, numeric);
    (flex.shrink, flex.grow, flex.width_factor)
}

#[test]
fn character_keys_are_one_wide_and_space_grows() {
    assert_eq!(factors(mode::CHARACTERS, 97, false), (1.0, 0.0, 1.0));
    assert_eq!(
        factors(mode::CHARACTERS, code::SPACE, false),
        (1.0, 1.0, 1.0)
    );
    assert_eq!(
        factors(mode::SYMBOLS, code::CJK_SPACE, true),
        (1.0, 1.0, 1.0)
    );
}

#[test]
fn shift_and_delete_shrink_most_and_mode_keys_not_at_all() {
    assert_eq!(
        factors(mode::CHARACTERS, code::SHIFT, false),
        (1.5, 0.0, 1.56)
    );
    assert_eq!(
        factors(mode::SYMBOLS2, code::DELETE, false),
        (1.5, 0.0, 1.56)
    );
    for code in [
        code::VIEW_CHARACTERS,
        code::VIEW_SYMBOLS,
        code::VIEW_SYMBOLS2,
        code::ENTER,
    ] {
        assert_eq!(factors(mode::CHARACTERS, code, false), (0.0, 0.0, 1.56));
    }
}

#[test]
fn number_pads_are_wide_and_never_grow() {
    for mode in [mode::NUMERIC, mode::PHONE, mode::PHONE2] {
        assert_eq!(factors(mode, code::SPACE, true), (1.0, 0.0, 2.68));
        assert_eq!(factors(mode, code::SHIFT, false), (1.0, 0.0, 2.68));
    }
}

#[test]
fn the_advanced_number_pad_grows_its_numbers() {
    assert_eq!(factors(mode::NUMERIC_ADVANCED, 49, true), (1.0, 1.0, 1.56));
    assert_eq!(
        factors(mode::NUMERIC_ADVANCED, code::SPACE, false),
        (1.0, 0.0, 1.56)
    );
    assert_eq!(
        factors(mode::NUMERIC_ADVANCED, code::COMMA, false),
        (1.0, 0.0, 1.0)
    );
    assert_eq!(
        factors(mode::NUMERIC_ADVANCED, code::PERIOD, true),
        (1.0, 1.0, 1.0)
    );
    assert_eq!(
        factors(mode::NUMERIC_ADVANCED, code::EQUALS, false),
        (1.0, 0.0, 1.26)
    );
    assert_eq!(
        factors(mode::NUMERIC_ADVANCED, code::VIEW_SYMBOLS, false),
        (1.0, 0.0, 1.26)
    );
}

#[test]
fn unknown_modes_are_treated_as_characters() {
    for mode in [-1, 1, 8, 9, 10, i32::MIN, i32::MAX] {
        assert_eq!(factors(mode, code::SHIFT, true), (1.5, 0.0, 1.56));
        assert_eq!(factors(mode, code::SPACE, true), (1.0, 1.0, 1.0));
    }
}
