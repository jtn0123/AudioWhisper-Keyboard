use render::{settings_ui::App, text::Text, touch::Motion};
fn app() -> (App, Text) {
    let mut a = App {
        config: [480.0, 640.0, 1.0, 2.0],
        ..App::default()
    };
    a.navigate("settings/keyboard");
    (a, Text::new(Vec::<Vec<u8>>::new()))
}
#[test]
fn scroll_fling_cancel_and_pointer_loss_do_not_activate_a_control() {
    let (mut a, mut text) = app();
    a.scene(&mut text);
    let original = a.prefs.clone();
    for (action, y, time) in [
        (0, 400.0, 1),
        (2, 200.0, 50_000_001),
        (2, 100.0, 70_000_001),
        (1, 100.0, 80_000_001),
    ] {
        a.motion(&Motion {
            action,
            index: 0,
            ids: vec![9],
            times: vec![0, time],
            coords: vec![470.0, y],
        });
        a.scene(&mut text);
    }
    assert!(a.scroll() > 0.0);
    assert!(a.fling.is_some());
    assert_eq!(a.prefs, original);
    a.tick(100_000_001);
    a.scene(&mut text);
    assert!(a.scroll() <= a.max_scroll);
    a.motion(&Motion {
        action: 3,
        index: 0,
        ids: vec![9],
        times: vec![0, 110_000_001],
        coords: vec![470.0, 100.0],
    });
    assert!(a.fling.is_none());
    assert!(a.down.is_none());
    assert_eq!(a.prefs, original);
    a.motion(&Motion {
        action: 0,
        index: 0,
        ids: vec![1],
        times: vec![0, 120_000_001],
        coords: vec![50.0, 100.0],
    });
    a.motion(&Motion {
        action: 5,
        index: 1,
        ids: vec![1, 2],
        times: vec![0, 130_000_001],
        coords: vec![50.0, 100.0, 55.0, 105.0],
    });
    assert!(a.down.is_none());
    assert_eq!(a.prefs, original);
}
#[test]
fn imported_nonfinite_and_out_of_range_slider_values_never_reach_geometry() {
    for raw in ["NaN", "Infinity", "-Infinity", "-500", "500"] {
        let (mut a, mut text) = app();
        a.config[1] = 8192.0;
        a.prefs
            .insert("keyboard__key_spacing_vertical".into(), raw.into());
        a.scene(&mut text);
        assert!(a.activate(1015));
        let scene = a.scene(&mut text);
        assert!(scene.panel.unwrap().items.iter().all(|i| {
            [i.bounds.left, i.bounds.top, i.bounds.right, i.bounds.bottom]
                .iter()
                .all(|v| v.is_finite())
        }));
        for c in &a.controls {
            if let Some(range) = c.range {
                assert!(range.iter().all(|v| v.is_finite()));
                assert!((range[0]..=range[1]).contains(&range[2]));
            }
        }
        a.activate(7);
        assert_eq!(a.prefs["keyboard__key_spacing_vertical"], raw);
    }
}
#[test]
fn only_modal_controls_are_exposed_and_cancelled_back_preserves_drafts() {
    let (mut a, mut text) = app();
    a.focus = 1008;
    a.reveal_focus = true;
    a.scene(&mut text);
    assert!(a.activate(1008));
    a.set_slider(0, 125.0);
    a.scene(&mut text);
    assert!(
        a.controls
            .iter()
            .all(|c| [4, 5, 6, 7, 4000, 4001].contains(&c.id))
    );
    let before = a.modal.as_ref().unwrap().values.clone();
    a.back_progress = 0.8;
    a.back_progress = 0.0;
    assert_eq!(a.modal.as_ref().unwrap().values, before);
    assert!(a.effects.is_empty());
    a.back();
    assert!(a.modal.is_none());
    assert!(a.effects.is_empty());
}
