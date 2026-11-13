use keys::{Bounds, Rect, codes::*};
use render::{
    accessibility::Semantics,
    accessibility_labels::Labels,
    protocol::*,
    scene::{Key, Popup, Scene},
    smartbar::Config,
    smartbar_scene::{Action, Smartbar},
    touch::Action as Packet,
};

fn scene() -> Scene {
    let rect = Rect {
        left: 0.0,
        top: 0.0,
        right: 50.0,
        bottom: 60.0,
    };
    let mut bar = Smartbar::new(
        Config::decode(&[1., 2., 0., 0., 0., 0., 1., 1., 3., 0., 1., 40.]).unwrap(),
        vec![
            Action {
                code: VOICE_INPUT,
                enabled: true,
                ..Default::default()
            },
            Action {
                code: READ_ALOUD,
                enabled: true,
                ..Default::default()
            },
            Action {
                code: IME_UI_MODE_MEDIA,
                enabled: true,
                ..Default::default()
            },
            Action {
                code: TOGGLE_ACTIONS_OVERFLOW,
                enabled: true,
                ..Default::default()
            },
            Action {
                enabled: true,
                ..Default::default()
            },
            Action {
                code: TOGGLE_AUTOCORRECT,
                enabled: true,
                ..Default::default()
            },
            Action {
                code: SETTINGS,
                enabled: false,
                ..Default::default()
            },
            Action {
                code: 0,
                label: "x".into(),
                enabled: true,
                ..Default::default()
            },
        ],
    );
    bar.intrinsic = vec![50., 60., 70.];
    bar.relayout(600.);
    Scene {
        width: 600,
        height: 300,
        strip: 40.,
        desired_key: [50., 60.],
        keys: [SHIFT, 101, DELETE, VIEW_SYMBOLS]
            .iter()
            .enumerate()
            .map(|(i, &code)| Key {
                code,
                label: if i == 1 { "e".into() } else { String::new() },
                hint: String::new(),
                icon: String::new(),
                bounds: Bounds {
                    touch: Rect {
                        left: i as f32 * 50.,
                        right: (i + 1) as f32 * 50.,
                        ..rect
                    },
                    visible: rect,
                },
                styles: Default::default(),
                hints: Default::default(),
                enabled: true,
            })
            .collect(),
        candidates: vec!["one".into(), "two".into(), "three".into()],
        candidate_tokens: vec![10, 20, 30],
        smartbar: Some(bar),
        ..Default::default()
    }
}
#[test]
fn stable_slots_states_and_control_labels() {
    let mut s = scene();
    let mut a = Semantics {
        popup_keys: vec![false, true],
        ..Default::default()
    };
    let before = a.snapshot(&s, "unshifted");
    for (state, label, checked, locked) in [
        ("unshifted", "Shift", false, false),
        ("shifted_manual", "Shift enabled", true, false),
        ("shifted_automatic", "Shift enabled", true, false),
        ("caps_lock", "Caps lock enabled", true, true),
    ] {
        s.keys[1].label = if checked { "E".into() } else { "e".into() };
        let n = a.snapshot(&s, state);
        assert_eq!(
            n.iter().map(|n| n.id).collect::<Vec<_>>(),
            before.iter().map(|n| n.id).collect::<Vec<_>>()
        );
        assert_eq!(n[0].description, label);
        assert_eq!(n[0].checked, Some(checked));
        assert_eq!(n[0].locked, locked);
        assert_eq!(n[1].bounds.top, 40.);
        assert!(n[1].actions.contains(&ACCESS_LONG_CLICK));
        assert_eq!(n[2].label, "Delete");
        assert_eq!(n[3].label, "Symbols");
    }
    s.keys[1].code = 33;
    s.keys[1].label = "!".into();
    s.keys[2].enabled = false;
    s.pressed = vec![1];
    let n = a.snapshot(&s, "unshifted");
    assert_eq!(n[1].label, "Exclamation mark");
    assert!(n[1].pressed);
    assert!(n[2].actions.is_empty());
    assert!(a.activate(&mut s, "unshifted", 3, ACCESS_CLICK).is_empty());
    assert_eq!(
        a.activate(&mut s, "unshifted", 1, ACCESS_LONG_CLICK),
        vec![Packet(render::protocol::CAPS_LOCK, 0, -1)]
    );
}
#[test]
fn smartbar_candidates_expansion_localization_and_identity() {
    let mut s = scene();
    let mut a = Semantics::default();
    a.labels
        .0
        .insert("quick_action__voice_input".into(), "Localized mic".into());
    let first = a.snapshot(&s, "unshifted");
    assert!(
        first
            .iter()
            .any(|n| n.label == "Localized mic" && n.role == "button")
    );
    assert!(
        first
            .iter()
            .any(|n| n.label == "Expand actions" && n.checked == Some(false))
    );
    let old = first.iter().find(|n| n.label == "two").unwrap().id;
    s.candidates.swap(0, 1);
    s.candidate_tokens.swap(0, 1);
    assert_eq!(
        a.snapshot(&s, "caps_lock")
            .iter()
            .find(|n| n.label == "two")
            .unwrap()
            .id,
        old
    );
    s.smartbar.as_mut().unwrap().config.shared = true;
    s.smartbar.as_mut().unwrap().relayout(600.);
    a.checked_codes = vec![TOGGLE_AUTOCORRECT];
    a.action_text = vec![String::new(); 8];
    a.action_text[7] = "full insert text".into();
    let n = a.snapshot(&s, "unshifted");
    assert!(
        n.iter()
            .any(|n| n.label == "Collapse actions" && n.checked == Some(true))
    );
    assert!(
        n.iter()
            .any(|n| n.label == "Autocorrect" && n.role == "toggle" && n.checked == Some(true))
    );
    assert!(
        n.iter()
            .any(|n| n.label == "Settings" && !n.enabled && n.actions.is_empty())
    );
    assert!(n.iter().any(|n| n.label == "full insert text"));
    assert!(!n.iter().any(|n| n.role == "candidate"));
    assert_eq!(
        Labels::default().action(VOICE_INPUT, "", 1),
        "Stop dictating"
    );
    assert_eq!(Labels::default().action(READ_ALOUD, "", 2), "Stop reading");
}
#[test]
fn popup_focus_choices_have_exact_grid_bounds_and_restore_cancel() {
    let mut s = scene();
    let r = Rect {
        left: 50.0,
        top: 100.0,
        right: 100.0,
        bottom: 160.0,
    };
    s.keys[1].bounds = Bounds {
        touch: r,
        visible: r,
    };
    let mut a = Semantics {
        popup_keys: vec![false, true],
        ..Default::default()
    };
    assert_eq!(
        a.activate(&mut s, "unshifted", 2, ACCESS_LONG_CLICK),
        vec![Packet(KEY_DOWN, 1, -1), Packet(OPEN_POPUP, 1, -1)]
    );
    s.popup = Some(Popup {
        key: 1,
        labels: vec!["3".into(), "é".into(), "è".into()],
        selected: 0,
    });
    render::popup_scene::configure(&mut s, 1, false).unwrap();
    let n = a.snapshot(&s, "unshifted");
    let popup: Vec<_> = n.iter().filter(|n| n.role == "popup item").collect();
    assert_eq!(popup.len(), 3);
    assert_eq!(popup.iter().filter(|n| n.selected).count(), 1);
    for (i, n) in popup.iter().enumerate() {
        assert_eq!(n.bounds, s.popup_choice_bounds(i).unwrap());
    }
    let id = popup.iter().find(|n| n.label == "é").unwrap().id;
    assert_eq!(
        a.activate(&mut s, "unshifted", id, ACCESS_CLICK),
        vec![Packet(POPUP_SELECT, 1, -1)]
    );
    assert!(s.popup.is_none());
    assert!(s.pressed.is_empty());
    assert!(a.activate(&mut s, "unshifted", id, ACCESS_CLICK).is_empty());
    a.popup_source = Some(1);
    s.popup = Some(Popup {
        key: 1,
        labels: vec!["é".into()],
        selected: 0,
    });
    let id = a.snapshot(&s, "unshifted").last().unwrap().id;
    assert_eq!(
        a.activate(&mut s, "unshifted", id, ACCESS_DISMISS),
        vec![Packet(KEY_CANCEL, 1, -1)]
    );
}

#[test]
fn popup_bounds_candidate_clipping_and_editor_action_labels_follow_the_scene() {
    let mut s = scene();
    s.popup = Some(Popup {
        key: 1,
        labels: vec!["é".into()],
        selected: 0,
    });
    render::popup_scene::configure(&mut s, 0, false).unwrap();
    let a = Semantics::default();
    let nodes = a.snapshot(&s, "unshifted");
    assert_eq!(
        nodes.last().unwrap().bounds,
        Rect {
            left: -2.5,
            top: -50.0,
            right: 52.5,
            bottom: 10.0
        }
    );
    // A scrolled candidate's semantics stop at the same center clip used by painting/hit tests.
    s.smartbar.as_mut().unwrap().layout.candidates[0]
        .bounds
        .left = -100.0;
    let n = a.snapshot(&s, "unshifted");
    let candidate = n.iter().find(|n| n.label == "one").unwrap();
    assert_eq!(
        candidate.bounds.left,
        s.smartbar.as_ref().unwrap().layout.center.left
    );
    let labels = Labels::default();
    for (action, expected) in [
        (0, "Return"),
        (1, "Return"),
        (2, "Go"),
        (3, "Search"),
        (4, "Send"),
        (5, "Next"),
        (6, "Done"),
        (7, "Previous"),
    ] {
        assert_eq!(labels.enter(action, 0), expected);
        assert_eq!(labels.enter(action | 0x4000_0000, 0), "Return");
        assert_eq!(labels.enter(action, 0x0002_0001), "Return");
    }
    for (mode, expected) in [
        ("characters", "Letters"),
        ("symbols", "Symbols"),
        ("symbols2", "More symbols"),
        ("numeric", "Numeric"),
        ("numeric_advanced", "Numeric"),
        ("phone", "Phone"),
        ("phone2", "Phone"),
    ] {
        assert_eq!(labels.mode(mode), expected);
    }
}

#[test]
fn key_semantics_clip_extended_touch_targets_to_the_renderer_viewport() {
    let mut s = scene();
    s.keys[1].bounds.touch = Rect {
        left: -20.,
        top: -80.,
        right: 650.,
        bottom: 400.,
    };
    let nodes = Semantics::default().snapshot(&s, "unshifted");
    let key = nodes.iter().find(|n| n.id == 2).unwrap();
    assert_eq!(
        key.bounds,
        Rect {
            left: 0.,
            top: 0.,
            right: 600.,
            bottom: 300.
        }
    );
    // A completely clipped physical slot must not create an unreachable virtual node.
    s.keys[1].bounds.touch.top = 400.;
    assert!(
        !Semantics::default()
            .snapshot(&s, "unshifted")
            .iter()
            .any(|n| n.id == 2)
    );
}
