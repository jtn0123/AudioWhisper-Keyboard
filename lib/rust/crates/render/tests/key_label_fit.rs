//! A key's label too big for its key stays inside it: under every `text-max-lines` and
//! `text-overflow` a theme can set, as Compose's `SnyggText` broke and ended it, and, where the
//! theme sets neither, squeezed, shrunk and ellipsised as a suggestion is. See docs/rust.md,
//! "A label too big for its key".
mod key_label_fit_common;

use key_label_fit_common::*;
use keys::Rect;
use render::{
    label_fit::{Align, Fitted, Overflow, TextRules, fit},
    scene::{Key, Popup, Scene},
    style::Style,
    text::Text,
    theme::{Request, Theme},
};

fn fitted(label: &str, style: &Style, room: [f32; 2]) -> Fitted {
    fit(&mut Text::new(fonts()), label, style, room)
}

fn texts(f: &Fitted) -> Vec<&str> {
    f.lines.iter().map(|l| l.text.as_str()).collect()
}

/// Every red pixel of `scene` drawn is inside `r`, and there are some.
fn inside(scene: &Scene, r: Rect, what: &str) -> render::raster::Raster {
    let raster = draw(scene);
    let (inn, out) = red_in_and_out(&raster, scene, r);
    assert_eq!(out, 0, "{what}: {out} red pixels outside its box");
    assert!(inn > 50, "{what}: only {inn} red pixels drawn");
    raster
}

#[test]
fn a_label_too_wide_stays_inside_its_key_under_every_rule() {
    for rules in every_rule() {
        let scene = row(LONG, &style(40.0, rules), 150.0);
        let raster = inside(&scene, key_box(&scene, 1), &format!("{rules:?}"));
        assert_eq!(raster.label_fits(), 1, "{rules:?}");
    }
}

#[test]
fn a_label_that_fits_draws_where_it_always_did_and_is_never_fitted() {
    for rules in every_rule() {
        let scene = row("abc", &style(40.0, rules), 150.0);
        let raster = inside(&scene, key_box(&scene, 1), "abc");
        assert_eq!(raster.label_fits(), 0, "{rules:?}");
        let ink = red_bounds(&raster, &scene).expect("ink");
        let key = key_box(&scene, 1);
        let (left, right) = (ink.left - key.left, key.right - ink.right);
        assert!((left - right).abs() <= 2.0, "centred: {left} and {right}");
    }
}

#[test]
fn where_the_theme_says_nothing_a_label_squeezes_then_shrinks_then_ellipsises() {
    let style = style(40.0, TextRules::default());
    let mut text = Text::new(fonts());
    let (width, height) = (
        text.measure("Wide", 40.0, false, 400),
        text.height("Wide", 40.0, false, 400),
    );
    let squeezed = fitted("Wide", &style, [width * 0.85, ROW]);
    assert_eq!((texts(&squeezed), squeezed.size), (vec!["Wide"], 40.0));
    assert!((0.70..=0.85).contains(&squeezed.squeeze), "{squeezed:?}");
    let shrunk = fitted("Wide", &style, [width * 0.6, ROW]);
    assert_eq!((texts(&shrunk), shrunk.squeeze), (vec!["Wide"], 0.70));
    assert!((30.0..40.0).contains(&shrunk.size), "{shrunk:?}");
    let cut = fitted(LONG, &style, [width, ROW]);
    assert_eq!((cut.squeeze, cut.size), (0.70, 30.0));
    let line = &cut.lines[0].text;
    assert!(line.ends_with('…') && LONG.starts_with(line.trim_end_matches('…')));
    assert!(cut.width * cut.squeeze <= width, "{cut:?}");
    let short = fitted("Wide", &style, [width * 2.0, height * 0.8]);
    assert_eq!(short.squeeze, 1.0);
    assert!(
        short.size < 40.0 && short.height <= height * 0.8 + 1.0,
        "{short:?}"
    );
    assert!(!cut.hangs && !cut.spills);
}

#[test]
fn compose_rules_break_cut_and_end_a_label_as_snygg_text_did() {
    let room = [150.0, ROW];
    let one = |overflow| fitted(LONG, &style(40.0, rules(Some(1), overflow)), room);
    let clipped = one(Overflow::Clip);
    assert_eq!(clipped.lines.len(), 1);
    assert!(LONG.starts_with(&clipped.lines[0].text) && !clipped.lines[0].text.contains('…'));
    let ended = one(Overflow::Ellipsis);
    let line = &ended.lines[0];
    assert!(line.text.ends_with('…') && LONG.starts_with(line.text.trim_end_matches('…')));
    assert!(line.width <= 150.0 && line.left == 0.0, "{ended:?}");
    assert_eq!(
        texts(&one(Overflow::Unset)),
        texts(&clipped),
        "clip is the default"
    );
    assert!(one(Overflow::Visible).spills && !clipped.spills);

    let two = fitted(LONG, &style(40.0, rules(Some(2), Overflow::Clip)), room);
    assert_eq!(two.lines.len(), 2);
    assert!(LONG.starts_with(&texts(&two).join(" ")), "{two:?}");
    let two = fitted(LONG, &style(40.0, rules(Some(2), Overflow::Ellipsis)), room);
    assert!(two.lines.len() == 2 && two.lines[1].text.ends_with('…'));

    // With an ellipsis, no more lines than fit the height; clipped, every line, hung from the top.
    let line_height = two.lines[0].height;
    let low = [150.0, line_height * 2.5];
    let capped = fitted(LONG, &style(40.0, rules(None, Overflow::Ellipsis)), low);
    assert_eq!(capped.lines.len(), 2, "{capped:?}");
    assert!(capped.lines[1].text.ends_with('…') && capped.height <= low[1]);
    let all = fitted(LONG, &style(40.0, rules(None, Overflow::Clip)), low);
    assert!(
        all.lines.len() > 2 && all.height > low[1] && all.hangs,
        "{all:?}"
    );
    assert!(all.lines.iter().all(|l| l.width <= 150.0));
    assert!(
        all.width <= 150.0 && two.width <= 150.0,
        "no wider than the box"
    );
}

#[test]
fn a_paragraph_taller_than_its_key_hangs_from_the_keys_top() {
    // Three lines of 40px in a key 100px tall: Compose draws the first whole, from the top, and
    // clips the last; centred, the first would be cut at the key's top edge.
    let scene = row(LONG, &style(40.0, rules(None, Overflow::Clip)), 150.0);
    let key = key_box(&scene, 1);
    let raster = inside(&scene, key, "hung");
    let ink = red_bounds(&raster, &scene).expect("ink");
    assert!(
        ink.top - key.top >= 3.0,
        "the first line whole: {ink:?} in {key:?}"
    );
    assert_eq!(ink.bottom, key.bottom, "the last line clipped");
}

#[test]
fn text_max_lines_cuts_a_label_at_its_line_break() {
    let room = [150.0, 30.0];
    let cut = |overflow| fitted("1 2\n3 4", &style(24.0, rules(Some(1), overflow)), room);
    assert_eq!(texts(&cut(Overflow::Clip)), ["1 2"]);
    assert_eq!(texts(&cut(Overflow::Ellipsis)), ["1 2…"]);
    // Two lines are allowed, as Floris themes allow the `1 2 / 3 4` key, so it is drawn whole.
    let scene = row(
        "1 2\n3 4",
        &style(24.0, rules(Some(2), Overflow::Unset)),
        150.0,
    );
    assert_eq!(
        inside(&scene, key_box(&scene, 1), "1 2/3 4").label_fits(),
        0
    );
    let scene = row(
        "1 2\n3 4",
        &style(24.0, rules(Some(1), Overflow::Unset)),
        150.0,
    );
    assert_eq!(inside(&scene, key_box(&scene, 1), "1 2").label_fits(), 1);
}

#[test]
fn text_align_places_each_line_of_a_label_that_fills_its_key() {
    let mut lefts = Vec::new();
    for align in [
        Align::Start,
        Align::Center,
        Align::End,
        Align::Left,
        Align::Right,
    ] {
        let rules = TextRules {
            align,
            ..rules(Some(2), Overflow::Clip)
        };
        let f = fitted(LONG, &style(40.0, rules), [150.0, ROW]);
        let line = &f.lines[1];
        lefts.push((line.left, f.width - line.width));
    }
    let free = lefts[0].1;
    assert!(free > 1.0, "the second line is the shorter: {lefts:?}");
    let want = [0.0, free / 2.0, free, 0.0, free];
    for (i, (left, _)) in lefts.iter().enumerate() {
        assert!((left - want[i]).abs() < 0.01, "{i}: {lefts:?}");
    }
}

fn floris_day(space_size: bool) -> Theme {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../app/src/main/assets/ime/theme/org.florisboard.themes/stylesheets/floris_day.json"
    );
    let mut json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("theme")).expect("json");
    if !space_size {
        json["key[code=32]"]
            .as_object_mut()
            .expect("space rule")
            .remove("font-size");
    }
    Theme::new(&json.to_string(), vec![], vec![], 2.625, 1.0, 1.0).expect("theme")
}

fn request(element: &str, code: i32) -> Request {
    Request {
        element: element.into(),
        mode: "characters".into(),
        shift: "unshifted".into(),
        code,
        key_type: "character".into(),
        selector: 0,
        auto_commit: false,
        popup_source: None,
    }
}

#[test]
fn floris_themes_set_the_rules_compose_drew_keys_with() {
    let mut theme = floris_day(true);
    let space = theme.style(&request("key", 32));
    assert_eq!(space.rules, rules(Some(1), Overflow::Ellipsis));
    assert_eq!(
        theme.style(&request("key", 97)).rules,
        rules(Some(1), Overflow::Unset)
    );
    assert_eq!(theme.style(&request("key", -205)).rules.max_lines, Some(2));
    assert_eq!(
        theme.style(&request("key-hint", 97)).rules.max_lines,
        Some(1)
    );
    assert!(!theme.style(&request("key-popup-box", 97)).rules.set());
}

#[test]
fn the_space_keys_language_name_ends_in_an_ellipsis_from_the_keys_start() {
    // The theme on the owed list: Floris Day with the space key's font size taken out, so the
    // language name takes the keys' 22sp. Compose drew "English (Unite…" from the key's start.
    let mut theme = floris_day(false);
    let space = Style {
        foreground: RED,
        ..theme.style(&request("key", 32))
    };
    assert_eq!(space.size, 22.0 * 2.625);
    let scene = row(LONG, &space, 420.0);
    let key = key_box(&scene, 1);
    let raster = inside(&scene, key, "space");
    assert_eq!(raster.label_fits(), 1);
    let ink = red_bounds(&raster, &scene).expect("ink");
    assert!(
        ink.left - key.left < 6.0,
        "from the key's start: {ink:?} in {key:?}"
    );
    let line = &fitted(LONG, &space, [key.width(), key.height()]).lines[0].text;
    assert!(line.ends_with('…') && line.len() > 10, "{line}");
    // With its 12sp, as the theme ships, the name fits and draws as it always did.
    let mut theme = floris_day(true);
    let space = Style {
        foreground: RED,
        ..theme.style(&request("key", 32))
    };
    assert_eq!(
        inside(&row(LONG, &space, 420.0), key, "12sp").label_fits(),
        0
    );
}

#[test]
fn a_hint_too_big_stays_inside_its_key_at_its_top() {
    // Floris Day's sizes: the hint's corner is 0.6 by 0.7 of the key's 22sp; the hint is 12sp.
    let hint = style(12.0 * 2.625, rules(Some(1), Overflow::Unset));
    let mut scene = row(
        "a",
        &style(22.0 * 2.625, rules(Some(1), Overflow::Unset)),
        150.0,
    );
    scene.keys[1].hint = "%".into();
    scene.keys[1].hints = [hint.clone(), hint];
    let key = key_box(&scene, 1);
    let raster = inside(&scene, key, "small hint");
    assert_eq!(raster.label_fits(), 0, "a hint that fits its corner");
    // From here only the hint is red. One too big for its corner sits at the key's top end, as
    // Compose aligns it; one too wide for the key fills the key's width from its start.
    let black = Style {
        size: 22.0 * 2.625,
        ..Style::default()
    };
    scene.keys[1].styles = [black.clone(), black];
    let corner = 22.0 * 2.625 * 0.6;
    for (hint, end) in [("%", true), ("%%%%%%", false)] {
        scene.keys[1].hint = hint.into();
        scene.keys[1].hints = [
            style(
                if end { 50.0 } else { 60.0 },
                rules(Some(1), Overflow::Unset),
            ),
            Style::default(),
        ];
        let raster = inside(&scene, key, hint);
        assert_eq!(raster.label_fits(), 1);
        let ink = red_bounds(&raster, &scene).expect("ink");
        assert!(ink.top - key.top < 20.0, "{hint} at the top: {ink:?}");
        if end {
            assert!(key.right - ink.right < 6.0, "{hint} at the end: {ink:?}");
        } else {
            assert!(
                ink.width() > corner + 5.0,
                "{hint} wider than its corner: {ink:?}"
            );
        }
    }
}

#[test]
fn a_label_stays_out_of_its_keys_padding() {
    // Compose's key box pads its label. Each padding below is room the label, centred in the
    // key as it was, would reach into.
    for (label, padding, rules) in [
        (
            "abcd",
            [60.0, 0.0, 0.0, 0.0],
            rules(Some(1), Overflow::Unset),
        ),
        ("ab", [0.0, 40.0, 0.0, 0.0], rules(Some(1), Overflow::Unset)),
        (
            "abcd",
            [0.0, 0.0, 60.0, 0.0],
            rules(Some(1), Overflow::Unset),
        ),
        ("ab", [0.0, 0.0, 0.0, 40.0], rules(Some(1), Overflow::Unset)),
        (LONG, [0.0, 0.0, 0.0, 30.0], rules(None, Overflow::Clip)),
    ] {
        let padded = Style {
            padding,
            ..style(40.0, rules)
        };
        let scene = row(label, &padded, 150.0);
        let key = key_box(&scene, 1);
        let room = Rect {
            left: key.left + padding[0],
            top: key.top + padding[1],
            right: key.right - padding[2],
            bottom: key.bottom - padding[3],
        };
        assert_eq!(inside(&scene, room, label).label_fits(), 1, "{padding:?}");
    }
}

#[test]
fn a_phone_pads_digit_too_big_stays_inside_its_key() {
    let mut scene = row("1", &style(30.0, TextRules::default()), 150.0);
    scene.telpad = vec![false, true, false];
    let key = key_box(&scene, 1);
    let raster = inside(&scene, key, "digit");
    assert_eq!(raster.label_fits(), 0);
    // A digit that fits keeps its side of the key, a quarter of the room from its left.
    let ink = red_bounds(&raster, &scene).expect("ink");
    assert!(
        (ink.left + ink.right) / 2.0 < (key.left + key.right) / 2.0 - 10.0,
        "{ink:?}"
    );
    scene.keys[1].label = "1234567890123".into();
    assert_eq!(inside(&scene, key, "digits").label_fits(), 1);
}

#[test]
fn a_long_press_choice_too_wide_stays_inside_its_box() {
    let mut scene = row("", &Style::default(), 150.0);
    scene.popup = Some(Popup {
        key: 1,
        labels: vec![LONG.into(), "x".into()],
        selected: 0,
    });
    let choice = style(40.0, TextRules::default());
    scene.popup_styles = vec![[choice.clone(), choice.clone()], [choice.clone(), choice]];
    let r = scene.popup_choice_bounds(0).expect("choice");
    let raster = draw(&scene);
    let (inn, _) = red_in_and_out(&raster, &scene, r);
    let x = scene.popup_choice_bounds(1).expect("x");
    let (_, out) = red_in_and_out(&raster, &scene, union(r, x));
    assert!(inn > 50 && out == 0, "{inn} in, {out} out");
    assert_eq!(raster.label_fits(), 1);
}

fn union(a: Rect, b: Rect) -> Rect {
    render::scene::union(a, b)
}

#[test]
fn a_fit_is_kept_and_made_again_for_a_new_label_size_theme_or_key() {
    let base = style(40.0, rules(Some(1), Overflow::Ellipsis));
    let scene = row(LONG, &base, 150.0);
    let mut raster = draw(&scene);
    let mut again = |scene: &Scene, made: usize, what: &str| {
        raster.draw(scene, scene.buffer_rect()).expect("raster");
        let (inn, out) = red_in_and_out(&raster, scene, key_box(scene, 1));
        assert!(inn > 50 && out == 0, "{what}: {inn} in, {out} out");
        assert_eq!(raster.label_fits(), made, "{what}");
    };
    again(&scene, 1, "the same frame again");
    again(
        &row("Deutsch (Deutschland)", &base, 150.0),
        2,
        "a new label",
    );
    again(
        &row(
            LONG,
            &Style {
                size: 50.0,
                ..base.clone()
            },
            150.0,
        ),
        3,
        "a new size",
    );
    let clip = rules(Some(1), Overflow::Clip);
    again(
        &row(
            LONG,
            &Style {
                rules: clip,
                ..base.clone()
            },
            150.0,
        ),
        4,
        "a new theme",
    );
    let mut narrower = scene.clone();
    narrower.keys[1].bounds.visible.right -= 30.0;
    again(&narrower, 5, "a narrower key");
    again(&scene, 5, "the first frame, its fit kept");
}

#[test]
fn a_keyboard_whose_every_label_is_too_big_keeps_every_fit() {
    // A theme too big for every key overflows each label and hint, some 80 on a letter layout.
    let mut scene = row(
        LONG,
        &style(40.0, rules(Some(1), Overflow::Ellipsis)),
        150.0,
    );
    let key = scene.keys[1].clone();
    scene.keys = (0..120)
        .map(|i| Key {
            label: format!("{LONG} {i}"),
            ..key.clone()
        })
        .collect();
    let mut raster = draw(&scene);
    assert_eq!(raster.label_fits(), 120, "each label fitted once");
    raster.draw(&scene, scene.buffer_rect()).expect("raster");
    assert_eq!(
        raster.label_fits(),
        120,
        "every fit kept for the next frame"
    );
}
