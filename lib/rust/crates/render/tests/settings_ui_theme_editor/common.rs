//! What the theme editor tests share: a stylesheet and Kotlin's report of its component, an app
//! on the editor with the report in, taps by control id, rows by part, and typing into a field.
use render::{
    settings_ui::{
        App, Control,
        catalogue::{
            Kind,
            languages::Lang,
            theme_editor::{CodeRow, Part},
        },
        theme_editor::{EDIT, codes::CODE_ADD, codes_paint::CODE_ADD_ID},
    },
    text::Text,
};
use wire::settings_ui::theme_editor::{effect, outcome, report};

pub const WORKSPACE: &str = "3f1c-workspace";
/// A small stylesheet: `@defines`, a key and its pressed and enter variants, and a font.
pub const SHEET: &str = r##"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet",
    "@defines":{"--accent":"#ff0000"},
    "key":{"background":"var(--accent)","foreground":"#000000"},
    "key:pressed":{"background":"#00ff00"},
    "key[code=10]":{"background":"#0000ff"},
    "@font `Sans`":[{"src":"uri(`flex:/fonts/a.ttf`)","font-weight":"bold"}]}"##;

/// The editor's route for component `index` of the test workspace.
pub fn route(index: usize) -> String {
    format!("{EDIT}/{WORKSPACE}/{index}")
}
/// Kotlin's report of component 0 with `sheet` as its stylesheet.
pub fn loaded(sheet: &str) -> Vec<String> {
    let mut p = vec![String::new(); report::SIZE];
    p[report::OUTCOME] = outcome::LOADED.into();
    p[report::WORKSPACE] = WORKSPACE.into();
    p[report::COMPONENT] = "0".into();
    p[report::STYLESHEET] = sheet.into();
    p[report::EXTENSION] = "local.my.themes".into();
    p[report::ID] = "sunny".into();
    p[report::LABEL] = "Sunny".into();
    p[report::AUTHORS] = "Ada\nGrace".into();
    p[report::NIGHT] = wire::flag::FALSE.into();
    p[report::PATH] = String::new();
    p[report::FONTS] = "a.ttf".into();
    p[report::IMAGES] = String::new();
    p[report::PALETTE] = "Neutral".into();
    p[report::CONTRAST] = "Default".into();
    p[report::SPEC] = "SPEC_2021".into();
    p[report::PALETTES] = "TonalSpot/Neutral/Vibrant".into();
    p[report::CONTRASTS] = "Default/Medium/High".into();
    p[report::SPECS] = "SPEC_2021/SPEC_2025".into();
    p[report::OTHER_IDS] = "moon".into();
    p
}
pub fn text() -> Text {
    Text::new(vec![
        include_bytes!("../../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ])
}
/// An app on the editor, asked for its component and not yet answered.
pub fn opened() -> App {
    let mut a = App {
        config: [1080.0, 6000.0, 2.625, 1.0],
        ..App::default()
    };
    for (k, v) in [
        ("snygg__rule_element__key", "Key"),
        ("snygg__rule_selector__pressed", "Pressed"),
        ("snygg__property_name__background", "Background"),
        ("snygg__property_name__foreground", "Foreground"),
        ("snygg__property_name__src", "Source"),
        ("snygg__property_name__font_weight", "Font weight"),
        ("enum__keyboard_mode__numeric", "Numeric"),
        ("action__delete_confirm_title", "Confirm delete"),
        (
            "settings__theme_editor__delete_rule_message",
            "Delete \"{name}\"?",
        ),
    ] {
        a.strings.insert(k.into(), v.into());
    }
    a.navigate(&route(0));
    assert!(
        a.effects
            .contains(&vec![effect::LOAD.into(), WORKSPACE.into(), "0".into()]),
        "the editor asked for no component"
    );
    a.effects.clear();
    a
}
/// An app on the editor with `sheet` loaded, painted, its effects cleared.
pub fn app(sheet: &str) -> (App, Text) {
    let mut a = opened();
    a.set_theme_editor(&loaded(sheet)).unwrap();
    let mut text = text();
    a.scene(&mut text);
    a.effects.clear();
    (a, text)
}
pub fn control(a: &App, id: i32) -> Option<Control> {
    a.controls.iter().find(|c| c.id == id).cloned()
}
/// Paints, taps `id` as it now stands, and paints again.
pub fn tap(a: &mut App, text: &mut Text, id: i32) -> bool {
    a.scene(text);
    let tapped = a.activate(id);
    a.scene(text);
    tapped
}
pub fn row_id(a: &App, pick: impl Fn(&Kind) -> bool) -> i32 {
    1000 + a.rows().iter().position(|r| pick(&r.kind)).unwrap() as i32
}
pub fn part_id(a: &App, part: Part) -> i32 {
    row_id(a, |k| matches!(k, Kind::ThemeEdit(q) if *q == part))
}
pub fn lang_id(a: &App, part: Lang) -> i32 {
    row_id(a, |k| matches!(k, Kind::Lang(q) if *q == part))
}
/// Focuses line field `part`, types `value` over what it held, and paints.
pub fn type_into(a: &mut App, text: &mut Text, part: Part, value: &str) {
    let id = part_id(a, part);
    assert!(tap(a, text, id), "{part:?}");
    a.editor.text = value.encode_utf16().collect();
    a.editor.anchor = a.editor.text.len();
    a.editor.focus = a.editor.text.len();
    a.sync_field();
    a.scene(text);
}
/// From the rule editor, the code editor's Add with `field` typed: back on the rule editor.
pub fn add_codes(a: &mut App, text: &mut Text, field: &str) {
    assert!(tap(a, text, CODE_ADD_ID));
    assert_eq!(a.stack.last().unwrap(), CODE_ADD);
    type_into(a, text, Part::Code(CodeRow::Field), field);
    let add = lang_id(a, Lang::Save);
    assert!(tap(a, text, add));
    assert_ne!(a.stack.last().unwrap(), CODE_ADD, "{:?}", a.code_message());
}
/// The rules' titles in the order the list shows them.
pub fn titles(a: &App) -> Vec<String> {
    let rules = &a.theme_editor.rules;
    rules.iter().map(|(r, _)| a.rule_lines(r).0).collect()
}
/// The last packet of effect `kind`.
pub fn last(a: &App, kind: &str) -> Option<Vec<String>> {
    a.effects.iter().rev().find(|e| e[0] == kind).cloned()
}
/// The modal list's entries' labels, which start at control 3000.
pub fn choices(a: &App) -> Vec<String> {
    (0..)
        .map_while(|j| control(a, 3000 + j))
        .map(|c| c.label)
        .collect()
}
