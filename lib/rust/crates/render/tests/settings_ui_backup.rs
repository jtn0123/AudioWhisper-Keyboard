//! Back up (stage4-backup-restore.md): what each tap, picker answer and piece of file work leads
//! to, and what the screen says. Also what both screens share: how Other opens them, work lost
//! with the app, and leaving. Restore is in settings_ui_restore.rs. Kotlin's side, and the
//! archives themselves, are held by the JVM tests BackupFilesTest and SettingsFilesTest.
mod backup_common;
use backup_common::*;
use render::settings_ui::{
    Control,
    catalogue::{BACKUP_ROUTE, Part, RESTORE_ROUTE},
};
use wire::settings_ui::files::{done, effect, job, mode, outcome, pick, report, result, work};

#[test]
fn other_opens_both_screens_in_rust() {
    for (key, route) in [
        ("backup_and_restore__back_up__title", BACKUP_ROUTE),
        ("backup_and_restore__restore__title", RESTORE_ROUTE),
    ] {
        let (mut a, mut text) = app("settings/other");
        let i = a.rows().iter().position(|r| r.key == key).unwrap();
        assert!(a.activate(1000 + i as i32));
        assert_eq!(a.stack.last().map(String::as_str), Some(route));
        a.scene(&mut text);
        assert!(has_label(&a, "action__cancel"));
    }
}

#[test]
fn back_up_starts_as_compose_did_and_its_checkboxes_behave_alike() {
    let (mut a, mut text) = app(BACKUP_ROUTE);
    let to_file = control(&a, Part::Destination(0)).unwrap();
    assert_eq!((to_file.role, to_file.selected), ("radio", true));
    for g in 0..3 {
        assert_eq!(control(&a, Part::Group(g)).unwrap().checked, Some(true));
    }
    let clipboard = control(&a, Part::Clipboard).unwrap();
    assert_eq!(
        (clipboard.role, clipboard.checked),
        ("checkbox", Some(false))
    );
    // One clip makes the clipboard partly on; tapping it then turns every clip on, then off.
    assert!(tap(&mut a, &mut text, Part::Group(3)));
    assert_eq!(control(&a, Part::Clipboard).unwrap().checked, None);
    assert!(tap(&mut a, &mut text, Part::Clipboard));
    assert_eq!(a.backups.backup_mask, 0b111111);
    assert_eq!(control(&a, Part::Clipboard).unwrap().checked, Some(true));
    assert!(tap(&mut a, &mut text, Part::Clipboard));
    assert_eq!(a.backups.backup_mask, 0b000111);
    assert!(tap(&mut a, &mut text, Part::Destination(1)));
    assert!(a.backups.share);
    assert!(control(&a, Part::Destination(1)).unwrap().selected);
    // Nothing selected: Back up is off, as Compose's button was.
    for g in 0..3 {
        assert!(tap(&mut a, &mut text, Part::Group(g)));
    }
    assert!(!control(&a, Part::Start).unwrap().enabled);
    assert!(a.effects.is_empty());
}

#[test]
fn back_up_prepares_then_asks_where_then_writes_and_closes() {
    let (mut a, mut text) = app(BACKUP_ROUTE);
    assert!(tap(&mut a, &mut text, Part::Start));
    let prepare = work_packet(job::BACKUP_PREPARE, &[(work::MASK, "7")]);
    assert_eq!(std::mem::take(&mut a.effects), vec![prepare]);
    assert_eq!(
        note(&a),
        Some(("settings__backup__preparing".into(), false))
    );
    // While work runs only Cancel can be pressed, so no job starts twice.
    let pressable =
        |c: &&Control| c.id >= 1000 && matches!(c.role, "radio" | "checkbox" | "button");
    for c in a.controls.iter().filter(pressable) {
        assert_eq!(c.enabled, c.label == "action__cancel", "{}", c.label);
    }
    let name = "backup_com.jtn0123.audiowhisper.keyboard_120_1.zip";
    let fields = [(done::NAME, name)];
    report_to(
        &mut a,
        &mut text,
        done_report(job::BACKUP_PREPARE, result::OK, &fields),
    );
    let mut ask = vec![String::new(); pick::SIZE];
    ask[pick::EFFECT] = effect::PICK.into();
    ask[pick::MODE] = mode::CREATE.into();
    ask[pick::MIME] = "application/zip".into();
    ask[pick::NAME] = name.into();
    assert_eq!(std::mem::take(&mut a.effects), vec![ask]);
    assert_eq!(a.files.busy, None);
    assert_eq!(a.files.waiting.as_deref(), Some("backupSave"));
    report_to(
        &mut a,
        &mut text,
        picked_report(outcome::PICKED, &[("4", "b.zip")]),
    );
    let write = work_packet(job::BACKUP_WRITE, &[(work::DOCUMENT, "4")]);
    assert_eq!(std::mem::take(&mut a.effects), vec![write]);
    assert_eq!(note(&a), Some(("settings__backup__saving".into(), false)));
    report_to(
        &mut a,
        &mut text,
        done_report(job::BACKUP_WRITE, result::OK, &[]),
    );
    let toast = vec![
        effect::TOAST.to_owned(),
        "backup_and_restore__back_up__success".into(),
    ];
    assert_eq!(a.effects[0], toast);
    assert_eq!(a.effects[1], work_packet(job::DISCARD, &[]));
    assert_eq!(a.stack.last().map(String::as_str), Some("settings/other"));
}

#[test]
fn sharing_offers_the_archive_and_says_when_no_app_takes_it() {
    let (mut a, mut text) = app(BACKUP_ROUTE);
    assert!(tap(&mut a, &mut text, Part::Destination(1)));
    assert!(tap(&mut a, &mut text, Part::Start));
    a.effects.clear();
    report_to(
        &mut a,
        &mut text,
        done_report(job::BACKUP_PREPARE, result::OK, &[]),
    );
    assert_eq!(a.effects, vec![work_packet(job::BACKUP_SHARE, &[])]);
    report_to(
        &mut a,
        &mut text,
        done_report(job::BACKUP_SHARE, result::NO_APP, &[]),
    );
    assert_eq!(note(&a), Some(("settings__backup__no_app".into(), true)));
    report_to(
        &mut a,
        &mut text,
        done_report(job::BACKUP_SHARE, result::OK, &[]),
    );
    assert_eq!(
        note(&a),
        Some(("settings__backup__share_ready".into(), false))
    );
}

#[test]
fn a_closed_save_picker_or_failed_work_says_what_happened_and_keeps_the_screen() {
    for (answer, said) in [
        (outcome::CANCELLED, ("settings__backup__cancelled", false)),
        (outcome::NO_PICKER, ("settings__backup__no_picker", true)),
    ] {
        let (mut a, mut text) = app(BACKUP_ROUTE);
        assert!(tap(&mut a, &mut text, Part::Start));
        report_to(
            &mut a,
            &mut text,
            done_report(job::BACKUP_PREPARE, result::OK, &[]),
        );
        a.effects.clear();
        report_to(&mut a, &mut text, picked_report(answer, &[]));
        assert_eq!(a.effects, vec![work_packet(job::DISCARD, &[])]);
        assert_eq!(note(&a), Some((said.0.into(), said.1)));
        assert_eq!(a.stack.last().map(String::as_str), Some(BACKUP_ROUTE));
        assert!(control(&a, Part::Start).unwrap().enabled);
    }
    for (kind, how, said) in [
        (
            job::BACKUP_PREPARE,
            result::FAILED,
            "settings__backup__prepare_failed",
        ),
        (
            job::BACKUP_WRITE,
            result::UNWRITABLE,
            "settings__backup__write_failed",
        ),
        (job::BACKUP_WRITE, result::GONE, "settings__backup__gone"),
    ] {
        let (mut a, mut text) = app(BACKUP_ROUTE);
        report_to(&mut a, &mut text, done_report(kind, how, &[]));
        assert_eq!(note(&a), Some((said.into(), true)));
        let status = a.controls.iter().find(|c| c.role == "status").unwrap();
        assert_eq!((status.id, status.label.as_str()), (30, said));
    }
}

#[test]
fn work_lost_with_the_app_is_said_and_idle_alone_changes_nothing() {
    let (mut a, mut text) = app(BACKUP_ROUTE);
    report_to(&mut a, &mut text, vec![report::IDLE.into()]);
    assert_eq!(note(&a), None);
    assert!(tap(&mut a, &mut text, Part::Start));
    report_to(&mut a, &mut text, vec![report::IDLE.into()]);
    assert_eq!(
        note(&a),
        Some(("settings__files__interrupted".into(), true))
    );
    assert!(control(&a, Part::Start).unwrap().enabled);
    let (mut a, mut text) = app(RESTORE_ROUTE);
    read(&mut a, &mut text, PACKAGE, "120", FOUR, 0);
    assert!(tap(&mut a, &mut text, Part::Start));
    assert!(a.activate(7));
    report_to(&mut a, &mut text, vec![report::IDLE.into()]);
    assert_eq!(
        note(&a),
        Some(("settings__files__interrupted".into(), true))
    );
    assert!(a.backups.file.is_none() && a.backups.applying.is_none());
}

#[test]
fn leaving_forgets_the_choices_and_discards_archives_unless_a_restore_runs() {
    let (mut a, mut text) = app(BACKUP_ROUTE);
    assert!(tap(&mut a, &mut text, Part::Destination(1)));
    assert!(tap(&mut a, &mut text, Part::Group(0)));
    assert!(tap(&mut a, &mut text, Part::Cancel));
    assert_eq!(a.effects, vec![work_packet(job::DISCARD, &[])]);
    assert!(!a.backups.share && a.backups.backup_mask == 0b111);
    // A backup prepared after its screen closed is thrown away, not offered for saving.
    a.navigate(BACKUP_ROUTE);
    assert!(tap(&mut a, &mut text, Part::Start));
    a.back();
    a.effects.clear();
    report_to(
        &mut a,
        &mut text,
        done_report(job::BACKUP_PREPARE, result::OK, &[]),
    );
    assert_eq!(a.effects, vec![work_packet(job::DISCARD, &[])]);
    assert_eq!(a.files.waiting, None);

    let (mut a, mut text) = app(RESTORE_ROUTE);
    read(&mut a, &mut text, PACKAGE, "120", FOUR, 0);
    a.back();
    assert_eq!(a.effects, vec![work_packet(job::DISCARD, &[])]);
    assert_eq!(a.backups.file, None);
    // A restore still running keeps what it restores from, and its toast still comes.
    a.navigate(RESTORE_ROUTE);
    read(&mut a, &mut text, PACKAGE, "120", FOUR, 0);
    assert!(tap(&mut a, &mut text, Part::Start));
    assert!(a.activate(7));
    a.effects.clear();
    a.back();
    assert!(a.effects.is_empty() && a.backups.file.is_some());
    report_to(
        &mut a,
        &mut text,
        done_report(job::RESTORE_APPLY, result::OK, &[]),
    );
    assert_eq!(a.effects[0][0], effect::TOAST);
    assert_eq!(a.stack.last().map(String::as_str), Some("settings/other"));
}
