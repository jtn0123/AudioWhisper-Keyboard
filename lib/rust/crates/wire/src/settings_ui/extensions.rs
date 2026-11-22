//! What the extension screens (the addon home, the extension lists, an extension's page and Check
//! for updates) ask Kotlin for. Rust draws them, builds the update link and decides what each
//! page shows; Kotlin reads the extension index and deletes an extension, which live in the
//! extension manager, and names a link's host as Android's `Uri` parses it.
//!
//! A value the index holds as null is a row left out: an extension with no description sends no
//! `description` row, and a maintainer with no email no `email` row.

/// The effects the extension screens return.
pub mod effect {
    crate::declare! {
        /// The effects the extension screens return: each packet's first field.
        pub const KINDS: object SettingsUiExtensionEffect(str) {
            /// Read the installed extensions. Kotlin answers with an `EXTENSIONS` report, and
            /// with another whenever an index changes.
            LOAD = "extensions",
            /// Delete the extension whose id is the next field. Kotlin answers with an
            /// `EXTENSION_DONE` packet, after a toast when the delete failed.
            DELETE = "extensionDelete",
        }
    }
}

/// What a report row is.
pub mod kind {
    crate::declare! {
        /// What a [`SettingsUiExtension`](super::row::FIELDS) row is. Every row but an
        /// extension's belongs to the extension row before it.
        pub const KINDS: object SettingsUiExtensionKind(str) {
            /// An extension: the keyboard extensions, then the themes, then the language packs,
            /// each in its index's order. `ID` its serial type, `LABEL` its title, `DETAIL` its
            /// version, `TEXT` its license and `FLAG` whether the extension manager can delete
            /// it.
            EXTENSION = "extension",
            /// The extension's description: `TEXT`.
            DESCRIPTION = "description",
            /// The extension's homepage: `TEXT` the link as written, `DETAIL` its host as
            /// `FlorisRef.fromUrl(link).authority` reads it.
            HOMEPAGE = "homepage",
            /// The extension's issue tracker, as a homepage.
            ISSUE_TRACKER = "issueTracker",
            /// One of the extension's keywords, in order: `LABEL`.
            KEYWORD = "keyword",
            /// One of the extension's maintainers, in order: `LABEL` its name.
            MAINTAINER = "maintainer",
            /// The email of the maintainer row before it: `TEXT`.
            EMAIL = "email",
            /// The website of the maintainer row before it: `TEXT`.
            URL = "url",
            /// One of a theme or language pack extension's components, in order: `ID` its id,
            /// `LABEL` its label, `DETAIL` a theme's `stylesheetPath()` or a language pack item's
            /// locale tag, and `FLAG` whether a theme is a night theme.
            COMPONENT = "component",
            /// One of the component row before it's authors, in order: `LABEL`.
            AUTHOR = "author",
        }
    }
}

/// A row of an `EXTENSIONS` report.
pub mod row {
    crate::declare! {
        /// A row of an `EXTENSIONS` report; a report is any number of them. Flags are
        /// `wire::flag`s.
        pub const FIELDS: fields SettingsUiExtension {
            /// A [`SettingsUiExtensionKind`](super::kind::KINDS).
            KIND,
            /// The id of the extension the row belongs to.
            EXTENSION,
            ID,
            LABEL,
            DETAIL,
            TEXT,
            FLAG,
        }
    }
}

/// What an `EXTENSION_DONE` packet reports.
pub mod work {
    crate::declare! {
        /// The work an [`SettingsUiExtensionDone`](super::done::FIELDS) packet reports on.
        pub const KINDS: object SettingsUiExtensionWork(str) {
            /// An `extensionDelete` effect.
            DELETE = "delete",
        }
    }
}

/// An `EXTENSION_DONE` packet.
pub mod done {
    crate::declare! {
        /// How work on an extension ended.
        pub const FIELDS: fields SettingsUiExtensionDone {
            /// A [`SettingsUiExtensionWork`](super::work::KINDS).
            WORK,
            /// The extension's id.
            EXTENSION,
            /// A flag: whether the work succeeded.
            DONE,
        }
    }
}
