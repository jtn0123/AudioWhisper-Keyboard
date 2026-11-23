//! `ImeSubtypesNative`: the keyboard's subtypes, which Rust holds and answers from the stored list
//! Android reads from the settings, answered by the `imesubtype` crate.
//!
//! Rust decodes and repairs the stored list, keeps it and the active subtype, switches between
//! them, and says what each subtype names. Android writes the settings Rust asks it to, names a
//! locale tag as `java.util.Locale` does, and reports which currency sets it loaded.

crate::declare! {
    /// The operations `ImeSubtypesNative.call` takes.
    pub const OPS: enum ImeSubtypeOp(str) {
        /// A [`load`] row: a [`loaded`] row, then either the locale tags Android must name first
        /// (`ASKS` of them, after which it answers with `LOCALES` and loads again), or a
        /// [`subtype`] row for each of the `COUNT` subtypes in the list.
        LOAD = "load",
        /// Each tag the last load asked about, as `FlorisLocale.fromTag(tag).languageTag()` names
        /// it, in order. Answered with nothing.
        LOCALES = "locales",
        /// An [`evaluate`] row: which subtype is active once the list has changed, as a
        /// [`switched`] row and the active subtype's [`subtype`] row.
        EVALUATE = "evaluate",
        /// Nothing: the language key's next subtype, as a [`switched`] row and a [`subtype`] row.
        NEXT = "next",
        /// Nothing: the language key's previous subtype, answered as `NEXT` is.
        PREVIOUS = "previous",
        /// Nothing: the active subtype, answered as `NEXT` is but never `WRITE`.
        ACTIVE = "active",
        /// A [`select`] row: the subtype with that id, answered as `NEXT` is, or a [`switched`]
        /// row alone, not `WRITE`, when the list has no such subtype.
        SELECT = "select",
        /// The names of the currency sets Android loaded, as it reports them. Answered with
        /// nothing.
        SETS = "sets",
        /// A currency set's name, as a subtype names it: its index among the sets last reported,
        /// or -1 for none (Android then uses its fallback set).
        CURRENCY = "currency",
    }
}

/// What a load reads.
pub mod load {
    crate::declare! {
        /// The stored list.
        pub const FIELDS: fields ImeSubtypeLoad {
            /// The `localization__subtypes` setting's text, which may be blank.
            RAW,
        }
    }
}

/// What came of a load.
pub mod loaded {
    crate::declare! {
        /// What came of a load.
        pub const FIELDS: fields ImeSubtypeLoaded {
            /// How many locale tags Android must name before the list can load; when not 0,
            /// nothing else in the row is set.
            ASKS,
            /// A flag: the list was repaired, so Android writes `RAW` back to the setting.
            CHANGED,
            RAW,
            /// How many subtypes the list holds.
            COUNT,
        }
    }
}

/// Which subtype becomes active after a load.
pub mod evaluate {
    crate::declare! {
        /// The active subtype id the settings store.
        pub const FIELDS: fields ImeSubtypeEvaluate {
            STORED,
        }
    }
}

/// A switch by id.
pub mod select {
    crate::declare! {
        /// The id of the subtype to switch to.
        pub const FIELDS: fields ImeSubtypeSelect {
            ID,
        }
    }
}

/// What came of a switch.
pub mod switched {
    crate::declare! {
        /// What came of a switch: the active subtype's id and whether to store it.
        pub const FIELDS: fields ImeSubtypeSwitched {
            /// A flag: Android stores `ID` as the active subtype id.
            WRITE,
            ID,
            /// A flag: a [`super::subtype`] row follows, the subtype now active.
            ACTIVE,
        }
    }
}

/// One subtype, as the keyboard reads it.
pub mod subtype {
    crate::declare! {
        /// One subtype: what it names, then `SECONDARY` secondary locale tags.
        pub const FIELDS: fields ImeSubtypeRow {
            ID,
            /// The key graph cache key: one number for each distinct subtype.
            KEY,
            /// The primary locale's tag, as Android named it.
            LOCALE,
            /// The NLP providers' names.
            SPELLING,
            SUGGESTION,
            /// The composer, currency set, punctuation rule and popup mapping, each as an
            /// extension component name.
            COMPOSER,
            CURRENCY,
            PUNCTUATION,
            POPUP,
            /// The layout for each `LayoutType`, in its order, empty for a type no subtype names.
            CHARACTERS,
            CHARACTERS_MOD,
            EXTENSION,
            NUMERIC,
            NUMERIC_ADVANCED,
            NUMERIC_ROW,
            PHONE,
            PHONE2,
            SYMBOLS,
            SYMBOLS_MOD,
            SYMBOLS2,
            SYMBOLS2_MOD,
            /// How many secondary locale tags follow the row.
            SECONDARY,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &OPS,
    &load::FIELDS,
    &loaded::FIELDS,
    &evaluate::FIELDS,
    &select::FIELDS,
    &switched::FIELDS,
    &subtype::FIELDS,
];
