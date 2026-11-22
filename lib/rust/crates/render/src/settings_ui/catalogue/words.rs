//! The user dictionary screens (`settings_ui/words`), for the Floris and the system dictionary
//! alike: the languages words are stored under, a language's words, and the word editor, which
//! adds and edits at two routes. Rows whose kind is `Words` are painted by that module; the
//! editor's Cancel and Save are the item editor's bottom bar (`Lang::Cancel`, `Lang::Save`).
use super::{Kind, Row, Screen, languages::Lang, row};
use crate::settings_ui::{items::Source, words as screens};

/// A user dictionary screen's own parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// The languages words are stored under, or the empty dictionary's card.
    Languages,
    /// An action ListItem: add a word, import, export, or open the system's own manager.
    Action(Act),
    /// The word list's search field.
    Search,
    /// The message when the search matches no word.
    NotFound,
    /// A field of the word editor: 0 the word, 1 its frequency, 2 its shortcut, 3 its locale.
    Field(usize),
}

/// What an action row does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Add,
    Import,
    Export,
    /// The system dictionary's "Open system manager UI".
    Manager,
}

const fn part(label: &'static str, p: Part) -> Row {
    row("", label, "", Kind::Words(p))
}
const fn action(label: &'static str, a: Act) -> Row {
    part(label, Part::Action(a))
}
const LANGUAGES: Row = part("settings__udm__no_words_in_dictionary", Part::Languages);
const ACTIONS: [Row; 3] = [
    action("settings__udm__dialog__title_add", Act::Add),
    action("action__import", Act::Import),
    action("action__export", Act::Export),
];
const FLORIS_ROWS: &[Row] = &[LANGUAGES, ACTIONS[0], ACTIONS[1], ACTIONS[2]];
const SYSTEM_ROWS: &[Row] = &[
    LANGUAGES,
    ACTIONS[0],
    ACTIONS[1],
    ACTIONS[2],
    action("settings__udm__open_system_manager_ui", Act::Manager),
];
/// A language's words: the search, then the editable list without Move up and Move down.
const WORD_ROWS: &[Row] = &[
    part("settings__udm__search_placeholder", Part::Search),
    part("settings__udm__search_not_found", Part::NotFound),
    row(
        "",
        "settings__udm__no_words_in_dictionary",
        "settings__udm__dialog__title_add",
        Kind::List(Source::Words),
    ),
];
const FIELDS: [Row; 4] = [
    part("settings__udm__dialog__word_label", Part::Field(0)),
    part("settings__udm__dialog__freq_label", Part::Field(1)),
    part("settings__udm__dialog__shortcut_label", Part::Field(2)),
    part("settings__udm__dialog__locale_label", Part::Field(3)),
];
const CANCEL: Row = row("", "action__cancel", "", Kind::Lang(Lang::Cancel));
/// UserDictionaryScreen's add dialog: its fields, then Cancel and Add.
const ADD_ROWS: &[Row] = &[
    FIELDS[0],
    FIELDS[1],
    FIELDS[2],
    FIELDS[3],
    CANCEL,
    row("", "action__add", "", Kind::Lang(Lang::Save)),
];
/// The edit dialog: Cancel and Apply; its Delete is the top bar's.
const EDIT_ROWS: &[Row] = &[
    FIELDS[0],
    FIELDS[1],
    FIELDS[2],
    FIELDS[3],
    CANCEL,
    row("", "action__apply", "", Kind::Lang(Lang::Save)),
];
const fn screen(route: &'static str, title: &'static str, rows: &'static [Row]) -> Screen {
    Screen {
        route,
        title,
        icon_space: false,
        preview: false,
        rows,
    }
}
const FLORIS: &str = "settings__udm__title_floris";
const SYSTEM: &str = "settings__udm__title_system";
const ADD: &str = "settings__udm__dialog__title_add";
const EDIT: &str = "settings__udm__dialog__title_edit";
/// Each dictionary's languages, words, add and edit screens.
pub const FLORIS_LIST: Screen = screen(screens::FLORIS, FLORIS, FLORIS_ROWS);
pub const FLORIS_WORDS: Screen = screen(screens::FLORIS_WORDS, FLORIS, WORD_ROWS);
pub const FLORIS_ADD: Screen = screen(screens::FLORIS_ADD, ADD, ADD_ROWS);
pub const FLORIS_EDIT: Screen = screen(screens::FLORIS_EDIT, EDIT, EDIT_ROWS);
pub const SYSTEM_LIST: Screen = screen(screens::SYSTEM, SYSTEM, SYSTEM_ROWS);
pub const SYSTEM_WORDS: Screen = screen(screens::SYSTEM_WORDS, SYSTEM, WORD_ROWS);
pub const SYSTEM_ADD: Screen = screen(screens::SYSTEM_ADD, ADD, ADD_ROWS);
pub const SYSTEM_EDIT: Screen = screen(screens::SYSTEM_EDIT, EDIT, EDIT_ROWS);
