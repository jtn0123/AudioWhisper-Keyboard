//! `NlpNative.event`: one typing event in, one plan out, answered by `nlp::Session::event`.
//!
//! The session counts, learns and teaches the tap trail itself; the plan names only what Android
//! must do after it: when to save, whether to write the kept words, whether a picked candidate may
//! go in, how to undo an auto-correction, and a [`crate::nlp::snapshot`] to publish. Key presses
//! and the screen's orientation go to the tap trail directly: they are the hit test's input, on
//! the touch path, and change nothing this plans.

/// What happened.
pub mod event {
    crate::declare! {
        /// What happened, with the `NlpEvent` fields each kind reads. A kind marked "with a
        /// context" takes an `NlpContext` row after its event row, which the session configures
        /// itself to first, as the editor stands.
        pub const KINDS: object NlpEventKind(i32) {
            /// A key that types went down: a space or Enter.
            KEY = 0,
            /// A tapped character went in: `WORD` is its text.
            TYPED = 1,
            /// The tap trail's last character was deleted.
            TAP_DELETED = 2,
            /// A backspace, with the selection at `SELECTION_START`..`SELECTION_END` before it.
            BACKSPACE = 3,
            /// The undo key.
            UNDO = 4,
            /// With a context: the word being typed (`WORD`) was finished by `TRIGGER` (-1 for a
            /// space or Enter) and stays as typed.
            FINISHED = 5,
            /// With a context: the suggestion `CANDIDATE` (empty when none is shown) was tapped.
            /// With `COMMIT` it goes in, so a word candidate (`WORD_CANDIDATE`) whose text is
            /// `WORD` is learned; with `CORRECTS` too it is the auto-correction of `TYPED`, which a
            /// backspace straight after undoes.
            PICKED = 6,
            /// With a context: the word `WORD` was swiped in.
            SWIPED = 7,
            /// What was just done can no longer undo an auto-correction.
            DISARM = 8,
            /// With a context: a backspace that may undo the last auto-correction, with the cursor
            /// at `SELECTION_START` if `CURSOR_MODE`, and `TEXT_BEFORE_SELECTION` before it.
            UNDO_CORRECTION = 9,
            /// The undo an `UNDO_CORRECTION` plan asked for put back what was typed.
            CORRECTION_UNDONE = 10,
            /// Input moved to another field.
            FIELD = 11,
        }
    }
}

/// An event.
pub mod row {
    crate::declare! {
        /// One typing event: its `NlpEventKind` and the facts that kind reads; the rest are empty.
        pub const FIELDS: fields NlpEvent {
            /// An `NlpEventKind`.
            KIND,
            /// The uptime in milliseconds, which saves are timed by.
            NOW,
            /// A flag: the setting to learn where this user taps.
            AIM,
            /// The code point that finished the word, or -1 for none.
            TRIGGER,
            /// The word, or the text typed.
            WORD,
            /// The text before the word being typed.
            BEFORE_COMPOSING,
            /// Where the word being typed starts, or -1.
            COMPOSING_START,
            /// A flag: the local composing region is valid.
            COMPOSING_VALID,
            /// The text before the selection.
            TEXT_BEFORE_SELECTION,
            SELECTION_START,
            SELECTION_END,
            /// The word being typed.
            TYPED,
            /// The id of the candidate tapped.
            CANDIDATE,
            /// A flag: the candidate is a word.
            WORD_CANDIDATE,
            /// A flag: the candidate goes in, rather than being only asked about.
            COMMIT,
            /// A flag: the candidate is the auto-correction of the word being typed.
            CORRECTS,
            /// A flag: the selection is a cursor.
            CURSOR_MODE,
        }
    }
}

/// What to do after an event.
pub mod plan {
    crate::declare! {
        /// What Android does after an event, then an `NlpSnapshot` row if `PUBLISH`.
        pub const FIELDS: fields NlpEventPlan {
            /// Milliseconds to wait before saving the learned words, the tap aim and the counters,
            /// or -1 for no save.
            SAVE,
            /// A second save to time the same way, or -1: asked for when a save that was due
            /// had not run yet.
            SAVE_AGAIN,
            /// A flag: write the kept words now.
            KEPT,
            /// A flag: the candidate tapped may go in.
            ACCEPTED,
            /// How many characters before the cursor to replace to undo the auto-correction, or
            /// -1 for no undo.
            REVERT,
            /// What to put in their place.
            REVERT_TEXT,
            /// A flag: leave a phantom space after it.
            PHANTOM,
            /// A flag: a snapshot follows, to publish.
            PUBLISH,
        }
    }
}

pub const GROUPS: &[&crate::Group] = &[&event::KINDS, &row::FIELDS, &plan::FIELDS];
