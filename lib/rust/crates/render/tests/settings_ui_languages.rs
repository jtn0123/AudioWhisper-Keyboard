//! Languages & Layouts (stage4-languages.md): the editable subtype list, what each tap writes,
//! undo, its accessibility actions, and how the screens ask Kotlin for the report. The editor is
//! in settings_ui_languages_editor.rs; Select language and the pack manager in
//! settings_ui_languages_picker.rs.
mod languages_common;
use languages_common::*;
use render::settings_ui::{
    App,
    items::{ADD_ID, EMPTY_ID, UNDO_ID, item_id},
    languages::{self, ADD_ROUTE, EDIT_ROUTE, LIST_ROUTE, PACKS_ROUTE, PICKER_ROUTE},
};

fn three() -> String {
    list(&[english(), german(), french()])
}

#[test]
fn every_language_screen_is_native_and_asks_kotlin_for_the_report_once() {
    for route in [
        LIST_ROUTE,
        PICKER_ROUTE,
        PACKS_ROUTE,
        ADD_ROUTE,
        &format!("{EDIT_ROUTE}/1"),
    ] {
        let mut a = App::default();
        a.navigate(route);
        assert_eq!(a.stack.last().map(String::as_str), Some(route));
        assert!(a.effects.iter().all(|e| e[0] != "compose"), "{route}");
        let asks = a.effects.iter().filter(|e| e[0] == "languages").count();
        assert_eq!(asks, 1, "{route}");
        // A second screen before the answer does not ask again.
        a.navigate(PICKER_ROUTE);
        assert_eq!(a.effects.iter().filter(|e| e[0] == "languages").count(), 1);
    }
    // With the report in, the list and the pack manager read it again; the others reuse it.
    let (mut a, _) = app(LIST_ROUTE, &three());
    a.navigate(PICKER_ROUTE);
    assert!(a.effects.is_empty());
    a.back();
    a.navigate(PACKS_ROUTE);
    assert_eq!(a.effects, [["languages"]]);
}

#[test]
fn each_subtype_is_titled_with_its_language_and_summarised_as_localization_screen_did() {
    let odd = stored(9, "ja-JP", "en", "unknown", "yen");
    let (a, _) = app(LIST_ROUTE, &list(&[english(), german(), odd]));
    let items = a.list_items(render::settings_ui::items::Source::Subtypes);
    let titles: Vec<_> = items.iter().map(|i| i.title.as_str()).collect();
    assert_eq!(
        titles,
        [
            "English (United States)",
            "German (Germany)",
            "Japanese (Japan)"
        ]
    );
    // The summary string is the key here: no strings are loaded, so labels show their keys.
    let summary = |s: &str| s.replace("settings__localization__subtype_summary", "");
    assert_eq!(summary(&items[0].summary), "");
    let mut a = a;
    a.strings.insert(
        "settings__localization__subtype_summary".into(),
        "{characters_name} / {symbols_name} / {currency_set_name}".into(),
    );
    let items = a.list_items(render::settings_ui::items::Source::Subtypes);
    assert_eq!(items[1].summary, "QWERTZ / Western / Euro");
    // A component the keyboard does not have shows its name, where Compose showed "null".
    assert_eq!(
        items[2].summary,
        "org.florisboard.layouts:unknown / Western / org.florisboard.currencysets:yen"
    );
    // Native names when Display language names in asks for them.
    a.prefs
        .insert(languages::NAMES_KEY.into(), "NATIVE_LOCALE".into());
    assert_eq!(
        a.list_items(render::settings_ui::items::Source::Subtypes)[1].title,
        "Deutsch (Deutschland)"
    );
}

#[test]
fn moving_a_subtype_writes_the_reordered_list_byte_for_byte() {
    let (mut a, mut text) = app(LIST_ROUTE, &three());
    // The first has no Move up and the last no Move down: they are drawn off.
    assert!(!control(&a, item_id(0, 1)).unwrap().enabled);
    assert!(!control(&a, item_id(2, 2)).unwrap().enabled);
    assert!(!tap(&mut a, &mut text, item_id(0, 1)));
    assert!(tap(&mut a, &mut text, item_id(1, 1)));
    assert_eq!(written(&a).unwrap(), list(&[german(), english(), french()]));
    // German keeps its node ids while it moves, so accessibility focus stays on it.
    assert_eq!(
        control(&a, item_id(1, 0)).unwrap().label,
        "German (Germany)"
    );
    let bounds = |a: &App, slot| control(a, item_id(slot, 0)).unwrap().bounds.top;
    assert!(bounds(&a, 1) < bounds(&a, 0));
    a.effects.clear();
    assert!(tap(&mut a, &mut text, item_id(1, 2)));
    assert_eq!(written(&a).unwrap(), three());
}

#[test]
fn a_deleted_subtype_comes_back_where_it_stood_with_undo() {
    let (mut a, mut text) = app(LIST_ROUTE, &three());
    a.strings
        .insert("settings__list__deleted".into(), "Deleted {name}".into());
    assert!(tap(&mut a, &mut text, item_id(1, 3)));
    assert_eq!(written(&a).unwrap(), list(&[english(), french()]));
    assert_eq!(a.focus, UNDO_ID);
    assert_eq!(
        control(&a, UNDO_ID + 1).unwrap().label,
        "Deleted German (Germany)"
    );
    assert_eq!(control(&a, UNDO_ID + 1).unwrap().role, "status");
    a.effects.clear();
    assert!(tap(&mut a, &mut text, UNDO_ID));
    assert_eq!(written(&a).unwrap(), three());
    assert!(control(&a, UNDO_ID).is_none());
}

#[test]
fn undo_lasts_until_the_list_changes_or_the_user_leaves() {
    let (mut a, mut text) = app(LIST_ROUTE, &three());
    assert!(tap(&mut a, &mut text, item_id(0, 3)));
    assert!(control(&a, UNDO_ID).is_some());
    // A move ends it: putting the old item back would undo the move too.
    assert!(tap(&mut a, &mut text, item_id(2, 1)));
    assert!(control(&a, UNDO_ID).is_none());
    assert!(tap(&mut a, &mut text, item_id(2, 3)));
    a.navigate(PACKS_ROUTE);
    a.back();
    a.scene(&mut text);
    assert!(control(&a, UNDO_ID).is_none());
    // Undo puts nothing back over a subtype with the same id that returned meanwhile.
    let (mut a, mut text) = app(LIST_ROUTE, &three());
    assert!(tap(&mut a, &mut text, item_id(0, 3)));
    a.prefs.insert(languages::SUBTYPES_KEY.into(), three());
    a.effects.clear();
    assert!(tap(&mut a, &mut text, UNDO_ID));
    assert_eq!(written(&a), None);
}

#[test]
fn an_item_offers_move_and_delete_as_custom_actions_naming_the_controls_they_press() {
    let (a, _) = app(LIST_ROUTE, &three());
    let up = "settings__list__move_up".to_string();
    let down = "settings__list__move_down".to_string();
    let delete = "action__delete".to_string();
    assert_eq!(
        a.custom_actions_for(item_id(0, 0)),
        [
            (item_id(0, 2), down.clone()),
            (item_id(0, 3), delete.clone())
        ]
    );
    assert_eq!(
        a.custom_actions_for(item_id(1, 0)),
        [
            (item_id(1, 1), up.clone()),
            (item_id(1, 2), down),
            (item_id(1, 3), delete.clone())
        ]
    );
    assert_eq!(
        a.custom_actions_for(item_id(2, 0)),
        [(item_id(2, 1), up), (item_id(2, 3), delete)]
    );
    assert!(a.custom_actions_for(ADD_ID).is_empty());
    // Android tells a custom action from a standard one by its id: a standard id is one bit.
    for slot in 0..4096 {
        for part in 1..4 {
            assert_ne!(
                item_id(slot, part).count_ones(),
                1,
                "slot {slot} part {part}"
            );
        }
    }
    // Each button says which item it moves.
    let c = control(&a, item_id(2, 1)).unwrap();
    assert_eq!((c.role, c.summary.as_str()), ("button", "French (France)"));
}

#[test]
fn the_focus_order_walks_each_item_then_its_buttons_then_add() {
    let (mut a, mut text) = app(LIST_ROUTE, &list(&[english(), german()]));
    let mut seen = vec![];
    a.focus = 1;
    for _ in 0..20 {
        a.focus_next(false);
        a.scene(&mut text);
        seen.push(a.focus);
    }
    let want = [
        item_id(0, 0),
        item_id(0, 2),
        item_id(0, 3),
        item_id(1, 0),
        item_id(1, 1),
        item_id(1, 3),
        ADD_ID,
    ];
    let at = seen
        .iter()
        .position(|&f| f == want[0])
        .unwrap_or_else(|| panic!("{seen:?}"));
    assert_eq!(&seen[at..at + want.len()], &want);
}

#[test]
fn an_empty_list_warns_and_add_opens_the_editor() {
    let (mut a, mut text) = app(LIST_ROUTE, "[]");
    let warning = control(&a, EMPTY_ID).unwrap();
    assert_eq!(
        warning.label,
        "settings__localization__subtype_no_subtypes_configured_warning"
    );
    assert!(tap(&mut a, &mut text, ADD_ID));
    assert_eq!(a.stack.last().unwrap(), ADD_ROUTE);
    // Opening an item opens its editor at its id.
    let (mut a, mut text) = app(LIST_ROUTE, &three());
    assert!(tap(&mut a, &mut text, item_id(2, 0)));
    assert_eq!(a.stack.last().unwrap(), &format!("{EDIT_ROUTE}/3"));
}

#[test]
fn the_list_is_read_as_kotlin_normalised_it_while_the_preference_is_unchanged() {
    // Kotlin read "en_US" and gave back "en-US": the list shows and writes the normalised form.
    let raw = list(&[english().replace("en-US", "en_US"), german()]);
    let (mut a, mut text) = app(LIST_ROUTE, &raw);
    let mut report = report(&raw);
    report[wire::settings_ui::languages::row::DETAIL] = list(&[english(), german()]);
    a.set_languages(&report).unwrap();
    assert!(tap(&mut a, &mut text, item_id(1, 3)));
    assert_eq!(written(&a).unwrap(), list(&[english()]));
    // Once the preference changes, the stored list is read as it is.
    a.prefs
        .insert(languages::SUBTYPES_KEY.into(), list(&[french()]));
    assert_eq!(a.subtypes().len(), 1);
}

#[test]
fn a_malformed_report_is_refused_whole() {
    let mut a = App::default();
    let good = report(&three());
    assert!(a.set_languages(&good[1..]).is_err());
    let mut bad_kind = good.clone();
    bad_kind[0] = "nonsense".into();
    assert!(a.set_languages(&bad_kind).is_err());
    let orphan = report_row(&[(0, "packItem"), (1, "org.none"), (2, "x")]);
    assert!(a.set_languages(&[good.clone(), orphan].concat()).is_err());
    let mut bad_preset = good.clone();
    let at = bad_preset.iter().position(|f| f == "preset").unwrap();
    bad_preset[at + 5] = "{".into();
    assert!(a.set_languages(&bad_preset).is_err());
    assert!(a.languages.resources.is_none());
    assert!(a.set_languages(&good).is_ok());
    let r = a.languages.resources.as_ref().unwrap();
    assert_eq!((r.locales.len(), r.presets.len(), r.packs.len()), (6, 3, 2));
    assert_eq!(r.packs[0].items, ["Zhengma", "Cangjie"]);
}
