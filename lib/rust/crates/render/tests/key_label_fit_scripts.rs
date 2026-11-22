//! A label too big for its box stays inside it in Hebrew, which runs right to left, and in CJK,
//! which breaks between its characters, and the press bubble's label stays inside the bubble.
//! Noto Sans Hebrew and Noto Sans CJK JP subsets draw them (tests/fixtures/NOTO-TEXT-SUBSETS.md).
mod key_label_fit_common;

use key_label_fit_common::*;
use render::{
    label_fit::{Overflow, fit},
    popup_scene::headroom,
    press_bubble::{Bubble, mark},
    scene::Scene,
    style::Style,
    text::Text,
};

const HEBREW: &str = "עברית (ישראל)";
const CJK: &str = "中文（简体，中国）日本語";

/// `label` in a key too narrow for it stays inside the key under every rule, and is fitted once.
/// It is drawn from its Noto subset, not as Roboto's missing-glyph boxes.
fn stays_inside(label: &str) {
    let roboto = Text::new(fonts().into_iter().take(1)).measure(label, 40.0, false, 400);
    assert_ne!(Text::new(fonts()).measure(label, 40.0, false, 400), roboto);
    for rules in every_rule() {
        let scene = row(label, &style(40.0, rules), 150.0);
        let raster = draw(&scene);
        let (inn, out) = red_in_and_out(&raster, &scene, key_box(&scene, 1));
        assert!(inn > 50 && out == 0, "{rules:?}: {inn} in, {out} out");
        assert_eq!(raster.label_fits(), 1, "{rules:?}");
    }
}

#[test]
fn a_right_to_left_label_too_wide_stays_inside_its_key_from_its_right() {
    stays_inside(HEBREW);
    let two = style(40.0, rules(Some(2), Overflow::Clip));
    let f = fit(&mut Text::new(fonts()), HEBREW, &two, [150.0, ROW]);
    assert_eq!(f.lines.len(), 2, "{f:?}");
    let short = f
        .lines
        .iter()
        .min_by(|a, b| a.width.total_cmp(&b.width))
        .expect("line");
    assert!(
        short.left > 1.0,
        "a line starts at the paragraph's right: {f:?}"
    );
    assert!((short.left + short.width - f.width).abs() < 0.01, "{f:?}");
    let one = style(40.0, rules(Some(1), Overflow::Ellipsis));
    let line = &fit(&mut Text::new(fonts()), HEBREW, &one, [150.0, ROW]).lines[0];
    assert!(line.text.ends_with('…'), "{line:?}");
    let scene = row(HEBREW, &one, 150.0);
    let (raster, key) = (draw(&scene), key_box(&scene, 1));
    let ink = red_bounds(&raster, &scene).expect("ink");
    assert!(
        key.right - ink.right < 6.0,
        "drawn from the key's right: {ink:?} in {key:?}"
    );
}

#[test]
fn cjk_too_wide_breaks_between_its_characters_and_stays_inside_its_key() {
    stays_inside(CJK);
    let two = style(40.0, rules(Some(2), Overflow::Clip));
    let f = fit(&mut Text::new(fonts()), CJK, &two, [150.0, ROW]);
    let joined: String = f.lines.iter().map(|l| l.text.as_str()).collect();
    assert!(f.lines.len() == 2 && CJK.starts_with(&joined), "{f:?}");
    assert!(
        f.lines
            .iter()
            .all(|l| !l.text.is_empty() && l.width <= 150.0),
        "{f:?}"
    );
    let one = style(40.0, rules(Some(1), Overflow::Ellipsis));
    let line = &fit(&mut Text::new(fonts()), CJK, &one, [150.0, ROW]).lines[0].text;
    assert!(
        line.ends_with('…') && CJK.starts_with(line.trim_end_matches('…')),
        "{line}"
    );
}

/// The row with its middle key pressed and its bubble open, the bubble's label `label` in red
/// `size` px under the theme's `rules` for `key-popup-box`; the keys themselves draw in black,
/// small enough to fit.
fn bubble(label: &str, size: f32, overflow: Option<Overflow>) -> Scene {
    let rules = overflow.map_or_else(Default::default, |o| rules(Some(1), o));
    let small = Style {
        size: 8.0,
        ..Style::default()
    };
    let mut scene = row(label, &small, 150.0);
    for key in &mut scene.keys {
        key.styles = [small.clone(), small.clone()];
    }
    scene.desired_key = [150.0, ROW];
    mark(&mut scene, "characters", &vec!["character".into(); 3]);
    scene.headroom = headroom(&scene, false);
    scene.pressed = vec![1];
    scene.bubble = Some(Bubble {
        key: 1,
        style: Some(Style {
            background: 0xff445566,
            ..style(size, rules)
        }),
        indicator: None,
    });
    scene
}

#[test]
fn the_bubbles_label_too_big_stays_inside_the_bubble() {
    // A 60sp `key-popup-box` font, as the device check used, and a long label.
    for (label, size) in [
        ("w", 60.0 * 2.625),
        (LONG, 40.0),
        (HEBREW, 40.0),
        (CJK, 40.0),
    ] {
        for overflow in [None, Some(Overflow::Clip), Some(Overflow::Ellipsis)] {
            let scene = bubble(label, size, overflow);
            let r = scene.bubble_bounds(1).expect("bubble");
            let raster = draw(&scene);
            let (inn, out) = red_in_and_out(&raster, &scene, r);
            assert!(
                inn > 50 && out == 0,
                "{label} {overflow:?}: {inn} in, {out} out"
            );
            assert_eq!(raster.label_fits(), 1, "{label} {overflow:?}");
        }
    }
    // A bubble's label that fits draws as it always did.
    let scene = bubble("w", 22.0 * 2.625, None);
    let raster = draw(&scene);
    assert_eq!(raster.label_fits(), 0);
    let (r, ink) = (
        scene.bubble_bounds(1).expect("bubble"),
        red_bounds(&raster, &scene),
    );
    let ink = ink.expect("ink");
    assert!(
        ((ink.left - r.left) - (r.right - ink.right)).abs() <= 2.0,
        "{ink:?} in {r:?}"
    );
}
