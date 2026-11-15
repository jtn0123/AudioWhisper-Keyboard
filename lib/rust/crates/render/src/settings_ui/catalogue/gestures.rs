//! GesturesScreen.
use super::{
    Choices, Cond::*, Row, Screen, Slider, entries, heading, list, single, slider, toggle, unit,
};
const GENERAL: Choices = entries(&[
    ("NO_ACTION", "enum__swipe_action__no_action", ""),
    (
        "CYCLE_TO_PREVIOUS_KEYBOARD_MODE",
        "enum__swipe_action__cycle_to_previous_keyboard_mode",
        "",
    ),
    (
        "CYCLE_TO_NEXT_KEYBOARD_MODE",
        "enum__swipe_action__cycle_to_next_keyboard_mode",
        "",
    ),
    ("DELETE_WORD", "enum__swipe_action__delete_word", ""),
    ("HIDE_KEYBOARD", "enum__swipe_action__hide_keyboard", ""),
    ("INSERT_SPACE", "enum__swipe_action__insert_space", ""),
    ("MOVE_CURSOR_UP", "enum__swipe_action__move_cursor_up", ""),
    (
        "MOVE_CURSOR_DOWN",
        "enum__swipe_action__move_cursor_down",
        "",
    ),
    (
        "MOVE_CURSOR_LEFT",
        "enum__swipe_action__move_cursor_left",
        "",
    ),
    (
        "MOVE_CURSOR_RIGHT",
        "enum__swipe_action__move_cursor_right",
        "",
    ),
    (
        "MOVE_CURSOR_START_OF_LINE",
        "enum__swipe_action__move_cursor_start_of_line",
        "",
    ),
    (
        "MOVE_CURSOR_END_OF_LINE",
        "enum__swipe_action__move_cursor_end_of_line",
        "",
    ),
    (
        "MOVE_CURSOR_START_OF_PAGE",
        "enum__swipe_action__move_cursor_start_of_page",
        "",
    ),
    (
        "MOVE_CURSOR_END_OF_PAGE",
        "enum__swipe_action__move_cursor_end_of_page",
        "",
    ),
    ("SHIFT", "enum__swipe_action__shift", ""),
    ("REDO", "enum__swipe_action__redo", ""),
    ("UNDO", "enum__swipe_action__undo", ""),
    (
        "SWITCH_TO_CLIPBOARD_CONTEXT",
        "enum__swipe_action__switch_to_clipboard_context",
        "",
    ),
    (
        "SWITCH_TO_MEDIA_CONTEXT",
        "enum__swipe_action__switch_to_media_context",
        "",
    ),
    (
        "SHOW_INPUT_METHOD_PICKER",
        "enum__swipe_action__show_input_method_picker",
        "",
    ),
    (
        "SHOW_SUBTYPE_PICKER",
        "enum__swipe_action__show_subtype_picker",
        "",
    ),
    (
        "SWITCH_TO_PREV_SUBTYPE",
        "enum__swipe_action__switch_to_prev_subtype",
        "",
    ),
    (
        "SWITCH_TO_NEXT_SUBTYPE",
        "enum__swipe_action__switch_to_next_subtype",
        "",
    ),
    (
        "SWITCH_TO_PREV_KEYBOARD",
        "enum__swipe_action__switch_to_prev_keyboard",
        "",
    ),
    (
        "TOGGLE_SMARTBAR_VISIBILITY",
        "enum__swipe_action__toggle_smartbar_visibility",
        "",
    ),
]);
const DELETE_SWIPE: Choices = entries(&[
    ("NO_ACTION", "enum__swipe_action__no_action", ""),
    (
        "DELETE_CHARACTERS_PRECISELY",
        "enum__swipe_action__delete_characters_precisely",
        "",
    ),
    ("DELETE_WORD", "enum__swipe_action__delete_word", ""),
    (
        "DELETE_WORDS_PRECISELY",
        "enum__swipe_action__delete_words_precisely",
        "",
    ),
    (
        "SELECT_CHARACTERS_PRECISELY",
        "enum__swipe_action__select_characters_precisely",
        "",
    ),
    (
        "SELECT_WORDS_PRECISELY",
        "enum__swipe_action__select_words_precisely",
        "",
    ),
]);
const DELETE_LONG_PRESS: Choices = entries(&[
    (
        "DELETE_CHARACTER",
        "enum__swipe_action__delete_character",
        "",
    ),
    ("DELETE_WORD", "enum__swipe_action__delete_word", ""),
]);
const MILLISECONDS: Slider = single(unit("unit__milliseconds__symbol"));
const VELOCITY: Slider = single(unit("unit__display_pixel_per_seconds__symbol"));
const DISTANCE: Slider = single(unit("unit__display_pixel__symbol"));
const GLIDE: super::Cond = Yes("glide__enabled");
const ROWS: &[Row] = &[
    heading("pref__glide__title"),
    toggle(
        "glide__enabled",
        "pref__glide__enabled__label",
        "pref__glide__enabled__summary",
    ),
    toggle(
        "glide__show_trail",
        "pref__glide__show_trail__label",
        "pref__glide__show_trail__summary",
    )
    .on(GLIDE),
    slider(
        "glide__trail_fade_duration",
        "pref__glide_trail_fade_duration",
        &MILLISECONDS,
    )
    .on(All(&[GLIDE, Yes("glide__show_trail")])),
    toggle(
        "glide__show_preview",
        "pref__glide__show_preview",
        "pref__glide__show_preview__summary",
    )
    .on(GLIDE),
    slider(
        "glide__preview_refresh_delay",
        "pref__glide_preview_refresh_delay",
        &MILLISECONDS,
    )
    .on(All(&[GLIDE, Yes("glide__show_preview")])),
    toggle(
        "glide__immediate_backspace_deletes_word",
        "pref__glide__immediate_backspace_deletes_word__label",
        "pref__glide__immediate_backspace_deletes_word__summary",
    )
    .on(GLIDE),
    heading("pref__gestures__general_title"),
    list(
        "gestures__swipe_up",
        "pref__gestures__swipe_up__label",
        &GENERAL,
    )
    .on(No("glide__enabled")),
    list(
        "gestures__swipe_down",
        "pref__gestures__swipe_down__label",
        &GENERAL,
    )
    .on(No("glide__enabled")),
    list(
        "gestures__swipe_left",
        "pref__gestures__swipe_left__label",
        &GENERAL,
    )
    .on(No("glide__enabled")),
    list(
        "gestures__swipe_right",
        "pref__gestures__swipe_right__label",
        &GENERAL,
    )
    .on(No("glide__enabled")),
    heading("pref__gestures__space_bar_title"),
    list(
        "gestures__space_bar_swipe_up",
        "pref__gestures__space_bar_swipe_up__label",
        &GENERAL,
    ),
    list(
        "gestures__space_bar_swipe_left",
        "pref__gestures__space_bar_swipe_left__label",
        &GENERAL,
    ),
    list(
        "gestures__space_bar_swipe_right",
        "pref__gestures__space_bar_swipe_right__label",
        &GENERAL,
    ),
    list(
        "gestures__space_bar_long_press",
        "pref__gestures__space_bar_long_press__label",
        &GENERAL,
    ),
    heading("pref__gestures__other_title"),
    list(
        "gestures__delete_key_swipe_left",
        "pref__gestures__delete_key_swipe_left__label",
        &DELETE_SWIPE,
    ),
    list(
        "gestures__delete_key_long_press",
        "pref__gestures__delete_key_long_press__label",
        &DELETE_LONG_PRESS,
    ),
    slider(
        "gestures__swipe_velocity_threshold",
        "pref__gestures__swipe_velocity_threshold__label",
        &VELOCITY,
    ),
    slider(
        "gestures__swipe_distance_threshold",
        "pref__gestures__swipe_distance_threshold__label",
        &DISTANCE,
    ),
];
pub(super) const SCREEN: Screen = Screen {
    route: "settings/gestures",
    title: "settings__gestures__title",
    icon_space: true,
    preview: true,
    rows: ROWS,
};
