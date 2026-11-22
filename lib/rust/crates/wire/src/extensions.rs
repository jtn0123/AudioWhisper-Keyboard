//! `ExtensionPolicyNative.call`: its operations, answered by `extensions::call`, and the packets
//! and names they pass.

crate::declare! {
    /// The operations `ExtensionPolicyNative.call` takes, answered by `extensions::call`.
    pub const OPS: enum ExtensionOp(i32) {
        /// A field checked by an [`ExtensionRule`](RULES): a [`validation`] row.
        VALIDATE = 0,
        /// Where a maintainer's parts lie: a [`maintainer`] row, or nothing.
        MAINTAINER_RANGES = 1,
        /// A maintainer's name, email and URL as one string.
        FORMAT_MAINTAINER = 2,
        /// `ext:comp` split: a [`component`] row.
        COMPONENT_NAME = 3,
        /// The root, then an [`archive_entry`] row for each entry: an
        /// [`ExtensionArchiveAction`](ARCHIVE_ACTIONS) for each.
        ARCHIVE = 4,
        /// A manifest's raw and canonical JSON: a [`manifest`] row.
        MANIFEST = 5,
        /// A serial type's group: a [`group`] row.
        GROUP = 6,
        /// Whether each archive entry's name is accepted: a flag each.
        ARCHIVE_NAMES = 9,
        /// A theme component's id, label, stylesheet and authors: a [`theme`] row.
        THEME_COMPONENT = 10,
        /// A theme component's stylesheet path.
        STYLESHEET = 11,
        /// A composer's [`ExtensionComposerType`](COMPOSER_TYPES).
        COMPOSER_TYPE = 14,
        /// Whether an extension's id, version, title, maintainers and license are valid: a flag.
        METADATA_VALID = 15,
    }
}

crate::declare! {
    /// The fields [`VALIDATE`] can check.
    pub const RULES: enum ExtensionRule(str) {
        ID = "id",
        VERSION = "version",
        TITLE = "title",
        MAINTAINERS = "maintainers",
        LICENSE = "license",
        COMPONENT_ID = "componentId",
        COMPONENT_LABEL = "componentLabel",
        COMPONENT_AUTHORS = "componentAuthors",
        STYLESHEET_PATH = "stylesheet",
    }
}

crate::declare! {
    /// A [`validation`] row's status.
    pub const VALIDITY: object ExtensionValidity(str) {
        VALID = "valid",
        INVALID = "invalid",
    }
}

crate::declare! {
    /// What to do with an archive entry.
    pub const ARCHIVE_ACTIONS: object ExtensionArchiveAction(str) {
        SKIP = "skip",
        DIRECTORY = "directory",
        COPY = "copy",
        EMPTY = "empty",
    }
}

crate::declare! {
    /// How a keyboard extension composes, by its serial name.
    pub const COMPOSER_TYPES: object ExtensionComposerType(str) {
        APPENDER = "appender",
        WITH_RULES = "with-rules",
    }
}

/// A checked field.
pub mod validation {
    crate::declare! {
        /// A checked field: its status, the resource key of its message (empty if none), and the
        /// name and pattern of a regex the message shows (empty if none).
        pub const FIELDS: fields ExtensionValidationPlan {
            /// An [`ExtensionValidity`](super::VALIDITY).
            STATUS,
            RESOURCE,
            ARGUMENT,
            PATTERN,
        }
    }
}

/// Where a maintainer's parts lie.
pub mod maintainer {
    crate::declare! {
        /// Where a maintainer's name, email and URL start and end. A missing part's start is -1.
        pub const FIELDS: fields ExtensionMaintainerRanges {
            NAME,
            NAME_END,
            EMAIL,
            EMAIL_END,
            URL,
            URL_END,
        }
    }
}

/// An extension component's name.
pub mod component {
    crate::declare! {
        /// An extension component's name: an error (empty if none), then its parts.
        pub const FIELDS: fields ExtensionComponentPlan {
            ERROR,
            EXTENSION,
            COMPONENT,
        }
    }
}

/// An archive entry.
pub mod archive_entry {
    crate::declare! {
        /// An archive entry as the platform reads it.
        pub const FIELDS: fields ExtensionArchiveEntry {
            NAME,
            CANONICAL_PATH,
            /// Its size in bytes.
            BYTES,
            /// A flag.
            DIRECTORY,
        }
    }
}

/// A manifest's kind.
pub mod manifest {
    crate::declare! {
        /// A manifest's kind: its serial type ("unknown" if none), group and component field.
        pub const FIELDS: fields ExtensionManifest {
            SERIAL,
            GROUP,
            COMPONENT_FIELD,
        }
    }
}

/// A serial type's group.
pub mod group {
    crate::declare! {
        /// A serial type's group: an error (empty if none), then the group.
        pub const FIELDS: fields ExtensionGroupPlan {
            ERROR,
            GROUP,
        }
    }
}

/// A theme component.
pub mod theme {
    crate::declare! {
        /// A theme component: an error (empty if none), its trimmed id and label, its stylesheet
        /// and a flag of whether it has one, then its authors.
        pub const FIELDS: fields ExtensionThemePlan {
            ERROR,
            ID,
            LABEL,
            STYLESHEET,
            STYLESHEET_PRESENT,
            /// The first author; the rest follow.
            AUTHORS,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &OPS,
    &RULES,
    &VALIDITY,
    &ARCHIVE_ACTIONS,
    &COMPOSER_TYPES,
    &validation::FIELDS,
    &maintainer::FIELDS,
    &component::FIELDS,
    &archive_entry::FIELDS,
    &manifest::FIELDS,
    &group::FIELDS,
    &theme::FIELDS,
];
