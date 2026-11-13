use render::{
    scene::Scene,
    smartbar::{self, Config},
    smartbar_scene::{Action, Smartbar},
    touch::{Action as Event, Motion, Touch},
};
fn config() -> Config {
    Config::decode(&[1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 19.0, 0.0, 1.0, 40.0]).unwrap()
}
fn scene() -> Scene {
    let mut bar = Smartbar::new(
        config(),
        vec![
            Action {
                enabled: true,
                ..Action::default()
            };
            24
        ],
    );
    bar.intrinsic = vec![70.0, 500.0, 30.0];
    bar.relayout(400.0);
    Scene {
        width: 400,
        height: 200,
        strip: 40.0,
        smartbar: Some(bar),
        candidates: vec!["a".into(); 3],
        ..Scene::default()
    }
}
fn event(action: i32, x: f32, y: f32) -> Motion {
    Motion {
        action,
        index: 0,
        ids: vec![7],
        times: vec![1, 2],
        coords: vec![x, y],
    }
}
#[test]
fn pinned_regions_cannot_commit_candidates_and_cancel_releases_actions() {
    let mut scene = scene();
    let mut touch = Touch::default();
    assert_eq!(
        touch.event(&mut scene, &event(0, 20.0, 20.0)),
        vec![Event(5, 0, 7)]
    );
    assert_eq!(
        touch.event(&mut scene, &event(3, 20.0, 20.0)),
        vec![Event(7, 0, 7)]
    );
    assert!(touch.event(&mut scene, &event(1, 20.0, 20.0)).is_empty());
    let slot = scene.smartbar.as_ref().unwrap().layout.candidates[0].bounds;
    let x = (slot.left + slot.right) / 2.0;
    assert_eq!(
        touch.event(&mut scene, &event(0, x, 20.0)),
        vec![Event(8, 0, 7)]
    );
    assert_eq!(
        touch.event(&mut scene, &event(2, 20.0, 20.0)),
        vec![Event(10, 0, 7)]
    );
    assert!(touch.event(&mut scene, &event(1, x, 20.0)).is_empty());
}
#[test]
fn candidates_scroll_without_committing_and_keep_hit_regions_clipped() {
    let mut scene = scene();
    let bar = scene.smartbar.as_mut().unwrap();
    bar.config.mode = 2;
    bar.intrinsic = vec![160.0; 8];
    bar.relayout(400.0);
    let mut touch = Touch::default();
    touch.event(&mut scene, &event(0, 150.0, 20.0));
    assert_eq!(
        touch.event(&mut scene, &event(2, 100.0, 20.0)),
        vec![Event(10, 0, 7)]
    );
    assert!(scene.smartbar.as_ref().unwrap().scroll > 0.0);
    assert_eq!(
        touch.event(&mut scene, &event(1, 100.0, 20.0)),
        vec![Event(10, 0, 7)]
    );
    let bar = scene.smartbar.as_ref().unwrap();
    assert_eq!(bar.layout.hit(20.0, 20.0), Some((true, 0)));
}
#[test]
fn classic_single_dynamic_and_extended_rules_are_distinct() {
    let mut c = config();
    c.layout = 0;
    let single = smartbar::layout(&c, 400.0, &[60.0], 0.0);
    // Classic's inner weighted column fills the lone candidate, despite the outer fill=false.
    assert_eq!(single.candidates[0].bounds.left, 0.0);
    assert_eq!(single.candidates[0].bounds.width(), 400.0);
    c.mode = 1;
    assert_eq!(
        smartbar::layout(&c, 400.0, &[60.0], 0.0).candidates[0]
            .bounds
            .left,
        170.0
    );
    c.mode = 0;
    let many = smartbar::layout(&c, 400.0, &[60.0; 9], 0.0);
    assert_eq!(many.candidates.len(), 3);
    c.mode = 1;
    assert_eq!(
        smartbar::layout(&c, 400.0, &[900.0; 9], 0.0).candidates[0]
            .bounds
            .width(),
        160.0
    );
    c.layout = 3;
    c.extended = true;
    c.placement = 2;
    assert_eq!((c.height(), c.inset()), (80.0, 40.0));
    let extended = smartbar::layout(&c, 400.0, &[], 0.0);
    assert_eq!(extended.center.top, 40.0);
    assert!(extended.actions.iter().any(|s| s.bounds.top == 0.0));
}

#[test]
fn toolbar_gesture_never_slides_into_character_keys() {
    let mut s = scene();
    s.keys.push(render::scene::Key {
        code: 97,
        bounds: keys::Bounds {
            touch: keys::Rect {
                left: 0.0,
                top: 0.0,
                right: 400.0,
                bottom: 160.0,
            },
            visible: keys::Rect::default(),
        },
        label: "a".into(),
        hint: String::new(),
        icon: String::new(),
        styles: Default::default(),
        hints: Default::default(),
        enabled: true,
    });
    let mut t = Touch::default();
    assert_eq!(t.event(&mut s, &event(0, 20.0, 20.0)), vec![Event(5, 0, 7)]);
    assert_eq!(t.event(&mut s, &event(2, 20.0, 80.0)), vec![Event(7, 0, 7)]);
    assert!(t.event(&mut s, &event(1, 20.0, 80.0)).is_empty());
}
