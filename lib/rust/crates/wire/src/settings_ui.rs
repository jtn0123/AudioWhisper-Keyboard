//! `RenderNative.settings`: the native settings screens' operations, answered in awnative's
//! `render_settings`, and the rows they pass. Every operation answers with a [`reply`] row.

crate::declare! {
    /// The operations `RenderNative.settings` takes.
    pub const OPS: enum SettingsUiOp(i32) {
        /// The strings and icons the screens draw, as name and value pairs.
        RESOURCES = 1,
        /// The preferences, as key and value pairs.
        PREFERENCES = 2,
        /// The screen's size, density, colours and type scale.
        CONFIG = 3,
        /// A route to open.
        NAVIGATE = 4,
        /// A control id, tapped.
        ACTIVATE = 5,
        /// A [`SettingsUiBack`](back::PHASES) phase, then the gesture's progress.
        BACK = 6,
        /// A key event: its action, key code, character and whether shift is down.
        KEY = 7,
        /// A frame time, in nanoseconds.
        FRAME = 8,
        /// A [`SettingsUiEdit`](edit::OPS), then its fields.
        EDIT = 9,
        /// Nothing: the reply carries the saved state after its [`reply`] row.
        SAVE = 10,
        /// The saved state to restore.
        RESTORE = 11,
        /// Drop the touch in progress.
        CANCEL_TOUCH = 12,
        /// A scroll by a fraction of the screen's height.
        SCROLL = 14,
        /// A slider's control id and its new value.
        SLIDER = 15,
        /// The plural strings the screens need: the reply's `QUERY`, one per line.
        PLURALS = 16,
        /// A control id and a `RenderProtocol` scroll action.
        ACCESSIBILITY_SCROLL = 17,
        /// The string names the screens need: the reply's `QUERY`, one per line.
        STRING_KEYS = 18,
        /// The routes drawn natively: the reply's `QUERY`, one per line.
        ROUTES = 19,
        /// Facts about the device, as name and value pairs.
        FACTS = 20,
        /// Documents a load effect asked for, as name and text pairs.
        DOCUMENTS = 21,
        /// A [`SettingsUiFiles`](files::report::KINDS) report: how the file picker closed, or
        /// how file work ended.
        FILES = 22,
        /// The language resources a [`SettingsUiLanguageEffect`](languages::effect::KINDS) asked
        /// for: [`SettingsUiLanguage`](languages::row::FIELDS) rows.
        LANGUAGES = 23,
        /// A user dictionary a [`SettingsUiWordEffect`](words::effect::KINDS) asked for:
        /// [`SettingsUiWord`](words::row::FIELDS) rows.
        WORDS = 24,
        /// An asset a [`SettingsUiTextEffect`](text::effect::KINDS) asked for: its path, a
        /// [`SettingsUiAssetOutcome`](text::outcome::KINDS), then the text or the error message.
        ASSET = 25,
        /// A [`SettingsUiSelection`](text::selection::KINDS) on the read-only text, then its
        /// fields.
        SELECTION = 26,
        /// The installed themes a [`SettingsUiThemeEffect`](themes::effect::KINDS) asked for:
        /// [`SettingsUiTheme`](themes::row::FIELDS) rows.
        THEMES = 27,
        /// The activity opened without a saved state: the route of the `ui://florisboard/` link
        /// it was opened with, or nothing. Rust picks the first screen: Setup until it is
        /// finished, unless the link names a screen.
        START = 40,
        /// The notification permission request's answer: a
        /// [`SettingsUiPermission`](setup::answer::FIELDS) packet.
        PERMISSION = 41,
    }
}

/// Every reply's head.
pub mod reply {
    crate::declare! {
        /// Every reply's head. After `SAVE`, the saved state follows.
        pub const FIELDS: fields SettingsUiReply {
            /// The screen's state, as JSON.
            STATE,
            /// The focused field's text.
            TEXT,
            /// What a query asked for.
            QUERY,
        }
    }
}

/// The phases of a back gesture.
pub mod back {
    crate::declare! {
        /// The phases of a back gesture.
        pub const PHASES: object SettingsUiBack(i32) {
            STARTED = 0,
            PROGRESSED = 1,
            CANCELLED = 2,
            /// The gesture finished: go back.
            PRESSED = 3,
        }
    }
}

/// What `EDIT` does to the focused field.
pub mod edit {
    crate::declare! {
        /// What `EDIT` does to the focused field, as an input connection asks.
        pub const OPS: enum SettingsUiEdit(str) {
            /// Text, then the new cursor position.
            COMMIT = "commit",
            /// Composing text, then the new cursor position.
            COMPOSE = "compose",
            /// A selection's start and end.
            SELECT = "select",
            /// The composing region's start and end.
            REGION = "region",
            FINISH = "finish",
            BEGIN = "begin",
            END = "end",
            CLOSE = "close",
            /// How many characters to delete before and after the cursor.
            DELETE = "delete",
            /// How many code points to delete before and after the cursor.
            CODEPOINTS = "codepoints",
            /// A [`SettingsUiQuery`](super::query::KINDS) and a length: the reply's `QUERY`.
            QUERY = "query",
            /// Focus the editor, and the field with the control id that may follow.
            FOCUS = "focus",
            DISMISS_TOOLBAR = "dismissToolbar",
            SELECT_ALL = "selectAll",
            CUT = "cut",
            BLUR = "blur",
            /// Replace the field's text.
            SET = "set",
        }
    }
}

/// What a `QUERY` asks for.
pub mod query {
    crate::declare! {
        /// What a `QUERY` asks for.
        pub const KINDS: object SettingsUiQuery(str) {
            /// Up to the length's characters before the selection.
            BEFORE = "before",
            /// Up to the length's characters after the selection.
            AFTER = "after",
            SELECTED = "selected",
            /// The caps mode at the cursor, for the length's requested modes.
            CAPS = "caps",
        }
    }
}

pub mod files;
pub mod languages;
pub mod setup;
pub mod text;
pub mod themes;
pub mod words;

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &OPS,
    &reply::FIELDS,
    &back::PHASES,
    &edit::OPS,
    &query::KINDS,
    &files::effect::KINDS,
    &files::report::KINDS,
    &files::mode::KINDS,
    &files::outcome::KINDS,
    &files::job::KINDS,
    &files::result::KINDS,
    &files::pick::FIELDS,
    &files::picked::FIELDS,
    &files::document::FIELDS,
    &files::work::FIELDS,
    &files::done::FIELDS,
    &languages::effect::KINDS,
    &languages::kind::KINDS,
    &languages::row::FIELDS,
    &setup::effect::KINDS,
    &setup::answer::FIELDS,
    &setup::fact::KEYS,
    &text::effect::KINDS,
    &text::outcome::KINDS,
    &text::selection::KINDS,
    &text::node::IDS,
    &themes::effect::KINDS,
    &themes::kind::KINDS,
    &themes::row::FIELDS,
    &words::effect::KINDS,
    &words::dictionary::KINDS,
    &words::kind::KINDS,
    &words::outcome::KINDS,
    &words::row::FIELDS,
];
