//! ClipboardScreen.
use super::{
    Choices, Cond::*, Row, Screen, Slider, dual, entries, heading, list, plural, single, slider,
    toggle, unit,
};
const SYNC: Choices = entries(&[
    (
        "NO_EVENTS",
        "enum__clipboard_sync_behavior__no_events",
        "enum__clipboard_sync_behavior__no_events__description",
    ),
    (
        "ONLY_CLEAR_EVENTS",
        "enum__clipboard_sync_behavior__only_clear_events",
        "enum__clipboard_sync_behavior__only_clear_events__description",
    ),
    (
        "ONLY_SET_EVENTS",
        "enum__clipboard_sync_behavior__only_set_events",
        "enum__clipboard_sync_behavior__only_set_events__description",
    ),
    (
        "ALL_EVENTS",
        "enum__clipboard_sync_behavior__all_events",
        "enum__clipboard_sync_behavior__all_events__description",
    ),
]);
const TIMEOUT: Slider = single(unit("pref__clipboard__suggestion_timeout__summary"));
/// `CLIPBOARD_HISTORY_NUM_GRID_COLUMNS_AUTO` (0) reads "Automatic"; others are bare numbers.
const COLUMNS: Slider = dual(
    "clipboard__history_num_grid_columns_landscape",
    super::Unit {
        key: "",
        plural: false,
        zero: "general__auto",
    },
    [
        "screen_orientation__portrait",
        "screen_orientation__landscape",
    ],
);
const MINUTES: Slider = single(plural("unit__minutes__written", ""));
const SECONDS: Slider = single(plural("unit__seconds__written", ""));
const ITEMS: Slider = single(plural("unit__items__written", ""));
const INTERNAL: super::Cond = Yes("clipboard__use_internal_clipboard");
const HISTORY: super::Cond = Yes("clipboard__history_enabled");
const ROWS: &[Row] = &[
    toggle(
        "clipboard__use_internal_clipboard",
        "pref__clipboard__use_internal_clipboard__label",
        "pref__clipboard__use_internal_clipboard__summary",
    ),
    list(
        "clipboard__sync_to_floris",
        "pref__clipboard__sync_from_system_clipboard__label",
        &SYNC,
    )
    .on(INTERNAL),
    list(
        "clipboard__sync_to_system",
        "pref__clipboard__sync_to_system_clipboard__label",
        &SYNC,
    )
    .on(INTERNAL),
    heading("pref__clipboard__group_clipboard_suggestion__label"),
    toggle(
        "clipboard__suggestion_enabled",
        "pref__clipboard__suggestion_enabled__label",
        "pref__clipboard__suggestion_enabled__summary",
    ),
    slider(
        "clipboard__suggestion_timeout",
        "pref__clipboard__suggestion_timeout__label",
        &TIMEOUT,
    )
    .on(Yes("clipboard__suggestion_enabled")),
    heading("pref__clipboard__group_clipboard_history__label"),
    toggle(
        "clipboard__history_enabled",
        "pref__clipboard__enable_clipboard_history__label",
        "pref__clipboard__enable_clipboard_history__summary",
    ),
    slider(
        "clipboard__history_num_grid_columns_portrait",
        "pref__clipboard__num_history_grid_columns__label",
        &COLUMNS,
    )
    .on(HISTORY),
    toggle(
        "clipboard__history_auto_clean_old_enabled",
        "pref__clipboard__clean_up_old__label",
        "",
    )
    .on(HISTORY),
    slider(
        "clipboard__history_auto_clean_old_after",
        "pref__clipboard__clean_up_after__label",
        &MINUTES,
    )
    .on(All(&[
        HISTORY,
        Yes("clipboard__history_auto_clean_old_enabled"),
    ])),
    toggle(
        "clipboard__history_auto_clean_sensitive_enabled",
        "pref__clipboard__auto_clean_sensitive__label",
        "",
    )
    .on(HISTORY),
    slider(
        "clipboard__history_auto_clean_sensitive_after",
        "pref__clipboard__auto_clean_sensitive_after__label",
        &SECONDS,
    )
    .on(All(&[
        HISTORY,
        Yes("clipboard__history_auto_clean_sensitive_enabled"),
    ])),
    toggle(
        "clipboard__history_size_limit_enabled",
        "pref__clipboard__limit_history_size__label",
        "",
    )
    .on(HISTORY),
    slider(
        "clipboard__history_size_limit",
        "pref__clipboard__max_history_size__label",
        &ITEMS,
    )
    .on(All(&[
        HISTORY,
        Yes("clipboard__history_size_limit_enabled"),
    ])),
    toggle(
        "clipboard__history_hide_on_paste",
        "pref__clipboard__history_hide_on_paste__label",
        "",
    )
    .on(HISTORY),
    toggle(
        "clipboard__history_hide_on_next_text_field",
        "pref__clipboard__history_hide_on_next_text_field__label",
        "",
    )
    .on(HISTORY),
    toggle(
        "clipboard__clear_primary_clip_affects_history_if_unpinned",
        "pref__clipboard__clear_primary_clip_affects_history_if_unpinned__label",
        "pref__clipboard__clear_primary_clip_affects_history_if_unpinned__summary",
    )
    .on(HISTORY),
];
pub(super) const SCREEN: Screen = Screen {
    route: "settings/clipboard",
    title: "settings__clipboard__title",
    icon_space: true,
    preview: true,
    rows: ROWS,
};
