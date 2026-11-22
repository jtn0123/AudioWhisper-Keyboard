//! Opening the editor: what it asks Kotlin for, the draft it starts from, what it says when it
//! cannot open, and the screen ExtensionEditScreen drew once it did.
use super::common::*;

#[test]
fn edit_asks_kotlin_for_a_workspace_and_says_it_is_opening() {
    let (mut a, mut text) = asked(EDIT, MINE);
    a.scene(&mut text);
    assert_eq!(a.title(), "Edit theme extension");
    assert_eq!(control(&a, STATUS_ID).label, "Opening the extension…");
    // Improvement: the bottom bar's Cancel and Save wait for the workspace too; a bar that
    // ignored its rows' conditions drew them over the not-found message.
    let bar = [Lang::Cancel, Lang::Save].map(|l| lang_id(&a, l));
    assert!(
        !bar.iter().any(|&id| has(&a, id)),
        "the bar waits for the workspace"
    );
    assert!(!has(&a, MANAGE_ID));
    assert_eq!(focus_order(&mut a), [1]);
    // Painting again asks nothing more.
    a.scene(&mut text);
    assert!(last(&a, effect::OPEN).is_none());
}

#[test]
fn edit_reads_the_manifest_kotlin_unpacked() {
    let (a, _) = editing();
    let e = &a.extensions.editor;
    assert_eq!(e.load, Load::Ready);
    assert_eq!(e.workspace, WS);
    assert!(!e.changed);
    assert_eq!(e.draft.meta.id, MINE);
    assert_eq!(e.draft.meta.license, "apache-2.0");
    assert_eq!(theme_ids(&a), ["day", "night"]);
    let night = &e.draft.themes[1];
    assert!(night.night && !e.draft.themes[0].night);
    assert_eq!(night.flags, ["Vibrant", "High", "SPEC_2025"]);
    assert_eq!(night.path, "stylesheets/dark.json");
    assert_eq!(e.files, [(0, "a.ttf".into()), (1, "bg.png".into())]);
    assert_eq!(e.installed, ["org.florisboard.themes", MINE]);
    assert_eq!(e.defaults, ["TonalSpot", "Default", "SPEC_2021"]);
    // An extension taken up afresh needs no copy sent: Kotlin wrote the one it unpacked.
    let (mut b, _) = asked(EDIT, MINE);
    b.set_ext_editor(&ready(MINE, MANIFEST, false)).unwrap();
    let sync = last(&b, effect::SYNC).expect("the theme editor's copy");
    assert_eq!(sync[1], WS);
    assert_eq!(sync[2], b.extensions.editor.draft.raw_json());
}

#[test]
fn create_starts_a_draft_named_by_the_time_and_hands_kotlin_a_copy() {
    let (mut a, mut text) = asked(CREATE, "");
    a.set_ext_editor(&ready("", "", false)).unwrap();
    let id = format!("local.themes.{NOW}");
    let e = &a.extensions.editor;
    assert_eq!(e.draft.meta.id, id);
    assert_eq!(e.draft.meta.version, "0.0.0");
    assert_eq!(e.draft.meta.title, "My themes");
    assert_eq!(e.draft.meta.maintainers, ["Local"]);
    assert_eq!(e.draft.meta.license, "(none specified)");
    assert!(e.draft.themes.is_empty() && !e.changed);
    let sync = last(&a, effect::SYNC).expect("the theme editor's copy");
    assert_eq!(sync[1], WS);
    assert!(
        sync[2].contains(&format!("\"id\": \"{id}\"")),
        "{}",
        sync[2]
    );
    a.scene(&mut text);
    assert_eq!(a.title(), "Create theme extension");
    assert_eq!(control(&a, NONE_ID).label, "No bundled components.");
}

/// Improvement: Compose showed ExtensionNotFoundScreen for a missing id and a stack trace when
/// the archive would not unpack; each case is said here in the user's language.
#[test]
fn why_it_cannot_open_is_said_in_words() {
    for (how, said) in [
        (
            outcome::MISSING,
            "No extension with ID \"my.themes\" could be found.",
        ),
        (outcome::FAILED, "It could not be unpacked."),
        (outcome::GONE, "Its draft is gone."),
    ] {
        let (mut a, mut text) = asked(EDIT, MINE);
        a.set_ext_editor(&head(MINE, how, "", false)).unwrap();
        a.scene(&mut text);
        assert_eq!(control(&a, STATUS_ID).label, said, "{how}");
        assert!(!has(&a, MANAGE_ID) && !has(&a, ADD_ID), "{how}");
        assert!(!has(&a, lang_id(&a, Lang::Save)), "{how}");
        assert!(a.extensions.editor.workspace.is_empty(), "{how}");
        // Back leaves at once: there is nothing to lose.
        a.back();
        assert_eq!(top(&a), "settings/home", "{how}");
    }
}

#[test]
fn a_type_the_editor_cannot_make_is_not_found_without_asking_kotlin() {
    let (mut a, mut text) = fresh();
    a.navigate("ext/edit/null?create=ime.extension.keyboard");
    assert!(last(&a, effect::OPEN).is_none());
    a.scene(&mut text);
    let said = "No extension with ID \"null\" could be found.";
    assert_eq!(control(&a, STATUS_ID).label, said);
}

/// Improvement: a workspace Kotlin opened for an editor that has since gone (the user left
/// before it answered) is closed, where Compose's stayed in the cache.
#[test]
fn a_report_the_editor_did_not_ask_for_closes_its_workspace() {
    let (mut a, _) = editing();
    let mut other = ready("other.themes", MANIFEST, false);
    other[row::WORKSPACE] = "ws-2".into();
    a.set_ext_editor(&other).unwrap();
    assert_eq!(a.effects, [[effect::CLOSE, "ws-2"]]);
    assert_eq!(a.extensions.editor.workspace, WS);
    assert_eq!(theme_ids(&a), ["day", "night"]);
    // Left before Kotlin answered: the late workspace is closed too.
    let (mut b, _) = asked(EDIT, MINE);
    b.back();
    b.effects.clear();
    b.set_ext_editor(&ready(MINE, MANIFEST, false)).unwrap();
    assert_eq!(b.effects, [[effect::CLOSE, WS]]);
}

#[test]
fn a_broken_report_is_refused() {
    let (mut a, _) = asked(EDIT, MINE);
    assert!(a.set_ext_editor(&[]).is_err());
    assert!(a.set_ext_editor(&rest()).is_err(), "no workspace row first");
    let mut bad = ready(MINE, MANIFEST, false);
    bad[row::DETAIL] = "TonalSpot".into();
    assert!(a.set_ext_editor(&bad).is_err(), "flags");
    assert!(
        a.set_ext_editor(&ready(MINE, "{", false)).is_err(),
        "manifest"
    );
    assert_eq!(a.extensions.editor.load, Load::Waiting);
}

#[test]
fn the_editor_is_laid_out_as_compose_had_it() {
    let (mut a, mut text) = editing();
    a.scene(&mut text);
    let titles: Vec<String> = (0..3).map(|k| control(&a, MANAGE_ID + k).label).collect();
    assert_eq!(
        titles,
        [
            "Manage meta data",
            "Manage dependencies",
            "Manage archive files"
        ]
    );
    let add = control(&a, ADD_ID);
    assert_eq!((add.label.as_str(), add.role), ("Add", "button"));
    let header = control(&a, HEADER_ID);
    assert_eq!(header.label, "Bundled themes");
    assert_eq!(control(&a, COMP_ID).label, "Day");
    assert_eq!(control(&a, COMP_ID + 1).label, "my.themes:day");
    let day = "authors = [Me]\nisNightTheme = false\nstylesheetPath = stylesheets/day.json";
    assert_eq!(control(&a, COMP_ID + 2).label, day);
    assert_eq!(control(&a, COMP_ID + 8).label, "Night");
    let night = "authors = [Me]\nisNightTheme = true\nstylesheetPath = stylesheets/dark.json";
    assert_eq!(control(&a, COMP_ID + 10).label, night);
    for c in [0, 8] {
        let [delete, edit] = [3, 4].map(|k| control(&a, COMP_ID + c + k));
        assert_eq!((delete.label.as_str(), delete.role), ("Delete", "button"));
        assert_eq!((edit.label.as_str(), edit.role), ("Edit", "button"));
        assert!(delete.bounds.left < edit.bounds.left, "Delete at the start");
    }
    // Down the screen: the Manage box, the header with Add, then the boxes in order.
    let tops = [MANAGE_ID + 2, ADD_ID, COMP_ID + 1, COMP_ID + 9].map(|id| control(&a, id).bounds);
    assert!(tops.windows(2).all(|w| w[0].bottom <= w[1].top), "{tops:?}");
    let [cancel, save] = [Lang::Cancel, Lang::Save].map(|l| control(&a, lang_id(&a, l)));
    assert_eq!([cancel.label, save.label], ["Cancel", "Save"]);
}

#[test]
fn focus_runs_down_the_screen_then_the_bottom_bar() {
    let (mut a, mut text) = editing();
    a.scene(&mut text);
    let bar = [Lang::Cancel, Lang::Save].map(|l| lang_id(&a, l));
    let mut order = vec![1];
    order.extend((0..3).map(|k| MANAGE_ID + k));
    order.extend([ADD_ID, COMP_ID + 3, COMP_ID + 4, COMP_ID + 11, COMP_ID + 12]);
    order.extend(bar);
    assert_eq!(focus_order(&mut a), order);
}

#[test]
fn a_components_edit_opens_the_theme_editor_on_it() {
    let (mut a, mut text) = editing();
    assert!(tap(&mut a, &mut text, COMP_ID + 12));
    assert_eq!(a.effects, [[effect::THEME, WS, "1"]]);
    assert!(
        !a.extensions.editor.changed,
        "opening it changes nothing yet"
    );
}

/// Improvement: Compose's Delete removed the component without marking the extension changed,
/// so Back left without asking and the deletion was lost; here it is a change.
#[test]
fn a_components_delete_drops_it_and_counts_as_a_change() {
    let (mut a, mut text) = editing();
    assert!(tap(&mut a, &mut text, COMP_ID + 3));
    assert_eq!(theme_ids(&a), ["night"]);
    assert!(a.extensions.editor.changed);
    let sync = last(&a, effect::SYNC).expect("the copy follows");
    assert!(!sync[2].contains("\"day\""), "{}", sync[2]);
    assert!(tap(&mut a, &mut text, COMP_ID + 3));
    assert!(theme_ids(&a).is_empty());
    assert_eq!(control(&a, NONE_ID).label, "No bundled components.");
    assert!(!has(&a, COMP_ID + 3));
}
