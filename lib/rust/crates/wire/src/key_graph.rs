//! `KeyGraphNative`: one keyboard's key graph, which Rust builds from the layout files Android
//! reads and computes from the facts Android reports, answered by `keygraph::Graph`.
//!
//! A computation may ask questions only Android can answer (a letter's case in a locale, whether a
//! code point is a non-spacing mark); Android answers them and computes again.

crate::declare! {
    /// The operations `KeyGraphNative.call` takes.
    pub const OPS: enum KeyGraphOp(str) {
        /// A [`facts`] row, then a code and a label for each currency slot, then a resource token
        /// and its label for each resource: a [`question`] row for each question still open, or
        /// nothing once every listed key is computed.
        COMPUTE = "compute",
        /// One answer for each question the last computation asked, in order.
        ANSWER = "answer",
        /// A [`key`] row for each listed key.
        KEYS = "keys",
        /// A key's index, then a [`hints`] row: how many keys are prioritized, then a [`data`] row
        /// for each popup key, prioritized first.
        POPUPS = "popups",
        /// A [`hints`] row: a flag for each listed key of whether it has popup keys.
        POPUP_FLAGS = "popupFlags",
        /// A multi-text key's id: its code points.
        CODE_POINTS = "codePoints",
        /// An [`info`] row.
        INFO = "info",
        /// An [`action`] row for each of the smartbar's actions: an [`action_answer`] row for
        /// each, what the smartbar's evaluator said of the action's key.
        ACTIONS = "actions",
    }
}

/// What a keyboard is built from.
pub mod build {
    crate::declare! {
        /// What a keyboard is built from: the layout files Android read. A file that did not load
        /// is left out of `LOADED` and its field is empty.
        pub const FIELDS: fields KeyGraphBuild {
            /// The `KeyboardMode` value.
            MODE,
            /// A flag: the hinted number row is on.
            NUMBER_HINTS,
            /// The [`file`] bits of the files that loaded.
            LOADED,
            /// The main, modifier and extension layouts, and the modifier the main layout's index
            /// entry names.
            MAIN,
            MODIFIER,
            OWN_MODIFIER,
            EXTENSION,
            /// The symbols keyboard's, which a characters keyboard takes its hints from.
            SYMBOLS_MAIN,
            SYMBOLS_MODIFIER,
            SYMBOLS_OWN_MODIFIER,
            SYMBOLS_EXTENSION,
            /// The subtype's popup mapping, and the default one behind it.
            MAPPING,
            FALLBACK,
        }
    }
}

/// Which of a [`build`] row's files loaded.
pub mod file {
    crate::declare! {
        /// A [`super::build`] file's bit in `LOADED`.
        pub const BITS: object KeyGraphFile(i32) {
            MAIN = 1,
            MODIFIER = 2,
            /// The main layout's index entry names a modifier, the file `OWN_MODIFIER` holds if
            /// it loaded.
            NAMES_MODIFIER = 4,
            OWN_MODIFIER = 8,
            EXTENSION = 16,
            SYMBOLS_MAIN = 32,
            SYMBOLS_MODIFIER = 64,
            SYMBOLS_NAMES_MODIFIER = 128,
            SYMBOLS_OWN_MODIFIER = 256,
            SYMBOLS_EXTENSION = 512,
            MAPPING = 1024,
            FALLBACK = 2048,
        }
    }
}

/// The facts of one computation.
pub mod facts {
    crate::declare! {
        /// What Android's key evaluator reads, for one computation.
        pub const FIELDS: fields KeyGraphFacts {
            /// The keyboard state's raw value, with the evaluator's shift reset applied.
            STATE,
            /// The editor's raw input type and IME options.
            INPUT,
            OPTIONS,
            /// A flag: the utility key is on; its action's ordinal; a flag: the language switch
            /// shows.
            UTILITY,
            UTILITY_ACTION,
            LANGUAGE_SWITCH,
            /// Flags: a rich editor, a locked device or keyguard, a pastable primary clip.
            RICH_EDITOR,
            LOCKED,
            PASTABLE,
            /// The incognito setting's ordinal; a flag: the editor asks for no learning.
            INCOGNITO,
            NO_LEARNING,
            /// How many subtypes there are; a flag: dictation may type into the field.
            SUBTYPES,
            DICTATION,
            /// The subtype's language name, which the space key shows, and `Subtype.DEFAULT`'s,
            /// which the smartbar's evaluator names.
            LANGUAGE,
            DEFAULT_LANGUAGE,
            /// The [`hints`](super::hints) fields, as `HINT_SYMBOL`, `HINT_NUMBER` and `HINT_MERGE`.
            HINT_SYMBOL,
            HINT_NUMBER,
            HINT_MERGE,
            /// How many currency slots, then resources, follow the row.
            SLOTS,
            RESOURCES,
        }
    }
}

/// The hint settings popup keys are ordered by.
pub mod hints {
    crate::declare! {
        /// The hint settings popup keys are ordered by, `KeyHintConfiguration`.
        pub const FIELDS: fields KeyGraphHints {
            /// The `KeyHintMode` ordinals for symbols and numbers.
            SYMBOL,
            NUMBER,
            /// A flag: hint popups merge.
            MERGE,
        }
    }
}

/// A question for Android.
pub mod question {
    crate::declare! {
        /// What [`ASKS`] gives for `VALUE` in the [`LOCALES`].
        pub const FIELDS: fields KeyGraphQuestion {
            ASK,
            LOCALE,
            /// A code point in decimal, or a text.
            VALUE,
        }
    }

    crate::declare! {
        /// What a question asks; a code's answer is a code point in decimal, a mark's a flag.
        pub const ASKS: object KeyGraphAsk(i32) {
            /// A code point's first code point, lowercased.
            LOWER_CODE = 0,
            /// A code point's first code point, uppercased.
            UPPER_CODE = 1,
            /// A text, lowercased.
            LOWER_TEXT = 2,
            /// A text, uppercased.
            UPPER_TEXT = 3,
            /// Whether a code point is a non-spacing mark, by ICU; the locale is ignored.
            MARK = 4,
        }
    }

    crate::declare! {
        /// Which locale a question is in.
        pub const LOCALES: object KeyGraphLocale(i32) {
            /// The subtype's primary locale.
            SUBTYPE = 0,
            /// `Subtype.DEFAULT`'s.
            DEFAULT = 1,
        }
    }
}

/// A computed key, as a `KeyData`.
pub mod data {
    crate::declare! {
        /// A computed key: a `TextKeyData`, or a `MultiTextKeyData` when `MULTI` names one.
        pub const FIELDS: fields KeyGraphData {
            /// The `KeyType` ordinal.
            TYPE,
            CODE,
            LABEL,
            GROUP,
            /// The popup set it came with, by its id in the graph, or -1.
            POPUP,
            /// The multi-text key it is, by its id in the graph, or -1.
            MULTI,
        }
    }
}

/// One listed key.
pub mod key {
    crate::declare! {
        /// One listed key as computed: flags, then its [`data`](super::data) row's fields, then what it shows.
        pub const FIELDS: fields KeyGraphKey {
            /// Flags: it shows, it is enabled.
            VISIBLE,
            ENABLED,
            TYPE,
            CODE,
            LABEL,
            GROUP,
            POPUP,
            MULTI,
            /// A flag: it shows a label, `SHOWN`.
            LABELED,
            SHOWN,
            /// A flag: it shows a hint, `HINT`.
            HINTED,
            HINT,
            /// The icon token Android draws, or 0 for none.
            ICON,
            /// The code of its layout entry before any state computes it (0 for a selector).
            BASE,
        }
    }
}

/// What a graph is.
pub mod info {
    crate::declare! {
        /// What a graph is.
        pub const FIELDS: fields KeyGraphInfo {
            /// The `KeyboardMode` value.
            MODE,
            /// How many rows it has, empty ones included.
            ROWS,
            /// How many keys it lists.
            LISTED,
        }
    }
}

/// A smartbar action's key, which the action panels ask about.
pub mod action {
    crate::declare! {
        /// A smartbar action's key, asked about with the last computation's facts.
        pub const FIELDS: fields KeyGraphAction {
            /// The `KeyType` ordinal.
            TYPE,
            CODE,
            LABEL,
        }
    }
}

/// What the graph says of a smartbar action's key.
pub mod action_answer {
    crate::declare! {
        /// What the graph says of a smartbar action's key, as the smartbar's evaluator did.
        pub const FIELDS: fields KeyGraphActionAnswer {
            /// A flag: it is enabled.
            ENABLED,
            /// The icon token Android draws, or 0 for none.
            ICON,
            /// A flag: it shows a label, `LABEL`.
            LABELED,
            LABEL,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &OPS,
    &build::FIELDS,
    &file::BITS,
    &facts::FIELDS,
    &hints::FIELDS,
    &question::FIELDS,
    &question::ASKS,
    &question::LOCALES,
    &data::FIELDS,
    &key::FIELDS,
    &info::FIELDS,
    &action::FIELDS,
    &action_answer::FIELDS,
];
