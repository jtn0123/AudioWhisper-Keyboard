//! The read-only selectable text the project licence screen draws, as Compose's
//! SelectionContainer over a Text. Rust lays the text out, owns the selection, its handles and
//! the toolbar's place, and answers TalkBack; Kotlin reads the asset, shows Android's floating
//! toolbar and puts copied text on the clipboard.

/// The effects the read-only text returns.
pub mod effect {
    crate::declare! {
        /// The effects the read-only text returns: each packet's first field.
        pub const KINDS: object SettingsUiTextEffect(str) {
            /// Read the app asset at the next field's path off the main thread and answer with
            /// an `ASSET` report.
            ASSET = "asset",
            /// Put the next field on the clipboard as plain text, as Compose's
            /// `AndroidClipboardManager.setText` does.
            CLIP = "clip",
            /// The selection moved under the finger: `HapticFeedbackConstants.TEXT_HANDLE_MOVE`.
            HAPTIC = "textHaptic",
        }
    }
}

/// How an asset read ended.
pub mod outcome {
    crate::declare! {
        /// How an asset read ended: an `ASSET` report's second field.
        pub const KINDS: object SettingsUiAssetOutcome(str) {
            /// The asset was read; its text follows.
            OK = "ok",
            /// The read threw; the exception's message follows, or nothing when it had none.
            FAILED = "failed",
        }
    }
}

/// What a `SELECTION` operation does.
pub mod selection {
    crate::declare! {
        /// What a `SELECTION` operation does: its first field.
        pub const KINDS: object SettingsUiSelection(str) {
            /// The toolbar's or TalkBack's Copy: put the selected text on the clipboard.
            COPY = "copy",
            /// The toolbar's Select all.
            SELECT_ALL = "selectAll",
            /// TalkBack's `ACTION_SET_SELECTION`: the start and end, in UTF-16 units, follow.
            SET = "set",
            /// TalkBack's next or previous at a movement granularity: the
            /// `AccessibilityNodeInfo` granularity, then `1` forward or `0` back, then `1` to
            /// extend the selection or `0` to move the cursor.
            TRAVERSE = "traverse",
        }
    }
}

/// The read-only text's accessibility nodes.
pub mod node {
    crate::declare! {
        /// The read-only text's accessibility node ids, which Kotlin routes the text actions by.
        pub const IDS: object SettingsUiTextNode(i32) {
            /// The selectable text itself.
            TEXT = 11,
            /// The horizontal scroller around it.
            SCROLL = 12,
        }
    }
}
