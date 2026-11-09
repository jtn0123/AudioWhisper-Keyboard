use super::*;

/// A key's width in dp.
const KEY: f32 = 36.0;

fn down(glide: &mut GlideStart, first: bool, id: i32, at: (f32, f32), now: i64) -> Outcome {
    glide.touch(Touch::Down { first, id, at }, now, 1.0, None)
}

fn moved(glide: &mut GlideStart, id: i32, points: &[f32], now: i64) -> Outcome {
    glide.touch(Touch::Move { id, points }, now, 1.0, None)
}

#[test]
fn a_quick_long_move_is_a_glide_and_hands_on_every_point() {
    let mut glide = GlideStart::new(KEY);
    let started = down(&mut glide, true, 0, (0.0, 0.0), 1_000);
    assert!(started.cleared_before && started.took && started.started && !started.gesture);
    let first = moved(&mut glide, 0, &[10.0, 0.0, 20.0, 0.0], 1_010);
    assert_eq!(first.emit_from, None);
    assert_eq!(first.decision, None);
    // 37 dp in 21 ms.
    let glided = moved(&mut glide, 0, &[30.0, 0.0, 37.0, 0.0], 1_020);
    assert!(glided.gesture && glided.took);
    assert_eq!(glided.emit_from, Some(0));
    assert_eq!(glided.decision, Some(true));
    let next = moved(&mut glide, 0, &[40.0, 0.0, 41.0, 0.0], 2_000);
    assert_eq!(next.emit_from, Some(5));
    let up = glide.touch(Touch::Up { id: 0 }, 2_000, 1.0, None);
    assert!(up.complete && up.cleared_after && !up.gesture);
    assert_eq!(up.decision, None);
}

#[test]
fn the_point_that_decides_and_those_after_it_are_handed_on() {
    let mut glide = GlideStart::new(KEY);
    down(&mut glide, true, 0, (0.0, 0.0), 0);
    let glided = moved(&mut glide, 0, &[1.0, 0.0, 40.0, 0.0, 50.0, 0.0], 10);
    assert_eq!(glided.emit_from, Some(0));
    moved(&mut glide, 0, &[60.0, 0.0], 10);
    let later = moved(&mut glide, 0, &[70.0, 0.0, 80.0, 0.0], 10);
    assert_eq!(later.emit_from, Some(5));
}

#[test]
fn a_slow_move_is_never_a_glide_after_500_ms() {
    let mut glide = GlideStart::new(KEY);
    down(&mut glide, true, 0, (0.0, 0.0), 0);
    assert_eq!(
        moved(&mut glide, 0, &[1.0, 0.0], MAX_DETECT_TIME - 1).decision,
        None
    );
    let late = moved(&mut glide, 0, &[2.0, 0.0], MAX_DETECT_TIME);
    assert_eq!(late.decision, Some(false));
    let fast = moved(&mut glide, 0, &[500.0, 0.0], MAX_DETECT_TIME);
    assert!(!fast.gesture && fast.emit_from.is_none());
    let up = glide.touch(Touch::Up { id: 0 }, 0, 1.0, None);
    assert!(!up.complete && up.cleared_after);
}

#[test]
fn the_velocity_is_more_than_a_tenth_of_a_dp_a_millisecond() {
    let mut glide = GlideStart::new(KEY);
    down(&mut glide, true, 0, (0.0, 0.0), 0);
    // 40 dp in 400 ms (399 + 1) is 0.1 exactly as a Float, a hair above 0.1 as a Double.
    let tenth = moved(&mut glide, 0, &[40.0, 0.0], 399);
    assert_eq!(tenth.decision, Some(true), "{}", 40.0f32 / 400.0);
}

#[test]
fn swipe_keys_dont_glide() {
    for code in SWIPE_GESTURE_KEYS {
        let mut glide = GlideStart::new(KEY);
        glide.touch(
            Touch::Down {
                first: true,
                id: 0,
                at: (0.0, 0.0),
            },
            0,
            1.0,
            Some(code),
        );
        let outcome = glide.touch(
            Touch::Move {
                id: 0,
                points: &[100.0, 0.0],
            },
            1,
            1.0,
            Some(code),
        );
        assert_eq!(outcome.decision, None, "{code}");
    }
    let mut glide = GlideStart::new(KEY);
    glide.touch(
        Touch::Down {
            first: true,
            id: 0,
            at: (0.0, 0.0),
        },
        0,
        1.0,
        Some('a' as i32),
    );
    let outcome = glide.touch(
        Touch::Move {
            id: 0,
            points: &[100.0, 0.0],
        },
        1,
        1.0,
        Some('a' as i32),
    );
    assert_eq!(outcome.decision, Some(true));
}

#[test]
fn distances_are_in_dp() {
    let mut glide = GlideStart::new(KEY);
    glide.touch(
        Touch::Down {
            first: true,
            id: 0,
            at: (0.0, 0.0),
        },
        0,
        3.0,
        None,
    );
    let outcome = glide.touch(
        Touch::Move {
            id: 0,
            points: &[100.0, 0.0],
        },
        1,
        3.0,
        None,
    );
    assert_eq!(outcome.decision, None);
    let outcome = glide.touch(
        Touch::Move {
            id: 0,
            points: &[0.0, 109.0],
        },
        1,
        3.0,
        None,
    );
    assert_eq!(outcome.decision, Some(true));
}

#[test]
fn a_second_finger_and_others_events_are_ignored() {
    let mut glide = GlideStart::new(KEY);
    down(&mut glide, true, 0, (0.0, 0.0), 0);
    assert_eq!(
        down(&mut glide, false, 1, (5.0, 5.0), 0),
        Outcome::default()
    );
    assert_eq!(moved(&mut glide, 1, &[500.0, 0.0], 1), Outcome::default());
    assert_eq!(
        glide.touch(Touch::Up { id: 1 }, 1, 1.0, None),
        Outcome::default()
    );
    assert_eq!(glide.touch(Touch::Other, 1, 1.0, None), Outcome::default());
    assert!(moved(&mut glide, 0, &[500.0, 0.0], 1).gesture);
    let cancel = glide.touch(Touch::Cancel, 1, 1.0, None);
    assert!(cancel.cancelled && cancel.cleared_after);
    let adopted = down(&mut glide, false, 1, (5.0, 5.0), 2);
    assert!(adopted.took && !adopted.cleared_before);
}

#[test]
fn the_clock_wraps_as_kotlins_long_does() {
    let mut glide = GlideStart::new(KEY);
    down(&mut glide, true, 0, (0.0, 0.0), i64::MAX);
    assert_eq!(
        moved(&mut glide, 0, &[100.0, 0.0], i64::MIN).decision,
        Some(true)
    );
    // A clock gone back a millisecond: no time at all, so an infinite velocity.
    down(&mut glide, true, 0, (0.0, 0.0), 10);
    assert_eq!(moved(&mut glide, 0, &[100.0, 0.0], 9).decision, Some(true));
    // Further back: a negative velocity, never a glide, and never too late.
    down(&mut glide, true, 0, (0.0, 0.0), 10);
    assert_eq!(moved(&mut glide, 0, &[100.0, 0.0], 0).decision, None);
}

#[test]
fn a_touch_reads_androids_actions() {
    let point = [1.0, 2.0];
    assert!(matches!(
        Touch::of(action::DOWN, 4, &point),
        Ok(Touch::Down {
            first: true,
            id: 4,
            at: (1.0, 2.0)
        })
    ));
    assert!(matches!(
        Touch::of(action::POINTER_DOWN, 4, &point),
        Ok(Touch::Down { first: false, .. })
    ));
    assert!(matches!(
        Touch::of(action::MOVE, 4, &point),
        Ok(Touch::Move { id: 4, .. })
    ));
    assert!(matches!(
        Touch::of(action::UP, 4, &[]),
        Ok(Touch::Up { id: 4 })
    ));
    assert!(matches!(
        Touch::of(action::POINTER_UP, 4, &[]),
        Ok(Touch::Up { id: 4 })
    ));
    assert!(matches!(
        Touch::of(action::CANCEL, 4, &[]),
        Ok(Touch::Cancel)
    ));
    assert!(matches!(Touch::of(4, 4, &[]), Ok(Touch::Other)));
    assert!(Touch::of(action::DOWN, 4, &[]).is_err());
    assert!(Touch::of(action::MOVE, 4, &[1.0]).is_err());
}
