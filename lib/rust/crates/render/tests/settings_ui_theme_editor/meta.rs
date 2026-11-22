//! The component meta editor: the label, the authors and the night flag, with the extension
//! editor's checks.
use super::common::*;
use render::settings_ui::{
    catalogue::{Kind, languages::Lang, theme_editor::Part},
    theme_editor::{META, Meta, meta::NIGHT_ID, paint::SAVE_ID},
};
use wire::settings_ui::theme_editor::effect;

#[test]
fn edit_shows_the_meta_and_apply_changes_it_for_save_to_write() {
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, 400_013));
    assert_eq!(a.stack.last().unwrap(), META);
    assert_eq!(
        a.line_text(part_id(&a, Part::MetaField(0))).unwrap(),
        "Sunny"
    );
    assert_eq!(
        a.line_text(part_id(&a, Part::MetaField(1))).unwrap(),
        "Ada, Grace"
    );
    assert_eq!(control(&a, NIGHT_ID).unwrap().checked, Some(false));
    type_into(&mut a, &mut text, Part::MetaField(0), " Moonlit ");
    type_into(&mut a, &mut text, Part::MetaField(1), "Ada, , Linus");
    assert!(tap(&mut a, &mut text, NIGHT_ID));
    assert_eq!(control(&a, NIGHT_ID).unwrap().checked, Some(true));
    let apply = lang_id(&a, Lang::Save);
    assert!(tap(&mut a, &mut text, apply));
    assert!(a.stack.last().unwrap().starts_with("settings/theme/edit/"));
    let meta = Meta {
        label: "Moonlit".into(),
        authors: vec!["Ada".into(), "Linus".into()],
        night: true,
        ..a.theme_editor.base.clone().unwrap().1
    };
    assert_eq!(a.theme_editor.meta, meta);
    assert!(a.theme_editor.changed());
    assert_eq!(control(&a, 400_010).unwrap().label, "Moonlit");
    assert!(tap(&mut a, &mut text, SAVE_ID));
    let save = last(&a, effect::SAVE).unwrap();
    assert_eq!(save[4..7], ["Moonlit", "Ada\nLinus", wire::flag::TRUE]);
}

#[test]
fn edit_offers_the_id_the_path_and_material_you_and_save_writes_them() {
    let (mut a, mut text) = app(SHEET);
    let body = control(&a, 400_012).unwrap().label;
    assert!(
        body.ends_with("stylesheetPath = stylesheets/sunny.json"),
        "{body}"
    );
    assert!(tap(&mut a, &mut text, 400_013));
    // Compose's order: the id first, the path and Material You after the night switch.
    let parts: Vec<_> = a.rows().iter().map(|r| r.kind).collect();
    let at = |p: Part| {
        let row = parts
            .iter()
            .position(|k| matches!(k, Kind::ThemeEdit(q) if *q == p));
        row.unwrap()
    };
    assert!(at(Part::MetaField(2)) < at(Part::MetaField(0)));
    assert!(
        at(Part::Night) < at(Part::MetaField(3))
            && at(Part::MetaField(3)) < at(Part::MetaChoice(0))
    );
    assert_eq!(
        a.line_text(part_id(&a, Part::MetaField(2))).unwrap(),
        "sunny"
    );
    assert_eq!(a.line_text(part_id(&a, Part::MetaField(3))).unwrap(), "");
    // Each Material You flag is a dropdown of its enum's names with the component's chosen.
    let palette = part_id(&a, Part::MetaChoice(0));
    assert_eq!(control(&a, palette).unwrap().summary, "Neutral");
    assert!(tap(&mut a, &mut text, palette));
    assert_eq!(choices(&a), ["TonalSpot", "Neutral", "Vibrant"]);
    assert!(tap(&mut a, &mut text, 3002));
    let spec = part_id(&a, Part::MetaChoice(2));
    assert!(tap(&mut a, &mut text, spec));
    assert!(tap(&mut a, &mut text, 3001));
    type_into(&mut a, &mut text, Part::MetaField(2), "dusk");
    let apply = lang_id(&a, Lang::Save);
    assert!(tap(&mut a, &mut text, apply));
    // The box names the new id, and the stylesheet follows it while no path is set.
    assert_eq!(control(&a, 400_011).unwrap().label, "local.my.themes:dusk");
    let body = control(&a, 400_012).unwrap().label;
    assert!(
        body.ends_with("stylesheetPath = stylesheets/dusk.json"),
        "{body}"
    );
    assert!(tap(&mut a, &mut text, 400_013));
    type_into(&mut a, &mut text, Part::MetaField(3), " sheets/dusk.json ");
    assert!(tap(&mut a, &mut text, apply));
    assert!(tap(&mut a, &mut text, SAVE_ID));
    let save = last(&a, effect::SAVE).unwrap();
    assert_eq!(
        save[7..],
        [
            "dusk",
            "sheets/dusk.json",
            "Vibrant",
            "Default",
            "SPEC_2025"
        ]
    );
}

#[test]
fn a_bad_or_taken_id_or_a_bad_path_refuses_apply_and_says_why() {
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, 400_013));
    let apply = lang_id(&a, Lang::Save);
    let (id, path) = (
        part_id(&a, Part::MetaField(2)),
        part_id(&a, Part::MetaField(3)),
    );
    a.strings.insert(
        "ext__validation__error_component_id".into(),
        "An ID matching {component_id_regex}".into(),
    );
    for (typed, why) in [
        ("moon", "settings__theme_editor__component_id_exists"),
        ("Moon", "An ID matching ^[a-z][a-z0-9_]*$"),
        ("", "ext__validation__enter_component_id"),
    ] {
        type_into(&mut a, &mut text, Part::MetaField(2), typed);
        assert!(tap(&mut a, &mut text, apply));
        assert_eq!(a.stack.last().unwrap(), META);
        assert_eq!(control(&a, id).unwrap().summary, why, "{typed:?}");
    }
    type_into(&mut a, &mut text, Part::MetaField(2), "sunny");
    type_into(&mut a, &mut text, Part::MetaField(3), "a:b.json");
    assert!(tap(&mut a, &mut text, apply));
    assert_eq!(control(&a, id).unwrap().summary, "");
    assert_eq!(
        control(&a, path).unwrap().summary,
        "ext__validation__error_stylesheet_path"
    );
    type_into(&mut a, &mut text, Part::MetaField(3), "");
    assert!(tap(&mut a, &mut text, apply));
    assert!(a.stack.last().unwrap().starts_with("settings/theme/edit/"));
    assert!(!a.theme_editor.changed(), "the meta as it was");
}

#[test]
fn a_blank_label_or_no_author_refuses_apply_and_says_why_under_the_field() {
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, 400_013));
    a.strings.insert(
        "ext__validation__enter_component_label".into(),
        "Please enter a component label".into(),
    );
    type_into(&mut a, &mut text, Part::MetaField(0), "  ");
    type_into(&mut a, &mut text, Part::MetaField(1), " , ");
    let label = part_id(&a, Part::MetaField(0));
    assert_eq!(
        control(&a, label).unwrap().summary,
        "",
        "no message before Apply"
    );
    let apply = lang_id(&a, Lang::Save);
    assert!(tap(&mut a, &mut text, apply));
    assert_eq!(a.stack.last().unwrap(), META);
    assert_eq!(
        control(&a, label).unwrap().summary,
        "Please enter a component label"
    );
    let authors = part_id(&a, Part::MetaField(1));
    assert_eq!(
        control(&a, authors).unwrap().summary,
        "ext__validation__error_author"
    );
    assert!(!a.theme_editor.changed());
    // Cancel leaves the meta as it was.
    let cancel = lang_id(&a, Lang::Cancel);
    assert!(tap(&mut a, &mut text, cancel));
    assert_eq!(a.theme_editor.meta.label, "Sunny");
}
