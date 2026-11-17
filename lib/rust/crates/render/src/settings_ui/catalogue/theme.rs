//! ThemeScreen (`iconSpaceReserved` true, the preview field shown): the theme mode, the day and
//! night themes (links to the theme manager, which stays in Compose), the sunrise and sunset
//! times, the keyboard's accent colour and AddonManagementReferenceBox for theme extensions.
use super::{Addons, Choices, Cond, Kind, Row, Screen, entries, list, row};
const MODE: Choices = entries(&[
    ("ALWAYS_DAY", "enum__theme_mode__always_day", ""),
    ("ALWAYS_NIGHT", "enum__theme_mode__always_night", ""),
    ("FOLLOW_SYSTEM", "enum__theme_mode__follow_system", ""),
    ("FOLLOW_TIME", "enum__theme_mode__follow_time", ""),
]);
const FOLLOW_TIME: Cond = Cond::Is("theme__mode", "FOLLOW_TIME");
const ADDONS: Addons = Addons {
    title: "ext__list__ext_theme",
    route: "ext/list/EXT_THEME?showUpdate=true",
};
const ROWS: &[Row] = &[
    list("theme__mode", "pref__theme__mode__label", &MODE).icon("brightness_auto"),
    row(
        "theme__day_theme_id",
        "pref__theme__day",
        "",
        Kind::Theme("settings/theme/manage/SELECT_DAY"),
    )
    .icon("light_mode")
    .on(Cond::Not("theme__mode", "ALWAYS_NIGHT")),
    row(
        "theme__night_theme_id",
        "pref__theme__night",
        "",
        Kind::Theme("settings/theme/manage/SELECT_NIGHT"),
    )
    .icon("dark_mode")
    .on(Cond::Not("theme__mode", "ALWAYS_DAY")),
    row(
        "theme__sunrise_time",
        "pref__theme__sunrise_time__label",
        "",
        Kind::Time,
    )
    .icon("wb_twilight")
    .on(FOLLOW_TIME),
    row(
        "theme__sunset_time",
        "pref__theme__sunset_time__label",
        "",
        Kind::Time,
    )
    .icon("brightness_2")
    .on(FOLLOW_TIME),
    row(
        "theme__accent_color",
        "pref__theme__theme_accent_color__label",
        "",
        Kind::Color,
    )
    .icon("color_lens"),
    row(
        "ext_theme",
        "ext__list__ext_theme",
        "",
        Kind::Addons(&ADDONS),
    ),
];
pub const SCREEN: Screen = Screen {
    route: "settings/theme",
    title: "settings__theme__title",
    icon_space: true,
    preview: true,
    rows: ROWS,
};
