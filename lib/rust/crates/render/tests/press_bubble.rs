//! The bubble over a pressed key (`press_bubble`): which keys show one, where it draws, how the
//! theme styles it and what it draws. A top-row key's bubble shows whole in the room the buffer
//! keeps above the keys, an edge key's stays on the keyboard, mirrored rows too, in portrait,
//! landscape and a one-handed keyboard's narrower surface.
use keys::{Rect, codes::*};
use render::{
    gesture_config::GestureConfig,
    press_bubble::{Bubble, mark, suitable},
    raster::Raster,
    scene::{Popup, Scene},
    theme::{Request, Theme},
};

mod press_bubble_common;
use press_bubble_common::*;

fn rgba(argb: u32) -> [u8; 4] {
    let [a, r, g, b] = argb.to_be_bytes();
    [r, g, b, a]
}

/// The rows of `r` in which some pixel is not the bubble's background: where it drew text.
fn inked_rows(raster: &Raster, scene: &Scene, r: Rect) -> Vec<f32> {
    let background = rgba(style().background);
    let (left, right) = (r.left as i32 + 2, r.right as i32 - 2);
    (r.top as i32 + 8..r.bottom as i32 - 8)
        .filter(|&y| (left..right).any(|x| pixel(raster, scene, x as f32, y as f32) != background))
        .map(|y| y as f32)
        .collect()
}

#[test]
fn characters_show_a_bubble_and_controls_and_number_pads_do_not() {
    for code in [97, 65, 44, 46, 48, 233, 0x4e2d] {
        assert!(suitable(code, "characters", "character"), "{code}");
        assert!(suitable(code, "symbols", "character"), "{code}");
    }
    for code in [
        SPACE, CJK_SPACE, ENTER, TAB, SHIFT, CAPS_LOCK, DELETE, 0, -202,
    ] {
        assert!(!suitable(code, "characters", "character"), "{code}");
    }
    for mode in ["numeric", "phone", "phone2"] {
        assert!(!suitable(49, mode, "numeric"), "{mode}");
        assert!(!suitable(44, mode, "character"), "{mode}");
    }
    // The advanced number pad keeps bubbles off its digits, not off its other keys.
    assert!(!suitable(49, "numeric_advanced", "numeric"));
    assert!(suitable(44, "numeric_advanced", "character"));
}

#[test]
fn marking_follows_each_key_and_the_mode() {
    let mut scene = keyboard(WIDTH, 10, 2.0, 0.0);
    scene.keys[0].code = SPACE;
    scene.keys[1].code = DELETE;
    scene.keys[2].code = 49;
    let mut types = vec!["character".to_string(); 30];
    types[2] = "numeric".into();
    mark(&mut scene, "numeric_advanced", &types);
    assert_eq!(scene.bubbles[..4], [false, false, false, true]);
    mark(&mut scene, "phone", &types);
    assert!(scene.bubbles.iter().all(|shown| !shown));
    // A key the snapshot gave no type is a character.
    mark(&mut scene, "characters", &types[..1]);
    assert_eq!(scene.bubbles.len(), 30);
    assert!(scene.bubbles[2] && !scene.bubbles[0]);
}

#[test]
fn a_bubble_is_the_preview_box_centred_over_its_key_down_to_its_bottom() {
    for strip in [0.0, 30.0] {
        for landscape in [false, true] {
            let scene = oriented(keyboard(WIDTH, 10, 2.0, strip), landscape);
            let [w, h] = scene.desired_key;
            let (wide, tall) = if landscape { (1.0, 3.0) } else { (1.1, 2.5) };
            for key in [4, 15, 25] {
                let visible = scene.keys[key].bounds.visible;
                let r = scene.bubble_bounds(key).expect("bubble");
                assert_eq!(r.width(), (w * wide).round());
                assert_eq!(r.height(), (h * tall).round());
                assert!((r.bottom - (visible.bottom + strip)).abs() <= 1.0, "{r:?}");
                let centre = (r.left + r.right) / 2.0;
                assert!((centre - (visible.left + visible.right) / 2.0).abs() <= 1.5);
            }
            let mut sizeless = scene.clone();
            sizeless.desired_key = [0.0, 0.0];
            assert_eq!(sizeless.bubble_bounds(4), None, "no size, no bubble");
        }
    }
}

#[test]
fn every_bubble_and_its_shadow_fit_the_buffer_and_the_top_row_reaches_above() {
    for (width, columns) in [(WIDTH, 10), (300, 10), (WIDTH, 11)] {
        for gap in [0.0, 2.0] {
            for strip in [0.0, 30.0] {
                for landscape in [false, true] {
                    let scene = oriented(keyboard(width, columns, gap, strip), landscape);
                    let room = scene.buffer_rect();
                    for key in 0..scene.keys.len() {
                        let r = scene.bubble_bounds(key).expect("bubble");
                        let shadow = pressed(&scene, key, false).bubble_damage().expect("view");
                        let case = format!("{width}/{columns}/{gap}/{strip}/{landscape} key {key}");
                        assert!(
                            r.left >= 0.0 && r.right <= f32::from(width),
                            "{case}: {r:?}"
                        );
                        assert!(shadow.top >= room.top, "{case}: {shadow:?} above {room:?}");
                        if key < columns {
                            assert!(r.top < strip, "{case}: a top-row bubble is over the app");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn edge_bubbles_move_in_only_as_far_as_the_keyboard_edge() {
    // Keys flush to the edges: a bubble wider than its key would hang over both.
    for width in [WIDTH, 300] {
        let scene = oriented(keyboard(width, 10, 0.0, 0.0), false);
        let wide = (scene.desired_key[0] * 1.1).round();
        let first = scene.bubble_bounds(0).expect("bubble");
        let last = scene.bubble_bounds(9).expect("bubble");
        assert_eq!((first.left, first.width()), (0.0, wide));
        assert_eq!((last.right, last.width()), (f32::from(width), wide));
        let middle = scene.bubble_bounds(4).expect("bubble");
        let key = scene.keys[4].bounds.visible;
        assert!(
            middle.left < key.left && middle.right > key.right,
            "centred, not moved"
        );
    }
}

/// `scene` with every key reflected left for right, as a left-handed layout mirrors its rows.
fn mirrored(scene: &Scene) -> Scene {
    let width = f32::from(scene.width);
    let flip = |r: Rect| Rect {
        left: width - r.right,
        right: width - r.left,
        ..r
    };
    let mut mirror = scene.clone();
    for key in &mut mirror.keys {
        key.bounds.touch = flip(key.bounds.touch);
        key.bounds.visible = flip(key.bounds.visible);
    }
    mirror
}

#[test]
fn mirrored_rows_mirror_their_bubbles_edges_included() {
    for gap in [0.0, 2.0] {
        for landscape in [false, true] {
            let mut scene = keyboard(WIDTH, 10, gap, 0.0);
            // A wide key at one end of the bottom row, as a utility key sits beside z.
            scene.keys[20].bounds.visible.right += 30.0;
            let scene = oriented(scene, landscape);
            let mirror = mirrored(&scene);
            for key in 0..scene.keys.len() {
                let r = scene.bubble_bounds(key).expect("bubble");
                let m = mirror.bubble_bounds(key).expect("bubble");
                assert_eq!(m.width(), r.width());
                let reflected = f32::from(WIDTH) - r.right;
                assert!(
                    (m.left - reflected).abs() <= 1.0,
                    "key {key}: {m:?} vs {r:?}"
                );
                assert!(m.left >= 0.0 && m.right <= f32::from(WIDTH));
            }
        }
    }
}

#[test]
fn the_theme_styles_a_bubble_as_its_keys_popup_box() {
    let json = r##"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet",
        "key-popup-box":{"background":"#112233","foreground":"#445566","font-size":"22sp",
            "shadow-elevation":"2dp"},
        "key-popup-box[shiftstate=`caps_lock`]":{"foreground":"#ff0000"},
        "key-popup-extended-indicator":{"font-size":"16sp"}}"##;
    let mut theme = Theme::new(json, vec![], vec![], 2.0, 1.0, 1.0).unwrap();
    let request = Request {
        element: String::new(),
        mode: "characters".into(),
        shift: "unshifted".into(),
        code: 100,
        key_type: "character".into(),
        selector: 0,
        auto_commit: false,
        popup_source: None,
    };
    let mut scene = keyboard(WIDTH, 10, 2.0, 0.0);
    theme.bubble(&mut scene, &request, true);
    assert_eq!(scene.bubble, None, "nothing to style before a press");
    scene.bubble = Some(Bubble::new(3));
    theme.bubble(&mut scene, &request, false);
    theme.popup_styles(&mut scene, &request, &[]);
    let bubble = scene.bubble.clone().expect("bubble");
    assert_eq!(bubble.style.as_ref(), Some(&scene.popup_style));
    assert_eq!(scene.popup_style.background, 0xff112233);
    assert!(scene.popup_style.shadow > 0.0);
    assert_eq!(bubble.indicator, None, "no dots without long-press choices");
    for (shift, foreground) in [("shifted", 0xff445566), ("caps_lock", 0xffff0000)] {
        let request = Request {
            shift: shift.into(),
            ..request.clone()
        };
        theme.bubble(&mut scene, &request, true);
        let bubble = scene.bubble.as_ref().expect("bubble");
        let (style, dots) = (bubble.style.as_ref().unwrap(), bubble.indicator.as_ref());
        let dots = dots.expect("dots for a key with choices");
        assert_eq!(style.foreground, foreground, "{shift}");
        assert_eq!(
            dots.foreground, foreground,
            "the dots take the box's colour"
        );
        assert!((dots.size / style.size - 16.0 / 22.0).abs() < 0.01);
    }
}

#[test]
fn the_setting_arrives_as_the_twelfth_gesture_value() {
    let values = |count: usize, bubble: f32| {
        let all = [
            1., 32., 0., 0., 0., 1., 200., 32., 1900., 0., 0., bubble, 0.,
        ];
        all[..count].to_vec()
    };
    let decode = |v: Vec<f32>| GestureConfig::decode(&v, vec!["NO_ACTION".into(); 9]);
    assert!(decode(values(12, 1.0)).expect("config").bubble);
    assert!(!decode(values(12, 0.0)).expect("config").bubble);
    assert!(
        !decode(values(11, 1.0)).expect("config").bubble,
        "off unless sent"
    );
    assert!(decode(values(13, 1.0)).is_none());
    assert!(!GestureConfig::default().bubble);
}

#[test]
fn a_bubble_draws_its_label_in_a_key_high_box_at_its_top_over_its_key() {
    let scene = oriented(keyboard(WIDTH, 10, 2.0, 0.0), false);
    // Key "c" of the top row.
    let open = pressed(&scene, 2, false);
    let raster = draw(&open);
    let r = open.bubble_bounds(2).expect("bubble");
    let key = open.keys[2].bounds.visible;
    let background = rgba(style().background);
    assert!(r.top < 0.0, "the bubble reaches over the app");
    assert_eq!(
        pixel(&raster, &open, (r.left + r.right) / 2.0, r.top + 2.0),
        background
    );
    // It covers its own key, below the label.
    let centre = (key.left + key.right) / 2.0;
    assert_eq!(pixel(&raster, &open, centre, key.bottom - 3.0), background);
    let rows = inked_rows(&raster, &open, r);
    assert!(!rows.is_empty(), "the label is drawn");
    let label_bottom = r.top + key.height().round();
    assert!(
        rows.iter().all(|&y| y < label_bottom),
        "label only in the top box"
    );
    // The room above the keyboard stays clear beside it.
    assert_eq!(pixel(&raster, &open, r.right + 20.0, r.top + 4.0), [0; 4]);
    assert_eq!(pixel(&raster, &open, r.left - 20.0, r.top + 4.0), [0; 4]);
}

#[test]
fn a_shifted_label_draws_the_capital_and_dots_show_choices() {
    let scene = oriented(keyboard(WIDTH, 10, 2.0, 0.0), false);
    let lower = pressed(&scene, 12, false);
    let mut upper = lower.clone();
    upper.keys[12].label = "C".into();
    let r = lower.bubble_bounds(12).expect("bubble");
    let (a, b) = (draw(&lower), draw(&upper));
    assert!(
        a.pixels() != b.pixels(),
        "the bubble shows the label the key types"
    );
    // Dots at the end, half way down: ink below the label box only with them.
    let dotted = pressed(&scene, 12, true);
    let label_bottom = r.top + lower.keys[12].bounds.visible.height().round();
    let below = |raster: &Raster, scene: &Scene| {
        inked_rows(
            raster,
            scene,
            Rect {
                top: label_bottom,
                ..r
            },
        )
        .len()
    };
    assert_eq!(below(&a, &lower), 0);
    assert!(below(&draw(&dotted), &dotted) > 0);
}

#[test]
fn nothing_draws_without_a_style_a_bubble_key_or_while_a_popup_is_open() {
    let scene = oriented(keyboard(WIDTH, 10, 2.0, 0.0), false);
    let open = pressed(&scene, 2, false);
    let closed = draw(&Scene {
        bubble: None,
        ..open.clone()
    });
    let mut unstyled = open.clone();
    unstyled.bubble = Some(Bubble::new(2));
    let mut control = open.clone();
    control.bubbles[2] = false;
    let mut popup = open.clone();
    popup.popup = Some(Popup {
        key: 2,
        labels: vec!["3".into()],
        selected: 0,
    });
    for (case, scene) in [
        ("unstyled", &unstyled),
        ("control", &control),
        ("popup", &popup),
    ] {
        assert_eq!(scene.bubble_view(), None, "{case}");
        assert_eq!(scene.bubble_damage(), None, "{case}");
    }
    assert!(draw(&unstyled).pixels() == closed.pixels());
    assert!(draw(&control).pixels() == closed.pixels());
    assert!(open.bubble_view().is_some());
}
