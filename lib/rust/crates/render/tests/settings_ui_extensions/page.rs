//! An extension's page: its rows in ExtensionViewScreen's order, what is left out, the links,
//! the chips and their dialogs, Delete and Export, the bundled components, and the page before
//! the index comes or when no extension has its id.
use super::common::*;
use page::{
    COMPONENT_ID, DELETE_ID, DESCRIPTION_ID, EXPORT_ID, HEADER_ID, KEYWORD_ID, LABEL_ID,
    MAINTAINER_ID, NONE_ID, NOT_FOUND_ID, VALUE_ID,
};
use wire::settings_ui::extensions::kind;

fn view(id: &str) -> String {
    format!("ext/view/{id}")
}

#[test]
fn the_page_shows_the_description_then_each_row_in_order() {
    let (a, _) = app(&view(FLORIS));
    assert_eq!(a.title(), "FlorisBoard themes");
    let order = [
        DESCRIPTION_ID,
        MAINTAINER_ID,
        VALUE_ID + 1,
        VALUE_ID + 2,
        KEYWORD_ID,
        VALUE_ID + 4,
        VALUE_ID + 5,
        VALUE_ID + 6,
        EXPORT_ID,
        HEADER_ID,
        COMPONENT_ID,
    ];
    let tops: Vec<f32> = order.iter().map(|id| control(&a, *id).bounds.top).collect();
    assert!(tops.windows(2).all(|w| w[0] < w[1]), "{tops:?}");
    // A label sits level with its chips or its link.
    for (label, value) in [
        (LABEL_ID, MAINTAINER_ID),
        (LABEL_ID + 3, KEYWORD_ID),
        (LABEL_ID + 4, VALUE_ID + 4),
    ] {
        let (l, v) = (control(&a, label).bounds, control(&a, value).bounds);
        assert!(
            ((l.top + l.bottom) / 2.0 - (v.top + v.bottom) / 2.0).abs() <= 1.0,
            "{label}"
        );
    }
    let about = control(&a, DESCRIPTION_ID);
    assert_eq!(
        (about.label.as_str(), about.bounds.left),
        ("The themes that ship with the app.", px(16.0))
    );
    // A row's value ends 16 dp from the edge.
    for k in [1, 2, 6] {
        assert_eq!(control(&a, VALUE_ID + k).bounds.right, 1080.0 - px(16.0));
    }
}

/// Improvement: Compose's label and value were separate texts, so TalkBack read "ID" and the id
/// apart; here the row is one node, its label then its value.
#[test]
fn the_id_version_and_license_rows_read_as_one() {
    let (a, _) = app(&view(FLORIS));
    for (k, label, value) in [
        (1, "ID", FLORIS),
        (2, "Version", "0.2.0"),
        (6, "License", "apache-2.0"),
    ] {
        let row = control(&a, VALUE_ID + k);
        assert_eq!(
            (row.label.as_str(), row.summary.as_str(), row.role),
            (label, value, "text")
        );
        assert!(!has(&a, LABEL_ID + k));
    }
}

#[test]
fn a_built_in_extension_can_be_exported_but_not_deleted() {
    let (a, _) = app(&view(FLORIS));
    assert!(!has(&a, DELETE_ID));
    assert_eq!(control(&a, EXPORT_ID).label, "Export");
    let (a, _) = app(&view(MINE));
    let (delete, export) = (control(&a, DELETE_ID), control(&a, EXPORT_ID));
    assert_eq!((delete.label.as_str(), delete.role), ("Delete", "button"));
    assert_eq!(delete.bounds.top, export.bounds.top);
    assert_eq!(delete.bounds.left, px(16.0));
    assert_eq!(export.bounds.right, 1080.0 - px(16.0));
}

/// Difference: Compose drew an empty Text for a missing description; Keywords, Homepage and
/// Issue tracker were already left out when empty or blank.
#[test]
fn rows_with_nothing_to_show_are_left_out() {
    let (a, _) = app(&view(MINE));
    for id in [
        DESCRIPTION_ID,
        LABEL_ID + 3,
        LABEL_ID + 4,
        LABEL_ID + 5,
        VALUE_ID + 4,
        VALUE_ID + 5,
    ] {
        assert!(!has(&a, id), "{id}");
    }
    // The Maintainers row stays with nothing in it, as Compose kept it.
    assert!(has(&a, LABEL_ID));
    let blank = [
        ext(MINE, extensions::THEME, "My themes", "1.0.0", true),
        link(kind::HOMEPAGE, MINE, " \t", ""),
        link(kind::ISSUE_TRACKER, MINE, "", ""),
    ]
    .concat();
    let (mut a, mut text) = fresh();
    a.navigate(&view(MINE));
    a.set_extensions(&blank).unwrap();
    a.scene(&mut text);
    for id in [LABEL_ID + 4, LABEL_ID + 5, VALUE_ID + 4, VALUE_ID + 5] {
        assert!(!has(&a, id), "{id}");
    }
    assert!(has(&a, VALUE_ID + 6));
}

/// Improvement: FlorisHyperlinkText was a text TalkBack read as the host alone; here it is a
/// link that names its host and then where it goes.
#[test]
fn a_link_names_its_host_and_reads_where_it_goes() {
    let (mut a, mut text) = app(&view(FLORIS));
    for (k, host, link) in [
        (4, "florisboard.org", "https://florisboard.org/themes"),
        (5, "github.com", "https://github.com/florisboard/issues"),
    ] {
        let c = control(&a, VALUE_ID + k);
        assert_eq!(
            (c.label.as_str(), c.summary.as_str(), c.role),
            (host, link, "link")
        );
        assert_eq!(c.bounds.right, 1080.0 - px(16.0));
        assert_eq!(control(&a, LABEL_ID + k).role, "text");
        assert!(tap(&mut a, &mut text, VALUE_ID + k));
        assert_eq!(url(&a).as_deref(), Some(link));
    }
}

#[test]
fn chips_flow_into_lines_that_end_at_the_edge() {
    assert_eq!(
        chips::flow(&[10.0, 10.0, 10.0], 25.0, 2.0),
        [(2, 22.0), (1, 10.0)]
    );
    assert_eq!(
        chips::flow(&[30.0, 5.0, 5.0], 25.0, 2.0),
        [(1, 30.0), (2, 12.0)]
    );
    assert_eq!(chips::flow(&[], 25.0, 2.0), []);
}

/// Improvement: Compose scrolled a row of chips sideways, hiding all but the first few on a
/// phone; here they wrap onto lines, each ending where the single line did.
#[test]
fn many_keywords_wrap_and_stay_on_screen() {
    let mut report = vec![ext(MINE, extensions::THEME, "My themes", "1.0.0", true)];
    let words = [
        "Autumn", "Winter", "Spring", "Summer", "Pastel", "Contrast", "Ocean", "Forest",
    ];
    report.extend(words.iter().map(|w| label_row(kind::KEYWORD, MINE, w)));
    let (mut a, mut text) = fresh();
    a.config[0] = 720.0;
    a.navigate(&view(MINE));
    a.set_extensions(&report.concat()).unwrap();
    a.scene(&mut text);
    let chips: Vec<Control> = (0..words.len())
        .map(|k| control(&a, KEYWORD_ID + k as i32))
        .collect();
    let mut tops: Vec<f32> = chips.iter().map(|c| c.bounds.top).collect();
    tops.dedup();
    assert!(tops.len() >= 3, "{tops:?}");
    let label = control(&a, LABEL_ID + 3).bounds;
    for c in &chips {
        assert!(
            c.bounds.left >= label.right + px(24.0) - 0.5,
            "{:?}",
            c.bounds
        );
        assert!(c.bounds.right <= 720.0 - px(16.0) + 0.5, "{:?}", c.bounds);
        assert_eq!(c.bounds.height(), px(48.0));
    }
    for top in tops {
        let right = chips
            .iter()
            .filter(|c| c.bounds.top == top)
            .map(|c| c.bounds.right);
        assert_eq!(right.fold(0.0, f32::max), 720.0 - px(16.0));
    }
    // The row grows to hold its lines, and the next row starts under them.
    let last = chips.iter().map(|c| c.bounds.bottom).fold(0.0, f32::max);
    assert!(control(&a, VALUE_ID + 6).bounds.top >= last);
}

/// Improvement: keywords were disabled chips at 38% ink that TalkBack called dimmed buttons;
/// here they are in full ink and read as text.
#[test]
fn keywords_are_text_in_full_ink() {
    let (mut a, mut text) = app(&view(FLORIS));
    a.colors[2] = 0xff11_2233;
    let items = painted(&mut a, &mut text);
    for (k, word) in ["Day", "Night"].into_iter().enumerate() {
        let c = control(&a, KEYWORD_ID + k as i32);
        assert_eq!((c.label.as_str(), c.role, c.enabled), (word, "text", true));
        let item = items.iter().find(|i| i.label == word).unwrap();
        assert_eq!(item.text.foreground, 0xff11_2233);
        assert!(!tap(&mut a, &mut text, c.id) || a.modal.is_none());
        assert!(a.effects.is_empty());
    }
}

/// Improvement: a maintainer chip was a bare name; here TalkBack hears what its dialog offers.
#[test]
fn a_maintainer_chip_says_what_it_offers_and_lists_its_addresses() {
    let (mut a, mut text) = app(&view(FLORIS));
    let chip = control(&a, MAINTAINER_ID);
    assert_eq!(
        (chip.label.as_str(), chip.summary.as_str(), chip.role),
        ("patrickgold", "Email, Website", "button")
    );
    // The chip is as wide as its name: drawn whole, not cut short.
    assert!(
        painted(&mut a, &mut text)
            .iter()
            .any(|i| i.label == "patrickgold")
    );
    assert!(tap(&mut a, &mut text, MAINTAINER_ID));
    assert_eq!(a.maintainer_dialog(), Some(0));
    assert!(
        painted(&mut a, &mut text)
            .iter()
            .any(|i| i.label == "patrickgold")
    );
    let (mail, site) = (control(&a, 3000), control(&a, 3001));
    assert_eq!(
        (mail.label.as_str(), mail.summary.as_str(), mail.role),
        ("patrick@example.org", "Email", "link")
    );
    assert_eq!(
        (site.label.as_str(), site.summary.as_str(), site.role),
        ("https://patrickgold.dev", "Website", "link")
    );
    assert_eq!(mail.checked, None);
    // Cancel only: the dialog has nothing to confirm.
    assert!(!has(&a, 5) && !has(&a, 7));
    assert_eq!(control(&a, 6).label, "Cancel");
    assert!(tap(&mut a, &mut text, 3000));
    assert_eq!(url(&a).as_deref(), Some("mailto:patrick@example.org"));
    assert!(a.modal.is_none());
    assert!(tap(&mut a, &mut text, MAINTAINER_ID));
    assert!(tap(&mut a, &mut text, 6));
    assert!(a.modal.is_none());
    assert_eq!(a.effects.len(), 1);
}

/// Improvement: Compose disabled a maintainer chip with no address, which TalkBack read as a
/// dimmed button; here it is text, and focus passes it by.
#[test]
fn a_maintainer_with_no_address_reads_as_text() {
    let (mut a, mut text) = app(&view(MINE));
    let chip = control(&a, MAINTAINER_ID);
    assert_eq!(
        (chip.label.as_str(), chip.summary.as_str(), chip.role),
        ("Me", "", "text")
    );
    tap(&mut a, &mut text, MAINTAINER_ID);
    assert!(a.modal.is_none() && a.effects.is_empty());
    assert!(!focus_order(&mut a).contains(&MAINTAINER_ID));
}

#[test]
fn delete_asks_first_naming_the_extension_and_closes_the_page_once_it_is_gone() {
    let (mut a, mut text) = app(extensions::THEMES);
    a.navigate(&view(MINE));
    a.scene(&mut text);
    assert!(tap(&mut a, &mut text, DELETE_ID));
    let items = painted(&mut a, &mut text);
    assert!(items.iter().any(|i| i.label == "Confirm delete"));
    assert!(items.iter().any(|i| i.label == "Delete \"My themes\"?"));
    assert_eq!(
        (control(&a, 6).label.as_str(), control(&a, 7).label.as_str()),
        ("Cancel", "Delete")
    );
    assert!(tap(&mut a, &mut text, 6));
    assert!(a.modal.is_none() && a.effects.is_empty());
    assert!(tap(&mut a, &mut text, DELETE_ID));
    assert!(tap(&mut a, &mut text, 7));
    assert_eq!(a.effects, [["extensionDelete", MINE]]);
    // Deletion failed: Kotlin showed the error; the page stays.
    a.extension_done(&["delete".into(), MINE.into(), wire::flag::FALSE.into()])
        .unwrap();
    assert_eq!(a.stack.last(), Some(&view(MINE)));
    // Another extension went: still this page.
    a.extension_done(&["delete".into(), HAN.into(), wire::flag::TRUE.into()])
        .unwrap();
    assert_eq!(a.stack.last(), Some(&view(MINE)));
    a.extension_done(&["delete".into(), MINE.into(), wire::flag::TRUE.into()])
        .unwrap();
    assert_eq!(a.stack.last().map(String::as_str), Some(extensions::THEMES));
}

/// Improvement: Compose read the extension once, so a page opened before the index loaded said
/// the extension was not found; here it waits, then fills in.
#[test]
fn a_page_opened_before_the_index_comes_waits_for_it() {
    let (mut a, mut text) = fresh();
    a.navigate(&view(MINE));
    assert_eq!(a.effects, [["extensions"]]);
    a.scene(&mut text);
    assert_eq!(a.title(), "");
    for id in [NOT_FOUND_ID, LABEL_ID, EXPORT_ID, HEADER_ID] {
        assert!(!has(&a, id), "{id}");
    }
    a.set_extensions(&report()).unwrap();
    a.scene(&mut text);
    assert_eq!(a.title(), "My themes");
    assert!(has(&a, EXPORT_ID));
}

#[test]
fn an_unknown_id_says_no_extension_has_it() {
    let (a, _) = app(&view("nope"));
    assert_eq!(a.title(), "Extension not found");
    let c = control(&a, NOT_FOUND_ID);
    assert_eq!(
        (c.label.as_str(), c.role),
        ("No extension with ID \"nope\" could be found.", "text")
    );
    assert_eq!(
        (c.bounds.left, c.bounds.right),
        (px(16.0), 1080.0 - px(16.0))
    );
    assert!(!has(&a, EXPORT_ID) && !has(&a, HEADER_ID) && !has(&a, LABEL_ID));
}

#[test]
fn the_bundled_components_say_what_they_are() {
    let (a, _) = app(&view(FLORIS));
    let header = control(&a, HEADER_ID);
    assert_eq!(
        (header.label.as_str(), header.role),
        ("Bundled themes", "heading")
    );
    let texts = [
        (
            "Floris Day",
            "org.florisboard.themes:floris_day",
            "authors = [patrickgold, jtn]\nisNightTheme = false\nstylesheetPath = stylesheets/floris_day.json",
        ),
        (
            "Floris Night",
            "org.florisboard.themes:floris_night",
            "authors = []\nisNightTheme = true\nstylesheetPath = stylesheets/floris_night.json",
        ),
    ];
    for (c, (label, name, body)) in texts.into_iter().enumerate() {
        let id = COMPONENT_ID + 4 * c as i32;
        let parts = [control(&a, id), control(&a, id + 1), control(&a, id + 2)];
        let got: Vec<(&str, &str)> = parts.iter().map(|p| (p.label.as_str(), p.role)).collect();
        assert_eq!(got, [(label, "text"), (name, "text"), (body, "text")]);
    }
    let (a, _) = app(&view(HAN));
    assert_eq!(control(&a, HEADER_ID).label, "Bundled language packs");
    assert_eq!(
        control(&a, COMPONENT_ID + 2).label,
        "authors = [florisboard]\nlocale = zh-CN\n"
    );
    let (a, _) = app(&view(LAYOUTS));
    assert!(!has(&a, HEADER_ID) && !has(&a, NONE_ID) && !has(&a, COMPONENT_ID));
}

/// Where TalkBack's swipes reach the control with `id`.
fn read_at(a: &App, id: i32) -> usize {
    a.controls
        .iter()
        .position(|c| c.id == id)
        .expect("on screen")
}

#[test]
fn a_box_reads_its_name_before_what_is_in_it() {
    let (a, _) = app(&view(FLORIS));
    let (first, second) = (COMPONENT_ID, COMPONENT_ID + 4);
    let order = [
        HEADER_ID,
        first,
        first + 1,
        first + 2,
        second,
        second + 1,
        second + 2,
    ];
    let at: Vec<usize> = order.iter().map(|&id| read_at(&a, id)).collect();
    assert!(at.is_sorted(), "{at:?}");
    // The list's boxes too: the name, the id, then View and Edit.
    use super::common::extensions::THEMES;
    use boxes::ext_id;
    let (a, _) = app(THEMES);
    let order = [
        ext_id(0, 0),
        ext_id(0, 1),
        ext_id(0, 3),
        ext_id(1, 0),
        ext_id(1, 1),
    ];
    let at: Vec<usize> = order.iter().map(|&id| read_at(&a, id)).collect();
    assert!(at.is_sorted(), "{at:?}");
}

#[test]
fn an_extension_without_components_says_none_were_found() {
    let (mut a, mut text) = app(&view(MINE));
    let none = control(&a, NONE_ID);
    assert_eq!(
        (none.label.as_str(), none.role),
        ("No bundled components.", "text")
    );
    let items = painted(&mut a, &mut text);
    assert!(
        items
            .iter()
            .find(|i| i.label == "No bundled components.")
            .unwrap()
            .text
            .italic
    );
}

/// Improvement: ExtensionExportScreen was a screen of its own, blank behind the picker, which
/// launched the picker again whenever it was drawn again; Export opens only the picker.
#[test]
fn export_asks_for_a_flex_named_after_the_extension_and_opens_no_screen() {
    let (mut a, mut text) = app(&view(MINE));
    assert!(tap(&mut a, &mut text, EXPORT_ID));
    let mime = "application/vnd.florisboard.extension+zip";
    assert_eq!(a.effects, [["pick", "create", mime, "my.themes.flex"]]);
    assert_eq!(a.stack.last(), Some(&view(MINE)));
    assert_eq!(compose(&a), None);
}

#[test]
fn the_page_focus_order_visits_what_can_be_tapped() {
    let (mut a, _) = app(&view(FLORIS));
    assert_eq!(
        focus_order(&mut a),
        [1, MAINTAINER_ID, VALUE_ID + 4, VALUE_ID + 5, EXPORT_ID]
    );
    let (mut a, _) = app(&view(MINE));
    assert_eq!(focus_order(&mut a), [1, DELETE_ID, EXPORT_ID]);
}

#[test]
fn every_control_is_labelled_and_on_screen() {
    let routes = [
        extensions::HOME.to_owned(),
        extensions::THEMES.to_owned(),
        extensions::PACKS.to_owned(),
        extensions::UPDATES.to_owned(),
        view(FLORIS),
        view(MINE),
        view(HAN),
        view("nope"),
    ];
    for route in routes {
        let (a, _) = app(&route);
        for c in &a.controls {
            assert!(!c.label.trim().is_empty(), "{route} {}", c.id);
            assert!(
                c.bounds.left >= 0.0 && c.bounds.right <= 1080.0,
                "{route} {} {:?}",
                c.id,
                c.bounds
            );
            assert!(c.bounds.height() > 0.0, "{route} {}", c.id);
        }
    }
}
