//! MediaScreen (`iconSpaceReserved = true`) and its DeleteEmojiHistoryConfirmDialog.
use super::{
    Ask, Choices, Cond::*, Kind, Row, Screen, Slider, dual, entries, heading, list, plural, row,
    single, slider, toggle,
};
/// DeleteEmojiHistoryConfirmDialog: the reset confirmation naming what it deletes.
const fn reset(which: &'static [&'static str], name: &'static str) -> Ask {
    Ask {
        title: "action__reset_confirm_title",
        message: "action__reset_confirm_message",
        name,
        dismiss: "action__no",
        confirm: "action__yes",
        effect: which,
        toast: "",
    }
}
const RESET_PINNED: Ask = reset(
    &["deleteEmojiHistory", "pinned"],
    "prefs__media__emoji_history_pinned_reset__name",
);
const RESET_RECENT: Ask = reset(
    &["deleteEmojiHistory", "recent"],
    "prefs__media__emoji_history_reset__name",
);
const SKIN_TONE: Choices = Choices {
    entries: &[
        ("DEFAULT", "enum__emoji_skin_tone__default", ""),
        (
            "LIGHT_SKIN_TONE",
            "enum__emoji_skin_tone__light_skin_tone",
            "",
        ),
        (
            "MEDIUM_LIGHT_SKIN_TONE",
            "enum__emoji_skin_tone__medium_light_skin_tone",
            "",
        ),
        (
            "MEDIUM_SKIN_TONE",
            "enum__emoji_skin_tone__medium_skin_tone",
            "",
        ),
        (
            "MEDIUM_DARK_SKIN_TONE",
            "enum__emoji_skin_tone__medium_dark_skin_tone",
            "",
        ),
        (
            "DARK_SKIN_TONE",
            "enum__emoji_skin_tone__dark_skin_tone",
            "",
        ),
    ],
    selected_only: false,
    emoji: &[
        "\u{1F44B}",
        "\u{1F44B}\u{1F3FB}",
        "\u{1F44B}\u{1F3FC}",
        "\u{1F44B}\u{1F3FD}",
        "\u{1F44B}\u{1F3FE}",
        "\u{1F44B}\u{1F3FF}",
    ],
};
const STRATEGY: Choices = entries(&[
    (
        "AUTO_SORT_PREPEND",
        "enum__emoji_history_update_strategy__auto_sort_prepend",
        "enum__emoji_history_update_strategy__auto_sort_prepend__description",
    ),
    (
        "AUTO_SORT_APPEND",
        "enum__emoji_history_update_strategy__auto_sort_append",
        "enum__emoji_history_update_strategy__auto_sort_append__description",
    ),
    (
        "MANUAL_SORT_PREPEND",
        "enum__emoji_history_update_strategy__manual_sort_prepend",
        "enum__emoji_history_update_strategy__manual_sort_prepend__description",
    ),
    (
        "MANUAL_SORT_APPEND",
        "enum__emoji_history_update_strategy__manual_sort_append",
        "enum__emoji_history_update_strategy__manual_sort_append__description",
    ),
]);
const SUGGESTION_TYPE: Choices = entries(&[
    (
        "LEADING_COLON",
        "enum__emoji_suggestion_type__leading_colon",
        "enum__emoji_suggestion_type__leading_colon__description",
    ),
    (
        "INLINE_TEXT",
        "enum__emoji_suggestion_type__inline_text",
        "enum__emoji_suggestion_type__inline_text__description",
    ),
]);
/// `EmojiHistory.MaxSizeUnlimited` (0) reads "Unlimited".
const MAX_SIZE: Slider = dual(
    "emoji__history_recent_max_size",
    plural("unit__items__written", "general__unlimited"),
    ["emoji__history__pinned", "emoji__history__recent"],
);
const CHARACTERS: Slider = single(plural("unit__characters__written", ""));
const CANDIDATES: Slider = single(plural("unit__candidates__written", ""));
const HISTORY: super::Cond = Yes("emoji__history_enabled");
const SUGGESTION: super::Cond = Yes("emoji__suggestion_enabled");
const ROWS: &[Row] = &[
    list(
        "emoji__preferred_skin_tone",
        "prefs__media__emoji_preferred_skin_tone",
        &SKIN_TONE,
    ),
    heading("prefs__media__emoji_history__title"),
    toggle(
        "emoji__history_enabled",
        "prefs__media__emoji_history_enabled",
        "prefs__media__emoji_history_enabled__summary",
    )
    .icon("schedule"),
    list(
        "emoji__history_pinned_update_strategy",
        "prefs__media__emoji_history_pinned_update_strategy",
        &STRATEGY,
    )
    .on(HISTORY),
    list(
        "emoji__history_recent_update_strategy",
        "prefs__media__emoji_history_recent_update_strategy",
        &STRATEGY,
    )
    .on(HISTORY),
    slider(
        "emoji__history_pinned_max_size",
        "prefs__media__emoji_history_max_size",
        &MAX_SIZE,
    )
    .on(HISTORY),
    row(
        "prefs__media__emoji_history_pinned_reset",
        "prefs__media__emoji_history_pinned_reset",
        "",
        Kind::Confirm(&RESET_PINNED),
    )
    .on(HISTORY),
    row(
        "prefs__media__emoji_history_reset",
        "prefs__media__emoji_history_reset",
        "",
        Kind::Confirm(&RESET_RECENT),
    )
    .on(HISTORY),
    heading("prefs__media__emoji_suggestion__title"),
    toggle(
        "emoji__suggestion_enabled",
        "prefs__media__emoji_suggestion_enabled",
        "prefs__media__emoji_suggestion_enabled__summary",
    )
    .icon("emoji_symbols"),
    list(
        "emoji__suggestion_type",
        "prefs__media__emoji_suggestion_type",
        &SUGGESTION_TYPE,
    )
    .on(SUGGESTION),
    toggle(
        "emoji__suggestion_update_history",
        "prefs__media__emoji_suggestion_update_history",
        "prefs__media__emoji_suggestion_update_history__summary",
    )
    .on(All(&[SUGGESTION, HISTORY])),
    toggle(
        "emoji__suggestion_candidate_show_name",
        "prefs__media__emoji_suggestion_candidate_show_name",
        "prefs__media__emoji_suggestion_candidate_show_name__summary",
    )
    .on(SUGGESTION),
    slider(
        "emoji__suggestion_query_min_length",
        "prefs__media__emoji_suggestion_query_min_length",
        &CHARACTERS,
    )
    .on(SUGGESTION),
    slider(
        "emoji__suggestion_candidate_max_count",
        "prefs__media__emoji_suggestion_candidate_max_count",
        &CANDIDATES,
    )
    .on(SUGGESTION),
];
pub(super) const SCREEN: Screen = Screen {
    route: "settings/media",
    title: "settings__media__title",
    icon_space: true,
    preview: true,
    rows: ROWS,
};
