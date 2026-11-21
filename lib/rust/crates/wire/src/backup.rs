//! `BackupNative.call`: its operations, answered by `backup::call`, and the packets and names they
//! pass.

crate::declare! {
    /// The operations `BackupNative.call` takes, answered by `backup::call`.
    pub const OPS: enum BackupOp(i32) {
        /// A backup's package, version and the app's version: a [`metadata_status`] row.
        METADATA = 0,
        /// Where each group lives, as a [`layout`] row of paths.
        LAYOUT = 1,
        /// A [`restore_request`] row, then a [`layout`] row of flags of which are present: an
        /// [`effect`] row for each step.
        RESTORE = 2,
        /// A backup's file name from its package, version and time.
        FILE_NAME = 3,
        /// The mask and preferences path: a [`manifest`] row for each selected group.
        MANIFEST = 4,
        /// A clipboard item kind, then each item's kind: the places of those of that kind.
        ITEMS = 5,
        /// A flag for each incoming item of whether it exists: the places of the new ones.
        INCOMING = 6,
        /// A mask: a [`selection`] row.
        SELECTION = 7,
        /// The fixed names: a [`names`] row.
        NAMES = 8,
        /// The root, then a `wire::extensions::archive_entry` row for each entry: an
        /// `ExtensionArchiveAction` for each.
        ARCHIVE = 9,
    }
}

crate::declare! {
    /// What a restore step does.
    pub const EFFECTS: object BackupEffect(str) {
        PREFERENCES_MERGE = "preferences-merge",
        PREFERENCES_ERASE = "preferences-erase",
        CLEAR_DIRECTORY = "clear-directory",
        COPY_DIRECTORY = "copy-directory",
        CLEAR_CLIPBOARD = "clear-clipboard",
        RESTORE_TEXT = "text",
        RESTORE_IMAGES = "image",
        RESTORE_VIDEO = "video",
    }
}

crate::declare! {
    /// How much of the clipboard is selected.
    pub const CLIPBOARD_STATES: object BackupClipboardState(str) {
        OFF = "off",
        ON = "on",
        MIXED = "mixed",
    }
}

crate::declare! {
    /// What is wrong with a backup's metadata.
    pub const METADATA_MESSAGES: object BackupMetadataMessage(str) {
        DIFFERENT_VERSION = "different_version",
        DIFFERENT_VENDOR = "different_vendor",
        INVALID_METADATA = "invalid_metadata",
    }
}

/// The groups a backup holds.
pub mod layout {
    crate::declare! {
        /// The groups a backup holds, in order. A group's position is also its bit in a mask.
        pub const FIELDS: fields BackupLayout {
            PREFERENCES,
            KEYBOARDS,
            THEMES,
            CLIPBOARD_TEXT,
            CLIPBOARD_IMAGES,
            CLIPBOARD_VIDEO,
        }
    }
}

/// A restore's request.
pub mod restore_request {
    crate::declare! {
        /// A restore: the mask, a flag of whether to erase first, and the preferences path.
        pub const FIELDS: fields BackupRestoreRequest {
            MASK,
            ERASE,
            PREFERENCES,
        }
    }
}

/// A restore step.
pub mod effect {
    crate::declare! {
        /// A restore step: a [`BackupEffect`](super::EFFECTS) and the path it acts on.
        pub const FIELDS: fields BackupEffectRow {
            EFFECT,
            PATH,
        }
    }
}

/// A group to back up.
pub mod manifest {
    crate::declare! {
        /// A group to back up: its [`layout`](super::layout) position and its path.
        pub const FIELDS: fields BackupManifestRow {
            GROUP,
            PATH,
        }
    }
}

/// What a mask selects.
pub mod selection {
    crate::declare! {
        /// What a mask selects.
        pub const FIELDS: fields BackupSelection {
            /// A flag: any group.
            ANY,
            /// A flag: any clipboard group.
            CLIPBOARD,
            /// A [`BackupClipboardState`](super::CLIPBOARD_STATES).
            CLIPBOARD_STATE,
        }
    }
}

/// The fixed names.
pub mod names {
    crate::declare! {
        /// The metadata file's name, the error when it is missing, the clipboard media
        /// directory and the name a picked archive is copied to.
        pub const FIELDS: fields BackupNames {
            METADATA,
            MISSING_METADATA,
            MEDIA_DIRECTORY,
            INPUT_ARCHIVE,
        }
    }
}

/// What is wrong with a backup's metadata.
pub mod metadata_status {
    crate::declare! {
        /// What is wrong with a backup's metadata: a warning and an error, each a
        /// [`BackupMetadataMessage`](super::METADATA_MESSAGES) or empty.
        pub const FIELDS: fields BackupMetadataStatus {
            WARNING,
            ERROR,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &OPS,
    &EFFECTS,
    &CLIPBOARD_STATES,
    &METADATA_MESSAGES,
    &layout::FIELDS,
    &restore_request::FIELDS,
    &effect::FIELDS,
    &manifest::FIELDS,
    &selection::FIELDS,
    &names::FIELDS,
    &metadata_status::FIELDS,
];
