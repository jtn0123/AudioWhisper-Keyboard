//! The property editor: Add property on each rule's title line, a property tapped to edit it,
//! the name and encoder dropdowns, a variable's name field, Apply adding at the end or replacing
//! in place, the refusals, Delete with Undo, Cancel, and the search line.
use super::{common::*, props::*};
use render::settings_ui::{
    catalogue::{self, languages::Lang, theme_editor::Part},
    items::UNDO_ID,
    theme_editor::{
        PROPERTY_ADD, PROPERTY_EDIT,
        paint::{DELETE_ID, rule_id},
        property::EXISTS,
        property_paint::PROP_ERROR_ID,
    },
};

#[test]
fn add_property_is_on_every_title_line_and_adds_at_the_end_of_the_set() {
    let (mut a, mut text) = named_app();
    // `@defines` has Add only; an element rule has Duplicate, Edit and Add.
    for r in [0, 2, 3, 4] {
        let add = control(&a, rule_id(r, 4)).unwrap();
        assert_eq!(
            add.label, "settings__theme_editor__add_property",
            "rule {r}"
        );
    }
    assert!(control(&a, rule_id(0, 1)).is_none());
    let add = control(&a, rule_id(2, 4)).unwrap();
    assert_eq!(add.label, "settings__theme_editor__add_property");
    assert_eq!(add.summary, "Key");
    let edit = control(&a, rule_id(2, 2)).unwrap();
    assert!(
        add.bounds.left > edit.bounds.left,
        "Add is last on the line"
    );
    add_to(&mut a, &mut text, 2);
    let screen = catalogue::screen(PROPERTY_ADD).unwrap();
    assert_eq!(screen.title, "settings__theme_editor__add_property");
    // The encoder waits for a name; the names are the spec's, "select" first.
    let encoder = control(&a, part_id(&a, Part::PropEncoder)).unwrap();
    assert!(!encoder.enabled);
    assert_eq!(encoder.summary, PLACEHOLDER);
    let names = listed(&mut a, &mut text, Part::PropName);
    assert_eq!(names.len(), 1 + snygg::spec::elements().len());
    assert_eq!(names[..3], [PLACEHOLDER, "Background", "Foreground"]);
    choose(&mut a, &mut text, "Border width", "Size (dp)");
    assert_eq!(field(&a, Part::PropValue).0, "0");
    type_into(&mut a, &mut text, Part::PropValue, "2.5");
    apply(&mut a, &mut text);
    assert!(a.stack.last().unwrap().starts_with("settings/theme/edit/"));
    assert!(a.theme_editor.prop.is_none());
    assert!(writes(&a, "border-width", "2.5dp"));
    let props = a.rule_properties(2);
    assert_eq!(props.last().unwrap().raw, "border-width");
    assert_eq!(props.len(), 3);
    assert!(a.theme_editor.changed());
}

#[test]
fn a_tapped_property_opens_with_its_name_fixed_and_apply_replaces_it_in_place() {
    let (mut a, mut text) = named_app();
    let row = control(&a, rule_id(2, 64)).unwrap();
    assert_eq!(row.role, "button");
    edit(&mut a, &mut text, 2, 0);
    let screen = catalogue::screen(PROPERTY_EDIT).unwrap();
    assert_eq!(screen.title, "settings__theme_editor__edit_property");
    let name = control(&a, part_id(&a, Part::PropName)).unwrap();
    assert_eq!((name.summary.as_str(), name.enabled), ("Background", false));
    let encoder = control(&a, part_id(&a, Part::PropEncoder)).unwrap();
    assert_eq!(encoder.summary, "Variable");
    // Background's encoders: inherit and a variable, then its own three.
    let encoders = listed(&mut a, &mut text, Part::PropEncoder);
    assert_eq!(encoders.len(), 6);
    assert_eq!(
        encoders[..3],
        [
            PLACEHOLDER,
            "snygg__property_value__explicit_inherit",
            "Variable"
        ]
    );
    assert_eq!(
        control(&a, part_id(&a, Part::PropValue)).unwrap().summary,
        "--accent"
    );
    pick(&mut a, &mut text, Part::PropEncoder, "Solid color");
    assert_eq!(field(&a, Part::PropValue).0, "rgba(0,0,0,1)");
    type_into(&mut a, &mut text, Part::PropValue, "#00ff00");
    apply(&mut a, &mut text);
    assert!(writes(&a, "background", "rgba(0,255,0,1)"));
    let props = a.rule_properties(2);
    assert_eq!(
        (props[0].raw.as_str(), props[1].raw.as_str()),
        ("background", "foreground")
    );
}

#[test]
fn apply_refuses_a_missing_name_encoder_or_value_and_a_name_the_set_has() {
    let (mut a, mut text) = named_app();
    let before = a.theme_editor.rules.clone();
    add_to(&mut a, &mut text, 2);
    assert!(control(&a, PROP_ERROR_ID).is_none());
    apply(&mut a, &mut text);
    assert_eq!(a.stack.last().unwrap(), PROPERTY_ADD);
    assert!(a.theme_editor.prop.as_ref().unwrap().checked);
    // A name and no encoder: refused, and a new name clears what was chosen after it.
    pick(&mut a, &mut text, Part::PropName, "Border width");
    apply(&mut a, &mut text);
    assert_eq!(a.stack.last().unwrap(), PROPERTY_ADD);
    pick(&mut a, &mut text, Part::PropEncoder, "Size (dp)");
    type_into(&mut a, &mut text, Part::PropValue, "-1");
    assert_eq!(
        field(&a, Part::PropValue).1,
        "ext__validation__enter_positive_number"
    );
    type_into(&mut a, &mut text, Part::PropValue, "x");
    assert_eq!(
        field(&a, Part::PropValue).1,
        "ext__validation__enter_valid_number"
    );
    apply(&mut a, &mut text);
    assert_eq!(a.stack.last().unwrap(), PROPERTY_ADD);
    pick(&mut a, &mut text, Part::PropName, "Text size");
    let f = a.theme_editor.prop.clone().unwrap();
    assert_eq!((f.encoder, f.texts[0].as_str()), (None, ""));
    // A name the set has: Compose overwrote it; here Apply says so.
    choose(&mut a, &mut text, "Background", "Solid color");
    apply(&mut a, &mut text);
    assert_eq!(a.stack.last().unwrap(), PROPERTY_ADD);
    assert_eq!(control(&a, PROP_ERROR_ID).unwrap().label, EXISTS);
    assert_eq!(a.theme_editor.rules, before);
    // The message stays until the next Apply, as the rule editor's does; this one adds.
    choose(&mut a, &mut text, "Border width", "Size (dp)");
    assert_eq!(control(&a, PROP_ERROR_ID).unwrap().label, EXISTS);
    apply(&mut a, &mut text);
    assert!(writes(&a, "border-width", "0dp"));
}

#[test]
fn delete_in_the_top_bar_goes_with_undo_and_a_fonts_src_has_none() {
    let (mut a, mut text) = named_app();
    edit(&mut a, &mut text, 2, 0);
    let delete = control(&a, DELETE_ID).unwrap();
    assert_eq!(
        (delete.label.as_str(), delete.summary.as_str()),
        ("action__delete", "Background")
    );
    assert!(tap(&mut a, &mut text, DELETE_ID));
    assert!(a.stack.last().unwrap().starts_with("settings/theme/edit/"));
    assert_eq!(a.rule_properties(2).len(), 1);
    assert!(tap(&mut a, &mut text, UNDO_ID));
    assert_eq!(a.rule_properties(2)[0].raw, "background");
    // The font's `src` is required: its editor has no Delete.
    edit(&mut a, &mut text, 1, 0);
    assert!(control(&a, DELETE_ID).is_none());
    edit_back(&mut a, &mut text);
    edit(&mut a, &mut text, 1, 1);
    assert!(control(&a, DELETE_ID).is_some());
}

fn edit_back(a: &mut render::settings_ui::App, text: &mut render::text::Text) {
    let cancel = lang_id(a, Lang::Cancel);
    assert!(tap(a, text, cancel));
    assert!(a.stack.last().unwrap().starts_with("settings/theme/edit/"));
}

#[test]
fn cancel_and_back_leave_the_rules_as_they_were() {
    let (mut a, mut text) = named_app();
    let before = a.theme_editor.rules.clone();
    edit(&mut a, &mut text, 2, 0);
    pick(&mut a, &mut text, Part::PropEncoder, "Solid color");
    edit_back(&mut a, &mut text);
    assert!(a.theme_editor.prop.is_none());
    assert_eq!(a.theme_editor.rules, before);
    add_to(&mut a, &mut text, 3);
    choose(&mut a, &mut text, "Border width", "Size (dp)");
    a.back();
    a.scene(&mut text);
    assert!(a.stack.last().unwrap().starts_with("settings/theme/edit/"));
    assert_eq!(a.theme_editor.rules, before);
    assert!(!a.theme_editor.changed());
}

#[test]
fn a_variable_is_named_in_a_field_checked_as_compose_checked_it() {
    let (mut a, mut text) = named_app();
    add_to(&mut a, &mut text, 0);
    assert!(control(&a, part_id(&a, Part::PropSearch)).is_none());
    let name = control(&a, part_id(&a, Part::PropName)).unwrap();
    assert_eq!(name.role, "edit");
    type_into(&mut a, &mut text, Part::PropName, "accent");
    assert!(!control(&a, part_id(&a, Part::PropEncoder)).unwrap().enabled);
    assert_eq!(field(&a, Part::PropName).1, "", "no message before Apply");
    apply(&mut a, &mut text);
    assert_eq!(
        field(&a, Part::PropName).1,
        "ext__validation__error_property"
    );
    a.strings.insert(
        "ext__validation__error_property".into(),
        "Must match {variable_name_regex}".into(),
    );
    a.scene(&mut text);
    assert_eq!(field(&a, Part::PropName).1, "Must match --[a-zA-Z0-9-]+");
    type_into(&mut a, &mut text, Part::PropName, " ");
    assert_eq!(
        field(&a, Part::PropName).1,
        "ext__validation__enter_property"
    );
    type_into(&mut a, &mut text, Part::PropName, "--Accent-2 ");
    assert_eq!(field(&a, Part::PropName).1, "");
    pick(&mut a, &mut text, Part::PropEncoder, "Solid color");
    type_into(&mut a, &mut text, Part::PropValue, "#102030");
    apply(&mut a, &mut text);
    assert!(writes(&a, "--Accent-2", "rgba(16,32,48,1)"));
    // A variable's name is fixed when editing, and shown, not typed.
    edit(&mut a, &mut text, 0, 1);
    let name = control(&a, part_id(&a, Part::PropName)).unwrap();
    assert_eq!((name.role, name.enabled), ("button", false));
}

#[test]
fn search_narrows_the_names_by_label_or_name_ignoring_case() {
    let (mut a, mut text) = named_app();
    add_to(&mut a, &mut text, 2);
    let search = control(&a, part_id(&a, Part::PropSearch)).unwrap();
    assert_eq!(search.role, "edit");
    type_into(&mut a, &mut text, Part::PropSearch, "BORDER-");
    let names = listed(&mut a, &mut text, Part::PropName);
    assert_eq!(
        names,
        [
            PLACEHOLDER,
            "snygg__property_name__border_color",
            "snygg__property_name__border_style",
            "Border width"
        ]
    );
    // By its label only: "Text size" is `font-size`.
    type_into(&mut a, &mut text, Part::PropSearch, " text SIZE ");
    assert_eq!(
        listed(&mut a, &mut text, Part::PropName),
        [PLACEHOLDER, "Text size"]
    );
    pick(&mut a, &mut text, Part::PropName, "Text size");
    assert_eq!(a.theme_editor.prop.as_ref().unwrap().name, "font-size");
    type_into(&mut a, &mut text, Part::PropSearch, "nothing like it");
    assert_eq!(listed(&mut a, &mut text, Part::PropName), [PLACEHOLDER]);
    // No search line when editing.
    edit_back(&mut a, &mut text);
    edit(&mut a, &mut text, 2, 1);
    assert!(control(&a, part_id(&a, Part::PropSearch)).is_none());
}

#[test]
fn the_rules_focus_order_takes_add_and_each_property() {
    let (mut a, mut text) = named_app();
    let order = focus_order(&mut a);
    let at = |id: i32| order.iter().position(|o| *o == id);
    for (r, n) in [(2, 0), (2, 1), (0, 0)] {
        assert!(at(rule_id(r, 64 + 2 * n)).is_some(), "{r} {n}");
    }
    assert!(at(rule_id(2, 2)) < at(rule_id(2, 4)));
    assert!(at(rule_id(2, 4)) < at(rule_id(2, 64)));
    assert!(at(rule_id(0, 4)).is_some());
    // The editor: Back, Delete when editing, then its fields and Show keyboard.
    edit(&mut a, &mut text, 2, 0);
    let order = focus_order(&mut a);
    assert_eq!(order[..2], [1, DELETE_ID]);
    assert_eq!(order.iter().filter(|o| **o == DELETE_ID).count(), 1);
    assert!(order.contains(&part_id(&a, Part::PropEncoder)));
    assert!(
        !order.contains(&part_id(&a, Part::PropName)),
        "a fixed name is not focused"
    );
}
