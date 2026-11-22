//! A static colour through the settings' colour picker: its alpha typed or dragged, OK setting
//! the form's value and the preview, and what Apply writes; the value row and the rules list in
//! each of fine-tune's colour representations, formatted as `formatColor(color, withAlpha =
//! true)`; and the picker's theme layout beside the accent rows' (`settings_ui_other_theme.rs`).
use super::{common::*, props::*, values::added};
use render::{
    settings_ui::{App, catalogue::theme_editor::Part, color, color_alpha::ALPHA_ID},
    text::Text,
    touch::Motion,
};
use wire::settings_ui::theme_editor::effect;

const REPRESENTATION: &str = "theme__editor_color_representation";

fn picked(typed: &'static str) -> impl FnOnce(&mut App, &mut Text) {
    move |a, text| pick_color(a, text, typed)
}
fn touch(a: &mut App, action: i32, x: f32, y: f32, t: i64) {
    a.motion(&Motion {
        action,
        index: 0,
        ids: vec![0],
        times: vec![0, t],
        coords: vec![x, y],
    });
}
/// The value row's summary: the colour as the row shows it.
fn value_row(a: &App) -> String {
    control(a, part_id(a, Part::PropValue)).unwrap().summary
}
/// The value row tapped: the picker.
fn open_picker(a: &mut App, text: &mut Text) {
    let row = part_id(a, Part::PropValue);
    assert!(tap(a, text, row));
    assert!(a.color_picker().is_some());
}
/// The property editor adding a solid foreground to `key[code=10]`.
fn adding() -> (App, Text) {
    let (mut a, mut text) = named_app();
    add_to(&mut a, &mut text, 3);
    choose(&mut a, &mut text, "Foreground", "Solid color");
    (a, text)
}

#[test]
fn a_colour_typed_with_its_alpha_in_any_form_is_written_as_compose_wrote_it() {
    let some = |s: &str| Some(s.to_owned());
    let hex = added("Foreground", "Solid color", picked("#ff000080"));
    assert_eq!(hex, some("rgba(255,0,0,0.5019608)"));
    let rgb = added("Foreground", "Solid color", picked("rgba(1, 2, 3, 0.25)"));
    assert_eq!(rgb, some("rgba(1,2,3,0.2509804)"));
    let hsv = added(
        "Foreground",
        "Solid color",
        picked("hsva(120, 100, 100, .5)"),
    );
    assert_eq!(hsv, some("rgba(0,255,0,0.5019608)"));
    let opaque = added("Foreground", "Solid color", picked("rgb(1, 2, 3)"));
    assert_eq!(opaque, some("rgba(1,2,3,1)"));
    // What the theme's picker refuses, its error showing until the text changes.
    let (mut a, mut text) = adding();
    for bad in [
        "#ff00008",
        "rgba(1, 2, 3, 1.5)",
        "rgba(1, 2, 3, 0.5, 1)",
        "red",
        "",
    ] {
        assert!(!picker_typed(&mut a, &mut text, bad), "{bad}");
        assert!(tap(&mut a, &mut text, 6), "Cancel");
        assert!(a.modal.is_none());
    }
    assert_eq!(value_row(&a), "#000000ff", "Cancel keeps the colour");
}

#[test]
fn ok_sets_the_value_and_previews_it_and_apply_writes_it() {
    let (mut a, mut text) = adding();
    assert_eq!(value_row(&a), "#000000ff");
    open_picker(&mut a, &mut text);
    let p = a.color_picker().unwrap().clone();
    assert_eq!((p.value, p.alpha, p.custom), (0xff000000, Some(1.0), true));
    assert_eq!(control(&a, 7000).unwrap().summary, "#000000ff");
    assert!(tap(&mut a, &mut text, 6));
    a.effects.clear();
    assert!(picker_typed(&mut a, &mut text, "#0c223880"));
    assert_eq!(control(&a, 7000).unwrap().summary, "#0c223880");
    assert_eq!(last(&a, effect::PREVIEW), None, "nothing before OK");
    assert!(tap(&mut a, &mut text, 7));
    assert_eq!(value_row(&a), "#0c223880");
    let shown = last(&a, effect::PREVIEW).unwrap();
    assert!(shown[3].contains("\"foreground\": \"rgba(12,34,56,0.5019608)\""));
    // The picker opens again on the colour OK set, its alpha with it.
    open_picker(&mut a, &mut text);
    let p = a.color_picker().unwrap();
    assert_eq!((p.value, p.alpha), (0x800c2238, Some(128.0 / 255.0)));
    assert!(tap(&mut a, &mut text, 6));
    apply(&mut a, &mut text);
    assert!(writes(&a, "foreground", "rgba(12,34,56,0.5019608)"));
}

#[test]
fn the_alpha_bar_follows_the_finger_outside_the_editor() {
    let (mut a, mut text) = adding();
    open_picker(&mut a, &mut text);
    let bar = control(&a, ALPHA_ID).unwrap().bounds;
    let hue = control(&a, 7131).unwrap().bounds;
    assert!(bar.top > hue.top, "the alpha bar is under the hue bar");
    assert_eq!((bar.left, bar.width()), (hue.left, hue.width()));
    let middle = bar.top + bar.height() / 2.0;
    touch(&mut a, 0, bar.left + bar.width() / 2.0, middle, 1);
    touch(&mut a, 1, bar.left + bar.width() / 2.0, middle, 2);
    let p = a.color_picker().unwrap();
    assert_eq!((p.alpha, p.value), (Some(0.5), 0x80000000));
    let panel = a.scene(&mut text).panel.unwrap();
    assert_eq!(control(&a, 7000).unwrap().summary, "#00000080");
    // Its thumb shows the colour over white, as Jetpref's does.
    let fill = color::over(0x80000000, 0xffffffff);
    assert!(panel.items.iter().any(|i| i.styles[0].background == fill));
    // Past the left end: clear; the editor greys the bar out and takes its drag away.
    touch(&mut a, 0, bar.left + 4.0, middle, 3);
    touch(&mut a, 2, bar.left - 90.0, middle, 4);
    touch(&mut a, 1, bar.left - 90.0, middle, 5);
    assert_eq!(a.color_picker().unwrap().value, 0x00000000);
    assert!(tap(&mut a, &mut text, 7140));
    assert!(control(&a, ALPHA_ID).is_none());
    assert!(tap(&mut a, &mut text, 7142));
    assert!(tap(&mut a, &mut text, 7));
    assert_eq!(value_row(&a), "#00000000");
}

#[test]
fn the_theme_picker_is_the_custom_page_with_a_wide_plane_and_no_page_switch() {
    let (mut a, mut text) = adding();
    open_picker(&mut a, &mut text);
    let plane = control(&a, 7130).unwrap().bounds;
    assert_eq!(plane.height(), (plane.width() / 1.5).round());
    assert!(control(&a, 5).is_none(), "no Presets");
    assert!(control(&a, 7100).is_none());
    assert_eq!(control(&a, 7).unwrap().label, "action__ok");
    assert_eq!(control(&a, 6).unwrap().label, "action__cancel");
    a.focus = -1;
    a.focus_next(false);
    let mut order = vec![a.focus];
    for _ in 0..3 {
        a.focus_next(false);
        order.push(a.focus);
    }
    assert_eq!(order, [7000, 7140, 6, 7]);
}

#[test]
fn the_value_row_and_the_rules_list_show_each_representation() {
    for (rep, row, accent) in [
        ("HEX", "#0c223880", "#ff0000ff"),
        ("RGB", "rgba(12, 34, 56, 0.5)", "rgba(255, 0, 0, 1)"),
        ("HSV", "hsva(210, 79, 22, 0.5)", "hsva(0, 100, 100, 1)"),
    ] {
        let (mut a, mut text) = adding();
        a.prefs.insert(REPRESENTATION.into(), rep.into());
        pick_color(&mut a, &mut text, "#0c223880");
        assert_eq!(value_row(&a), row, "{rep}");
        open_picker(&mut a, &mut text);
        assert_eq!(control(&a, 7000).unwrap().summary, row, "{rep}");
        assert!(tap(&mut a, &mut text, 6));
        apply(&mut a, &mut text);
        let defines = a.rule_properties(0);
        assert_eq!(defines[0].value, accent, "{rep}");
        let added = a.rule_properties(3);
        let foreground = added.iter().find(|p| p.raw == "foreground").unwrap();
        assert_eq!(foreground.value, row, "{rep}");
    }
}

#[test]
fn a_picked_colour_leaves_no_picker_on_the_next_value() {
    let (mut a, mut text) = adding();
    pick_color(&mut a, &mut text, "#0c223880");
    pick(&mut a, &mut text, Part::PropEncoder, "Variable");
    let listed = listed(&mut a, &mut text, Part::PropValue);
    assert_eq!(listed, [PLACEHOLDER, "--accent"]);
}

#[test]
fn the_value_row_shows_its_swatch_after_the_colour() {
    let (mut a, mut text) = adding();
    for (typed, argb, checkered) in [
        ("#0c223880", 0x800c2238, true),
        ("#0c2238", 0xff0c2238, false),
    ] {
        pick_color(&mut a, &mut text, typed);
        let panel = a.scene(&mut text).panel.unwrap();
        let row = control(&a, part_id(&a, Part::PropValue)).unwrap().bounds;
        let swatch = panel.items.iter().find(|i| i.styles[0].background == argb);
        let b = swatch.unwrap_or_else(|| panic!("no {typed} swatch")).bounds;
        let d = a.config[2];
        assert_eq!((b.width(), b.height()), (24.0 * d, 24.0 * d));
        assert_eq!(b.right, a.config[0] - 16.0 * d);
        let gap = b.left - row.right;
        assert!(
            (gap - 12.0 * d).abs() <= 0.5,
            "12 dp after the dropdown: {gap}"
        );
        assert_eq!(b.top - row.top, ((row.height() - b.height()) / 2.0).round());
        let backing = panel
            .items
            .iter()
            .any(|i| i.media.is_some() && i.bounds == b);
        assert_eq!(
            backing, checkered,
            "{typed}: checkered only under a translucent colour"
        );
    }
}
