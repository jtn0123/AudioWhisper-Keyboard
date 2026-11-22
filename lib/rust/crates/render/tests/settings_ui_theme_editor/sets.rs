//! A font's source sets: Add source set on its title line, each "Source set" heading's Up,
//! Down, Delete (with Undo) and Add, and the Errors box naming a required property a set lacks.
use super::{common::*, props::*};
use render::settings_ui::{
    App,
    catalogue::theme_editor::Part,
    items::UNDO_ID,
    theme_editor::{
        PROPERTY_ADD,
        paint::rule_id,
        sets::{REQUIRED, set_id},
    },
};
use render::text::Text;
use snygg::{resolve::PropertySets, text as name};

/// The names in each set of rule `r`.
fn sets(a: &App, r: usize) -> Vec<Vec<String>> {
    match &a.theme_editor.rules[r].1 {
        PropertySets::Multiple(all) => all
            .iter()
            .map(|set| {
                set.iter()
                    .map(|(n, _)| String::from_utf16_lossy(n))
                    .collect()
            })
            .collect(),
        PropertySets::Single(_) => panic!("rule {r} has one set"),
    }
}
/// The font rule with a second, empty set.
fn two_sets() -> (App, Text) {
    let (mut a, mut text) = named_app();
    a.strings.insert(
        REQUIRED.into(),
        "Required property '{name}' does not exist".into(),
    );
    let add = control(&a, rule_id(1, 4)).unwrap();
    assert_eq!(add.label, "settings__theme_editor__add_source_set");
    assert!(tap(&mut a, &mut text, rule_id(1, 4)));
    assert_eq!(sets(&a, 1), [vec!["src", "font-weight"], vec![]]);
    assert!(a.theme_editor.changed());
    (a, text)
}

#[test]
fn add_source_set_appends_an_empty_set_the_errors_box_says_lacks_src() {
    let (a, _) = two_sets();
    assert!(a.stack.last().unwrap().starts_with("settings/theme/edit/"));
    assert!(
        control(&a, set_id(1, 0, 5)).is_none(),
        "the first set has its src"
    );
    let title = control(&a, set_id(1, 1, 6)).unwrap();
    assert_eq!(title.label, "settings__theme_editor__property_errors");
    let message = control(&a, set_id(1, 1, 5)).unwrap();
    assert_eq!(message.label, "Required property 'src' does not exist");
    let heading = control(&a, set_id(1, 1, 0)).unwrap();
    assert_eq!(heading.label, "Source set");
    assert!(
        heading.bounds.bottom <= message.bounds.top,
        "under its heading"
    );
    // The stylesheet cannot be written so, as Compose's `toJson` could not: the box says why.
    let refused = a.theme_editor.json().unwrap_err();
    assert!(
        refused.contains("missing required property 'src'"),
        "{refused}"
    );
}

#[test]
fn only_a_set_lacking_src_has_the_box_and_an_element_rule_never_does() {
    let (mut a, mut text) = two_sets();
    assert!(tap(&mut a, &mut text, rule_id(1, 4)));
    assert!(tap(&mut a, &mut text, set_id(1, 2, 4)));
    choose(&mut a, &mut text, "Source", "snygg__property_value__uri");
    pick(&mut a, &mut text, Part::PropValue, "flex:/fonts/a.ttf");
    apply(&mut a, &mut text);
    let boxes: Vec<_> = (0..3)
        .map(|s| control(&a, set_id(1, s, 5)).is_some())
        .collect();
    assert_eq!(boxes, [false, true, false]);
    assert!((2..5).all(|r| control(&a, set_id(r, 0, 5)).is_none()));
    assert!(tap(&mut a, &mut text, set_id(1, 1, 3)));
    assert!(
        a.theme_editor.json().is_ok(),
        "written again once no set lacks src"
    );
}

#[test]
fn a_sets_heading_moves_it_up_and_down_but_not_past_the_ends() {
    let (mut a, mut text) = two_sets();
    let up = control(&a, set_id(1, 0, 1)).unwrap();
    assert_eq!(
        (up.label.as_str(), up.enabled),
        ("settings__theme_editor__move_up", false)
    );
    assert!(!control(&a, set_id(1, 1, 2)).unwrap().enabled);
    assert_eq!(
        control(&a, set_id(1, 1, 2)).unwrap().label,
        "settings__theme_editor__move_down"
    );
    assert!(control(&a, set_id(1, 0, 2)).unwrap().enabled);
    assert!(tap(&mut a, &mut text, set_id(1, 1, 1)));
    assert_eq!(sets(&a, 1), [vec![], vec!["src", "font-weight"]]);
    assert!(
        control(&a, set_id(1, 0, 5)).is_some(),
        "the box moved with its set"
    );
    assert!(tap(&mut a, &mut text, set_id(1, 0, 2)));
    assert_eq!(sets(&a, 1), [vec!["src", "font-weight"], vec![]]);
    // Disabled: Up on the first does nothing.
    a.activate(set_id(1, 0, 1));
    assert_eq!(sets(&a, 1), [vec!["src", "font-weight"], vec![]]);
}

#[test]
fn deleting_a_set_leaves_undo_where_it_was_and_undo_puts_it_back() {
    let (mut a, mut text) = two_sets();
    let delete = control(&a, set_id(1, 0, 3)).unwrap();
    assert_eq!(delete.label, "action__delete");
    assert!(tap(&mut a, &mut text, set_id(1, 0, 3)));
    assert_eq!(sets(&a, 1), [Vec::<String>::new()]);
    let undo = control(&a, UNDO_ID).unwrap();
    let heading = control(&a, set_id(1, 0, 0)).unwrap();
    assert!(
        undo.bounds.bottom <= heading.bounds.top,
        "Undo stands where the set was"
    );
    assert!(undo.label.contains("Source set") || undo.summary.contains("Source set"));
    assert!(tap(&mut a, &mut text, UNDO_ID));
    assert_eq!(sets(&a, 1), [vec!["src", "font-weight"], vec![]]);
    // The last set deleted: Undo after the remaining ones.
    assert!(tap(&mut a, &mut text, set_id(1, 1, 3)));
    let undo = control(&a, UNDO_ID).unwrap();
    assert!(undo.bounds.top >= control(&a, set_id(1, 0, 0)).unwrap().bounds.bottom);
    assert!(tap(&mut a, &mut text, UNDO_ID));
    assert_eq!(sets(&a, 1).len(), 2);
}

#[test]
fn a_sets_add_adds_a_property_to_that_set() {
    let (mut a, mut text) = two_sets();
    assert!(tap(&mut a, &mut text, set_id(1, 1, 4)));
    assert_eq!(a.stack.last().unwrap(), PROPERTY_ADD);
    assert_eq!(a.theme_editor.prop.as_ref().unwrap().set, 1);
    let names = listed(&mut a, &mut text, Part::PropName);
    assert_eq!(
        names,
        [
            PLACEHOLDER,
            "Source",
            "snygg__property_name__font_style",
            "Font weight"
        ]
    );
    choose(&mut a, &mut text, "Source", "snygg__property_value__uri");
    pick(&mut a, &mut text, Part::PropValue, "flex:/fonts/a.ttf");
    apply(&mut a, &mut text);
    assert_eq!(sets(&a, 1), [vec!["src", "font-weight"], vec!["src"]]);
    assert!(
        control(&a, set_id(1, 1, 5)).is_none(),
        "the box goes with the lack"
    );
    // The first set has a weight already.
    assert!(tap(&mut a, &mut text, set_id(1, 0, 4)));
    choose(
        &mut a,
        &mut text,
        "Font weight",
        "snygg__property_value__font_weight",
    );
    apply(&mut a, &mut text);
    assert_eq!(a.stack.last().unwrap(), PROPERTY_ADD);
    // A property of the second set opens as the list counts them, after the first set's.
    a.back();
    edit(&mut a, &mut text, 1, 2);
    let f = a.theme_editor.prop.as_ref().unwrap();
    assert_eq!((f.set, f.name.as_str()), (1, "src"));
    assert_eq!(f.original.as_deref().map(name), Some(name("src")));
}

#[test]
fn the_focus_order_takes_each_headings_enabled_buttons() {
    let (mut a, _) = two_sets();
    let order = focus_order(&mut a);
    let at = |id: i32| order.iter().position(|o| *o == id);
    assert!(at(set_id(1, 0, 1)).is_none(), "a disabled Up is skipped");
    assert!(at(set_id(1, 1, 2)).is_none(), "a disabled Down is skipped");
    let wanted = [
        rule_id(1, 4),
        set_id(1, 0, 2),
        set_id(1, 0, 3),
        set_id(1, 0, 4),
        rule_id(1, 64),
        set_id(1, 1, 1),
        set_id(1, 1, 3),
        set_id(1, 1, 4),
    ];
    let found: Vec<_> = wanted.iter().map(|id| at(*id).unwrap()).collect();
    assert!(found.windows(2).all(|w| w[0] < w[1]), "{found:?}");
}
