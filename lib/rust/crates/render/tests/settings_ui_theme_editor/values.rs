//! Every value kind through the property editor: chosen or typed, applied, and written by the
//! stylesheet as Compose's dialog would have built it; the checks each field makes; and the
//! value helpers the screen uses (whole percents, the file selector's URIs).
use super::{common::*, props::*};
use render::settings_ui::{
    App,
    catalogue::theme_editor::Part,
    theme_editor::{
        PROPERTY_ADD,
        property_paint::{ALL_ID, HINT_ID, NO_FILES_ID},
        property_values::{flex_uri, texts_of, uri_encode, value_of},
    },
};
use render::text::Text;
use snygg::value::{Encoder, Unit, Value};

/// What the stylesheet writes for property `name` of `key[code=10]`.
fn written(a: &App, name: &str) -> Option<String> {
    let json = a.theme_editor.json().unwrap();
    let at = json.find("\"key[code=10]\": {")?;
    let rule = &json[at..at + json[at..].find('}')?];
    let from = rule.find(&format!("\"{name}\": \""))? + name.len() + 5;
    Some(rule[from..from + rule[from..].find('"')?].to_owned())
}
/// The property labelled `label` added to `key[code=10]` with `encoder`, its value set by
/// `set`, then applied: what the stylesheet writes, None if Apply refused.
fn added(label: &str, encoder: &str, set: impl FnOnce(&mut App, &mut Text)) -> Option<String> {
    let (mut a, mut text) = named_app();
    add_to(&mut a, &mut text, 3);
    choose(&mut a, &mut text, label, encoder);
    let name = a.theme_editor.prop.as_ref().unwrap().name.clone();
    set(&mut a, &mut text);
    apply(&mut a, &mut text);
    if a.stack.last().unwrap() == PROPERTY_ADD {
        return None;
    }
    written(&a, &name)
}
fn prop(name: &str) -> String {
    format!("snygg__property_name__{name}")
}
fn kind(name: &str) -> String {
    format!("snygg__property_value__{name}")
}
fn nothing(_: &mut App, _: &mut Text) {}
fn value_pick(label: &'static str) -> impl FnOnce(&mut App, &mut Text) {
    move |a, text| pick(a, text, Part::PropValue, label)
}
fn typed(value: &'static str) -> impl FnOnce(&mut App, &mut Text) {
    move |a, text| type_into(a, text, Part::PropValue, value)
}
fn sides(values: [&'static str; 4]) -> impl FnOnce(&mut App, &mut Text) {
    move |a, text| {
        for (k, v) in values.into_iter().enumerate() {
            type_into(a, text, Part::PropSide(k), v);
        }
    }
}

#[test]
fn keywords_and_shapes_without_sizes_have_nothing_to_edit() {
    let some = |s: &str| Some(s.to_owned());
    let inherit = kind("explicit_inherit");
    assert_eq!(added("Foreground", &inherit, nothing), some("inherit"));
    assert_eq!(added(&prop("clip"), &kind("yes"), nothing), some("yes"));
    assert_eq!(added(&prop("clip"), &kind("no"), nothing), some("no"));
    let shape = prop("shape");
    let rect = added(&shape, &kind("rectangle_shape"), nothing);
    assert_eq!(rect, some("rectangle()"));
    let circle = added(&shape, &kind("circle_shape"), nothing);
    assert_eq!(circle, some("circle()"));
}

#[test]
fn a_variable_is_chosen_from_the_defines_names() {
    let (mut a, mut text) = named_app();
    add_to(&mut a, &mut text, 3);
    choose(&mut a, &mut text, "Foreground", "Variable");
    let listed = listed(&mut a, &mut text, Part::PropValue);
    assert_eq!(listed, [PLACEHOLDER, "--accent"]);
    apply(&mut a, &mut text);
    assert_eq!(a.stack.last().unwrap(), PROPERTY_ADD, "no variable chosen");
    pick(&mut a, &mut text, Part::PropValue, "--accent");
    apply(&mut a, &mut text);
    assert_eq!(written(&a, "foreground").unwrap(), "var(--accent)");
}

#[test]
fn material_you_colours_are_chosen_by_name() {
    let light = added("Foreground", &kind("material_you_light_color"), |a, t| {
        let names = listed(a, t, Part::PropValue);
        assert_eq!(names.len(), snygg::value::COLOR_NAMES.len());
        assert_eq!(names[0], "primary");
        pick(a, t, Part::PropValue, "onSurface");
    });
    assert_eq!(light.unwrap(), "dynamic-light-color(onSurface)");
    let dark = added(
        &prop("border_color"),
        &kind("material_you_dark_color"),
        nothing,
    );
    assert_eq!(dark.unwrap(), "dynamic-dark-color(primary)");
}

#[test]
fn keyword_kinds_are_chosen_from_their_keywords() {
    let cases = [
        ("font_family", "font_family_generic", "monospace"),
        ("font_style", "font_style", "italic"),
        ("text_align", "text_align", "center"),
        ("text_decoration_line", "text_decoration_line", "underline"),
        ("text_overflow", "text_overflow", "ellipsis"),
        ("content_scale", "content_scale", "crop"),
    ];
    for (name, encoder, keyword) in cases {
        let got = added(&prop(name), &kind(encoder), value_pick(keyword));
        assert_eq!(got.as_deref(), Some(keyword), "{name}");
    }
    // The nine weights, once each; Compose listed the 18 keys, each weight twice.
    let weight = added("Font weight", &kind("font_weight"), |a, t| {
        let words = listed(a, t, Part::PropValue);
        assert_eq!(words.len(), 9);
        assert_eq!((words[0].as_str(), words[8].as_str()), ("thin", "black"));
        assert_eq!(
            control(a, part_id(a, Part::PropValue)).unwrap().summary,
            "normal"
        );
        pick(a, t, Part::PropValue, "semi-bold");
    });
    assert_eq!(weight.unwrap(), "semi-bold");
}

#[test]
fn a_custom_font_is_one_of_the_stylesheets_fonts() {
    let font = added(&prop("font_family"), &kind("font_family_custom"), |a, t| {
        assert_eq!(listed(a, t, Part::PropValue), [PLACEHOLDER, "Sans"]);
        pick(a, t, Part::PropValue, "Sans");
    });
    assert_eq!(font.unwrap(), "`Sans`");
    let none = added(&prop("font_family"), &kind("font_family_custom"), nothing);
    assert_eq!(none, None, "the placeholder is no font");
}

#[test]
fn a_uri_is_one_of_the_workspaces_files_or_a_note_says_there_are_none() {
    let uri = added(&prop("background_image"), &kind("uri"), |a, t| {
        assert!(control(a, NO_FILES_ID).is_none());
        assert_eq!(listed(a, t, Part::PropValue), ["flex:/fonts/a.ttf"]);
        pick(a, t, Part::PropValue, "flex:/fonts/a.ttf");
    });
    assert_eq!(uri.unwrap(), "uri(`flex:/fonts/a.ttf`)");
    let (mut a, mut text) = named_app();
    a.theme_editor.component.fonts.clear();
    a.theme_editor.component.images = vec!["sky blue.png".into()];
    add_to(&mut a, &mut text, 3);
    choose(&mut a, &mut text, &prop("background_image"), &kind("uri"));
    assert_eq!(
        listed(&mut a, &mut text, Part::PropValue),
        ["flex:/images/sky%20blue.png"]
    );
    a.theme_editor.component.images.clear();
    a.scene(&mut text);
    let note = control(&a, NO_FILES_ID).unwrap().label;
    assert!(note.starts_with("settings__theme_editor__file_selector_no_files_text"));
    assert!(!control(&a, part_id(&a, Part::PropValue)).unwrap().enabled);
}

#[test]
fn sizes_and_line_counts_are_typed_beside_their_unit() {
    let some = |s: &str| Some(s.to_owned());
    assert_eq!(
        added("Border width", "Size (dp)", typed("2.5")),
        some("2.5dp")
    );
    assert_eq!(
        added("Text size", &kind("sp_size"), typed(" 14.5")),
        some("14.5sp")
    );
    assert_eq!(
        added("Text size", &kind("sp_size"), typed("0.5")),
        None,
        "sp from 1"
    );
    assert_eq!(added("Border width", "Size (dp)", typed("-2")), None);
    let lines = prop("text_max_lines");
    assert_eq!(
        added(&lines, &kind("text_max_lines"), typed("3")),
        some("3")
    );
    assert_eq!(
        added(&lines, &kind("text_max_lines"), nothing),
        some("none")
    );
    assert_eq!(added(&lines, &kind("text_max_lines"), typed("0")), None);
}

#[test]
fn a_solid_colour_is_typed_until_the_picker_comes() {
    let some = |s: &str| Some(s.to_owned());
    let hex = added("Foreground", "Solid color", typed("#ff000080"));
    assert_eq!(hex, some("rgba(255,0,0,0.5019608)"));
    let rgb = added("Foreground", "Solid color", typed("rgb(1, 2, 3)"));
    assert_eq!(rgb, some("rgba(1,2,3,1)"));
    let (mut a, mut text) = named_app();
    add_to(&mut a, &mut text, 3);
    choose(&mut a, &mut text, "Foreground", "Solid color");
    type_into(&mut a, &mut text, Part::PropValue, "red");
    assert_eq!(field(&a, Part::PropValue).1, "ext__validation__error_color");
    type_into(&mut a, &mut text, Part::PropValue, "");
    assert_eq!(field(&a, Part::PropValue).1, "ext__validation__enter_color");
}

#[test]
fn padding_takes_four_sides_and_the_chip_copies_the_last_to_all() {
    let padding = added(
        &prop("padding"),
        &kind("padding"),
        sides(["1", "2.5", "3", "4"]),
    );
    assert_eq!(padding.unwrap(), "1dp 2.5dp 3dp 4dp");
    let (mut a, mut text) = named_app();
    add_to(&mut a, &mut text, 3);
    choose(&mut a, &mut text, &prop("margin"), &kind("padding"));
    let titles: Vec<_> = (0..4)
        .map(|k| control(&a, part_id(&a, Part::PropSide(k))).unwrap().label)
        .collect();
    assert_eq!(titles[0], "settings__theme_editor__padding_start");
    assert_eq!(titles[3], "settings__theme_editor__padding_bottom");
    assert_eq!(
        control(&a, ALL_ID).unwrap().label,
        "settings__theme_editor__padding_apply_for_all"
    );
    type_into(&mut a, &mut text, Part::PropSide(2), "6");
    assert!(tap(&mut a, &mut text, ALL_ID));
    assert_eq!(field(&a, Part::PropSide(0)).0, "6");
    apply(&mut a, &mut text);
    assert_eq!(written(&a, "margin").unwrap(), "6dp 6dp 6dp 6dp");
}

#[test]
fn corners_in_dp_or_whole_percents_are_checked_as_compose_checked_them() {
    let shape = prop("shape");
    let rounded = added(
        &shape,
        &kind("rounded_corner_shape_dp"),
        sides(["1", "2", "3", "4.5"]),
    );
    assert_eq!(rounded.unwrap(), "rounded-corner(1dp,2dp,3dp,4.5dp)");
    let cut = added(
        &shape,
        &kind("cut_corner_shape_percent"),
        sides(["10", "20", "30", "40"]),
    );
    assert_eq!(cut.unwrap(), "cut-corner(10%,20%,30%,40%)");
    // A cut corner the stylesheet would read back as a whole number is refused.
    let fraction = added(
        &shape,
        &kind("cut_corner_shape_dp"),
        sides(["2.5", "0", "0", "0"]),
    );
    assert_eq!(fraction, None);
    let (mut a, mut text) = named_app();
    add_to(&mut a, &mut text, 3);
    choose(
        &mut a,
        &mut text,
        &shape,
        &kind("rounded_corner_shape_percent"),
    );
    let corner = control(&a, part_id(&a, Part::PropSide(1))).unwrap();
    assert_eq!(corner.label, "enum__shape_corner__top_end");
    let chip = control(&a, ALL_ID).unwrap().label;
    assert_eq!(
        chip,
        "settings__theme_editor__property_value_shape_apply_for_all_corners"
    );
    type_into(&mut a, &mut text, Part::PropSide(1), "60");
    assert_eq!(field(&a, Part::PropSide(1)).1, "", "above half is a hint");
    let hint = control(&a, HINT_ID + 1).unwrap().label;
    assert_eq!(hint, "ext__validation__hint_value_above_50_percent");
    type_into(&mut a, &mut text, Part::PropSide(1), "101");
    assert_eq!(
        field(&a, Part::PropSide(1)).1,
        "ext__validation__enter_number_between_0_100"
    );
    type_into(&mut a, &mut text, Part::PropSide(1), "2.5");
    assert_eq!(
        field(&a, Part::PropSide(1)).1,
        "ext__validation__enter_valid_number"
    );
    type_into(&mut a, &mut text, Part::PropSide(1), "");
    assert_eq!(
        field(&a, Part::PropSide(1)).1,
        "ext__validation__enter_percent_size"
    );
    choose(&mut a, &mut text, &shape, &kind("cut_corner_shape_dp"));
    type_into(&mut a, &mut text, Part::PropSide(0), "");
    assert_eq!(
        field(&a, Part::PropSide(0)).1,
        "ext__validation__enter_dp_size"
    );
    type_into(&mut a, &mut text, Part::PropSide(0), "2.5");
    assert_eq!(
        field(&a, Part::PropSide(0)).1,
        "ext__validation__enter_valid_number"
    );
}

#[test]
fn an_edited_value_shows_in_its_fields_as_written() {
    let sheet = SHEET.replace(
        r##""key[code=10]":{"background":"#0000ff"}"##,
        r##""key[code=10]":{"background":"#0000ff","padding":"1dp 2dp 3.5dp 4dp","shape":"cut-corner(5%,6%,7%,8%)","font-size":"14sp"}"##,
    );
    let (mut a, mut text) = app(&sheet);
    named(&mut a);
    edit(&mut a, &mut text, 3, 1);
    let shown: Vec<_> = (0..4).map(|k| field(&a, Part::PropSide(k)).0).collect();
    assert_eq!(shown, ["1", "2", "3.5", "4"]);
    a.back();
    edit(&mut a, &mut text, 3, 2);
    assert_eq!(field(&a, Part::PropSide(3)).0, "8");
    a.back();
    edit(&mut a, &mut text, 3, 3);
    assert_eq!(field(&a, Part::PropValue).0, "14");
    edit_value(&mut a, &mut text);
}

fn edit_value(a: &mut App, text: &mut Text) {
    type_into(a, text, Part::PropValue, "16");
    apply(a, text);
    assert_eq!(written(a, "font-size").unwrap(), "16sp");
}

#[test]
fn a_percentage_is_shown_and_taken_as_a_whole_percent() {
    // No property of the v2 spec takes a percentage, so the screen never offers one.
    let half = Value::Dimension(Unit::Percentage, 0.5);
    assert_eq!(texts_of(&half).1[0], "50");
    let texts = [
        "12.5".to_owned(),
        String::new(),
        String::new(),
        String::new(),
    ];
    let v = value_of(Encoder::Percentage, "", &texts).unwrap();
    assert_eq!(v, Value::Dimension(Unit::Percentage, 0.125));
    assert_eq!(
        Encoder::Percentage.serialize(&v).unwrap(),
        snygg::text("12.5%")
    );
}

#[test]
fn file_uris_are_encoded_as_android_encoded_them() {
    assert_eq!(
        flex_uri("fonts", "Roboto-Light.ttf"),
        "flex:/fonts/Roboto-Light.ttf"
    );
    assert_eq!(uri_encode("a b+c/d", "/"), "a%20b%2Bc/d");
    assert_eq!(uri_encode("x_y-z!.~'()*", ""), "x_y-z!.~'()*");
    assert_eq!(uri_encode("é#%?&=", ""), "%C3%A9%23%25%3F%26%3D");
    assert_eq!(flex_uri("images", "a/b c.png"), "flex:/images/a/b%20c.png");
}
