use super::*;

const NOTHING: Facts = Facts {
    behavior: Behavior::DoubleTap,
    caps_mode: CapsMode::None,
    shift_pressed: false,
    any_pressed: false,
    consecutive_down: false,
    uninterrupted: false,
    auto_capitalization: false,
    locale_capitalizes: false,
};

const STATES: [State; 4] = [
    State::Unshifted,
    State::ShiftedManual,
    State::ShiftedAutomatic,
    State::CapsLock,
];

const CAPITALISING: Facts = Facts {
    caps_mode: CapsMode::Sentences,
    auto_capitalization: true,
    locale_capitalizes: true,
    ..NOTHING
};

#[test]
fn auto_capitalisation_needs_the_pref_the_language_and_the_editor() {
    assert_eq!(
        next(State::Unshifted, Event::Reevaluate, &CAPITALISING),
        State::ShiftedAutomatic
    );
    for mode in [CapsMode::All, CapsMode::Words] {
        let facts = Facts {
            caps_mode: mode,
            ..CAPITALISING
        };
        assert_eq!(
            next(State::ShiftedManual, Event::Reevaluate, &facts),
            State::ShiftedAutomatic
        );
    }
    for facts in [
        Facts {
            caps_mode: CapsMode::None,
            ..CAPITALISING
        },
        Facts {
            auto_capitalization: false,
            ..CAPITALISING
        },
        Facts {
            locale_capitalizes: false,
            ..CAPITALISING
        },
    ] {
        assert_eq!(
            next(State::ShiftedAutomatic, Event::Reevaluate, &facts),
            State::Unshifted
        );
    }
}

#[test]
fn reevaluating_leaves_caps_lock_and_a_held_shift() {
    assert_eq!(
        next(State::CapsLock, Event::Reevaluate, &NOTHING),
        State::CapsLock
    );
    let held = Facts {
        shift_pressed: true,
        ..CAPITALISING
    };
    for state in STATES {
        assert_eq!(next(state, Event::Reevaluate, &held), state);
    }
}

#[test]
fn a_double_tap_locks_and_a_single_press_toggles() {
    let double = Facts {
        consecutive_down: true,
        ..NOTHING
    };
    for state in STATES {
        assert_eq!(next(state, Event::ShiftDown, &double), State::CapsLock);
    }
    assert_eq!(
        next(State::Unshifted, Event::ShiftDown, &NOTHING),
        State::ShiftedManual
    );
    for state in [
        State::ShiftedManual,
        State::ShiftedAutomatic,
        State::CapsLock,
    ] {
        assert_eq!(next(state, Event::ShiftDown, &NOTHING), State::Unshifted);
    }
}

#[test]
fn cycling_steps_through_shift_and_lock_and_ignores_double_taps() {
    for consecutive_down in [false, true] {
        let cycle = Facts {
            behavior: Behavior::Cycle,
            consecutive_down,
            ..NOTHING
        };
        let stepped = STATES.map(|state| next(state, Event::ShiftDown, &cycle));
        assert_eq!(
            stepped,
            [
                State::ShiftedManual,
                State::CapsLock,
                State::Unshifted,
                State::Unshifted
            ]
        );
    }
}

#[test]
fn releasing_shift_unshifts_only_after_other_keys_and_with_none_down() {
    let after_others = NOTHING;
    assert_eq!(
        next(State::ShiftedManual, Event::ShiftUp, &after_others),
        State::Unshifted
    );
    assert_eq!(
        next(State::CapsLock, Event::ShiftUp, &after_others),
        State::CapsLock
    );
    for facts in [
        Facts {
            any_pressed: true,
            ..NOTHING
        },
        Facts {
            uninterrupted: true,
            ..NOTHING
        },
    ] {
        assert_eq!(
            next(State::ShiftedManual, Event::ShiftUp, &facts),
            State::ShiftedManual
        );
    }
}

#[test]
fn caps_lock_locks_and_a_cancel_unshifts() {
    for state in STATES {
        assert_eq!(next(state, Event::CapsLock, &NOTHING), State::CapsLock);
        assert_eq!(next(state, Event::ShiftCancel, &NOTHING), State::Unshifted);
    }
}

#[test]
fn shift_lasts_one_key_unless_locked_or_held() {
    for state in [State::ShiftedManual, State::ShiftedAutomatic] {
        assert_eq!(next(state, Event::KeyTyped, &NOTHING), State::Unshifted);
        let held = Facts {
            shift_pressed: true,
            ..NOTHING
        };
        assert_eq!(next(state, Event::KeyTyped, &held), state);
    }
    assert_eq!(
        next(State::CapsLock, Event::KeyTyped, &NOTHING),
        State::CapsLock
    );
}

#[test]
fn consecutive_means_the_same_key_within_the_timeout() {
    assert!(is_consecutive(-11, 1_000, -11, 1_299, 300));
    assert!(!is_consecutive(-11, 1_000, -11, 1_300, 300));
    assert!(!is_consecutive(-11, 1_000, 32, 1_001, 300));
    // Earlier than the last counts, as Kotlin's subtraction gives a negative gap.
    assert!(is_consecutive(32, 1_000, 32, 0, 300));
    // The difference wraps as a Long's does.
    assert!(is_consecutive(32, i64::MAX, 32, i64::MIN + 298, 300));
    assert!(!is_consecutive(32, i64::MAX, 32, i64::MIN + 299, 300));
    assert!(is_consecutive(32, i64::MIN, 32, i64::MAX, 300));
}

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

#[test]
fn a_second_space_after_a_word_is_a_full_stop() {
    assert!(is_double_space_period(&units("a ")));
    assert!(is_double_space_period(&units("9\t")));
    assert!(is_double_space_period(&units(", ")));
    for text in [". ", "! ", "? ", "‽ ", "  ", "ab", "a", "", "ab ", " a"] {
        assert!(!is_double_space_period(&units(text)), "{text:?}");
    }
}

#[test]
fn a_surrogate_pair_is_one_character_so_no_space_follows_it() {
    assert!(!is_double_space_period(&units("😀")));
    // Half a pair is a character of its own.
    assert!(is_double_space_period(&[0xD83D, 0x20]));
    assert!(is_double_space_period(&[0xDE00, 0x20]));
}

#[cfg(not(target_os = "android"))]
#[test]
fn on_the_jvm_regex_spaces_are_ascii_only() {
    assert!(!is_double_space_period(&[u16::from(b'a'), 0x00A0]));
    assert!(is_double_space_period(&[0x00A0, 0x20]));
}
