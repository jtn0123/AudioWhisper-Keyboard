//! `ReadingNative`, a reading's steps and which window it reads, answered by awnative's `reading`
//! module and the reader crate's `reading` and `window`, and `ReadingPlan` and `ReadingRules`, the
//! Kotlin over it.

use crate::{
    API, Api, CLASS, Class, Element, HANDLE, Handle, MEMBER, METHOD, Member, Method, Param, Return,
    Shape,
};

const HANDLE_PARAM: (&str, Param) = ("handle", Param::Long);
const FLAGS: Param = Param::Array(Element::Boolean);
const INTS: Param = Param::Array(Element::Int);
const EFFECT: Return = Return::Array(Element::Int);

/// A method of `ReadingNative`.
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

/// A member with the method's own name and signature.
const fn member(name: &'static str, doc: &'static str) -> Member {
    Member {
        name,
        doc,
        ..MEMBER
    }
}

pub const READING_NATIVE: Class = Class {
    name: "ReadingNative",
    doc: "A reading's steps and which window it reads, in libawnative: \
          lib/rust/crates/awnative/src/reading.rs and the reader crate. [ReadingPlan] and \
          [ReadingRules] wrap them.",
    module: "reading",
    methods: &[
        method("create", &[], Return::Long),
        method("free", &[HANDLE_PARAM], Return::Unit),
        method(
            "start",
            &[
                HANDLE_PARAM,
                ("from", Param::Int),
                ("selection", Param::NullableString),
            ],
            EFFECT,
        ),
        method("done", &[HANDLE_PARAM], EFFECT),
        method("answer", &[HANDLE_PARAM, ("value", Param::Int)], EFFECT),
        method(
            "looked",
            &[
                HANDLE_PARAM,
                ("app", Param::NullableString),
                ("texts", Param::Strings),
                ("edges", INTS),
                ("scrolls", FLAGS),
                ("facts", INTS),
            ],
            EFFECT,
        ),
        method("lookedNothing", &[HANDLE_PARAM], EFFECT),
        method(
            "window",
            &[
                ("application", FLAGS),
                ("inputMethod", FLAGS),
                ("active", FLAGS),
                ("focused", FLAGS),
                ("layers", INTS),
            ],
            Return::Array(Element::Int),
        ),
        method(
            "visibleBottom",
            &[
                ("top", Param::Int),
                ("bottom", Param::Int),
                ("keyboardTop", Param::Int),
            ],
            Return::Int,
        ),
    ],
    ..CLASS
};

pub const READING_PLAN: Api = Api {
    name: "ReadingPlan",
    doc: "A reading of the page aloud, step by step, in Rust (reader::reading). Each call \
          answers the next effect for Kotlin to run: a `ReadingEffect` code, then two \
          arguments. Owns no Android resources; closing it frees it, and a call after that \
          throws.",
    module: "reading",
    native: "ReadingNative",
    shape: Shape::Handle(Handle {
        create: "create",
        free: "free",
        close: true,
        synchronized: true,
        ..HANDLE
    }),
    members: &[
        member(
            "start",
            "Starts reading [from] a `ReadingFrom`, with the [selection] for `SELECTION`.",
        ),
        member("done", "The effect that takes no answer is done."),
        member(
            "answer",
            "The effect that asked a question answered [value], as `ReadingEffect` says.",
        ),
        member(
            "looked",
            "The walk found blocks of these [texts], [edges] (each top, then its bottom) and \
             [scrolls] flags in [app], with these [facts], a `ReadingFact` row.",
        ),
        member("lookedNothing", "The walk found no window."),
    ],
    ..API
};

pub const READING_RULES: Api = Api {
    name: "ReadingRules",
    doc: "Which window a reading reads, in Rust (reader::window).",
    module: "reading",
    native: "ReadingNative",
    members: &[
        member(
            "window",
            "Of windows that are an [application]'s or an [inputMethod]'s, [active], \
             [focused] and on these [layers]: the app's (the active one, else the focused one, \
             else the first on top), then the keyboard's (the first input method); -1 for none.",
        ),
        member(
            "visibleBottom",
            "The bottom of the part of the screen an app between [top] and [bottom] shows, cut \
             to the keyboard at [keyboardTop] if that starts inside it.",
        ),
    ],
    ..API
};
