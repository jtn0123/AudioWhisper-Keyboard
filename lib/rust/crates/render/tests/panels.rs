use keys::codes;
use render::{
    accessibility::Semantics,
    arrangement::{Arrangement, Editor},
    panels::{Command, Entry, Panel},
    protocol::*,
    scene::Scene,
    touch::{Motion, Touch},
};
fn scene(editor: bool) -> Scene {
    let arrangement = Arrangement {
        sticky: 0,
        dynamic: (1..10).collect(),
        hidden: vec![10],
    };
    let mut panel = Panel {
        kind: if editor { 3 } else { 2 },
        density: 2.0,
        row: 96.0,
        repeat_delay: 50_000_000,
        visible_actions: 3,
        entries: (0..11)
            .map(|id| Entry {
                id,
                code: if id == 4 {
                    codes::ARROW_LEFT
                } else {
                    codes::CLIPBOARD_SELECT_ALL
                },
                enabled: id != 5,
                insert_text: false,
                label: format!("Action {id}"),
                glyph: String::new(),
                icon: String::new(),
            })
            .collect(),
        editor: editor.then(|| Editor::new(arrangement.clone())),
        arrangement,
        titles: vec![
            "Customize".into(),
            "Edit actions".into(),
            "Sticky {n}".into(),
            "Dynamic {n}".into(),
            "Hidden {n}".into(),
            "Back".into(),
        ],
        ..Panel::default()
    };
    panel.layout(1080.0, 650.0, 96.0);
    Scene {
        width: 1080,
        height: 650,
        strip: 96.0,
        panel: Some(panel),
        ..Scene::default()
    }
}
fn motion(action: i32, x: f32, y: f32, t: i64) -> Motion {
    Motion {
        action,
        index: 0,
        ids: vec![0],
        times: vec![0, t],
        coords: vec![x, y],
    }
}
#[test]
fn held_arrow_repeats_on_the_native_clock_and_release_does_not_add_an_extra_move() {
    let mut s = scene(false);
    let p = s.panel.as_ref().unwrap();
    let r = p
        .items
        .iter()
        .find(|i| i.command == Command::Action(4))
        .unwrap()
        .bounds;
    let (x, y) = ((r.left + r.right) / 2.0, (r.top + r.bottom) / 2.0);
    let mut touch = Touch::default();
    assert_eq!(touch.event(&mut s, &motion(0, x, y, 0))[0].0, PANEL_DOWN);
    assert!(touch.tick(&mut s, 299_999_999).is_empty());
    assert_eq!(
        touch.tick(&mut s, 300_000_000)[0],
        render::touch::Action(PANEL_REPEAT, 4, codes::ARROW_LEFT)
    );
    assert_eq!(touch.next_deadline(), Some(350_000_000));
    assert_eq!(
        touch.event(&mut s, &motion(1, x, y, 310_000_000))[0].0,
        PANEL_CANCEL
    );
    assert_eq!(touch.next_deadline(), None);
}
#[test]
fn scroll_cancels_press_and_cannot_activate_an_offscreen_or_disabled_item() {
    let mut s = scene(false);
    let mut t = Touch::default();
    t.event(&mut s, &motion(0, 100.0, 160.0, 0));
    assert!(
        t.event(&mut s, &motion(2, 100.0, 120.0, 30_000_000))
            .iter()
            .any(|a| a.0 == PANEL_CANCEL)
    );
    assert!(
        t.event(&mut s, &motion(1, 100.0, 120.0, 50_000_000))
            .is_empty()
    );
    let semantics = Semantics::default();
    for n in semantics.snapshot(&s, "unshifted") {
        assert!(n.bounds.top >= 0.0 && n.bounds.bottom <= 650.0);
        if !n.enabled {
            assert!(n.actions.is_empty());
        }
    }
}
#[test]
fn editor_accessibility_clicks_move_a_tile_between_groups() {
    let mut s = scene(true);
    let mut semantics = Semantics::default();
    let source = semantics
        .snapshot(&s, "unshifted")
        .into_iter()
        .find(|n| n.label == "Action 1")
        .unwrap();
    semantics.activate(&mut s, "unshifted", source.id, ACCESS_CLICK);
    let target = semantics
        .snapshot(&s, "unshifted")
        .into_iter()
        .find(|n| n.label == "Action 0")
        .unwrap();
    semantics.activate(&mut s, "unshifted", target.id, ACCESS_CLICK);
    let a = s.panel.as_mut().unwrap().editor.as_mut().unwrap().saved();
    assert_eq!(a.sticky, 1);
    assert_eq!(a.dynamic[0], 0);
}

#[test]
fn fling_uses_native_deadlines_stops_at_bounds_and_new_down_cancels_it() {
    let mut s = scene(false);
    let p = s.panel.as_mut().unwrap();
    p.max_scroll = 1000.0;
    let mut t = Touch::default();
    t.event(&mut s, &motion(0, 100.0, 500.0, 0));
    t.event(&mut s, &motion(2, 100.0, 480.0, 20_000_000));
    t.event(&mut s, &motion(2, 100.0, 440.0, 40_000_000));
    t.event(&mut s, &motion(1, 100.0, 440.0, 50_000_000));
    let before = s.panel.as_ref().unwrap().scroll;
    assert!(t.next_deadline().is_some());
    t.tick(&mut s, 80_000_000);
    assert!(s.panel.as_ref().unwrap().scroll > before);
    t.event(&mut s, &motion(0, 100.0, 400.0, 90_000_000));
    let stopped = s.panel.as_ref().unwrap().scroll;
    t.tick(&mut s, 150_000_000);
    assert_eq!(stopped, s.panel.as_ref().unwrap().scroll);
    t.event(&mut s, &motion(3, 100.0, 400.0, 160_000_000));
    assert_eq!(None, t.next_deadline());
}

#[test]
fn overflow_grid_reserves_parent_margin_before_adaptive_cell_distribution() {
    let mut s = scene(false);
    let panel = s.panel.as_mut().unwrap();
    panel.grid_margin = [11.0; 4];
    panel.row = 113.0;
    panel.layout(1080.0, 650.0, 96.0);
    let tile = panel
        .items
        .iter()
        .find(|i| i.element == "smartbar-action-tile")
        .unwrap();
    assert_eq!(tile.bounds.left, 11.0);
    assert_eq!(tile.bounds.top, 107.0);
    assert_eq!(tile.bounds.width(), 265.0);
}
