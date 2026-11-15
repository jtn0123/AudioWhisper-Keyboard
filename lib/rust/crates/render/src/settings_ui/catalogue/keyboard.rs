//! KeyboardScreen.
use super::{
    Choices, Cond::*, Kind, Row, Screen, Slider, dual, entries, heading, list, row, single, slider,
    toggle, unit,
};
const HINT: Choices = Choices {
    entries: &[
        (
            "ACCENT_PRIORITY",
            "enum__key_hint_mode__accent_priority",
            "enum__key_hint_mode__accent_priority__description",
        ),
        (
            "HINT_PRIORITY",
            "enum__key_hint_mode__hint_priority",
            "enum__key_hint_mode__hint_priority__description",
        ),
        (
            "SMART_PRIORITY",
            "enum__key_hint_mode__smart_priority",
            "enum__key_hint_mode__smart_priority__description",
        ),
    ],
    selected_only: true,
    emoji: &[],
};
const UTILITY: Choices = entries(&[
    (
        "SWITCH_TO_EMOJIS",
        "enum__utility_key_action__switch_to_emojis",
        "",
    ),
    (
        "SWITCH_LANGUAGE",
        "enum__utility_key_action__switch_language",
        "",
    ),
    (
        "SWITCH_KEYBOARD_APP",
        "enum__utility_key_action__switch_keyboard_app",
        "",
    ),
    (
        "DYNAMIC_SWITCH_LANGUAGE_EMOJIS",
        "enum__utility_key_action__dynamic_switch_language_emojis",
        "",
    ),
]);
const SPACE_BAR: Choices = entries(&[
    ("NOTHING", "enum__space_bar_mode__nothing", ""),
    (
        "CURRENT_LANGUAGE",
        "enum__space_bar_mode__current_language",
        "",
    ),
    ("SPACE_BAR_KEY", "enum__space_bar_mode__space_bar_key", ""),
]);
const CAPITALIZATION: Choices = entries(&[
    (
        "CAPSLOCK_BY_DOUBLE_TAP",
        "enum__capitalization_behavior__capslock_by_double_tap",
        "",
    ),
    (
        "CAPSLOCK_BY_CYCLE",
        "enum__capitalization_behavior__capslock_by_cycle",
        "",
    ),
]);
const INCOGNITO: Choices = entries(&[
    (
        "REPLACE_SHARED_ACTIONS_TOGGLE",
        "enum__incognito_display_mode__replace_shared_actions_toggle",
        "",
    ),
    (
        "DISPLAY_BEHIND_KEYBOARD",
        "enum__incognito_display_mode__display_behind_keyboard",
        "",
    ),
]);
const ONE_HANDED: Choices = entries(&[
    ("START", "enum__one_handed_mode__start", ""),
    ("END", "enum__one_handed_mode__end", ""),
]);
const LANDSCAPE: Choices = entries(&[
    (
        "NEVER_SHOW",
        "enum__landscape_input_ui_mode__never_show",
        "",
    ),
    (
        "ALWAYS_SHOW",
        "enum__landscape_input_ui_mode__always_show",
        "",
    ),
    (
        "DYNAMICALLY_SHOW",
        "enum__landscape_input_ui_mode__dynamically_show",
        "",
    ),
]);
const ORIENTATION: [&str; 2] = [
    "screen_orientation__portrait",
    "screen_orientation__landscape",
];
const PERCENT: Slider = single(unit("unit__percent__symbol"));
const FONT: Slider = dual(
    "keyboard__font_size_multiplier_landscape",
    unit("unit__percent__symbol"),
    ORIENTATION,
);
const HEIGHT: Slider = dual(
    "keyboard__height_factor_landscape",
    unit("unit__percent__symbol"),
    ORIENTATION,
);
const SPACING: Slider = dual(
    "keyboard__key_spacing_horizontal",
    unit("unit__display_pixel__symbol"),
    [
        "screen_orientation__vertical",
        "screen_orientation__horizontal",
    ],
);
const OFFSET: Slider = dual(
    "keyboard__bottom_offset_landscape",
    unit("unit__display_pixel__symbol"),
    ORIENTATION,
);
const DELAY: Slider = single(unit("unit__milliseconds__symbol"));
pub const KEYBOARD: &[Row] = &[
    toggle(
        "keyboard__rust_renderer_prototype",
        "pref__keyboard__rust_renderer_prototype__label",
        "pref__keyboard__rust_renderer_prototype__summary",
    ),
    toggle(
        "keyboard__number_row",
        "pref__keyboard__number_row__label",
        "pref__keyboard__number_row__summary",
    ),
    list(
        "keyboard__hinted_number_row_mode",
        "pref__keyboard__hinted_number_row_mode__label",
        &HINT,
    )
    .linked("keyboard__hinted_number_row_enabled", "state__disabled")
    .on(No("keyboard__number_row")),
    list(
        "keyboard__hinted_symbols_mode",
        "pref__keyboard__hinted_symbols_mode__label",
        &HINT,
    )
    .linked("keyboard__hinted_symbols_enabled", "state__disabled"),
    toggle(
        "keyboard__utility_key_enabled",
        "pref__keyboard__utility_key_enabled__label",
        "pref__keyboard__utility_key_enabled__summary",
    ),
    list(
        "keyboard__utility_key_action",
        "pref__keyboard__utility_key_action__label",
        &UTILITY,
    )
    .shown(Yes("keyboard__utility_key_enabled")),
    list(
        "keyboard__space_bar_display_mode",
        "pref__keyboard__space_bar_mode__label",
        &SPACE_BAR,
    ),
    list(
        "keyboard__capitalization_behavior",
        "pref__keyboard__capitalization_behavior__label",
        &CAPITALIZATION,
    ),
    slider(
        "keyboard__font_size_multiplier_portrait",
        "pref__keyboard__font_size_multiplier__label",
        &FONT,
    ),
    list(
        "keyboard__incognito_indicator",
        "pref__keyboard__incognito_indicator__label",
        &INCOGNITO,
    ),
    heading("pref__keyboard__group_layout__label"),
    list(
        "keyboard__one_handed_mode",
        "pref__keyboard__one_handed_mode__label",
        &ONE_HANDED,
    )
    .linked("keyboard__one_handed_mode_enabled", "state__disabled"),
    slider(
        "keyboard__one_handed_mode_scale_factor",
        "pref__keyboard__one_handed_mode_scale_factor__label",
        &PERCENT,
    )
    .on(Yes("keyboard__one_handed_mode_enabled")),
    list(
        "keyboard__landscape_input_ui_mode",
        "pref__keyboard__landscape_input_ui_mode__label",
        &LANDSCAPE,
    ),
    slider(
        "keyboard__height_factor_portrait",
        "pref__keyboard__height_factor__label",
        &HEIGHT,
    ),
    slider(
        "keyboard__key_spacing_vertical",
        "pref__keyboard__key_spacing__label",
        &SPACING,
    ),
    slider(
        "keyboard__bottom_offset_portrait",
        "pref__keyboard__bottom_offset__label",
        &OFFSET,
    ),
    heading("pref__keyboard__group_keypress__label"),
    row(
        "settings__input_feedback__title",
        "settings__input_feedback__title",
        "",
        Kind::Route("settings/keyboard/input-feedback"),
    ),
    toggle(
        "keyboard__popup_enabled",
        "pref__keyboard__popup_enabled__label",
        "pref__keyboard__popup_enabled__summary",
    ),
    toggle(
        "keyboard__merge_hint_popups_enabled",
        "pref__keyboard__merge_hint_popups_enabled__label",
        "pref__keyboard__merge_hint_popups_enabled__summary",
    ),
    slider(
        "keyboard__long_press_delay",
        "pref__keyboard__long_press_delay__label",
        &DELAY,
    ),
    toggle(
        "keyboard__space_bar_switches_to_characters",
        "pref__keyboard__space_bar_switches_to_characters__label",
        "pref__keyboard__space_bar_switches_to_characters__summary",
    ),
];
pub(super) const SCREEN: Screen = Screen {
    route: "settings/keyboard",
    title: "settings__keyboard__title",
    icon_space: true,
    preview: true,
    rows: KEYBOARD,
};
