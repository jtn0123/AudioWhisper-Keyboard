//! `PlaybackNative.call` and `.system`: their operations, answered by `reader::play_queue`, and
//! the rows they answer with.

crate::declare! {
    /// The operations `PlaybackNative.call` takes, answered by `reader::play_queue::PlayQueue::call`.
    /// Each takes a block id and a frame, where it needs them.
    pub const OPS: enum PlaybackOp(i32) {
        /// Queue a block.
        ADD = 0,
        /// A block began playing at the frame.
        BEGINS = 1,
        /// A block finished playing at the frame.
        ENDS = 2,
        /// The playback head reached the frame: a [`heard`] row, then the ids of the blocks that
        /// started.
        HEARD = 3,
        /// Drop every block: how many there were.
        CLEAR = 4,
        /// How many blocks are queued.
        COUNT = 5,
    }
}

crate::declare! {
    /// The parts of a system speech utterance id: the block, the separator, then the sentence's
    /// place or [`LAST`].
    pub const UTTERANCE: object PlaybackUtterance(str) {
        SEPARATOR = ":",
        /// The block's last sentence.
        LAST = "last",
    }
}

/// What the playback head passed.
pub mod heard {
    crate::declare! {
        /// What the playback head passed; the ids of the blocks that started follow.
        pub const FIELDS: fields PlaybackHeard {
            /// How many blocks finished.
            FINISHED,
        }
    }
}

/// System speech progress.
pub mod speech {
    crate::declare! {
        /// The operations `PlaybackNative.system` takes, answered by
        /// `reader::play_queue::PlayQueue::system`. Each answers with a [`FIELDS`] row.
        pub const OPS: enum PlaybackSpeechOp(i32) {
            /// Queue a block, by id.
            QUEUE = 0,
            /// The engine started an utterance, by its
            /// [`PlaybackUtterance`](super::UTTERANCE) id.
            START = 1,
            /// The engine finished an utterance, or gave up on it.
            END = 2,
            /// Drop every block.
            STOP = 3,
            /// Just the row.
            STATE = 4,
        }
    }

    crate::declare! {
        /// What a speech operation changed.
        pub const FIELDS: fields PlaybackSpeech {
            /// The block that started or ended, or -1.
            BLOCK,
            /// 1 if `BLOCK` started or ended; after `STOP`, how many blocks were dropped.
            MATCHED,
            /// How many blocks are still to finish.
            PENDING,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &OPS,
    &UTTERANCE,
    &heard::FIELDS,
    &speech::OPS,
    &speech::FIELDS,
];
