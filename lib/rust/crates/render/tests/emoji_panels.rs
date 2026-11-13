use render::{
    accessibility::Semantics,
    emoji_panel::{EmojiGroup, EmojiPanel, EmojiValue},
    panels::{Command, Panel},
    protocol::*,
    scene::Scene,
    touch::{Motion, Touch},
};
fn scene() -> Scene {
    let mut panel = Panel {
        kind: 1,
        density: 2.0,
        row: 96.0,
        repeat_delay: 50_000_000,
        gesture_delay: 500_000_000,
        emoji: Some(EmojiPanel {
            values: (0..7)
                .map(|i| EmojiValue {
                    value: format!("fixture-{i}"),
                    tone: if i == 0 { 0 } else { i - 1 },
                })
                .collect(),
            groups: vec![
                EmojiGroup {
                    category: 1,
                    start: 0,
                    count: 1,
                },
                EmojiGroup {
                    category: 2,
                    start: 1,
                    count: 6,
                },
            ],
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
fn motion(action: i32, x: f32, y: f32, time: i64) -> Motion {
    Motion {
        action,
        index: 0,
        ids: vec![0],
        times: vec![0, time],
        coords: vec![x, y],
    }
}
fn layout(scene: &mut Scene) {
    scene.panel.as_mut().unwrap().layout(900.0, 600.0, 96.0);
}
#[test]
fn preferred_tone_and_sticky_popup_release_commit_exactly_one_choice() {
    let mut s = scene();
    let mut t = Touch::default();
    assert!(
        s.panel
            .as_ref()
            .unwrap()
            .items
            .iter()
            .any(|i| i.command == Command::Emoji(4))
    );
    t.event(&mut s, &motion(0, 45.0, 141.0, 0));
    assert!(t.tick(&mut s, 499_999_999).is_empty());
    t.tick(&mut s, 500_000_000);
    layout(&mut s);
    assert_eq!(
        s.panel.as_ref().unwrap().emoji.as_ref().unwrap().popup,
        Some((4, false, false))
    );
    assert!(
        t.event(&mut s, &motion(1, 45.0, 141.0, 550_000_000))
            .iter()
            .all(|a| a.0 != PANEL_EMOJI)
    );
    let p = s.panel.as_ref().unwrap();
    let choices: Vec<_> = p.items.iter().filter(|i| i.id >= 400_000).collect();
    assert_eq!(choices.len(), 5);
    assert!(choices.iter().all(|i| i.command != Command::Emoji(4)));
    let r = choices[0].bounds;
    let (x, y) = ((r.left + r.right) / 2.0, (r.top + r.bottom) / 2.0);
    t.event(&mut s, &motion(0, x, y, 600_000_000));
    let out = t.event(&mut s, &motion(1, x, y, 630_000_000));
    assert_eq!(out.iter().filter(|a| a.0 == PANEL_EMOJI).count(), 1);
    assert!(
        s.panel
            .as_ref()
            .unwrap()
            .emoji
            .as_ref()
            .unwrap()
            .popup
            .is_none()
    );
}
#[test]
fn category_refreshes_frozen_recents_and_locked_history_exposes_no_values() {
    let mut s = scene();
    let p = s.panel.as_mut().unwrap();
    let e = p.emoji.as_mut().unwrap();
    e.recent = vec![2, 1];
    e.shown_recent = vec![1, 2];
    p.activate(Command::Category(0));
    layout(&mut s);
    assert_eq!(
        s.panel
            .as_ref()
            .unwrap()
            .emoji
            .as_ref()
            .unwrap()
            .shown_recent,
        vec![2, 1]
    );
    let e = s.panel.as_mut().unwrap().emoji.as_mut().unwrap();
    e.recent = vec![3, 2, 1];
    assert_eq!(e.shown_recent, vec![2, 1]);
    e.locked = true;
    layout(&mut s);
    assert!(
        s.panel
            .as_ref()
            .unwrap()
            .items
            .iter()
            .all(|i| !matches!(i.command, Command::Emoji(_)))
    );
}
#[test]
fn accessibility_can_open_select_and_dismiss_variants_without_a_physical_pointer() {
    let mut s = scene();
    let mut semantics = Semantics::default();
    let key = semantics
        .snapshot(&s, "unshifted")
        .into_iter()
        .find(|n| n.label == "fixture-4")
        .unwrap();
    assert!(key.actions.contains(&ACCESS_LONG_CLICK));
    assert!(
        !semantics
            .activate(&mut s, "unshifted", key.id, ACCESS_LONG_CLICK)
            .is_empty()
    );
    layout(&mut s);
    let item = semantics
        .snapshot(&s, "unshifted")
        .into_iter()
        .find(|n| n.role == "popup item")
        .unwrap();
    assert!(item.actions.contains(&ACCESS_DISMISS));
    semantics.activate(&mut s, "unshifted", item.id, ACCESS_DISMISS);
    assert!(
        s.panel
            .as_ref()
            .unwrap()
            .emoji
            .as_ref()
            .unwrap()
            .popup
            .is_none()
    );
}
#[test]
fn history_popup_keeps_compose_manual_strategy_precedence() {
    let mut s = scene();
    let p = s.panel.as_mut().unwrap();
    let e = p.emoji.as_mut().unwrap();
    e.category = 0;
    e.shown_pinned = vec![1];
    e.pinned_strategy = 0;
    e.recent_strategy = 2;
    p.long_press_emoji(1);
    layout(&mut s);
    let commands: Vec<_> = s
        .panel
        .as_ref()
        .unwrap()
        .items
        .iter()
        .filter(|i| i.id >= 400_000)
        .map(|i| i.command.clone())
        .collect();
    assert_eq!(
        commands,
        vec![
            Command::History(2, 1),
            Command::History(7, 1),
            Command::History(8, 1),
            Command::History(4, 1)
        ]
    );
}

#[test]
fn holding_an_emoji_without_variants_consumes_the_tap() {
    let mut s = scene();
    s.panel.as_mut().unwrap().activate(Command::Category(1));
    layout(&mut s);
    let mut t = Touch::default();
    t.event(&mut s, &motion(0, 45.0, 141.0, 0));
    t.tick(&mut s, 500_000_000);
    assert!(
        s.panel
            .as_ref()
            .unwrap()
            .emoji
            .as_ref()
            .unwrap()
            .popup
            .is_none()
    );
    assert!(
        t.event(&mut s, &motion(1, 45.0, 141.0, 550_000_000))
            .iter()
            .all(|a| a.0 != PANEL_EMOJI)
    );
}

#[test]
fn a_single_rounding_pixel_below_the_grid_does_not_expose_an_offscreen_target() {
    let mut s = scene();
    let p = s.panel.as_mut().unwrap();
    let mut clipped = p
        .items
        .iter()
        .find(|i| matches!(i.command, Command::Emoji(_)))
        .unwrap()
        .clone();
    clipped.id = 399_999;
    clipped.label = "clipped-rounding-pixel".into();
    clipped.bounds.top = clipped.clip.bottom - 1.0;
    clipped.bounds.bottom = clipped.bounds.top + 90.0;
    p.items.push(clipped);
    assert!(
        Semantics::default()
            .snapshot(&s, "unshifted")
            .iter()
            .all(|n| n.label != "clipped-rounding-pixel")
    );
}

#[test]
fn category_tabs_preserve_each_grids_scroll_position_and_refresh_history() {
    let mut s = scene();
    let p = s.panel.as_mut().unwrap();
    p.scroll = 72.0;
    p.activate(Command::Category(1));
    assert_eq!(0.0, p.scroll);
    p.scroll = 108.0;
    p.activate(Command::Category(2));
    assert_eq!(72.0, p.scroll);
    p.activate(Command::Category(1));
    assert_eq!(108.0, p.scroll);
}
