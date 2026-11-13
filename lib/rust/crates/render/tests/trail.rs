use keys::{Bounds, Rect};
use render::{
    gesture_config::GestureConfig,
    protocol::*,
    raster::Raster,
    scene::{Key, Scene},
    touch::{Motion, Touch},
    trail::Trail,
};
fn motion(action: i32, x: f32, time: i64) -> Motion {
    Motion {
        action,
        index: 0,
        ids: vec![0],
        times: vec![0, time],
        coords: vec![x, 60.0],
    }
}
fn scene() -> Scene {
    Scene {
        width: 200,
        height: 120,
        strip: 20.0,
        background: 0xffffffff,
        trail_color: 0xff4488dd,
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
#[test]
fn actual_glide_callbacks_feed_trail_and_outside_cancel_never_commits() {
    let mut scene = scene();
    let mut touch = Touch::default();
    touch.configure(GestureConfig {
        glide: true,
        ..GestureConfig::default()
    });
    touch.event(&mut scene, &motion(0, 50.0, 1_000_000));
    let moved = touch.event(&mut scene, &motion(2, 150.0, 17_000_000));
    assert_eq!(moved.iter().filter(|a| a.0 == GLIDE_POINT).count(), 2);
    assert!(touch.animate(&mut scene, 20_000_000));
    assert!(!scene.trail.is_empty());
    let cancel = touch.event(&mut scene, &motion(2, -10.0, 33_000_000));
    assert_eq!(cancel, vec![render::touch::Action(GLIDE_CANCEL, 0, 0)]);
    assert!(
        touch
            .event(&mut scene, &motion(2, 100.0, 49_000_000))
            .is_empty()
    );
    assert!(
        touch
            .event(&mut scene, &motion(1, 100.0, 65_000_000))
            .is_empty()
    );
    assert!(!touch.animate(&mut scene, 1_000_000_000));
    assert!(scene.trail.is_empty());
}
#[test]
fn trail_dirty_repaints_equal_full_repaints_through_fade_and_become_idle() {
    let mut scene = scene();
    scene.trail_color = 0x806633cc;
    let full = Rect {
        left: 0.0,
        top: 0.0,
        right: 200.0,
        bottom: 120.0,
    };
    let mut raster = Raster::new(200, 120, vec![]);
    let mut reference = Raster::new(200, 120, vec![]);
    raster.draw(&scene, full).unwrap();
    let mut trail = Trail::default();
    let point =
        |x: f32, y: f32| render::touch::Action(GLIDE_POINT, x.to_bits() as i32, y.to_bits() as i32);
    trail.actions(
        &[point(30.0, 40.0), point(90.0, 70.0), point(160.0, 40.0)],
        1_000_000_000,
    );
    trail.actions(
        &[render::touch::Action(GLIDE_COMPLETE, 0, 0)],
        1_010_000_000,
    );
    trail.actions(&[point(50.0, 80.0), point(130.0, 30.0)], 1_020_000_000);
    for step in 0..35 {
        let before = scene.clone();
        trail.frame(
            &mut scene,
            1_020_000_000 + step * 10_000_000,
            true,
            200_000_000,
        );
        raster
            .draw_regions(&scene, &before.damage_regions(&scene))
            .unwrap();
        reference.draw(&scene, full).unwrap();
        assert_eq!(raster.pixels(), reference.pixels(), "trail frame {step}");
    }
    assert!(scene.trail.is_empty());
    let before = scene.clone();
    for frame in 0..120 {
        assert!(!trail.frame(
            &mut scene,
            2_000_000_000 + frame * 16_666_667,
            true,
            200_000_000
        ));
        assert!(before.damage_regions(&scene).is_empty());
    }
}
#[test]
fn hidden_trail_needs_no_animation_frames() {
    let mut trail = Trail::default();
    let mut scene = scene();
    trail.actions(
        &[
            render::touch::Action(
                GLIDE_POINT,
                30.0f32.to_bits() as i32,
                40.0f32.to_bits() as i32,
            ),
            render::touch::Action(
                GLIDE_POINT,
                130.0f32.to_bits() as i32,
                40.0f32.to_bits() as i32,
            ),
        ],
        0,
    );
    assert!(trail.frame(&mut scene, 1, true, 200_000_000));
    assert!(!trail.frame(&mut scene, 2, false, 200_000_000));
    assert!(scene.trail.is_empty());
}

#[test]
fn leaving_before_glide_detection_cancels_without_feeding_the_classifier() {
    let mut scene = scene();
    let mut touch = Touch::default();
    touch.configure(GestureConfig {
        glide: true,
        ..GestureConfig::default()
    });
    touch.event(&mut scene, &motion(0, 50.0, 1_000_000));
    let outside = touch.event(&mut scene, &motion(2, 210.0, 17_000_000));
    assert!(
        outside
            .iter()
            .all(|a| a.0 != GLIDE_POINT && a.0 != GLIDE_COMPLETE)
    );
    assert!(
        touch
            .event(&mut scene, &motion(1, 210.0, 33_000_000))
            .is_empty()
    );
}
