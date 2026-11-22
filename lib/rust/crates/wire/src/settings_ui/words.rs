//! What the user dictionary screens ask Kotlin for. Rust draws them, checks a word, reads an
//! import and writes an export's text; Kotlin reads the two dictionaries and changes them, as
//! the Floris dictionary's Room database and the system's content provider need, and names
//! locales as Java does.

/// The effects the user dictionary screens return.
pub mod effect {
    crate::declare! {
        /// The effects the user dictionary screens return: each packet's first field, then a
        /// [`SettingsUiDictionary`](super::dictionary::KINDS). Kotlin answers every one with a
        /// fresh `WORDS` report.
        pub const KINDS: object SettingsUiWordEffect(str) {
            /// Read the dictionary.
            LOAD = "words",
            /// Insert the [`SettingsUiWord`](super::row::FIELDS) row that follows, its locale
            /// normalised by `FlorisLocale`, as the add dialog inserted.
            ADD = "wordAdd",
            /// Update the row that follows, by its `ID`, its locale normalised, as the edit dialog
            /// updated.
            CHANGE = "wordChange",
            /// Delete the row that follows, by its `ID`.
            DELETE = "wordDelete",
            /// Insert the row that follows as it was stored: a delete undone.
            RESTORE = "wordRestore",
            /// Upsert each row that follows, in order, as `importCombinedList` did: the word
            /// with its locale updated when it is there, inserted when not.
            IMPORT = "wordImport",
            /// Open the system's own user dictionary settings.
            MANAGER = "wordManager",
        }
    }
}

/// Which user dictionary.
pub mod dictionary {
    crate::declare! {
        /// Which user dictionary: the names `UserDictionaryType` has in its route.
        pub const KINDS: object SettingsUiDictionary(str) {
            /// The keyboard's own, in Room.
            FLORIS = "FLORIS",
            /// Android's, through `UserDictionary.Words`.
            SYSTEM = "SYSTEM",
        }
    }
}

/// What a report row is.
pub mod kind {
    crate::declare! {
        /// What a [`SettingsUiWord`](super::row::FIELDS) report row is.
        pub const KINDS: object SettingsUiWordKind(str) {
            /// The report's first row: `ID` the dictionary, `WORD` a flag, whether it is on, and
            /// `LABEL` a [`SettingsUiWordOutcome`](super::outcome::KINDS).
            DICTIONARY = "dictionary",
            /// A stored word, in `queryAll()` order: what an export lists.
            WORD = "word",
            /// Each distinct locale the words are stored under, once: `LOCALE` as stored,
            /// `LABEL` its `displayName()` and `LANGUAGE` its `displayLanguage()`, which
            /// `queryLanguageList()` sorted by.
            LOCALE = "locale",
        }
    }
}

/// What the effect a report answers did.
pub mod outcome {
    crate::declare! {
        /// What the effect a report answers did, when it is worth telling.
        pub const KINDS: object SettingsUiWordOutcome(str) {
            /// Nothing to tell.
            NONE = "none",
            /// An import's rows were all written.
            IMPORTED = "imported",
            /// The database refused a change, or the dictionary is off.
            FAILED = "failed",
        }
    }
}

/// A word, reported or acted on.
pub mod row {
    crate::declare! {
        /// A word, reported or acted on. A null locale or shortcut is empty with its flag false.
        pub const FIELDS: fields SettingsUiWord {
            /// A [`SettingsUiWordKind`](super::kind::KINDS); empty in an effect.
            KIND,
            /// The row id.
            ID,
            WORD,
            /// The frequency, in decimal.
            FREQ,
            LOCALE,
            /// A flag: whether the locale is not null.
            HAS_LOCALE,
            SHORTCUT,
            /// A flag: whether the shortcut is not null.
            HAS_SHORTCUT,
            LABEL,
            LANGUAGE,
        }
    }
}
