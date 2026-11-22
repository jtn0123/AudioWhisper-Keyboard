//! The sheets ExtensionEditScreen switched between, each a screen here: the meta data, the
//! dependencies and Create component.
use super::common::*;

/// The editor's Manage row `k` tapped: its sheet on top.
fn manage(a: &mut App, text: &mut Text, k: i32) {
    assert!(tap(a, text, MANAGE_ID + k));
    a.effects.clear();
}
/// Create component, open from the editor.
fn adding() -> (App, Text) {
    let (mut a, mut text) = editing();
    assert!(tap(&mut a, &mut text, ADD_ID));
    assert_eq!(top(&a), COMPONENT_ADD);
    (a, text)
}
fn create_id(a: &App) -> i32 {
    lang_id(a, Lang::Save)
}

#[test]
fn the_meta_data_opens_filled_in_with_the_installed_id_read_only() {
    let (mut a, mut text) = editing();
    manage(&mut a, &mut text, 0);
    assert_eq!(top(&a), META);
    assert_eq!(a.title(), "Manage meta data");
    let id = control(&a, FIXED_ID);
    assert_eq!((id.label.as_str(), id.summary.as_str()), ("ID", MINE));
    assert!(!id.enabled && id.role == "edit");
    let form = a.extensions.editor.meta_form.clone().unwrap();
    assert_eq!(form.fields[1..3], ["1.0.0", "My themes"]);
    assert_eq!(form.fields[7..], ["Me", "apache-2.0"]);
    assert_eq!(control(&a, 1001).role, "edit");
    let bar = [Lang::Cancel, Lang::Save].map(|l| control(&a, lang_id(&a, l)).label);
    assert_eq!(bar, ["Cancel", "Apply"]);
    // The read-only id takes no focus, as Compose's disabled field took none.
    let mut order = vec![1];
    order.extend(1001..=1010);
    assert_eq!(focus_order(&mut a), order);
}

#[test]
fn apply_changes_the_meta_and_counts_as_a_change() {
    let (mut a, mut text) = editing();
    manage(&mut a, &mut text, 0);
    type_into(&mut a, &mut text, Edit::Meta(1), " 1.0.1 ");
    type_into(&mut a, &mut text, Edit::Meta(4), "day, , night");
    type_into(&mut a, &mut text, Edit::Meta(5), "https://example.org");
    assert!(confirm(&mut a, &mut text));
    assert_eq!(top(&a), EDIT);
    let e = &a.extensions.editor;
    assert_eq!(e.draft.meta.version, "1.0.1");
    assert_eq!(
        e.draft.meta.keywords,
        Some(vec!["day".into(), "night".into()])
    );
    assert_eq!(
        e.draft.meta.homepage.as_deref(),
        Some("https://example.org")
    );
    assert_eq!(e.draft.meta.id, MINE);
    assert!(e.changed && e.meta_form.is_none());
    let sync = last(&a, effect::SYNC).expect("the copy follows");
    assert!(sync[2].contains("\"version\": \"1.0.1\""));
}

#[test]
fn apply_with_nothing_changed_is_no_change() {
    let (mut a, mut text) = editing();
    manage(&mut a, &mut text, 0);
    assert!(confirm(&mut a, &mut text));
    assert_eq!(top(&a), EDIT);
    assert!(!a.extensions.editor.changed);
    assert!(last(&a, effect::SYNC).is_none());
}

#[test]
fn a_refused_field_says_why_under_it_and_cancel_keeps_the_meta() {
    let (mut a, mut text) = editing();
    manage(&mut a, &mut text, 0);
    type_into(&mut a, &mut text, Edit::Meta(1), "");
    assert!(confirm(&mut a, &mut text));
    assert_eq!(top(&a), META, "Apply waits for the field");
    assert!(!a.ext_error(Edit::Meta(1)).is_empty());
    assert!(
        a.ext_error(Edit::Meta(2)).is_empty(),
        "a good field says nothing"
    );
    assert!(cancel(&mut a, &mut text));
    assert_eq!(top(&a), EDIT);
    assert_eq!(a.extensions.editor.draft.meta.version, "1.0.0");
    assert!(!a.extensions.editor.changed);
}

#[test]
fn a_new_extension_can_change_its_id() {
    let (mut a, mut text) = creating();
    manage(&mut a, &mut text, 0);
    assert!(!has(&a, FIXED_ID));
    type_into(&mut a, &mut text, Edit::Meta(0), "my.new.themes");
    assert!(confirm(&mut a, &mut text));
    assert_eq!(a.extensions.editor.draft.meta.id, "my.new.themes");
    let [.., name, _] = {
        assert!(confirm(&mut a, &mut text));
        work(&a).unwrap()
    };
    assert_eq!(name, "my.new.themes.flex");
}

#[test]
fn the_dependencies_are_listed_or_said_to_be_none() {
    let (mut a, mut text) = editing();
    manage(&mut a, &mut text, 1);
    assert_eq!(top(&a), DEPENDENCIES);
    assert_eq!(a.title(), "Manage dependencies");
    assert_eq!(control(&a, 420_000).label, "Dependencies note.");
    assert_eq!(control(&a, NO_DEPENDENCIES_ID).label, "No dependencies.");

    let with = manifest_with(
        "\n  \"themes\": [",
        "\n  \"dependencies\": [\n    \"org.florisboard.themes\"\n  ],\n  \"themes\": [",
    );
    let (mut b, mut text) = asked(EDIT, MINE);
    b.set_ext_editor(&ready(MINE, &with, false)).unwrap();
    manage(&mut b, &mut text, 1);
    assert_eq!(control(&b, DEPENDENCY_ID).label, "org.florisboard.themes");
    assert!(!has(&b, NO_DEPENDENCIES_ID));
    // Save writes them back where kotlinx put them.
    b.back();
    assert_eq!(b.extensions.editor.draft.json().unwrap(), with);
}

#[test]
fn create_component_offers_its_own_themes_then_the_others() {
    let (mut a, mut text) = adding();
    a.scene(&mut text);
    assert_eq!(a.title(), "Create theme");
    let [existing, empty] = [0, 1].map(|k| control(&a, FROM_ID + k));
    assert_eq!(
        [existing.label.as_str(), empty.label.as_str()],
        ["From existing", "Empty"]
    );
    assert!(existing.selected && !empty.selected && existing.role == "radio");
    let offered: Vec<(String, String)> = (0..3)
        .map(|k| control(&a, SOURCE_ID + k))
        .map(|c| (c.label, c.summary))
        .collect();
    let name = |a: &str, b: &str| (a.to_owned(), b.to_owned());
    assert_eq!(
        offered,
        [
            name("Day", "my.themes:day"),
            name("Night", "my.themes:night"),
            name("Floris Day", "org.florisboard.themes:floris_day"),
        ]
    );
    assert!(
        !has(&a, SOURCE_ID + 3),
        "its own night theme from the index is not offered twice"
    );
    assert_eq!(control(&a, create_id(&a)).label, "Create");
    assert!(!control(&a, create_id(&a)).enabled, "nothing chosen yet");
    assert!(tap(&mut a, &mut text, SOURCE_ID + 2));
    assert!(control(&a, SOURCE_ID + 2).selected);
    assert!(control(&a, create_id(&a)).enabled);
}

#[test]
fn a_copy_from_another_extension_keeps_its_meta_once_its_stylesheet_is_in() {
    let (mut a, mut text) = adding();
    assert!(tap(&mut a, &mut text, SOURCE_ID + 2));
    assert!(confirm(&mut a, &mut text));
    let sheet = "stylesheets/floris_day.json";
    let packet = work(&a).expect("Kotlin copies the stylesheet");
    let floris = "org.florisboard.themes";
    assert_eq!(
        packet,
        [job::EXT_COMPONENT_COPY, WS, "", sheet, sheet, floris]
    );
    assert_eq!(theme_ids(&a), ["day", "night"], "not before the copy is in");
    a.file_report(&done_report(job::EXT_COMPONENT_COPY, result::OK, sheet, ""))
        .unwrap();
    assert_eq!(top(&a), EDIT);
    assert_eq!(theme_ids(&a), ["day", "night", "floris_day"]);
    let copy = &a.extensions.editor.draft.themes[2];
    assert_eq!(copy.label, "Floris Day");
    assert_eq!(copy.authors, ["patrickgold", "jtn"]);
    assert_eq!(copy.flags, ["Neutral", "Medium", "SPEC_2021"]);
    assert!(!copy.night && copy.path.is_empty());
    assert!(a.extensions.editor.changed);
}

/// Improvement: Compose's copy of a component of the same extension took the default Material
/// You flags, and threw on one whose stylesheet did not exist yet; here the copy keeps the flags
/// it had, and Kotlin copies a stylesheet that is not there as none.
#[test]
fn a_copy_of_its_own_theme_gets_a_new_id_and_keeps_its_flags() {
    let (mut a, mut text) = adding();
    assert!(tap(&mut a, &mut text, SOURCE_ID + 1));
    assert!(confirm(&mut a, &mut text));
    let packet = work(&a).unwrap();
    let to = "stylesheets/night_1.json";
    assert_eq!(
        packet,
        [
            job::EXT_COMPONENT_COPY,
            WS,
            "",
            "stylesheets/dark.json",
            to,
            ""
        ]
    );
    a.file_report(&done_report(job::EXT_COMPONENT_COPY, result::OK, to, ""))
        .unwrap();
    assert_eq!(theme_ids(&a), ["day", "night", "night_1"]);
    let copy = &a.extensions.editor.draft.themes[2];
    assert_eq!(copy.flags, ["Vibrant", "High", "SPEC_2025"]);
    assert!(copy.night);
}

#[test]
fn a_failed_or_lost_copy_adds_nothing() {
    let (mut a, mut text) = adding();
    assert!(tap(&mut a, &mut text, SOURCE_ID + 2));
    assert!(confirm(&mut a, &mut text));
    a.file_report(&done_report(
        job::EXT_COMPONENT_COPY,
        result::FAILED,
        "",
        "",
    ))
    .unwrap();
    assert_eq!(toast(&a).as_deref(), Some("Copy failed."));
    assert_eq!(top(&a), COMPONENT_ADD);
    assert_eq!(theme_ids(&a), ["day", "night"]);
    assert!(confirm(&mut a, &mut text));
    a.file_report(&idle()).unwrap();
    assert_eq!(toast(&a).as_deref(), Some("Interrupted."));
    let ok = done_report(job::EXT_COMPONENT_COPY, result::OK, "", "");
    a.file_report(&ok).unwrap();
    assert_eq!(
        theme_ids(&a),
        ["day", "night"],
        "a late report adds nothing"
    );
    assert!(!a.extensions.editor.changed);
}

/// Improvement: Compose said a taken id in an English toast after Create; it is said under the
/// field here.
#[test]
fn an_empty_component_checks_its_fields_and_refuses_a_taken_id() {
    let (mut a, mut text) = adding();
    assert!(tap(&mut a, &mut text, FROM_ID + 1));
    assert!(control(&a, FROM_ID + 1).selected);
    assert!(!has(&a, SOURCE_ID));
    assert_eq!(control(&a, 420_002).label, "Empty warning.");
    assert!(!control(&a, create_id(&a)).enabled, "the fields are blank");
    type_into(&mut a, &mut text, Edit::New(0), "day");
    type_into(&mut a, &mut text, Edit::New(1), " Dawn ");
    type_into(&mut a, &mut text, Edit::New(2), "Me, , You");
    assert!(confirm(&mut a, &mut text));
    assert_eq!(top(&a), COMPONENT_ADD);
    assert_eq!(a.ext_error(Edit::New(0)), "The ID is taken.");
    type_into(&mut a, &mut text, Edit::New(0), "dawn");
    assert!(confirm(&mut a, &mut text));
    assert_eq!(top(&a), EDIT);
    assert_eq!(theme_ids(&a), ["day", "night", "dawn"]);
    let new = &a.extensions.editor.draft.themes[2];
    assert_eq!((new.label.as_str(), new.night), ("Dawn", true));
    assert_eq!(new.authors, ["Me", "You"]);
    assert_eq!(new.flags, ["TonalSpot", "Default", "SPEC_2021"]);
    assert!(a.extensions.editor.changed);
    assert!(last(&a, effect::SYNC).unwrap()[2].contains("\"dawn\""));
}

#[test]
fn cancel_leaves_create_component_with_nothing_added() {
    let (mut a, mut text) = adding();
    assert!(tap(&mut a, &mut text, SOURCE_ID + 2));
    assert!(cancel(&mut a, &mut text));
    assert_eq!(top(&a), EDIT);
    assert!(a.extensions.editor.comp_form.is_none());
    assert_eq!(theme_ids(&a), ["day", "night"]);
    assert!(!a.extensions.editor.changed);
}
