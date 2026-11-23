//! An index as a fresh install has it plus extensions imported here, the strings the screens
//! show, and the app at a route with that index reported.
pub use render::{
    settings_ui::{
        App, Control,
        extensions::{self, boxes, chips, import, page},
    },
    text::Text,
};
use wire::settings_ui::extensions::{kind, row};

pub const LAYOUTS: &str = "org.florisboard.layouts";
pub const FLORIS: &str = "org.florisboard.themes";
pub const MINE: &str = "my.themes";
pub const HAN: &str = "org.florisboard.han";

/// A report row.
pub fn line(k: &str, e: &str, [id, label, detail, text]: [&str; 4], flag: bool) -> Vec<String> {
    let mut f = vec![String::new(); row::SIZE];
    f[row::KIND] = k.into();
    f[row::EXTENSION] = e.into();
    f[row::ID] = id.into();
    f[row::LABEL] = label.into();
    f[row::DETAIL] = detail.into();
    f[row::TEXT] = text.into();
    f[row::FLAG] = wire::flag::encode(flag).into();
    f
}
/// An extension row: serial, title, version, license and whether it can be deleted.
pub fn ext(id: &str, serial: &str, title: &str, version: &str, deletable: bool) -> Vec<String> {
    line(
        kind::EXTENSION,
        id,
        [serial, title, version, "apache-2.0"],
        deletable,
    )
}
pub fn text_row(k: &str, e: &str, text: &str) -> Vec<String> {
    line(k, e, ["", "", "", text], false)
}
pub fn label_row(k: &str, e: &str, label: &str) -> Vec<String> {
    line(k, e, ["", label, "", ""], false)
}
pub fn link(k: &str, e: &str, url: &str, host: &str) -> Vec<String> {
    line(k, e, ["", "", host, url], false)
}
pub fn component(e: &str, id: &str, label: &str, detail: &str, night: bool) -> Vec<String> {
    line(kind::COMPONENT, e, [id, label, detail, ""], night)
}

/// The keyboard layouts and the themes the app ships, one theme extension imported here and a
/// language pack imported here, in `combinedExtensionList()` order.
pub fn report() -> Vec<String> {
    [
        ext(
            LAYOUTS,
            extensions::KEYBOARD,
            "FlorisBoard layouts",
            "0.1.0",
            false,
        ),
        ext(
            FLORIS,
            extensions::THEME,
            "FlorisBoard themes",
            "0.2.0",
            false,
        ),
        text_row(
            kind::DESCRIPTION,
            FLORIS,
            "The themes that ship with the app.",
        ),
        label_row(kind::MAINTAINER, FLORIS, "patrickgold"),
        text_row(kind::EMAIL, FLORIS, "patrick@example.org"),
        text_row(kind::URL, FLORIS, "https://patrickgold.dev"),
        label_row(kind::KEYWORD, FLORIS, "Day"),
        label_row(kind::KEYWORD, FLORIS, "Night"),
        link(
            kind::HOMEPAGE,
            FLORIS,
            "https://florisboard.org/themes",
            "florisboard.org",
        ),
        link(
            kind::ISSUE_TRACKER,
            FLORIS,
            "https://github.com/florisboard/issues",
            "github.com",
        ),
        component(
            FLORIS,
            "floris_day",
            "Floris Day",
            "stylesheets/floris_day.json",
            false,
        ),
        label_row(kind::AUTHOR, FLORIS, "patrickgold"),
        label_row(kind::AUTHOR, FLORIS, "jtn"),
        component(
            FLORIS,
            "floris_night",
            "Floris Night",
            "stylesheets/floris_night.json",
            true,
        ),
        ext(MINE, extensions::THEME, "My themes", "1.0.0", true),
        label_row(kind::MAINTAINER, MINE, "Me"),
        ext(
            HAN,
            extensions::LANGUAGE_PACK,
            "Han language pack",
            "0.3.0",
            true,
        ),
        component(HAN, "zh_CN", "Chinese (China)", "zh-CN", false),
        label_row(kind::AUTHOR, HAN, "florisboard"),
    ]
    .concat()
}
pub fn text() -> Text {
    Text::new(vec![
        include_bytes!("../../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ])
}
/// The strings the screens show, in English, and the icons they draw.
pub fn strings(a: &mut App) {
    for (k, v) in [
        ("ext__home__title", "Addons & Extensions"),
        ("ext__list__ext_theme", "Theme extensions"),
        ("ext__list__ext_keyboard", "Keyboard extensions"),
        ("ext__list__ext_languagepack", "Language pack extensions"),
        (
            "ext__home__info",
            "You can download and install extensions.",
        ),
        ("ext__home__visit_store", "Visit Addons Store"),
        ("action__import", "Import"),
        (
            "ext__update_box__manual_updates_hint",
            "Updates are checked by hand.",
        ),
        ("ext__update_box__search_for_updates", "Search for Updates"),
        ("ext__editor__title_create_any", "Create extension"),
        ("ext__list__view_details", "View details"),
        ("action__edit", "Edit"),
        ("action__delete", "Delete"),
        ("action__export", "Export"),
        ("action__cancel", "Cancel"),
        ("ext__meta__maintainers", "Maintainers"),
        ("ext__meta__id", "ID"),
        ("ext__meta__version", "Version"),
        ("ext__meta__keywords", "Keywords"),
        ("ext__meta__homepage", "Homepage"),
        ("ext__meta__issue_tracker", "Issue tracker"),
        ("ext__meta__license", "License"),
        ("ext__meta__components_theme", "Bundled themes"),
        (
            "ext__meta__components_language_pack",
            "Bundled language packs",
        ),
        ("ext__meta__components_none_found", "No bundled components."),
        ("ext__error__not_found_title", "Extension not found"),
        (
            "ext__error__not_found_description",
            "No extension with ID \"{id}\" could be found.",
        ),
        ("ext__meta__email", "Email"),
        ("ext__meta__website", "Website"),
        ("ext__check_updates__title", "Check for Updates"),
        ("action__delete_confirm_title", "Confirm delete"),
        ("action__delete_confirm_message", "Delete \"{name}\"?"),
        ("ext__import__ext_any", "Import extension"),
        ("ext__import__ext_keyboard", "Import keyboard extension"),
        ("ext__import__ext_theme", "Import theme extension"),
        (
            "ext__import__ext_languagepack",
            "Import language pack extension",
        ),
        ("action__select_files", "Select files"),
        ("state__no_files_selected", "No files selected"),
        ("ext__meta__maintainers_by", "By: {maintainers}"),
        (
            "ext__import__file_skip",
            "File can not be imported. Reason:",
        ),
        ("ext__import__file_skip_unsupported", "Unsupported."),
        ("ext__import__file_skip_ext_core", "Core."),
        ("ext__import__file_skip_ext_corrupted", "Corrupted."),
        ("settings__ext__skip_other_type", "Other type."),
        ("settings__ext__read_failed", "The files could not be read."),
        ("settings__ext__files_gone", "The files are gone."),
        (
            "settings__ext__unwritable",
            "The file could not be written.",
        ),
        ("ext__import__success", "Imported!"),
        ("ext__import__failure", "Import failed: {error_message}"),
        ("ext__export__success", "Exported!"),
        ("ext__export__failure", "Export failed: {error_message}"),
        ("error__snackbar_message", "Something went wrong"),
        ("settings__udm__no_picker", "No picker."),
        ("settings__files__interrupted", "Interrupted."),
    ] {
        a.strings.insert(k.into(), v.into());
    }
}
/// A phone-sized app.
pub fn fresh() -> (App, Text) {
    let mut a = App {
        config: [1080.0, 6000.0, 2.625, 1.0],
        ..App::default()
    };
    strings(&mut a);
    (a, text())
}
/// The app at `route` with the index reported and painted, its effects cleared.
pub fn app(route: &str) -> (App, Text) {
    let (mut a, mut text) = fresh();
    a.navigate(route);
    assert_eq!(a.effects, [["extensions"]], "{route}");
    a.set_extensions(&report()).unwrap();
    a.scene(&mut text);
    a.effects.clear();
    (a, text)
}
pub fn control(a: &App, id: i32) -> Control {
    a.controls
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .unwrap_or_else(|| panic!("no control {id}"))
}
pub fn has(a: &App, id: i32) -> bool {
    a.controls.iter().any(|c| c.id == id)
}
pub fn tap(a: &mut App, text: &mut Text, id: i32) -> bool {
    a.scene(text);
    let done = a.activate(id);
    a.scene(text);
    done
}
/// The last `url` effect's link.
pub fn url(a: &App) -> Option<String> {
    a.effects
        .iter()
        .rev()
        .find(|e| e[0] == "url")
        .map(|e| e[1].clone())
}
/// `dp` in whole pixels at the phone's density.
pub fn px(dp: f32) -> f32 {
    (dp * 2.625_f32).round()
}
/// The control ids focus visits, from Back round to Back.
pub fn focus_order(a: &mut App) -> Vec<i32> {
    a.focus = -1;
    a.focus_next(false);
    let first = a.focus;
    let mut out = vec![first];
    loop {
        a.focus_next(false);
        if a.focus == first {
            return out;
        }
        out.push(a.focus);
    }
}
/// What the scene paints.
pub fn painted(a: &mut App, text: &mut Text) -> Vec<render::panels::Item> {
    a.scene(text).panel.expect("a settings panel").items
}
