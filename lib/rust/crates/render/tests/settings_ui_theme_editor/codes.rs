//! The code editor (part 3): a rule's codes as chips, each opening the editor, which adds, edits
//! and removes them one at a time or several at once, refuses a code out of range or one the rule
//! has, records a key from this app's keyboard through `THEME_CODE` reports, and shows the key
//! data preview.
use super::common::*;
use render::settings_ui::{
    App,
    catalogue::{
        languages::Lang,
        theme_editor::{CodeRow, Part},
    },
    theme_editor::{
        RULE_EDIT,
        codes::{
            CODE_ADD, CODE_EDIT, EXISTS, INVALID_CODE, NEEDS_KEYBOARD, checked_codes, with_recorded,
        },
        codes_paint::{CODE_ADD_ID, CODE_ID, LINK_ID, NO_CODES_ID, PREVIEW_ID, RECORD_ID},
        paint::{DELETE_ID, rule_id},
    },
};
use wire::settings_ui::theme_codes::{effect, kind, report};

/// The rule editor on `key[code=10]`.
fn editing_rule() -> (App, render::text::Text) {
    let (mut a, mut text) = app(SHEET);
    assert!(tap(&mut a, &mut text, rule_id(3, 2)));
    assert_eq!(a.stack.last().unwrap(), RULE_EDIT);
    a.effects.clear();
    (a, text)
}
fn codes(a: &App) -> Vec<String> {
    a.theme_editor.form.as_ref().unwrap().values(0)
}
fn field(a: &App) -> i32 {
    part_id(a, Part::Code(CodeRow::Field))
}
/// What the code field holds.
fn typed(a: &App) -> String {
    a.line_text(field(a)).unwrap()
}
fn save(a: &mut App, text: &mut render::text::Text) {
    let save = lang_id(a, Lang::Save);
    assert!(tap(a, text, save));
}
/// A `THEME_CODE` report of `kind`, with `code` and the rest empty.
fn report_of(kind: &str, code: &str) -> Vec<String> {
    let mut p = vec![String::new(); report::SIZE];
    p[report::KIND] = kind.into();
    p[report::CODE] = code.into();
    p
}
fn described(code: &str, name: &str, mark: bool, icon: &str) -> Vec<String> {
    let mut p = report_of(kind::DESCRIBED, code);
    p[report::NAME] = name.into();
    p[report::NON_SPACING] = wire::flag::encode(mark).into();
    p[report::ICON] = icon.into();
    p
}
fn has(a: &App, packet: &[&str]) -> bool {
    a.effects.iter().any(|e| e == packet)
}

#[test]
fn a_rules_codes_are_chips_that_open_the_editor_and_add_opens_it_empty() {
    let (mut a, mut text) = editing_rule();
    assert_eq!(control(&a, CODE_ID).unwrap().label, "10");
    assert!(control(&a, NO_CODES_ID).is_none());
    assert!(tap(&mut a, &mut text, CODE_ID));
    assert_eq!(a.stack.last().unwrap(), CODE_EDIT);
    assert_eq!(typed(&a), "10");
    // Delete in the top bar names the code; Back keeps the rule editor and what it holds.
    assert_eq!(control(&a, DELETE_ID).unwrap().summary, "10");
    a.back();
    assert_eq!(a.stack.last().unwrap(), RULE_EDIT);
    assert_eq!(codes(&a), ["10"]);
    assert!(a.theme_editor.code.is_none());
    assert!(tap(&mut a, &mut text, CODE_ADD_ID));
    assert_eq!(a.stack.last().unwrap(), CODE_ADD);
    assert_eq!(typed(&a), "");
    let hint = control(&a, field(&a)).unwrap();
    assert_eq!(
        (hint.label.as_str(), hint.role),
        ("settings__theme_editor__code_placeholder", "edit")
    );
    assert!(control(&a, DELETE_ID).is_none());
}

#[test]
fn codes_are_added_edited_and_removed_one_at_a_time() {
    let (mut a, mut text) = editing_rule();
    add_codes(&mut a, &mut text, "13");
    assert_eq!(codes(&a), ["10", "13"]);
    // Edit 13 into -7: the old code goes and the new one comes. The preview asks Kotlin about
    // the code it opens on.
    a.effects.clear();
    assert!(tap(&mut a, &mut text, CODE_ID + 1));
    assert!(has(&a, &[effect::DESCRIBE, "13"]));
    type_into(&mut a, &mut text, Part::Code(CodeRow::Field), "-7");
    save(&mut a, &mut text);
    assert_eq!(a.stack.last().unwrap(), RULE_EDIT);
    assert_eq!(codes(&a), ["-7", "10"]);
    // The top bar's Delete removes the code at once.
    assert!(tap(&mut a, &mut text, CODE_ID + 1));
    assert!(tap(&mut a, &mut text, DELETE_ID));
    assert_eq!(a.stack.last().unwrap(), RULE_EDIT);
    assert_eq!(codes(&a), ["-7"]);
    // Nothing reaches the rules until the rule editor applies.
    assert_eq!(
        String::from_utf16_lossy(&a.theme_editor.rules[3].0.format()),
        "key[code=10]"
    );
    save(&mut a, &mut text);
    assert_eq!(
        String::from_utf16_lossy(&a.theme_editor.rules[3].0.format()),
        "key[code=-7]"
    );
}

#[test]
fn several_codes_go_in_at_once_and_an_unchanged_code_just_closes() {
    let (mut a, mut text) = editing_rule();
    add_codes(&mut a, &mut text, " 32 ,-1,, 97 ");
    assert_eq!(codes(&a), ["-1", "10", "32", "97"]);
    // Editing 10 to "10, 13" keeps 10 (it is the code edited) and adds 13.
    assert!(tap(&mut a, &mut text, CODE_ID + 1));
    type_into(&mut a, &mut text, Part::Code(CodeRow::Field), "10, 13");
    save(&mut a, &mut text);
    assert_eq!(codes(&a), ["-1", "10", "13", "32", "97"]);
    // Applied unchanged, as written differently, the editor closes and nothing changes.
    assert!(tap(&mut a, &mut text, CODE_ID + 2));
    type_into(&mut a, &mut text, Part::Code(CodeRow::Field), "013");
    save(&mut a, &mut text);
    assert_eq!(a.stack.last().unwrap(), RULE_EDIT);
    assert_eq!(codes(&a), ["-1", "10", "13", "32", "97"]);
}

#[test]
fn every_refusal_keeps_the_editor_open_with_why() {
    let (mut a, mut text) = editing_rule();
    a.strings.insert(
        INVALID_CODE.into(),
        "{c_min}-{c_max} or {i_min}-{i_max}".into(),
    );
    add_codes(&mut a, &mut text, "13");
    assert!(tap(&mut a, &mut text, CODE_ADD_ID));
    let ranges = "1-65535 or -9999--1";
    for (typed, why) in [
        ("", ranges),
        (" , ", ranges),
        ("0", ranges),
        ("65536", ranges),
        ("-10000", ranges),
        ("abc", ranges),
        ("97, 1.5", ranges),
        ("10", EXISTS),
        ("97, 13", EXISTS),
    ] {
        type_into(&mut a, &mut text, Part::Code(CodeRow::Field), typed);
        save(&mut a, &mut text);
        assert_eq!(a.stack.last().unwrap(), CODE_ADD, "{typed:?}");
        assert_eq!(control(&a, field(&a)).unwrap().summary, why, "{typed:?}");
    }
    // Editing 13 into 10, which the rule has, is refused too; typing clears the message.
    a.back();
    a.back();
    assert!(tap(&mut a, &mut text, CODE_ID + 1));
    type_into(&mut a, &mut text, Part::Code(CodeRow::Field), "10");
    save(&mut a, &mut text);
    assert_eq!(a.code_message(), EXISTS);
    type_into(&mut a, &mut text, Part::Code(CodeRow::Field), "1");
    assert_eq!(a.code_message(), "");
    assert_eq!(codes(&a), ["10", "13"]);
    assert_eq!(
        checked_codes("1, -9999, 65535", None, &[]).unwrap(),
        ["1", "-9999", "65535"]
    );
}

#[test]
fn recording_needs_this_apps_keyboard_enabled_and_selected() {
    let (mut a, mut text) = editing_rule();
    assert!(tap(&mut a, &mut text, CODE_ADD_ID));
    a.strings
        .insert(NEEDS_KEYBOARD.into(), "Needs the keyboard".into());
    a.set_ime(false, false);
    a.effects.clear();
    assert!(tap(&mut a, &mut text, RECORD_ID));
    assert!(
        has(&a, &["enable"]),
        "not enabled: the system's input method settings"
    );
    assert!(!a.effects.iter().any(|e| e[0] == effect::RECORD));
    assert_eq!(
        control(&a, field(&a)).unwrap().summary,
        "Needs the keyboard"
    );
    a.set_ime(true, false);
    a.effects.clear();
    assert!(tap(&mut a, &mut text, RECORD_ID));
    assert!(has(&a, &["picker"]), "enabled, not selected: the picker");
    assert!(!a.theme_editor.code.as_ref().unwrap().recording);
    // Once the keyboard is selected the message goes, and recording starts.
    a.set_ime(true, true);
    a.scene(&mut text);
    assert_eq!(control(&a, field(&a)).unwrap().summary, "");
    a.effects.clear();
    assert!(tap(&mut a, &mut text, RECORD_ID));
    assert!(has(&a, &[effect::RECORD]));
    assert!(has(&a, &["editor"]), "the keyboard shows for the key");
    assert!(a.editor.focused && a.field == Some((field(&a) - 1000) as usize));
}

#[test]
fn a_recorded_key_sets_the_code_being_typed_and_stops_recording() {
    let (mut a, mut text) = editing_rule();
    a.strings.insert(
        "settings__theme_editor__code_recording_placeholder".into(),
        "Recording…".into(),
    );
    assert!(tap(&mut a, &mut text, CODE_ADD_ID));
    type_into(&mut a, &mut text, Part::Code(CodeRow::Field), "13, 9");
    assert!(tap(&mut a, &mut text, RECORD_ID));
    let record = control(&a, RECORD_ID).unwrap();
    assert_eq!((record.checked, record.role), (Some(true), "button"));
    assert_eq!(record.label, "settings__theme_editor__code_record");
    assert_eq!(control(&a, field(&a)).unwrap().summary, "Recording…");
    a.effects.clear();
    a.theme_code(&report_of(kind::KEY, "-7")).unwrap();
    a.scene(&mut text);
    assert_eq!(typed(&a), "13, -7");
    assert_eq!(
        String::from_utf16_lossy(&a.editor.text),
        "13, -7",
        "the field shows it"
    );
    assert!(!a.theme_editor.code.as_ref().unwrap().recording);
    assert!(!has(&a, &[effect::STOP]), "Kotlin stopped listening itself");
    assert!(has(&a, &[effect::DESCRIBE, "-7"]));
    assert_eq!(control(&a, RECORD_ID).unwrap().checked, Some(false));
    // A key arriving when not recording does nothing.
    a.effects.clear();
    a.theme_code(&report_of(kind::KEY, "97")).unwrap();
    assert_eq!(typed(&a), "13, -7");
    assert!(a.effects.is_empty());
    save(&mut a, &mut text);
    assert_eq!(codes(&a), ["-7", "10", "13"]);
    assert_eq!(with_recorded("", 97), "97");
    assert_eq!(with_recorded("10", 97), "97");
    assert_eq!(with_recorded("10 ,", 97), "10, 97");
}

#[test]
fn recording_stops_when_tapped_again_typed_over_left_or_stopped_by_kotlin() {
    let (mut a, mut text) = editing_rule();
    assert!(tap(&mut a, &mut text, CODE_ADD_ID));
    let recording = |a: &App| a.theme_editor.code.as_ref().unwrap().recording;
    assert!(tap(&mut a, &mut text, RECORD_ID));
    a.effects.clear();
    assert!(tap(&mut a, &mut text, RECORD_ID));
    assert!(!recording(&a) && has(&a, &[effect::STOP]));
    assert!(tap(&mut a, &mut text, RECORD_ID));
    a.effects.clear();
    type_into(&mut a, &mut text, Part::Code(CodeRow::Field), "4");
    assert!(!recording(&a) && has(&a, &[effect::STOP]));
    assert!(tap(&mut a, &mut text, RECORD_ID));
    a.theme_code(&report_of(kind::STOPPED, "")).unwrap();
    assert!(!recording(&a));
    assert!(tap(&mut a, &mut text, RECORD_ID));
    a.effects.clear();
    let cancel = lang_id(&a, Lang::Cancel);
    assert!(tap(&mut a, &mut text, cancel));
    assert_eq!(a.stack.last().unwrap(), RULE_EDIT);
    assert!(has(&a, &[effect::STOP]));
    assert!(a.theme_code(&report_of("other", "")).is_err());
    assert!(a.theme_code(&report_of(kind::KEY, "x")).is_err());
    assert!(a.theme_code(&[kind::KEY.into()]).is_err());
}

#[test]
fn the_preview_of_a_character_with_its_unicode_name() {
    let (mut a, mut text) = editing_rule();
    assert!(tap(&mut a, &mut text, CODE_ADD_ID));
    assert!(control(&a, PREVIEW_ID).unwrap().summary == "unspecified");
    type_into(&mut a, &mut text, Part::Code(CodeRow::Field), "97");
    assert!(has(&a, &[effect::DESCRIBE, "97"]));
    // Before Kotlin answers: the label and the kind, no name.
    assert_eq!(control(&a, PREVIEW_ID).unwrap().label, "a");
    a.theme_code(&described("97", "LATIN SMALL LETTER A", false, ""))
        .unwrap();
    a.scene(&mut text);
    let preview = control(&a, PREVIEW_ID).unwrap();
    assert_eq!(
        (preview.label.as_str(), preview.summary.as_str()),
        ("a, LATIN SMALL LETTER A", "character")
    );
    // The same code written again is not asked about again, and keeps its name meanwhile.
    a.effects.clear();
    type_into(&mut a, &mut text, Part::Code(CodeRow::Field), "097");
    assert!(!a.effects.iter().any(|e| e[0] == effect::DESCRIBE));
    assert_eq!(
        control(&a, PREVIEW_ID).unwrap().label,
        "a, LATIN SMALL LETTER A"
    );
    // A non-spacing mark sits on a dotted circle, as the keyboard labels it.
    type_into(&mut a, &mut text, Part::Code(CodeRow::Field), "769");
    a.theme_code(&described("769", "COMBINING ACUTE ACCENT", true, ""))
        .unwrap();
    a.scene(&mut text);
    assert_eq!(
        control(&a, PREVIEW_ID).unwrap().label,
        "\u{25cc}\u{301}, COMBINING ACUTE ACCENT"
    );
    // An answer about another code than the one shown is dropped.
    a.theme_code(&described("98", "LATIN SMALL LETTER B", false, ""))
        .unwrap();
    assert_eq!(
        a.theme_editor
            .code
            .as_ref()
            .unwrap()
            .said
            .as_ref()
            .unwrap()
            .code,
        769
    );
}

#[test]
fn the_preview_of_internal_codes_with_an_icon_a_label_or_neither() {
    let (mut a, mut text) = editing_rule();
    a.strings.insert("key__view_symbols".into(), "?123".into());
    assert!(tap(&mut a, &mut text, CODE_ADD_ID));
    type_into(&mut a, &mut text, Part::Code(CodeRow::Field), "-7");
    assert!(has(&a, &[effect::DESCRIBE, "-7"]));
    // Kotlin answers with the key's icon token; Rust draws the icon it names.
    a.theme_code(&described("-7", "", false, "-7")).unwrap();
    a.scene(&mut text);
    let preview = control(&a, PREVIEW_ID).unwrap();
    assert_eq!(
        (preview.label.as_str(), preview.summary.as_str()),
        ("delete", "enter_editing")
    );
    let panel = a.scene(&mut text).panel.unwrap();
    assert!(
        panel.items.iter().any(|i| i.icon == icons::key_path(-7)),
        "the key's icon is drawn"
    );
    // A code no key has is the unspecified key, and Kotlin is not asked about it; what it said
    // of the key before, its icon, is not shown on this one.
    a.effects.clear();
    type_into(&mut a, &mut text, Part::Code(CodeRow::Field), "-9000");
    assert!(!a.effects.iter().any(|e| e[0] == effect::DESCRIBE));
    let preview = control(&a, PREVIEW_ID).unwrap();
    assert_eq!(
        (preview.label.as_str(), preview.summary.as_str()),
        ("unspecified", "unspecified")
    );
    let panel = a.scene(&mut text).panel.unwrap();
    assert!(!panel.items.iter().any(|i| i.icon == icons::key_path(-7)));
    // A view key's label is the keyboard's resource.
    type_into(&mut a, &mut text, Part::Code(CodeRow::Field), "-202");
    a.theme_code(&described("-202", "", false, "")).unwrap();
    a.scene(&mut text);
    let preview = control(&a, PREVIEW_ID).unwrap();
    assert_eq!(
        (preview.label.as_str(), preview.summary.as_str()),
        ("?123, view_symbols", "system_gui")
    );
    // An icon, when the keyboard has one for the key, takes its label's place.
    a.theme_code(&described("-202", "", false, "-7")).unwrap();
    a.scene(&mut text);
    assert_eq!(control(&a, PREVIEW_ID).unwrap().label, "view_symbols");
}

#[test]
fn the_help_links_open_their_pages_and_the_focus_order_reaches_every_control() {
    let (mut a, mut text) = editing_rule();
    a.strings.insert(
        "florisboard__internal_key_codes_url".into(),
        "https://example.test/codes".into(),
    );
    assert!(tap(&mut a, &mut text, CODE_ID));
    assert_eq!(
        control(&a, LINK_ID).unwrap().label,
        "Characters (unicode-table.com)"
    );
    assert_eq!(control(&a, LINK_ID + 1).unwrap().role, "link");
    a.effects.clear();
    assert!(tap(&mut a, &mut text, LINK_ID + 1));
    assert!(has(&a, &["url", "https://example.test/codes"]));
    let order = super::props::focus_order(&mut a);
    let at = |id: i32| {
        order
            .iter()
            .position(|&o| o == id)
            .unwrap_or_else(|| panic!("{id} not in {order:?}"))
    };
    let ids = [DELETE_ID, field(&a), RECORD_ID, LINK_ID, LINK_ID + 1];
    assert!(ids.windows(2).all(|w| at(w[0]) < at(w[1])), "{order:?}");
}
