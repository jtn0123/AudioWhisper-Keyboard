use render::{
    accessibility::Semantics,
    ime_chrome::{Chrome, one_handed, side_buttons},
    panels::Panel,
    protocol::*,
    scene::Scene,
    touch::{Action, Motion, Touch},
};
#[test]
fn side_controls_share_touch_and_accessibility_and_landscape_disables_compact_width() {
    assert_eq!(one_handed(true, true, 80, true), [1.0, 0.0, 0.0]);
    let b = side_buttons(80.0, 300.0);
    assert_eq!(b[0].height(), 80.0);
    assert_eq!(b[1].height(), 220.0);
    let mut p = Panel {
        kind: 8,
        density: 1.0,
        chrome: Some(Chrome {
            labels: vec![
                "Close one-handed mode".into(),
                "Move keyboard to left".into(),
            ],
            ..Default::default()
        }),
        ..Default::default()
    };
    p.layout(80.0, 300.0, 0.0);
    let mut s = Scene {
        width: 80,
        height: 300,
        panel: Some(p),
        ..Default::default()
    };
    let mut sem = Semantics::default();
    let nodes = sem.snapshot(&s, "");
    assert_eq!(nodes.len(), 2);
    assert_eq!(
        sem.activate(&mut s, "", nodes[0].id, ACCESS_CLICK),
        vec![Action(IME_CHROME, 0, 0)]
    );
    let mut touch = Touch::default();
    let motion = |action| Motion {
        action,
        index: 0,
        ids: vec![0],
        times: vec![0, 1],
        coords: vec![40.0, 200.0],
    };
    touch.event(&mut s, &motion(0));
    assert_eq!(
        touch.event(&mut s, &motion(1)),
        vec![Action(IME_CHROME, 1, 0)]
    );
    touch.event(&mut s, &motion(0));
    touch.event(&mut s, &motion(3));
    assert!(touch.event(&mut s, &motion(1)).is_empty());
}
#[test]
fn extraction_geometry_and_preview_graphemes_and_incognito_policy() {
    let r = render::extract_chrome::geometry(400.0, 200.0, 1.0, [80.0, 36.0]);
    assert_eq!(r[0].right, 304.0);
    assert_eq!(r[1].left, 312.0);
    assert_eq!(r[1].top, 82.0);
    for incognito in [false, true] {
        for behind in [false, true] {
            assert_eq!(
                render::ime_decorations::show_watermark(incognito, behind),
                incognito && behind
            );
        }
    }
    let text = "abc 👩🏽‍💻";
    let result = render::ime_decorations::start_ellipsis(text, 3.0, |s| {
        unicode_segmentation::UnicodeSegmentation::graphemes(s, true).count() as f32
    });
    assert_eq!(result, "… 👩🏽‍💻");
    assert_eq!(render::key_preview::size(2.0, 100.0), [100, 72]);
}
