//! The language screens (`settings_ui/languages`): Languages & Layouts, Select language, the
//! language pack manager and the subtype editor, which adds and edits at two routes. Rows whose
//! kind is `Lang` or `Pick` are painted by those modules; the ones shown `Never` are dialogs.
use super::{Choices, Cond, Kind, Row, Screen, heading, list, row, toggle};
use crate::settings_ui::{items::Source, languages as screens};

/// A subtype editor field: what a `Pick` row chooses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// The primary language, chosen on Select language.
    Locale,
    Popup,
    /// A layout, by its place in `layoutMap` (characters, symbols, ... phone2).
    Layout(usize),
    /// The suggestion engine, which sets spelling and suggestion together.
    Provider,
    Composer,
    Currency,
}

/// The language screens' own parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    /// Add subtype's suggested presets card and its Show all button.
    Presets,
    /// The editor's 32 dp spacer between groups of fields.
    Gap,
    /// Select language's search field.
    Search,
    /// Select language's list, or its message when nothing matches.
    Locales,
    /// The language pack manager's Import box.
    Import,
    /// The language pack manager's boxes, one per extension.
    Packs,
    /// The editor's error dialog.
    Message,
    /// Every subtype preset, in a dialog.
    AllPresets,
    /// The language pack manager's delete confirmation.
    DeletePack,
    /// The editor's bottom bar: Cancel, and Save.
    Cancel,
    Save,
}

const NAMES: Choices = Choices {
    entries: &[
        (
            "SYSTEM_LOCALE",
            "enum__display_language_names_in__system_locale",
            "enum__display_language_names_in__system_locale__description",
        ),
        (
            "NATIVE_LOCALE",
            "enum__display_language_names_in__native_locale",
            "enum__display_language_names_in__native_locale__description",
        ),
    ],
    selected_only: true,
    emoji: &[],
};
const LIST_ROWS: &[Row] = &[
    list(
        screens::NAMES_KEY,
        "settings__localization__display_language_names_in__label",
        &NAMES,
    ),
    toggle(
        "localization__display_keyboard_labels_in_subtype_language",
        "settings__localization__display_keyboard_labels_in_subtype_language",
        "",
    ),
    row(
        "settings__localization__language_pack_title",
        "settings__localization__language_pack_title",
        "settings__localization__language_pack_summary",
        Kind::Route(screens::PACKS_ROUTE),
    ),
    heading("settings__localization__group_subtypes__label"),
    row(
        screens::SUBTYPES_KEY,
        "settings__localization__subtype_no_subtypes_configured_warning",
        "settings__localization__subtype_add_title",
        Kind::List(Source::Subtypes),
    ),
];
/// LocalizationScreen: `previewFieldVisible`, no icon space.
pub const LIST: Screen = Screen {
    route: screens::LIST_ROUTE,
    title: "settings__localization__title",
    icon_space: false,
    preview: true,
    rows: LIST_ROWS,
};
const PICKER_ROWS: &[Row] = &[
    row(
        "",
        "settings__localization__subtype_search_locale_placeholder",
        "",
        Kind::Lang(Lang::Search),
    ),
    row(
        "",
        "settings__localization__subtype_search_locale_not_found",
        "",
        Kind::Lang(Lang::Locales),
    ),
];
pub const PICKER: Screen = Screen {
    route: screens::PICKER_ROUTE,
    title: "settings__localization__subtype_select_locale",
    icon_space: false,
    preview: false,
    rows: PICKER_ROWS,
};
const PACK_ROWS: &[Row] = &[
    row("", "action__import", "", Kind::Lang(Lang::Import)),
    row("", "action__delete", "", Kind::Lang(Lang::Packs)),
    row(
        "",
        "action__delete_confirm_title",
        "action__delete_confirm_message",
        Kind::Lang(Lang::DeletePack),
    )
    .shown(Cond::Never),
];
pub const PACKS: Screen = Screen {
    route: screens::PACKS_ROUTE,
    title: "settings__localization__language_pack_title",
    icon_space: false,
    preview: false,
    rows: PACK_ROWS,
};
const fn pick(label: &'static str, field: Field) -> Row {
    row("", label, "", Kind::Pick(field))
}
const GAP: Row = row("", "", "", Kind::Lang(Lang::Gap));
/// SubtypeEditorScreen's fields in its order; adding shows the presets card first.
const EDITOR_ROWS: &[Row] = &[
    row(
        "",
        "settings__localization__suggested_subtype_presets",
        "settings__localization__suggested_subtype_presets_none_found",
        Kind::Lang(Lang::Presets),
    ),
    pick("settings__localization__subtype_locale", Field::Locale),
    pick(
        "settings__localization__subtype_popup_mapping",
        Field::Popup,
    ),
    pick(
        "settings__localization__subtype_characters_layout",
        Field::Layout(0),
    ),
    GAP,
    pick(
        "settings__localization__subtype_suggestion_provider",
        Field::Provider,
    ),
    GAP,
    pick(
        "settings__localization__subtype_symbols_layout",
        Field::Layout(1),
    ),
    pick(
        "settings__localization__subtype_symbols2_layout",
        Field::Layout(2),
    ),
    pick("settings__localization__subtype_composer", Field::Composer),
    pick(
        "settings__localization__subtype_currency_set",
        Field::Currency,
    ),
    GAP,
    pick(
        "settings__localization__subtype_numeric_layout",
        Field::Layout(3),
    ),
    pick(
        "settings__localization__subtype_numeric_advanced_layout",
        Field::Layout(4),
    ),
    pick(
        "settings__localization__subtype_numeric_row_layout",
        Field::Layout(5),
    ),
    GAP,
    pick(
        "settings__localization__subtype_phone_layout",
        Field::Layout(6),
    ),
    pick(
        "settings__localization__subtype_phone2_layout",
        Field::Layout(7),
    ),
    row("", "error__title", "", Kind::Lang(Lang::Message)).shown(Cond::Never),
    row(
        "",
        "settings__localization__subtype_presets",
        "settings__localization__subtype_presets_view_all",
        Kind::Lang(Lang::AllPresets),
    )
    .shown(Cond::Never),
    row("", "action__cancel", "", Kind::Lang(Lang::Cancel)),
    row("", "action__save", "", Kind::Lang(Lang::Save)),
];
pub const ADD: Screen = Screen {
    route: screens::ADD_ROUTE,
    title: "settings__localization__subtype_add_title",
    icon_space: false,
    preview: false,
    rows: EDITOR_ROWS,
};
pub const EDIT: Screen = Screen {
    route: screens::EDIT_ROUTE,
    title: "settings__localization__subtype_edit_title",
    icon_space: false,
    preview: false,
    rows: EDITOR_ROWS,
};
