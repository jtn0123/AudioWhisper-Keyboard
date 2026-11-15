//! SmartbarScreen.
use super::{Choices, Cond::*, Kind, Row, Screen, entries, heading, list, row, toggle};
const LAYOUT: Choices = entries(&[
    (
        "SUGGESTIONS_ONLY",
        "enum__smartbar_layout__suggestions_only",
        "enum__smartbar_layout__suggestions_only__description",
    ),
    (
        "ACTIONS_ONLY",
        "enum__smartbar_layout__actions_only",
        "enum__smartbar_layout__actions_only__description",
    ),
    (
        "SUGGESTIONS_ACTIONS_SHARED",
        "enum__smartbar_layout__suggestions_action_shared",
        "enum__smartbar_layout__suggestions_action_shared__description",
    ),
    (
        "SUGGESTIONS_ACTIONS_EXTENDED",
        "enum__smartbar_layout__suggestions_actions_extended",
        "enum__smartbar_layout__suggestions_actions_extended__description",
    ),
]);
const DISPLAY: Choices = entries(&[
    ("CLASSIC", "enum__candidates_display_mode__classic", ""),
    ("DYNAMIC", "enum__candidates_display_mode__dynamic", ""),
    (
        "DYNAMIC_SCROLLABLE",
        "enum__candidates_display_mode__dynamic_scrollable",
        "",
    ),
]);
const PLACEMENT: Choices = Choices {
    entries: &[
        (
            "ABOVE_CANDIDATES",
            "enum__extended_actions_placement__above_candidates",
            "enum__extended_actions_placement__above_candidates__description",
        ),
        (
            "BELOW_CANDIDATES",
            "enum__extended_actions_placement__below_candidates",
            "enum__extended_actions_placement__below_candidates__description",
        ),
        (
            "OVERLAY_APP_UI",
            "enum__extended_actions_placement__overlay_app_ui",
            "enum__extended_actions_placement__overlay_app_ui__description",
        ),
    ],
    selected_only: true,
    emoji: &[],
};
const ON: super::Cond = Yes("smartbar__enabled");
const SHARED: super::Cond = Is("smartbar__layout", "SUGGESTIONS_ACTIONS_SHARED");
const EXTENDED: super::Cond = Is("smartbar__layout", "SUGGESTIONS_ACTIONS_EXTENDED");
const ROWS: &[Row] = &[
    toggle(
        "smartbar__enabled",
        "pref__smartbar__enabled__label",
        "pref__smartbar__enabled__summary",
    ),
    list("smartbar__layout", "pref__smartbar__layout__label", &LAYOUT).on(ON),
    heading("pref__smartbar__group_layout_specific__label"),
    list(
        "suggestion__display_mode",
        "pref__suggestion__display_mode__label",
        &DISPLAY,
    )
    .on(ON)
    .shown(Not("smartbar__layout", "ACTIONS_ONLY")),
    toggle(
        "smartbar__show_read_aloud",
        "pref__smartbar__show_read_aloud__label",
        "pref__smartbar__show_read_aloud__summary",
    )
    .on(ON)
    .shown(Any(&[SHARED, EXTENDED])),
    toggle(
        "smartbar__flip_toggles",
        "pref__smartbar__flip_toggles__label",
        "pref__smartbar__flip_toggles__summary",
    )
    .on(ON)
    .shown(Any(&[SHARED, EXTENDED])),
    row(
        "pref__smartbar__shared_actions_auto_expand_collapse__label",
        "pref__smartbar__shared_actions_auto_expand_collapse__label",
        "pref__smartbar__shared_actions_auto_expand_collapse__summary",
        Kind::Fixed,
    )
    .on(Never)
    .shown(SHARED),
    list(
        "smartbar__extended_actions_placement",
        "pref__smartbar__extended_actions_placement__label",
        &PLACEMENT,
    )
    .on(ON)
    .shown(EXTENDED),
];
pub(super) const SCREEN: Screen = Screen {
    route: "settings/smartbar",
    title: "settings__smartbar__title",
    icon_space: true,
    preview: true,
    rows: ROWS,
};
