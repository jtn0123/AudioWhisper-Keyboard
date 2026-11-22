//! ExtensionEditFilesScreen: each folder's files, a folder's Add through the picker, and a
//! file's properties with Rename and Delete, each change Kotlin's file work.
use super::common::*;

const FONTS: &str = "font/*|application/font-*|application/x-font-*|application/vnd.ms-fontobject";

/// The files sheet, open from the editor, its effects cleared.
fn files() -> (App, Text) {
    let (mut a, mut text) = editing();
    assert!(tap(&mut a, &mut text, MANAGE_ID + 2));
    assert_eq!(top(&a), FILES);
    a.effects.clear();
    (a, text)
}
/// The fonts' Add tapped and the picker answering with `name`: the import sheet on top.
fn importing(name: &str) -> (App, Text) {
    let (mut a, mut text) = files();
    assert!(tap(&mut a, &mut text, FOLDER_ID + 1));
    a.file_report(&picked_report(how::PICKED, &[("7", name)]))
        .unwrap();
    a.scene(&mut text);
    a.effects.clear();
    (a, text)
}
/// The font `a.ttf`'s properties, open.
fn properties() -> (App, Text) {
    let (mut a, mut text) = files();
    assert!(tap(&mut a, &mut text, FOLDER_ID + 10));
    assert_eq!(top(&a), FILE);
    a.effects.clear();
    (a, text)
}

#[test]
fn each_folder_lists_its_files_under_a_header_with_add() {
    let (mut a, mut text) = files();
    a.scene(&mut text);
    assert_eq!(a.title(), "Manage archive files");
    for (f, title, file) in [(0, "Fonts", "a.ttf"), (1, "Images", "bg.png")] {
        let id = FOLDER_ID + 1000 * f;
        assert_eq!(control(&a, id).label, title);
        let add = control(&a, id + 1);
        assert_eq!([add.label.as_str(), add.summary.as_str()], ["Add", title]);
        assert_eq!(control(&a, id + 10).label, file);
        assert!(!has(&a, id + 11));
    }
    let order = [
        1,
        FOLDER_ID + 1,
        FOLDER_ID + 10,
        FOLDER_ID + 1001,
        FOLDER_ID + 1010,
    ];
    assert_eq!(focus_order(&mut a), order);
}

/// Improvement: the picker offers only what the folder takes, as ExtensionEditFilesScreen's
/// filter did, and Kotlin checks the type again before copying.
#[test]
fn a_folders_add_opens_the_picker_for_its_media_types() {
    for (f, types) in [(0, FONTS), (1, "image/*")] {
        let (mut a, mut text) = files();
        assert!(tap(&mut a, &mut text, FOLDER_ID + 1000 * f + 1));
        assert_eq!(a.effects, [["pick", "open", types, ""]]);
        assert_eq!(a.files.waiting, Some(format!("extEditor:{f}")));
    }
}

#[test]
fn a_picked_file_is_named_then_copied_in() {
    let (mut a, mut text) = importing("b.ttf");
    assert_eq!(top(&a), FILE_ADD);
    assert_eq!(a.title(), "Import file");
    assert_eq!(
        a.extensions.editor.file_form.as_ref().unwrap().name,
        "b.ttf"
    );
    type_into(&mut a, &mut text, Edit::FileName, " c.ttf ");
    assert_eq!(control(&a, lang_id(&a, Lang::Save)).label, "Add");
    assert!(confirm(&mut a, &mut text));
    let packet = work(&a).expect("Kotlin copies it in");
    assert_eq!(
        packet,
        [job::EXT_FILE_ADD, WS, "7", FONTS, "fonts/c.ttf", ""]
    );
    a.file_report(&done_report(
        job::EXT_FILE_ADD,
        result::OK,
        "fonts/c.ttf",
        "",
    ))
    .unwrap();
    assert_eq!(top(&a), FILES);
    let e = &a.extensions.editor;
    let want = [
        (0, "a.ttf".into()),
        (1, "bg.png".into()),
        (0, "c.ttf".into()),
    ];
    assert_eq!(e.files, want);
    assert!(e.changed && e.file_form.is_none());
    assert!(last(&a, effect::SYNC).is_some());
    a.scene(&mut text);
    assert_eq!(control(&a, FOLDER_ID + 11).label, "c.ttf");
}

/// Improvement: Compose said nothing when a copy failed; each way it can fail is said here.
#[test]
fn an_add_that_fails_says_why_and_keeps_the_sheet() {
    for (how, said) in [
        (result::TAKEN, "The file is there."),
        (result::WRONG_TYPE, "Wrong type."),
        (result::FAILED, "File failed."),
        (result::GONE, "File failed."),
    ] {
        let (mut a, mut text) = importing("b.ttf");
        assert!(confirm(&mut a, &mut text));
        a.file_report(&done_report(job::EXT_FILE_ADD, how, "fonts/b.ttf", ""))
            .unwrap();
        assert_eq!(toast(&a).as_deref(), Some(said), "{how}");
        assert_eq!(top(&a), FILE_ADD, "{how}");
        assert_eq!(a.extensions.editor.files.len(), 2, "{how}");
        assert!(!a.extensions.editor.changed, "{how}");
    }
}

#[test]
fn a_name_that_leaves_the_folder_or_is_taken_is_said_under_the_field() {
    for (name, said) in [
        ("../b.ttf", "Bad file name."),
        ("..", "Bad file name."),
        ("  ", "Bad file name."),
        ("a.ttf", "The file is there."),
    ] {
        let (mut a, mut text) = importing("b.ttf");
        type_into(&mut a, &mut text, Edit::FileName, name);
        assert!(confirm(&mut a, &mut text));
        assert!(work(&a).is_none(), "{name}");
        assert_eq!(a.ext_error(Edit::FileName), said, "{name}");
        assert_eq!(top(&a), FILE_ADD);
    }
}

#[test]
fn a_cancelled_picker_changes_nothing_and_no_picker_says_so() {
    let (mut a, mut text) = files();
    assert!(tap(&mut a, &mut text, FOLDER_ID + 1));
    a.file_report(&picked_report(how::CANCELLED, &[])).unwrap();
    assert_eq!(top(&a), FILES);
    assert!(a.extensions.editor.file_form.is_none());
    assert!(tap(&mut a, &mut text, FOLDER_ID + 1));
    a.file_report(&picked_report(how::NO_PICKER, &[])).unwrap();
    assert_eq!(toast(&a).as_deref(), Some("No picker."));
    assert_eq!(top(&a), FILES);
}

#[test]
fn a_files_properties_rename_it() {
    let (mut a, mut text) = properties();
    assert_eq!(a.title(), "Properties");
    assert_eq!(control(&a, FILE_DELETE_ID).label, "Delete");
    assert_eq!(control(&a, lang_id(&a, Lang::Save)).label, "Apply");
    type_into(&mut a, &mut text, Edit::FileName, "b.ttf");
    assert!(confirm(&mut a, &mut text));
    let packet = work(&a).expect("Kotlin renames it");
    assert_eq!(
        packet,
        [
            job::EXT_FILE_RENAME,
            WS,
            "",
            "fonts/a.ttf",
            "fonts/b.ttf",
            ""
        ]
    );
    let done = done_report(
        job::EXT_FILE_RENAME,
        result::OK,
        "fonts/b.ttf",
        "fonts/a.ttf",
    );
    a.file_report(&done).unwrap();
    assert_eq!(top(&a), FILES);
    let want = [(0, "b.ttf".into()), (1, "bg.png".into())];
    assert_eq!(a.extensions.editor.files, want);
    assert!(a.extensions.editor.changed);
}

#[test]
fn apply_with_the_same_name_closes_without_work() {
    let (mut a, mut text) = properties();
    assert!(confirm(&mut a, &mut text));
    assert_eq!(top(&a), FILES);
    assert!(work(&a).is_none());
    assert!(!a.extensions.editor.changed);
}

#[test]
fn a_files_delete_removes_it() {
    let (mut a, mut text) = properties();
    assert!(tap(&mut a, &mut text, FILE_DELETE_ID));
    let packet = work(&a).expect("Kotlin deletes it");
    assert_eq!(
        packet,
        [job::EXT_FILE_DELETE, WS, "", "", "fonts/a.ttf", ""]
    );
    // Delete while it runs asks nothing more.
    assert!(tap(&mut a, &mut text, FILE_DELETE_ID));
    assert_eq!(a.effects.iter().filter(|e| e[0] == "work").count(), 1);
    a.file_report(&done_report(
        job::EXT_FILE_DELETE,
        result::OK,
        "fonts/a.ttf",
        "",
    ))
    .unwrap();
    assert_eq!(top(&a), FILES);
    assert_eq!(a.extensions.editor.files, [(1, "bg.png".into())]);
    a.scene(&mut text);
    assert!(!has(&a, FOLDER_ID + 10));
}

/// Improvement: Compose changed the archive's files without marking the extension changed, so
/// Back left without asking; a file change is a change to lose here.
#[test]
fn a_file_change_is_an_unsaved_change() {
    let (mut a, mut text) = properties();
    assert!(tap(&mut a, &mut text, FILE_DELETE_ID));
    a.file_report(&done_report(
        job::EXT_FILE_DELETE,
        result::OK,
        "fonts/a.ttf",
        "",
    ))
    .unwrap();
    a.back();
    assert_eq!(top(&a), EDIT);
    a.back();
    assert!(a.unsaved_open());
}

#[test]
fn a_file_job_lost_with_the_activity_says_so() {
    let (mut a, mut text) = properties();
    assert!(tap(&mut a, &mut text, FILE_DELETE_ID));
    a.file_report(&idle()).unwrap();
    assert_eq!(toast(&a).as_deref(), Some("Interrupted."));
    assert_eq!(top(&a), FILE);
    assert!(a.files.busy.is_none());
}
