use keys::{KeyFlex, desired_key, flex, layout};
use render::{
    accessibility::Semantics,
    protocol::*,
    scene::{Key, Popup, Scene},
    touch::{Action, Motion, Touch},
};

fn event(action: i32, x: f32, y: f32) -> Motion {
    Motion {
        action,
        index: 0,
        ids: vec![0],
        times: vec![0, 1],
        coords: vec![x, y],
    }
}

#[test]
fn evaluated_geometry_and_pointer_state_can_switch_every_mode_on_one_scene() {
    let mut scene = Scene {
        width: 1080,
        height: 800,
        strip: 113.,
        ..Default::default()
    };
    let mut touch = Touch::default();
    // Main text modes and the evaluator's auxiliary rows. Both 4/5-row character layouts
    // use the same renderer and the already-matched keys crate's geometry, without a QWERTY copy.
    for mode in [0, 2, 3, 0, 4, 5, 2, 0, 6, 7, 6, 0, 9, 8, 10, -1] {
        for row_count in [1, 4, 5] {
            assert!(
                touch
                    .event(&mut scene, &event(3, 0., 0.))
                    .iter()
                    .all(|a| a.0 == KEY_CANCEL)
            );
            assert!(scene.popup.is_none());
            assert!(scene.pressed.is_empty());
            let code = if mode == 0 { 101 } else { 49 };
            let factor = flex(mode, code, true);
            let row = vec![
                KeyFlex {
                    width_factor: factor.width_factor,
                    shrink: factor.shrink,
                    grow: factor.grow
                };
                if [4, 6, 7].contains(&mode) { 3 } else { 10 }
            ];
            let rows = vec![row.as_slice(); row_count];
            let (desired, visible) = desired_key(mode, 1080., 687., row_count as i32, 150., 2., 5.);
            let bounds = layout(&rows, desired, visible, 1080., 687., true).unwrap();
            scene.keys = bounds
                .into_iter()
                .map(|bounds| Key {
                    code,
                    bounds,
                    label: char::from_u32(code as u32).unwrap().to_string(),
                    hint: String::new(),
                    icon: String::new(),
                    styles: Default::default(),
                    hints: Default::default(),
                    enabled: true,
                })
                .collect();
            let target = scene.keys[0].bounds.visible;
            let x = (target.left + target.right) / 2.;
            let y = scene.strip + (target.top + target.bottom) / 2.;
            assert_eq!(
                touch.event(&mut scene, &event(0, x, y)),
                vec![Action(KEY_DOWN, 0, 0)]
            );
            assert_eq!(
                touch.event(&mut scene, &event(1, x, y)),
                vec![Action(KEY_UP, 0, 0)]
            );
            let nodes = Semantics::default().snapshot(&scene, "unshifted");
            assert_eq!(nodes.len(), scene.keys.len());
            assert_eq!(nodes[0].id, 1);
            // A held popup on the old layout cannot select a slot in the new one.
            touch.event(&mut scene, &event(0, x, y));
            scene.popup = Some(Popup {
                key: 0,
                labels: vec!["fixture".into()],
                selected: 0,
            });
        }
    }
    assert_eq!(touch.cancel(&mut scene), vec![Action(KEY_CANCEL, 0, 0)]);
}
