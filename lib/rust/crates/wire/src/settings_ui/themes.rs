//! What the theme manager asks Kotlin for. Rust groups, sorts and draws the installed themes and
//! writes the day or night theme's preference; Kotlin reads the extension index and sets the
//! keyboard's preview theme, which live in the extension manager and the theme manager.

/// The effects the theme manager returns.
pub mod effect {
    crate::declare! {
        /// The effects the theme manager returns: each packet's first field.
        pub const KINDS: object SettingsUiThemeEffect(str) {
            /// Read the installed theme extensions. Kotlin answers with a `THEMES` report, and
            /// with another whenever the index changes.
            LOAD = "themes",
            /// Show the keyboard in the theme that follows, an `extension:component` name, as
            /// ThemeManager's `previewThemeId`; empty gives the keyboard back its own theme.
            PREVIEW = "themePreview",
        }
    }
}

/// What a report row is.
pub mod kind {
    crate::declare! {
        /// What a [`SettingsUiTheme`](super::row::FIELDS) report row is.
        pub const KINDS: object SettingsUiThemeKind(str) {
            /// A theme extension, in the index's order: `EXTENSION` its id, `LABEL` its title and
            /// `DELETABLE` whether the extension manager can delete it.
            EXTENSION = "extension",
            /// A theme of the extension row before it, in the extension's order: `COMPONENT` its
            /// id, `LABEL` its label and `NIGHT` whether it is a night theme.
            THEME = "theme",
        }
    }
}

/// A row of a `THEMES` report.
pub mod row {
    crate::declare! {
        /// A row of a `THEMES` report. Flags are `wire::flag`s.
        pub const FIELDS: fields SettingsUiTheme {
            /// A [`SettingsUiThemeKind`](super::kind::KINDS).
            KIND,
            /// The extension's id.
            EXTENSION,
            /// The theme's id; empty on an extension row.
            COMPONENT,
            LABEL,
            /// A flag: whether the theme is a night theme.
            NIGHT,
            /// A flag: whether the extension can be deleted, so it was imported or made here.
            DELETABLE,
        }
    }
}
