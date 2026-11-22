//! The theme editor (`settings_ui/theme_editor`): ThemeEditorScreen's component box and rules, the
//! rule editor and the component meta editor, which Compose drew as dialogs over it, and its
//! fine-tune dialog as a screen of its three preferences. Rows whose kind is `ThemeEdit` are
//! painted by that module; the rule and meta editors' Cancel and Apply are the item editor's
//! bottom bar (`Lang::Cancel`, `Lang::Save`), and its questions are language dialogs.
use super::{Choices, Cond, Kind, Row, Screen, languages::Lang, row};
use crate::settings_ui::theme_editor as screens;

/// A theme editor screen's own parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// The loading message, the stylesheet error page, the missing workspace and a failed save.
    Status,
    /// The component box: label, name, authors, night flag and path, and its Edit button.
    Meta,
    /// Fine-tune, a ListItem where Compose had a top bar button.
    Tune,
    /// The rules in `compare_order`, each with its properties, the undo bar and Add rule.
    Rules,
    /// The rule editor's element, and the font's name for `@font`.
    Element,
    Font,
    /// The rule editor's selector chips.
    Selectors,
    /// The rule editor's attribute boxes: 0 codes, 1 modes, 2 shift states.
    Attribute(usize),
    /// The rule editor's message: the rule exists, or no codes are given.
    RuleError,
    /// The meta editor's fields: 0 the label, 1 the authors, comma-separated.
    MetaField(usize),
    /// The meta editor's night theme switch.
    Night,
}

const fn part(label: &'static str, p: Part) -> Row {
    row("", label, "", Kind::ThemeEdit(p))
}
const fn hidden(label: &'static str, summary: &'static str, l: Lang) -> Row {
    let mut r = row("", label, summary, Kind::Lang(l));
    r.visible = Cond::Never;
    r
}
const CANCEL: Row = row("", "action__cancel", "", Kind::Lang(Lang::Cancel));
const DISCARD: Row = hidden(
    "action__discard_confirm_title",
    "action__discard_confirm_message",
    Lang::Discard,
);
const EDIT_ROWS: &[Row] = &[
    part("", Part::Status),
    part("", Part::Meta),
    part("settings__theme_editor__fine_tune__title", Part::Tune),
    part("settings__theme_editor__no_rules_defined", Part::Rules),
    DISCARD,
];
const RULE: [Row; 8] = [
    part("", Part::RuleError),
    part("settings__theme_editor__rule_name", Part::Element),
    part("snygg__rule_annotation__font_name", Part::Font),
    part("settings__theme_editor__rule_selectors", Part::Selectors),
    part("settings__theme_editor__rule_codes", Part::Attribute(0)),
    part("settings__theme_editor__rule_modes", Part::Attribute(1)),
    part(
        "settings__theme_editor__rule_shift_states",
        Part::Attribute(2),
    ),
    CANCEL,
];
const RULE_ADD_ROWS: &[Row] = &[
    RULE[0],
    RULE[1],
    RULE[2],
    RULE[3],
    RULE[4],
    RULE[5],
    RULE[6],
    RULE[7],
    row("", "action__add", "", Kind::Lang(Lang::Save)),
];
/// Deleting asks first, though the deleted rule can be put back from the rules.
const RULE_EDIT_ROWS: &[Row] = &[
    RULE[0],
    RULE[1],
    RULE[2],
    RULE[3],
    RULE[4],
    RULE[5],
    RULE[6],
    RULE[7],
    row("", "action__apply", "", Kind::Lang(Lang::Save)),
    hidden(
        "action__delete_confirm_title",
        "settings__theme_editor__delete_rule_message",
        Lang::DeleteRule,
    ),
];
const META_ROWS: &[Row] = &[
    part("ext__meta__label", Part::MetaField(0)),
    part("ext__meta__authors", Part::MetaField(1)),
    part(
        "settings__theme_editor__component_meta_is_night_theme",
        Part::Night,
    ),
    CANCEL,
    row("", "action__apply", "", Kind::Lang(Lang::Save)),
];
const LEVEL: Choices = described(&[
    (
        "BASIC",
        "enum__snygg_level__basic",
        "enum__snygg_level__basic__description",
    ),
    (
        "ADVANCED",
        "enum__snygg_level__advanced",
        "enum__snygg_level__advanced__description",
    ),
    (
        "DEVELOPER",
        "enum__snygg_level__developer",
        "enum__snygg_level__developer__description",
    ),
]);
const COLORS: Choices = Choices {
    entries: &[
        (
            "HEX",
            "enum__color_representation__hex",
            "general__example_given",
        ),
        (
            "RGB",
            "enum__color_representation__rgb",
            "general__example_given",
        ),
        (
            "HSV",
            "enum__color_representation__hsv",
            "general__example_given",
        ),
    ],
    selected_only: true,
    emoji: &[
        "#4caf50ff",
        "rgba(76, 175, 80, 1.0)",
        "hsva(122, 56, 68, 1.0)",
    ],
};
const AFTER_DIALOGS: Choices = described(&[
    (
        "ALWAYS",
        "enum__display_kbd_after_dialogs__always",
        "enum__display_kbd_after_dialogs__always__description",
    ),
    (
        "NEVER",
        "enum__display_kbd_after_dialogs__never",
        "enum__display_kbd_after_dialogs__never__description",
    ),
    (
        "REMEMBER",
        "enum__display_kbd_after_dialogs__remember",
        "enum__display_kbd_after_dialogs__remember__description",
    ),
]);
const TUNE_ROWS: &[Row] = &[
    super::list(
        "theme__editor_level",
        "settings__theme_editor__fine_tune__level",
        &LEVEL,
    ),
    super::list(
        "theme__editor_color_representation",
        "settings__theme_editor__fine_tune__color_representation",
        &COLORS,
    ),
    super::list(
        "theme__editor_display_kbd_after_dialogs",
        "settings__theme_editor__fine_tune__display_kbd_after_dialogs",
        &AFTER_DIALOGS,
    ),
];
/// Jetpref's `showDescriptionOnlyIfSelected`, which all three lists set.
const fn described(entries: &'static [super::Entry]) -> Choices {
    Choices {
        entries,
        selected_only: true,
        emoji: &[],
    }
}
/// The rules and fine-tune keep the keyboard's preview field under them, as ThemeEditorScreen
/// did; the rule and meta editors have the item editor's bottom bar instead.
const fn screen(
    route: &'static str,
    title: &'static str,
    rows: &'static [Row],
    preview: bool,
) -> Screen {
    Screen {
        route,
        title,
        icon_space: false,
        preview,
        rows,
    }
}
pub const EDIT: Screen = screen(
    screens::EDIT,
    "ext__editor__edit_component__title_theme",
    EDIT_ROWS,
    true,
);
pub const RULE_ADD: Screen = screen(
    screens::RULE_ADD,
    "settings__theme_editor__add_rule",
    RULE_ADD_ROWS,
    false,
);
pub const RULE_EDIT: Screen = screen(
    screens::RULE_EDIT,
    "settings__theme_editor__edit_rule",
    RULE_EDIT_ROWS,
    false,
);
pub const META: Screen = screen(
    screens::META,
    "ext__editor__metadata__title",
    META_ROWS,
    false,
);
pub const TUNE: Screen = screen(
    screens::TUNE,
    "settings__theme_editor__fine_tune__title",
    TUNE_ROWS,
    true,
);
