//! Glide points with their times, for the keyboard's swipe scoring: only when the gesture config
//! asks, so the action stream is otherwise as it was.
use keys::{Bounds, Rect};
use render::{
    gesture_config::GestureConfig,
    protocol::*,
    scene::{Key, Scene},
    touch::{Action, Motion, Touch},
};

fn scene() -> Scene {
    Scene {
        width: 200,
        height: 120,
        strip: 20.0,
        keys: (0..2)
            .map(|i| Key {
                code: 97 + i,
                bounds: Bounds {
                    touch: Rect {
                        left: i as f32 * 100.0,
                        top: 0.0,
                        right: (i + 1) as f32 * 100.0,
                        bottom: 100.0,
                    },
                    visible: Rect::default(),
                },
                label: String::new(),
                hint: String::new(),
                icon: String::new(),
                styles: Default::default(),
                hints: Default::default(),
                enabled: true,
            })
            .collect(),
        ..Scene::default()
    }
}

/// A one-finger event at `xs` (its history, then now) at `times` in ms, down at 5 ms.
fn motion(action: i32, xs: &[f32], times: &[i64]) -> Motion {
    let mut all = vec![5_000_000];
    all.extend(times.iter().map(|t| t * 1_000_000));
    Motion {
        action,
        index: 0,
        ids: vec![0],
        times: all,
        coords: xs.iter().flat_map(|&x| [x, 60.0]).collect(),
    }
}

/// The glide actions of a swipe from "a" across to "b" and up.
fn glide(glide_times: bool) -> Vec<Action> {
    let mut scene = scene();
    let mut touch = Touch::default();
    let values = [
        1.0,
        32.0,
        1.0,
        0.0,
        0.0,
        1.0,
        200.0,
        32.0,
        1900.0,
        0.0,
        f32::from(u8::from(glide_times)),
    ];
    let actions = vec!["NO_ACTION".to_string(); 9];
    let config = GestureConfig::decode(&values, actions).unwrap();
    assert_eq!(config.glide_times, glide_times);
    touch.configure(config);
    let mut out = touch.event(&mut scene, &motion(0, &[50.0], &[5]));
    out.extend(touch.event(&mut scene, &motion(2, &[90.0, 120.0, 150.0], &[13, 17, 21])));
    out.extend(touch.event(&mut scene, &motion(1, &[150.0], &[30])));
    out.retain(|a| [GLIDE_TIME, GLIDE_POINT, GLIDE_COMPLETE].contains(&a.0));
    out
}

#[test]
fn each_glide_point_follows_its_time_when_asked() {
    let timed = glide(true);
    let untimed = glide(false);
    assert!(untimed.iter().any(|a| a.0 == GLIDE_POINT));
    assert!(untimed.iter().all(|a| a.0 != GLIDE_TIME));
    // The same points, each after its time: microseconds since the touch went down.
    let points: Vec<Action> = timed
        .iter()
        .copied()
        .filter(|a| a.0 != GLIDE_TIME)
        .collect();
    assert_eq!(points, untimed);
    let times: Vec<i32> = timed
        .windows(2)
        .filter(|w| w[1].0 == GLIDE_POINT)
        .map(|w| {
            assert_eq!(w[0].0, GLIDE_TIME);
            w[0].1
        })
        .collect();
    assert_eq!(times, vec![0, 8_000, 12_000, 16_000]);
}

#[test]
fn a_config_of_nine_or_ten_values_glides_without_times() {
    let actions = || vec!["NO_ACTION".to_string(); 9];
    let nine = [1.0, 32.0, 1.0, 0.0, 0.0, 1.0, 200.0, 32.0, 1900.0];
    assert!(!GestureConfig::decode(&nine, actions()).unwrap().glide_times);
    let ten = [1.0, 32.0, 1.0, 0.0, 0.0, 1.0, 200.0, 32.0, 1900.0, 1.0];
    assert!(!GestureConfig::decode(&ten, actions()).unwrap().glide_times);
    let twelve = [
        1.0, 32.0, 1.0, 0.0, 0.0, 1.0, 200.0, 32.0, 1900.0, 1.0, 1.0, 1.0,
    ];
    assert!(GestureConfig::decode(&twelve, actions()).is_none());
}
