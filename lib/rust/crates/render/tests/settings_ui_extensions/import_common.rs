//! The importer at a route, the reports Kotlin sends it, and the files a workspace holds: a theme
//! archive, a keyboard archive, a core one, one that would not unpack and a text file.
use super::common::*;
use wire::settings_ui::extensions::file;
use wire::settings_ui::files::{document, done, picked, report};

pub use import::{ANY, KEYBOARDS, PACKS, THEMES};
pub use render::settings_ui::extensions::import_view::{FAILED_ID, FILE_ID, NONE_ID, SELECT_ID};
pub const FLEX: &str = "application/vnd.florisboard.extension+zip";

/// A file row: its name, media type, size and whether it is typed as a `.flex`.
pub fn file_row(name: &str, media: &str, size: &str, flex: bool) -> Vec<String> {
    line(file::FILE, "", ["", name, media, size], flex)
}
/// An archive row: its id, serial name, title, version and whether it replaces a core one.
pub fn archive(id: &str, serial: &str, title: &str, core: bool) -> Vec<String> {
    line(file::ARCHIVE, id, [serial, title, "1.2.0", ""], core)
}
/// A theme archive by two people with a description and two themes, a keyboard archive, a core
/// theme archive, a `.flex` that would not unpack and a text file.
pub fn five() -> Vec<Vec<String>> {
    vec![
        file_row("ocean.flex", FLEX, "12 kB", true),
        archive("my.ocean", extensions::THEME, "Ocean themes", false),
        line(
            file::DESCRIPTION,
            "",
            ["", "", "", "Blue and green."],
            false,
        ),
        line(file::MAINTAINER, "", ["", "Ann", "", ""], false),
        line(file::MAINTAINER, "", ["", "Bo", "", ""], false),
        line(file::COMPONENT, "", ["ocean_day", "", "", ""], false),
        line(file::COMPONENT, "", ["ocean_night", "", "", ""], false),
        file_row("keys.flex", FLEX, "3 kB", true),
        archive("my.keys", extensions::KEYBOARD, "Keys", false),
        file_row("floris.flex", FLEX, "40 kB", true),
        archive(FLORIS, extensions::THEME, "FlorisBoard themes", true),
        file_row("broken.flex", FLEX, "1 kB", true),
        file_row("notes.txt", "", "2 B", false),
    ]
}
/// The `EXTENSION_FILES` report on workspace `uuid`, open or not, holding `files`.
pub fn files_report(uuid: &str, open: bool, files: &[Vec<String>]) -> Vec<String> {
    let mut out = line(file::WORKSPACE, uuid, ["", "", "", ""], open);
    out.extend(files.concat());
    out
}
pub fn picked_report(how: &str, handles: &[&str]) -> Vec<String> {
    let mut p = vec![String::new(); picked::SIZE];
    p[picked::REPORT] = report::PICKED.into();
    p[picked::OUTCOME] = how.into();
    for h in handles {
        let mut d = vec![String::new(); document::SIZE];
        d[document::HANDLE] = (*h).into();
        p.extend(d);
    }
    p
}
pub fn done_report(job: &str, how: &str, name: &str) -> Vec<String> {
    let mut p = vec![String::new(); done::SIZE];
    p[done::REPORT] = report::DONE.into();
    p[done::JOB] = job.into();
    p[done::RESULT] = how.into();
    p[done::NAME] = name.into();
    p
}
/// The importer at `route`, painted; it asks Kotlin for nothing until files are chosen.
pub fn importer(route: &str) -> (App, Text) {
    let (mut a, mut text) = fresh();
    a.navigate(render::settings_ui::catalogue::HOME_ROUTE);
    a.effects.clear();
    a.navigate(route);
    a.scene(&mut text);
    (a, text)
}
/// The importer at `route` showing workspace `uuid` with `files`, its effects cleared.
pub fn showing(route: &str, uuid: &str, files: &[Vec<String>]) -> (App, Text) {
    let (mut a, mut text) = importer(&format!("{route}?files={uuid}"));
    assert_eq!(a.effects, [["extensionFiles", uuid]]);
    a.set_import_files(&files_report(uuid, true, files))
        .unwrap();
    a.effects.clear();
    a.scene(&mut text);
    (a, text)
}
/// The label of every control in file `f`'s box, in the order TalkBack reads them.
pub fn box_labels(a: &App, f: usize) -> Vec<String> {
    let first = FILE_ID + 1000 * f as i32;
    a.controls
        .iter()
        .filter(|c| (first..first + 1000).contains(&c.id))
        .map(|c| c.label.clone())
        .collect()
}
/// The toast effects' texts.
pub fn toasts(a: &App) -> Vec<String> {
    let toasts = a.effects.iter().filter(|e| e[0] == "toast");
    toasts.map(|e| e[1].clone()).collect()
}
/// The footer's Cancel and Import: rows 1 and 2.
pub const CANCEL: i32 = 1001;
pub const IMPORT: i32 = 1002;
