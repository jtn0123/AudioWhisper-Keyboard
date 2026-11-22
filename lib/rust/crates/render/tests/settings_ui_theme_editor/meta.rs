//! The component meta editor: the label, the authors and the night flag, with the extension
//! editor's checks.
use super::common::*;
use render::settings_ui::{
    catalogue::{languages::Lang, theme_editor::Part},
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
    };
    assert_eq!(a.theme_editor.meta, meta);
    assert!(a.theme_editor.changed());
    assert_eq!(control(&a, 400_010).unwrap().label, "Moonlit");
    assert!(tap(&mut a, &mut text, SAVE_ID));
    let save = last(&a, effect::SAVE).unwrap();
    assert_eq!(save[4..], ["Moonlit", "Ada\nLinus", wire::flag::TRUE]);
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
