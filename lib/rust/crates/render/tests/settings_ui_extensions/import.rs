//! The importer: its four routes, Select files and the read, Import, the ways out, a file another
//! app shared, recreation, and what each failure says.
use super::common::*;
use super::import_common::*;
use render::settings_ui::catalogue::{HOME_ROUTE, native};
use wire::settings_ui::extensions::link;
use wire::settings_ui::files::{job, outcome as how, report, result};

/// The `work` effect for `job` with its document, text and extension fields.
fn work(job: &str, [document, text, extension]: [&str; 3]) -> Vec<String> {
    let fields = ["work", job, "", "", document, text, extension];
    fields.iter().map(|f| f.to_string()).collect()
}
/// A fresh app given `a`'s stack and file work, as `render_settings_state::restore` does.
fn recreated(a: &App) -> App {
    let mut b = App {
        config: a.config,
        strings: a.strings.clone(),
        ..App::default()
    };
    b.stack = a.stack.clone();
    b.scrolls = a.scrolls.clone();
    b.files = a.files.clone();
    b.extensions_restored();
    b
}
/// Select files, the picker answering with `handles`; the effects cleared.
fn pick(a: &mut App, text: &mut Text, handles: &[&str]) {
    assert!(tap(a, text, SELECT_ID));
    a.file_report(&picked_report(how::PICKED, handles)).unwrap();
    a.effects.clear();
}

#[test]
fn each_importer_is_drawn_here_with_its_title() {
    let titles = [
        (ANY, "Import extension"),
        (KEYBOARDS, "Import keyboard extension"),
        (THEMES, "Import theme extension"),
        (PACKS, "Import language pack extension"),
    ];
    for (route, title) in titles {
        let (mut a, mut text) = importer(route);
        assert!(native(route) && native(&format!("{route}?uuid=x")));
        assert_eq!(a.title(), title);
        // Unlike the other extension screens, the importer needs no index.
        assert!(a.effects.is_empty(), "{:?}", a.effects);
        let select = control(&a, SELECT_ID);
        assert_eq!(
            (select.label.as_str(), select.role),
            ("Select files", "button")
        );
        let centre = (select.bounds.left + select.bounds.right) / 2.0;
        assert!((centre - 540.0).abs() <= 1.0, "{centre}");
        let none = control(&a, NONE_ID);
        assert_eq!(
            (none.label.as_str(), none.role),
            ("No files selected", "text")
        );
        // 16 dp below the 40 dp button, which is touched 4 dp past its edges.
        let below = none.bounds.top - (select.bounds.bottom - px(4.0));
        assert!((below - px(16.0)).abs() <= 1.0, "{below}");
        let items = painted(&mut a, &mut text);
        let italic = items.iter().find(|i| i.label == "No files selected");
        assert!(italic.unwrap().text.italic);
        // Cancel and Import in the bottom bar; with nothing to import, Import is off.
        assert_eq!(control(&a, CANCEL).label, "Cancel");
        let import = control(&a, IMPORT);
        assert_eq!((import.label.as_str(), import.enabled), ("Import", false));
        assert_eq!(focus_order(&mut a)[1..], [SELECT_ID, CANCEL]);
    }
}

#[test]
fn select_files_reads_the_picked_files_then_shows_what_they_hold() {
    let (mut a, mut text) = importer(ANY);
    assert!(tap(&mut a, &mut text, SELECT_ID));
    assert_eq!(a.effects, [["pick", "openMany", "*/*", ""]]);
    a.effects.clear();
    a.file_report(&picked_report(how::PICKED, &["4", "5"]))
        .unwrap();
    assert_eq!(a.effects, [work(job::EXT_READ, ["4|5", "", ""])]);
    a.effects.clear();
    a.file_report(&done_report(job::EXT_READ, result::OK, "w1"))
        .unwrap();
    assert_eq!(a.stack.last(), Some(&format!("{ANY}?files=w1")));
    assert_eq!(a.effects, [["extensionFiles", "w1"]]);
    a.set_import_files(&files_report("w1", true, &five()))
        .unwrap();
    a.scene(&mut text);
    let names: Vec<String> = (0..5)
        .map(|f| control(&a, FILE_ID + 1000 * f).label)
        .collect();
    let expected = [
        "ocean.flex",
        "keys.flex",
        "floris.flex",
        "broken.flex",
        "notes.txt",
    ];
    assert_eq!(names, expected);
    assert!(!has(&a, NONE_ID));
    // Files picked here can be swapped for others, as Compose allowed.
    assert!(has(&a, SELECT_ID));
    assert!(control(&a, IMPORT).enabled);
}

/// Improvement: Compose read the files, unpacking every archive, on the main thread, and
/// imported and exported there too; Kotlin does all three off it, and the importer waits for the
/// read without claiming nothing was chosen.
#[test]
fn the_importer_waits_for_the_read_without_freezing() {
    let (mut a, mut text) = importer(ANY);
    pick(&mut a, &mut text, &["4"]);
    a.scene(&mut text);
    assert_eq!(a.files.busy.as_deref(), Some(job::EXT_READ));
    assert!(!has(&a, NONE_ID) && !has(&a, FAILED_ID));
    assert!(!control(&a, IMPORT).enabled);
    // A tap meanwhile is still answered.
    assert!(tap(&mut a, &mut text, SELECT_ID));
    assert_eq!(a.effects, [["pick", "openMany", "*/*", ""]]);
}

#[test]
fn import_imports_the_files_it_can_then_leaves() {
    let (mut a, mut text) = showing(ANY, "w1", &five());
    assert!(tap(&mut a, &mut text, IMPORT));
    // Ocean and Keys: Floris is a core extension, Broken would not unpack, Notes is no archive.
    assert_eq!(a.effects, [work(job::EXT_IMPORT, ["", "0,1", "w1"])]);
    a.effects.clear();
    a.scene(&mut text);
    // While it runs, Import is off: one tap imports once.
    assert!(!control(&a, IMPORT).enabled);
    a.file_report(&done_report(job::EXT_IMPORT, result::OK, ""))
        .unwrap();
    assert_eq!(toasts(&a), ["Imported!"]);
    assert_eq!(a.stack.last().map(String::as_str), Some(HOME_ROUTE));
    assert!(
        a.effects
            .contains(&vec!["extensionFilesClose".into(), "w1".into()])
    );
}

/// Improvement: Compose closed the workspace only on Cancel; Back and Up left what was read in
/// the cache. Leaving the importer any way deletes it, and so does choosing other files.
#[test]
fn leaving_the_importer_any_way_closes_its_workspace() {
    let close = vec!["extensionFilesClose".to_string(), "w1".into()];
    let (mut a, mut text) = showing(ANY, "w1", &five());
    assert!(tap(&mut a, &mut text, CANCEL));
    assert_eq!(a.effects, std::slice::from_ref(&close));
    assert_eq!(a.stack.last().map(String::as_str), Some(HOME_ROUTE));
    let (mut a, _) = showing(THEMES, "w1", &five());
    a.back();
    assert_eq!(a.effects, std::slice::from_ref(&close));
    let (mut a, mut text) = showing(ANY, "w1", &five());
    assert!(tap(&mut a, &mut text, SELECT_ID));
    a.file_report(&picked_report(how::PICKED, &["9"])).unwrap();
    assert_eq!(a.effects[1..], [close, work(job::EXT_READ, ["9", "", ""])]);
    assert_eq!(a.stack.last().map(String::as_str), Some(ANY));
    // An importer with nothing read closes nothing.
    let (mut a, _) = importer(ANY);
    a.back();
    assert!(a.effects.is_empty(), "{:?}", a.effects);
}

#[test]
fn a_dismissed_picker_changes_nothing_and_a_missing_one_says_so() {
    let (mut a, mut text) = showing(ANY, "w1", &five());
    assert!(tap(&mut a, &mut text, SELECT_ID));
    a.effects.clear();
    a.file_report(&picked_report(how::CANCELLED, &[])).unwrap();
    assert!(a.effects.is_empty());
    assert_eq!(a.stack.last(), Some(&format!("{ANY}?files=w1")));
    assert!(tap(&mut a, &mut text, SELECT_ID));
    a.effects.clear();
    a.file_report(&picked_report(how::NO_PICKER, &[])).unwrap();
    assert_eq!(toasts(&a), ["No picker."]);
}

/// A file another app shared, as FlorisAppActivity hands it over: read here, then shown without
/// Select files, as Compose showed the workspace its route carried.
#[test]
fn a_shared_file_is_read_and_shown_without_select_files() {
    let (mut a, mut text) = fresh();
    a.start(Some(&format!("{}7", link::SHARED)));
    assert_eq!(
        a.stack,
        [HOME_ROUTE.to_string(), format!("{ANY}?document=7")]
    );
    a.scene(&mut text);
    assert_eq!(a.effects, [work(job::EXT_READ, ["7", "", ""])]);
    assert_eq!(a.title(), "Import extension");
    assert!(!has(&a, SELECT_ID) && !has(&a, NONE_ID));
    // The activity recreated while it reads does not read it twice.
    assert!(recreated(&a).effects.is_empty());
    a.effects.clear();
    a.file_report(&done_report(job::EXT_READ, result::OK, "w9"))
        .unwrap();
    assert_eq!(a.stack.last(), Some(&format!("{ANY}?uuid=w9")));
    assert_eq!(a.effects, [["extensionFiles", "w9"]]);
    a.set_import_files(&files_report("w9", true, &five()[..2]))
        .unwrap();
    a.scene(&mut text);
    assert!(!has(&a, SELECT_ID));
    assert_eq!(control(&a, FILE_ID).label, "ocean.flex");
}

/// Improvement: Compose kept the files read in a plain `remember`, so turning the phone lost them
/// and left them in the cache; the workspace is in the route, so they come back.
#[test]
fn the_selection_survives_recreation() {
    let (a, mut text) = showing(THEMES, "w1", &five());
    let mut b = recreated(&a);
    assert_eq!(b.effects, [["extensionFiles", "w1"]]);
    b.set_import_files(&files_report("w1", true, &five()))
        .unwrap();
    b.scene(&mut text);
    assert_eq!(b.title(), "Import theme extension");
    assert_eq!(control(&b, FILE_ID + 4000).label, "notes.txt");
    assert!(control(&b, IMPORT).enabled);
}

/// Improvement: after Android ended the app, Compose's importer came back to its old workspace
/// with no files and no Select files, a dead end; a closed workspace leaves Select files.
#[test]
fn a_workspace_android_cleared_offers_select_files_again() {
    let (mut a, mut text) = importer(&format!("{ANY}?uuid=w1"));
    assert!(!has(&a, SELECT_ID));
    // A report on some other workspace changes nothing.
    a.set_import_files(&files_report("w0", false, &[])).unwrap();
    assert_eq!(a.stack.last(), Some(&format!("{ANY}?uuid=w1")));
    a.set_import_files(&files_report("w1", false, &[])).unwrap();
    a.scene(&mut text);
    assert_eq!(a.stack.last().map(String::as_str), Some(ANY));
    assert!(has(&a, SELECT_ID) && has(&a, NONE_ID));
}

/// Improvement: Compose put an exception's own text in its toasts, which can hold a file's
/// address, and showed a read that failed as a stack trace; every failure is told in plain words.
#[test]
fn import_failures_are_told_in_plain_words() {
    let (mut a, mut text) = importer(ANY);
    pick(&mut a, &mut text, &["4"]);
    a.file_report(&done_report(job::EXT_READ, result::UNREADABLE, ""))
        .unwrap();
    a.scene(&mut text);
    let failed = control(&a, FAILED_ID);
    assert_eq!(
        (failed.label.as_str(), failed.role),
        ("The files could not be read.", "text")
    );
    assert!(has(&a, SELECT_ID) && !has(&a, NONE_ID));
    let (mut a, mut text) = showing(ANY, "w1", &five());
    assert!(tap(&mut a, &mut text, IMPORT));
    a.file_report(&done_report(job::EXT_IMPORT, result::FAILED, ""))
        .unwrap();
    assert_eq!(toasts(&a), ["Import failed: Something went wrong"]);
    assert_eq!(a.stack.last(), Some(&format!("{ANY}?files=w1")));
    let (mut a, mut text) = showing(ANY, "w1", &five());
    assert!(tap(&mut a, &mut text, IMPORT));
    a.file_report(&done_report(job::EXT_IMPORT, result::GONE, ""))
        .unwrap();
    a.scene(&mut text);
    assert_eq!(toasts(&a), ["Import failed: The files are gone."]);
    assert_eq!(a.stack.last().map(String::as_str), Some(ANY));
    assert!(has(&a, SELECT_ID) && has(&a, NONE_ID));
}

#[test]
fn work_lost_with_the_app_says_so() {
    let (a, mut text) = importer(&format!("{ANY}?document=7"));
    let mut b = recreated(&a);
    b.file_report(&[report::IDLE.into()]).unwrap();
    b.scene(&mut text);
    assert_eq!(toasts(&b), ["Interrupted."]);
    assert_eq!(b.stack.last().map(String::as_str), Some(ANY));
    assert!(has(&b, SELECT_ID));
    let (mut a, mut text) = importer(ANY);
    pick(&mut a, &mut text, &["4"]);
    a.file_report(&[report::IDLE.into()]).unwrap();
    assert_eq!(toasts(&a), ["Interrupted."]);
}

#[test]
fn export_writes_the_extension_into_the_picked_document() {
    let route = format!("ext/view/{MINE}");
    let (mut a, mut text) = app(&route);
    assert!(tap(&mut a, &mut text, page::EXPORT_ID));
    a.effects.clear();
    a.file_report(&picked_report(how::PICKED, &["3"])).unwrap();
    assert_eq!(a.effects, [work(job::EXT_EXPORT, ["3", "", MINE])]);
    a.effects.clear();
    a.file_report(&done_report(job::EXT_EXPORT, result::OK, ""))
        .unwrap();
    assert_eq!(toasts(&a), ["Exported!"]);
    assert_eq!(a.stack.last(), Some(&route));
    let outcomes = [
        (
            result::UNWRITABLE,
            "Export failed: The file could not be written.",
        ),
        (result::GONE, "Export failed: Something went wrong"),
        (result::FAILED, "Export failed: Something went wrong"),
    ];
    for (outcome, toast) in outcomes {
        assert!(tap(&mut a, &mut text, page::EXPORT_ID));
        a.file_report(&picked_report(how::PICKED, &["3"])).unwrap();
        a.effects.clear();
        a.file_report(&done_report(job::EXT_EXPORT, outcome, ""))
            .unwrap();
        assert_eq!(toasts(&a), [toast]);
    }
    // A dismissed picker exports nothing, and says nothing.
    assert!(tap(&mut a, &mut text, page::EXPORT_ID));
    a.effects.clear();
    a.file_report(&picked_report(how::CANCELLED, &[])).unwrap();
    assert!(a.effects.is_empty());
}
