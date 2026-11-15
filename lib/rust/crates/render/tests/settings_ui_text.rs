//! Settings text geometry: Android font sizes, line heights, tracking, ellipsis and the
//! Material 3 list-item and button measurements that depend on them.
use render::{settings_ui::App, text::Text};
fn app() -> (App, Text) {
    let mut a = App {
        config: [480.0, 640.0, 1.0, 2.0],
        ..App::default()
    };
    a.navigate("settings/keyboard");
    (a, Text::new(Vec::<Vec<u8>>::new()))
}
#[test]
fn platform_nonlinear_font_sizes_are_used_and_line_heights_keep_their_ratio() {
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
    // Compose's resolveLineHeightInPx: 20/14 of the scaled 14 sp, not the scaled 20 sp (89.25).
    assert_eq!(summary.text.line_height, 97.5);
}

#[test]
fn rows_follow_material_list_item_padding_and_the_multiline_heuristic() {
    let (mut a, _) = app();
    let mut text = Text::new(vec![
        include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ]);
    a.config = [1080.0, 8192.0, 2.625, 2.0];
    for (sp, px) in [(14, 68.25), (16, 75.6), (20, 89.25), (30, 105.0)] {
        a.sp_pixels[sp] = px;
    }
    for (key, value) in [
        ("pref__keyboard__number_row__label", "Number row"),
        (
            "pref__keyboard__number_row__summary",
            "Show a number row above the character layout",
        ),
        (
            "pref__keyboard__hinted_number_row_mode__label",
            "Hinted number row",
        ),
        (
            "enum__key_hint_mode__smart_priority",
            "Smart prioritization",
        ),
        ("settings__localization__title", "Languages & Layouts"),
    ] {
        a.strings.insert(key.into(), value.into());
    }
    a.scene(&mut text);
    let d = 2.625;
    fn heights(a: &App, text: &mut Text, id: i32, width: f32) -> (f32, f32, f32) {
        let c = a.controls.iter().find(|c| c.id == id).unwrap();
        let title = text.android(&c.label, 75.6, 400, [0.0, 0.0], Some(width));
        let summary = text.android(
            &c.summary,
            68.25,
            400,
            [97.5, 0.2 * 2.625 * 2.0],
            Some(width),
        );
        (c.bounds.height(), title.height, summary.height)
    }
    // Hinted number row: one 97.5 px summary line is not over 30 sp (105 px), so a two-line
    // item, 8 dp padding; Number row's summary wraps, so a three-line item, 12 dp padding.
    let (row, title, summary) = heights(&a, &mut text, 1002, 1080.0 - 144.0 * d);
    assert!(summary <= 105.0, "one summary line: {summary}");
    assert!((row - (title + summary + 16.0 * d).max(72.0 * d)).abs() < 0.01);
    let (row, title, summary) = heights(&a, &mut text, 1001, 1080.0 - 136.0 * d);
    assert!(summary > 105.0, "wrapped summary: {summary}");
    assert!((row - (title + summary + 24.0 * d).max(88.0 * d)).abs() < 0.01);
    a.navigate("settings/home");
    a.scene(&mut text);
    // One-line home items: 8 dp padding around the label, at least 56 dp.
    let (row, title, _) = heights(&a, &mut text, 100, 1080.0 - 72.0 * d);
    assert!(
        (row - (title + 16.0 * d).max(56.0 * d)).abs() < 0.01,
        "{row} {title}"
    );
}

#[test]
fn letter_spacing_is_the_resolved_material_3_tracking() {
    let (mut a, mut text) = app();
    a.config = [1080.0, 8192.0, 2.625, 1.0];
    let panel = a.scene(&mut text).panel.unwrap();
    let item = |label: &str| panel.items.iter().find(|i| i.label == label).unwrap();
    // bodyMedium is 0.2 sp in Material 3 1.4.0; the app's bodyLarge override sets none.
    assert_eq!(
        item("pref__keyboard__number_row__summary")
            .text
            .letter_spacing,
        0.2 * 2.625
    );
    assert_eq!(
        item("pref__keyboard__number_row__label")
            .text
            .letter_spacing,
        0.0
    );
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
fn titles_ellipsize_per_cluster_like_static_layout() {
    // StaticLayout's END ellipsis keeps whole clusters, mid-word, while they and "…" fit; the old
    // word-boundary cut drew "One-handed mode…" where Compose draws "One-handed mode k…".
    let title = "One-handed mode keyboard width";
    let mut a = App {
        config: [1080.0, 2340.0, 2.625, 1.0],
        ..App::default()
    };
    a.strings.insert(
        "pref__keyboard__one_handed_mode_scale_factor__label".into(),
        title.into(),
    );
    a.prefs
        .insert("keyboard__one_handed_mode_enabled".into(), "true".into());
    a.navigate("settings/keyboard");
    let mut text = Text::new(vec![
        include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ]);
    a.scene(&mut text);
    let row = render::settings_ui::catalogue::KEYBOARD
        .iter()
        .position(|r| r.key == "keyboard__one_handed_mode_scale_factor")
        .unwrap();
    a.scrolls[1] = a.max_scroll;
    a.scene(&mut text);
    assert!(a.activate(1000 + row as i32), "open the width dialog");
    let scene = a.scene(&mut text);
    let item = scene
        .panel
        .as_ref()
        .unwrap()
        .items
        .iter()
        .find(|i| i.label.starts_with("One-handed mode ") && i.label.ends_with('…'))
        .expect("an ellipsized dialog title")
        .clone();
    let style = &item.text;
    let width = item.bounds.width();
    let layout = |text: &mut Text, s: &str, wrap: Option<f32>| {
        text.android(
            s,
            style.size,
            style.weight,
            [style.line_height, style.letter_spacing],
            wrap,
        )
    };
    let kept = item.label.trim_end_matches('…');
    assert!(title.starts_with(kept) && kept.len() > "One-handed mode ".len());
    assert_eq!(layout(&mut text, &item.label, Some(width)).lines.len(), 1);
    // One more cluster would not fit beside the ellipsis.
    let next = title[kept.len()..].chars().next().unwrap();
    let longer = layout(&mut text, &format!("{kept}{next}"), None).width;
    assert!(longer + layout(&mut text, "…", None).width > width);
}

#[test]
fn dialog_buttons_measure_in_whole_pixels_like_compose() {
    let (mut a, mut text) = app();
    a.config = [1080.0, 2400.0, 2.625, 1.0];
    a.scene(&mut text);
    assert!(a.activate(1006)); // Spacebar label.
    let panel = a.scene(&mut text).panel.unwrap();
    let cancel = a.controls.iter().find(|c| c.id == 6).unwrap().bounds;
    let ok = a.controls.iter().find(|c| c.id == 7).unwrap().bounds;
    // roundToPx(58 dp) = 152 (not 152.25); roundToPx(8 dp) = 21 between the buttons.
    assert_eq!(ok.width(), 152.0);
    assert_eq!(ok.left - cancel.right, 21.0);
    // With no fonts the label is 0 px wide: it sits roundToPx(12 dp) = 32 px in, plus the
    // rounded half of the 88 px left over (31.5 + 44 drawn as 75.5 would be a half pixel off).
    let label = panel
        .items
        .iter()
        .find(|i| i.label == "action__ok" && i.bounds.top >= ok.top)
        .unwrap();
    assert_eq!(label.bounds.left - ok.left, 76.0);
    // 48 dp touch box (126) around the 40 dp button (105): 11 + 21 + round((63 - 53) / 2).
    assert_eq!(label.bounds.top - ok.top, 37.0);
}
