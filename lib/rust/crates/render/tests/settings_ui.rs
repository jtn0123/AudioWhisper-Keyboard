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
#[test]
fn short_dialog_wraps_content() {
    let (mut a, mut text) = app();
    a.config = [480.0, 900.0, 1.0, 1.0];
    a.scene(&mut text);
    assert!(a.activate(1006)); // Spacebar label.
    a.scene(&mut text);
    let first = a.controls.iter().find(|c| c.id == 3000).unwrap();
    let last = a.controls.iter().find(|c| c.id == 3002).unwrap();
    let footer = a.controls.iter().find(|c| c.id == 5).unwrap();
    assert!(footer.bounds.top - last.bounds.bottom < 48.0);
    assert!(first.bounds.top > 200.0); // The small dialog is vertically centered.
}
#[test]
fn dialog_radios_expose_checked_state() {
    let (mut a, mut text) = app();
    a.scrolls[1] = 300.0; // Font-scale 2 rows now use Material line heights; reveal the row.
    a.scene(&mut text);
    assert!(a.activate(1006));
    a.scene(&mut text);
    for radio in a.controls.iter().filter(|c| c.role == "radio") {
        assert_eq!(radio.checked, Some(radio.selected));
    }
}
#[test]
fn radio_checked_semantics_do_not_draw_switch_tracks() {
    let (mut a, mut text) = app();
    a.config = [480.0, 900.0, 1.0, 1.0];
    a.scene(&mut text);
    assert!(a.activate(1006));
    let panel = a.scene(&mut text).panel.unwrap();
    let modal = panel.items.iter().skip_while(|item| {
        item.element != "settings-box" || item.styles[0].background != 0x99000000
    });
    assert!(modal.clone().count() > 1);
    assert!(modal.clone().all(|item| {
        item.element != "settings-box"
            || item.bounds.width() != 52.0
            || item.bounds.height() != 32.0
    }));
}

#[test]
fn material_switch_metrics_palette_and_link_divider_match_tokens() {
    let (mut a, mut text) = app();
    a.config = [480.0, 4000.0, 1.0, 1.0];
    a.prefs
        .insert("keyboard__number_row".into(), "false".into());
    a.prefs
        .insert("keyboard__hinted_symbols_enabled".into(), "true".into());
    let panel = a.scene(&mut text).panel.unwrap();
    let boxes: Vec<_> = panel
        .items
        .iter()
        .filter(|i| i.element == "settings-box")
        .collect();
    assert!(boxes.iter().any(|i| i.bounds.width() == 52.0
        && i.bounds.height() == 32.0
        && i.styles[0].background == a.colors[7]));
    assert!(boxes.iter().any(|i| i.bounds.width() == 16.0
        && i.bounds.height() == 16.0
        && i.styles[0].background == a.colors[7]));
    assert!(boxes.iter().any(|i| i.bounds.width() == 24.0
        && i.bounds.height() == 24.0
        && i.styles[0].background == a.colors[6]));
    assert!(
        boxes
            .iter()
            .any(|i| i.bounds.width() == 2.0 && i.styles[0].background == a.colors[10])
    );
    let heading = panel
        .items
        .iter()
        .find(|i| i.label == "pref__keyboard__group_layout__label")
        .unwrap();
    assert_eq!(heading.bounds.left, 56.0);
    assert_eq!(heading.text.weight, 700);
    assert_eq!(heading.text.foreground, a.colors[9]);
}

#[test]
fn platform_nonlinear_sp_values_and_material_line_heights_are_used_verbatim() {
    let (mut a, mut text) = app();
    a.config = [1080.0, 8192.0, 2.625, 2.0];
    for (sp, px) in [
        (14, 68.25),
        (16, 75.6),
        (20, 89.25),
        (22, 94.5),
        (24, 99.75),
        (28, 110.25),
        (32, 120.75),
    ] {
        a.sp_pixels[sp] = px;
    }
    let panel = a.scene(&mut text).panel.unwrap();
    let title = panel
        .items
        .iter()
        .find(|i| i.label == "pref__keyboard__number_row__label")
        .unwrap();
    assert_eq!(title.text.size, 75.6);
    assert_eq!(title.text.line_height, 0.0); // The app bodyLarge override uses natural font metrics.
    let summary = panel
        .items
        .iter()
        .find(|i| i.label == "pref__keyboard__number_row__summary")
        .unwrap();
    assert_eq!(summary.text.size, 68.25);
    assert_eq!(summary.text.line_height, 89.25);
}

#[test]
fn selection_handles_are_native_primary_teardrops_and_drag_selection() {
    let (mut a, _) = app();
    let mut text = Text::new(vec![
        include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ]);
    a.config = [480.0, 900.0, 1.0, 1.0];
    a.editor.text = "abcdef".encode_utf16().collect();
    a.editor.focused = true;
    assert!(a.editor.select(1, 3));
    let panel = a.scene(&mut text).panel.unwrap();
    let end = a.controls.iter().find(|c| c.id == 21).unwrap().bounds;
    assert!(a.controls.iter().any(|c| c.id == 20));
    assert_eq!(
        panel
            .items
            .iter()
            .filter(|i| i.bounds.width() == 25.0 && i.styles[0].background == a.colors[3])
            .count(),
        2
    );
    let x = (end.left + end.right) / 2.0;
    let y = (end.top + end.bottom) / 2.0;
    for (action, dx, t) in [(0, 0.0, 1), (2, 30.0, 16_000_001), (1, 30.0, 32_000_001)] {
        a.motion(&Motion {
            action,
            index: 0,
            ids: vec![0],
            times: vec![0, t],
            coords: vec![x + dx, y],
        });
        a.scene(&mut text);
    }
    assert_eq!(a.editor.anchor, 1);
    assert!(a.editor.focus > 3);
    assert_eq!(a.editor.text, "abcdef".encode_utf16().collect::<Vec<_>>());
}

#[test]
fn shaping_owners_share_the_platform_font_blob_without_copying_its_payload() {
    use parley::PositionedLayoutItem;
    let blob = render::text::FontBlob::from(
        include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    );
    let mut first = Text::shared(vec![blob.clone()]);
    let mut second = Text::shared(vec![blob.clone()]);
    for owner in [&mut first, &mut second] {
        let layout = owner.layout("Settings", 16.0);
        let mut runs = 0;
        for line in layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(run) = item {
                    assert_eq!(run.run().font().data.id(), blob.id());
                    runs += 1;
                }
            }
        }
        assert!(runs > 0);
    }
}

#[test]
fn material_sliders_use_the_pinned_track_gap_and_vertical_thumb_tokens() {
    let (mut a, mut text) = app();
    a.config = [480.0, 8192.0, 1.0, 1.0];
    a.scene(&mut text);
    assert!(a.activate(1008));
    let panel = a.scene(&mut text).panel.unwrap();
    assert_eq!(
        panel
            .items
            .iter()
            .filter(|i| i.bounds.width() == 4.0 && i.bounds.height() == 44.0)
            .count(),
        2
    );
    assert!(panel.items.iter().any(|i| i.bounds.height() == 16.0
        && i.bounds.width() > 16.0
        && i.styles[0].background == a.colors[3]));
    assert!(panel.items.iter().any(|i| i.bounds.height() == 16.0
        && i.bounds.width() > 16.0
        && i.styles[0].background == a.colors[4]));
}

#[test]
fn modal_keeps_underlying_screen_text_geometry_and_dims_system_bars() {
    let (mut a, mut text) = app();
    a.config = [1080.0, 2300.0, 2.625, 1.0];
    a.colors[0] = 0xffffffff;
    a.colors[12] = 0xffe0e0e0;
    a.scrolls[1] = 100.0;
    let before = a.scene(&mut text);
    let label = a
        .controls
        .iter()
        .find(|c| c.id == 1001)
        .unwrap()
        .label
        .clone();
    let bounds = before
        .panel
        .as_ref()
        .unwrap()
        .items
        .iter()
        .find(|i| i.label == label)
        .unwrap()
        .bounds;
    assert!(a.activate(1002));
    let after = a.scene(&mut text);
    assert_eq!(
        after
            .panel
            .as_ref()
            .unwrap()
            .items
            .iter()
            .find(|i| i.label == label)
            .unwrap()
            .bounds,
        bounds
    );
    assert_eq!(a.system_bar_color(), 0xff5a5a5a);
    assert!(!a.light_system_bars());
    a.back();
    assert_eq!(a.system_bar_color(), 0xffe0e0e0);
    assert!(a.light_system_bars());
}

#[test]
fn reversed_requests_normalize_like_compose_and_off_center_grabs_can_cross_a_collapsed_range() {
    let (mut a, _) = app();
    let mut text = Text::new(vec![
        include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ]);
    a.config = [480.0, 640.0, 1.0, 1.0];
    a.editor.focused = true;
    a.editor.text = "abcdef".encode_utf16().collect();
    assert!(a.editor.select(4, 1));
    // Compose's SetSelectionCommand normalizes reversed requests to min/max.
    assert_eq!((a.editor.anchor, a.editor.focus), (1, 4));
    a.scene(&mut text);
    let handle = a.controls.iter().find(|c| c.id == 21).unwrap().bounds;
    let other = a.controls.iter().find(|c| c.id == 20).unwrap().bounds;
    assert!(handle.left > other.right);
    let (x, y) = (handle.left + 3.0, handle.top + 3.0);
    let motion = |a: &mut App, text: &mut Text, action, x, t| {
        a.motion(&Motion {
            action,
            index: 0,
            ids: vec![0],
            times: vec![0, t],
            coords: vec![x, y],
        });
        a.scene(text);
    };
    motion(&mut a, &mut text, 0, x, 1);
    motion(&mut a, &mut text, 2, x + 16.0, 16_000_001);
    motion(&mut a, &mut text, 1, x + 16.0, 32_000_001);
    assert_eq!(a.editor.anchor, 1);
    assert!(a.editor.focus > 4);
    let handle = a.controls.iter().find(|c| c.id == 21).unwrap().bounds;
    let other = a.controls.iter().find(|c| c.id == 20).unwrap().bounds;
    motion(&mut a, &mut text, 0, handle.left + 3.0, 48_000_001);
    motion(&mut a, &mut text, 2, other.right + 3.0, 64_000_001);
    assert_eq!((a.editor.anchor, a.editor.focus), (1, 1));
    assert!(a.controls.iter().all(|c| c.id != 20 && c.id != 21));
    motion(&mut a, &mut text, 2, other.right - 13.0, 80_000_001);
    motion(&mut a, &mut text, 1, other.right - 13.0, 96_000_001);
    assert_eq!((a.editor.anchor, a.editor.focus), (0, 1));
    assert_eq!(a.scrolls[1], 0.0);
    assert!(a.fling.is_none());
}
