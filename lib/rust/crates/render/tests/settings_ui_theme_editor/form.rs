//! The rule editor: the element list, the selector chips, codes, modes and shift states, the
//! checks Add and Apply make, Apply renaming a rule in place, and Cancel.
use super::common::*;
use render::settings_ui::{
    catalogue::{Kind, languages::Lang, theme_editor::Part},
    theme_editor::{
        RULE_ADD, RULE_EDIT,
        form_paint::{ERROR_ID, SELECTOR_ID, VALUE_ID},
        paint::{ADD_ID, rule_id},
    },
};
use snygg::{grammar::Rule, resolve::PropertySets};

fn modal_part(a: &render::settings_ui::App) -> Option<Part> {
    match a.modal.as_ref().map(|m| a.rows()[m.row].kind) {
        Some(Kind::ThemeEdit(p)) => Some(p),
        _ => None,
    }
}
/// The rule editor adding, with element `n` of its list chosen (0 is "select", 1 `@font`).
fn adding(n: i32) -> (render::settings_ui::App, render::text::Text) {
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, ADD_ID));
    assert_eq!(a.stack.last().unwrap(), RULE_ADD);
    let element = part_id(&a, Part::Element);
    assert!(tap(&mut a, &mut text, element));
    assert_eq!(modal_part(&a), Some(Part::Element));
    assert!(tap(&mut a, &mut text, 3000 + n));
    assert!(a.modal.is_none());
    (a, text)
}

#[test]
fn add_with_no_element_marks_the_dropdown_and_adds_nothing() {
    let (mut a, mut text) = app(SHEET);
    a.strings.insert(
        "general__select_dropdown_value_placeholder".into(),
        "- select -".into(),
    );
    assert!(tap(&mut a, &mut text, ADD_ID));
    let element = part_id(&a, Part::Element);
    assert_eq!(control(&a, element).unwrap().summary, "- select -");
    // Selectors and attributes show only for an element.
    assert!(control(&a, SELECTOR_ID + 1).is_none());
    let add = lang_id(&a, Lang::Save);
    assert!(tap(&mut a, &mut text, add));
    assert!(a.theme_editor.form.as_ref().unwrap().checked);
    assert_eq!(a.theme_editor.rules.len(), 5);
    assert_eq!(a.stack.last().unwrap(), RULE_ADD);
    // The list: "select", the font annotation, then every element in FlorisImeUi's order.
    assert!(tap(&mut a, &mut text, element));
    let labels = choices(&a);
    assert_eq!(
        labels[..5],
        [
            "- select -",
            "snygg__rule_annotation__font",
            "snygg__rule_element__root",
            "snygg__rule_element__window",
            "Key"
        ]
    );
    assert_eq!(
        render::settings_ui::theme_editor::names::ELEMENTS[..2],
        ["root", "window"]
    );
}

#[test]
fn a_new_element_rule_takes_its_selector_codes_modes_and_shift_states() {
    let (mut a, mut text) = adding(4);
    assert_eq!(a.theme_editor.form.as_ref().unwrap().element, "key");
    assert!(tap(&mut a, &mut text, SELECTOR_ID + 1));
    assert!(control(&a, SELECTOR_ID + 1).unwrap().selected);
    // A second tap on the chosen selector clears it, as `updateCurrentRule` did.
    assert!(tap(&mut a, &mut text, SELECTOR_ID + 1));
    assert!(!control(&a, SELECTOR_ID + 1).unwrap().selected);
    assert!(tap(&mut a, &mut text, SELECTOR_ID + 4));
    type_into(&mut a, &mut text, Part::Attribute(0), " 32, -1 ,");
    // Modes: the Add chip lists the modes, here Numeric chosen.
    let modes = part_id(&a, Part::Attribute(1));
    assert!(control(&a, VALUE_ID + 16 + 15).is_some());
    assert!(tap(&mut a, &mut text, VALUE_ID + 16 + 15));
    assert_eq!(modal_part(&a), Some(Part::Attribute(1)));
    assert_eq!(choices(&a).len(), 7);
    assert!(tap(&mut a, &mut text, 3003));
    assert_eq!(control(&a, VALUE_ID + 16).unwrap().label, "Numeric");
    assert!(modes > 1000);
    // Shift states: two added, one removed by tapping its chip.
    for _ in 0..2 {
        assert!(tap(&mut a, &mut text, VALUE_ID + 32 + 15));
        assert!(tap(&mut a, &mut text, 3000));
    }
    assert!(tap(&mut a, &mut text, VALUE_ID + 32));
    let add = lang_id(&a, Lang::Save);
    assert!(tap(&mut a, &mut text, add));
    assert!(a.stack.last().unwrap().starts_with("settings/theme/edit/"));
    let (rule, sets) = a.theme_editor.rules.last().unwrap().clone();
    assert_eq!(
        String::from_utf16_lossy(&rule.format()),
        "key[code=-1,32][mode=`numeric`][shiftstate=`unshifted`]:disabled"
    );
    assert_eq!(sets, PropertySets::Single(vec![]));
    assert!(a.theme_editor.changed());
}

#[test]
fn a_rule_that_exists_or_a_bad_code_is_refused_with_why() {
    let (mut a, mut text) = adding(4);
    type_into(&mut a, &mut text, Part::Attribute(0), "10");
    let add = lang_id(&a, Lang::Save);
    assert!(tap(&mut a, &mut text, add));
    assert_eq!(
        control(&a, ERROR_ID).unwrap().label,
        "settings__theme_editor__rule_already_exists"
    );
    a.strings.insert(
        "settings__theme_editor__code_invalid".into(),
        "{c_min}-{c_max} or {i_min}-{i_max}".into(),
    );
    type_into(&mut a, &mut text, Part::Attribute(0), "10, 70000");
    assert!(tap(&mut a, &mut text, add));
    assert_eq!(control(&a, ERROR_ID).unwrap().label, "1-65535 or -9999--1");
    type_into(&mut a, &mut text, Part::Attribute(0), "0");
    assert!(tap(&mut a, &mut text, add));
    assert_eq!(a.stack.last().unwrap(), RULE_ADD);
    assert_eq!(a.theme_editor.rules.len(), 5);
}

#[test]
fn a_font_needs_a_name_the_stylesheet_can_read_back() {
    let (mut a, mut text) = adding(1);
    assert!(control(&a, SELECTOR_ID + 1).is_none());
    type_into(&mut a, &mut text, Part::Font, "My_font");
    let add = lang_id(&a, Lang::Save);
    assert!(tap(&mut a, &mut text, add));
    assert_eq!(a.rule_error(), "settings__theme_editor__font_name_invalid");
    type_into(&mut a, &mut text, Part::Font, "");
    assert!(tap(&mut a, &mut text, add));
    assert_eq!(a.stack.last().unwrap(), RULE_ADD);
    type_into(&mut a, &mut text, Part::Font, " Mono 2 ");
    assert!(tap(&mut a, &mut text, add));
    let font = a
        .theme_editor
        .rules
        .iter()
        .find(|(r, _)| *r == Rule::Font("Mono 2".encode_utf16().collect()));
    assert_eq!(font.unwrap().1, PropertySets::Multiple(vec![]));
}

#[test]
fn apply_renames_a_rule_in_place_and_keeps_its_properties() {
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, rule_id(3, 2)));
    assert_eq!(a.stack.last().unwrap(), RULE_EDIT);
    let element = part_id(&a, Part::Element);
    assert!(
        !control(&a, element).unwrap().enabled,
        "the element is fixed when editing"
    );
    assert!(!tap(&mut a, &mut text, element));
    assert!(a.modal.is_none());
    assert_eq!(a.theme_editor.form.as_ref().unwrap().codes, "10");
    type_into(&mut a, &mut text, Part::Attribute(0), "10, 13");
    let apply = lang_id(&a, Lang::Save);
    assert!(tap(&mut a, &mut text, apply));
    assert_eq!(a.theme_editor.rules.len(), 5);
    let (rule, sets) = &a.theme_editor.rules[3];
    assert_eq!(String::from_utf16_lossy(&rule.format()), "key[code=10,13]");
    assert!(matches!(sets, PropertySets::Single(p) if p.len() == 1));
    // Applying it unchanged is no clash with itself.
    assert!(tap(&mut a, &mut text, rule_id(3, 2)));
    assert!(tap(&mut a, &mut text, apply));
    assert!(a.stack.last().unwrap().starts_with("settings/theme/edit/"));
}

#[test]
fn cancel_leaves_the_rules_as_they_were() {
    let (mut a, mut text) = adding(4);
    let cancel = lang_id(&a, Lang::Cancel);
    assert!(tap(&mut a, &mut text, cancel));
    assert!(a.stack.last().unwrap().starts_with("settings/theme/edit/"));
    assert_eq!(a.theme_editor.rules.len(), 5);
    assert!(a.theme_editor.form.is_none() && !a.theme_editor.changed());
}
