//! DictionaryScreen (`iconSpaceReserved = true`, `previewFieldVisible = true`).
use super::{Cond::*, Kind, Row, Screen, row, toggle};
const ROWS: &[Row] = &[
    toggle(
        "suggestion__enable_system_user_dictionary",
        "pref__dictionary__enable_system_user_dictionary__label",
        "pref__dictionary__enable_system_user_dictionary__summary",
    ),
    row(
        "pref__dictionary__manage_system_user_dictionary__label",
        "pref__dictionary__manage_system_user_dictionary__label",
        "pref__dictionary__manage_system_user_dictionary__summary",
        Kind::Route("settings/dictionary/user-dictionary/SYSTEM"),
    )
    .on(Yes("suggestion__enable_system_user_dictionary")),
    toggle(
        "suggestion__enable_floris_user_dictionary",
        "pref__dictionary__enable_internal_user_dictionary__label",
        "pref__dictionary__enable_internal_user_dictionary__summary",
    ),
    row(
        "pref__dictionary__manage_floris_user_dictionary__label",
        "pref__dictionary__manage_floris_user_dictionary__label",
        "pref__dictionary__manage_floris_user_dictionary__summary",
        Kind::Route("settings/dictionary/user-dictionary/FLORIS"),
    )
    .on(Yes("suggestion__enable_floris_user_dictionary")),
];
pub const SCREEN: Screen = Screen {
    route: "settings/dictionary",
    title: "settings__dictionary__title",
    icon_space: true,
    preview: true,
    rows: ROWS,
};
