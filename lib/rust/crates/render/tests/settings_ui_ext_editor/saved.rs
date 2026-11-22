//! The editor across a recreated activity, a turned screen or a process Android ended: the
//! draft, its changes and an open sheet come back, and Kotlin is asked to take the workspace up
//! again with the draft as it stood. Compose kept the workspace in memory and lost it.
use super::common::*;

const THEMES: &str = "ext/list/EXT_THEME";
const LOAD: &str = wire::settings_ui::extensions::effect::LOAD;

/// A fresh app given `a`'s stack, scrolls, dialog, file work and the editor's saved part, as
/// `render_settings_state::restore` does.
fn recreated(a: &App) -> App {
    let mut b = App {
        config: a.config,
        strings: a.strings.clone(),
        ..App::default()
    };
    b.stack = a.stack.clone();
    b.scrolls = a.scrolls.clone();
    b.modal = a.modal.clone();
    b.files = a.files.clone();
    b.restore_ext_editor(&a.ext_editor_saved()).unwrap();
    b.extensions_restored();
    b
}
/// Kotlin took the workspace up again: Rust's copy is the draft, the theme editor saved nothing.
/// Kotlin keeps the copy the `OPEN` effect handed it, so nothing is sent back.
fn taken_up(b: &mut App, text: &mut Text) {
    let manifest = b.extensions.editor.draft.raw_json();
    b.set_ext_editor(&ready(MINE, &manifest, false)).unwrap();
    assert!(
        last(b, effect::SYNC).is_none(),
        "Kotlin kept the copy it was sent"
    );
    b.scene(text);
    b.effects.clear();
}
/// The editor with its first component deleted.
fn changed() -> (App, Text) {
    let (mut a, mut text) = editing();
    assert!(tap(&mut a, &mut text, COMP_ID + 3));
    a.effects.clear();
    (a, text)
}

#[test]
fn the_draft_and_its_changes_survive_recreation() {
    let (a, mut text) = changed();
    let mut b = recreated(&a);
    let open = last(&b, effect::OPEN).expect("Kotlin is asked to take the workspace up");
    let draft = a.extensions.editor.draft.raw_json();
    assert_eq!(open, [effect::OPEN, MINE, WS, &draft]);
    b.scene(&mut text);
    assert_eq!(control(&b, STATUS_ID).label, "Opening the extension…");
    taken_up(&mut b, &mut text);
    assert_eq!(theme_ids(&b), ["night"]);
    assert!(b.extensions.editor.changed);
    b.back();
    assert!(b.unsaved_open(), "the change is still one to lose");
}

#[test]
fn an_open_sheet_and_what_it_holds_survive_recreation() {
    // The meta data, typed and not applied.
    let (mut a, mut text) = editing();
    assert!(tap(&mut a, &mut text, MANAGE_ID));
    type_into(&mut a, &mut text, Edit::Meta(1), "2.0.0");
    let mut b = recreated(&a);
    taken_up(&mut b, &mut text);
    assert_eq!(top(&b), META);
    assert_eq!(
        b.extensions.editor.meta_form.as_ref().unwrap().fields[1],
        "2.0.0"
    );
    assert!(confirm(&mut b, &mut text));
    assert_eq!(b.extensions.editor.draft.meta.version, "2.0.0");

    // An empty component, half typed.
    let (mut a, mut text) = editing();
    assert!(tap(&mut a, &mut text, ADD_ID));
    assert!(tap(&mut a, &mut text, FROM_ID + 1));
    type_into(&mut a, &mut text, Edit::New(0), "dusk");
    let mut b = recreated(&a);
    taken_up(&mut b, &mut text);
    assert_eq!(top(&b), COMPONENT_ADD);
    let form = b.extensions.editor.comp_form.clone().unwrap();
    assert!(form.empty);
    assert_eq!(form.fields[0], "dusk");

    // A file's properties, its new name typed.
    let (mut a, mut text) = editing();
    assert!(tap(&mut a, &mut text, MANAGE_ID + 2));
    assert!(tap(&mut a, &mut text, FOLDER_ID + 10));
    type_into(&mut a, &mut text, Edit::FileName, "b.ttf");
    let mut b = recreated(&a);
    taken_up(&mut b, &mut text);
    assert_eq!(top(&b), FILE);
    assert!(has(&b, FILE_DELETE_ID));
    assert!(confirm(&mut b, &mut text));
    let [job, ws, _, from, to, _] = work(&b).unwrap();
    assert_eq!(
        [job, ws, from, to],
        [job::EXT_FILE_RENAME, WS, "fonts/a.ttf", "fonts/b.ttf"]
    );
}

#[test]
fn the_unsaved_question_survives_recreation_and_discard_closes_the_workspace() {
    let (mut a, mut text) = changed();
    a.back();
    let mut b = recreated(&a);
    b.scene(&mut text);
    assert!(b.unsaved_open());
    b.effects.clear();
    assert!(tap(&mut b, &mut text, 6));
    assert_eq!(top(&b), "settings/home");
    assert_eq!(
        last(&b, effect::CLOSE),
        Some(vec![effect::CLOSE.into(), WS.into()])
    );
}

/// The theme editor saved a component while it was over the editor: Kotlin's copy, which it
/// changed, becomes the draft, whether the editor was recreated in between or not.
#[test]
fn a_component_the_theme_editor_saved_becomes_the_draft() {
    let themed = manifest_with("\"label\": \"Night\"", "\"label\": \"Night 2\"");
    let (mut a, _) = editing();
    a.set_ext_editor(&ready(MINE, &themed, true)).unwrap();
    assert_eq!(a.extensions.editor.draft.themes[1].label, "Night 2");
    assert!(a.extensions.editor.changed);

    let (a, _) = editing();
    let mut b = recreated(&a);
    b.set_ext_editor(&ready(MINE, &themed, true)).unwrap();
    assert_eq!(b.extensions.editor.draft.themes[1].label, "Night 2");
    assert!(b.extensions.editor.changed);
    assert!(
        last(&b, effect::SYNC).is_none(),
        "the theme editor's copy is Kotlin's already"
    );
    // A report the theme editor did not change keeps the draft Rust has.
    b.set_ext_editor(&ready(MINE, MANIFEST, false)).unwrap();
    assert_eq!(b.extensions.editor.draft.themes[1].label, "Night 2");
}

#[test]
fn a_workspace_gone_after_recreation_says_so() {
    let (a, mut text) = changed();
    let mut b = recreated(&a);
    b.set_ext_editor(&head(MINE, outcome::GONE, "", false))
        .unwrap();
    b.scene(&mut text);
    assert_eq!(b.extensions.editor.load, Load::Gone);
    assert_eq!(control(&b, STATUS_ID).label, "Its draft is gone.");
    b.effects.clear();
    b.back();
    assert_eq!(top(&b), "settings/home");
    assert!(b.effects.is_empty(), "nothing left to close");
}

/// Fix: a list under the editor asked for the extension index only when it was opened, and a
/// recreated activity asked only for the top screen's; Back asks for nothing, so leaving the
/// restored editor showed the theme list empty.
#[test]
fn a_list_under_a_restored_editor_has_the_index_after_back() {
    let (mut a, mut text) = fresh();
    a.navigate(THEMES);
    a.navigate(EDIT);
    a.set_ext_editor(&ready(MINE, MANIFEST, false)).unwrap();
    a.scene(&mut text);
    let b = recreated(&a);
    assert!(b.extensions.requested, "the index is asked for");
    assert!(b.effects.iter().any(|e| e[0] == LOAD));
    // With no list on the stack, nothing more is asked than the workspace.
    let (a, _) = editing();
    let b = recreated(&a);
    assert!(!b.extensions.requested);
    assert!(b.effects.iter().all(|e| e[0] != LOAD));
}

#[test]
fn an_editor_whose_workspace_had_not_opened_asks_for_a_new_one() {
    let (a, _) = asked(EDIT, MINE);
    assert_eq!(a.ext_editor_saved(), "");
    let b = recreated(&a);
    let open = last(&b, effect::OPEN).unwrap();
    assert_eq!(open, [effect::OPEN, MINE, "", ""]);
}

/// Improvement: a Save that finished while the activity was recreated reports before Kotlin
/// answers the restored editor; it closes the editor on the saved extension, where it would
/// have been told its draft was gone.
#[test]
fn a_save_that_ended_during_recreation_closes_the_editor() {
    let (mut a, mut text) = changed();
    assert!(confirm(&mut a, &mut text));
    let mut b = recreated(&a);
    assert_eq!(b.files.busy.as_deref(), Some(job::EXT_SAVE));
    b.file_report(&done_report(job::EXT_SAVE, result::OK, WS, ""))
        .unwrap();
    assert_eq!(top(&b), "settings/home");
    b.effects.clear();
    // Kotlin's answer to the restored editor finds the workspace closed; it changes nothing.
    b.set_ext_editor(&head(MINE, outcome::GONE, "", false))
        .unwrap();
    assert!(b.effects.is_empty());
    assert_eq!(top(&b), "settings/home");
}

#[test]
fn a_broken_saved_part_is_refused() {
    let (a, _) = editing();
    let good = a.ext_editor_saved();
    let mut b = App::default();
    assert!(b.restore_ext_editor("{").is_err());
    let elsewhere = good.replace(EDIT, "ext/view/my.themes");
    assert!(
        b.restore_ext_editor(&elsewhere).is_err(),
        "not an editor route"
    );
    let closed = good.replace(&format!("\"{WS}\""), "\"\"");
    assert!(b.restore_ext_editor(&closed).is_err(), "no workspace");
    assert!(b.restore_ext_editor("").is_ok());
    assert_eq!(b.extensions.editor.load, Load::Waiting);
    assert!(b.restore_ext_editor(&good).is_ok());
    assert_eq!(b.extensions.editor.workspace, WS);
}
