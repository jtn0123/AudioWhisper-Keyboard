//! The extension screens (`settings_ui/extensions`): ExtensionHomeScreen, the three extension
//! lists, an extension's page, CheckUpdatesScreen and the importer. Rows whose kind is `Ext` are
//! painted by that module; the page's delete question and a maintainer's addresses are dialogs on
//! hidden rows, and the importer's Cancel and Import are the bottom bar the subtype editor has.
use super::{Cond, Kind, Row, Screen, languages::Lang, row};
use crate::settings_ui::extensions as screens;

/// The extension screens' own parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// ImportExtensionBox: where extensions come from, Visit Addons Store and Import. A list
    /// shows it only when its route asks for `showUpdate=true`.
    Store,
    /// UpdateBox: updates are checked by hand, and Search for Updates.
    Updates,
    /// The theme list's Create extension, which Compose had as a floating button.
    Create,
    /// One outlined box per extension of the list's type.
    Extensions,
    /// An extension's page: its description, details, Delete and Export; or that no extension
    /// has the id.
    About,
    /// An extension's page: its bundled themes or language packs.
    Components,
    /// The importer: Select files, then each file read and what it holds, or that none is.
    Import,
    /// The extension editor's own parts (`extensions/editor`).
    Editor(super::ext_editor::Edit),
}

const fn ext(part: Part) -> Row {
    row("", "", "", Kind::Ext(part))
}
const fn list(route: &'static str, title: &'static str, icon: &'static str) -> Row {
    row(title, title, "", Kind::Route(route)).icon(icon)
}
const fn screen(route: &'static str, title: &'static str, rows: &'static [Row]) -> Screen {
    Screen {
        route,
        title,
        icon_space: true,
        preview: false,
        rows,
    }
}

pub const HOME: Screen = screen(
    screens::HOME,
    "ext__home__title",
    &[
        ext(Part::Store),
        ext(Part::Updates),
        list(screens::THEMES, "ext__list__ext_theme", "palette"),
        list(screens::KEYBOARDS, "ext__list__ext_keyboard", "keyboard"),
        list(screens::PACKS, "ext__list__ext_languagepack", "language"),
    ],
);
const LIST: &[Row] = &[ext(Part::Store), ext(Part::Updates), ext(Part::Extensions)];
pub const THEMES: Screen = screen(
    screens::THEMES,
    "ext__list__ext_theme",
    &[
        ext(Part::Store),
        ext(Part::Updates),
        ext(Part::Create),
        ext(Part::Extensions),
    ],
);
pub const KEYBOARDS: Screen = screen(screens::KEYBOARDS, "ext__list__ext_keyboard", LIST);
pub const PACKS: Screen = screen(screens::PACKS, "ext__list__ext_languagepack", LIST);
/// The page's title is the extension's, or the not-found title (`extensions::ext_title`).
pub const VIEW: Screen = screen(
    screens::VIEW,
    "ext__error__not_found_title",
    &[
        ext(Part::About),
        ext(Part::Components),
        row(
            "",
            "action__delete_confirm_title",
            "action__delete_confirm_message",
            Kind::Lang(Lang::DeleteExtension),
        )
        .shown(Cond::Never),
        row("", "", "", Kind::Lang(Lang::Maintainer)).shown(Cond::Never),
    ],
);
/// The importer's rows: its files, then the bottom bar's Cancel and Import.
const IMPORT: &[Row] = &[
    ext(Part::Import),
    row("", "action__cancel", "", Kind::Lang(Lang::Cancel)),
    row("", "action__import", "", Kind::Lang(Lang::Save)).on(Cond::Import),
];
pub const IMPORT_ANY: Screen = screen(screens::import::ANY, "ext__import__ext_any", IMPORT);
pub const IMPORT_KEYBOARDS: Screen = screen(
    screens::import::KEYBOARDS,
    "ext__import__ext_keyboard",
    IMPORT,
);
pub const IMPORT_THEMES: Screen = screen(screens::import::THEMES, "ext__import__ext_theme", IMPORT);
pub const IMPORT_PACKS: Screen = screen(
    screens::import::PACKS,
    "ext__import__ext_languagepack",
    IMPORT,
);
pub const UPDATES: Screen = screen(
    screens::UPDATES,
    "ext__check_updates__title",
    &[ext(Part::Updates)],
);
