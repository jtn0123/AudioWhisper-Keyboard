use render::{
    accessibility::Semantics,
    panels::{Command, Panel},
    protocol::*,
    scene::Scene,
    subtype_panel::{Subtypes, rows},
};
#[test]
fn configured_order_labels_selection_clipping_and_dismissal() {
    let ids = vec![i64::MAX, -9, 81];
    let labels = vec![
        "English (US)".into(),
        "Español".into(),
        "English (UK)".into(),
    ];
    assert_eq!(
        rows(&ids, &labels, -9),
        vec![
            (i64::MAX, labels[0].clone(), false),
            (-9, labels[1].clone(), true),
            (81, labels[2].clone(), false)
        ]
    );
    let mut p = Panel {
        kind: 7,
        density: 1.0,
        subtypes: Some(Subtypes {
            ids,
            labels,
            selected: -9,
            header: "Select subtype".into(),
            header_height: 48.0,
            row_height: 56.0,
            ..Default::default()
        }),
        ..Default::default()
    };
    p.layout(400.0, 300.0, 48.0);
    assert_eq!(p.items[2].bounds.top, 84.0);
    assert_eq!(p.items[4].command, Command::Subtype(Some(1)));
    let mut s = Scene {
        width: 400,
        height: 300,
        panel: Some(p),
        ..Default::default()
    };
    let mut sem = Semantics::default();
    let n = sem.snapshot(&s, "unshifted");
    let selected = n.iter().find(|n| n.label == "Español").unwrap();
    assert!(selected.selected);
    assert_eq!(
        sem.activate(&mut s, "unshifted", selected.id, ACCESS_CLICK),
        vec![render::touch::Action(PANEL_SUBTYPE, 1, 1)]
    );
    assert_eq!(
        s.panel.as_mut().unwrap().activate(Command::Dismiss),
        vec![render::touch::Action(PANEL_SUBTYPE, 0, 0)]
    );
    let p = s.panel.as_mut().unwrap();
    p.layout(400.0, 100.0, 0.0);
    assert!(p.max_scroll > 0.0);
    p.scroll = 10000.0;
    p.layout(400.0, 100.0, 0.0);
    assert_eq!(p.scroll, p.max_scroll);
}

#[test]
fn sheet_stays_above_navigation_while_scrim_covers_navigation() {
    let mut panel = Panel {
        kind: 7,
        subtypes: Some(Subtypes {
            ids: vec![1],
            labels: vec!["English".into()],
            header_height: 40.0,
            row_height: 56.0,
            bottom_inset: 24.0,
            ..Default::default()
        }),
        ..Default::default()
    };
    panel.layout(400.0, 300.0, 0.0);
    assert_eq!(panel.items[0].bounds.bottom, 300.0);
    assert_eq!(panel.items[1].bounds.bottom, 276.0);
    assert_eq!(
        panel.hit(200.0, 290.0).unwrap().command,
        Command::Subtype(None)
    );
    assert_eq!(
        panel.hit(200.0, 250.0).unwrap().command,
        Command::Subtype(Some(0))
    );
}

#[test]
fn disabled_clickable_header_minimum_target_is_clipped_at_first_row() {
    let mut panel = Panel {
        kind: 7,
        density: 2.625,
        subtypes: Some(Subtypes {
            ids: vec![1],
            labels: vec!["English".into()],
            header: "Select subtype".into(),
            header_height: 109.0,
            row_height: 135.0,
            ..Default::default()
        }),
        ..Default::default()
    };
    panel.layout(1080.0, 2274.0, 0.0);
    let target = panel.target_bounds(2);
    assert_eq!(target.top, 2021.5);
    assert_eq!(target.bottom, 2139.0);
    let header = panel.subtypes.as_ref().unwrap().header.clone();
    let scene = Scene {
        width: 1080,
        height: 2274,
        panel: Some(panel),
        ..Default::default()
    };
    let nodes = Semantics::default().snapshot(&scene, "");
    assert_eq!(
        nodes.iter().find(|n| n.label == header).unwrap().bounds,
        target
    );
}
