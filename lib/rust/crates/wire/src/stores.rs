//! The numbers the keyboard's stores (crates/stores) pass across JNI: a clip's and a word's
//! flags, how a store's file was found, and what importing an old database did.

/// A clip's flags.
pub mod clip_flag {
    crate::declare! {
        /// A clip's flags, as the bits of one `Int`. The store keeps the first three; a row of
        /// the history adds the last two, as its texts cross as empty when there are none.
        pub const FLAGS: enum ClipFlag(i32) {
            /// Pinned: clearing the history and its size limit leave it.
            PINNED = 1,
            /// Marked sensitive by the app that copied it.
            SENSITIVE = 2,
            /// Copied on another device.
            REMOTE = 4,
            /// It has a text.
            HAS_TEXT = 8,
            /// It has a URI.
            HAS_URI = 16,
        }
    }
}

/// A user dictionary word's flags.
pub mod word_flag {
    crate::declare! {
        /// Which of a word's optional texts it has, as the bits of one `Int` in a row of the
        /// user dictionary, whose texts cross as empty when there are none.
        pub const FLAGS: enum WordFlag(i32) {
            /// It has a locale; without one it is for every language.
            HAS_LOCALE = 1,
            /// It has a shortcut.
            HAS_SHORTCUT = 2,
        }
    }
}

/// How a store's file was found.
pub mod opened {
    crate::declare! {
        /// How a store's file was found when it was opened, in `stores::store::Opened`'s order.
        pub const STATES: enum StoreOpened(i32) {
            /// There was none; the first change writes it.
            MISSING = 0,
            LOADED = 1,
            /// Not a whole store: it starts empty, and its first change replaces the file.
            DAMAGED = 2,
            /// Written by a newer version: kept as it is, and the store refuses changes.
            NEWER = 3,
            /// It could not be read: kept as it is, and the store refuses changes.
            UNREADABLE = 4,
        }
    }
}

/// What importing an old database did.
pub mod imported {
    crate::declare! {
        /// What importing an old Room database did, in `stores::legacy::Imported`'s order.
        pub const OUTCOMES: enum StoreImport(i32) {
            /// It had no rows; its files are gone.
            NOTHING = 0,
            /// Its rows are in the store, and its files are gone.
            ROWS = 1,
            /// The store had its rows already; its files are gone.
            ALREADY = 2,
            /// The store keeps a file it could not read, so the database stays.
            KEPT = 3,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &clip_flag::FLAGS,
    &word_flag::FLAGS,
    &opened::STATES,
    &imported::OUTCOMES,
];
