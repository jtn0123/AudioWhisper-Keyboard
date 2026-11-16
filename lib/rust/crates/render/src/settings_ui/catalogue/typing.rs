//! TypingScreen (`iconSpaceReserved = true`, `previewFieldVisible = true`), its
//! ForgetLearnedWordsDialog, the auto-space feedback card and SpellCheckerServiceSelector.
use super::{
    Ask, Card, Choices, Cond, Cond::*, Kind, Row, Screen, entries, heading, list, row, toggle,
};
const INCOGNITO: Choices = Choices {
    entries: &[
        (
            "FORCE_OFF",
            "enum__incognito_mode__force_off",
            "enum__incognito_mode__force_off__description",
        ),
        (
            "DYNAMIC_ON_OFF",
            "enum__incognito_mode__dynamic_on_off",
            "enum__incognito_mode__dynamic_on_off__description",
        ),
        (
            "FORCE_ON",
            "enum__incognito_mode__force_on",
            "enum__incognito_mode__force_on__description",
        ),
    ],
    selected_only: true,
    emoji: &[],
};
const LANGUAGE_MODE: Choices = entries(&[
    (
        "USE_SYSTEM_LANGUAGES",
        "enum__spelling_language_mode__use_system_languages",
        "",
    ),
    (
        "USE_KEYBOARD_SUBTYPES",
        "enum__spelling_language_mode__use_keyboard_subtypes",
        "",
    ),
]);
/// What the keyboard learned is erased only after this; the words themselves never leave Kotlin.
const FORGET: Ask = Ask {
    title: "pref__suggestion__forget_learned__label",
    message: "pref__suggestion__forget_learned__confirm",
    name: "",
    dismiss: "action__cancel",
    confirm: "action__delete",
    effect: &["forgetLearnedWords"],
    toast: "pref__suggestion__forget_learned__done",
};
const AUTO_SPACE_FEEDBACK: Card = Card {
    text: "pref__correction__auto_space_punctuation__feedback",
    link: "pref__correction__auto_space_punctuation__feedback_link",
    url: "florisboard__auto_space_feedback_url",
};
const SUGGESTIONS: Cond = Yes("suggestion__enabled");
const ROWS: &[Row] = &[
    heading("pref__suggestion__title"),
    toggle(
        "suggestion__enabled",
        "pref__suggestion__enabled__label",
        "pref__suggestion__enabled__summary",
    ),
    toggle(
        "suggestion__block_possibly_offensive",
        "pref__suggestion__block_possibly_offensive__label",
        "pref__suggestion__block_possibly_offensive__summary",
    )
    .on(SUGGESTIONS),
    toggle(
        "suggestion__api30_inline_suggestions_enabled",
        "pref__suggestion__api30_inline_suggestions_enabled__label",
        "pref__suggestion__api30_inline_suggestions_enabled__summary",
    ),
    list(
        "suggestion__incognito_mode",
        "pref__suggestion__incognito_mode__label",
        &INCOGNITO,
    )
    .icon("incognito"),
    toggle(
        "suggestion__learn_from_typing",
        "pref__suggestion__learn_from_typing__label",
        "pref__suggestion__learn_from_typing__summary",
    )
    .on(SUGGESTIONS),
    toggle(
        "suggestion__learn_tap_aim",
        "pref__suggestion__learn_tap_aim__label",
        "pref__suggestion__learn_tap_aim__summary",
    )
    .on(SUGGESTIONS),
    row(
        "pref__suggestion__forget_learned__label",
        "pref__suggestion__forget_learned__label",
        "pref__suggestion__forget_learned__summary",
        Kind::Confirm(&FORGET),
    ),
    heading("pref__correction__title"),
    toggle(
        "correction__auto_capitalization",
        "pref__correction__auto_capitalization__label",
        "pref__correction__auto_capitalization__summary",
    ),
    toggle(
        "correction__auto_space_punctuation",
        "pref__correction__auto_space_punctuation__label",
        "pref__correction__auto_space_punctuation__summary",
    )
    .icon("space_bar"),
    row(
        "pref__correction__auto_space_punctuation__feedback",
        "pref__correction__auto_space_punctuation__feedback",
        "",
        Kind::Card(&AUTO_SPACE_FEEDBACK),
    )
    .shown(Yes("correction__auto_space_punctuation")),
    toggle(
        "correction__remember_caps_lock_state",
        "pref__correction__remember_caps_lock_state__label",
        "pref__correction__remember_caps_lock_state__summary",
    ),
    toggle(
        "correction__double_space_period",
        "pref__correction__double_space_period__label",
        "pref__correction__double_space_period__summary",
    ),
    heading("pref__spelling__title"),
    row("pref__spelling__title#selector", "", "", Kind::Spell),
    list(
        "spelling__language_mode",
        "pref__spelling__language_mode__label",
        &LANGUAGE_MODE,
    )
    .icon("language")
    .on(FlorisSpellChecker),
    toggle(
        "spelling__use_contacts",
        "pref__spelling__use_contacts__label",
        "pref__spelling__use_contacts__summary",
    )
    .icon("contacts")
    .on(FlorisSpellChecker)
    .shown(Never),
    toggle(
        "spelling__use_udm_entries",
        "pref__spelling__use_udm_entries__label",
        "pref__spelling__use_udm_entries__summary",
    )
    .icon("library_books")
    .on(FlorisSpellChecker)
    .shown(Never),
    heading("settings__dictionary__title"),
    row(
        "settings__dictionary__title#route",
        "settings__dictionary__title",
        "",
        Kind::Route("settings/dictionary"),
    )
    .icon("library_books"),
];
pub const SCREEN: Screen = Screen {
    route: "settings/typing",
    title: "settings__typing__title",
    icon_space: true,
    preview: true,
    rows: ROWS,
};
