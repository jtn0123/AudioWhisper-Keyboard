//! The first-run setup screen. Rust decides which step is open, when to ask for the notification
//! permission and what each answer means; Kotlin makes the Android calls it cannot: the system's
//! input method settings and picker, the permission request, the app's notification settings, and
//! bringing the app back to the front.

/// The effects the setup screen returns.
pub mod effect {
    crate::declare! {
        /// The effects the setup screen returns: each packet's first field. Opening the system's
        /// input method settings and its picker are the `enable` and `picker` effects Home uses.
        pub const KINDS: object SettingsUiSetupEffect(str) {
            /// Ask for `POST_NOTIFICATIONS` and answer with a `PERMISSION` packet.
            ASK = "askNotifications",
            /// Open the system's notification settings for this app: the permission was denied
            /// for good, so asking again would show nothing.
            NOTIFICATION_SETTINGS = "notificationSettings",
            /// The keyboard was enabled while the user was in the system's settings: bring the app
            /// back to the front, as Compose's setup did.
            RETURN = "setupReturn",
        }
    }
}

/// A `PERMISSION` packet.
pub mod answer {
    crate::declare! {
        /// A `PERMISSION` packet: how the notification permission request ended.
        pub const FIELDS: fields SettingsUiPermission {
            /// `true` when the permission is granted, else `false`.
            GRANTED,
            /// `shouldShowRequestPermissionRationale` after the answer, `true` or `false`: false
            /// after a denial means Android will not ask again.
            RATIONALE,
        }
    }
}

/// The facts the setup screen reads from settings op 20.
pub mod fact {
    crate::declare! {
        /// The facts the setup screen reads from settings op 20, each `true` or `false`.
        pub const KEYS: object SettingsUiSetupFact(str) {
            /// Whether `POST_NOTIFICATIONS` is granted now.
            NOTIFICATIONS = "notifications_granted",
            /// `shouldShowRequestPermissionRationale(POST_NOTIFICATIONS)` now.
            RATIONALE = "notifications_rationale",
        }
    }
}
