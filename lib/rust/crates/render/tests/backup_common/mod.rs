//! What the Back up and Restore tests share: an app on either screen, taps by part, and the
//! reports Kotlin sends. settings_ui_backup.rs and settings_ui_restore.rs each use all of it.
use render::{
    settings_ui::{
        App, Control,
        catalogue::{GROUP_NAMES, Kind, Part},
    },
    text::Text,
};
use wire::settings_ui::files::{
    document, done, effect, job, outcome, picked, report, result, work,
};

pub const PACKAGE: &str = "com.jtn0123.audiowhisper.keyboard";
pub const DATE: &str = "Nov 21, 2026, 10:00";
/// Preferences, keyboards, themes and text clips: a backup without images or video.
pub const FOUR: i32 = 0b001111;

pub fn app(route: &str) -> (App, Text) {
    let mut a = App {
        config: [1080.0, 6000.0, 2.625, 1.0],
        ..App::default()
    };
    a.facts.insert("version_code".into(), "120".into());
    a.facts.insert("package".into(), PACKAGE.into());
    for (key, name) in GROUP_NAMES.iter().zip(["P", "K", "T", "X", "I", "V"]) {
        a.strings.insert((*key).into(), name.into());
    }
    a.navigate("settings/other");
    a.navigate(route);
    let mut text = Text::new(vec![
        include_bytes!("../../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ]);
    a.scene(&mut text);
    a.effects.clear();
    (a, text)
}
pub fn row(a: &App, part: Part) -> usize {
    a.rows()
        .iter()
        .position(|r| matches!(r.kind, Kind::Backup(q) if q == part))
        .unwrap()
}
pub fn control(a: &App, part: Part) -> Option<Control> {
    let id = 1000 + row(a, part) as i32;
    a.controls.iter().find(|c| c.id == id).cloned()
}
/// Paints, taps `part` as it now stands, and paints again.
pub fn tap(a: &mut App, text: &mut Text, part: Part) -> bool {
    a.scene(text);
    let tapped = a.activate(1000 + row(a, part) as i32);
    a.scene(text);
    tapped
}
pub fn done_report(kind: &str, how: &str, fields: &[(usize, &str)]) -> Vec<String> {
    let mut p = vec![String::new(); done::SIZE];
    p[done::REPORT] = report::DONE.into();
    p[done::JOB] = kind.into();
    p[done::RESULT] = how.into();
    for (at, value) in fields {
        p[*at] = (*value).into();
    }
    p
}
pub fn picked_report(how: &str, documents: &[(&str, &str)]) -> Vec<String> {
    let mut p = vec![String::new(); picked::SIZE];
    p[picked::REPORT] = report::PICKED.into();
    p[picked::OUTCOME] = how.into();
    for (handle, name) in documents {
        let mut d = vec![String::new(); document::SIZE];
        d[document::HANDLE] = (*handle).into();
        d[document::NAME] = (*name).into();
        p.extend(d);
    }
    p
}
pub fn work_packet(kind: &str, fields: &[(usize, &str)]) -> Vec<String> {
    let mut p = vec![String::new(); work::SIZE];
    p[work::EFFECT] = effect::WORK.into();
    p[work::JOB] = kind.into();
    for (at, value) in fields {
        p[*at] = (*value).into();
    }
    p
}
pub fn report_to(a: &mut App, text: &mut Text, p: Vec<String>) {
    a.file_report(&p).unwrap();
    a.scene(text);
}
pub fn note(a: &App) -> Option<(String, bool)> {
    a.backups.note.as_ref().map(|n| (n.text.clone(), n.error))
}
pub fn has_label(a: &App, label: &str) -> bool {
    a.controls.iter().any(|c| c.label == label)
}
/// Restore, with `aw.zip` picked and read: a backup of `package` at `version`.
pub fn read(
    a: &mut App,
    text: &mut Text,
    package: &str,
    version: &str,
    present: i32,
    damaged: i32,
) {
    assert!(tap(a, text, Part::Choose));
    report_to(a, text, picked_report(outcome::PICKED, &[("0", "aw.zip")]));
    let (present, damaged) = (present.to_string(), damaged.to_string());
    let fields = [
        (done::PACKAGE, package),
        (done::VERSION_CODE, version),
        (done::VERSION_NAME, "1.2"),
        (done::DATE, DATE),
        (done::PRESENT, present.as_str()),
        (done::DAMAGED, damaged.as_str()),
    ];
    report_to(a, text, done_report(job::RESTORE_READ, result::OK, &fields));
    a.effects.clear();
}
