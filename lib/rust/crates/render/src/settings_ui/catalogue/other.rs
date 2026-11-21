//! OtherScreen (`iconSpaceReserved = true`, no preview field): the settings theme, the settings
//! accent colour (a ColorPickerPreference whose system accent stores as no colour), Physical
//! keyboard, and Back up and Restore, which Rust draws too (`backup.rs`).
use super::{Choices, Kind, Row, Screen, entries, heading, list, row};
const THEME: Choices = entries(&[
    ("AUTO", "settings__system_default", ""),
    (
        "AUTO_AMOLED",
        "pref__other__settings_theme__auto_amoled",
        "",
    ),
    ("LIGHT", "pref__other__settings_theme__light", ""),
    ("DARK", "pref__other__settings_theme__dark", ""),
    (
        "AMOLED_DARK",
        "pref__other__settings_theme__amoled_dark",
        "",
    ),
]);
const ROWS: &[Row] = &[
    list(
        "other__settings_theme",
        "pref__other__settings_theme__label",
        &THEME,
    )
    .icon("palette"),
    row(
        "other__accent_color",
        "pref__other__settings_accent_color__label",
        "",
        Kind::Color,
    )
    .icon("format_color_fill"),
    row(
        "physical_keyboard__title",
        "physical_keyboard__title",
        "",
        Kind::Route("settings/other/physical-keyboard"),
    )
    .icon("keyboard_keys"),
    heading("backup_and_restore__title"),
    row(
        "backup_and_restore__back_up__title",
        "backup_and_restore__back_up__title",
        "backup_and_restore__back_up__summary",
        Kind::Route("settings/other/backup"),
    )
    .icon("archive"),
    row(
        "backup_and_restore__restore__title",
        "backup_and_restore__restore__title",
        "backup_and_restore__restore__summary",
        Kind::Route("settings/other/restore"),
    )
    .icon("settings_backup_restore"),
];
pub const SCREEN: Screen = Screen {
    route: "settings/other",
    title: "settings__other__title",
    icon_space: true,
    preview: false,
    rows: ROWS,
};
