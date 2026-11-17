//! A screen reader visits the keyboard in screen order: the strip first, left to right, then the
//! keys or panel row by row, with a popup read straight after whatever opened it.
use keys::{Bounds, Rect, codes::*};
use render::{
    accessibility::{Node, Semantics, Target},
    clipboard_panel::{ClipEntry, ClipboardPanel},
    emoji_panel::{EmojiGroup, EmojiPanel, EmojiValue},
    panels::{Command, Panel},
    scene::{Key, Popup, Scene},
    smartbar::Config,
    smartbar_scene::{Action, Smartbar},
};

fn rect(left: f32, top: f32, right: f32, bottom: f32) -> Rect {
    Rect {
        left,
        top,
        right,
        bottom,
    }
}
fn key(code: i32, r: Rect) -> Key {
    Key {
        code,
        label: char::from_u32(code as u32).unwrap().to_string(),
        hint: String::new(),
        icon: String::new(),
        bounds: Bounds {
            touch: r,
            visible: r,
        },
        styles: Default::default(),
        hints: Default::default(),
        enabled: true,
    }
}
/// A bar from the 17 Smartbar settings, with a mic, a speaker, the arrow and settings.
fn bar(config: &[f32; 17]) -> Smartbar {
    let actions = [
        VOICE_INPUT,
        READ_ALOUD,
        IME_UI_MODE_MEDIA,
        TOGGLE_ACTIONS_OVERFLOW,
    ]
    .into_iter()
    .chain([0, SETTINGS])
    .map(|code| Action {
        code,
        enabled: true,
        ..Default::default()
    })
    .collect();
    let mut bar = Smartbar::new(Config::decode(config).unwrap(), actions);
    bar.intrinsic = vec![50.0, 60.0, 70.0];
    bar.relayout(400.0);
    bar
}
/// Two rows of four keys, listed bottom row first and each row right to left.
fn scene() -> Scene {
    let keys = (0..8)
        .map(|i| {
            let (row, column) = (1 - i / 4, 3 - i % 4);
            let (x, y) = (column as f32 * 100.0, row as f32 * 60.0);
            key(97 + i, rect(x, y, x + 100.0, y + 60.0))
        })
        .collect();
    Scene {
        width: 400,
        height: 160,
        strip: 40.0,
        keys,
        candidates: vec!["one".into(), "two".into(), "three".into()],
        candidate_tokens: vec![10, 20, 30],
        smartbar: Some(bar(&[
            1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 19.0, 0.0, 1.0, 40.0, 0.0, 0.0, 1.0, 0.0, 0.0,
        ])),
        ..Default::default()
    }
}
fn middle(r: Rect) -> f32 {
    (r.top + r.bottom) / 2.0
}
/// Each node reads later in its row, or in a lower row.
fn assert_rows(nodes: &[&Node], what: &str) {
    for pair in nodes.windows(2) {
        let (a, b) = (pair[0].bounds, pair[1].bounds);
        let same_row = b.top < middle(a) && a.top < middle(b);
        assert!(
            if same_row {
                b.left >= a.left
            } else {
                b.top > a.top
            },
            "{what}: {} then {}",
            pair[0].id,
            pair[1].id
        );
    }
}
fn assert_chain(nodes: &[Node]) {
    assert_eq!(nodes[0].after, None);
    for pair in nodes.windows(2) {
        assert_eq!(pair[1].after, Some(pair[0].id));
    }
}

#[test]
fn the_bar_reads_first_then_the_keys_row_by_row_left_to_right() {
    let nodes = Semantics::default().snapshot(&scene(), "unshifted");
    let first_key = nodes.iter().position(|n| n.role == "key").unwrap();
    assert!(first_key > 0);
    assert!(nodes[..first_key].iter().all(|n| n.role != "key"));
    let keys: Vec<i32> = nodes[first_key..].iter().map(|n| n.id).collect();
    assert_eq!(keys, vec![8, 7, 6, 5, 4, 3, 2, 1], "top row, then bottom");
    let strip: Vec<&Node> = nodes[..first_key].iter().collect();
    assert_rows(&strip, "strip");
    assert_chain(&nodes);
    // Touch still finds the bar over the keys: it was drawn after them.
    let top_key = nodes.iter().filter(|n| n.role == "key").map(|n| n.layer);
    assert!(top_key.max() < nodes[..first_key].iter().map(|n| n.layer).min());
}

#[test]
fn every_bar_layout_and_mode_reads_in_rows_left_to_right() {
    let mut two_rows = false;
    for layout in 0..4 {
        for bits in 0..16 {
            let on = |b: usize| f32::from(u8::from(bits & (1 << b) != 0));
            let mut s = scene();
            s.smartbar = Some(bar(&[
                1.0,
                layout as f32,
                on(0),
                on(1),
                on(2),
                0.0,
                1.0,
                1.0,
                19.0,
                0.0,
                1.0,
                40.0,
                0.0,
                0.0,
                on(3),
                0.0,
                0.0,
            ]));
            let nodes = Semantics::default().snapshot(&s, "unshifted");
            let strip: Vec<&Node> = nodes.iter().take_while(|n| n.role != "key").collect();
            assert!(!strip.is_empty(), "layout={layout} bits={bits}");
            assert_rows(&strip, &format!("layout={layout} bits={bits}"));
            assert!(nodes[strip.len()..].iter().all(|n| n.role == "key"));
            assert_chain(&nodes);
            two_rows |= strip
                .iter()
                .any(|n| n.bounds.top >= middle(strip[0].bounds));
        }
    }
    assert!(two_rows, "the extended layout's second row was covered");
}

#[test]
fn a_held_keys_choices_read_straight_after_it_and_stay_on_top_for_touch() {
    let mut s = scene();
    s.popup = Some(Popup {
        key: 6,
        labels: vec!["1".into(), "2".into(), "3".into(), "4".into()],
        selected: 0,
    });
    s.desired_key = [100.0, 60.0];
    render::popup_scene::configure(&mut s, 1, false).unwrap();
    let nodes = Semantics::default().snapshot(&s, "unshifted");
    let at = nodes.iter().position(|n| n.id == 7).unwrap();
    let block: Vec<&Node> = nodes[at + 1..]
        .iter()
        .take_while(|n| n.role == "popup item")
        .collect();
    assert_eq!(block.len(), 4);
    assert_eq!(nodes.iter().filter(|n| n.role == "popup item").count(), 4);
    assert_rows(&block, "popup");
    let others = nodes.iter().filter(|n| n.role != "popup item");
    let below = others.map(|n| n.layer).max().unwrap();
    assert!(block.iter().all(|n| n.layer > below));
    assert_chain(&nodes);
}

fn emoji() -> Scene {
    let mut panel = Panel {
        kind: 1,
        density: 2.0,
        row: 96.0,
        emoji: Some(EmojiPanel {
            values: (0..67)
                .map(|i| EmojiValue {
                    value: format!("fixture-{i}"),
                    tone: if (1..7).contains(&i) { i - 1 } else { 0 },
                })
                .collect(),
            // One emoji with variants, then enough plain ones to overflow the grid so it scrolls.
            groups: [(1, 0, 1), (2, 1, 6)]
                .into_iter()
                .chain((7..67).map(|start| (2, start, 1)))
                .map(|(category, start, count)| EmojiGroup {
                    category,
                    start,
                    count,
                })
                .collect(),
            pinned: vec![],
            recent: vec![],
            shown_pinned: vec![],
            shown_recent: vec![],
            history: true,
            locked: false,
            tone: 3,
            pinned_strategy: 0,
            recent_strategy: 0,
            category: 2,
            popup: None,
            labels: (0..15).map(|i| format!("Label {i}")).collect(),
            icons: vec![String::new(); 14],
            bottom: 120.0,
            gifs: false,
        }),
        ..Panel::default()
    };
    panel.layout(900.0, 600.0, 96.0);
    Scene {
        width: 900,
        height: 600,
        strip: 96.0,
        panel: Some(panel),
        ..Scene::default()
    }
}
/// The panel's popup as a slice of the reading order: one block, drawn over everything else. A
/// popup's items start with the backdrop that dismisses it.
fn popup_block(scene: &Scene, nodes: &[Node]) -> (usize, usize) {
    let items = &scene.panel.as_ref().unwrap().items;
    let start = items
        .iter()
        .position(|i| matches!(i.command, Command::Dismiss | Command::Clipboard(10, _)))
        .unwrap();
    let inside = |n: &Node| matches!(n.target, Target::Panel(i) if i >= start);
    let first = nodes.iter().position(inside).unwrap();
    let len = nodes[first..].iter().take_while(|n| inside(n)).count();
    assert_eq!(nodes.iter().filter(|n| inside(n)).count(), len, "one block");
    let below = nodes.iter().filter(|n| !inside(n)).map(|n| n.layer).max();
    assert!(
        nodes[first..first + len]
            .iter()
            .all(|n| Some(n.layer) > below)
    );
    (first, len)
}

#[test]
fn the_emoji_panel_reads_in_rows_and_a_variant_popup_after_its_emoji() {
    let mut s = emoji();
    let mut semantics = Semantics::default();
    let nodes = semantics.snapshot(&s, "unshifted");
    let scroll = nodes.iter().position(|n| n.role == "scroll").unwrap();
    let first_emoji = nodes.iter().position(|n| n.label.starts_with("fixture-"));
    assert_eq!(
        Some(scroll + 1),
        first_emoji,
        "the grid's scroll area opens it"
    );
    let plain: Vec<&Node> = nodes.iter().filter(|n| n.role != "scroll").collect();
    assert_rows(&plain, "emoji panel");
    assert_chain(&nodes);
    let cell = nodes.iter().find(|n| n.label == "fixture-4").unwrap().id;
    semantics.activate(
        &mut s,
        "unshifted",
        cell,
        render::protocol::ACCESS_LONG_CLICK,
    );
    s.panel.as_mut().unwrap().layout(900.0, 600.0, 96.0);
    let nodes = semantics.snapshot(&s, "unshifted");
    let (first, len) = popup_block(&s, &nodes);
    assert_eq!(nodes[first - 1].id, cell);
    let items: Vec<&Node> = nodes[first..first + len]
        .iter()
        .filter(|n| n.role == "popup item")
        .collect();
    assert!(!items.is_empty());
    assert_rows(&items, "variants");
    assert_chain(&nodes);
}

fn clipboard() -> Panel {
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
                    pinned: false,
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
fn a_clipboard_menu_reads_after_its_item_and_the_clear_dialog_after_its_button() {
    for (op, index, opener) in [(6, 1, 2_700_001), (2, 0, 2_000_000 - 65)] {
        let mut p = clipboard();
        p.clipboard_activate(op, index);
        p.layout(360.0, 300.0, 48.0);
        let s = Scene {
            width: 360,
            height: 300,
            panel: Some(p),
            ..Default::default()
        };
        let nodes = Semantics::default().snapshot(&s, "UNSHIFTED");
        let (first, len) = popup_block(&s, &nodes);
        assert_eq!(nodes[first - 1].id, opener, "op={op}");
        assert_eq!(
            nodes[first].id,
            2_000_000 - 110,
            "the backdrop opens the block"
        );
        let rest: Vec<&Node> = nodes[first + 1..first + len].iter().collect();
        assert!(!rest.is_empty());
        assert_rows(&rest, "clipboard popup");
        assert_chain(&nodes);
    }
}
