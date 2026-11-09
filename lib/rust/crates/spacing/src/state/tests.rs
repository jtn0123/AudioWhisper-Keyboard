use super::*;

#[test]
fn setting_a_flag_marks_it_to_survive_one_update() {
    assert_eq!(auto_space_active(true), 0x5);
    assert_eq!(auto_space_active(false), 0x1);
    assert_eq!(phantom_space_active(true, true), 0x7);
    assert_eq!(phantom_space_active(false, true), 0x5);
    assert_eq!(phantom_space_active(true, false), 0x3);
    assert_eq!(phantom_space_active(false, false), 0x1);
}

#[test]
fn the_flags_read_their_bits() {
    for state in 0..8 {
        assert_eq!(is_active(state), state & 1 != 0);
        assert_eq!(shows_composing_region(state), state & 2 != 0);
        assert_eq!(keeps_candidate(state), state & 4 != 0);
    }
    assert!(!is_active(INACTIVE));
}

#[test]
fn an_update_clears_the_mark_then_the_flag() {
    let state = phantom_space_active(true, true);
    let once = after_update(state);
    assert_eq!(once, 0x3);
    assert!(is_active(once) && shows_composing_region(once));
    assert_eq!(after_update(once), INACTIVE);
    assert_eq!(after_update(auto_space_active(false)), INACTIVE);
}

#[test]
fn an_update_keeps_other_bits_as_kotlin_does() {
    assert_eq!(after_update(-1), !0x4);
    assert_eq!(after_update(0x7FFF_FFF8), INACTIVE);
    assert_eq!(after_update(0x7FFF_FFFC), 0x7FFF_FFF8);
}
