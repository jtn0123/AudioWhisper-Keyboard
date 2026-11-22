//! The addon home, the three extension lists and Check for Updates: which boxes each shows, what
//! each box holds, where each tap goes, the update link, and the index Kotlin reports.
use super::common::extensions::{HOME, IMPORT, KEYBOARDS, PACKS, STORE, THEMES, UPDATES};
use super::common::*;
use boxes::{CREATE_ID, IMPORT_ID, STORE_ID, STORE_NOTE_ID, UPDATE_ID, UPDATE_NOTE_ID, ext_id};
use wire::settings_ui::extensions::kind;

#[test]
fn the_home_shows_the_store_and_update_boxes_then_each_list() {
    let (mut a, mut text) = app(HOME);
    let order = [
        STORE_NOTE_ID,
        STORE_ID,
        UPDATE_NOTE_ID,
        UPDATE_ID,
        1002,
        1003,
        1004,
    ];
    let tops: Vec<f32> = order.iter().map(|id| control(&a, *id).bounds.top).collect();
    assert!(tops.windows(2).all(|w| w[0] <= w[1]), "{tops:?}");
    // Visit Addons Store at the start of its row, Import at the end, as ImportExtensionBox had.
    let (store, import) = (control(&a, STORE_ID), control(&a, IMPORT_ID));
    assert_eq!(store.bounds.top, import.bounds.top);
    assert_eq!(store.bounds.left, px(16.0) + px(6.0));
    assert_eq!(import.bounds.right, 1080.0 - px(16.0) - px(6.0));
    assert_eq!(
        (store.label.as_str(), import.label.as_str()),
        ("Visit Addons Store", "Import")
    );
    let lists = [
        (1002, THEMES, "Theme extensions"),
        (1003, KEYBOARDS, "Keyboard extensions"),
        (1004, PACKS, "Language pack extensions"),
    ];
    for (id, route, label) in lists {
        assert_eq!(control(&a, id).label, label);
        assert!(tap(&mut a, &mut text, id));
        assert_eq!(a.stack.last().map(String::as_str), Some(route));
        // The index came once; a list does not ask again.
        assert!(a.effects.is_empty(), "{:?}", a.effects);
        // A list opened from home has no boxes: Compose passed showUpdate = false.
        assert!(!has(&a, STORE_ID) && !has(&a, UPDATE_ID));
        a.back();
        a.scene(&mut text);
    }
}

#[test]
fn the_store_opens_in_the_browser_and_import_opens_the_importer() {
    let (mut a, mut text) = app(HOME);
    assert!(tap(&mut a, &mut text, STORE_ID));
    assert_eq!(url(&a).as_deref(), Some(STORE));
    assert_eq!(STORE, "https://beta.addons.florisboard.org/");
    assert!(tap(&mut a, &mut text, IMPORT_ID));
    assert_eq!(IMPORT, "ext/import/EXT_ANY");
    assert_eq!(a.stack.last().map(String::as_str), Some(IMPORT));
}

#[test]
fn each_list_shows_its_own_type_in_index_order() {
    for (route, titles) in [
        (THEMES, vec!["FlorisBoard themes", "My themes"]),
        (KEYBOARDS, vec!["FlorisBoard layouts"]),
        (PACKS, vec!["Han language pack"]),
    ] {
        let (a, _) = app(route);
        let shown: Vec<String> = (0..titles.len())
            .map(|e| control(&a, ext_id(e, 0)).label)
            .collect();
        assert_eq!(shown, titles, "{route}");
        assert!(!has(&a, ext_id(titles.len(), 0)), "{route}");
    }
}

#[test]
fn a_list_shows_the_store_and_update_boxes_only_when_its_route_asks() {
    for (query, shown) in [
        ("", false),
        ("?showUpdate=false", false),
        ("?showUpdate=true", true),
    ] {
        let route = format!("{THEMES}{query}");
        let (a, _) = app(&route);
        assert_eq!(a.stack.last(), Some(&route));
        assert_eq!(has(&a, STORE_ID), shown, "{route}");
        assert_eq!(has(&a, UPDATE_ID), shown, "{route}");
        assert_eq!(control(&a, ext_id(0, 0)).label, "FlorisBoard themes");
    }
    let (a, _) = app(UPDATES);
    assert!(has(&a, UPDATE_ID) && !has(&a, STORE_ID));
}

#[test]
fn search_for_updates_links_the_listed_extensions_ids_and_versions() {
    let all = "https://beta.addons.florisboard.org/check-updates#data={\
               \"org.florisboard.layouts\":\"0.1.0\",\"org.florisboard.themes\":\"0.2.0\",\
               \"my.themes\":\"1.0.0\",\"org.florisboard.han\":\"0.3.0\"}";
    let themes = "https://beta.addons.florisboard.org/check-updates#data={\
                  \"org.florisboard.themes\":\"0.2.0\",\"my.themes\":\"1.0.0\"}";
    for (route, link) in [
        (HOME, all),
        (UPDATES, all),
        ("ext/list/EXT_THEME?showUpdate=true", themes),
    ] {
        let (mut a, mut text) = app(route);
        assert_eq!(control(&a, UPDATE_ID).label, "Search for Updates");
        assert!(tap(&mut a, &mut text, UPDATE_ID));
        assert_eq!(url(&a).as_deref(), Some(link), "{route}");
    }
    // No extensions: an empty object, as Compose's loop wrote it.
    let none: [&extensions::Extension; 0] = [];
    assert_eq!(
        extensions::update_url(none),
        "https://beta.addons.florisboard.org/check-updates#data={}"
    );
}

/// Fix: `generateUpdateUrl` left out the comma after any extension `equals` to the list's last.
/// KeyboardExtension is a data class, so two keyboard extensions alike in every field ran
/// together and the store got malformed JSON. Here every pair is followed by a comma but the
/// final one.
#[test]
fn the_update_link_separates_every_pair_even_when_an_earlier_one_equals_the_last() {
    let twice = [
        report(),
        ext(MINE, extensions::THEME, "My themes", "1.0.0", true),
    ]
    .concat();
    let list = extensions::parse(&twice).unwrap();
    let link = extensions::update_url(&list);
    assert!(
        link.contains("\"my.themes\":\"1.0.0\",\"org.florisboard.han\""),
        "{link}"
    );
    assert!(link.ends_with(",\"my.themes\":\"1.0.0\"}"), "{link}");
    assert!(!link.contains("\"\""), "{link}");
}

#[test]
fn a_box_holds_the_title_id_description_and_view_details() {
    let (a, _) = app(THEMES);
    let (title, id, about) = (
        control(&a, ext_id(0, 0)),
        control(&a, ext_id(0, 1)),
        control(&a, ext_id(0, 2)),
    );
    assert_eq!(
        (title.label.as_str(), id.label.as_str()),
        ("FlorisBoard themes", FLORIS)
    );
    assert_eq!(
        (about.label.as_str(), about.role),
        ("The themes that ship with the app.", "text")
    );
    let view = control(&a, ext_id(0, 3));
    assert!(title.bounds.bottom <= id.bounds.top && id.bounds.bottom <= about.bounds.top);
    assert!(about.bounds.bottom <= view.bounds.top);
    assert_eq!(view.label, "View details");
    // The description is 16 dp in from the box, which is 16 dp in from the screen.
    assert_eq!(about.bounds.left, 2.0 * px(16.0));
}

/// Difference: Compose drew an empty Text for an extension without a description, a blank
/// line in its box; here the box closes up.
#[test]
fn a_box_without_a_description_takes_no_blank_line() {
    let (a, _) = app(THEMES);
    assert!(!has(&a, ext_id(1, 2)));
    let gap = |e| control(&a, ext_id(e, 3)).bounds.top - control(&a, ext_id(e, 1)).bounds.bottom;
    assert!(gap(1) < gap(0));
    assert_eq!(gap(1), px(4.0));
    let about = control(&a, ext_id(0, 2)).bounds;
    assert_eq!(gap(0) - gap(1), about.bottom - about.top + 2.0 * px(4.0));
    // An empty description is no description.
    let empty = [
        ext(MINE, extensions::THEME, "My themes", "1.0.0", true),
        text_row(kind::DESCRIPTION, MINE, ""),
    ];
    let (mut a, mut text) = fresh();
    a.navigate(THEMES);
    a.set_extensions(&empty.concat()).unwrap();
    a.scene(&mut text);
    assert!(!has(&a, ext_id(0, 2)));
    let gap = control(&a, ext_id(0, 3)).bounds.top - control(&a, ext_id(0, 1)).bounds.bottom;
    assert_eq!(gap, px(4.0));
}

/// Improvement: Compose drew Edit greyed out on an extension the app ships, which the
/// extension editor cannot save; here such a box has no Edit.
#[test]
fn a_built_in_extension_has_no_edit_button() {
    let (a, _) = app(THEMES);
    assert!(!has(&a, ext_id(0, 4)));
    let edit = control(&a, ext_id(1, 4));
    assert_eq!((edit.label.as_str(), edit.enabled), ("Edit", true));
    assert!(a.controls.iter().all(|c| c.enabled));
}

#[test]
fn the_name_id_and_view_details_open_the_page_and_edit_opens_the_editor() {
    for (id, route) in [
        (ext_id(0, 0), format!("ext/view/{FLORIS}")),
        (ext_id(0, 1), format!("ext/view/{FLORIS}")),
        (ext_id(0, 3), format!("ext/view/{FLORIS}")),
        (ext_id(1, 3), format!("ext/view/{MINE}")),
    ] {
        let (mut a, mut text) = app(THEMES);
        assert!(tap(&mut a, &mut text, id));
        assert_eq!(a.stack.last(), Some(&route));
        assert!(a.effects.is_empty());
    }
    let (mut a, mut text) = app(THEMES);
    assert!(tap(&mut a, &mut text, ext_id(1, 4)));
    assert_eq!(a.stack.last(), Some(&format!("ext/edit/{MINE}")));
}

/// Difference: Compose's floating Create extension button; here a box above the extensions,
/// as the theme manager has Create theme. Only the theme list has it.
#[test]
fn the_theme_list_offers_create_extension_above_the_extensions() {
    let (mut a, mut text) = app(THEMES);
    let create = control(&a, CREATE_ID);
    assert_eq!(create.label, "Create extension");
    assert!(create.bounds.bottom <= control(&a, ext_id(0, 0)).bounds.top);
    assert!(tap(&mut a, &mut text, CREATE_ID));
    let create = "ext/edit/null?create=ime.extension.theme";
    assert_eq!(a.stack.last().map(String::as_str), Some(create));
    for route in [KEYBOARDS, PACKS] {
        let (a, _) = app(route);
        assert!(!has(&a, CREATE_ID), "{route}");
    }
}

#[test]
fn the_focus_order_follows_the_boxes() {
    let (mut a, _) = app("ext/list/EXT_THEME?showUpdate=true");
    let want = [
        1,
        STORE_ID,
        IMPORT_ID,
        UPDATE_ID,
        CREATE_ID,
        ext_id(0, 0),
        ext_id(0, 1),
        ext_id(0, 3),
        ext_id(1, 0),
        ext_id(1, 1),
        ext_id(1, 3),
        ext_id(1, 4),
    ];
    assert_eq!(focus_order(&mut a), want);
}

#[test]
fn kotlin_is_asked_once_and_every_report_redraws() {
    let (mut a, mut text) = app(HOME);
    a.navigate(THEMES);
    a.navigate(&format!("ext/view/{MINE}"));
    a.back();
    a.navigate(UPDATES);
    assert!(
        a.effects.iter().all(|e| e[0] != "extensions"),
        "{:?}",
        a.effects
    );
    a.navigate(THEMES);
    // Kotlin reports again when an index changes: the imported theme extension is gone.
    let fewer: Vec<String> = report()
        .chunks(wire::settings_ui::extensions::row::SIZE)
        .filter(|f| f[wire::settings_ui::extensions::row::EXTENSION] != MINE)
        .flatten()
        .cloned()
        .collect();
    a.set_extensions(&fewer).unwrap();
    a.scene(&mut text);
    assert_eq!(control(&a, ext_id(0, 0)).label, "FlorisBoard themes");
    assert!(!has(&a, ext_id(1, 0)));
}

#[test]
fn a_bad_report_changes_nothing() {
    use wire::settings_ui::extensions::kind;
    let (mut a, _) = app(THEMES);
    let before = a.extensions.list.clone();
    let orphan = text_row(kind::DESCRIPTION, MINE, "no extension before it");
    let stray = [
        ext(MINE, extensions::THEME, "My themes", "1", true),
        text_row(kind::EMAIL, MINE, "a@b"),
    ]
    .concat();
    let unknown = [
        ext(MINE, extensions::THEME, "My themes", "1", true),
        text_row("colour", MINE, "x"),
    ]
    .concat();
    let wrong_owner = [
        ext(MINE, extensions::THEME, "My themes", "1", true),
        text_row(kind::DESCRIPTION, FLORIS, "x"),
    ]
    .concat();
    let mut short = report();
    short.pop();
    for bad in [orphan, stray, unknown, wrong_owner, short] {
        assert!(a.set_extensions(&bad).is_err());
        assert_eq!(a.extensions.list, before);
    }
    assert!(a.extension_done(&["delete".into(), MINE.into()]).is_err());
    assert!(
        a.extension_done(&["export".into(), MINE.into(), wire::flag::TRUE.into()])
            .is_err()
    );
}

#[test]
fn every_extension_route_is_native_and_its_title_is_the_screens() {
    for (route, title) in [
        (HOME, "Addons & Extensions"),
        (THEMES, "Theme extensions"),
        (KEYBOARDS, "Keyboard extensions"),
        (PACKS, "Language pack extensions"),
        (UPDATES, "Check for Updates"),
    ] {
        let (mut a, mut text) = app(route);
        assert_eq!(a.stack.last().map(String::as_str), Some(route));
        assert_eq!(a.title(), title, "{route}");
        assert!(
            painted(&mut a, &mut text).iter().any(|i| i.label == title),
            "{route}"
        );
    }
}
