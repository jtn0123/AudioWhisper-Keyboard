//! What the extension editor (`ext/edit/…`, ExtensionEditScreen's place) asks Kotlin for. Rust
//! holds the draft (the meta, the dependencies and the components) and decides every screen,
//! check and message; Kotlin keeps the editor's workspace, the files the draft is made of
//! unpacked from the archive being edited, with a copy of the draft for the theme editor, and
//! reports what it holds. Saving, adding, renaming and deleting a file, and copying a
//! component's stylesheet are file work (`files::job`).

/// The effects the extension editor returns.
pub mod effect {
    crate::declare! {
        /// The effects the extension editor returns: each packet's first field.
        pub const KINDS: object SettingsUiExtEditorEffect(str) {
            /// Open a workspace: a [`SettingsUiExtEditorOpen`](super::open::FIELDS) packet.
            /// Kotlin answers with an `EXT_EDITOR` report.
            OPEN = "extEditor",
            /// The draft changed: the workspace whose id is the next field keeps the
            /// `extension.json` after it as its copy of the draft, which the theme editor
            /// reads. Kotlin does not answer.
            SYNC = "extEditorSync",
            /// Close the workspace whose id is the next field, deleting it. Kotlin does not
            /// answer.
            CLOSE = "extEditorClose",
            /// Open the theme editor on that workspace's component at the index after it, as
            /// ExtensionEditScreen's Edit did. When the theme editor changed Kotlin's copy of
            /// the draft, Kotlin reports the workspace again as the editor comes back.
            THEME = "extEditorTheme",
        }
    }
}

/// The `OPEN` effect.
pub mod open {
    crate::declare! {
        /// The `OPEN` effect.
        pub const FIELDS: fields SettingsUiExtEditorOpen {
            EFFECT,
            /// The installed extension to edit, or empty to create a theme extension.
            EXTENSION,
            /// The workspace a saved state named, to take up again, or empty for a new one.
            WORKSPACE,
            /// With a `WORKSPACE`, the draft's `extension.json`: the copy Kotlin keeps.
            MANIFEST,
        }
    }
}

/// How opening a workspace ended.
pub mod outcome {
    crate::declare! {
        /// How opening a workspace ended.
        pub const KINDS: object SettingsUiExtEditorOutcome(str) {
            /// The workspace is open; for an edit, with the archive unpacked into it.
            READY = "ready",
            /// No installed theme extension has the id asked for.
            MISSING = "missing",
            /// The workspace the saved state named is gone; nothing was opened.
            GONE = "gone",
            /// The archive could not be unpacked; nothing was opened.
            FAILED = "failed",
        }
    }
}

/// The workspace's folders a file can be in.
pub mod folder {
    crate::declare! {
        /// The workspace's folders a file can be in, ExtensionEditFilesScreen's two lists.
        pub const KINDS: object SettingsUiExtEditorFolder(str) {
            FONTS = "fonts",
            IMAGES = "images",
        }
    }
}

/// What an `EXT_EDITOR` report row is.
pub mod kind {
    crate::declare! {
        /// What a [`SettingsUiExtEditor`](super::row::FIELDS) row is. A report is a
        /// `workspace` row, then the others.
        pub const KINDS: object SettingsUiExtEditorKind(str) {
            /// The workspace: `ID` a [`SettingsUiExtEditorOutcome`](super::outcome::KINDS),
            /// `EXTENSION` the extension asked for, `TEXT` its `extension.json` as kotlinx
            /// encodes it (empty to create, or the copy Kotlin keeps when a workspace was taken
            /// up again), `LABEL` the time now in milliseconds, which names a new extension,
            /// `DETAIL` the Material You flags a new component has, joined by `/`, and `FLAG`
            /// whether the theme editor changed the copy since Rust last sent it.
            WORKSPACE = "workspace",
            /// A file in the workspace, in name order: `ID` its
            /// [`SettingsUiExtEditorFolder`](super::folder::KINDS), `LABEL` its name.
            FILE = "file",
            /// A theme component another installed extension has, as Create component lists
            /// it: `EXTENSION` and `ID` its name's parts, `LABEL` its label, `TEXT` its authors
            /// one per line, `FLAG` whether it is a night theme, `DETAIL` its Material You
            /// palette style, contrast level and spec version names, joined by `/`, and `PATH`
            /// where its archive keeps its stylesheet.
            THEME = "theme",
            /// An installed extension: `EXTENSION` its id, which a new extension cannot take.
            INSTALLED = "installed",
        }
    }
}

/// A row of an `EXT_EDITOR` report.
pub mod row {
    crate::declare! {
        /// A row of an `EXT_EDITOR` report. Flags are `wire::flag`s.
        pub const FIELDS: fields SettingsUiExtEditor {
            /// A [`SettingsUiExtEditorKind`](super::kind::KINDS).
            KIND,
            /// The workspace the report is about, on every row.
            WORKSPACE,
            EXTENSION,
            ID,
            LABEL,
            DETAIL,
            TEXT,
            FLAG,
            PATH,
        }
    }
}
