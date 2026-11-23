//! `GifSessionNative`: the GIF panel's operations, answered by `gif::session::Session`, the plan
//! they answer with, and the steps of sending one GIF (`gif::send`).
//!
//! A call answers with a [`plan`] row, then the effects for Kotlin to run in order: each an
//! [`effect`] name followed by its fields. A GIF crosses as a [`row`] of six fields.

crate::declare! {
    /// The operations `GifSessionNative.call` takes. Each answers with a [`plan`] row and
    /// effects.
    pub const OPS: enum GifOp(str) {
        /// A flag of whether GIFs can be fetched, then GIPHY's key, which only goes into URLs.
        SOURCE = "source",
        /// The recents file's path: read it unless it is the one already read.
        ATTACH = "attach",
        /// The recents file was read on another thread: offer Recents if it held any.
        MERGED = "merged",
        /// A flag: the media panel shows GIFs, or emoji.
        SHOW = "show",
        /// A flag of whether there is text before the cursor, the text, and a flag of whether the
        /// field is private: offer the text's last word as a search.
        OFFER = "offer",
        /// A flag of whether the keyboard is incognito, then the [`row`] of the GIF just sent.
        SENT = "sent",
        /// Open the search field.
        START_TYPING = "startTyping",
        /// Close the search field.
        STOP_TYPING = "stopTyping",
        /// Text typed into the search.
        TYPE = "type",
        /// A glide-typed word.
        TYPE_WORD = "typeWord",
        /// The delete key.
        DELETE_LAST = "deleteLast",
        /// Delete the last word.
        DELETE_WORD = "deleteWord",
        /// A delete-key swipe: a flag of whether it deletes words, how many, and a flag of
        /// whether it just began.
        SWIPE = "swipe",
        /// Search for what was typed.
        SUBMIT = "submit",
        /// A search to show, or Trending or Recents.
        LOAD = "load",
        /// A [`fetch`](effect::EFFECTS) answered: its generation, a [`GifReply`](reply::REPLIES),
        /// the HTTP status, and the body.
        FETCHED = "fetched",
        /// The keyboard went away.
        RESET = "reset",
        /// Nothing happened: just the plan.
        SNAPSHOT = "snapshot",
    }
}

/// The state at the head of every plan.
pub mod plan {
    crate::declare! {
        /// The state at the head of every plan; effects follow.
        pub const FIELDS: fields GifPlan {
            /// A flag: the media panel shows GIFs.
            SHOWING,
            /// A flag: the search field is open and typing goes into it.
            TYPING,
            /// What is in the search field.
            QUERY,
            /// The search, Trending or Recents, whose GIFs are shown.
            SHOWN,
        }
    }
}

/// What Kotlin does, in order, after a call.
pub mod effect {
    crate::declare! {
        /// What Kotlin does, in order, after a call.
        pub const EFFECTS: object GifEffect(str) {
            /// Show new results: a [`GifResults`](super::results::RESULTS), the problem to show
            /// when they failed, a count, then that many GIF [`row`](super::row::FIELDS)s.
            RESULTS = "results",
            /// Show new quick searches: a count, then that many.
            CHIPS = "chips",
            /// Fetch a URL: a generation, the URL, then a [`GifRequest`](super::request::FIELDS).
            /// Answer with `FETCHED` for that generation, unless it is cancelled first.
            FETCH = "fetch",
            /// Stop the fetch of the generation that follows; it must not answer.
            CANCEL = "cancel",
            /// Read recents: a generation of recents, then the file. Give `GifSessionNative.read`
            /// what it held on that thread, then answer with `MERGED` on the main one.
            READ = "read",
            /// Save recents: a generation of recents. On another thread, write what
            /// `GifSessionNative.rows` gives then.
            SAVE = "save",
            /// Drop the previews held in memory.
            FORGET_PREVIEWS = "forgetPreviews",
        }
    }
}

/// What the results are.
pub mod results {
    crate::declare! {
        /// What the results are.
        pub const RESULTS: object GifResults(i32) {
            LOADING = 0,
            LOADED = 1,
            FAILED = 2,
        }
    }
}

/// How a fetch answered.
pub mod reply {
    crate::declare! {
        /// How a fetch answered.
        pub const REPLIES: object GifReply(str) {
            /// The status the request asked for, and the body.
            OK = "ok",
            /// Another HTTP status; the body wasn't read.
            STATUS = "status",
            /// The body was longer than the request allowed.
            TOO_LARGE = "tooLarge",
            /// No answer: the connection failed or timed out.
            UNREACHABLE = "unreachable",
        }
    }
}

/// One GIF, as it crosses.
pub mod row {
    crate::declare! {
        /// One GIF, as it crosses.
        pub const FIELDS: fields GifRow {
            ID,
            TITLE,
            PREVIEW_URL,
            /// The preview's width over its height: a float's bits, as an integer.
            RATIO,
            SEND_URL,
            /// The GIF's page, typed instead when an app can't take GIFs.
            PAGE_URL,
        }
    }
}

/// How to make a request.
pub mod request {
    crate::declare! {
        /// How to make a request, as `GifSessionNative.request` gives it for a
        /// [`GifMedia`](super::media::MEDIA) and a fetch effect carries it.
        pub const FIELDS: fields GifRequest {
            /// The longest body to read, in bytes.
            MAX_BYTES,
            /// How long connecting, and then each read, may take, in milliseconds.
            TIMEOUT_MS,
            /// The only HTTP status whose body is read.
            OK_STATUS,
        }
    }
}

/// What a request fetches.
pub mod media {
    crate::declare! {
        /// What a request fetches.
        pub const MEDIA: object GifMedia(i32) {
            /// A page of GIPHY's results.
            API = 0,
            /// A preview for the panel.
            PREVIEW = 1,
            /// The GIF being sent.
            SEND = 2,
        }
    }
}

/// The steps of sending one GIF, answered by `gif::send::next`.
pub mod send {
    crate::declare! {
        /// What just happened while sending a GIF; each comes with a flag of whether it worked.
        pub const STEPS: enum GifSendStep(str) {
            /// Sending began; the flag says whether the app takes GIFs.
            START = "start",
            /// The sent-GIF folder was pruned; the flag says whether the GIF's file is there.
            CHECKED = "checked",
            /// The download into the part file ended.
            SAVED = "saved",
            /// The part file was renamed to the GIF's file.
            RENAMED = "renamed",
            /// The GIF was offered to the app as an image.
            COMMITTED = "committed",
            /// The GIF's link was typed.
            TYPED = "typed",
        }
    }

    crate::declare! {
        /// What to do next while sending a GIF, in order.
        pub const ACTIONS: object GifSendAction(str) {
            /// Type the GIF's page link, and answer with `TYPED`.
            TYPE_LINK = "typeLink",
            /// Make the sent-GIF folder, delete its stale files, and answer with `CHECKED`.
            PRUNE = "prune",
            /// Mark the GIF's file as used now.
            TOUCH = "touch",
            /// Download the GIF into its part file, and answer with `SAVED`.
            DOWNLOAD = "download",
            /// Rename the part file to the GIF's file, and answer with `RENAMED`.
            RENAME = "rename",
            /// Delete the part file.
            DELETE_PART = "deletePart",
            /// Offer the GIF's file to the app, and answer with `COMMITTED`.
            COMMIT = "commit",
            /// It was sent as an image.
            SENT = "sent",
            /// Its link was typed instead.
            LINKED = "linked",
            /// It could not be sent.
            FAILED = "failed",
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &OPS,
    &plan::FIELDS,
    &effect::EFFECTS,
    &results::RESULTS,
    &reply::REPLIES,
    &row::FIELDS,
    &request::FIELDS,
    &media::MEDIA,
    &send::STEPS,
    &send::ACTIONS,
];
