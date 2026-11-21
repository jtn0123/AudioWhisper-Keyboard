//! `ComposingNative`, FlorisBoard's text composers, answered by awnative's `composing` module, and
//! `ComposingRules`, the Kotlin over it.

use crate::{
    API, Api, ArgKind, CLASS, Class, HANDLE, Handle, MEMBER, METHOD, Member, Method, Out, Param,
    Return, Shape, arg,
};

pub const COMPOSING_NATIVE: Class = Class {
    name: "ComposingNative",
    doc: "The text composers' entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/composing.rs. [ComposingRules] wraps them.",
    module: "composing",
    methods: &[
        Method {
            name: "newRules",
            params: &[("keys", Param::Strings), ("values", Param::Strings)],
            returns: Return::Long,
            ..METHOD
        },
        Method {
            name: "freeRules",
            params: &[("rules", Param::Long)],
            ..METHOD
        },
        Method {
            name: "rules",
            params: &[
                ("rules", Param::Long),
                ("precedingText", Param::String),
                ("toInsert", Param::String),
            ],
            returns: Return::Strings,
            ..METHOD
        },
    ],
    ..CLASS
};

pub const COMPOSING_RULES: Api = Api {
    name: "ComposingRules",
    doc: "A table of rules, each an ending in [keys] and what it becomes in [values] (such as \
          Telex for Vietnamese), held in Rust and tried in the order given: the first whose \
          ending the text, with the keystroke, ends with when lower-cased replaces it, \
          upper-cased if the ending began upper-case. Composing.kt makes one from pairs. Safe to \
          share between threads, as it never changes. Its native memory is freed once nothing \
          can reach it.",
    module: "composing",
    native: "ComposingNative",
    shape: Shape::Handle(Handle {
        params: &[arg("keys", ArgKind::List), arg("values", ArgKind::List)],
        create: "newRules",
        free: "freeRules",
        ..HANDLE
    }),
    members: &[Member {
        name: "actions",
        doc: "What typing [toInsert] after [precedingText] does: how many characters before the \
              cursor to delete and what to type instead, `0 to toInsert` when nothing.",
        call: "rules",
        result: Out::Pair("Int", "String"),
        ..MEMBER
    }],
    ..API
};
