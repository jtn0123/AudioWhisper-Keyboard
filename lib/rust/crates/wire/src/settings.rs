//! `SettingsNative.call`: its operations, answered by `settings::call`, and the packets they pass.

crate::declare! {
    /// The operations `SettingsNative.call` takes, answered by `settings::call`.
    pub const OPS: enum SettingsOp(i32) {
        /// Every preference, a [`catalogue`] row each.
        CATALOGUE = 0,
        /// A choice's place among a preference's choices.
        ENUM_INDEX = 1,
        /// A number kept within a preference's range.
        CLAMP = 2,
        /// A stored scalar admitted for its key, or nothing.
        SCALAR = 3,
        /// A stored kind's JSON as a value packet.
        DECODE = 10,
        /// A value packet as a stored kind's JSON.
        ENCODE = 11,
        /// What becomes of an old preference: a [`migration`] row.
        MIGRATE = 12,
        /// The default subtype, as a value packet.
        DEFAULT_SUBTYPE = 20,
        /// A subtype edit's plan, as a value packet.
        SUBTYPE = 21,
        /// The preset for a locale.
        PRESET = 22,
        /// Whether the subtype editor's fields may be saved: a flag.
        EDITOR_VALID = 23,
        /// A colour as sixteen hex digits.
        COLOR = 24,
        /// A subtype's place among the ids, or -1.
        SUBTYPE_INDEX = 25,
        /// A stored time as a [`time`] row, or nothing.
        TIME_DECODE = 26,
        /// A [`time`] row as a stored time.
        TIME_ENCODE = 27,
        /// A [`layouts`] row as a subtype's `name=value` string.
        LAYOUT_MAP = 28,
        /// A [`key_hints`] row: a [`key_hint_modes`] row, each mode or DISABLED.
        KEY_HINTS = 29,
        /// A value's place among the choices, or 0.
        CHOICE_INDEX = 30,
        /// An extension component as its stored string.
        COMPONENT_ENCODE = 31,
        /// A stored extension component as a [`component`] row, or nothing.
        COMPONENT_DECODE = 32,
    }
}

crate::declare! {
    /// How a settings value packet tags each value.
    pub const VALUE_TAGS: object SettingsValueTag(str) {
        NULL = "z",
        TEXT = "s",
        NUMBER = "n",
        ARRAY = "a",
        OBJECT = "o",
    }
}

crate::declare! {
    /// What a migration does with an old preference.
    pub const MIGRATION_ACTIONS: object SettingsMigrationAction(str) {
        RESET = "reset",
        KEEP = "keep",
        TRANSFORM = "transform",
    }
}

/// One preference in the catalogue.
pub mod catalogue {
    crate::declare! {
        /// One preference in the settings catalogue.
        pub const FIELDS: fields SettingsCatalogue {
            KEY,
            KIND,
            DEFAULT,
            MIN,
            MAX,
            STEP,
            CHOICES,
        }
    }
}

/// A stored time's parts.
pub mod time {
    crate::declare! {
        /// A stored time's parts.
        pub const FIELDS: fields SettingsTime {
            HOUR,
            MINUTE,
            SECOND,
            MILLISECOND,
        }
    }
}

/// An extension component's parts.
pub mod component {
    crate::declare! {
        /// An extension component's parts.
        pub const FIELDS: fields SettingsComponent {
            EXTENSION,
            COMPONENT,
        }
    }
}

/// What a migration does with an old preference.
pub mod migration {
    crate::declare! {
        /// What a migration does with an old preference, and its type, key and value after.
        pub const FIELDS: fields SettingsMigration {
            /// A [`MIGRATION_ACTIONS`](super::MIGRATION_ACTIONS) name.
            ACTION,
            TYPE,
            KEY,
            VALUE,
        }
    }
}

/// The key hints' settings.
pub mod key_hints {
    crate::declare! {
        /// The number row's and symbols' hint settings: each enabled flag and mode.
        pub const FIELDS: fields SettingsKeyHints {
            NUMBER_ENABLED,
            NUMBER,
            SYMBOLS_ENABLED,
            SYMBOLS,
        }
    }
}

/// The key hints' modes.
pub mod key_hint_modes {
    crate::declare! {
        /// The number row's and symbols' hint modes.
        pub const FIELDS: fields SettingsKeyHintModes {
            NUMBER,
            SYMBOLS,
        }
    }
}

/// A subtype's layouts.
pub mod layouts {
    crate::declare! {
        /// A subtype's layout for each keyboard mode, in `settings::codec::LAYOUTS` order.
        pub const FIELDS: fields SettingsLayouts {
            CHARACTERS,
            SYMBOLS,
            SYMBOLS2,
            NUMERIC,
            NUMERIC_ADVANCED,
            NUMERIC_ROW,
            PHONE,
            PHONE2,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &OPS,
    &VALUE_TAGS,
    &MIGRATION_ACTIONS,
    &catalogue::FIELDS,
    &time::FIELDS,
    &component::FIELDS,
    &migration::FIELDS,
    &key_hints::FIELDS,
    &key_hint_modes::FIELDS,
    &layouts::FIELDS,
];
