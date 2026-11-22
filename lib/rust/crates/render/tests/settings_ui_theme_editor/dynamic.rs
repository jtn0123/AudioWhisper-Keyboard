//! Material You dynamic colours drawn from the palette Kotlin's report carries: its names with a
//! swatch each in the light and dark lists, the swatch after the value row and in the rules list
//! beside a static colour's, and no dynamic swatch when the report has no palette.
use super::{common::*, props::*};
use render::settings_ui::theme_editor::dynamic::Palette;
use render::{
    settings_ui::{App, catalogue::theme_editor::Part, theme_editor::paint::rule_id},
    text::Text,
    touch::Motion,
};
use snygg::value::COLOR_NAMES;
use wire::settings_ui::theme_editor::report;

const LIGHT: &str = "snygg__property_value__material_you_light_color";
const DARK: &str = "snygg__property_value__material_you_dark_color";

/// A light and a dark colour for name `i`, none of them on the screen otherwise.
pub fn light(i: usize) -> u32 {
    0xff10_2030 + i as u32 * 0x0001_0101
}
fn dark(i: usize) -> u32 {
    0xff80_4000 + i as u32 * 0x0001_0203
}
fn colors(f: fn(usize) -> u32) -> String {
    let all: Vec<String> = (0..COLOR_NAMES.len()).map(|i| f(i).to_string()).collect();
    all.join("/")
}
/// The test report with the palette in it.
pub fn with_palette(sheet: &str) -> Vec<String> {
    let mut p = loaded(sheet);
    p[report::COLOR_NAMES] = COLOR_NAMES.join("/");
    p[report::LIGHT_COLORS] = colors(light);
    p[report::DARK_COLORS] = colors(dark);
    p
}
/// The editor on the test stylesheet with the palette, or without one, adding a foreground to
/// `key[code=10]` in `encoder`.
fn adding(palette: bool, encoder: &str) -> (App, Text) {
    let mut a = opened();
    let p = if palette {
        with_palette(SHEET)
    } else {
        loaded(SHEET)
    };
    a.set_theme_editor(&p).unwrap();
    named(&mut a);
    let mut text = text();
    a.scene(&mut text);
    add_to(&mut a, &mut text, 3);
    choose(&mut a, &mut text, "Foreground", encoder);
    (a, text)
}
/// The panel's 24 dp box filled with `argb`, if it draws one.
fn swatch(a: &mut App, text: &mut Text, argb: u32) -> Option<keys::Rect> {
    swatch_in(a, text, argb, |_| true)
}
/// The panel's 24 dp box filled with `argb` where `within` holds, if it draws one.
fn swatch_in(
    a: &mut App,
    text: &mut Text,
    argb: u32,
    within: impl Fn(keys::Rect) -> bool,
) -> Option<keys::Rect> {
    let size = 24.0 * a.config[2];
    let panel = a.scene(text).panel.unwrap();
    let found = panel.items.iter().find(|i| {
        let b = i.bounds;
        i.styles[0].background == argb && b.width() == size && b.height() == size && within(b)
    });
    found.map(|i| i.bounds)
}
fn touch(a: &mut App, action: i32, (x, y): (f32, f32), t: i64) {
    a.motion(&Motion {
        action,
        index: 0,
        ids: vec![0],
        times: vec![0, t],
        coords: vec![x, y],
    });
}

#[test]
fn the_report_carries_the_palette_or_none() {
    let read = Palette::read(&with_palette(SHEET));
    assert_eq!(read.names, COLOR_NAMES);
    assert_eq!((read.light[2], read.dark[2]), (light(2), dark(2)));
    assert_eq!(
        read.color("onSurface", true),
        Some(dark(
            read.names.iter().position(|n| n == "onSurface").unwrap()
        ))
    );
    assert_eq!(read.color("nothing", false), None);
    assert_eq!(Palette::read(&loaded(SHEET)), Palette::default());
    let mut short = with_palette(SHEET);
    short[report::DARK_COLORS] = colors(dark).rsplit_once('/').unwrap().0.into();
    assert_eq!(Palette::read(&short), Palette::default(), "a colour short");
    let mut bad = with_palette(SHEET);
    bad[report::LIGHT_COLORS] = bad[report::LIGHT_COLORS].replacen('/', "/x", 1);
    assert_eq!(
        Palette::read(&bad),
        Palette::default(),
        "a colour that does not read"
    );
}

#[test]
fn each_name_in_the_lists_has_its_swatch_from_the_palette() {
    for (encoder, shade) in [(LIGHT, light as fn(usize) -> u32), (DARK, dark)] {
        let (mut a, mut text) = adding(true, encoder);
        let value = part_id(&a, Part::PropValue);
        assert!(tap(&mut a, &mut text, value));
        let d = a.config[2];
        let labels = choices(&a);
        assert_eq!(labels.len(), COLOR_NAMES.len());
        for j in [0, 7, COLOR_NAMES.len() - 1] {
            let row = control(&a, 3000 + j as i32).unwrap().bounds;
            let inside = |b: keys::Rect| b.top >= row.top && b.bottom <= row.bottom;
            let b = swatch_in(&mut a, &mut text, shade(j), inside)
                .unwrap_or_else(|| panic!("{encoder} {j}"));
            assert_eq!(b.right, row.right - 16.0 * d, "{encoder} {j}");
            assert_eq!(b.top - row.top, ((row.height() - b.height()) / 2.0).round());
            // The name keeps clear of it: the list keeps 12 dp before the swatch.
            let panel = a.scene(&mut text).panel.unwrap();
            let mut names = panel.items.iter().filter(|i| i.label == labels[j]);
            let name = names.find(|i| inside(i.bounds)).unwrap();
            assert!(
                name.bounds.right <= b.left - 12.0 * d + 0.5,
                "{encoder} {j}"
            );
        }
    }
}

#[test]
fn the_value_row_and_the_rules_list_show_the_colour_the_palette_gives() {
    let (mut a, mut text) = adding(true, DARK);
    pick(&mut a, &mut text, Part::PropValue, "onSurface");
    let i = COLOR_NAMES.iter().position(|n| *n == "onSurface").unwrap();
    let d = a.config[2];
    let row = control(&a, part_id(&a, Part::PropValue)).unwrap().bounds;
    let b = swatch(&mut a, &mut text, dark(i)).expect("the value row's swatch");
    assert_eq!(b.right, a.config[0] - 16.0 * d);
    assert!(
        (b.left - row.right - 12.0 * d).abs() <= 0.5,
        "12 dp after the dropdown"
    );
    assert!(
        swatch(&mut a, &mut text, light(i)).is_none(),
        "the dark colour, not the light"
    );
    apply(&mut a, &mut text);
    // key[code=10]: its static background, then the dynamic foreground, each with its swatch in
    // the slot before Delete, which the row's tap takes.
    let props = a.rule_properties(3);
    for (n, argb) in [(0, 0xff00_00ff), (1, dark(i))] {
        assert!(props[n].raw == ["background", "foreground"][n]);
        let b = swatch(&mut a, &mut text, argb).unwrap_or_else(|| panic!("property {n}"));
        let delete = control(&a, rule_id(3, 65 + 2 * n)).unwrap().bounds;
        let gap = delete.left - b.right;
        assert!(
            (gap - 12.0 * d).abs() <= 0.5,
            "property {n}: centred in its slot"
        );
        assert_eq!(
            control(&a, rule_id(3, 64 + 2 * n)).unwrap().bounds.right,
            delete.left
        );
    }
    let b = swatch(&mut a, &mut text, dark(i)).unwrap();
    let centre = (b.left + b.width() / 2.0, b.top + b.height() / 2.0);
    touch(&mut a, 0, centre, 1);
    touch(&mut a, 1, centre, 2);
    a.scene(&mut text);
    assert_eq!(
        a.theme_editor.prop.as_ref().map(|f| f.name.as_str()),
        Some("foreground")
    );
}

#[test]
fn without_a_palette_a_dynamic_colour_has_no_swatch() {
    let (mut a, mut text) = adding(false, LIGHT);
    let value = part_id(&a, Part::PropValue);
    assert!(tap(&mut a, &mut text, value));
    let row = control(&a, 3000).unwrap().bounds;
    let panel = a.scene(&mut text).panel.unwrap();
    let size = 24.0 * a.config[2];
    let boxed = panel.items.iter().any(|i| {
        i.bounds.width() == size && i.bounds.top >= row.top && i.bounds.bottom <= row.bottom
    });
    assert!(!boxed, "no swatch in the list");
    a.back();
    pick(&mut a, &mut text, Part::PropValue, "primary");
    let value = control(&a, part_id(&a, Part::PropValue)).unwrap().bounds;
    let d = a.config[2];
    assert_eq!(
        value.right,
        a.config[0] - 16.0 * d,
        "the dropdown keeps the row"
    );
    apply(&mut a, &mut text);
    // The foreground's value runs on where the static background's stops for its swatch.
    let panel = a.scene(&mut text).panel.unwrap();
    let summary = |label: &str| {
        panel
            .items
            .iter()
            .find(|i| i.label == label)
            .unwrap()
            .bounds
    };
    let (background, foreground) = (
        summary("#0000ffff"),
        summary("dynamic-light-color(primary)"),
    );
    assert_eq!(
        foreground.right - background.right,
        48.0 * d,
        "no icon slot"
    );
}
