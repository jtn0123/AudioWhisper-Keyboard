//! Restore (stage4-backup-restore.md): reading a picked backup, what it shows and warns about,
//! the confirmation, part-way failures, malformed reports and the saved state. Back up, and
//! what both screens share, are in settings_ui_backup.rs.
mod backup_common;
use backup_common::*;
use render::settings_ui::{
    backup_saved::parse_backups,
    backup_screens::Note,
    catalogue::{Part, RESTORE_ROUTE},
};
use wire::settings_ui::files::{done, effect, job, mode, outcome, pick, report, result, work};

#[test]
fn restore_reads_the_picked_backup_and_shows_what_it_holds() {
    let (mut a, mut text) = app(RESTORE_ROUTE);
    assert!(control(&a, Part::About(0)).is_none());
    assert!(control(&a, Part::Group(0)).is_none());
    assert!(!control(&a, Part::Start).unwrap().enabled);
    let choose = control(&a, Part::Choose).unwrap();
    assert_eq!(
        (choose.role, choose.summary.as_str()),
        ("button", "state__no_file_selected")
    );
    assert!(tap(&mut a, &mut text, Part::Choose));
    let mut ask = vec![String::new(); pick::SIZE];
    ask[pick::EFFECT] = effect::PICK.into();
    ask[pick::MODE] = mode::OPEN.into();
    ask[pick::MIME] = "*/*".into();
    assert_eq!(std::mem::take(&mut a.effects), vec![ask]);
    report_to(
        &mut a,
        &mut text,
        picked_report(outcome::PICKED, &[("2", "aw.zip")]),
    );
    let read = work_packet(job::RESTORE_READ, &[(work::DOCUMENT, "2")]);
    assert_eq!(std::mem::take(&mut a.effects), vec![read]);
    assert_eq!(note(&a), Some(("settings__restore__reading".into(), false)));
    let (present, damaged) = (FOUR | (1 << 5), 1 << 5);
    let (present, damaged) = (present.to_string(), damaged.to_string());
    let fields = [
        (done::PACKAGE, PACKAGE),
        (done::VERSION_CODE, "120"),
        (done::VERSION_NAME, "1.2"),
        (done::DATE, DATE),
        (done::PRESENT, present.as_str()),
        (done::DAMAGED, damaged.as_str()),
    ];
    report_to(
        &mut a,
        &mut text,
        done_report(job::RESTORE_READ, result::OK, &fields),
    );
    assert_eq!(note(&a), None);
    assert_eq!(control(&a, Part::Choose).unwrap().summary, "aw.zip");
    let about: Vec<_> = (0..3)
        .map(|n| control(&a, Part::About(n)).unwrap())
        .collect();
    assert_eq!(about[0].label, DATE);
    assert_eq!(about[1].label, "1.2 (120)");
    assert_eq!(about[2].label, PACKAGE);
    assert!(about.iter().all(|c| c.role == "text"));
    // This app's own backup at this version: no warning row.
    assert!(control(&a, Part::Warning).is_none());
    // Images are missing and video damaged: both are off and say why. Text is offered, unticked.
    let images = control(&a, Part::Group(4)).unwrap();
    assert_eq!(
        (images.enabled, images.summary.as_str()),
        (false, "settings__restore__missing")
    );
    let video = control(&a, Part::Group(5)).unwrap();
    assert_eq!(
        (video.enabled, video.summary.as_str()),
        (false, "settings__restore__damaged")
    );
    assert_eq!(control(&a, Part::Group(3)).unwrap().checked, Some(false));
    assert_eq!(a.restorable(), FOUR);
    assert_eq!(a.backups.restore_mask, 0b111);
    assert!(control(&a, Part::Start).unwrap().enabled);
    // The clipboard's checkbox counts only the text it can restore.
    assert!(tap(&mut a, &mut text, Part::Clipboard));
    assert_eq!(control(&a, Part::Clipboard).unwrap().checked, Some(true));
}

#[test]
fn restore_warns_as_compose_did_but_not_about_its_own_backups() {
    for (package, version, said, usable) in [
        (PACKAGE, "130", "settings__restore__newer", true),
        (
            PACKAGE,
            "100",
            "backup_and_restore__restore__metadata_warn_different_version",
            true,
        ),
        (
            "org.example.keyboard",
            "120",
            "backup_and_restore__restore__metadata_warn_different_vendor",
            true,
        ),
        (
            PACKAGE,
            "10",
            "backup_and_restore__restore__metadata_error_invalid_metadata",
            false,
        ),
        (
            " ",
            "120",
            "backup_and_restore__restore__metadata_error_invalid_metadata",
            false,
        ),
        ("dev.patrickgold.florisboard", "120", "", true),
        (PACKAGE, "120", "", true),
    ] {
        let (mut a, mut text) = app(RESTORE_ROUTE);
        read(&mut a, &mut text, package, version, FOUR, 0);
        let warning = control(&a, Part::Warning);
        assert_eq!(
            warning.map(|c| c.label).unwrap_or_default(),
            said,
            "{package} {version}"
        );
        assert_eq!(control(&a, Part::Group(0)).is_some(), usable);
        assert_eq!(control(&a, Part::Start).unwrap().enabled, usable);
    }
}

#[test]
fn restore_asks_first_and_names_what_it_will_replace() {
    let (mut a, mut text) = app(RESTORE_ROUTE);
    a.strings.insert(
        "settings__restore__confirm_merge".into(),
        "Merge {groups} from {date}?".into(),
    );
    a.strings.insert(
        "settings__restore__confirm_erase".into(),
        "Erase, then {groups} from {date}?".into(),
    );
    read(&mut a, &mut text, PACKAGE, "120", FOUR, 0);
    assert!(tap(&mut a, &mut text, Part::Start));
    assert!(a.modal.is_some() && a.effects.is_empty());
    assert!(has_label(&a, &format!("Merge P, K, T from {DATE}?")));
    assert!(has_label(&a, "action__restore"));
    // Cancel restores nothing.
    assert!(a.activate(6));
    assert!(a.modal.is_none() && a.effects.is_empty());
    assert!(tap(&mut a, &mut text, Part::Mode(true)));
    assert!(tap(&mut a, &mut text, Part::Group(3)));
    assert!(tap(&mut a, &mut text, Part::Start));
    assert!(has_label(
        &a,
        &format!("Erase, then P, K, T, X from {DATE}?")
    ));
    assert!(a.activate(7));
    let apply = work_packet(
        job::RESTORE_APPLY,
        &[(work::MASK, "15"), (work::ERASE, "1")],
    );
    assert_eq!(a.effects, vec![apply]);
    assert_eq!(a.backups.applying, Some((FOUR, true)));
    a.scene(&mut text);
    assert_eq!(
        note(&a),
        Some(("settings__restore__restoring".into(), false))
    );
}

#[test]
fn a_restore_that_stops_part_way_says_what_came_back_and_can_be_tried_again() {
    for (erase, mask, steps, said) in [
        (true, "15", "3", "Back: P, K. Not back: T, X. Cleared."),
        (false, "7", "1", "Back: P. Not back: K, T."),
        (false, "7", "0", "settings__restore__nothing_changed"),
    ] {
        let (mut a, mut text) = app(RESTORE_ROUTE);
        a.strings.insert(
            "settings__restore__part_way".into(),
            "Back: {done}. Not back: {rest}.".into(),
        );
        a.strings
            .insert("settings__restore__cleared".into(), "Cleared.".into());
        read(&mut a, &mut text, PACKAGE, "120", FOUR, 0);
        if erase {
            assert!(tap(&mut a, &mut text, Part::Mode(true)));
            assert!(tap(&mut a, &mut text, Part::Group(3)));
        }
        assert!(tap(&mut a, &mut text, Part::Start));
        assert!(a.activate(7));
        assert_eq!(a.effects[0][work::MASK], mask);
        let fields = [(done::STEPS, steps)];
        report_to(
            &mut a,
            &mut text,
            done_report(job::RESTORE_APPLY, result::FAILED, &fields),
        );
        assert_eq!(note(&a), Some((said.into(), true)));
        assert!(a.backups.file.is_some() && a.backups.applying.is_none());
        assert!(control(&a, Part::Start).unwrap().enabled);
    }
}

#[test]
fn restore_says_why_a_file_was_not_read_and_keeps_what_it_had() {
    for (how, said) in [
        (result::UNREADABLE, "settings__restore__unreadable"),
        (result::NOT_BACKUP, "settings__restore__not_backup"),
        (result::BAD_METADATA, "settings__restore__bad_metadata"),
        (result::GONE, "settings__restore__unreadable"),
    ] {
        let (mut a, mut text) = app(RESTORE_ROUTE);
        assert!(tap(&mut a, &mut text, Part::Choose));
        report_to(
            &mut a,
            &mut text,
            picked_report(outcome::PICKED, &[("0", "photo.jpg")]),
        );
        report_to(&mut a, &mut text, done_report(job::RESTORE_READ, how, &[]));
        assert_eq!(note(&a), Some((said.into(), true)));
        assert!(a.backups.file.is_none() && a.backups.reading.is_none());
        assert!(!control(&a, Part::Start).unwrap().enabled);
    }
    for (answer, said) in [
        (outcome::CANCELLED, ("settings__restore__no_file", false)),
        (outcome::PICKED, ("settings__restore__no_file", false)),
        (outcome::NO_PICKER, ("settings__restore__no_picker", true)),
    ] {
        let (mut a, mut text) = app(RESTORE_ROUTE);
        read(&mut a, &mut text, PACKAGE, "120", FOUR, 0);
        assert!(tap(&mut a, &mut text, Part::Choose));
        report_to(&mut a, &mut text, picked_report(answer, &[]));
        assert_eq!(note(&a), Some((said.0.into(), said.1)));
        assert_eq!(
            a.backups.file.as_ref().map(|f| f.name.as_str()),
            Some("aw.zip")
        );
    }
}

#[test]
fn a_restore_closes_the_screen_with_compose_s_toast() {
    let (mut a, mut text) = app(RESTORE_ROUTE);
    read(&mut a, &mut text, PACKAGE, "120", FOUR, 0);
    assert!(tap(&mut a, &mut text, Part::Start));
    assert!(a.activate(7));
    a.effects.clear();
    report_to(
        &mut a,
        &mut text,
        done_report(job::RESTORE_APPLY, result::OK, &[]),
    );
    let toast = vec![
        effect::TOAST.to_owned(),
        "backup_and_restore__restore__success".into(),
    ];
    assert_eq!(a.effects, vec![toast, work_packet(job::DISCARD, &[])]);
    assert_eq!(a.stack.last().map(String::as_str), Some("settings/other"));
    assert_eq!(a.backups.file, None);
}

#[test]
fn reports_nothing_waits_for_change_nothing_and_malformed_ones_fail() {
    let (mut a, mut text) = app(RESTORE_ROUTE);
    report_to(
        &mut a,
        &mut text,
        picked_report(outcome::PICKED, &[("0", "x.zip")]),
    );
    assert!(a.effects.is_empty() && a.backups.reading.is_none());
    let mut odd = picked_report(outcome::PICKED, &[("0", "x.zip")]);
    odd.pop();
    assert!(a.file_report(&odd).is_err());
    assert!(a.file_report(&picked_report("maybe", &[])).is_err());
    assert!(a.file_report(&[report::PICKED.into()]).is_err());
    let mut short = done_report(job::RESTORE_READ, result::OK, &[]);
    short.pop();
    assert!(a.file_report(&short).is_err());
    assert!(a.file_report(&["nonsense".into()]).is_err());
    assert!(a.file_report(&[]).is_err());
}

#[test]
fn both_screens_survive_the_saved_state_and_bad_saved_state_fails() {
    let (mut a, mut text) = app(RESTORE_ROUTE);
    read(&mut a, &mut text, PACKAGE, "120", FOUR, 1 << 3);
    assert!(tap(&mut a, &mut text, Part::Mode(true)));
    assert!(tap(&mut a, &mut text, Part::Choose));
    a.backups.note = Some(Note {
        route: RESTORE_ROUTE.into(),
        text: "words".into(),
        error: true,
    });
    a.backups.applying = Some((3, true));
    a.files.busy = Some(job::RESTORE_APPLY.into());
    let saved = a.backups_saved();
    assert_eq!(
        parse_backups(&saved),
        Ok((a.files.clone(), a.backups.clone()))
    );
    let v: serde_json::Value = serde_json::from_str(&saved).unwrap();
    let broken = |path: &[&str], index: Option<usize>, value: serde_json::Value| {
        let mut v = v.clone();
        let mut at = &mut v;
        for key in path {
            at = &mut at[*key];
        }
        match index {
            Some(i) => at[i] = value,
            None => *at = value,
        }
        v.to_string()
    };
    for bad in [
        broken(&["choices"], Some(1), 64.into()),
        broken(&["choices"], Some(3), (-1).into()),
        broken(&["note"], Some(0), "settings/other".into()),
        broken(&["file"], None, serde_json::json!([1, 2])),
        broken(&["applying"], Some(1), "yes".into()),
        broken(&["busy"], None, 5.into()),
        "not json".into(),
    ] {
        assert!(parse_backups(&bad).is_err(), "{bad}");
    }
}
