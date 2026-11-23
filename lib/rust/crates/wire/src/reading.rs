//! `ReadingNative`: a reading of the page aloud, step by step (`reader::reading`), and which of
//! the screen's windows it reads (`reader::window`).

/// What Kotlin does next.
pub mod effect {
    crate::declare! {
        /// What Kotlin does next in a reading, at the head of a `ReadingPlan` effect: the code,
        /// then two arguments.
        pub const EFFECTS: object ReadingEffect(i32) {
            /// Show the stop button, on the main thread. Then `done`.
            SHOW_BUTTON = 0,
            /// Take the neural voice, loading it if need be, then ask whether it just failed to
            /// load. `answer` 1 if there is one, plus 2 if it just failed.
            TAKE_VOICE = 1,
            /// Say a `ReadingMessage` (the first argument), long if the second is 1. Then `done`.
            TELL = 2,
            /// Make the reading's voice, on the main thread: the neural one if the first argument
            /// is 1, else the phone's. `answer` 1 if it says it can't speak, else 0.
            MAKE_VOICE = 3,
            /// Take audio focus for the voice. `answer` 1 if Android gave it, else 0.
            BEGIN = 4,
            /// Walk the app's window, skipping this app's own if the first argument is 1.
            /// `looked`, or `lookedNothing` if there was no window.
            WALK = 5,
            /// Wait the first argument's ms. Then `done`.
            WAIT = 6,
            /// The last walk is now the screen read: blocks and scrolls are its. Then `done`.
            KEEP = 7,
            /// Bring the screen's block at the first argument into sight. Then `done`.
            REVEAL = 8,
            /// Scroll the screen's list back a page. `answer` 1 if it moved, else 0.
            SCROLL_BACK = 9,
            /// Scroll the screen's list on a page. `answer` 1 if it moved, else 0.
            SCROLL_FORWARD = 10,
            /// Start bringing each block into sight as the voice starts it. Then `done`.
            FOLLOW = 11,
            /// Stop doing that. Then `done`.
            UNFOLLOW = 12,
            /// Wait until no more than the first argument's blocks are queued. Then `done`.
            AWAIT_BLOCKS = 13,
            /// Queue the screen's block at the first argument, from the UTF-16 offset in the
            /// second, to be brought into sight as it starts. Then `done`.
            QUEUE = 14,
            /// Queue the selected text itself: it isn't on the screen. Then `done`.
            SPEAK_SELECTION = 15,
            /// The reading is over.
            DONE = 16,
        }
    }
}

/// Where a reading starts.
pub mod from {
    crate::declare! {
        /// Where a reading starts: `ReadingPlan.start`.
        pub const FROM: object ReadingFrom(i32) {
            /// The first block on the screen.
            SCREEN = 0,
            /// The top of the page, scrolling back to it first.
            TOP = 1,
            /// Where the selected text is on the screen.
            SELECTION = 2,
        }
    }
}

/// What a reading says.
pub mod message {
    crate::declare! {
        /// What a reading says in a `TELL` effect.
        pub const MESSAGES: object ReadingMessage(i32) {
            /// The neural voice wouldn't load; the phone's reads instead.
            VOICE_FAILED = 0,
            /// There was nothing on the screen to read.
            NOTHING = 1,
            /// Why the voice can't speak, as it said.
            UNAVAILABLE = 2,
        }
    }
}

/// What a walk of the screen found, besides its blocks.
pub mod fact {
    crate::declare! {
        /// The places in a `looked` facts row.
        pub const FACTS: object ReadingFact(i32) {
            /// The top of the part of the screen the app shows.
            VISIBLE_TOP = 0,
            /// Its bottom.
            VISIBLE_BOTTOM = 1,
            /// How many nodes the walk saw.
            NODES = 2,
            /// 1 if it is a web page.
            WEB = 3,
            /// 1 if it has a list that scrolls.
            SCROLLER = 4,
            /// The row's length.
            SIZE = 5,
        }
    }
}

/// What `VoiceHold.take` and `checked` tell Kotlin to do for the neural voice.
pub mod hold {
    crate::declare! {
        /// What to do for a reading that takes the neural voice.
        pub const HOLDS: object VoiceHoldStep(i32) {
            /// Use the voice already loaded.
            USE = 0,
            /// Read without it, with the phone's voice.
            NONE = 1,
            /// Check whether its files are there, and tell `checked`.
            CHECK = 2,
            /// Load it, then tell `loaded` whether that gave a voice.
            LOAD = 3,
        }
    }
}

/// What `ReadingRules.route` says to do with a reading asked for.
pub mod route {
    crate::declare! {
        /// Where a reading asked for goes.
        pub const ROUTES: object ReadRoute(i32) {
            /// The service is up: read.
            READ = 0,
            /// The service is off: ask the user to turn it on.
            ASK = 1,
            /// The service is on but not up yet: wait for it.
            WAIT = 2,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &effect::EFFECTS,
    &from::FROM,
    &message::MESSAGES,
    &fact::FACTS,
    &hold::HOLDS,
    &route::ROUTES,
];
