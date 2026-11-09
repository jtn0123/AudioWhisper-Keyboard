use super::*;

#[test]
fn a_tap_starts_dictation_and_its_release_leaves_it_on() {
    let mut key = MicKey::default();
    assert_eq!(key.press(Status::Idle, 1_000), Action::Start);
    assert_eq!(key.release(1_000 + HOLD_MS - 1), Action::Nothing);
}

#[test]
fn a_hold_dictates_until_released() {
    let mut key = MicKey::default();
    key.press(Status::Idle, 1_000);
    assert_eq!(key.release(1_000 + HOLD_MS), Action::EndHold);
}

#[test]
fn a_press_while_listening_finishes_on_release_however_long() {
    let mut key = MicKey::default();
    assert_eq!(key.press(Status::Listening, 0), Action::Nothing);
    assert_eq!(key.release(1), Action::Finish);
    key.press(Status::Listening, 0);
    assert_eq!(key.release(HOLD_MS * 10), Action::Finish);
}

#[test]
fn a_press_while_finishing_does_nothing() {
    let mut key = MicKey::default();
    assert_eq!(key.press(Status::Finishing, 0), Action::Nothing);
    assert_eq!(key.release(HOLD_MS * 10), Action::Nothing);
}

#[test]
fn a_release_without_a_press_toggles() {
    let mut key = MicKey::default();
    assert_eq!(key.release(0), Action::Toggle);
    key.press(Status::Idle, 0);
    key.release(0);
    assert_eq!(key.release(0), Action::Toggle);
}

#[test]
fn time_wraps_as_kotlins_long_does() {
    let mut key = MicKey::default();
    key.press(Status::Idle, i64::MAX);
    assert_eq!(key.release(i64::MIN + HOLD_MS), Action::EndHold);
    key.press(Status::Idle, 0);
    assert_eq!(key.release(-1), Action::Nothing);
}

#[test]
fn a_hold_is_300_ms() {
    assert_eq!(HOLD_MS, 300);
}

#[test]
fn a_clock_gone_backwards_is_no_hold() {
    let mut key = MicKey::default();
    key.press(Status::Idle, 10_000);
    assert_eq!(key.release(0), Action::Nothing);
}
