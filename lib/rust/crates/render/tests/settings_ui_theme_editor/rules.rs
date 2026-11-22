//! Loading, the rules list, Delete with Undo, Duplicate, fine-tune, Save and Back.
use super::common::*;
use render::settings_ui::{
    catalogue::languages::Lang,
    items::UNDO_ID,
    theme_editor::{
        Load, RULE_ADD, RULE_EDIT, TUNE,
        paint::{ADD_ID, DELETE_ID, EMPTY_ID, NO_ID, SAVE_ID, STATUS_ID, TUNE_ID, YES_ID, rule_id},
    },
};
use snygg::{json::parse, json_write::to_pretty_json};
use wire::settings_ui::theme_editor::{effect, outcome, report};

/// Rule `r`'s `n`th property's Delete.
fn delete_id(r: usize, n: usize) -> i32 {
    rule_id(r, 65 + 2 * n)
}
fn property_texts(a: &render::settings_ui::App, r: usize) -> Vec<(String, String)> {
    let props = a.rule_properties(r);
    props.into_iter().map(|p| (p.name, p.value)).collect()
}

#[test]
fn the_editor_says_it_is_loading_until_kotlin_answers_then_shows_the_component() {
    let mut a = opened();
    let mut text = text();
    a.strings
        .insert("settings__theme_editor__loading".into(), "Loading…".into());
    a.scene(&mut text);
    assert_eq!(control(&a, STATUS_ID).unwrap().label, "Loading…");
    assert!(control(&a, ADD_ID).is_none());
    // A report for another workspace is not this editor's.
    let mut other = loaded(SHEET);
    other[report::WORKSPACE] = "elsewhere".into();
    a.set_theme_editor(&other).unwrap();
    assert_eq!(a.theme_editor.load, Load::Waiting);
    assert!(a.set_theme_editor(&other[1..]).is_err());
    a.set_theme_editor(&loaded(SHEET)).unwrap();
    a.scene(&mut text);
    assert!(control(&a, STATUS_ID).is_none());
    assert_eq!(
        control(&a, ADD_ID).unwrap().label,
        "settings__theme_editor__add_rule"
    );
    // The component box: its label over `extension:id`, then its meta and path.
    let name = control(&a, 400_011).unwrap();
    assert_eq!(name.label, "local.my.themes:sunny");
    let body = control(&a, 400_012).unwrap().label;
    assert_eq!(
        body,
        "authors = [Ada, Grace]\nisNightTheme = false\nstylesheetPath = stylesheets/sunny.json"
    );
    assert_eq!(control(&a, 400_010).unwrap().label, "Sunny");
}

#[test]
fn a_gone_workspace_says_so_and_offers_nothing_to_edit() {
    let mut a = opened();
    let mut gone = loaded("");
    gone[report::OUTCOME] = outcome::MISSING.into();
    a.set_theme_editor(&gone).unwrap();
    let mut text = text();
    a.scene(&mut text);
    let status = control(&a, STATUS_ID).unwrap();
    assert_eq!(status.label, "settings__theme_editor__missing_workspace");
    assert!(control(&a, ADD_ID).is_none() && control(&a, SAVE_ID).is_none());
}

#[test]
fn a_stylesheet_that_fails_asks_and_yes_loads_it_leniently_and_no_starts_empty() {
    let broken = r##"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet",
        "key":{"background":"#ff0000","nope":"x"}}"##;
    let (mut a, mut text) = app(broken);
    assert!(matches!(a.theme_editor.load, Load::Failed(_)));
    assert!(control(&a, ADD_ID).is_none() && control(&a, SAVE_ID).is_none());
    assert_eq!(control(&a, YES_ID).unwrap().label, "action__yes");
    assert!(tap(&mut a, &mut text, YES_ID));
    assert_eq!(a.theme_editor.load, Load::Ready);
    assert_eq!(titles(&a), ["@defines", "Key"]);
    assert_eq!(property_texts(&a, 1).len(), 1);
    let (mut a, mut text) = app(broken);
    assert!(tap(&mut a, &mut text, NO_ID));
    assert_eq!(titles(&a), ["@defines"]);
    assert!(
        control(&a, EMPTY_ID).is_some(),
        "the note that no rule is defined"
    );
}

#[test]
fn the_rules_come_in_compare_order_with_their_properties_named_and_written_out() {
    let (a, _) = app(SHEET);
    assert_eq!(
        titles(&a),
        ["@defines", "@font `Sans`", "Key", "Key", "Key"]
    );
    let lines: Vec<_> = a
        .theme_editor
        .rules
        .iter()
        .map(|(r, _)| a.rule_lines(r))
        .collect();
    assert_eq!(
        lines[3],
        ("Key".into(), String::new(), vec!["code = [10]".into()])
    );
    assert_eq!(lines[4], ("Key".into(), "Pressed".into(), vec![]));
    assert_eq!(
        property_texts(&a, 2),
        [
            // translatePropertyValue: a variable by its name, translated where it can be.
            ("Background".into(), "--accent".into()),
            ("Foreground".into(), "#000000ff".into())
        ]
    );
    assert_eq!(
        property_texts(&a, 0),
        [("--accent".into(), "#ff0000ff".into())]
    );
    // Talkback hears a rule as its title, selector and attributes.
    let row = control(&a, rule_id(3, 0)).unwrap();
    assert_eq!(
        (row.label.as_str(), row.summary.as_str()),
        ("Key", "code = [10]")
    );
    assert!(control(&a, rule_id(0, 0)).unwrap().role == "text");
    assert!(control(&a, rule_id(0, 2)).is_none(), "@defines has no Edit");
}

#[test]
fn the_developer_level_shows_names_as_written() {
    let (mut a, _) = app(SHEET);
    a.prefs
        .insert("theme__editor_level".into(), "DEVELOPER".into());
    assert_eq!(
        titles(&a),
        ["@defines", "@font `Sans`", "key", "key", "key"]
    );
    assert_eq!(a.rule_lines(&a.theme_editor.rules[4].0).1, "pressed");
    assert_eq!(
        property_texts(&a, 2)[0],
        ("background".into(), "var(--accent)".into())
    );
}

#[test]
fn loading_previews_the_stylesheet_on_the_keyboard() {
    let mut a = opened();
    a.set_theme_editor(&loaded(SHEET)).unwrap();
    let preview = last(&a, effect::PREVIEW).unwrap();
    assert_eq!(preview[1..3], [WORKSPACE, "0"]);
    assert_eq!(preview[3], a.theme_editor.json().unwrap());
}

#[test]
fn a_deleted_property_leaves_undo_where_it_was_and_undo_puts_it_back() {
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, delete_id(2, 0)));
    assert_eq!(
        property_texts(&a, 2),
        [("Foreground".into(), "#000000ff".into())]
    );
    assert!(a.theme_editor.changed());
    let preview = last(&a, effect::PREVIEW).unwrap();
    assert!(!preview[3].contains("var(--accent)"));
    // The undo bar stands above the property now in the deleted one's place.
    let undo = control(&a, UNDO_ID).unwrap();
    let next = control(&a, rule_id(2, 64)).unwrap();
    assert!(undo.bounds.bottom <= next.bounds.top);
    assert!(undo.bounds.top > control(&a, rule_id(2, 0)).unwrap().bounds.top);
    assert!(tap(&mut a, &mut text, UNDO_ID));
    assert_eq!(
        property_texts(&a, 2)[0],
        ("Background".into(), "--accent".into())
    );
    assert!(!a.theme_editor.changed());
    assert!(control(&a, UNDO_ID).is_none());
}

#[test]
fn the_defines_note_and_an_undo_bar_keep_inside_the_rule_box() {
    let (mut a, mut text) = app(SHEET);
    // The box is 16 dp (42 px) in from each edge and its content 16 dp in from the box.
    let inner = 1080.0 - 2.0 * 42.0;
    let note = control(&a, rule_id(0, 3)).unwrap();
    assert!(note.bounds.right <= inner, "{:?}", note.bounds);
    assert!(tap(&mut a, &mut text, delete_id(2, 0)));
    let undo = control(&a, UNDO_ID).unwrap();
    assert!(undo.bounds.right <= inner, "{:?}", undo.bounds);
}

#[test]
fn a_fonts_source_has_no_delete_and_its_other_properties_do() {
    let (mut a, mut text) = app(SHEET);
    assert_eq!(a.rule_properties(1)[0].raw, "src");
    assert!(control(&a, delete_id(1, 0)).is_none());
    assert!(control(&a, delete_id(1, 1)).is_some());
    // The tap a stale control could still send is refused too.
    a.activate(delete_id(1, 1));
    a.scene(&mut text);
    assert_eq!(a.rule_properties(1).len(), 1);
}

#[test]
fn deleting_a_rule_asks_first_and_undo_puts_it_back_in_its_place() {
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, rule_id(4, 2)));
    assert_eq!(a.stack.last().unwrap(), RULE_EDIT);
    assert!(tap(&mut a, &mut text, DELETE_ID));
    assert!(a.controls.iter().any(|c| c.label == "Delete \"Key\"?"));
    // Cancel keeps it.
    assert!(tap(&mut a, &mut text, 6));
    assert_eq!(a.theme_editor.rules.len(), 5);
    assert!(tap(&mut a, &mut text, DELETE_ID));
    assert!(tap(&mut a, &mut text, 7));
    assert!(a.stack.last().unwrap().starts_with("settings/theme/edit/"));
    assert_eq!(a.theme_editor.rules.len(), 4);
    let undo = control(&a, UNDO_ID).unwrap();
    assert!(undo.bounds.top > control(&a, rule_id(3, 0)).unwrap().bounds.top);
    assert!(tap(&mut a, &mut text, UNDO_ID));
    assert_eq!(a.rule_lines(&a.theme_editor.rules[4].0).1, "Pressed");
    assert!(!a.theme_editor.changed());
}

#[test]
fn duplicate_adds_a_rule_like_another_with_its_properties() {
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, rule_id(4, 1)));
    assert_eq!(a.stack.last().unwrap(), RULE_ADD);
    // As it stands it is the rule it copies.
    let add = lang_id(&a, Lang::Save);
    assert!(tap(&mut a, &mut text, add));
    assert_eq!(a.stack.last().unwrap(), RULE_ADD);
    assert_eq!(
        a.rule_error(),
        "settings__theme_editor__rule_already_exists"
    );
    // Pressed off, Focus on: a new rule with the pressed rule's background.
    assert!(tap(&mut a, &mut text, 400_072));
    assert!(tap(&mut a, &mut text, add));
    assert_eq!(a.theme_editor.rules.len(), 6);
    let focus = a
        .theme_editor
        .rules
        .iter()
        .position(|(r, _)| a.rule_lines(r).1 == "snygg__rule_selector__focus");
    let props = property_texts(&a, focus.unwrap());
    assert_eq!(props, [("Background".into(), "#00ff00ff".into())]);
}

#[test]
fn fine_tune_lists_its_three_preferences_and_the_keyboard_comes_back_as_told() {
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, TUNE_ID));
    assert_eq!(a.stack.last().unwrap(), TUNE);
    let keys: Vec<_> = a.rows().iter().map(|r| r.key).collect();
    assert_eq!(
        keys,
        [
            "theme__editor_level",
            "theme__editor_color_representation",
            "theme__editor_display_kbd_after_dialogs"
        ]
    );
    a.prefs.insert(
        "theme__editor_display_kbd_after_dialogs".into(),
        "ALWAYS".into(),
    );
    a.back();
    assert!(a.editor.focused);
    assert!(a.effects.contains(&vec!["editor".to_string()]));
    a.effects.clear();
    a.editor.focused = false;
    a.prefs.insert(
        "theme__editor_display_kbd_after_dialogs".into(),
        "NEVER".into(),
    );
    assert!(tap(&mut a, &mut text, TUNE_ID));
    a.back();
    assert!(!a.editor.focused && a.effects.is_empty());
}

#[test]
fn save_writes_the_stylesheet_rust_built_and_the_meta_then_closes() {
    let (mut a, mut text) = app(SHEET);
    assert!(control(&a, SAVE_ID).unwrap().role == "button");
    assert!(tap(&mut a, &mut text, delete_id(2, 1)));
    let json = a.theme_editor.json().unwrap();
    // With the keyboard up, Save still closes: Back alone would only hide it.
    a.editor.focused = true;
    assert!(tap(&mut a, &mut text, SAVE_ID));
    let save = last(&a, effect::SAVE).unwrap();
    assert_eq!(
        save[1..],
        [
            WORKSPACE,
            "0",
            &json,
            "Sunny",
            "Ada\nGrace",
            wire::flag::FALSE,
            "sunny",
            "",
            "Neutral",
            "Default",
            "SPEC_2021"
        ]
    );
    assert_eq!(a.effects.last().unwrap(), &["finish"]);
    // Rust's JSON is what the serializer writes for what it reads back.
    let (schema, sheet) = parse(&json, false).unwrap();
    assert_eq!(to_pretty_json(&schema, &sheet).unwrap(), json);
    assert!(!a.theme_editor.changed());
}

#[test]
fn back_with_unsaved_changes_asks_to_discard_and_without_them_closes() {
    let (mut a, _) = app(SHEET);
    assert!(!a.theme_holds_back());
    a.back();
    assert_eq!(a.effects, [["finish"]]);
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, delete_id(2, 1)));
    assert!(a.theme_holds_back());
    a.effects.clear();
    a.back();
    assert!(a.modal.is_some() && a.effects.is_empty());
    // Cancel stays with the draft; Discard closes and writes nothing.
    assert!(tap(&mut a, &mut text, 6));
    assert!(a.modal.is_none() && a.theme_editor.changed());
    a.back();
    assert!(tap(&mut a, &mut text, 7));
    assert!(last(&a, effect::SAVE).is_none());
    assert_eq!(a.effects.last().unwrap(), &["finish"]);
}
