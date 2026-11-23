//! The fixture font draws: in the pixels of the key labels and the Smartbar, in each shaped run's
//! face, and in every element that inherits the window's `font-family`.
use super::common::*;
use render::{
    text::{Font, Text},
    theme_fonts,
};

#[test]
fn a_theme_font_draws_the_key_labels_and_the_smartbar() {
    let (plain, plain_styles) = draw(&files(&[]));
    let (themed, themed_styles) = draw(&files(&[REGULAR, BOLD]));
    assert!(plain_styles.iter().all(|f| *f == Font::Sans));
    assert!(themed_styles.iter().all(|f| matches!(f, Font::Theme(_))));
    let key = differing(&plain, &themed, |_, y| y >= STRIP as usize);
    let strip = differing(&plain, &themed, |_, y| y < STRIP as usize);
    eprintln!("theme font pixel diff: key labels {key}, smartbar {strip}");
    assert!(key > 1000, "the key label kept the default font: {key}");
    assert!(strip > 800, "the candidate kept the default font: {strip}");
    // Drawing the theme again gives the same pixels: the registry and the caches agree.
    assert_eq!(draw(&files(&[REGULAR, BOLD])).0, themed);
}

#[test]
fn each_run_names_the_theme_face_and_missing_glyphs_fall_through() {
    let mut theme = theme(&files(&[REGULAR, BOLD]));
    let font = theme.style(&window("key")).font;
    let mut text = Text::new([roboto()]);
    let regular = faces(&mut text, "Hello 7?", font, 400, false);
    eprintln!("chosen faces at 400: {regular:?}");
    assert_eq!(
        regular,
        [
            ("Hello ".into(), "Fixture Script Regular".into()),
            ("7?".into(), "Roboto Light".into())
        ]
    );
    let bold = faces(&mut text, "Hello", font, 700, false);
    assert_eq!(bold, [("Hello".into(), "Fixture Script Bold".into())]);
    // No italic face: the nearest upright one, as fontique matches `@font-face` styles.
    let italic = faces(&mut text, "Hi", font, 400, true);
    assert_eq!(italic, [("Hi".into(), "Fixture Script Regular".into())]);
    // The default fonts keep their faces in the same context once the theme font is registered.
    let sans = faces(&mut text, "Hello", Font::Sans, 700, false);
    assert_eq!(sans, [("Hello".into(), "Roboto Light".into())]);
}

#[test]
fn registering_a_theme_font_leaves_the_default_chain_byte_identical() {
    let mut theme = theme(&files(&[REGULAR]));
    let font = theme.style(&window("key")).font;
    let mut fresh = Text::new([roboto()]);
    let mut used = Text::new([roboto()]);
    used.measure("Hello", 40.0, font, 400);
    for (sample, family) in [("Hello 7?", Font::Sans), ("mono 0", Font::Mono)] {
        assert_eq!(
            glyphs(&mut fresh, sample, family),
            glyphs(&mut used, sample, family)
        );
    }
}

#[test]
fn the_panels_and_smartbar_elements_inherit_the_theme_font() {
    let mut theme = theme(&files(&[REGULAR]));
    let font = theme.style(&window("window")).font;
    assert!(matches!(font, Font::Theme(_)));
    for element in [
        "smartbar-candidate-word-text",
        "smartbar-action-key-text",
        "clipboard-header-text",
        "clipboard-item",
        "media-emoji-subheader",
        "subtype-panel-list-item-text",
        "key-popup-element",
    ] {
        assert_eq!(theme.style(&window(element)).font, font, "{element}");
    }
    // An element naming a family the theme has no `@font` for keeps the default chain.
    assert_eq!(
        theme.style(&window("clipboard-header-button")).font,
        Font::Sans
    );
}

#[test]
fn the_same_files_are_one_font() {
    let a = theme(&files(&[REGULAR, BOLD])).style(&window("key")).font;
    let b = theme(&files(&[REGULAR, BOLD])).style(&window("key")).font;
    assert_eq!(a, b);
    let regular_only = theme(&files(&[REGULAR])).style(&window("key")).font;
    assert_ne!(a, regular_only);
    let fonts = theme(&files(&[REGULAR, BOLD])).fonts().to_vec();
    assert_eq!(fonts.len(), 3, "three `@font` rules, three families");
    // The same weight and style from another file is another font.
    let other = vec![(REGULAR.0.to_string(), fixture(BOLD.1).display().to_string())];
    assert_ne!(theme(&other).style(&window("key")).font, regular_only);
    let id = match a {
        Font::Theme(id) => id,
        other => panic!("not a theme font: {other:?}"),
    };
    assert!((id as usize) < theme_fonts::MAX_FAMILIES);
}

#[test]
fn a_set_s_font_style_and_weight_pick_its_face() {
    let mut theme = theme(&files(&[REGULAR, BOLD]));
    let slanted = theme.style(&window("smartbar-action-tile-text")).font;
    let key = theme.style(&window("key")).font;
    assert_ne!(slanted, key);
    let mut text = Text::new([roboto()]);
    // Registered first in the same context, the key's font must stay a family of its own.
    let bold = faces(&mut text, "Hi", key, 700, false);
    assert_eq!(bold, [("Hi".into(), "Fixture Script Bold".into())]);
    let italic = faces(&mut text, "Hi", slanted, 400, true);
    assert_eq!(italic, [("Hi".into(), "Fixture Script Bold".into())]);
    let upright = faces(&mut text, "Hi", slanted, 700, false);
    assert_eq!(upright, [("Hi".into(), "Fixture Script Regular".into())]);
}

#[test]
fn fonts_loaded_after_a_style_was_resolved_replace_it() {
    let mut theme = theme(&[]);
    assert_eq!(theme.style(&window("key")).font, Font::Sans);
    theme.load_fonts(&files(&[REGULAR]));
    assert!(matches!(theme.style(&window("key")).font, Font::Theme(_)));
    theme.load_fonts(&[]);
    assert_eq!(theme.style(&window("key")).font, Font::Sans);
}

#[test]
fn a_set_s_declared_weight_overrides_the_file_s_own() {
    let mut theme = theme(&files(&[REGULAR, BOLD]));
    let swapped = theme.style(&window("one-handed-panel-button")).font;
    let mut text = Text::new([roboto()]);
    let normal = faces(&mut text, "Hi", swapped, 400, false);
    assert_eq!(normal, [("Hi".into(), "Fixture Script Bold".into())]);
    let bold = faces(&mut text, "Hi", swapped, 700, false);
    assert_eq!(bold, [("Hi".into(), "Fixture Script Regular".into())]);
}

#[test]
fn rules_of_one_name_are_one_family() {
    use snygg::{text, value::Value};
    let set = |uri: &str| vec![(text("src"), Value::Uri(text(uri)))];
    let rules = vec![
        (text("A"), vec![set(REGULAR.0)]),
        (text("A"), vec![set(BOLD.0)]),
    ];
    let loaded = theme_fonts::load(&rules, &files(&[REGULAR, BOLD]));
    assert_eq!(loaded.len(), 1);
}
