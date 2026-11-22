//! Leaving the editor and saving it: Back, Up and Cancel, the unsaved changes question, and Save
//! with every way it can end.
use super::common::*;

const HOME: &str = "settings/home";

/// The editor with its first component deleted: a change to lose.
fn changed() -> (App, Text) {
    let (mut a, mut text) = editing();
    assert!(tap(&mut a, &mut text, COMP_ID + 3));
    a.effects.clear();
    (a, text)
}
/// Save tapped: the work packet it sent.
fn save(a: &mut App, text: &mut Text) -> [String; 6] {
    assert!(confirm(a, text));
    work(a).expect("Save asks Kotlin to write the archive")
}
fn saves(a: &App) -> usize {
    a.effects.iter().filter(|e| e[0] == "work").count()
}

/// Improvement: leaving the editor closes its workspace however it was left, where Compose left
/// an unsaved one in the cache.
#[test]
fn back_up_and_cancel_without_changes_leave_and_close_the_workspace() {
    for how in ["back", "up", "cancel"] {
        let (mut a, mut text) = editing();
        match how {
            "back" => a.back(),
            "up" => assert!(tap(&mut a, &mut text, 1)),
            _ => assert!(cancel(&mut a, &mut text)),
        }
        assert_eq!(top(&a), HOME, "{how}");
        assert_eq!(
            last(&a, effect::CLOSE),
            Some(vec![effect::CLOSE.into(), WS.into()])
        );
        assert_eq!(
            a.extensions.editor.load,
            Load::Waiting,
            "{how}: the state is gone"
        );
        assert!(a.extensions.editor.workspace.is_empty(), "{how}");
    }
}

#[test]
fn back_up_and_cancel_with_changes_ask_first() {
    for how in ["back", "up", "cancel"] {
        let (mut a, mut text) = changed();
        match how {
            "back" => a.back(),
            "up" => assert!(tap(&mut a, &mut text, 1)),
            _ => assert!(cancel(&mut a, &mut text)),
        }
        a.scene(&mut text);
        assert!(a.unsaved_open(), "{how}");
        assert_eq!(top(&a), EDIT, "{how}");
        assert!(last(&a, effect::CLOSE).is_none(), "{how}");
        let buttons = [5, 6, 7].map(|id| control(&a, id).label);
        assert_eq!(buttons, ["Cancel", "Discard", "Save"], "{how}");
    }
}

#[test]
fn the_questions_cancel_and_back_stay_with_the_draft() {
    let (mut a, mut text) = changed();
    a.back();
    assert!(tap(&mut a, &mut text, 5));
    assert!(!a.unsaved_open());
    assert_eq!(top(&a), EDIT);
    assert!(a.extensions.editor.changed);
    a.back();
    a.back();
    assert!(!a.unsaved_open(), "Back closes the question");
    assert_eq!(theme_ids(&a), ["night"]);
    assert!(
        a.effects
            .iter()
            .all(|e| e[0] != effect::CLOSE && e[0] != "work")
    );
}

#[test]
fn discard_leaves_without_saving_and_closes_the_workspace() {
    let (mut a, mut text) = changed();
    a.back();
    assert!(tap(&mut a, &mut text, 6));
    assert_eq!(top(&a), HOME);
    assert_eq!(a.effects, [[effect::CLOSE, WS]]);
    assert_eq!(saves(&a), 0);
}

#[test]
fn the_questions_save_saves() {
    let (mut a, mut text) = changed();
    a.back();
    assert!(tap(&mut a, &mut text, 7));
    let [job, ws, ..] = work(&a).expect("the question's Save writes the archive");
    assert_eq!([job.as_str(), ws.as_str()], [job::EXT_SAVE, WS]);
}

#[test]
fn save_hands_kotlin_the_manifest_to_write_over_the_installed_archive() {
    let (mut a, mut text) = changed();
    let [job, ws, document, manifest, name, target] = save(&mut a, &mut text);
    assert_eq!([job, ws, document], [job::EXT_SAVE, WS, ""]);
    assert_eq!(manifest, a.extensions.editor.draft.json().unwrap());
    assert!(!manifest.contains("\"day\"") && manifest.contains("\"night\""));
    assert_eq!([name, target], ["my.themes.flex", MINE]);
    assert_eq!(a.files.busy.as_deref(), Some(job::EXT_SAVE));
    assert_eq!(top(&a), EDIT, "the editor waits for the archive");
}

#[test]
fn a_saved_extension_closes_the_editor() {
    let (mut a, mut text) = changed();
    save(&mut a, &mut text);
    a.effects.clear();
    a.file_report(&done_report(job::EXT_SAVE, result::OK, WS, ""))
        .unwrap();
    assert_eq!(top(&a), HOME);
    assert!(a.files.busy.is_none());
    // Kotlin closed the workspace once the archive was in place: Rust does not close it again.
    assert!(last(&a, effect::CLOSE).is_none());
    assert!(a.extensions.editor.workspace.is_empty());
}

/// Improvement: a second Save while the first ran wrote the archive twice in Compose; here it
/// does nothing, and a report after the editor closed changes nothing.
#[test]
fn a_second_save_never_runs_twice_or_crashes() {
    let (mut a, mut text) = changed();
    save(&mut a, &mut text);
    assert!(confirm(&mut a, &mut text));
    a.back();
    assert_eq!(saves(&a), 1);
    let ok = done_report(job::EXT_SAVE, result::OK, WS, "");
    a.file_report(&ok).unwrap();
    a.file_report(&ok).unwrap();
    assert_eq!(top(&a), HOME);
    assert_eq!(saves(&a), 1);
    assert!(a.files.busy.is_none());
}

/// Improvement: Back while Save ran closed the workspace under it; Back now waits for Save.
#[test]
fn back_and_up_wait_while_save_runs() {
    let (mut a, mut text) = changed();
    save(&mut a, &mut text);
    a.effects.clear();
    a.back();
    assert!(tap(&mut a, &mut text, 1));
    assert_eq!(top(&a), EDIT);
    assert!(!a.unsaved_open(), "nothing to ask while it saves");
    assert!(a.effects.is_empty(), "{:?}", a.effects);
}

#[test]
fn invalid_meta_data_is_explained_before_saving() {
    let blank = manifest_with("\"title\": \"My themes\"", "\"title\": \"\"");
    let (mut a, mut text) = asked(EDIT, MINE);
    a.set_ext_editor(&ready(MINE, &blank, false)).unwrap();
    assert!(confirm(&mut a, &mut text));
    assert_eq!(saves(&a), 0);
    let shown = a.scene(&mut text).panel.unwrap().items;
    assert!(shown.iter().any(|i| i.label == "Invalid meta data"));
    assert!(shown.iter().any(|i| i.label == "The meta data is invalid."));
    assert!(tap(&mut a, &mut text, 7), "OK");
    assert!(a.modal.is_none());
    assert_eq!(top(&a), EDIT);
}

/// Improvement: a component that cannot be built threw from `editor.build()` and crashed the
/// app; here Save says which one.
#[test]
fn a_component_that_cannot_be_built_is_named_and_not_saved() {
    for (from, to, name) in [
        (
            "\"label\": \"Day\"",
            "\"label\": \" \"",
            "Theme day is invalid.",
        ),
        ("\"id\": \"day\"", "\"id\": \"\"", "Theme Day is invalid."),
    ] {
        let (mut a, mut text) = editing();
        // The theme editor saved it so.
        let themed = ready(MINE, &manifest_with(from, to), true);
        a.set_ext_editor(&themed).unwrap();
        assert!(a.extensions.editor.changed);
        assert!(confirm(&mut a, &mut text));
        assert_eq!(saves(&a), 0, "{name}");
        assert_eq!(toast(&a).as_deref(), Some(name));
        assert!(a.files.busy.is_none());
    }
}

/// Improvement: Create wrote over any archive of the same name; it may not take an installed
/// extension's id here, and Kotlin refuses an archive already there.
#[test]
fn create_saves_a_new_archive_and_refuses_an_installed_id() {
    let (mut a, mut text) = creating();
    let id = format!("local.themes.{NOW}");
    let [job, ws, _, manifest, name, target] = save(&mut a, &mut text);
    assert_eq!([job, ws], [job::EXT_SAVE, WS]);
    assert!(manifest.contains(&format!("\"id\": \"{id}\"")));
    assert_eq!([name, target], [format!("{id}.flex"), String::new()]);
    a.file_report(&done_report(job::EXT_SAVE, result::TAKEN, WS, ""))
        .unwrap();
    assert_eq!(toast(&a).as_deref(), Some("The ID is installed."));
    assert_eq!(top(&a), CREATE);

    let (mut b, mut text) = creating();
    b.extensions.editor.installed.push(id);
    assert!(confirm(&mut b, &mut text));
    assert_eq!(saves(&b), 0);
    assert_eq!(toast(&b).as_deref(), Some("The ID is installed."));
}

#[test]
fn a_failed_save_keeps_the_draft_and_can_be_tried_again() {
    let (mut a, mut text) = changed();
    save(&mut a, &mut text);
    a.file_report(&done_report(job::EXT_SAVE, result::FAILED, WS, ""))
        .unwrap();
    assert_eq!(toast(&a).as_deref(), Some("Save failed."));
    assert_eq!(top(&a), EDIT);
    assert!(a.extensions.editor.changed);
    assert_eq!(theme_ids(&a), ["night"]);
    save(&mut a, &mut text);
    assert_eq!(saves(&a), 2, "the second Save runs");
}

#[test]
fn a_save_whose_workspace_is_gone_says_so() {
    let (mut a, mut text) = changed();
    save(&mut a, &mut text);
    a.file_report(&done_report(job::EXT_SAVE, result::GONE, WS, ""))
        .unwrap();
    assert_eq!(toast(&a).as_deref(), Some("Its draft is gone."));
    assert_eq!(a.extensions.editor.load, Load::Gone);
    a.scene(&mut text);
    assert_eq!(control(&a, STATUS_ID).label, "Its draft is gone.");
    assert!(!has(&a, lang_id(&a, Lang::Save)));
    a.back();
    assert_eq!(top(&a), HOME, "nothing left to lose");
}

/// Improvement: a Save the activity lost may have finished; the editor says it cannot tell, and
/// Save can be tried again.
#[test]
fn a_save_lost_with_the_activity_says_it_cannot_tell() {
    let (mut a, mut text) = changed();
    save(&mut a, &mut text);
    a.file_report(&idle()).unwrap();
    assert_eq!(toast(&a).as_deref(), Some("Save interrupted."));
    assert!(a.files.busy.is_none());
    assert_eq!(top(&a), EDIT);
    save(&mut a, &mut text);
}
