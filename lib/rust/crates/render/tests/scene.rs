use keys::{Bounds, Rect};
use render::style::Style;
use render::{
    raster::Raster,
    scene::{Key, Popup, Scene},
    touch::{Action, Motion, Touch},
};

fn scene() -> Scene {
    let keys = (0..2)
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
                styles: [
                    Style {
                        size: 20.0,
                        background: 0xff123456,
                        ..Style::default()
                    },
                    Style {
                        size: 20.0,
                        background: 0xffabcdef,
                        ..Style::default()
                    },
                ],
                hints: std::array::from_fn(|_| Style {
                    size: 10.0,
                    ..Style::default()
                }),
                enabled: true,
            }
        })
        .collect();
    Scene {
        width: 100,
        height: 80,
        strip: 20.0,
        keys,
        background: 0xffffffff,
        ..Scene::default()
    }
}

fn motion(action: i32, index: usize, ids: &[i32], coords: &[f32]) -> Motion {
    Motion {
        action,
        index,
        ids: ids.into(),
        times: vec![1, 2],
        coords: coords.into(),
    }
}

#[test]
fn strip_offset_edges_and_disabled_keys_use_existing_geometry() {
    let mut scene = scene();
    assert_eq!(scene.key_at(25.0, 19.0), None);
    assert_eq!(scene.key_at(49.99, 20.0), Some(0));
    assert_eq!(scene.key_at(50.0, 20.0), Some(1));
    assert_eq!(scene.key_at(100.0, 20.0), None);
    scene.keys[1].enabled = false;
    assert_eq!(scene.key_at(50.0, 20.0), None);
}

#[test]
fn pointer_reordering_cancel_and_history_preserve_action_order() {
    let mut scene = scene();
    scene.keys[0].code = keys::codes::SHIFT;
    let mut touch = Touch::default();
    assert_eq!(
        touch.event(&mut scene, &motion(0, 0, &[7], &[25.0, 40.0])),
        [Action(0, 0, 7)]
    );
    assert_eq!(
        touch.event(
            &mut scene,
            &motion(5, 1, &[7, 3], &[25.0, 40.0, 75.0, 40.0])
        ),
        [Action(0, 1, 3)]
    );
    assert_eq!(
        touch.event(
            &mut scene,
            &motion(6, 0, &[3, 7], &[75.0, 40.0, 25.0, 40.0])
        ),
        [Action(1, 1, 3)]
    );
    let history = Motion {
        action: 2,
        index: 0,
        ids: vec![7],
        times: vec![1, 2, 3],
        coords: vec![75.0, 40.0, 25.0, 40.0],
    };
    assert_eq!(
        touch.event(&mut scene, &history),
        [
            Action(2, 0, 7),
            Action(0, 1, 7),
            Action(2, 1, 7),
            Action(0, 0, 7)
        ]
    );
    assert_eq!(
        touch.event(&mut scene, &motion(3, 0, &[7], &[25.0, 40.0])),
        [Action(2, 0, 7)]
    );
    assert!(scene.pressed.is_empty());
    assert!(
        touch
            .event(&mut scene, &motion(1, 0, &[7], &[25.0, 40.0]))
            .is_empty()
    );
}

#[test]
fn malformed_event_cancels_instead_of_sticking_a_pressed_key() {
    let mut scene = scene();
    let mut touch = Touch::default();
    touch.event(&mut scene, &motion(0, 0, &[7], &[25.0, 40.0]));
    assert_eq!(
        touch.event(&mut scene, &motion(2, 0, &[7], &[f32::NAN, 40.0])),
        [Action(2, 0, 7)]
    );
    assert!(!motion(5, 1, &[7, 7], &[25.0, 40.0, 75.0, 40.0]).valid());
}

#[test]
fn popup_selection_is_an_action_and_cancel_discards_it() {
    let mut scene = scene();
    let mut touch = Touch::default();
    touch.event(&mut scene, &motion(0, 0, &[7], &[25.0, 40.0]));
    scene.popup = Some(Popup {
        key: 0,
        labels: vec!["é".into(), "è".into()],
        selected: 0,
    });
    touch.event(&mut scene, &motion(2, 0, &[7], &[80.0, 10.0]));
    assert_eq!(
        touch.event(&mut scene, &motion(1, 0, &[7], &[80.0, 10.0])),
        [Action(3, 1, 7)]
    );
    assert!(scene.popup.is_none());
}

#[test]
fn dirty_frames_equal_full_repaints_and_idle_has_no_damage() {
    let mut scene = scene();
    let mut raster = Raster::new(100, 80, vec![]);
    let full = Scene::default().damage(&scene).unwrap();
    raster.draw(&scene, full).unwrap();
    assert_eq!(
        &raster.pixels()[4 * (25 * 100 + 25)..][..4],
        &[0x12, 0x34, 0x56, 0xff]
    );
    assert_eq!(scene.damage(&scene), None);
    let old = scene.clone();
    scene.pressed = vec![1];
    let dirty = old.damage(&scene).unwrap();
    assert_eq!(
        dirty,
        Rect {
            left: 50.0,
            right: 100.0,
            top: 20.0,
            bottom: 80.0
        }
    );
    raster.draw(&scene, dirty).unwrap();
    let mut reference = Raster::new(100, 80, vec![]);
    reference.draw(&scene, full).unwrap();
    assert!(
        raster.pixels() == reference.pixels(),
        "dirty press differs from full repaint"
    );
    let old = scene.clone();
    scene.pressed.clear();
    raster.draw(&scene, old.damage(&scene).unwrap()).unwrap();
    reference.draw(&scene, full).unwrap();
    assert!(
        raster.pixels() == reference.pixels(),
        "dirty release differs from full repaint"
    );
}

#[test]
fn fractional_edges_do_not_accumulate_antialiasing_on_repaint() {
    let mut scene = scene();
    for key in &mut scene.keys {
        key.bounds.visible.left += 0.25;
        key.bounds.visible.right -= 0.25;
        key.bounds.visible.top += 0.25;
        key.bounds.visible.bottom -= 0.25;
        for style in &mut key.styles {
            style.corners = [3.75; 4];
        }
    }
    let full = Scene::default().damage(&scene).unwrap();
    let mut raster = Raster::new(100, 80, vec![]);
    raster.draw(&scene, full).unwrap();
    for i in 0..10 {
        let old = scene.clone();
        scene.pressed = if i % 2 == 0 { vec![0] } else { vec![] };
        raster.draw(&scene, old.damage(&scene).unwrap()).unwrap();
        let mut reference = Raster::new(100, 80, vec![]);
        reference.draw(&scene, full).unwrap();
        assert!(
            raster.pixels() == reference.pixels(),
            "fractional repaint differs at transition {i}"
        );
    }
}

#[test]
fn rolling_fingers_commit_characters_before_the_next_down() {
    let mut scene = scene();
    let mut touch = Touch::default();
    touch.event(&mut scene, &motion(0, 0, &[7], &[25.0, 40.0]));
    assert_eq!(
        touch.event(
            &mut scene,
            &motion(5, 1, &[7, 3], &[25.0, 40.0, 75.0, 40.0])
        ),
        [Action(1, 0, 7), Action(0, 1, 3)]
    );
    assert!(
        touch
            .event(
                &mut scene,
                &motion(6, 0, &[7, 3], &[25.0, 40.0, 75.0, 40.0])
            )
            .is_empty()
    );
    assert_eq!(
        touch.event(&mut scene, &motion(1, 0, &[3], &[75.0, 40.0])),
        [Action(1, 1, 3)]
    );
}

#[test]
fn leaving_and_reentering_keeps_pointer_identity_without_committing_outside() {
    let mut scene = scene();
    let mut touch = Touch::default();
    touch.event(&mut scene, &motion(0, 0, &[7], &[25.0, 40.0]));
    assert_eq!(
        touch.event(&mut scene, &motion(2, 0, &[7], &[-10.0, 40.0])),
        [Action(2, 0, 7)]
    );
    assert_eq!(
        touch.event(&mut scene, &motion(2, 0, &[7], &[75.0, 40.0])),
        [Action(0, 1, 7)]
    );
    assert_eq!(
        touch.event(&mut scene, &motion(1, 0, &[7], &[75.0, 40.0])),
        [Action(1, 1, 7)]
    );
    touch.event(&mut scene, &motion(0, 0, &[9], &[-10.0, 40.0]));
    assert!(
        touch
            .event(&mut scene, &motion(1, 0, &[9], &[-10.0, 40.0]))
            .is_empty()
    );
}

#[test]
fn shadow_damage_restores_neighbor_pixels() {
    let mut scene = scene();
    for key in &mut scene.keys {
        for style in &mut key.styles {
            style.shadow = 2.0;
        }
        for style in &mut key.styles {
            style.corners = [4.0; 4];
        }
    }
    let full = Scene::default().damage(&scene).unwrap();
    let mut raster = Raster::new(100, 80, vec![]);
    raster.draw(&scene, full).unwrap();
    assert_eq!(
        &raster.pixels()[4 * (45 * 100 + 25)..][..4],
        &[0x12, 0x34, 0x56, 0xff]
    );
    let old = scene.clone();
    scene.pressed = vec![0];
    raster.draw(&scene, old.damage(&scene).unwrap()).unwrap();
    let mut reference = Raster::new(100, 80, vec![]);
    reference.draw(&scene, full).unwrap();
    assert!(
        raster.pixels() == reference.pixels(),
        "shadow repaint differs from full repaint"
    );
}

#[test]
fn candidate_update_invalidates_the_strip_without_repainting_held_keys() {
    let mut before = scene();
    before.pressed = vec![0];
    let mut after = before.clone();
    after.candidates = vec!["fixture".into()];
    assert_eq!(
        before.damage(&after),
        Some(Rect {
            left: 0.0,
            top: 0.0,
            right: 100.0,
            bottom: 20.0
        })
    );
}

#[test]
fn candidate_click_requires_down_and_up_in_the_same_cell_without_leaving() {
    let mut scene = scene();
    scene.candidates = vec!["first".into(), "second".into()];
    let mut touch = Touch::default();
    touch.event(&mut scene, &motion(0, 0, &[7], &[75.0, 10.0]));
    assert_eq!(
        touch.event(&mut scene, &motion(1, 0, &[7], &[75.0, 10.0])),
        [Action(4, 1, 7)]
    );
    for (down, up) in [(-1.0, 25.0), (25.0, -1.0), (25.0, 75.0)] {
        touch.event(&mut scene, &motion(0, 0, &[7], &[down, 10.0]));
        assert!(
            touch
                .event(&mut scene, &motion(1, 0, &[7], &[up, 10.0]))
                .is_empty()
        );
    }
    touch.event(&mut scene, &motion(0, 0, &[7], &[25.0, 10.0]));
    touch.event(&mut scene, &motion(2, 0, &[7], &[75.0, 10.0]));
    touch.event(&mut scene, &motion(2, 0, &[7], &[25.0, 10.0]));
    assert!(
        touch
            .event(&mut scene, &motion(1, 0, &[7], &[25.0, 10.0]))
            .is_empty()
    );
}

#[test]
fn profiling_preserves_pixels_and_distinguishes_cold_and_warm_layouts() {
    // Existing repository font, not a pulled system font or a machine-specific path.
    let fonts =
        vec![include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec()];
    let mut scene = scene();
    scene.keys[0].label = "A".into();
    let full = Scene::default().damage(&scene).unwrap();
    let mut normal = Raster::new(100, 80, fonts.clone());
    normal.draw(&scene, full).unwrap();
    let off = normal.profile();
    assert_eq!(off.encode_ns + off.raster_ns + off.glyph_ns, 0);
    assert_eq!(off.alloc_sites + off.layout_misses + off.labels, 0);

    let mut measured = Raster::new(100, 80, fonts);
    measured.set_profiling(true);
    measured.draw(&scene, full).unwrap();
    assert_eq!(normal.pixels(), measured.pixels());
    let cold = measured.profile();
    assert_eq!(cold.layout_misses, 1);
    assert!(cold.cold_runs > 0);
    assert_eq!(cold.cold_runs, cold.glyph_runs);
    assert!(cold.encode_ns > 0 && cold.raster_ns > 0);
    measured.draw(&scene, full).unwrap();
    let warm = measured.profile();
    assert_eq!(warm.layout_misses + warm.cold_runs, 0);
    assert!(warm.glyph_runs > 0);
    assert_eq!(normal.pixels(), measured.pixels());
    measured.set_profiling(false);
    measured.draw(&scene, full).unwrap();
    assert_eq!(measured.profile().alloc_sites, 0);
    assert_eq!(measured.profile().encode_ns, 0);
}

#[test]
fn strip_and_key_damage_do_not_paint_the_rows_between_them() {
    let mut before = scene();
    for key in &mut before.keys {
        key.bounds.visible.top += 40.0;
        key.bounds.visible.bottom += 40.0;
    }
    before.height = 120;
    before.pressed = vec![1];
    let mut after = before.clone();
    after.pressed.clear();
    after.candidates = vec!["fixture".into()];
    let regions = before.damage_regions(&after);
    assert_eq!(regions.len(), 2);
    assert_eq!(regions[0].bottom, 20.0);
    assert_eq!(regions[1].top, 60.0);
}

#[test]
fn disjoint_and_overlapping_damage_equal_full_repaint_with_text_and_shadows() {
    let fonts =
        vec![include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec()];
    for background in [0xfffafafa, 0xff202020] {
        let mut scene = scene();
        scene.background = background;
        for (i, key) in scene.keys.iter_mut().enumerate() {
            key.bounds.visible.top += 30.25;
            key.bounds.visible.bottom += 30.25;
            key.bounds.visible.left += 0.25;
            key.bounds.visible.right -= 0.25;
            key.label = if i == 0 { "é" } else { "W" }.into();
            for style in &mut key.styles {
                style.corners = [3.75; 4];
                style.shadow = 1.5;
            }
        }
        scene.height = 120;
        scene.popup_style = Style {
            background: 0xffeeeeee,
            ..Style::default()
        };
        let full = Scene::default().damage(&scene).unwrap();
        let mut dirty = Raster::new(100, 120, fonts.clone());
        let mut reference = Raster::new(100, 120, fonts.clone());
        dirty.draw(&scene, full).unwrap();
        for step in 0..16 {
            let old = scene.clone();
            scene.pressed = if step % 2 == 0 { vec![0] } else { vec![] };
            scene.candidates = vec!["long clipped candidate".repeat(step + 1)];
            scene.popup = match step % 4 {
                0 => Some(Popup {
                    key: 0,
                    labels: vec!["è".into(), "é".into()],
                    selected: 0,
                }),
                1 => Some(Popup {
                    key: 0,
                    labels: vec!["è".into(), "é".into()],
                    selected: 1,
                }),
                _ => None,
            };
            dirty
                .draw_regions(&scene, &old.damage_regions(&scene))
                .unwrap();
            reference.draw(&scene, full).unwrap();
            let first = dirty
                .pixels()
                .as_chunks::<4>()
                .0
                .iter()
                .zip(reference.pixels().as_chunks::<4>().0.iter())
                .position(|(a, b)| a != b);
            assert!(
                first.is_none(),
                "transition {step}, first pixel {first:?}, regions {:?}",
                old.damage_regions(&scene)
            );
        }
    }
}

#[test]
fn profiled_damage_area_counts_overlap_rounding_and_clamping_once() {
    let regions = [
        Rect {
            left: -2.5,
            top: -0.5,
            right: 4.25,
            bottom: 2.25,
        },
        Rect {
            left: 3.0,
            top: 1.0,
            right: 20.0,
            bottom: 20.0,
        },
    ];
    assert_eq!(render::profile::damage_pixels(&regions, 10, 4), 32);
    assert_eq!(render::profile::damage_pixels(&[], 10, 4), 0);
}
