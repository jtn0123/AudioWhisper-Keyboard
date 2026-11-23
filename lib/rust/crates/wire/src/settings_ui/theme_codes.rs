//! The theme editor's code editor. Recording a key takes the next key-up from this app's own
//! keyboard, which only Kotlin reaches (it is the running input method's `InputEventDispatcher`, in
//! this process); a character's Unicode name, its mark category and a key's icon come from ICU and
//! the keyboard's icons, which are Kotlin's too. Rust decides when to record and what the code
//! editor shows. A recorded code is typed content: it goes nowhere but back to Rust.

/// The effects the code editor returns.
pub mod effect {
    crate::declare! {
        /// The effects the code editor returns: each packet's first field.
        pub const KINDS: object SettingsUiThemeCodeEffect(str) {
            /// Take the next key-up on this app's keyboard instead of the keyboard, then give the
            /// keyboard back the receiver it had and answer with a `THEME_CODE` key report.
            RECORD = "themeCodeRecord",
            /// Stop taking keys and give the keyboard back the receiver it had.
            STOP = "themeCodeStop",
            /// Describe the code that follows for the key data preview: Kotlin answers with a
            /// `THEME_CODE` described report.
            DESCRIBE = "themeCodeDescribe",
        }
    }
}

/// What a `THEME_CODE` report says.
pub mod kind {
    crate::declare! {
        /// What a [`SettingsUiThemeCode`](super::report::FIELDS) report says.
        pub const KINDS: object SettingsUiThemeCodeReport(str) {
            /// The key recorded: its code. Kotlin has stopped listening.
            KEY = "key",
            /// Recording ended without a key, as when the screen stopped: Kotlin has stopped
            /// listening, so a key typed in another app is never taken.
            STOPPED = "stopped",
            /// A code described: its name, mark flag and icon.
            DESCRIBED = "described",
        }
    }
}

/// A `THEME_CODE` report.
pub mod report {
    crate::declare! {
        /// A `THEME_CODE` report.
        pub const FIELDS: fields SettingsUiThemeCode {
            /// A [`SettingsUiThemeCodeReport`](super::kind::KINDS).
            KIND,
            /// The key's code, or the code described; empty when stopped.
            CODE,
            /// The character's Unicode name (ICU's name, else its extended name), empty for a code
            /// that is no character.
            NAME,
            /// A [`WireFlag`](crate::flag::FLAG): whether the character is a non-spacing mark,
            /// which the keyboard's label shows on a dotted circle.
            NON_SPACING,
            /// The keyboard's icon token for the code (`InputRules.visual`), which the renderer's
            /// `icons` crate draws; 0 when it draws none.
            ICON,
        }
    }
}
