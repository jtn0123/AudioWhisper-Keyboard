//! Select language and the language pack manager (stage4-languages.md), and the language
//! screens' part of the saved state.
mod languages_common;
use languages_common::*;
use render::settings_ui::{
    App,
    items::{UNDO_ID, item_id},
    languages::{
        self, EDIT_ROUTE, LIST_ROUTE, PACKS_ROUTE, PICKER_ROUTE,
        packs::{IMPORT_ID, pack_id},
        picker::{LOCALE_ID, SUGGESTED_ID},
    },
};

fn labels(a: &App, first: i32) -> Vec<String> {
    (0..)
        .map_while(|k| control(a, first + k))
        .map(|c| c.label)
        .collect()
}

#[test]
fn select_language_lists_every_locale_by_shown_name_with_the_system_languages_first() {
    let (mut a, mut text) = app(PICKER_ROUTE, "[]");
    assert_eq!(
        labels(&a, SUGGESTED_ID),
        ["English (United States)", "German (Germany)"]
    );
    assert_eq!(
        labels(&a, LOCALE_ID),
        [
            "English (United States)",
            "French (Canada)",
            "French (France)",
            "German (Germany)",
            "Japanese (Japan)",
            "Spanish (Spain)",
        ]
    );
    for (id, heading) in [
        (29998, "settings__localization__suggested_languages"),
        (29999, "settings__localization__all_languages"),
    ] {
        let c = control(&a, id).unwrap();
        assert_eq!((c.label.as_str(), c.role), (heading, "heading"));
    }
    // Native names sort by the native name, as SelectLocaleScreen sorted.
    a.prefs
        .insert(languages::NAMES_KEY.into(), "NATIVE_LOCALE".into());
    a.scene(&mut text);
    assert_eq!(
        labels(&a, LOCALE_ID)[..3],
        [
            "Deutsch (Deutschland)",
            "English (United States)",
            "español (España)"
        ]
    );
}

#[test]
fn the_search_filters_as_select_locale_screen_did() {
    let (mut a, mut text) = app(PICKER_ROUTE, "[]");
    let search = |a: &mut App, text: &mut render::text::Text, term: &str| {
        a.languages.search = term.into();
        a.scene(text);
        labels(a, LOCALE_ID)
    };
    // Any of the three names contains the term, ignoring case and spaces round it.
    assert_eq!(
        search(&mut a, &mut text, "  FRENCH "),
        ["French (Canada)", "French (France)"]
    );
    assert_eq!(search(&mut a, &mut text, "deutsch"), ["German (Germany)"]);
    // A tag only matches from its start, in either form.
    assert_eq!(search(&mut a, &mut text, "fr-c"), ["French (Canada)"]);
    assert_eq!(search(&mut a, &mut text, "fr_c"), ["French (Canada)"]);
    assert!(search(&mut a, &mut text, "r-ca").is_empty());
    // A search hides the suggestions.
    assert!(control(&a, SUGGESTED_ID).is_none());
}

#[test]
fn nothing_found_says_so_with_the_search_term() {
    let (mut a, mut text) = app(PICKER_ROUTE, "[]");
    a.strings.insert(
        "settings__localization__subtype_search_locale_not_found".into(),
        "No language found for \u{201c}{search_term}\u{201d}".into(),
    );
    a.languages.search = "klingon".into();
    a.scene(&mut text);
    assert!(control(&a, LOCALE_ID).is_none());
    let message = a
        .controls
        .iter()
        .find(|c| c.label.starts_with("No language"))
        .unwrap();
    assert_eq!(
        message.label,
        "No language found for \u{201c}klingon\u{201d}"
    );
    // The field is a single-line search: it starts empty each time the screen opens.
    let field = a.controls.iter().find(|c| c.role == "edit").unwrap();
    assert!(a.line_field(field.id));
    a.back();
    a.navigate(PICKER_ROUTE);
    assert!(a.languages.search.is_empty());
}

#[test]
fn a_tap_outside_the_editor_does_nothing_but_go_back() {
    // Select language reached some other way has no draft to answer: it only closes.
    let (mut a, mut text) = app(PICKER_ROUTE, "[]");
    assert!(tap(&mut a, &mut text, LOCALE_ID));
    assert_eq!(a.stack.last().unwrap(), LIST_ROUTE);
    assert!(a.effects.iter().all(|e| e[0] != "write"));
}

#[test]
fn the_pack_manager_boxes_each_pack_with_its_items_and_delete_when_it_may() {
    let (a, _) = app(PACKS_ROUTE, "[]");
    assert_eq!(control(&a, IMPORT_ID).unwrap().label, "action__import");
    assert_eq!(
        control(&a, pack_id(0, 0)).unwrap().label,
        "Han language pack"
    );
    assert_eq!(
        control(&a, pack_id(0, 1)).unwrap().label,
        "org.florisboard.han"
    );
    // Items in the report's order: Kotlin sorts them by label.
    assert_eq!(control(&a, pack_id(0, 3)).unwrap().label, "Zhengma");
    assert_eq!(control(&a, pack_id(0, 4)).unwrap().label, "Cangjie");
    assert_eq!(control(&a, pack_id(0, 2)).unwrap().label, "action__delete");
    // A pack the extension manager may not delete has no Delete.
    assert_eq!(control(&a, pack_id(1, 0)).unwrap().label, "Built in");
    assert!(control(&a, pack_id(1, 2)).is_none());
}

#[test]
fn delete_asks_first_naming_the_pack_and_only_then_asks_kotlin() {
    let (mut a, mut text) = app(PACKS_ROUTE, "[]");
    a.strings.insert(
        "action__delete_confirm_message".into(),
        "Delete {name}?".into(),
    );
    assert!(tap(&mut a, &mut text, pack_id(0, 2)));
    let message = a
        .controls
        .iter()
        .find(|c| c.label == "Delete Han language pack?");
    assert!(message.is_some());
    // Cancel asks nothing.
    assert!(tap(&mut a, &mut text, 6));
    assert!(a.modal.is_none() && a.effects.is_empty());
    assert!(tap(&mut a, &mut text, pack_id(0, 2)));
    assert!(tap(&mut a, &mut text, 7));
    assert_eq!(a.effects, [["deleteLanguagePack", "org.florisboard.han"]]);
}

#[test]
fn import_opens_the_language_pack_importer_and_a_packs_page_is_native() {
    let (mut a, mut text) = app(PACKS_ROUTE, "[]");
    assert!(tap(&mut a, &mut text, IMPORT_ID));
    assert!(a.effects.iter().all(|e| e[0] != "compose"));
    assert_eq!(a.stack.last().unwrap(), "ext/import/EXT_LANGUAGEPACK");
    for id in [pack_id(0, 0), pack_id(0, 1)] {
        let (mut a, mut text) = app(PACKS_ROUTE, "[]");
        assert!(tap(&mut a, &mut text, id));
        assert!(a.effects.iter().all(|e| e[0] != "compose"));
        assert_eq!(a.stack.last().unwrap(), "ext/view/org.florisboard.han");
    }
}

#[test]
fn the_draft_search_and_undo_survive_a_saved_state_and_bad_ones_change_nothing() {
    // In the editor: the draft, checked, and the search of Select language opened from it.
    let (mut a, _) = app(&format!("{EDIT_ROUTE}/2"), &list(&[english(), german()]));
    a.draft.as_mut().unwrap().checked = true;
    a.languages.search = "fr".into();
    let saved = a.languages_saved();
    let mut b = App::default();
    b.restore_languages(&saved).unwrap();
    assert_eq!(b.draft, a.draft);
    assert_eq!(b.languages.search, "fr");
    for bad in [
        "",
        "{}",
        r#"{"draft":[1,true,1,false],"search":"","deleted":null,"slots":[]}"#,
        r#"{"draft":null,"search":"","deleted":[0,"{",""],"slots":[]}"#,
        r#"{"draft":null,"search":3,"deleted":null,"slots":[]}"#,
        r#"{"draft":null,"search":"","deleted":null,"slots":[1]}"#,
    ] {
        let mut c = App::default();
        c.languages.search = "kept".into();
        assert!(c.restore_languages(bad).is_err(), "{bad}");
        assert_eq!(c.languages.search, "kept");
    }
    // On the list after a delete: the undo bar comes back, and the report is asked for again.
    let (mut a, mut text) = app(LIST_ROUTE, &list(&[english(), german()]));
    assert!(tap(&mut a, &mut text, item_id(1, 3)));
    let saved = a.languages_saved();
    let mut c = App {
        config: a.config,
        ..App::default()
    };
    c.prefs = a.prefs.clone();
    c.prefs
        .insert(languages::SUBTYPES_KEY.into(), list(&[english()]));
    c.navigate(LIST_ROUTE);
    c.languages.requested = false;
    c.effects.clear();
    c.restore_languages(&saved).unwrap();
    assert_eq!(c.list, a.list);
    c.languages_restored();
    assert_eq!(c.effects, [["languages"]]);
    c.set_languages(&report(&list(&[english()]))).unwrap();
    c.scene(&mut text);
    assert!(control(&c, UNDO_ID).is_some());
    c.effects.clear();
    assert!(tap(&mut c, &mut text, UNDO_ID));
    assert_eq!(written(&c).unwrap(), list(&[english(), german()]));
    // A restored editor whose draft was lost opens it again from the stored subtype.
    let mut d = App {
        prefs: a.prefs.clone(),
        ..App::default()
    };
    d.navigate(&format!("{EDIT_ROUTE}/1"));
    d.draft = None;
    d.languages_restored();
    assert_eq!(d.draft.as_ref().map(|x| x.id), Some(1));
}
