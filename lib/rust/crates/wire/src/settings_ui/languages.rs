//! What the language screens (Languages & Layouts, Select language, the language pack manager
//! and the subtype editor) ask Kotlin for. Rust draws them and edits the subtype list itself;
//! Kotlin only reads the keyboard's resources, names locales as Java does, and deletes a
//! language pack, which needs the extension manager.

/// The effects the language screens return.
pub mod effect {
    crate::declare! {
        /// The effects the language screens return: each packet's first field.
        pub const KINDS: object SettingsUiLanguageEffect(str) {
            /// Read the keyboard's language resources and answer with a `LANGUAGES` report.
            LOAD = "languages",
            /// Delete the language pack extension whose id is the next field. Kotlin answers with
            /// a fresh `LANGUAGES` report, or a toast when the delete failed.
            DELETE_PACK = "deleteLanguagePack",
        }
    }
}

/// What a report row is.
pub mod kind {
    crate::declare! {
        /// What a [`SettingsUiLanguage`](super::row::FIELDS) row is.
        pub const KINDS: object SettingsUiLanguageKind(str) {
            /// The subtype list normalised as Kotlin reads it: `ID` the raw preference it read,
            /// `DETAIL` the list encoded again after each locale went through `FlorisLocale`.
            SUBTYPES = "subtypes",
            /// A locale the picker offers: `ID` its language tag, `LABEL` its name in the app's
            /// locale, `NATIVE` in itself, `ENGLISH` in English, `DETAIL` its locale tag, `BASE`
            /// its Java locale's `toString()` and `LANGUAGE` its language code.
            LOCALE = "locale",
            /// A locale the device is set to, in order: `ID` its language tag, with its names.
            SYSTEM = "system",
            /// The names of a locale a subtype or preset uses: `ID` the tag as stored.
            NAME = "name",
            /// A layout: `DETAIL` its layout type's id (`characters`, `symbols`, ...), `ID` its
            /// component name, `LABEL` its label.
            LAYOUT = "layout",
            /// A composer: `ID` its component name, `LABEL` its label.
            COMPOSER = "composer",
            /// A currency set: `ID` its component name, `LABEL` its label.
            CURRENCY = "currency",
            /// A popup mapping: `ID` its component name, `LABEL` its label.
            POPUP = "popup",
            /// A subtype preset, in the order the keyboard lists them: `ID` its locale's language
            /// tag, `BASE` and `LANGUAGE` as a locale's, `DETAIL` the preset as a subtype, encoded
            /// as the subtype list stores one.
            PRESET = "preset",
            /// A language pack extension: `ID` its id, `LABEL` its title, `DETAIL` "true" when
            /// the extension manager may delete it.
            PACK = "pack",
            /// One of a language pack's items: `ID` its extension's id, `LABEL` its label.
            PACK_ITEM = "packItem",
        }
    }
}

/// A `LANGUAGES` report row.
pub mod row {
    crate::declare! {
        /// A `LANGUAGES` report row; a report is any number of them.
        pub const FIELDS: fields SettingsUiLanguage {
            /// A [`SettingsUiLanguageKind`](super::kind::KINDS).
            KIND,
            ID,
            LABEL,
            NATIVE,
            ENGLISH,
            DETAIL,
            BASE,
            LANGUAGE,
        }
    }
}
