use super::*;

/// The swipe distance preference's default, in dp: a unit is 8 dp.
const DISTANCE: i32 = 32;
/// The swipe velocity preference's default, in dp a second.
const VELOCITY: i32 = 1900;

fn swipe(direction: Direction, abs: (i32, i32), rel: (i32, i32)) -> Option<Swipe> {
    Some(Swipe {
        direction,
        abs_x: abs.0,
        abs_y: abs.1,
        rel_x: rel.0,
        rel_y: rel.1,
    })
}

#[test]
fn directions_are_eighths_of_the_circle_clockwise_from_right() {
    assert_eq!(direction(1.0, 0.0), Direction::Right);
    assert_eq!(direction(1.0, 1.0), Direction::DownRight);
    assert_eq!(direction(0.0, 1.0), Direction::Down);
    assert_eq!(direction(-1.0, 1.0), Direction::DownLeft);
    assert_eq!(direction(-1.0, 0.0), Direction::Left);
    assert_eq!(direction(-1.0, -1.0), Direction::UpLeft);
    assert_eq!(direction(0.0, -1.0), Direction::Up);
    assert_eq!(direction(1.0, -1.0), Direction::UpRight);
    assert_eq!(direction(1.0, -0.0), Direction::Right);
    assert_eq!(direction(1.0, -1e-300), Direction::Right);
    assert_eq!(direction(0.0, 0.0), Direction::Right);
    assert_eq!(direction(f64::NAN, 1.0), Direction::Right);
}

#[test]
fn a_sector_takes_its_lower_edge() {
    // tan(22.5°) = √2 - 1: just below the edge is right, just above down-right.
    let tan = std::f64::consts::SQRT_2 - 1.0;
    assert_eq!(direction(1.0, tan * 0.999_999), Direction::Right);
    assert_eq!(direction(1.0, tan * 1.000_001), Direction::DownRight);
    assert_eq!(direction(-1.0, -tan * 0.999_999), Direction::Left);
    assert_eq!(direction(-1.0, -tan * 1.000_001), Direction::UpLeft);
}

#[test]
fn a_move_swipes_past_half_the_distance_since_the_last() {
    let mut swipes = Swipes::default();
    swipes.down(0, 100.0, 100.0, 1.0);
    assert_eq!(swipes.moved(0, 116.0, 100.0, 1.0, DISTANCE, false), None);
    assert_eq!(
        swipes.moved(0, 117.0, 100.0, 1.0, DISTANCE, false),
        swipe(Direction::Right, (2, 0), (2, 0))
    );
    assert_eq!(
        swipes.moved(0, 117.0, 140.0, 1.0, DISTANCE, false),
        swipe(Direction::Down, (2, 5), (0, 5))
    );
}

#[test]
fn always_swipes_at_every_move() {
    let mut swipes = Swipes::default();
    swipes.down(3, 0.0, 0.0, 2.0);
    assert_eq!(
        swipes.moved(3, -2.0, 0.0, 2.0, DISTANCE, true),
        swipe(Direction::Left, (0, 0), (0, 0))
    );
    assert_eq!(swipes.moved(4, -2.0, 0.0, 2.0, DISTANCE, true), None);
}

#[test]
fn positions_are_scaled_by_density() {
    let mut swipes = Swipes::default();
    swipes.down(0, 0.0, 0.0, 2.0);
    assert_eq!(swipes.moved(0, 0.0, -32.0, 2.0, DISTANCE, false), None);
    assert_eq!(
        swipes.moved(0, 0.0, -34.0, 2.0, DISTANCE, false),
        swipe(Direction::Up, (0, -2), (0, -2))
    );
}

#[test]
fn lifting_swipes_if_far_and_fast_enough() {
    let mut swipes = Swipes::default();
    swipes.down(0, 0.0, 0.0, 1.0);
    let fast = (VELOCITY as f32 + 1.0, 0.0);
    assert_eq!(
        swipes.up(0, (33.0, 1.0), fast, 1.0, DISTANCE, VELOCITY),
        swipe(Direction::Right, (4, 0), (4, 0))
    );
    swipes.down(0, 0.0, 0.0, 1.0);
    assert_eq!(
        swipes.up(0, (32.0, 0.0), fast, 1.0, DISTANCE, VELOCITY),
        None
    );
    swipes.down(0, 0.0, 0.0, 1.0);
    let slow = (VELOCITY as f32, -(VELOCITY as f32));
    assert_eq!(
        swipes.up(0, (0.0, 90.0), slow, 1.0, DISTANCE, VELOCITY),
        None
    );
    swipes.down(0, 0.0, 0.0, 1.0);
    let fast_up = (0.0, -(VELOCITY as f32) - 1.0);
    assert_eq!(
        swipes.up(0, (0.0, -90.0), fast_up, 1.0, DISTANCE, VELOCITY),
        swipe(Direction::Up, (0, -11), (0, -11))
    );
}

#[test]
fn lifting_forgets_the_finger() {
    let mut swipes = Swipes::default();
    swipes.down(0, 0.0, 0.0, 1.0);
    swipes.up(0, (0.0, 0.0), (0.0, 0.0), 1.0, DISTANCE, VELOCITY);
    assert_eq!(swipes.moved(0, 100.0, 0.0, 1.0, DISTANCE, true), None);
}

#[test]
fn the_next_finger_in_a_lifted_swipes_slot_counts_from_its_units() {
    let mut swipes = Swipes::default();
    swipes.down(0, 0.0, 0.0, 1.0);
    swipes.up(0, (80.0, 0.0), (5_000.0, 0.0), 1.0, DISTANCE, VELOCITY);
    swipes.down(1, 0.0, 0.0, 1.0);
    assert_eq!(
        swipes.moved(1, 20.0, 0.0, 1.0, DISTANCE, false),
        swipe(Direction::Right, (2, 0), (-8, 0))
    );
    swipes.reset();
    swipes.down(1, 0.0, 0.0, 1.0);
    assert_eq!(
        swipes.moved(1, 20.0, 0.0, 1.0, DISTANCE, false),
        swipe(Direction::Right, (2, 0), (2, 0))
    );
}

#[test]
fn four_fingers_are_followed_and_a_fifth_ignored() {
    let mut swipes = Swipes::default();
    for id in 0..5 {
        swipes.down(id, 0.0, 0.0, 1.0);
    }
    for id in 0..4 {
        assert!(swipes.moved(id, 1.0, 0.0, 1.0, DISTANCE, true).is_some());
    }
    assert_eq!(swipes.moved(4, 1.0, 0.0, 1.0, DISTANCE, true), None);
    swipes.cancel(2);
    swipes.down(4, 0.0, 0.0, 1.0);
    assert!(swipes.moved(4, 1.0, 0.0, 1.0, DISTANCE, true).is_some());
    assert_eq!(swipes.moved(2, 1.0, 0.0, 1.0, DISTANCE, true), None);
}

#[test]
fn a_negative_id_finds_an_unused_slot() {
    let mut swipes = Swipes::default();
    assert_eq!(
        swipes.moved(-1, 0.0, 0.0, 1.0, DISTANCE, true),
        swipe(Direction::Right, (0, 0), (0, 0))
    );
    assert_eq!(swipes.moved(-2, 0.0, 0.0, 1.0, DISTANCE, true), None);
}

#[test]
fn units_saturate_and_nan_is_none() {
    let mut swipes = Swipes::default();
    swipes.down(0, 0.0, 0.0, 1.0);
    assert_eq!(
        swipes.moved(0, f32::MAX, f32::NAN, 1.0, DISTANCE, true),
        swipe(Direction::Right, (i32::MAX, 0), (i32::MAX, 0))
    );
    assert_eq!(
        swipes.moved(0, -f32::MAX, 0.0, 1.0, DISTANCE, true),
        // The last y was NaN, so this move's angle is too.
        swipe(Direction::Right, (i32::MIN, 0), (1, 0))
    );
}

#[test]
fn the_distance_goes_through_a_float() {
    // 2^24 + 3 is no Float: it reads as 2^24 + 4, so a move of 2^23 + 2 dp isn't past half of it.
    let mut swipes = Swipes::default();
    swipes.down(0, 0.0, 0.0, 1.0);
    assert_eq!(
        swipes.moved(0, 8_388_610.0, 0.0, 1.0, 16_777_219, false),
        None
    );
    assert_eq!(threshold_width(16_777_219), 16_777_220.0);
}
