//! The system file picker and the file work the settings screens ask Kotlin for. Rust decides
//! what a picked document is for and what happens next; Kotlin opens the picker, keeps the
//! documents and hands Rust numbered handles, never their addresses.

/// The effects that ask for the picker, file work or a toast.
pub mod effect {
    crate::declare! {
        /// The effects that ask for the picker, file work or a toast: each packet's first field.
        pub const KINDS: object SettingsUiFileEffect(str) {
            /// Open the picker: a [`SettingsUiPick`](super::pick::FIELDS) packet.
            PICK = "pick",
            /// Run file work: a [`SettingsUiWork`](super::work::FIELDS) packet.
            WORK = "work",
            /// Show the next field as a long toast.
            TOAST = "toast",
        }
    }
}

/// What a `FILES` report is.
pub mod report {
    crate::declare! {
        /// What a `FILES` report is: its first field.
        pub const KINDS: object SettingsUiFiles(str) {
            /// The picker closed: a [`SettingsUiPicked`](super::picked::FIELDS) packet, then a
            /// [`SettingsUiDocument`](super::document::FIELDS) row for each document picked.
            PICKED = "picked",
            /// File work ended: a [`SettingsUiDone`](super::done::FIELDS) packet.
            DONE = "done",
            /// A screen attached and no file work is running, so work Rust still waits for was
            /// lost, as when Android closed the app.
            IDLE = "idle",
        }
    }
}

/// How the picker opens.
pub mod mode {
    crate::declare! {
        /// How the picker opens.
        pub const KINDS: object SettingsUiPickMode(str) {
            /// One new document to write, as `ACTION_CREATE_DOCUMENT` with a suggested name.
            CREATE = "create",
            /// One document to read, as `ACTION_GET_CONTENT` with `CATEGORY_OPENABLE`.
            OPEN = "open",
            /// Several documents to read: `OPEN` that allows more than one.
            OPEN_MANY = "openMany",
        }
    }
}

/// How the picker closed.
pub mod outcome {
    crate::declare! {
        /// How the picker closed.
        pub const KINDS: object SettingsUiPickOutcome(str) {
            PICKED = "picked",
            CANCELLED = "cancelled",
            /// No app on the device can show the picker.
            NO_PICKER = "noPicker",
        }
    }
}

/// The file work Kotlin does.
pub mod job {
    crate::declare! {
        /// The file work Kotlin does.
        pub const KINDS: object SettingsUiJob(str) {
            /// Collect the selected groups into a backup archive in the cache.
            BACKUP_PREPARE = "backupPrepare",
            /// Copy the prepared archive into the picked document.
            BACKUP_WRITE = "backupWrite",
            /// Offer the prepared archive to other apps.
            BACKUP_SHARE = "backupShare",
            /// Copy the picked document into the cache, unpack it and read what it holds.
            RESTORE_READ = "restoreRead",
            /// Restore the selected groups from the unpacked backup.
            RESTORE_APPLY = "restoreApply",
            /// Delete the archives no work is using.
            DISCARD = "discard",
            /// Read the picked document as UTF-8 text, as a user dictionary import read it.
            WORDS_READ = "wordsRead",
            /// Write a user dictionary export to the picked document: `dictionary=`, the
            /// document's last path segment, then the work's `TEXT`.
            WORDS_WRITE = "wordsWrite",
            /// Read the picked documents (`DOCUMENT`, their handles joined by `|`) into a new
            /// import workspace. `NAME` reports its id.
            EXT_READ = "extRead",
            /// Import the files at the indexes in `TEXT`, joined by `,`, from the import
            /// workspace `EXTENSION`, then close it.
            EXT_IMPORT = "extImport",
            /// Write the extension `EXTENSION` into the picked document as a `.flex` archive.
            /// Kotlin deletes the document when it could not be written.
            EXT_EXPORT = "extExport",
        }
    }
}

/// How file work ended.
pub mod result {
    crate::declare! {
        /// How file work ended.
        pub const KINDS: object SettingsUiJobResult(str) {
            OK = "ok",
            /// The document could not be opened or read.
            UNREADABLE = "unreadable",
            /// The document is not a ZIP archive holding backup metadata.
            NOT_BACKUP = "notBackup",
            /// The backup's metadata could not be read.
            BAD_METADATA = "badMetadata",
            /// The document could not be written; Kotlin deleted what it had made.
            UNWRITABLE = "unwritable",
            /// No app can take what was offered.
            NO_APP = "noApp",
            /// The archive the work needs is gone, as when Android closed the app.
            GONE = "gone",
            /// Anything else. A restore's `STEPS` says how many of its steps finished.
            FAILED = "failed",
        }
    }
}

/// The `PICK` effect.
pub mod pick {
    crate::declare! {
        /// The `PICK` effect.
        pub const FIELDS: fields SettingsUiPick {
            EFFECT,
            /// A [`SettingsUiPickMode`](super::mode::KINDS).
            MODE,
            /// One or more MIME types, joined by `|`.
            MIME,
            /// The name a new document is offered, or empty.
            NAME,
        }
    }
}

/// A `PICKED` report's head.
pub mod picked {
    crate::declare! {
        /// A `PICKED` report's head; a [`SettingsUiDocument`](super::document::FIELDS) row for
        /// each document follows.
        pub const FIELDS: fields SettingsUiPicked {
            REPORT,
            /// A [`SettingsUiPickOutcome`](super::outcome::KINDS).
            OUTCOME,
        }
    }
}

/// A picked document.
pub mod document {
    crate::declare! {
        /// A picked document: the handle Kotlin keeps it under, and its display name or empty.
        pub const FIELDS: fields SettingsUiDocument {
            HANDLE,
            NAME,
        }
    }
}

/// The `WORK` effect.
pub mod work {
    crate::declare! {
        /// The `WORK` effect. Fields a job does not use are empty.
        pub const FIELDS: fields SettingsUiWork {
            EFFECT,
            /// A [`SettingsUiJob`](super::job::KINDS).
            JOB,
            /// The groups, as a `wire::backup::layout` mask.
            MASK,
            /// A flag: erase before restoring.
            ERASE,
            /// A picked document's handle.
            DOCUMENT,
            /// The text to write.
            TEXT,
            /// An extension's id, or an import workspace's.
            EXTENSION,
        }
    }
}

/// A `DONE` report.
pub mod done {
    crate::declare! {
        /// A `DONE` report. Fields a job does not fill are empty.
        pub const FIELDS: fields SettingsUiDone {
            REPORT,
            /// The [`SettingsUiJob`](super::job::KINDS) that ended.
            JOB,
            /// A [`SettingsUiJobResult`](super::result::KINDS).
            RESULT,
            /// The prepared archive's file name, or the id of the import workspace read.
            NAME,
            /// The backup's package, version code and version name.
            PACKAGE,
            VERSION_CODE,
            VERSION_NAME,
            /// When the backup was made, as the device words a date and time.
            DATE,
            /// The groups the backup holds, as a `wire::backup::layout` mask.
            PRESENT,
            /// The groups whose data could not be read, as a mask.
            DAMAGED,
            /// How many of a restore's steps finished.
            STEPS,
            /// The text read.
            TEXT,
        }
    }
}
