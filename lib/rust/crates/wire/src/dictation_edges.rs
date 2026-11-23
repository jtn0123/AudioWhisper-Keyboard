//! `DictationEdgesNative`: what a microphone read means (`dictation::audio`) and how a model's
//! files stand (`dictation::model_check`).

/// What a microphone read means.
pub mod read {
    crate::declare! {
        /// What the recording thread does with a read: `MicRules.read`.
        pub const READS: object MicRead(i32) {
            /// Hand the samples on, and their level.
            DELIVER = 0,
            /// Nothing was read: read again.
            SKIP = 1,
            /// The mic failed while recording: say so with the read's error code, and stop.
            REPORT = 2,
            /// The mic failed after it was stopped: stop without a word.
            END = 3,
        }
    }
}

/// How a model's files stand.
pub mod model_state {
    crate::declare! {
        /// How a model's files stand, at the head of a `ModelCheckRules` answer.
        pub const STATES: object ModelCheckState(i32) {
            /// All there and whole.
            READY = 0,
            /// Some are not there at all.
            MISSING = 1,
            /// All there, but some are the wrong size or content.
            DAMAGED = 2,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[&read::READS, &model_state::STATES];
