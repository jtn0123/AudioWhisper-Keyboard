use render::{
    accessibility::Semantics,
    clipboard_panel::{ClipEntry, ClipboardPanel},
    panels::{Command, Panel},
    protocol::*,
    scene::Scene,
};
fn panel() -> Panel {
    let mut p = Panel {
        kind: 4,
        row: 48.0,
        density: 1.0,
        clipboard: Some(ClipboardPanel {
            enabled: true,
            action_height: 48.0,
            now: 1000,
            group_height: 24.0,
            message_heights: vec![24.0; 21],
            labels: (0..21).map(|i| format!("label-{i}")).collect(),
            entries: (0..4)
                .map(|i| ClipEntry {
                    id: i,
                    semantic: 700_000 + i as i32,
                    timestamp: i,
                    kind: 1,
                    pinned: i == 0,
                    label: format!("aw-test-{i}"),
                    description: String::new(),
                    date: String::new(),
                    aspect: 1.0,
                    height: 48.0,
                    date_height: 24.0,
                    full_height: 48.0,
                })
                .collect(),
            ..Default::default()
        }),
        ..Default::default()
    };
    p.layout(360.0, 300.0, 48.0);
    p
}
#[test]
fn pin_delete_paste_and_filtered_clear_keep_original_indices() {
    let mut p = panel();
    p.clipboard_activate(6, 2);
    assert_eq!(p.clipboard_activate(7, 0)[0].1, 7);
    assert_eq!(p.clipboard_activate(7, 0).len(), 1);
    p.clipboard_activate(6, 3);
    assert_eq!(p.clipboard_activate(9, 0)[0].2, 3);
    p.clipboard_activate(2, 0);
    let actions = p.clipboard_activate(11, 0);
    assert_eq!(
        actions
            .iter()
            .filter(|a| a.1 == 12)
            .map(|a| a.2)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert!(
        actions
            .iter()
            .all(|a| a.0 == PANEL_CLIPBOARD && [12, 13].contains(&a.1))
    );
}
#[test]
fn filter_toggle_resets_scroll_and_popup_blocks_header() {
    let mut p = panel();
    p.scroll = 80.0;
    p.clipboard_activate(3, 0);
    assert_eq!(p.scroll, 0.0);
    p.clipboard_activate(4, 2);
    assert_eq!(p.clipboard.as_ref().unwrap().filters, 4);
    p.clipboard_activate(3, 0);
    assert_eq!(p.clipboard.as_ref().unwrap().filters, 0);
    p.clipboard_activate(6, 1);
    p.layout(360.0, 300.0, 48.0);
    assert!(
        !p.items
            .iter()
            .find(|i| i.command == Command::Clipboard(2, 0))
            .unwrap()
            .enabled
    );
    assert!(
        p.items
            .iter()
            .filter(|i| matches!(i.command, Command::Clipboard(5, _)))
            .all(|i| !i.enabled)
    );
}
#[test]
fn accessibility_includes_every_visible_item_long_click_and_menu_action() {
    let mut p = panel();
    let mut s = Scene {
        width: 360,
        height: 300,
        panel: Some(p.clone()),
        ..Default::default()
    };
    let semantics = Semantics::default();
    let nodes = semantics.snapshot(&s, "UNSHIFTED");
    for i in 0..4 {
        let n = nodes
            .iter()
            .find(|n| n.label == format!("aw-test-{i}"))
            .unwrap();
        assert!(n.actions.contains(&ACCESS_LONG_CLICK));
    }
    p.clipboard_activate(6, 1);
    p.layout(360.0, 300.0, 48.0);
    s.panel = Some(p);
    let nodes = semantics.snapshot(&s, "UNSHIFTED");
    for label in ["label-13", "label-15", "label-16"] {
        assert!(
            nodes
                .iter()
                .any(|n| n.label == label && n.actions.contains(&ACCESS_CLICK))
        );
    }
}
#[test]
fn disabled_and_locked_never_expose_history() {
    for locked in [false, true] {
        let mut p = panel();
        let c = p.clipboard.as_mut().unwrap();
        c.locked = locked;
        c.enabled = false;
        p.layout(360.0, 300.0, 48.0);
        assert!(
            p.items
                .iter()
                .all(|i| !matches!(i.command, Command::Clipboard(5, _)))
        );
    }
}

#[test]
fn popup_content_scrolls_without_moving_the_history_grid() {
    use render::touch::{Motion, Touch};
    let mut p = panel();
    p.clipboard.as_mut().unwrap().entries[1].full_height = 1000.0;
    p.clipboard_activate(6, 1);
    p.layout(360.0, 300.0, 48.0);
    let mut scene = Scene {
        width: 360,
        height: 300,
        panel: Some(p),
        ..Default::default()
    };
    let mut touch = Touch::default();
    for (action, y, time) in [(0, 100.0, 1), (2, 60.0, 20_000_000), (1, 60.0, 30_000_000)] {
        touch.event(
            &mut scene,
            &Motion {
                action,
                index: 0,
                ids: vec![0],
                times: vec![0, time],
                coords: vec![70.0, y],
            },
        );
    }
    let p = scene.panel.as_ref().unwrap();
    assert_eq!(p.scroll, 0.0);
    assert_eq!(p.clipboard.as_ref().unwrap().popup_scroll, 40.0);
    assert_eq!(touch.next_deadline(), None);
}
#[test]
fn room_identity_is_stable_when_pinning_reorders_the_snapshot() {
    let source = panel().clipboard.unwrap();
    let mut c = ClipboardPanel::default();
    c.update(source.clone());
    for e in &mut c.entries {
        e.aspect = 2.0;
    }
    let ids: Vec<_> = c.entries.iter().map(|e| (e.id, e.semantic)).collect();
    let mut next = source;
    next.entries.reverse();
    next.entries[0].pinned = true;
    c.update(next);
    assert!(c.entries.iter().all(|e| e.aspect == 2.0));
    for (id, semantic) in ids {
        assert_eq!(
            c.entries.iter().find(|e| e.id == id).unwrap().semantic,
            semantic
        );
    }
}

#[test]
fn header_touch_expansion_respects_history_but_not_pointer_only_delete() {
    let mut p = panel();
    p.row = 40.0;
    p.layout(360.0, 300.0, 40.0);
    let index = p
        .items
        .iter()
        .position(|i| i.command == Command::Clipboard(3, 0))
        .unwrap();
    let bounds = p.target_bounds(index);
    assert_eq!(bounds.bottom, 40.0);
    assert_eq!(bounds.right, 324.0);
}

#[test]
fn clear_is_disabled_when_a_filter_has_no_matching_items() {
    let mut p = panel();
    p.clipboard.as_mut().unwrap().filters = 1 << 2;
    p.layout(360.0, 300.0, 48.0);
    assert!(
        !p.items
            .iter()
            .find(|i| i.command == Command::Clipboard(2, 0))
            .unwrap()
            .enabled
    );
    p.clipboard_activate(2, 0);
    assert!(!p.clipboard.as_ref().unwrap().clear);
}

#[test]
fn clear_dialog_preserves_compose_history_enabled_semantics() {
    let mut p = panel();
    p.clipboard.as_mut().unwrap().clear = true;
    p.layout(360.0, 300.0, 48.0);
    assert!(
        p.items
            .iter()
            .filter(|i| matches!(i.command, Command::Clipboard(5, _)))
            .all(|i| i.enabled)
    );
}
