//! The theme manager (stage4-theme-editor.md, part 1): the installed theme extensions Kotlin
//! reports, grouped and sorted as ThemeManagerScreen had them; choosing the day and the night
//! theme; the keyboard's preview while the manager is open; and the hand-offs to Compose's
//! extension screens it adds: Create, Import, View details and Edit.
use render::{
    settings_ui::{
        App, Control,
        themes::{self, CREATE, DAY, IMPORT, NIGHT, manager},
    },
    text::Text,
};
use wire::settings_ui::themes::{kind, row};

const FLORIS: &str = "org.florisboard.themes";
const MINE: &str = "my.themes";

/// A report row: an extension, or a theme of one.
fn ext(id: &str, title: &str, deletable: bool) -> Vec<String> {
    report_row(kind::EXTENSION, id, "", title, false, deletable)
}
fn theme(ext: &str, id: &str, label: &str, night: bool) -> Vec<String> {
    report_row(kind::THEME, ext, id, label, night, false)
}
fn report_row(k: &str, e: &str, c: &str, label: &str, night: bool, del: bool) -> Vec<String> {
    let mut f = vec![String::new(); row::SIZE];
    f[row::KIND] = k.into();
    f[row::EXTENSION] = e.into();
    f[row::COMPONENT] = c.into();
    f[row::LABEL] = label.into();
    f[row::NIGHT] = wire::flag::encode(night).into();
    f[row::DELETABLE] = wire::flag::encode(del).into();
    f
}
/// The index as a fresh install has it, out of label order, then one theme made here.
fn report() -> Vec<String> {
    [
        ext(FLORIS, "FlorisBoard themes", false),
        theme(FLORIS, "floris_night", "Floris Night", true),
        theme(FLORIS, "floris_day", "Floris Day", false),
        ext(MINE, "My themes", true),
        theme(MINE, "sunny", "Sunny", false),
    ]
    .concat()
}
fn text() -> Text {
    Text::new(vec![
        include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ])
}
/// The manager at `route`, the report answered and painted, its effects cleared.
fn app(route: &str) -> (App, Text) {
    let mut a = App {
        config: [1080.0, 6000.0, 2.625, 1.0],
        ..App::default()
    };
    for (k, v) in [
        ("settings__theme_manager__title_day", "Select day theme"),
        ("ext__list__view_details", "View details"),
        ("action__edit", "Edit"),
        ("pref__theme__day", "Day theme"),
        ("pref__theme__night", "Night theme"),
        ("@icon:light_mode", "M0 0h24v24H0z"),
        ("@icon:dark_mode", "M0 0h24v24H0z"),
    ] {
        a.strings.insert(k.into(), v.into());
    }
    a.prefs
        .insert("theme__day_theme_id".into(), format!("{FLORIS}:floris_day"));
    a.prefs.insert(
        "theme__night_theme_id".into(),
        format!("{FLORIS}:floris_night"),
    );
    a.navigate(route);
    assert_eq!(a.effects[0], ["themes"]);
    a.set_themes(&report()).unwrap();
    let mut text = text();
    a.scene(&mut text);
    a.effects.clear();
    (a, text)
}
fn control(a: &App, id: i32) -> Control {
    a.controls.iter().find(|c| c.id == id).cloned().unwrap()
}
fn tap(a: &mut App, text: &mut Text, id: i32) {
    a.scene(text);
    assert!(a.activate(id), "{id}");
    a.scene(text);
}

#[test]
fn a_report_is_grouped_by_extension_in_index_order_and_sorted_by_label() {
    let rows = [
        ext("b", "B", false),
        theme("b", "z", "Zed", false),
        theme("b", "a", "apple", false),
        theme("b", "e", "Éclair", false),
        theme("b", "y", "Zed", true),
        ext("a", "A", true),
        theme("a", "one", "One", false),
        // A repeated id keeps its first place and takes the later extension, as `buildMap` did.
        ext("b", "B again", true),
        theme("b", "q", "Q", false),
    ]
    .concat();
    let e = themes::parse(&rows).unwrap();
    let ids: Vec<_> = e
        .iter()
        .map(|x| (x.id.as_str(), x.title.as_str()))
        .collect();
    assert_eq!(ids, [("b", "B again"), ("a", "A")]);
    assert!(e[0].deletable && e[1].deletable);
    assert_eq!(e[0].themes.len(), 1);
    let rows = [
        ext("b", "B", false),
        theme("b", "z", "Zed", false),
        theme("b", "a", "apple", false),
        theme("b", "e", "Éclair", false),
        theme("b", "y", "Zed", true),
    ]
    .concat();
    let labels: Vec<_> = themes::parse(&rows).unwrap()[0]
        .themes
        .iter()
        .map(|t| (t.id.clone(), t.night))
        .collect();
    // String.compareTo: capitals before lower case before accented, equal labels kept in order.
    let want = [("z", false), ("y", true), ("a", false), ("e", false)];
    assert_eq!(labels, want.map(|(i, n)| (i.to_string(), n)));
}

#[test]
fn a_malformed_report_is_refused_and_the_last_good_one_kept() {
    let (mut a, _) = app(DAY);
    let mut short = report();
    short.pop();
    let orphan = theme(FLORIS, "x", "X", false);
    let wrong = [ext(FLORIS, "F", false), theme(MINE, "x", "X", false)].concat();
    let unknown = report_row("nope", FLORIS, "", "", false, false);
    for bad in [short, orphan, wrong, unknown] {
        assert!(a.set_themes(&bad).is_err());
    }
    assert_eq!(a.theme_extensions().len(), 2);
    // An empty index is a report too: nothing installed.
    a.set_themes(&[]).unwrap();
    assert!(a.theme_extensions().is_empty());
}

#[test]
fn the_manager_opens_natively_with_the_preview_field_and_previews_the_chosen_theme() {
    let mut a = App::default();
    a.prefs.insert("theme__day_theme_id".into(), "x:day".into());
    a.prefs
        .insert("theme__night_theme_id".into(), "x:night".into());
    for (route, title, chosen) in [
        (DAY, "settings__theme_manager__title_day", "x:day"),
        (NIGHT, "settings__theme_manager__title_night", "x:night"),
    ] {
        a.effects.clear();
        a.navigate(route);
        assert_eq!(a.stack.last().map(String::as_str), Some(route));
        let screen = a.screen().unwrap();
        assert!(screen.preview && screen.title == title);
        assert_eq!(a.effects, [vec!["themes"], vec!["themePreview", chosen]]);
        // Back: the keyboard shows its own theme again, and a report is asked for afresh.
        a.effects.clear();
        a.back();
        assert_eq!(a.effects, [vec!["themePreview", ""]]);
        a.set_themes(&report()).unwrap();
    }
    // Leaving one manager for another previews the other's theme, without a clear between, and
    // does not ask again for the report still on its way.
    a.navigate(DAY);
    a.effects.clear();
    a.navigate(NIGHT);
    assert_eq!(a.effects, [vec!["themePreview", "x:night"]]);
}

#[test]
fn each_extension_is_a_box_titled_with_its_name_over_its_id_listing_its_themes() {
    let (a, _) = app(DAY);
    let title = control(&a, manager::theme_id(0, 0));
    let id = control(&a, manager::theme_id(0, 1));
    assert_eq!(
        (title.label.as_str(), id.label.as_str()),
        ("FlorisBoard themes", FLORIS)
    );
    assert!(title.bounds.bottom <= id.bounds.top + 1.0);
    let (day, night) = (
        control(&a, manager::theme_id(0, 4)),
        control(&a, manager::theme_id(0, 5)),
    );
    assert_eq!(
        (day.label.as_str(), night.label.as_str()),
        ("Floris Day", "Floris Night")
    );
    assert_eq!(
        (day.role, day.selected, night.selected),
        ("radio", true, false)
    );
    // A 56 dp ListItem the box's width, inside the 16 dp screen margin.
    assert_eq!(day.bounds.height(), (56.0 * 2.625_f32).round().max(147.0));
    assert!(day.bounds.left >= 42.0 && day.bounds.right <= 1080.0 - 42.0);
    assert!(night.bounds.top >= day.bounds.bottom);
    let mine = control(&a, manager::theme_id(1, 4));
    assert_eq!((mine.label.as_str(), mine.selected), ("Sunny", false));
    assert!(mine.bounds.top > night.bounds.bottom);
}

#[test]
fn talkback_hears_whether_each_theme_is_a_day_or_a_night_theme() {
    let (a, _) = app(DAY);
    let day = control(&a, manager::theme_id(0, 4));
    let night = control(&a, manager::theme_id(0, 5));
    assert_eq!(
        (day.summary.as_str(), night.summary.as_str()),
        ("Day theme", "Night theme")
    );
}

#[test]
fn each_theme_shows_a_faded_day_or_night_icon_at_its_end() {
    let (mut a, mut text) = app(DAY);
    let panel = a.scene(&mut text).panel.unwrap();
    let night = control(&a, manager::theme_id(0, 5)).bounds;
    let icon = panel
        .items
        .iter()
        .filter(|i| i.element == "settings-icon")
        .find(|i| i.bounds.top >= night.top && i.bounds.bottom <= night.bottom)
        .unwrap();
    let size = (18.0_f32 * 2.625).round();
    assert_eq!(icon.bounds.width(), size);
    assert_eq!(icon.bounds.right, night.right - 16.0 * 2.625);
    assert_eq!(icon.icon_style.foreground >> 24, 0x8f);
    assert_eq!(
        icon.icon_style.foreground & 0xffffff,
        a.colors[2] & 0xffffff
    );
}

#[test]
fn a_tap_writes_the_day_theme_and_previews_it() {
    let (mut a, mut text) = app(DAY);
    tap(&mut a, &mut text, manager::theme_id(1, 4));
    let chosen = format!("{MINE}:sunny");
    assert_eq!(
        a.effects,
        [
            vec![
                "write".to_string(),
                "theme__day_theme_id".into(),
                chosen.clone()
            ],
            vec!["themePreview".into(), chosen],
        ]
    );
    assert!(control(&a, manager::theme_id(1, 4)).selected);
    assert!(!control(&a, manager::theme_id(0, 4)).selected);
    // The theme already chosen is written again, as Compose's click did, and not previewed twice.
    a.effects.clear();
    tap(&mut a, &mut text, manager::theme_id(1, 4));
    assert_eq!(a.effects.len(), 1);
    assert_eq!(a.effects[0][..2], ["write", "theme__day_theme_id"]);
}

#[test]
fn the_night_manager_writes_the_night_theme() {
    let (mut a, mut text) = app(NIGHT);
    assert!(control(&a, manager::theme_id(0, 5)).selected);
    tap(&mut a, &mut text, manager::theme_id(0, 4));
    let chosen = format!("{FLORIS}:floris_day");
    assert_eq!(
        a.effects[0],
        ["write", "theme__night_theme_id", chosen.as_str()]
    );
    assert_eq!(a.value("theme__day_theme_id"), chosen);
}

#[test]
fn a_new_report_while_open_redraws_the_boxes() {
    let (mut a, mut text) = app(DAY);
    let more = [
        report(),
        ext("new.ext", "New", true),
        theme("new.ext", "t", "T", true),
    ]
    .concat();
    a.set_themes(&more).unwrap();
    a.scene(&mut text);
    assert_eq!(control(&a, manager::theme_id(2, 0)).label, "New");
    assert_eq!(control(&a, manager::theme_id(2, 4)).summary, "Night theme");
    // The chosen theme's extension gone: no radio button is on.
    a.set_themes(
        &[
            ext(MINE, "My themes", true),
            theme(MINE, "sunny", "Sunny", false),
        ]
        .concat(),
    )
    .unwrap();
    a.scene(&mut text);
    assert!(
        a.controls
            .iter()
            .filter(|c| c.role == "radio")
            .all(|c| !c.selected)
    );
}

#[test]
fn create_opens_the_extension_editor_and_import_opens_the_theme_importer() {
    let (mut a, mut text) = app(DAY);
    tap(&mut a, &mut text, manager::CREATE_ID);
    // The previews stop, and the editor asks Kotlin for a new theme extension's workspace.
    assert_eq!(
        a.effects,
        [vec!["themePreview", ""], vec!["extEditor", "", "", ""]]
    );
    assert_eq!(a.stack.last().map(String::as_str), Some(CREATE));
    assert_eq!(CREATE, "ext/edit/null?create=ime.extension.theme");
    a.back();
    a.scene(&mut text);
    a.effects.clear();
    tap(&mut a, &mut text, manager::IMPORT_ID);
    assert!(a.effects.iter().all(|e| e[0] != "compose"));
    assert_eq!(a.stack.last().map(String::as_str), Some(IMPORT));
    assert_eq!(IMPORT, "ext/import/EXT_THEME");
    a.back();
    a.scene(&mut text);
    let create = control(&a, manager::CREATE_ID);
    assert!(create.bounds.bottom <= control(&a, manager::theme_id(0, 0)).bounds.top);
}

#[test]
fn an_extensions_name_id_and_view_details_open_its_page_and_edit_its_editor() {
    let (mut a, mut text) = app(DAY);
    for (id, route) in [
        (manager::theme_id(0, 0), format!("ext/view/{FLORIS}")),
        (manager::theme_id(0, 1), format!("ext/view/{FLORIS}")),
        (manager::theme_id(0, 2), format!("ext/view/{FLORIS}")),
        (manager::theme_id(1, 2), format!("ext/view/{MINE}")),
        (manager::theme_id(1, 3), format!("ext/edit/{MINE}")),
    ] {
        a.effects.clear();
        tap(&mut a, &mut text, id);
        // An extension's page and its editor are native now.
        assert_eq!(a.stack.last(), Some(&route), "{id}");
        assert!(a.effects.iter().all(|e| e[0] != "compose"), "{id}");
        a.back();
        a.scene(&mut text);
    }
    assert_eq!(control(&a, manager::theme_id(0, 2)).label, "View details");
    assert_eq!(control(&a, manager::theme_id(1, 3)).label, "Edit");
    // A theme that came with the app has no Edit: the extension editor cannot save it.
    assert!(a.controls.iter().all(|c| c.id != manager::theme_id(0, 3)));
}

#[test]
fn the_keyboard_moves_through_actions_then_each_box_in_order() {
    let (mut a, mut text) = app(DAY);
    let i = |e, p| manager::theme_id(e, p);
    let want = [
        vec![manager::CREATE_ID, manager::IMPORT_ID],
        vec![i(0, 0), i(0, 1), i(0, 4), i(0, 5), i(0, 2)],
        vec![i(1, 0), i(1, 1), i(1, 4), i(1, 2), i(1, 3)],
    ]
    .concat();
    let mut got = vec![];
    // Back first, then the screen's own controls, then the preview field's.
    a.focus_next(false);
    for _ in 0..want.len() {
        a.focus_next(false);
        a.scene(&mut text);
        got.push(a.focus);
    }
    assert_eq!(got, want);
}

#[test]
fn a_restored_manager_asks_for_the_themes_and_previews_again() {
    let (mut a, _) = app(DAY);
    a.themes_restored();
    assert_eq!(
        a.effects,
        [
            vec!["themes".to_string()],
            vec!["themePreview".into(), format!("{FLORIS}:floris_day")]
        ]
    );
    // Restored elsewhere: nothing to ask, nothing to preview.
    let mut b = App::default();
    b.navigate("settings/theme");
    b.effects.clear();
    b.themes_restored();
    assert!(b.effects.is_empty());
}
