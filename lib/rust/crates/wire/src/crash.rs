//! `CrashNative`: what a crash leaves behind and the report the crash screen shows
//! (`utilities::crash`).

/// What to do after a crash, as flags.
pub mod after {
    crate::declare! {
        /// What `CrashRules.after` says to do after a crash, as flags: none, or either, or both.
        pub const AFTER: object CrashAfter(i32) {
            /// Read the last crash's report and drop it if this one is the same.
            CHECK_LAST = 1,
            /// Crashes are coming too fast: say so, and switch away from the keyboard.
            LOOP = 2,
        }
    }
}

/// The facts a report is made of.
pub mod fact {
    crate::declare! {
        /// Indexes into the facts row `CrashRules.report` takes.
        pub const FACTS: object CrashFact(i32) {
            /// The app's version name.
            VERSION_NAME = 0,
            /// The app's version code.
            VERSION_CODE = 1,
            /// The link to the build's commit, for a debug build.
            COMMIT_URL = 2,
            /// The link to the version's changelog, for a release build.
            CHANGELOG_URL = 3,
            /// The device's name.
            DEVICE = 4,
            /// The Android version.
            ANDROID = 5,
            /// The debug log's header.
            DEBUG_LOG = 6,
            /// The row's length.
            SIZE = 7,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[&after::AFTER, &fact::FACTS];
