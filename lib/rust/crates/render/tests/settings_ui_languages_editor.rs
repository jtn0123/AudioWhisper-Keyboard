//! The subtype editor (stage4-languages.md): adding from a preset, editing one field, the checks
//! and messages SubtypeEditorScreen had, Select language's answer, Delete, and what is written.
mod languages_common;
use languages_common::*;
use render::settings_ui::{
    App,
    catalogue::{
        Kind,
        languages::{Field, Lang},
    },
    items::UNDO_ID,
    languages::{
        ADD_ROUTE, EDIT_ROUTE, LIST_ROUTE, PICKER_ROUTE,
        editor::{self, Draft},
        form::{ALL_ID, DELETE_ID, SUGGESTION_ID},
        picker::LOCALE_ID,
    },
};
use settings::codec;

/// The index of the first row whose kind matches.
fn row_where(a: &App, pick: impl Fn(&Kind) -> bool) -> usize {
    a.rows().iter().position(|r| pick(&r.kind)).unwrap()
}
fn pick_row(a: &App, f: Field) -> i32 {
    1000 + row_where(a, |k| matches!(k, Kind::Pick(g) if *g == f)) as i32
}
fn lang_row(a: &App, l: Lang) -> i32 {
    1000 + row_where(a, |k| matches!(k, Kind::Lang(m) if *m == l)) as i32
}
fn modal_kind(a: &App) -> Option<Kind> {
    a.modal.as_ref().map(|m| a.rows()[m.row].kind)
}
fn draft(a: &App) -> &Draft {
    a.draft.as_ref().unwrap()
}
/// `value` with this test's stamp for an added subtype's id.
fn added(value: &str) -> String {
    value.replacen(&value[6..value.find(',').unwrap()], &NOW.to_string(), 1)
}

#[test]
fn adding_opens_the_presets_and_one_fills_every_field_then_saves_as_add_did() {
    let (mut a, mut text) = app(ADD_ROUTE, &list(&[english()]));
    assert!(matches!(modal_kind(&a), Some(Kind::Lang(Lang::AllPresets))));
    // The presets in the keyboard's order, named by language and characters layout.
    let labels: Vec<_> = (0..3)
        .map(|j| control(&a, 3000 + j).unwrap().label)
        .collect();
    assert_eq!(
        labels,
        [
            "English (United States)",
            "German (Germany)",
            "French (France)"
        ]
    );
    assert_eq!(control(&a, 3001).unwrap().summary, "QWERTZ");
    assert_eq!(control(&a, 3001).unwrap().role, "button");
    assert!(tap(&mut a, &mut text, 3001));
    assert!(a.modal.is_none());
    assert_eq!(
        draft(&a).get(Field::Layout(0)),
        "org.florisboard.layouts:qwertz"
    );
    assert!({
        let id = lang_row(&a, Lang::Save);
        tap(&mut a, &mut text, id)
    });
    assert_eq!(written(&a).unwrap(), list(&[english(), added(&german())]));
    assert_eq!(a.stack.last().unwrap(), LIST_ROUTE);
}

#[test]
fn the_suggested_presets_are_the_system_languages_and_show_all_lists_every_preset() {
    let (mut a, mut text) = app(ADD_ROUTE, "[]");
    a.back();
    a.scene(&mut text);
    // The system is English (US) then German, and both have presets.
    assert_eq!(
        control(&a, SUGGESTION_ID).unwrap().label,
        "English (United States)"
    );
    assert_eq!(
        control(&a, SUGGESTION_ID + 1).unwrap().label,
        "German (Germany)"
    );
    assert!(control(&a, SUGGESTION_ID + 2).is_none());
    assert!(tap(&mut a, &mut text, ALL_ID));
    assert!(matches!(modal_kind(&a), Some(Kind::Lang(Lang::AllPresets))));
    a.back();
    assert!(tap(&mut a, &mut text, SUGGESTION_ID + 1));
    assert_eq!(
        draft(&a).value,
        codec::decode(
            "subtype",
            &settings::text(&german().replacen('2', "-1", 1)),
            false
        )
        .unwrap()
    );
}

#[test]
fn saving_with_a_field_unset_explains_and_marks_the_fields_and_writes_nothing() {
    let (mut a, mut text) = app(ADD_ROUTE, "[]");
    a.back();
    assert!({
        let id = lang_row(&a, Lang::Save);
        tap(&mut a, &mut text, id)
    });
    assert!(matches!(modal_kind(&a), Some(Kind::Lang(Lang::Message))));
    assert_eq!(
        a.modal.as_ref().unwrap().selected,
        "settings__localization__subtype_error_fields_no_value"
    );
    assert!(draft(&a).checked);
    assert_eq!(written(&a), None);
    // OK closes the message and leaves the editor as it was.
    assert!(tap(&mut a, &mut text, 7));
    assert!(a.modal.is_none());
    assert_eq!(a.stack.last().unwrap(), ADD_ROUTE);
    // An unset field shows "‒ select ‒".
    a.strings.insert(
        "settings__localization__subtype_select_placeholder".into(),
        "- select -".into(),
    );
    assert_eq!(a.pick_text(Field::Composer), "- select -");
}

#[test]
fn adding_a_subtype_that_exists_says_so() {
    let (mut a, mut text) = app(ADD_ROUTE, &list(&[english()]));
    assert!(tap(&mut a, &mut text, 3000));
    assert!({
        let id = lang_row(&a, Lang::Save);
        tap(&mut a, &mut text, id)
    });
    assert_eq!(
        a.modal.as_ref().unwrap().selected,
        "settings__localization__subtype_error_already_exists"
    );
    assert_eq!(written(&a), None);
}

#[test]
fn editing_changes_one_field_and_writes_the_list_as_modify_did() {
    let (mut a, mut text) = app(&format!("{EDIT_ROUTE}/2"), &list(&[english(), german()]));
    assert!(a.modal.is_none());
    assert_eq!(a.pick_text(Field::Layout(0)), "QWERTZ");
    assert!({
        let id = pick_row(&a, Field::Layout(0));
        tap(&mut a, &mut text, id)
    });
    assert!(matches!(modal_kind(&a), Some(Kind::Pick(Field::Layout(0)))));
    // The characters layouts only, the current one ticked, as JetPrefDropdown listed them.
    let choices: Vec<_> = (0..3).map(|j| control(&a, 3000 + j).unwrap()).collect();
    assert_eq!(
        choices.iter().map(|c| c.label.as_str()).collect::<Vec<_>>(),
        ["QWERTY", "QWERTZ", "AZERTY"]
    );
    assert_eq!(choices[1].checked, Some(true));
    assert!(control(&a, 3003).is_none());
    assert!(tap(&mut a, &mut text, 3000));
    assert!({
        let id = lang_row(&a, Lang::Save);
        tap(&mut a, &mut text, id)
    });
    let changed = stored(2, "de-DE", "de", "qwerty", "euro");
    assert_eq!(written(&a).unwrap(), list(&[english(), changed]));
    assert_eq!(a.stack.last().unwrap(), LIST_ROUTE);
}

#[test]
fn cancel_and_back_leave_the_list_as_it_was() {
    for close in [true, false] {
        let (mut a, mut text) = app(&format!("{EDIT_ROUTE}/1"), &list(&[english()]));
        assert!({
            let id = pick_row(&a, Field::Currency);
            tap(&mut a, &mut text, id)
        });
        assert!(tap(&mut a, &mut text, 3001));
        assert_eq!(
            draft(&a).get(Field::Currency),
            "org.florisboard.currencysets:euro"
        );
        if close {
            assert!({
                let id = lang_row(&a, Lang::Cancel);
                tap(&mut a, &mut text, id)
            });
        } else {
            a.back();
        }
        assert_eq!(a.stack.last().unwrap(), LIST_ROUTE);
        assert_eq!(written(&a), None);
        assert!(a.draft.is_none());
    }
}

#[test]
fn a_picked_language_sets_its_popup() {
    let (mut a, mut text) = app(ADD_ROUTE, "[]");
    a.back();
    assert!({
        let id = pick_row(&a, Field::Locale);
        tap(&mut a, &mut text, id)
    });
    assert_eq!(a.stack.last().unwrap(), PICKER_ROUTE);
    let fr = (0..6)
        .map(|k| LOCALE_ID + k)
        .find(|&id| control(&a, id).unwrap().label == "French (France)")
        .unwrap();
    assert!(tap(&mut a, &mut text, fr));
    assert_eq!(a.stack.last().unwrap(), ADD_ROUTE);
    let d = draft(&a).clone();
    assert_eq!(d.get(Field::Locale), "fr-FR");
    assert_eq!(d.get(Field::Popup), "org.florisboard.localization:fr");
    assert!(d.unset(Field::Layout(0)));
}

#[test]
fn editing_a_language_changes_only_it_and_its_popup() {
    let (mut a, mut text) = app(&format!("{EDIT_ROUTE}/2"), &list(&[german()]));
    assert!({
        let id = pick_row(&a, Field::Locale);
        tap(&mut a, &mut text, id)
    });
    a.languages.search = "spanish".into();
    a.scene(&mut text);
    // Spanish has no preset: the popup mapping falls back to the default one.
    assert_eq!(control(&a, LOCALE_ID).unwrap().label, "Spanish (Spain)");
    assert!(tap(&mut a, &mut text, LOCALE_ID));
    let d = draft(&a);
    assert_eq!(d.get(Field::Locale), "es-ES");
    assert_eq!(d.get(Field::Popup), editor::DEFAULT_POPUP);
    assert_eq!(d.get(Field::Layout(0)), "org.florisboard.layouts:qwertz");
    // A language with a preset takes the preset's popup mapping and nothing else.
    assert!({
        let id = pick_row(&a, Field::Locale);
        tap(&mut a, &mut text, id)
    });
    a.languages.search = "fr-FR".into();
    a.scene(&mut text);
    assert!(tap(&mut a, &mut text, LOCALE_ID));
    assert_eq!(
        draft(&a).get(Field::Popup),
        "org.florisboard.localization:fr"
    );
    assert_eq!(
        draft(&a).get(Field::Layout(0)),
        "org.florisboard.layouts:qwertz"
    );
}

#[test]
fn delete_in_the_top_bar_removes_the_subtype_and_the_list_offers_undo() {
    let (mut a, mut text) = app(&format!("{EDIT_ROUTE}/2"), &list(&[english(), german()]));
    let delete = control(&a, DELETE_ID).unwrap();
    assert_eq!(
        (delete.label.as_str(), delete.summary.as_str()),
        ("action__delete", "German (Germany)")
    );
    assert!(tap(&mut a, &mut text, DELETE_ID));
    assert_eq!(a.stack.last().unwrap(), LIST_ROUTE);
    assert_eq!(written(&a).unwrap(), list(&[english()]));
    assert!(control(&a, UNDO_ID).is_some());
    a.effects.clear();
    assert!(tap(&mut a, &mut text, UNDO_ID));
    assert_eq!(written(&a).unwrap(), list(&[english(), german()]));
    // Adding has nothing to delete.
    let (a, _) = app(ADD_ROUTE, "[]");
    assert!(control(&a, DELETE_ID).is_none());
}

#[test]
fn a_draft_is_valid_exactly_when_subtype_editor_state_would_save_it() {
    let full = Draft {
        value: codec::decode("subtype", &settings::text(&german()), false).unwrap(),
        adding: false,
        id: 2,
        checked: false,
    };
    assert!(full.valid());
    assert!(
        !Draft {
            value: editor::blank(),
            ..full.clone()
        }
        .valid()
    );
    // Each field alone left at "‒ select ‒" fails the check.
    let mut fields = editor::fields();
    fields.push(Field::Locale);
    for f in fields {
        let mut d = full.clone();
        d.set(
            f,
            if f == Field::Locale {
                editor::LOCALE_UNSET
            } else {
                editor::UNSET
            },
        );
        assert!(d.unset(f));
        assert!(!d.valid(), "{f:?}");
    }
    // The punctuation rule and the spelling engine cannot be picked, but are checked too.
    for (path, inner) in [("punctuationRule", ""), ("nlpProviders", "spelling")] {
        let mut d = full.clone();
        if inner.is_empty() {
            d.value
                .set(path, settings::Value::string(editor::UNSET))
                .unwrap();
        } else {
            let mut v = d.value.get(path).unwrap().clone();
            v.set(inner, settings::Value::string(editor::UNSET))
                .unwrap();
            d.value.set(path, v).unwrap();
        }
        assert!(!d.valid(), "{path}");
    }
    // Only the exact placeholders count: a locale "00" or a component "00:x" is a value.
    let mut d = full.clone();
    d.set(Field::Locale, "00");
    d.set(Field::Composer, "00:valid");
    assert!(d.valid());
}

#[test]
fn the_focus_order_follows_the_fields_and_ends_on_the_bottom_bar() {
    let (mut a, mut text) = app(&format!("{EDIT_ROUTE}/1"), &list(&[english()]));
    let mut seen = vec![];
    a.focus = -1;
    for _ in 0..22 {
        a.focus_next(false);
        a.scene(&mut text);
        seen.push(a.focus);
    }
    let mut want = vec![1, DELETE_ID, pick_row(&a, Field::Locale)];
    want.extend([Field::Popup, Field::Layout(0), Field::Provider].map(|f| pick_row(&a, f)));
    assert_eq!(&seen[..want.len()], &want);
    let save = lang_row(&a, Lang::Save);
    assert!(seen.contains(&lang_row(&a, Lang::Cancel)) && seen.contains(&save));
}
