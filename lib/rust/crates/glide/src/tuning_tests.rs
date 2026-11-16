use super::*;

fn line(points: usize, step: f32) -> Gesture {
    let mut gesture = Gesture::new();
    for i in 0..points {
        gesture.push(i as f32 * step, 0.0);
    }
    gesture
}

#[test]
fn the_original_tuning_adds_nothing() {
    assert!(Tuning::ORIGINAL.is_original());
    assert_eq!(Tuning::ORIGINAL.ends, 0.0);
    assert_eq!(Tuning::ORIGINAL.dwell, 0.0);
    assert_eq!(Tuning::ORIGINAL.reach, 0.0);
}

#[test]
fn ends_on_their_keys_cost_nothing_and_off_them_cost_more_the_further() {
    let swipe = line(5, 10.0);
    let on = ends_probability(1.0, &swipe, (0.0, 0.0), (40.0, 0.0), 20.0);
    let near = ends_probability(1.0, &swipe, (5.0, 0.0), (40.0, 0.0), 20.0);
    let far = ends_probability(1.0, &swipe, (10.0, 0.0), (40.0, 0.0), 20.0);
    assert_eq!(on, 1.0);
    // 5 px is half a spread of 10 px: e^-(0.5² / 2).
    assert!((near - (-0.125f32).exp()).abs() < 1e-6, "{near}");
    assert!(far < near);
    assert_eq!(
        ends_probability(0.0, &swipe, (99.0, 0.0), (0.0, 9.0), 20.0),
        1.0
    );
}

#[test]
fn a_steady_swipe_weighs_every_point_alike() {
    let swipe = line(11, 30.0);
    let times: Vec<f32> = (0..11).map(|i| i as f32 * 15.0).collect();
    let weights = dwell_weights(1.0, &swipe, &times, 20.0);
    assert_eq!(weights.len(), SAMPLES);
    assert!(
        weights.iter().all(|&w| (w - 1.0).abs() < 1e-4),
        "{weights:?}"
    );
}

#[test]
fn slow_stretches_weigh_more_and_fast_ones_less() {
    // Ten 30 px steps, the sixth ten times slower than the rest.
    let swipe = line(11, 30.0);
    let mut times = vec![0.0f32];
    for i in 1..11 {
        times.push(times[i - 1] + if i == 6 { 150.0 } else { 15.0 });
    }
    let weights = dwell_weights(1.0, &swipe, &times, 20.0);
    // The slow step is from 150 to 180 px: samples 100 to 120 of 200.
    assert!(weights[110] > 2.0, "{}", weights[110]);
    assert!(weights[20] < 1.0, "{}", weights[20]);
    assert!(weights.iter().all(|&w| (0.25..=4.0).contains(&w)));
    // Raised to a power, and clamped.
    let strong = dwell_weights(3.0, &swipe, &times, 20.0);
    assert_eq!(strong[110], DWELL_LIMIT);
}

#[test]
fn unusable_times_weigh_every_point_alike() {
    let swipe = line(4, 30.0);
    let ones = vec![1.0f32; SAMPLES];
    assert_eq!(dwell_weights(1.0, &swipe, &[], 20.0), ones);
    assert_eq!(dwell_weights(1.0, &swipe, &[0.0, 1.0, 2.0], 20.0), ones);
    assert_eq!(
        dwell_weights(1.0, &swipe, &[0.0, 5.0, 4.0, 9.0], 20.0),
        ones
    );
    assert_eq!(dwell_weights(1.0, &swipe, &[3.0; 4], 20.0), ones);
    assert_eq!(
        dwell_weights(1.0, &swipe, &[0.0, 1.0, f32::NAN, 3.0], 20.0),
        ones
    );
    assert_eq!(
        dwell_weights(0.0, &swipe, &[0.0, 1.0, 2.0, 30.0], 20.0),
        ones
    );
    let still = line(4, 0.0);
    assert_eq!(
        dwell_weights(1.0, &still, &[0.0, 1.0, 2.0, 3.0], 20.0),
        ones
    );
}

#[test]
fn equal_weights_give_the_plain_location_distance() {
    let (a, b) = (line(201, 1.0), line(201, 1.5));
    let ones = vec![1.0f32; SAMPLES];
    let plain = {
        let mut total = 0.0f32;
        for i in 0..SAMPLES {
            total += (a.x(i) - b.x(i)).abs() + (a.y(i) - b.y(i)).abs();
        }
        total / SAMPLES as f32 / 2.0
    };
    assert_eq!(weighted_location_distance(&a, &b, &ones), plain);
    let mut late = ones.clone();
    late[150..].iter_mut().for_each(|w| *w = 3.0);
    assert!(weighted_location_distance(&a, &b, &late) > plain);
}

fn points(points: &[(f32, f32)]) -> Gesture {
    let mut gesture = Gesture::new();
    for &(x, y) in points {
        gesture.push(x, y);
    }
    gesture
}

#[test]
fn a_point_is_as_far_from_a_line_as_from_its_nearest_part() {
    let line = points(&[(0.0, 0.0), (100.0, 0.0), (100.0, 100.0)]);
    assert_eq!(farthest(&points(&[(50.0, 20.0)]), &line), 20.0);
    assert_eq!(farthest(&points(&[(130.0, 50.0)]), &line), 30.0);
    // Past the end, it is as far as the end.
    assert_eq!(farthest(&points(&[(-30.0, 40.0)]), &line), 50.0);
    // The farthest of several counts; a one-point line is a point.
    assert_eq!(
        farthest(&points(&[(50.0, 5.0), (60.0, -25.0)]), &line),
        25.0
    );
    assert_eq!(
        farthest(&points(&[(3.0, 4.0)]), &points(&[(0.0, 0.0)])),
        5.0
    );
}

#[test]
fn reach_costs_nothing_within_its_slack_and_more_the_further_beyond() {
    // Keys of radius 20: slack is 10 px.
    let path = points(&[(0.0, 0.0), (100.0, 0.0)]);
    let near = points(&[(0.0, 0.0), (50.0, 9.0), (100.0, 0.0)]);
    assert_eq!(reach_probability(1.0, &near, &path, 20.0), 1.0);
    // 30 px out is 1.5 radii, a radius past the slack: e^-(1² / 2).
    let out = points(&[(0.0, 0.0), (50.0, 30.0), (100.0, 0.0)]);
    let cost = reach_probability(1.0, &out, &path, 20.0);
    assert!((cost - (-0.5f32).exp()).abs() < 1e-6, "{cost}");
    let further = points(&[(0.0, 0.0), (50.0, 50.0), (100.0, 0.0)]);
    assert!(reach_probability(1.0, &further, &path, 20.0) < cost);
    // A key of the word's the swipe never nears costs as much as a stray of the swipe.
    let detour = points(&[(0.0, 0.0), (50.0, 30.0), (100.0, 0.0)]);
    let both = reach_probability(1.0, &path, &detour, 20.0);
    assert!((both - cost).abs() < 1e-6, "{both}");
    assert_eq!(reach_probability(0.0, &out, &path, 20.0), 1.0);
    assert_eq!(reach_probability(1.0, &Gesture::new(), &path, 20.0), 1.0);
}
