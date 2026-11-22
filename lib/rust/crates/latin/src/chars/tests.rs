use super::*;
use crate::golden::{record, tape};

/// Every code point, the beginning-of-sentence marker and a little past it, as the C++ answered.
#[test]
fn every_code_point_matches_the_goldens() {
    use crate::defines::NOT_A_CODE_POINT;
    let mut tape = tape("chars.every_code_point");
    for c in 0..=0x11_0100 {
        record!(tape, to_lower_case(c));
        record!(tape, to_base_code_point(c));
        record!(tape, to_base_lower_case(c));
    }
    for c in [
        NOT_A_CODE_POINT,
        -2,
        -0x41,
        -0x10000,
        i32::MIN,
        i32::MAX,
        0x7FFF_00C0,
    ] {
        record!(tape, to_lower_case(c));
    }
    tape.finish();
}

#[test]
fn lower_cases_by_its_own_table() {
    assert_eq!(to_lower_case('A' as i32), 'a' as i32);
    assert_eq!(to_lower_case('É' as i32), 'é' as i32);
    assert_eq!(to_lower_case('Ж' as i32), 'ж' as i32);
    // The C++ searched by the low 16 bits: U+100C0 comes back as à's small letter.
    assert_eq!(to_lower_case(0x1_00C0), 0xE0);
    assert_eq!(to_base_lower_case('Ê' as i32), 'e' as i32);
    assert_eq!(to_base_code_point(-1), -1);
}

#[test]
fn counts_and_omissions() {
    assert_eq!(code_point_count(&[1, 2, 0, 3]), 2);
    assert_eq!(code_point_count(&[1, 2]), 2);
    assert_eq!(space_count(&[32, 97, 32]), 2);
    assert!(is_intentional_omission_code_point('\'' as i32));
    assert!(is_intentional_omission_code_point('-' as i32));
    assert!(!is_intentional_omission_code_point('a' as i32));
    assert!(is_in_unicode_space(0x10FFFF));
    assert!(!is_in_unicode_space(0x110000));
    assert!(!is_in_unicode_space(-1));
}

#[test]
fn beginning_of_sentence_marker_comes_and_goes() {
    let mut code_points = [97, 98, 0, 0];
    assert_eq!(
        attach_beginning_of_sentence_marker(&mut code_points, 2, 4),
        3
    );
    assert_eq!(code_points[..3], [CODE_POINT_BEGINNING_OF_SENTENCE, 97, 98]);
    assert_eq!(
        attach_beginning_of_sentence_marker(&mut code_points, 3, 4),
        3
    );
    assert_eq!(remove_beginning_of_sentence_marker(&mut code_points, 3), 2);
    assert_eq!(code_points[..2], [97, 98]);
    assert_eq!(remove_beginning_of_sentence_marker(&mut code_points, 2), 2);
    assert_eq!(
        attach_beginning_of_sentence_marker(&mut code_points, 4, 4),
        0
    );
}
