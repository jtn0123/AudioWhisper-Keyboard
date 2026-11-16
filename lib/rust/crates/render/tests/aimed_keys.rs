//! The Rust renderer's hit test with this user's learned aim (`Scene::aims`): a tap near a border
//! types the key both keys' aims say it was meant for, and nothing else changes.

use keys::{Bounds, Rect};
use render::scene::{Key, Scene};
use render::touch::{Action, Motion, Touch};

/// Three keys 50 wide in one row under a 20-high strip.
fn scene() -> Scene {
    let keys = (0..3)
        .map(|i| {
            let r = Rect {
                left: i as f32 * 50.0,
                right: (i + 1) as f32 * 50.0,
                top: 0.0,
                bottom: 60.0,
            };
            Key {
                code: 97 + i,
                bounds: Bounds {
                    touch: r,
                    visible: r,
                },
                label: String::new(),
                hint: String::new(),
                icon: String::new(),
                styles: Default::default(),
                hints: Default::default(),
                enabled: true,
            }
        })
        .collect();
    Scene {
        width: 150,
        height: 80,
        strip: 20.0,
        keys,
        ..Scene::default()
    }
}

/// Every key's taps land 10 to the left of where they mean.
const SHORT: Option<(f32, f32)> = Some((-10.0, 0.0));

#[test]
fn a_tap_near_a_border_types_the_key_meant() {
    let mut scene = scene();
    assert_eq!(scene.aimed_key_at(45.0, 40.0), Some(0), "nothing learned");
    scene.aims = vec![SHORT; 3];
    assert_eq!(scene.aimed_key_at(45.0, 40.0), Some(1));
    assert_eq!(scene.aimed_key_at(95.0, 40.0), Some(2));
    // Away from a border, or past the last key, the tap stays.
    assert_eq!(scene.aimed_key_at(25.0, 40.0), Some(0));
    assert_eq!(scene.aimed_key_at(145.0, 40.0), Some(2));
    // In the strip it is no key at all.
    assert_eq!(scene.aimed_key_at(45.0, 10.0), None);
}

#[test]
fn both_keys_must_agree() {
    let mut scene = scene();
    // Key 1's aim is not known, or its taps land on its centre.
    scene.aims = vec![SHORT];
    assert_eq!(scene.aimed_key_at(45.0, 40.0), Some(0));
    scene.aims = vec![SHORT, Some((0.0, 0.0)), SHORT];
    assert_eq!(scene.aimed_key_at(45.0, 40.0), Some(0));
    // A disabled neighbour is never typed.
    scene.aims = vec![SHORT; 3];
    scene.keys[1].enabled = false;
    assert_eq!(scene.aimed_key_at(45.0, 40.0), Some(0));
}

#[test]
fn a_key_going_down_is_the_aimed_key() {
    let mut scene = scene();
    scene.aims = vec![SHORT; 3];
    let down = Motion {
        action: 0,
        index: 0,
        ids: vec![7],
        times: vec![1, 2],
        coords: vec![45.0, 40.0],
    };
    assert_eq!(Touch::default().event(&mut scene, &down), [Action(0, 1, 7)]);
}
