//! Other and Theme (stage4-other-theme.md): the accent colour dialog driven as a finger and the
//! keyboard drive it (a preset, the plane and hue bar, a typed colour, the system accent), and
//! what OK stores. SettingsOtherThemeMatchTest holds every row against the Compose sources.
use render::{
    settings_ui::{App, color},
    text::Text,
    touch::Motion,
};

const ACCENT: u32 = 0xff6750a4;

fn app(route: &str) -> (App, Text) {
    let mut a = App {
        config: [1080.0, 2400.0, 2.625, 1.0],
        ..App::default()
    };
    a.facts.insert("system_accent".into(), ACCENT.to_string());
    a.navigate(route);
    let mut text = Text::new(vec![
        include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ]);
    a.scene(&mut text);
    (a, text)
}
fn id(a: &App, key: &str) -> i32 {
    1000 + a.rows().iter().position(|r| r.key == key).unwrap() as i32
}
fn bounds(a: &App, id: i32) -> render::settings_ui::Control {
    a.controls.iter().find(|c| c.id == id).cloned().unwrap()
}
fn label(a: &App, id: i32) -> Option<String> {
    a.controls
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.label.clone())
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
/// Paints, then taps control `id` as it now stands.
fn act(a: &mut App, text: &mut Text, id: i32) -> bool {
    a.scene(text);
    a.activate(id)
}
fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

#[test]
fn a_preset_and_the_system_accent_store_as_compose_stores_them() {
    let (mut a, mut text) = app("settings/other");
    let row = id(&a, "other__accent_color");
    assert!(act(&mut a, &mut text, row));
    a.scene(&mut text);
    // No stored colour opens on the system accent: Presets, with Default ticked.
    let p = a.color_picker().unwrap();
    assert_eq!((p.value, p.preset, p.custom), (ACCENT, None, false));
    assert!(bounds(&a, 7120).selected);
    assert_eq!(
        label(&a, 5).as_deref(),
        Some("settings__color_picker__custom")
    );
    assert_eq!(label(&a, 7105).as_deref(), Some("#673ab7"));
    assert!(act(&mut a, &mut text, 7105));
    a.scene(&mut text);
    assert!(bounds(&a, 7105).selected && !bounds(&a, 7120).selected);
    assert!(act(&mut a, &mut text, 7));
    assert!(a.modal.is_none());
    assert_eq!(
        a.effects,
        [["write", "other__accent_color", "ff673ab700000000"]]
    );
    a.prefs
        .insert("other__accent_color".into(), "ff673ab700000000".into());
    a.effects.clear();
    // Opening again ticks the stored preset; Default then OK stores no colour at all.
    assert!(act(&mut a, &mut text, row));
    a.scene(&mut text);
    assert_eq!(a.color_picker().unwrap().preset, Some(5));
    assert!(act(&mut a, &mut text, 7120));
    assert!(act(&mut a, &mut text, 7));
    assert_eq!(
        a.effects,
        [["write", "other__accent_color", color::UNSPECIFIED]]
    );
    // Cancel stores nothing.
    a.effects.clear();
    assert!(act(&mut a, &mut text, row));
    assert!(act(&mut a, &mut text, 7110));
    assert!(act(&mut a, &mut text, 6));
    assert!(a.modal.is_none() && a.effects.is_empty());
}

#[test]
fn custom_follows_the_plane_the_hue_bar_and_a_typed_colour() {
    let (mut a, mut text) = app("settings/other");
    a.prefs
        .insert("other__accent_color".into(), "ff2196f300000000".into());
    let row = id(&a, "other__accent_color");
    assert!(act(&mut a, &mut text, row));
    assert!(act(&mut a, &mut text, 5));
    a.scene(&mut text);
    assert_eq!(
        label(&a, 5).as_deref(),
        Some("settings__color_picker__presets")
    );
    assert_eq!(bounds(&a, 7000).summary, "#2196f3");
    // The plane's top right corner is the hue at full saturation and value.
    let plane = bounds(&a, 7130).bounds;
    touch(
        &mut a,
        0,
        plane.left + 1.0,
        plane.top + plane.height() / 2.0,
        1,
    );
    touch(&mut a, 2, plane.right + 50.0, plane.top - 50.0, 2);
    touch(&mut a, 1, plane.right + 50.0, plane.top - 50.0, 3);
    let p = a.color_picker().unwrap();
    assert_eq!((p.hsv[1], p.hsv[2]), (1.0, 1.0));
    let hue = p.hsv[0];
    // The middle of the hue bar is cyan.
    a.scene(&mut text);
    let bar = bounds(&a, 7131).bounds;
    touch(&mut a, 0, bar.left + bar.width() / 2.0, bar.top + 4.0, 4);
    touch(&mut a, 1, bar.left + bar.width() / 2.0, bar.top + 4.0, 5);
    let p = a.color_picker().unwrap();
    assert!((p.hsv[0] - 180.0).abs() < 0.5 && p.hsv[0] != hue);
    assert_eq!(p.value, 0xff00ffff);
    // Edit, type an rgb() colour, Apply: the field shows it as typed and OK stores it.
    a.scene(&mut text);
    assert!(act(&mut a, &mut text, 7140));
    a.scene(&mut text);
    assert_eq!(bounds(&a, 7000).role, "edit");
    assert!(act(&mut a, &mut text, 7000));
    assert_eq!(a.field_id(), 7000);
    assert_eq!(a.editor.text, utf16("#00ffff"));
    a.editor.anchor = 0;
    assert!(a.editor.replace(&utf16("rgb(1, 2, 3"), 1, false));
    a.sync_field();
    assert!(act(&mut a, &mut text, 7141));
    assert!(a.color_picker().unwrap().error, "no closing bracket");
    assert!(a.editor.replace(&utf16(")"), 1, false));
    a.sync_field();
    assert!(act(&mut a, &mut text, 7141));
    let p = a.color_picker().unwrap();
    assert_eq!((p.value, p.editing, p.error), (0xff010203, false, false));
    assert_eq!(a.field_id(), 2, "the preview field has the editor back");
    a.scene(&mut text);
    assert_eq!(bounds(&a, 7000).summary, "rgb(1, 2, 3)");
    a.effects.clear();
    assert!(act(&mut a, &mut text, 7));
    assert_eq!(
        a.effects,
        [["write", "other__accent_color", "ff01020300000000"]]
    );
}

#[test]
fn the_editor_greys_the_plane_and_discard_keeps_the_colour() {
    let (mut a, mut text) = app("settings/other");
    let row = id(&a, "other__accent_color");
    assert!(act(&mut a, &mut text, row));
    assert!(act(&mut a, &mut text, 5));
    a.scene(&mut text);
    let plane = bounds(&a, 7130).bounds;
    assert!(act(&mut a, &mut text, 7140));
    a.scene(&mut text);
    assert!(!a.controls.iter().any(|c| c.id == 7130 || c.id == 7131));
    let before = a.color_picker().unwrap().clone();
    touch(&mut a, 0, plane.left + 2.0, plane.bottom - 2.0, 1);
    touch(&mut a, 1, plane.left + 2.0, plane.bottom - 2.0, 2);
    assert_eq!(
        a.color_picker().unwrap().hsv,
        before.hsv,
        "no drags in the editor"
    );
    assert!(act(&mut a, &mut text, 7000));
    a.editor.anchor = 0;
    assert!(a.editor.replace(&utf16("#000000"), 1, false));
    a.sync_field();
    assert!(act(&mut a, &mut text, 7142));
    let p = a.color_picker().unwrap();
    assert_eq!((p.value, p.editing), (before.value, false));
    // Presets and back: the field starts over on the colour in hex.
    assert!(act(&mut a, &mut text, 5));
    assert!(act(&mut a, &mut text, 5));
    a.scene(&mut text);
    assert_eq!(
        bounds(&a, 7000).summary,
        color::format(color::Format::Hex, ACCENT)
    );
    // Out of the editor the drag surfaces take touches again, though they are no buttons.
    assert_eq!(bounds(&a, 7130).role, "none");
}
