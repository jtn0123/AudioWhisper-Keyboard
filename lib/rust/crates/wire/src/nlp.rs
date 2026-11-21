//! `NlpNative.call`: its operations, answered by `nlp::Session::checked_call`, and the packets they
//! pass.
//!
//! Many calls answer with a [`snapshot`]: what the suggestion row shows.

crate::declare! {
    /// The operations `NlpNative.call` takes, answered by `nlp::Session::checked_call`.
    pub const OPS: enum NlpOp(str) {
        /// A [`context`] row: a [`snapshot`].
        CONTEXT = "context",
        /// A [`context`] row: a [`policy`] row.
        POLICY = "policy",
        /// A [`context`] row: the clipboard request's generation, then a [`snapshot`].
        CLIPBOARD_BEGIN_FOR = "clipboardBeginFor",
        /// A [`context`] row, then a [`typing`] row: a [`typing`] row, or nothing.
        TYPING_POLICY = "typingPolicy",
        /// A [`context`] row, a flag of whether the word was kept as typed, then a flag for each
        /// candidate of whether the decoder would auto-correct to it: a flag for each of whether it
        /// may.
        CANDIDATE_PLAN = "candidatePlan",
        /// The requested and current [`context`] rows, the clipboard request's generation, then a
        /// [`candidate`] row for each: a [`snapshot`].
        CLIPBOARD_FOR = "clipboardFor",
        /// A [`context`] row and the time: a [`begin`] row, then a [`snapshot`].
        BEGIN_FOR = "beginFor",
        /// A [`context`] row, the request's generation, then a [`candidate`] row for each: a flag of
        /// whether they were taken, then a [`snapshot`].
        COMPLETE_FOR = "completeFor",
        /// A language: a flag of whether the dictionary covers it.
        LANGUAGE = "language",
        /// The text before the cursor, as the next word sees it.
        NEXT = "next",
        /// A suggestion provider: a flag of whether it forces suggestions on.
        FORCE = "force",
        /// The time a glide preview starts.
        PREVIEW_START = "previewStart",
        /// A language and a word: a flag of whether to check its spelling.
        SPELL = "spell",
        /// The time: a [`begin`] row's generation and delay.
        BEGIN = "begin",
        /// How many inline suggestions there are: an [`inline_begin`] row.
        INLINE_BEGIN = "inlineBegin",
        /// An inline request's generation: a [`snapshot`].
        INLINE_CANCEL = "inlineCancel",
        /// The time: how long to wait before saving, 0 to save now, or -1 if a save is due.
        SAVE = "save",
        /// A request's generation and the time: a flag of whether to run it now.
        READY = "ready",
        /// The code point that ends the word (-1 for none) and a flag of whether a space is
        /// pending: a flag of whether to look for an auto-correction.
        AUTO = "auto",
        /// A [`snapshot`] with the suggestions cleared.
        CLEAR = "clear",
        /// A [`snapshot`].
        SNAPSHOT = "snapshot",
        /// A new field: a [`snapshot`].
        FIELD = "field",
        /// An external request's generation.
        EXTERNAL_BEGIN = "externalBegin",
        /// A flag of whether the row shows words or holds their place.
        WORDS = "words",
        /// A [`typing`] row: a [`typing`] row, or nothing.
        TYPING = "typing",
        /// The time, a flag of whether previews are on, and the delay: a flag of whether to refresh
        /// the glide preview.
        PREVIEW = "preview",
        /// How many glide suggestions there are, how many to show, and a flag of whether to commit
        /// the first: a [`glide_plan`] row.
        GLIDE_PLAN = "glidePlan",
        /// A [`learn`] row: a [`learn_plan`] row, or nothing.
        LEARN = "learn",
        /// An [`aim`] row: a flag of whether the word may teach where the user taps.
        AIM = "aim",
        /// The requested and current [`context`] rows, then a [`candidate`] row: a flag of whether
        /// it is the auto-correction.
        AUTO_CANDIDATE = "autoCandidate",
        /// A [`composing`] row: a [`composing_range`] row.
        COMPOSING = "composing",
        /// A word kept as typed: a flag of whether to remember it.
        KEEP = "keep",
        /// A candidate's id: a flag of whether it may be committed.
        PICK = "pick",
        /// An external request's generation: a flag of whether its result still applies.
        EXTERNAL_VALID = "externalValid",
        /// A request's generation, then a [`candidate`] row for each: a flag of whether they were
        /// taken.
        COMPLETE = "complete",
        /// A [`candidate`] row for each, shown at once: a [`snapshot`].
        DIRECT = "direct",
        /// A [`candidate`] row for each clipboard suggestion: a [`snapshot`].
        CLIPBOARD = "clipboard",
        /// An inline request's generation, then an [`inline_view`] row for each: a flag of whether
        /// they were taken.
        INLINE_COMPLETE = "inlineComplete",
        /// A [`clipboard_plan`] row, then a [`clipboard_span`] row for each match: nothing, or -1
        /// for the item itself followed by the place of each match worth suggesting.
        CLIPBOARD_PLAN = "clipboardPlan",
        /// A [`tally`] row.
        TALLY = "tally",
        /// The saved counters' text.
        TALLY_LOAD = "tallyLoad",
        /// The counters as text.
        TALLY_TEXT = "tallyText",
        /// A flag of whether a counter changed since the last time this was asked.
        TALLY_CHANGED = "tallyChanged",
        /// Every counter back to zero.
        TALLY_RESET = "tallyReset",
    }
}

/// The editor and settings a request is for.
pub mod context {
    crate::declare! {
        /// The editor and settings a request is for. Not a hash: two contexts are the same only
        /// when every field is.
        pub const FIELDS: fields NlpContext {
            /// The raw input type.
            INPUT_TYPE,
            /// A flag: a raw input editor.
            RAW_EDITOR,
            /// The suggestion provider.
            PROVIDER,
            /// The primary locale's language.
            LANGUAGE,
            /// A flag: word suggestions are on.
            SUGGESTIONS,
            /// A flag: emoji suggestions are on.
            EMOJI_SUGGESTIONS,
            /// A flag: learning from typing is on.
            LEARN,
            /// A flag: incognito.
            INCOGNITO,
            /// A flag: the Smartbar is on.
            SMARTBAR,
            /// A flag: the local selection is valid.
            SELECTION_VALID,
            SELECTION_START,
            SELECTION_END,
            COMPOSING_START,
            COMPOSING_END,
            /// The text the editor shows.
            TEXT,
            /// The subtype.
            SUBTYPE,
            /// The prefix that asks for emoji, or empty.
            EMOJI_PREFIX,
            /// A flag: the provider is ready.
            READY,
            /// A flag: inline suggestions are on.
            INLINE_SUGGESTIONS,
            /// A flag: possibly offensive words are blocked.
            BLOCK_OFFENSIVE,
            /// Where the text starts in the field.
            OFFSET,
            /// The raw IME options.
            IME_OPTIONS,
        }
    }
}

/// What a context allows.
pub mod policy {
    crate::declare! {
        /// What a context allows: a flag each.
        pub const FIELDS: fields NlpPolicy {
            /// Suggestions show.
            ON,
            WORDS,
            EMOJI,
            AUTOCORRECT,
            LEARN,
            /// Word or emoji suggestions are on, whatever the field.
            ENABLED,
        }
    }
}

/// A suggestion request that began.
pub mod begin {
    crate::declare! {
        /// A suggestion request that began. `BEGIN` answers only its generation and delay.
        pub const FIELDS: fields NlpBegin {
            GENERATION,
            /// How long to wait before running it, or -1 to not run it.
            DELAY,
            /// A flag: ask for words.
            WORDS,
            /// A flag: ask for emoji.
            EMOJI,
        }
    }
}

/// The head of what the suggestion row shows.
pub mod snapshot {
    crate::declare! {
        /// The head of what the suggestion row shows. Then come `COUNT` candidate ids, how many
        /// inline suggestions there are, and their ids.
        pub const FIELDS: fields NlpSnapshot {
            GENERATION,
            /// 1 to show the actions toolbar, 0 to show the row, or -1 to keep what is shown.
            EXPANDED,
            /// The id of the candidate to auto-commit, or empty.
            AUTO,
            /// How many candidates show.
            COUNT,
        }
    }
}

/// A suggestion candidate.
pub mod candidate {
    crate::declare! {
        /// A suggestion candidate.
        pub const FIELDS: fields NlpCandidate {
            /// The platform's id for it.
            ID,
            TEXT,
            /// An [`NlpCandidateKind`](KINDS).
            KIND,
            /// A flag: it may auto-commit.
            AUTO,
        }
    }

    crate::declare! {
        /// What a candidate is.
        pub const KINDS: object NlpCandidateKind(i32) {
            WORD = 0,
            EMOJI = 1,
            CLIPBOARD = 2,
        }
    }
}

/// The word being typed and the text around it.
pub mod typing {
    crate::declare! {
        /// The word being typed and the text around it.
        pub const FIELDS: fields NlpTyping {
            BEFORE,
            TYPED,
            AFTER,
        }
    }
}

/// A glide result's plan.
pub mod glide_plan {
    crate::declare! {
        /// A glide result's plan: show the suggestions from `FIRST` to before `END`.
        pub const FIELDS: fields NlpGlidePlan {
            /// 1 when the first is committed, else 0.
            FIRST,
            END,
            /// The suggestion to commit, or -1.
            COMMIT,
        }
    }
}

/// A finished word to learn from.
pub mod learn {
    crate::declare! {
        /// A finished word to learn from.
        pub const FIELDS: fields NlpLearn {
            /// An [`NlpLearnKind`](KINDS).
            KIND,
            /// The code point that ended it, or -1.
            TRIGGER,
            WORD,
            /// The text before the composing region.
            BEFORE_COMPOSING,
            COMPOSING_START,
            /// A flag: the composing region is valid.
            COMPOSING_VALID,
            TEXT_BEFORE_SELECTION,
        }
    }

    crate::declare! {
        /// How the word was finished.
        pub const KINDS: object NlpLearnKind(i32) {
            /// Typed, and kept as typed.
            TYPED = 0,
            /// A candidate picked.
            PICKED = 1,
            SWIPED = 2,
        }
    }
}

/// What the learner does with a finished word.
pub mod learn_plan {
    crate::declare! {
        /// What the learner does with a finished word.
        pub const FIELDS: fields NlpLearnPlan {
            /// An [`NlpLearnKind`](super::learn::KINDS).
            KIND,
            WORD,
            /// The text before it.
            BEFORE,
            /// Where it starts.
            START,
        }
    }
}

/// A kept word that may teach where the user taps.
pub mod aim {
    crate::declare! {
        /// A kept word that may teach where the user taps.
        pub const FIELDS: fields NlpAim {
            /// A flag: the setting is on.
            SETTING,
            /// The code point that ended it, or -1.
            TRIGGER,
            WORD,
            /// A flag: the composing region is valid.
            COMPOSING_VALID,
        }
    }
}

/// The last word before the cursor, as a word break iterator found it.
pub mod composing {
    crate::declare! {
        /// The last word before the cursor, as a word break iterator found it.
        pub const FIELDS: fields NlpComposing {
            /// The iterator's rule status: 0 when it is not a word.
            STATUS,
            START,
            END,
            /// The text before the selection.
            TEXT,
        }
    }
}

/// The composing region.
pub mod composing_range {
    crate::declare! {
        /// The composing region, each -1 when there is none.
        pub const FIELDS: fields NlpComposingRange {
            START,
            END,
        }
    }
}

/// Inline suggestions that began.
pub mod inline_begin {
    crate::declare! {
        /// Inline suggestions that began.
        pub const FIELDS: fields NlpInlineBegin {
            GENERATION,
            /// A flag: inflate them.
            PENDING,
            /// How long to wait for them, in milliseconds.
            TIMEOUT,
        }
    }
}

/// An inflated inline suggestion.
pub mod inline_view {
    crate::declare! {
        /// An inflated inline suggestion.
        pub const FIELDS: fields NlpInlineView {
            /// The platform's id for it.
            ID,
            /// A flag: pinned.
            PINNED,
        }
    }
}

/// The clipboard item to suggest.
pub mod clipboard_plan {
    crate::declare! {
        /// The clipboard item to suggest.
        pub const FIELDS: fields NlpClipboardPlan {
            /// A flag: clipboard suggestions are on.
            ENABLED,
            ITEM_ID,
            /// The id of the last item suggested.
            LAST_ID,
            /// The text the editor shows.
            CONTENT_TEXT,
            ITEM_TEXT,
            /// How old the item is, in milliseconds.
            AGE,
            /// How old it may be, in milliseconds.
            TIMEOUT,
            /// A flag: the item is sensitive.
            SENSITIVE,
        }
    }
}

/// A match in the clipboard item.
pub mod clipboard_span {
    crate::declare! {
        /// A match in the clipboard item: its text and its first and last character.
        pub const FIELDS: fields NlpClipboardSpan {
            VALUE,
            START,
            END,
        }
    }
}

/// A typing event for the counters.
pub mod tally {
    crate::declare! {
        /// A typing event for the counters: the selection's start and end matter only when
        /// something was deleted.
        pub const FIELDS: fields NlpTally {
            /// An [`NlpTallyEvent`](EVENTS).
            EVENT,
            START,
            END,
        }
    }

    crate::declare! {
        /// The typing events Kotlin passes to the counters.
        pub const EVENTS: enum NlpTallyEvent(str) {
            /// A key that types went down.
            KEY = "key",
            /// A backspace.
            DELETED = "deleted",
            /// A backspace undid the auto-correction.
            UNDONE = "undone",
            /// The undo key.
            UNDO = "undo",
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &OPS,
    &context::FIELDS,
    &policy::FIELDS,
    &begin::FIELDS,
    &snapshot::FIELDS,
    &candidate::FIELDS,
    &candidate::KINDS,
    &typing::FIELDS,
    &glide_plan::FIELDS,
    &learn::FIELDS,
    &learn::KINDS,
    &learn_plan::FIELDS,
    &aim::FIELDS,
    &composing::FIELDS,
    &composing_range::FIELDS,
    &inline_begin::FIELDS,
    &inline_view::FIELDS,
    &clipboard_plan::FIELDS,
    &clipboard_span::FIELDS,
    &tally::FIELDS,
    &tally::EVENTS,
];
