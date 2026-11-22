//! `SuggesterNative`, `NativeLearnerNative` and `NativeGlideNative`, what the keyboard learns from
//! each finished word and the glide ranking, answered by awnative's `suggest::word_learner` and
//! `suggest::native_glide` modules, and `Learner` and `NativeGlide`, the Kotlin over them.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, HANDLE, Handle, MEMBER, METHOD, Member, Method,
    Param, Return, Shape, arg,
};

const LEARNER: (&str, Param) = ("learner", Param::Long);
const STORE: (&str, Param) = ("store", Param::Long);

/// A method of one of these classes.
const fn method(
    name: &'static str,
    params: &'static [(&'static str, Param)],
    returns: Return,
) -> Method {
    Method {
        name,
        params,
        returns,
        ..METHOD
    }
}

pub const SUGGESTER_NATIVE: Class = Class {
    name: "SuggesterNative",
    doc: "The word learner's lifetime and bookkeeping entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/suggest/word_learner.rs. [Learner] wraps them.",
    module: "suggest::word_learner",
    file: "learner",
    methods: &[
        method("newLearner", &[], Return::Long),
        method("freeLearner", &[LEARNER], Return::Unit),
        method("learnerDropSwiped", &[LEARNER], Return::Unit),
        method(
            "learnerUndo",
            &[LEARNER, STORE, ("start", Param::Int)],
            Return::Boolean,
        ),
        method("learnerReset", &[LEARNER], Return::Unit),
    ],
    ..CLASS
};

pub const NATIVE_LEARNER_NATIVE: Class = Class {
    name: "NativeLearnerNative",
    doc: "Learning from a finished word in libawnative, answered by \
          lib/rust/crates/awnative/src/suggest/word_learner.rs. [Learner] wraps it.",
    module: "suggest::word_learner",
    file: "learner",
    methods: &[method(
        "call",
        &[
            LEARNER,
            STORE,
            ("word", Param::String),
            ("textBefore", Param::String),
            ("start", Param::Int),
            ("swiped", Param::Boolean),
            ("dictionary", Param::Long),
        ],
        Return::Boolean,
    )],
    ..CLASS
};

pub const NATIVE_GLIDE_NATIVE: Class = Class {
    name: "NativeGlideNative",
    doc: "The glide ranking in libawnative, answered by \
          lib/rust/crates/awnative/src/suggest/native_glide.rs. [NativeGlide] wraps it.",
    module: "suggest::native_glide",
    file: "learner",
    methods: &[method(
        "rank",
        &[
            ("session", Param::Long),
            ("history", Param::Long),
            ("words", Param::Strings),
            ("costs", Param::Array(Element::Float)),
            ("previous", Param::Strings),
            ("sentenceStart", Param::Boolean),
        ],
        Return::Array(Element::Int),
    )],
    ..CLASS
};

const KEPT_STORE: crate::Arg = arg("store", ArgKind::Fixed("store.handle"));

pub const LEARNER_API: Api = Api {
    name: "Learner",
    doc: "What the keyboard learns from each finished word, in Rust, feeding [store]. Its native \
          memory is freed once nothing can reach it.",
    module: "learner",
    native: "SuggesterNative",
    shape: Shape::Handle(Handle {
        keeps: &["private val store: LearnedWordStore"],
        create: "newLearner",
        free: "freeLearner",
        ..HANDLE
    }),
    members: &[
        Member {
            name: "learnNative",
            doc: "Learns [word], which starts at [start] right after [textBefore], or was swiped \
                  in if [swiped], asking [dictionary] if one is open. True if [store] changed.",
            native: "NativeLearnerNative",
            call: "call",
            args: &[KEPT_STORE, arg("dictionary", ArgKind::Lent("LatinSession"))],
            ..MEMBER
        },
        Member {
            name: "dropSwiped",
            doc: "The swiped word waiting to be kept was replaced or deleted.",
            call: "learnerDropSwiped",
            ..MEMBER
        },
        Member {
            name: "undo",
            doc: "The auto-correction that starts at [start] was undone. True if [store] changed.",
            call: "learnerUndo",
            args: &[KEPT_STORE],
            ..MEMBER
        },
        Member {
            name: "reset",
            doc: "Another field: forgets where words were.",
            call: "learnerReset",
            ..MEMBER
        },
    ],
    ..API
};

pub const NATIVE_GLIDE: Api = Api {
    name: "NativeGlide",
    doc: "One native context and rank exchange; the session's lock keeps `close()` out of it.",
    module: "learner",
    native: "NativeGlideNative",
    members: &[Member {
        name: "rank",
        doc: "The indexes of [words], whose glide [costs] these are, best first after \
              [previous].",
        args: &[
            arg("session", ArgKind::Lent("LatinSession")),
            arg("history", ArgKind::NullableHandle("LearnedWordStore")),
            arg("words", ArgKind::List),
            arg("previous", ArgKind::List),
        ],
        ..MEMBER
    }],
    ..API
};
