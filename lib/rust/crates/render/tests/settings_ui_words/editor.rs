//! The word editor: where a draft starts, the dialog's checks and messages, the refused
//! duplicate, the entry it builds, Delete, Cancel, and its single-line fields.
use crate::common::*;
use render::settings_ui::{
    catalogue::{
        languages::Lang,
        words::{Act, Part},
    },
    items::{ADD_ID, UNDO_ID, item_id},
    languages::form::DELETE_ID,
    words::{
        FLORIS, FLORIS_ADD, FLORIS_EDIT, FLORIS_WORDS, SYSTEM,
        editor::{Draft, WORD_REGEX, error_key, verdict},
        form::ERROR_ID,
    },
};
use wire::settings_ui::words::{dictionary, effect, outcome};
use wire::utilities::{EMPTY, INVALID};

fn fields(a: &render::settings_ui::App) -> Vec<String> {
    (0..4).map(|k| a.word_field_text(k)).collect()
}
fn errors(a: &render::settings_ui::App) -> Vec<String> {
    (0..4).map(|k| a.word_error(k)).collect()
}
fn save(a: &mut render::settings_ui::App, text: &mut render::text::Text) {
    let id = lang_id(a, Lang::Save);
    assert!(tap(a, text, id));
}
fn edit(id: i64) -> String {
    format!("{FLORIS_EDIT}/{id}")
}
fn en_us() -> String {
    format!("{FLORIS_WORDS}/en_US")
}

#[test]
fn a_new_word_starts_at_255_in_the_language_it_was_added_from() {
    let de = format!("{FLORIS_WORDS}/de_DE");
    let (mut a, mut text) = app(&[FLORIS, &de], &sample());
    assert!(tap(&mut a, &mut text, ADD_ID));
    assert_eq!(a.stack.last().unwrap(), FLORIS_ADD);
    assert_eq!(fields(&a), ["", "255", "", "de_DE"]);
    assert!(a.word_to_delete().is_none() && control(&a, DELETE_ID).is_none());
    assert_eq!(errors(&a), ["", "", "", ""]);
    // From the first screen or all languages, it has none, as the dialog had.
    for from in [vec![FLORIS], vec![FLORIS, FLORIS_WORDS]] {
        let (mut a, mut text) = app(&from, &sample());
        let id = if from.len() == 1 {
            part_id(&a, Part::Action(Act::Add))
        } else {
            ADD_ID
        };
        assert!(tap(&mut a, &mut text, id));
        assert_eq!(fields(&a), ["", "255", "", ""]);
    }
}

#[test]
fn editing_opens_the_stored_word_with_delete_in_the_top_bar() {
    let (mut a, mut text) = app(&[FLORIS, &en_us()], &sample());
    let slot = a.slot("2");
    assert!(tap(&mut a, &mut text, item_id(slot, 0)));
    assert_eq!(a.stack.last().unwrap(), &edit(2));
    assert_eq!(fields(&a), ["Quexor", "120", "qx", "en-us"]);
    let delete = control(&a, DELETE_ID).unwrap();
    assert_eq!(
        (delete.label.as_str(), delete.summary.as_str()),
        ("action__delete", "Quexor")
    );
    // Delete follows Back, before the first field.
    let order = focus_order(&mut a);
    assert_eq!(&order[..3], [1, DELETE_ID, part_id(&a, Part::Field(0))]);
}

#[test]
fn the_rules_are_the_ones_user_dictionary_validation_named() {
    let cases: [(usize, &str, &str); 14] = [
        (0, "", EMPTY),
        (0, "   ", EMPTY),
        (0, "two words", INVALID),
        (0, "semi;colon", INVALID),
        (0, "com,ma", INVALID),
        (0, "zorblat", ""),
        (1, "", EMPTY),
        (1, "many", EMPTY),
        (1, "0", INVALID),
        (1, "256", INVALID),
        (1, "1", ""),
        (1, "255", ""),
        (2, "", ""),
        (2, "q x", INVALID),
    ];
    for (k, value, expected) in cases {
        assert_eq!(verdict(k, value), expected, "field {k}");
    }
    assert_eq!(verdict(3, "any thing;at all"), "", "the locale has no rule");
    assert_eq!(error_key(2, EMPTY), None, "a shortcut may be empty");
    assert_eq!(error_key(3, INVALID), None);
}

#[test]
fn saving_an_invalid_draft_marks_each_field_and_sends_nothing() {
    let (mut a, mut text) = app(&[FLORIS, FLORIS_ADD], &sample());
    type_into(&mut a, &mut text, 0, "two words");
    type_into(&mut a, &mut text, 1, "0");
    type_into(&mut a, &mut text, 2, "a;b");
    assert_eq!(errors(&a), ["", "", "", ""], "no error shows before Save");
    save(&mut a, &mut text);
    assert!(word_effects(&a).is_empty());
    assert_eq!(a.stack.last().unwrap(), FLORIS_ADD);
    let expected = [
        format!("Please enter a word matching {WORD_REGEX}"),
        "settings__udm__dialog__freq_error_invalid".into(),
        "settings__udm__dialog__shortcut_error_invalid".into(),
        String::new(),
    ];
    assert_eq!(errors(&a), expected);
    for (k, message) in expected.iter().enumerate().take(3) {
        assert_eq!(&control(&a, ERROR_ID + k as i32).unwrap().label, message);
        let field = control(&a, part_id(&a, Part::Field(k))).unwrap();
        assert_eq!(&field.summary, message);
    }
    // Errors follow the typing from now on.
    type_into(&mut a, &mut text, 0, "");
    type_into(&mut a, &mut text, 1, "");
    assert_eq!(a.word_error(0), "settings__udm__dialog__word_error_empty");
    assert_eq!(a.word_error(1), "settings__udm__dialog__freq_error_empty");
    type_into(&mut a, &mut text, 2, "ok");
    assert!(control(&a, ERROR_ID + 2).is_none());
}

#[test]
fn save_adds_the_entry_the_dialog_built_and_closes() {
    let (mut a, mut text) = app(&[FLORIS, &en_us(), FLORIS_ADD], &sample());
    type_into(&mut a, &mut text, 0, "  blorp ");
    type_into(&mut a, &mut text, 2, "   ");
    type_into(&mut a, &mut text, 3, " de-de ");
    save(&mut a, &mut text);
    let entry = word(0, "blorp", 255, Some("de-de"), None);
    assert!(word_effects(&a).contains(&packet(effect::ADD, dictionary::FLORIS, &[entry])));
    assert_eq!(a.stack.last().unwrap(), &en_us());
    assert!(a.words.draft.is_none());
    let draft = |f: [&str; 4]| Draft {
        fields: f.map(String::from),
        stored: None,
        checked: false,
    };
    // A blank locale is none; the frequency is read trimmed, where the dialog's toInt threw.
    assert_eq!(
        draft(["w", " 7 ", "s ", " "]).entry(),
        Some(word(0, "w", 7, None, Some("s")))
    );
    assert_eq!(draft(["w", "x", "", ""]).entry(), None);
}

#[test]
fn a_word_already_in_its_language_is_refused_but_its_own_edit_is_not() {
    let (mut a, mut text) = app(&[FLORIS, FLORIS_ADD], &sample());
    a.strings
        .insert("settings__udm__word_error_exists".into(), "Exists".into());
    type_into(&mut a, &mut text, 0, "zorblat");
    for (locale, exists) in [("en-US", true), ("de_DE", false), ("en", false)] {
        type_into(&mut a, &mut text, 3, locale);
        a.words.draft.as_mut().unwrap().checked = true;
        assert_eq!(a.word_error(0) == "Exists", exists, "{locale}");
    }
    // `all` and nothing are both all languages in the Floris dictionary.
    type_into(&mut a, &mut text, 0, "blivet");
    for locale in ["", "all", "null"] {
        type_into(&mut a, &mut text, 3, locale);
        assert_eq!(a.word_error(0), "Exists", "{locale}");
    }
    save(&mut a, &mut text);
    assert!(word_effects(&a).is_empty());
    // The system's `all` is a locale of its own.
    let (mut a, mut text) = app(&[SYSTEM, &format!("{SYSTEM}/word/add")], &sample());
    type_into(&mut a, &mut text, 0, "snarf");
    a.words.draft.as_mut().unwrap().checked = true;
    assert!(a.word_error(0).is_empty());
    // Saving a word unchanged changes it in place.
    let (mut a, mut text) = app(&[FLORIS, &en_us(), &edit(2)], &sample());
    save(&mut a, &mut text);
    let quexor = sample()[1].clone();
    assert!(word_effects(&a).contains(&packet(effect::CHANGE, dictionary::FLORIS, &[quexor])));
}

#[test]
fn apply_changes_the_stored_word_and_delete_deletes_it_with_undo() {
    let (mut a, mut text) = app(&[FLORIS, &en_us(), &edit(2)], &sample());
    type_into(&mut a, &mut text, 1, "7");
    save(&mut a, &mut text);
    let changed = word(2, "Quexor", 7, Some("en-us"), Some("qx"));
    assert_eq!(
        word_effects(&a),
        [packet(effect::CHANGE, dictionary::FLORIS, &[changed])]
    );
    assert_eq!(a.stack.last().unwrap(), &en_us());
    a.effects.clear();
    let slot = a.slot("1");
    assert!(tap(&mut a, &mut text, item_id(slot, 0)));
    assert!(tap(&mut a, &mut text, DELETE_ID));
    assert_eq!(
        word_effects(&a),
        [packet(
            effect::DELETE,
            dictionary::FLORIS,
            &[sample()[0].clone()]
        )]
    );
    assert_eq!(a.stack.last().unwrap(), &en_us());
    assert_eq!(a.focus, UNDO_ID);
    assert!(control(&a, UNDO_ID).is_some());
}

#[test]
fn cancel_closes_without_a_change_and_hides_the_keyboard() {
    let (mut a, mut text) = app(&[FLORIS, &en_us(), &edit(1)], &sample());
    type_into(&mut a, &mut text, 0, "changed");
    assert!(a.editor.focused);
    a.effects.clear();
    let id = lang_id(&a, Lang::Cancel);
    assert!(tap(&mut a, &mut text, id));
    assert!(a.effects.contains(&vec!["hideEditor".to_string()]));
    assert!(word_effects(&a).iter().all(|e| e[0] == effect::LOAD));
    assert_eq!(a.stack.last().unwrap(), &en_us());
    assert!(a.words.draft.is_none());
}

#[test]
fn the_fields_are_single_lines_and_the_frequency_asks_for_numbers() {
    let (mut a, mut text) = app(&[FLORIS, FLORIS_ADD], &sample());
    for k in 0..4 {
        assert!(a.line_field(part_id(&a, Part::Field(k))));
    }
    let title = control(&a, part_id(&a, Part::Field(1))).unwrap().label;
    assert_eq!(title, "Frequency (between 1 and 255)");
    type_into(&mut a, &mut text, 1, "12");
    assert_eq!(
        (a.number_field(), a.multi_line_field()),
        (Some("done"), false)
    );
    type_into(&mut a, &mut text, 0, "line\nbreak\r");
    assert_eq!(a.word_field_text(0), "linebreak");
    assert_eq!((a.number_field(), a.multi_line_field()), (None, false));
    a.effects.clear();
    assert!(a.field_key(66, 0), "Enter is Done");
    assert!(!a.editor.focused);
    assert!(a.effects.contains(&vec!["hideEditor".to_string()]));
}

#[test]
fn an_editor_reopened_before_its_word_is_reported_fills_in_when_it_comes() {
    let (mut a, mut text) = app(&[FLORIS], &[]);
    a.navigate(&edit(4));
    assert!(a.words.draft.is_none());
    a.set_words(&report(dictionary::FLORIS, true, outcome::NONE, &sample()))
        .unwrap();
    a.scene(&mut text);
    assert_eq!(fields(&a), ["frobnic", "90", "fb", "de_DE"]);
    // A later report leaves the draft as typed.
    type_into(&mut a, &mut text, 0, "typed");
    a.set_words(&report(dictionary::FLORIS, true, outcome::NONE, &sample()))
        .unwrap();
    assert_eq!(a.word_field_text(0), "typed");
}
