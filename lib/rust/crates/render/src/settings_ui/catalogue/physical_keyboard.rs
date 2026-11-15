//! PhysicalKeyboardScreen (`iconSpaceReserved = true`, no preview field): the system settings
//! row opens ACTION_HARD_KEYBOARD_SETTINGS only while a keyboard is attached
//! (`Configuration.keyboard != KEYBOARD_NOKEYS`, the `physical_keyboard` fact).
use super::{Cond::*, Do, Kind, Row, Screen, row, toggle};
const ROWS: &[Row] = &[
    row(
        "physical_keyboard__system_settings__title",
        "physical_keyboard__system_settings__title",
        "physical_keyboard__system_settings__summary",
        Kind::Effect(Do::Intent("hardKeyboardSettings")),
    )
    .shown(Fact("physical_keyboard", "true")),
    row(
        "physical_keyboard__system_settings__not_attached",
        "physical_keyboard__system_settings__title",
        "physical_keyboard__system_settings__summary_not_attached",
        Kind::Info,
    )
    .shown(Fact("physical_keyboard", "false")),
    toggle(
        "physical_keyboard__show_on_screen_keyboard",
        "physical_keyboard__show_on_screen_keyboard__title",
        "physical_keyboard__show_on_screen_keyboard__summary",
    ),
];
pub const SCREEN: Screen = Screen {
    route: "settings/other/physical-keyboard",
    title: "physical_keyboard__title",
    icon_space: true,
    preview: false,
    rows: ROWS,
};
