//! `UtilityNative.call`: its operations, answered by `utilities::call`, and the packets and names
//! they pass. lib/kotlin makes this call, so its Kotlin goes there.

crate::declare! {
    /// The operations `UtilityNative.call` takes, answered by `utilities::call`.
    pub const OPS: enum UtilityOp(i32) {
        /// A MIME filter's patterns: a [`mime_filter`] row.
        MIME_FILTER = 0,
        /// The MIME types worth matching: a [`mime_input`] row for each.
        MIME_INPUTS = 1,
        /// A reference string as a [`reference::FIELDS`] row.
        NORMALIZE_REFERENCE = 2,
        /// A web address, with `https://` if it had no scheme.
        URL = 3,
        /// A reference's scheme, authority and path: a [`reference_facts`] row.
        REFERENCE_FACTS = 4,
        /// A locale tag as a [`locale`] row.
        PARSE_LOCALE = 5,
        /// A [`locale_tag`] row as a tag.
        LOCALE_TAG = 6,
        /// A [`locale_display`] row as a display name.
        LOCALE_DISPLAY_NAME = 7,
        /// Whether a language writes capitals: a flag.
        CAPITALIZES = 8,
        /// A version string as a [`version_plan`] row, or nothing.
        PARSE_VERSION = 9,
        /// Two [`version`] rows, one after the other: -1, 0 or 1.
        COMPARE_VERSIONS = 10,
        /// A [`version`] row, its extra name after a flag of whether it has one, as a string.
        FORMAT_VERSION = 11,
        /// The install, last changelog and current versions: a flag of whether to show the
        /// changelog.
        SHOWS_CHANGELOG = 12,
        /// Whether a version is the unset default: a flag.
        UNSET_VERSION = 13,
        /// A [`validation`] row: a [`UtilityValidity`](VALIDITY), or empty if valid.
        VALIDATE = 14,
        /// A [`mime_request`] row, then a [`mime_facts`] row for each input: a flag.
        MIME_MATCH = 15,
    }
}

crate::declare! {
    /// How many of the MIME types must match.
    pub const MIME_MODES: object UtilityMimeMode(i32) {
        /// The single type given.
        SINGLE = 0,
        ALL = 1,
        ANY = 2,
        /// Exactly one.
        ONE = 3,
    }
}

crate::declare! {
    /// What a reference points at.
    pub const REFERENCE_KINDS: object UtilityReferenceKind(i32) {
        OTHER = 0,
        APP_UI = 1,
        ASSETS = 2,
        CACHE = 3,
        INTERNAL = 4,
    }
}

crate::declare! {
    /// The user dictionary fields [`VALIDATE`] can check.
    pub const RULES: enum UtilityRule(str) {
        WORD = "word",
        FREQUENCY = "freq",
        SHORTCUT = "shortcut",
    }
}

crate::declare! {
    /// Why a user dictionary field isn't valid.
    pub const VALIDITY: object UtilityValidity(str) {
        EMPTY = "empty",
        INVALID = "invalid",
    }
}

crate::declare! {
    /// A [`version_plan`] row's status.
    pub const VERSION_STATUS: object UtilityVersionStatus(str) {
        OK = "ok",
        /// A part is not a number; the row's `BAD_NUMBER` holds it.
        NUMBER_ERROR = "number-error",
    }
}

/// A MIME filter's patterns.
pub mod mime_filter {
    crate::declare! {
        /// A MIME filter: an error (empty if none), then a regex for its type and its subtype.
        pub const FIELDS: fields UtilityMimeFilter {
            ERROR,
            TYPE,
            SUBTYPE,
        }
    }
}

/// A MIME type worth matching.
pub mod mime_input {
    crate::declare! {
        /// A MIME type worth matching: its place among the inputs, its type and its subtype.
        pub const FIELDS: fields UtilityMimeInput {
            INDEX,
            TYPE,
            SUBTYPE,
        }
    }
}

/// The head of a MIME match request.
pub mod mime_request {
    crate::declare! {
        /// A MIME match: a [`UtilityMimeMode`](super::MIME_MODES), how many inputs and how many
        /// filters there are.
        pub const FIELDS: fields UtilityMimeRequest {
            MODE,
            COUNT,
            FILTERS,
        }
    }
}

/// The head of an input's MIME facts.
pub mod mime_facts {
    crate::declare! {
        /// An input's MIME facts: its place among the inputs, then a
        /// [`mime_match`](super::mime_match) row for each filter.
        pub const FIELDS: fields UtilityMimeFacts {
            INDEX,
        }
    }
}

/// Whether a filter matches an input.
pub mod mime_match {
    crate::declare! {
        /// Whether a filter's type and subtype match an input's: a flag each.
        pub const FIELDS: fields UtilityMimeMatch {
            TYPE,
            SUBTYPE,
        }
    }
}

/// A reference.
pub mod reference {
    crate::declare! {
        /// A reference: its authority (empty unless a legacy one), then its path or whole string.
        pub const FIELDS: fields UtilityReference {
            AUTHORITY,
            PATH,
        }
    }
}

/// What a reference points at.
pub mod reference_facts {
    crate::declare! {
        /// What a reference points at.
        pub const FIELDS: fields UtilityReferenceFacts {
            /// A [`UtilityReferenceKind`](super::REFERENCE_KINDS).
            KIND,
            /// A flag: not one of FlorisBoard's own.
            EXTERNAL,
            /// A flag: it has a scheme and an authority.
            HAS_AUTHORITY,
            /// Its path without the leading `/`.
            RELATIVE_PATH,
        }
    }
}

/// A locale's parts.
pub mod locale {
    crate::declare! {
        /// A locale's parts.
        pub const FIELDS: fields UtilityLocale {
            LANGUAGE,
            COUNTRY,
            VARIANT,
        }
    }
}

/// A locale to write as a tag.
pub mod locale_tag {
    crate::declare! {
        /// A locale to write as a tag, and the UTF-16 code of the character between its parts.
        pub const FIELDS: fields UtilityLocaleTag {
            DELIMITER,
            LANGUAGE,
            COUNTRY,
            VARIANT,
        }
    }
}

/// A locale to name.
pub mod locale_display {
    crate::declare! {
        /// A locale to name: its parts' display names, then its parts.
        pub const FIELDS: fields UtilityLocaleDisplay {
            LANGUAGE,
            COUNTRY,
            VARIANT,
            RAW_LANGUAGE,
            RAW_COUNTRY,
            RAW_VARIANT,
        }
    }
}

/// A version.
pub mod version {
    crate::declare! {
        /// A version's parts. A missing extra is empty.
        pub const FIELDS: fields UtilityVersion {
            MAJOR,
            MINOR,
            PATCH,
            EXTRA_NAME,
            EXTRA_VALUE,
        }
    }
}

/// A parsed version.
pub mod version_plan {
    crate::declare! {
        /// A parsed version: a [`UtilityVersionStatus`](super::VERSION_STATUS), then its parts.
        pub const FIELDS: fields UtilityVersionPlan {
            STATUS,
            MAJOR,
            MINOR,
            PATCH,
            EXTRA_NAME,
            EXTRA_VALUE,
            /// The part that is not a number, if the status says so.
            BAD_NUMBER,
        }
    }
}

/// A user dictionary field to check.
pub mod validation {
    crate::declare! {
        /// A user dictionary field to check: its [`UtilityRule`](super::RULES), its value, and the
        /// lowest and highest frequency.
        pub const FIELDS: fields UtilityValidation {
            RULE,
            VALUE,
            MIN,
            MAX,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &OPS,
    &MIME_MODES,
    &REFERENCE_KINDS,
    &RULES,
    &VALIDITY,
    &VERSION_STATUS,
    &mime_filter::FIELDS,
    &mime_input::FIELDS,
    &mime_request::FIELDS,
    &mime_facts::FIELDS,
    &mime_match::FIELDS,
    &reference::FIELDS,
    &reference_facts::FIELDS,
    &locale::FIELDS,
    &locale_tag::FIELDS,
    &locale_display::FIELDS,
    &version::FIELDS,
    &version_plan::FIELDS,
    &validation::FIELDS,
];
