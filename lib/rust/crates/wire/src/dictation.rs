//! `DictationSessionNative.call` and `.model`: their operations, answered by
//! `dictation::session::Session`, and the plan they answer with.
//!
//! A call answers with a [`plan`] row, then the effects for Kotlin to run in order: each an
//! [`effect`] name, some followed by their fields.

crate::declare! {
    /// The events `DictationSessionNative.call` takes, answered by
    /// `dictation::session::Session::call`. Each answers with a [`plan`] row and effects.
    pub const EVENTS: enum DictationEvent(str) {
        /// The mic key went down.
        PRESS = "press",
        /// The mic key came up.
        RELEASE_MIC = "releaseMic",
        /// A [`ready`] row: whether dictation can start.
        READY = "ready",
        /// The microphone opened.
        OPENED = "opened",
        /// The microphone could not open.
        OPEN_ERROR = "openError",
        /// Stop listening and type what was heard.
        STOP = "stop",
        /// Stop listening and drop what was heard.
        CANCEL = "cancel",
        /// A flag: the keyboard shows.
        VISIBLE = "visible",
        /// The session ends for good.
        RELEASE = "release",
        /// Unload the model if idle.
        FREE = "free",
        /// The unload timer fired.
        UNLOAD = "unload",
        /// The model began loading.
        WARMING = "warming",
        /// The model finished loading.
        WARMED = "warmed",
        /// The model failed.
        MODEL_ERROR = "modelError",
        /// The microphone failed while recording.
        MIC_ERROR = "micError",
        /// The input level.
        LEVEL = "level",
        /// What a preview heard.
        PREVIEW = "preview",
        /// Voice began.
        VAD_ON = "vadOn",
        /// Voice stopped.
        VAD_OFF = "vadOff",
        /// The last phrase is being transcribed.
        FINALIZING = "finalizing",
        /// A flag of whether a phrase was transcribed, then its text.
        FINAL = "final",
        /// Every phrase is transcribed.
        FINISHED = "finished",
        /// A phrase after the tidy effect.
        TIDIED = "tidied",
        /// The text before the cursor, then a flag for each name the names effect asked about.
        COMPOSE = "compose",
        /// Nothing happened: just the plan.
        SNAPSHOT = "snapshot",
    }
}

/// The state at the head of every plan.
pub mod plan {
    crate::declare! {
        /// The state at the head of every plan; effects follow.
        pub const FIELDS: fields DictationPlan {
            /// A [`DictationStatus`](super::status::STATUSES).
            STATUS,
            /// A flag: the model is loading.
            WARMING,
            /// The input level, a float.
            LEVEL,
            /// What the preview heard.
            PREVIEW,
            GENERATION,
            /// A [`DictationPhase`](super::phase::PHASES).
            PHASE,
        }
    }
}

/// Whether the session listens.
pub mod status {
    crate::declare! {
        /// Whether the session listens.
        pub const STATUSES: object DictationStatus(i32) {
            IDLE = 0,
            LISTENING = 1,
            /// Stopped listening, still transcribing.
            FINISHING = 2,
        }
    }
}

/// What the session is doing, in detail.
pub mod phase {
    crate::declare! {
        /// What the session is doing, in detail.
        pub const PHASES: object DictationPhase(str) {
            IDLE = "idle",
            CANCELLED = "cancelled",
            FLUSHING = "flushing",
            ERROR = "error",
            LISTENING = "listening",
            WARMING = "warming",
            SPEECH = "speech",
            TRAILING_SILENCE = "trailing_silence",
            FINALIZING = "finalizing",
            COMMITTED = "committed",
        }
    }
}

/// What Kotlin does, in order, after a call.
pub mod effect {
    crate::declare! {
        /// What Kotlin does, in order, after a call.
        pub const EFFECTS: object DictationEffect(str) {
            /// Check whether dictation can start, and answer with `READY`.
            CHECK = "check",
            /// A [`DictationProblem`](super::problem::PROBLEMS) follows: show it.
            PROBLEM = "problem",
            /// Open the app's settings.
            APP_SETTINGS = "appSettings",
            /// Open the model settings.
            MODEL_SETTINGS = "modelSettings",
            /// A generation follows: open the microphone for it, and answer with `OPENED` or
            /// `OPEN_ERROR`.
            LISTEN = "listen",
            /// A generation follows: load the model for it.
            PRELOAD = "preload",
            /// Drop what was heard.
            DISCARD = "discard",
            /// Stop listening and keep what was heard.
            FLUSH = "flush",
            /// A generation follows: after the queued work, answer with `FINALIZING` and
            /// `FINISHED`.
            BARRIER = "barrier",
            /// A time follows: unload the model then.
            SCHEDULE = "schedule",
            /// Cancel the unload timer.
            REMOVE_UNLOAD = "removeUnload",
            /// Unload the model.
            UNLOAD = "unload",
            /// Unload the model and stop the worker.
            CLOSE_WORKER = "closeWorker",
            /// Tick the mic key.
            TICK = "tick",
            /// A phrase follows: tidy it, and answer with `TIDIED`.
            TIDY = "tidy",
            /// A count and that many words follow: answer with `COMPOSE` and whether each is a
            /// name.
            NAMES = "names",
            /// How many characters to replace, then the text to type.
            INSERT = "insert",
            /// Undo the last phrase.
            UNDO = "undo",
            /// Answer with `COMPOSE` again.
            RESUME = "resume",
        }
    }
}

/// Why dictation stopped or could not start.
pub mod problem {
    crate::declare! {
        /// Why dictation stopped or could not start.
        pub const PROBLEMS: object DictationProblem(str) {
            FIELD_OFF = "field_off",
            ALLOW_MIC = "allow_mic",
            NOT_INSTALLED = "not_installed",
            DAMAGED = "damaged",
            MIC_BUSY = "mic_busy",
            MODEL_FAILED = "model_failed",
            MIC_STOPPED = "mic_stopped",
            /// Nothing was typed, and a phrase failed.
            FAILED = "failed",
            /// Nothing was typed, and nothing failed.
            NOT_CAUGHT = "not_caught",
        }
    }
}

/// Whether dictation can start.
pub mod ready {
    crate::declare! {
        /// Whether dictation can start.
        pub const FIELDS: fields DictationReady {
            /// A flag: the field allows dictation.
            ALLOWED,
            /// A flag: the app may use the microphone.
            PERMISSION,
            /// A [`DictationModelAvailability`](AVAILABILITY).
            MODEL,
        }
    }

    crate::declare! {
        /// Whether the speech model is installed.
        pub const AVAILABILITY: object DictationModelAvailability(str) {
            READY = "ready",
            MISSING = "missing",
            DAMAGED = "damaged",
        }
    }
}

/// The model's lifecycle.
pub mod model {
    crate::declare! {
        /// The operations `DictationSessionNative.model` takes, answered by
        /// `dictation::session::Session::model`.
        pub const OPS: enum DictationModelOp(str) {
            /// A [`DictationModelState`](super::model_state::STATES).
            GET = "get",
            /// The model loaded.
            LOADED = "loaded",
            /// The model failed to load.
            FAILED = "failed",
            /// Forget a failure so the next get loads again.
            RETRY = "retry",
            /// 1 if the model is loaded, else 0.
            IS_LOADED = "isLoaded",
            /// 1 if there was a model to close, else 0.
            CLOSE = "close",
        }
    }
}

/// Where the model stands.
pub mod model_state {
    crate::declare! {
        /// Where the model stands.
        pub const STATES: object DictationModelState(i32) {
            /// Not loaded: load it.
            NONE = 0,
            LOADED = 1,
            /// It failed to load: throw the failure.
            FAILED = 2,
        }
    }
}

/// The groups the generator writes as Kotlin.
pub const GROUPS: &[&crate::Group] = &[
    &EVENTS,
    &plan::FIELDS,
    &status::STATUSES,
    &phase::PHASES,
    &effect::EFFECTS,
    &problem::PROBLEMS,
    &ready::FIELDS,
    &ready::AVAILABILITY,
    &model::OPS,
    &model_state::STATES,
];
