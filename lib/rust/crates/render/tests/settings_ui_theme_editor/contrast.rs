//! The property editor's low contrast note: WCAG 2.x's ratio, the pair it checks (a rule's
//! foreground and background, one being edited), what lies under a translucent background,
//! the note appearing and clearing as the value changes before Apply, and Apply still taking it.
use super::{common::*, dynamic::*, props::*};
use render::settings_ui::{
    App,
    catalogue::theme_editor::Part,
    theme_editor::{
        PROPERTY_ADD,
        contrast::{CONTRAST_ID, LOW_CONTRAST, MINIMUM, contrast, ratio, shown},
    },
};
use render::text::Text;
use snygg::text as utf16;

const NOTE: &str = "Low contrast: {ratio}:1, below 4.5:1";
const LIGHT_KEY: &str = "snygg__property_value__material_you_light_color";
/// The keyboard's black under a key's translucent black; a foreground drawn on its own rule's
/// background only; variables one level and two; and the keyboard's own colours.
const PAIRS: &str = r##"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet",
    "@defines":{"--ink":"#0000ff","--deep":"var(--ink)"},
    "keyboard":{"background":"#000000"},
    "key":{"background":"#00000080","foreground":"#ffffff"},
    "key:pressed":{"background":"#0000ff","foreground":"var(--ink)"},
    "key:focus":{"foreground":"#000000"},
    "smartbar":{"background":"#00000080","foreground":"#ffffff","border-color":"#000000"}}"##;

/// The rule written `name`'s index in the list.
fn rule(a: &App, name: &str) -> usize {
    let rules = &a.theme_editor.rules;
    rules
        .iter()
        .position(|(r, _)| r.format() == utf16(name))
        .unwrap()
}
/// The editor on `report` with the test labels and the note's text.
fn on(report: &[String]) -> (App, Text) {
    let mut a = opened();
    a.set_theme_editor(report).unwrap();
    named(&mut a);
    a.strings.insert(LOW_CONTRAST.into(), NOTE.into());
    let mut text = text();
    a.scene(&mut text);
    (a, text)
}
/// The property editor on `name` of rule `r`.
fn editing(a: &mut App, text: &mut Text, r: &str, name: &str) {
    let r = rule(a, r);
    let n = a.rule_properties(r).iter().position(|q| q.raw == name);
    edit(a, text, r, n.unwrap());
}
/// The note the screen shows under the value row, if any.
fn note(a: &mut App, text: &mut Text) -> Option<String> {
    a.scene(text);
    let c = control(a, CONTRAST_ID)?;
    let value = control(a, part_id(a, Part::PropValue)).unwrap().bounds;
    assert!(c.bounds.top >= value.bottom, "under the value row");
    let panel = a.scene(text).panel.unwrap();
    let drawn = panel
        .items
        .iter()
        .find(|i| i.label == c.label)
        .expect("drawn");
    assert_eq!(
        drawn.text.foreground, a.colors[2],
        "a note's ink, not an error's"
    );
    Some(c.label)
}
fn says(r: &str) -> Option<String> {
    Some(NOTE.replace("{ratio}", r))
}

#[test]
fn the_ratio_is_wcag_2s_and_shows_rounded_down() {
    let (black, white) = (0xff00_0000, 0xffff_ffff);
    assert!((ratio(black, white) - 21.0).abs() < 1e-9);
    assert!((ratio(white, black) - 21.0).abs() < 1e-9);
    for c in [black, white, 0xff77_7777, 0xff12_3456] {
        assert!((ratio(c, c) - 1.0).abs() < 1e-12);
    }
    let grey = ratio(0xff77_7777, white);
    assert!((grey - 4.478).abs() < 0.001, "{grey}");
    assert!(grey < MINIMUM && MINIMUM == 4.5);
    assert_eq!(
        [shown(grey), shown(2.13), shown(1.0), shown(21.0)],
        ["4.4", "2.1", "1.0", "21.0"]
    );
    // A translucent foreground is laid over its background; a translucent background over what
    // lies beneath, or over black and over white with the worse counting. Half (0x80) black
    // over white is #7f7f7f.
    let (half, grey) = (|c: u32| c & 0x00ff_ffff | 0x8000_0000, 0xff7f_7f7f);
    let veiled = contrast(half(black), white, None);
    assert!((veiled - ratio(grey, white)).abs() < 1e-9);
    assert!((contrast(white, half(black), Some(black)) - 21.0).abs() < 1e-9);
    let unknown = contrast(white, half(black), None);
    assert!((unknown - ratio(white, grey)).abs() < 1e-9, "{unknown}");
}

#[test]
fn the_note_comes_and_goes_with_the_value_and_apply_still_takes_it() {
    let (mut a, mut text) = on(&loaded(SHEET));
    // key[code=10]'s background is #0000ff.
    let r = rule(&a, "key[code=10]");
    add_to(&mut a, &mut text, r);
    choose(&mut a, &mut text, "Foreground", "Solid color");
    pick_color(&mut a, &mut text, "#000080");
    assert_eq!(note(&mut a, &mut text), says("1.8"));
    pick_color(&mut a, &mut text, "#ffffff");
    assert_eq!(note(&mut a, &mut text), None, "8.6:1 clears it");
    pick_color(&mut a, &mut text, "#000080");
    assert_eq!(note(&mut a, &mut text), says("1.8"));
    // Neither the note nor the ratio holds Apply back.
    apply(&mut a, &mut text);
    assert_ne!(a.stack.last().unwrap(), PROPERTY_ADD);
    assert!(writes(&a, "foreground", "rgba(0,0,128,1)"));
}

#[test]
fn a_background_is_checked_against_its_rules_foreground_over_the_keyboards() {
    let (mut a, mut text) = on(&loaded(PAIRS));
    // key: white on half black, over the keyboard's black, is 21:1.
    editing(&mut a, &mut text, "key", "foreground");
    assert_eq!(note(&mut a, &mut text), None);
    a.back();
    editing(&mut a, &mut text, "key", "background");
    assert_eq!(note(&mut a, &mut text), None);
    pick_color(&mut a, &mut text, "#ffffffc0");
    assert!(note(&mut a, &mut text).is_some(), "the white let through");
    // With no keyboard rule nothing is known beneath smartbar's half black: over black it is
    // 21:1, over white grey (#7f7f7f) and 4.0:1, which counts.
    let sheet = PAIRS.replace(r##""keyboard":{"background":"#000000"},"##, "");
    let (mut a, mut text) = on(&loaded(&sheet));
    editing(&mut a, &mut text, "smartbar", "foreground");
    assert_eq!(note(&mut a, &mut text), says("4.0"));
    // Nor beneath the keyboard's own background: made half black, it does not lie on the black
    // it was.
    let keyboard = PAIRS.replace(
        r##""keyboard":{"background":"#000000"}"##,
        r##""keyboard":{"background":"#000000","foreground":"#ffffff"}"##,
    );
    let (mut a, mut text) = on(&loaded(&keyboard));
    editing(&mut a, &mut text, "keyboard", "background");
    assert_eq!(note(&mut a, &mut text), None);
    pick_color(&mut a, &mut text, "#00000080");
    assert_eq!(note(&mut a, &mut text), says("4.0"));
}

#[test]
fn only_a_rules_own_pair_is_checked_and_variables_one_level() {
    let (mut a, mut text) = on(&loaded(PAIRS));
    // key:pressed's --ink on its own #0000ff.
    editing(&mut a, &mut text, "key:pressed", "foreground");
    assert_eq!(note(&mut a, &mut text), says("1.0"));
    // --deep names --ink: the keyboard does not follow it, and neither does the check.
    pick(&mut a, &mut text, Part::PropValue, "--deep");
    assert_eq!(note(&mut a, &mut text), None);
    let (mut a, mut text) = on(&loaded(PAIRS));
    // key:focus has no background of its own, and a border is not text.
    editing(&mut a, &mut text, "key:focus", "foreground");
    assert_eq!(note(&mut a, &mut text), None);
    a.back();
    editing(&mut a, &mut text, "smartbar", "border-color");
    assert_eq!(note(&mut a, &mut text), None);
}

#[test]
fn a_dynamic_colour_is_checked_as_the_palette_has_it() {
    // primary's light colour on a background a step from it; without the palette, unknown.
    let bg = format!("#{:06x}", light(1) & 0x00ff_ffff);
    let sheet = PAIRS.replace(
        r##""key:focus":{"foreground":"#000000"}"##,
        &format!(r##""key:focus":{{"foreground":"#000000","background":"{bg}"}}"##),
    );
    for (report, shows) in [(with_palette(&sheet), says("1.0")), (loaded(&sheet), None)] {
        let (mut a, mut text) = on(&report);
        editing(&mut a, &mut text, "key:focus", "foreground");
        pick(&mut a, &mut text, Part::PropEncoder, LIGHT_KEY);
        pick(&mut a, &mut text, Part::PropValue, "primary");
        assert_eq!(note(&mut a, &mut text), shows);
    }
}
