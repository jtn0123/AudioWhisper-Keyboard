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

/// Key 1's letter is far likelier to come next than key 0's.
fn likelier_middle() -> Vec<Option<f32>> {
    vec![Some(0.01f32.ln()), Some(0.9f32.ln()), Some(0.09f32.ln())]
}

#[test]
fn a_tap_near_a_likelier_neighbour_types_it() {
    let mut scene = scene();
    // 3 px (0.06 key) inside key 0: the border moves at most 0.15 key, 7.5 px.
    assert_eq!(scene.aimed_key_at(47.0, 40.0), Some(0), "no odds");
    scene.priors = likelier_middle();
    assert_eq!(scene.aimed_key_at(47.0, 40.0), Some(1));
    // Further inside than the cap, or towards a less likely key, the tap stays.
    assert_eq!(scene.aimed_key_at(42.0, 40.0), Some(0));
    assert_eq!(scene.aimed_key_at(53.0, 40.0), Some(1));
    // Key 2 is less likely than key 1, so its side of their border is key 1's too.
    assert_eq!(scene.aimed_key_at(103.0, 40.0), Some(1));
    // The strip is still no key.
    assert_eq!(scene.aimed_key_at(47.0, 10.0), None);
}

#[test]
fn the_learned_aim_goes_before_the_odds() {
    let mut scene = scene();
    scene.priors = likelier_middle();
    // Taps for every key land 10 to the right: a tap at 55 means 45, key 0, though key 1 is
    // likelier; measured from 45 (0.1 key inside key 0) the odds can't reach it.
    scene.aims = vec![Some((10.0, 0.0)); 3];
    assert_eq!(scene.aimed_key_at(55.0, 40.0), Some(0));
    // Where the aim keeps a tap, the odds still move it from where it was meant.
    scene.aims = vec![Some((1.0, 0.0)); 3];
    assert_eq!(scene.aimed_key_at(48.0, 40.0), Some(1));
}
