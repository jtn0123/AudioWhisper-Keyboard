//! What the theme editor asks Kotlin for. The stylesheet being edited, and the component's meta,
//! live in the extension editor's workspace (`CacheManager.ExtEditorWorkspace`), which only Kotlin
//! can reach; Rust parses, edits and serializes the stylesheet and checks the meta, and Kotlin
//! reads the workspace, writes Rust's stylesheet back into it on Save and shows the draft on the
//! keyboard through `ThemeManager.previewThemeInfo`.

/// The effects the theme editor returns.
pub mod effect {
    crate::declare! {
        /// The effects the theme editor returns: each packet's first field.
        pub const KINDS: object SettingsUiThemeEditorEffect(str) {
            /// Read the component: the workspace's uuid and the component's index in the
            /// extension's themes follow. Kotlin answers with a `THEME_EDITOR` report.
            LOAD = "themeEditor",
            /// Write the component back into the workspace, as the extension editor's save will
            /// zip it: the workspace, the index, the stylesheet's JSON, the label, the authors one
            /// per line, and the night flag follow.
            SAVE = "themeEditorSave",
            /// Show the keyboard in the draft: the workspace, the index and the stylesheet's JSON
            /// follow; with nothing after the kind, the keyboard gets its own theme back.
            PREVIEW = "themeEditorPreview",
        }
    }
}

/// How a load ended.
pub mod outcome {
    crate::declare! {
        /// How a `THEME_EDITOR` report's load ended.
        pub const KINDS: object SettingsUiThemeEditorOutcome(str) {
            LOADED = "loaded",
            /// The workspace or the component is gone, as after the process was killed while the
            /// editor was open: there is nothing to edit.
            MISSING = "missing",
        }
    }
}

/// A `THEME_EDITOR` report.
pub mod report {
    crate::declare! {
        /// A `THEME_EDITOR` report. Lists are joined with `/`, which no file name holds.
        pub const FIELDS: fields SettingsUiThemeEditor {
            /// A [`SettingsUiThemeEditorOutcome`](super::outcome::KINDS).
            OUTCOME,
            /// The workspace and the component index the load named.
            WORKSPACE,
            COMPONENT,
            /// The stylesheet's JSON: the workspace's file, or empty when there is none.
            STYLESHEET,
            /// The extension's id, which the component's name starts with.
            EXTENSION,
            /// The component's id, label, authors (one per line) and night flag.
            ID,
            LABEL,
            AUTHORS,
            NIGHT,
            /// Where the stylesheet is kept in the extension: `stylesheetPath()`.
            PATH,
            /// The workspace's font and image file names, for the property editor.
            FONTS,
            IMAGES,
        }
    }
}
