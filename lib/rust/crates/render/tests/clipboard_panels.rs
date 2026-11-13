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
        actions.iter().map(|a| a.2).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert!(actions.iter().all(|a| a.0 == PANEL_CLIPBOARD && a.1 == 8));
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
