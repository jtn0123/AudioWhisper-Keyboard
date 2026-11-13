use render::{
    accessibility::{Semantics, Target},
    scene::Scene,
    smartbar::{self, Config},
    smartbar_scene::{Action, Smartbar},
    touch::{Motion, Touch},
};

#[test]
fn platform_region_never_draws_candidates_or_steals_touches_or_semantics() {
    for layout in 0..=3 {
        for flip in 0..=1 {
            for shared in 0..=1 {
                for placement in 0..=2 {
                    let config = Config::decode(&[
                        1.,
                        layout as f32,
                        flip as f32,
                        shared as f32,
                        1.,
                        placement as f32,
                        1.,
                        1.,
                        19.,
                        2.,
                        2.625,
                        113.,
                        1.,
                        0.,
                    ])
                    .unwrap();
                    let mut bar = Smartbar::new(config.clone(), vec![Action::default(); 24]);
                    bar.intrinsic = vec![200.; 20];
                    bar.relayout(1080.);
                    let center = bar.layout.center;
                    let x = (center.left + center.right) / 2.;
                    let y = (center.top + center.bottom) / 2.;
                    if config.inline_visible() {
                        assert!(bar.layout.candidates.is_empty());
                        assert_eq!(bar.layout.hit(x, y), None);
                        assert!(bar.layout.actions.iter().all(|slot| {
                            let r = slot.bounds;
                            r.right <= center.left
                                || r.left >= center.right
                                || r.bottom <= center.top
                                || r.top >= center.bottom
                        }));
                        let mut scene = Scene {
                            width: 1080,
                            height: 800,
                            strip: config.height(),
                            smartbar: Some(bar),
                            candidates: vec!["fixture".into(); 20],
                            ..Default::default()
                        };
                        let mut touch = Touch::default();
                        for action in [0, 2, 1] {
                            assert!(
                                touch
                                    .event(
                                        &mut scene,
                                        &Motion {
                                            action,
                                            index: 0,
                                            ids: vec![0],
                                            times: vec![0, 1],
                                            coords: vec![x, y],
                                        }
                                    )
                                    .is_empty()
                            );
                        }
                        assert!(
                            Semantics::default()
                                .snapshot(&scene, "unshifted")
                                .iter()
                                .all(|node| !matches!(node.target, Target::Candidate(_)))
                        );
                    } else {
                        assert_eq!((layout, shared), (2, 1));
                    }
                }
            }
        }
    }
}

#[test]
fn inline_arrival_and_removal_keep_center_and_extended_insets_stable() {
    let raw = [1., 3., 1., 0., 1., 2., 1., 1., 4., 0., 2.625, 113., 0., 0.];
    let plain = Config::decode(&raw).unwrap();
    let mut raw_inline = raw;
    raw_inline[12] = 1.;
    let inline = Config::decode(&raw_inline).unwrap();
    let before = smartbar::layout(&plain, 1080., &[70., 120.], 0.);
    let during = smartbar::layout(&inline, 1080., &[70., 120.], 0.);
    assert_eq!(before.center, during.center);
    assert_eq!(before.actions, during.actions);
    assert_eq!(
        (plain.height(), plain.inset()),
        (inline.height(), inline.inset())
    );
    assert_eq!(smartbar::layout(&plain, 1080., &[70., 120.], 0.), before);
    raw_inline[13] = 1.;
    assert!(!Config::decode(&raw_inline).unwrap().inline_visible());
}
