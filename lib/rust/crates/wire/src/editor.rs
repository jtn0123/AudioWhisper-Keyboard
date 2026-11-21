//! `EditorNative.call` and `EditorNative.attributes`: their operations, answered by
//! `editor::Session::checked_call` and `editor::attributes::call`, and the packets they pass.
//!
//! A call answers with a [`plan`] or a query. A query's first field is the operation that answers
//! it, so the platform sends its answer back as that operation.

crate::declare! {
    /// The operations `EditorNative.call` takes, answered by `editor::Session::checked_call`.
    pub const OPS: enum EditorOp(str) {
        /// A [`cursor`] row, then a flag of whether the editor can be read: a plan.
        UPDATE = "update",
        /// A [`cursor`] row: a plan.
        MASS = "mass",
        /// A [`cursor`] row: a flag of whether it reports an edit the keyboard has since followed
        /// with more.
        LATE = "late",
        /// The selection's start and end, then a flag of whether the editor can be read: a plan.
        START = "start",
        /// An [`extracted`] row: a plan.
        EXTRACTED = "extracted",
        /// A [`config`] row.
        CONFIG = "config",
        /// A [`context`] row.
        CONTEXT = "context",
        /// The field's initial caps mode flags.
        INPUT_INFO = "inputInfo",
        /// The operation that couldn't run: a plan.
        UNAVAILABLE = "unavailable",
        /// Pairs of composer rule keys and values.
        COMPOSER_RULES = "composerRules",
        /// An arrow key code, a count and a flag of whether shift is held: a plan.
        ARROW = "arrow",
        /// A key code, its meta state and a count: a plan.
        KEYS = "keys",
        /// An editor action: a plan.
        ACTION = "action",
        /// A [`char_input`] row: a plan.
        CHAR = "char",
        COMMIT = "commit",
        FINALIZE = "finalize",
        /// A length and the text that replaces that much before the cursor: a plan.
        REPLACE = "replace",
        /// The selection's start and end: a plan.
        SELECT = "select",
        /// A flag of whether after the cursor, a flag of whether in words, and a count: a plan.
        DELETE = "delete",
        REFRESH = "refresh",
        TYPE_CHAR = "typeChar",
        TYPE_TEXT = "typeText",
        COMPLETION = "completion",
        GESTURE = "gesture",
        BACKWARD = "backward",
        FORWARD = "forward",
        /// A length, the text to restore and a flag of whether a phantom space is owed: a plan.
        REVERT = "revert",
        /// A count, a unit and a scope: a plan.
        SELECT_AROUND = "selectAround",
        /// The text and how much it replaces: a plan.
        DICTATION = "dictation",
        UNDO_DICTATION = "undoDictation",
        FORGET_DICTATION = "forgetDictation",
        DOUBLE_SPACE = "doubleSpace",
        /// The subtype's language: a plan.
        RAW_ENTER = "rawEnter",
        ENTER = "enter",
        SELECT_ALL = "selectAll",
        MASS_BEGIN = "massBegin",
        /// A flag of whether the last mass edit ended.
        MASS_END = "massEnd",
        /// A flag of whether a mass edit is running.
        MASS_STATE = "massState",
        MASS_RESET = "massReset",
        /// The published state: a [`state`] row.
        PUBLISHED = "published",
        /// The current state, expected or published: a [`state`] row.
        SNAPSHOT = "snapshot",
        RESET = "reset",
        LAST_COMMIT = "lastCommit",
        LAST_DELETE = "lastDelete",
        /// A [`spaces`] row.
        SPACES = "spaces",
        /// A flag for each effect of whether it worked: a flag of whether the plan did.
        RESULTS = "results",
        /// Answers a [`read_query`]: the text before the cursor, after it, and selected.
        READ = "read",
        /// Answers a [`word_query`]: the word's start and end.
        WORD = "word",
        /// Answers a caps query: the caps mode flags.
        CAPS = "caps",
        /// Answers a [`text_query`]: how long its first character is.
        CHARACTER = "character",
        /// Answers a composer query: how much to remove and the composed text.
        COMPOSER = "composer",
        /// Answers a [`text_query`]: its last character and its last three.
        SPACING = "spacing",
        /// Answers a [`measure`] query: the length.
        MEASURE = "measure",
        /// Answers a [`measure`] query for a high-level operation: the length.
        EXTENT = "extent",
    }
}

crate::declare! {
    /// What a reply that is a plan, not a query, starts with.
    pub const REPLIES: object EditorReply(str) {
        READY = "ready",
    }
}

/// `EditorNative.attributes`.
pub mod attribute {
    crate::declare! {
        /// The operations `EditorNative.attributes` takes first, answered by
        /// `editor::attributes::call`.
        pub const OPS: enum EditorAttributeOp(i32) {
            /// A raw input type: an [`input_type`](super::input_type) row.
            INPUT = 0,
            /// Raw IME options: an [`ime_options`](super::ime_options) row.
            OPTIONS = 1,
            /// Caps mode flags: a caps mode.
            CAPS = 2,
            /// A caps mode: its flags.
            CAPS_FLAGS = 3,
            /// A raw input type and the selection's start and end: a flag of whether it is rich.
            RICH = 4,
            /// A [`keyboard_request`](super::keyboard_request) row: a [`keyboard`](super::keyboard) row.
            KEYBOARD = 5,
            /// A cursor, the text and a raw input type: caps mode flags.
            CAPS_AT = 6,
            /// A raw input type, raw IME options and a flag of whether shift is held: the action, or -1.
            ENTER_ACTION = 7,
        }
    }
}

/// A plan.
pub mod plan {
    crate::declare! {
        /// A plan: [`EditorReply`](super::REPLIES)`.READY` and a flag of whether it worked, then an
        /// [`effect`](super::effect) row for each effect, in order.
        pub const FIELDS: fields EditorPlan {
            STATUS,
            RESULT,
        }
    }
}

/// An effect on the editor.
pub mod effect {
    crate::declare! {
        /// What an effect does: an `InputConnection` call.
        pub const NAMES: object EditorEffect(str) {
            BEGIN = "begin",
            END = "end",
            FINISH = "finish",
            COMPOSING = "composing",
            COMPOSING_TEXT = "composingText",
            COMMIT = "commit",
            DELETE = "delete",
            SELECTION = "selection",
            ACTION = "action",
            CONTEXT_MENU = "contextMenu",
            DOWN = "down",
            UP = "up",
        }
    }

    crate::declare! {
        /// An effect: an [`EditorEffect`](NAMES), two numbers and a value, each as it needs.
        pub const FIELDS: fields EditorEffectRow {
            EFFECT,
            FIRST,
            SECOND,
            VALUE,
        }
    }
}

/// A query for the editor's text.
pub mod read_query {
    crate::declare! {
        /// A query for the editor's text, answered by `READ`: flags of whether to read the text
        /// before the cursor and the selected text. The text after it is always read.
        pub const FIELDS: fields EditorReadQuery {
            QUERY,
            BEFORE,
            SELECTED,
        }
    }
}

/// A query for the word at the cursor.
pub mod word_query {
    crate::declare! {
        /// A query for the word at the cursor, answered by `WORD`: the text before it and the
        /// last commit's position within that text.
        pub const FIELDS: fields EditorWordQuery {
            QUERY,
            BEFORE,
            LAST_COMMIT,
        }
    }
}

/// A query about some text.
pub mod text_query {
    crate::declare! {
        /// A query about some text, answered by `CHARACTER` or `SPACING`.
        pub const FIELDS: fields EditorTextQuery {
            QUERY,
            TEXT,
        }
    }
}

/// A query to measure some text.
pub mod measure {
    crate::declare! {
        /// A query to measure `COUNT` characters or words of some text, answered by `MEASURE` or
        /// `EXTENT`.
        pub const FIELDS: fields EditorMeasure {
            QUERY,
            TEXT,
            /// A flag: from its start, not its end.
            FORWARD,
            /// A flag: words, not characters.
            WORDS,
            COUNT,
        }
    }
}

/// The editor's configuration.
pub mod config {
    crate::declare! {
        /// The editor's configuration, a flag each but the raw input type.
        pub const FIELDS: fields EditorConfig {
            RAW,
            COMPOSING,
            DETERMINE_COMPOSING,
            SPACE_PENDING,
            INPUT_TYPE,
        }
    }
}

/// What high-level operations need.
pub mod context {
    crate::declare! {
        /// What high-level operations need: the space states, the key variation, a flag of
        /// whether punctuation spaces itself, the language, the punctuation rule's symbols and a
        /// flag of whether backspace deletes a glided word at once.
        pub const FIELDS: fields EditorContext {
            AUTO_SPACE,
            PHANTOM_SPACE,
            KEY_VARIATION,
            AUTO_SPACE_PUNCTUATION,
            LANGUAGE,
            PRECEDING_AUTO_SPACE,
            FOLLOWING_AUTO_SPACE,
            PRECEDING_PHANTOM_SPACE,
            FOLLOWING_PHANTOM_SPACE,
            IMMEDIATE_WORD,
        }
    }
}

/// The space states after a high-level operation.
pub mod spaces {
    crate::declare! {
        /// What to do with the phantom space's candidate.
        pub const CANDIDATES: object EditorCandidate(i32) {
            /// Keep the candidate; take the new bits.
            KEEP = 0,
            RESET = 1,
            ACCEPT = 2,
        }
    }

    crate::declare! {
        /// The space states after a high-level operation.
        pub const FIELDS: fields EditorSpaces {
            AUTO_SPACE,
            PHANTOM_SPACE,
            /// An [`EditorCandidate`](CANDIDATES).
            CANDIDATE,
        }
    }
}

/// Where the cursor is.
pub mod cursor {
    crate::declare! {
        /// Where the selection and composing region are.
        pub const FIELDS: fields EditorCursor {
            SELECTION_START,
            SELECTION_END,
            COMPOSING_START,
            COMPOSING_END,
        }
    }
}

/// Extracted text.
pub mod extracted {
    crate::declare! {
        /// Extracted text, as `ExtractedText` has it.
        pub const FIELDS: fields EditorExtracted {
            SELECTION_START,
            SELECTION_END,
            START_OFFSET,
            PARTIAL_START,
            PARTIAL_END,
            TEXT,
            COMPOSING_START,
            COMPOSING_END,
        }
    }
}

/// A character to commit.
pub mod char_input {
    crate::declare! {
        /// A character to commit, and flags of how to space it.
        pub const FIELDS: fields EditorChar {
            TEXT,
            DELETE_SPACE_BEFORE,
            SPACE_BEFORE,
            SPACE_AFTER,
        }
    }
}

/// The editor's state.
pub mod state {
    crate::declare! {
        /// The editor's state.
        pub const FIELDS: fields EditorState {
            TEXT,
            OFFSET,
            SELECTION_START,
            SELECTION_END,
            COMPOSING_START,
            COMPOSING_END,
            WORD_START,
            WORD_END,
            CAPS,
            LAST_COMMIT,
            /// A flag: an expected state is waiting to be reported.
            EXPECTED,
            FAILURES,
        }
    }
}

/// A decoded input type.
pub mod input_type {
    crate::declare! {
        /// A decoded input type: its class, variation and caps mode, then 1 or 0 for each flag.
        pub const FIELDS: fields EditorInputType {
            TYPE,
            VARIATION,
            CAPS_MODE,
            NUMBER_DECIMAL,
            NUMBER_SIGNED,
            TEXT_AUTO_COMPLETE,
            TEXT_AUTO_CORRECT,
            TEXT_IME_MULTI_LINE,
            TEXT_MULTI_LINE,
            TEXT_NO_SUGGESTIONS,
        }
    }
}

/// Decoded IME options.
pub mod ime_options {
    crate::declare! {
        /// Decoded IME options: the action, then 1 or 0 for each flag.
        pub const FIELDS: fields EditorImeOptions {
            ACTION,
            FORCE_ASCII,
            NAVIGATE_NEXT,
            NAVIGATE_PREVIOUS,
            NO_ACCESSORY_ACTION,
            NO_ENTER_ACTION,
            NO_EXTRACT_UI,
            NO_FULLSCREEN,
            NO_PERSONALIZED_LEARNING,
        }
    }
}

/// What a keyboard needs to know about a field.
pub mod keyboard_request {
    crate::declare! {
        /// A field's raw input type and IME options, a flag of whether suggestions are on, the
        /// incognito mode and a flag of whether a field may force it.
        pub const FIELDS: fields EditorKeyboardRequest {
            INPUT_TYPE,
            IME_OPTIONS,
            SUGGESTIONS,
            INCOGNITO_MODE,
            FORCE_INCOGNITO,
        }
    }
}

/// How the keyboard shows for a field.
pub mod keyboard {
    crate::declare! {
        /// How the keyboard shows for a field: its mode and key variation, then 1 or 0 for whether
        /// it composes and is incognito.
        pub const FIELDS: fields EditorKeyboard {
            MODE,
            KEY_VARIATION,
            COMPOSING,
            INCOGNITO,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &OPS,
    &REPLIES,
    &attribute::OPS,
    &plan::FIELDS,
    &effect::NAMES,
    &effect::FIELDS,
    &read_query::FIELDS,
    &word_query::FIELDS,
    &text_query::FIELDS,
    &measure::FIELDS,
    &config::FIELDS,
    &context::FIELDS,
    &spaces::CANDIDATES,
    &spaces::FIELDS,
    &cursor::FIELDS,
    &extracted::FIELDS,
    &char_input::FIELDS,
    &state::FIELDS,
    &input_type::FIELDS,
    &ime_options::FIELDS,
    &keyboard_request::FIELDS,
    &keyboard::FIELDS,
];
