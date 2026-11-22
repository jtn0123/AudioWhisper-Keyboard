//! `KeyGraphNative`, one keyboard's key graph, answered by awnative's `key_graph` module, and
//! `KeyGraph`, the Kotlin over it.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, HANDLE, Handle, MEMBER, METHOD, Member, Method, Out,
    Param, Return, Shape, arg,
};

const HANDLE_PARAM: (&str, Param) = ("handle", Param::Long);
const FIELDS: (&str, Param) = ("fields", Param::StringVararg);

/// A method of `KeyGraphNative`.
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

pub const KEY_GRAPH_NATIVE: Class = Class {
    name: "KeyGraphNative",
    doc: "The key graph's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/key_graph.rs. [KeyGraph] wraps them.",
    module: "key_graph",
    methods: &[
        method("build", &[FIELDS], Return::Long),
        method("placeholder", &[], Return::Long),
        method("free", &[HANDLE_PARAM], Return::Unit),
        method(
            "call",
            &[HANDLE_PARAM, ("operation", Param::String), FIELDS],
            Return::Strings,
        ),
        method(
            "layout",
            &[HANDLE_PARAM, ("numbers", Param::Array(Element::Float))],
            Return::Array(Element::Float),
        ),
    ],
    ..CLASS
};

pub const KEY_GRAPH: Api = Api {
    name: "KeyGraph",
    doc: "One keyboard's keys, which Rust builds from the layout files Android reads and computes \
          from the facts Android reports: each key's data, label, hint, icon, enabled state and \
          popup keys, and their bounds. Its native memory is freed once nothing can reach it.",
    module: "key_graph",
    native: "KeyGraphNative",
    shape: Shape::Handle(Handle {
        free: "free",
        synchronized: true,
        ..HANDLE
    }),
    members: &[
        Member {
            name: "build",
            doc: "The keyboard a [KeyGraphBuild] row describes.",
            companion: true,
            args: &[arg(
                "fields",
                ArgKind::As("vararg fields: String", "fields"),
            )],
            result: Out::Wrap("KeyGraph"),
            ..MEMBER
        },
        Member {
            name: "placeholder",
            doc: "The keyboard shown before any is built, never computed.",
            companion: true,
            result: Out::Wrap("KeyGraph"),
            ..MEMBER
        },
        Member {
            name: "call",
            doc: "[operation]'s answer to [fields], each as its [KeyGraphOp] describes.",
            args: &[
                arg("operation", ArgKind::Code("KeyGraphOp")),
                arg("fields", ArgKind::As("vararg fields: String", "fields")),
            ],
            ..MEMBER
        },
        Member {
            name: "layout",
            doc: "Lays the keys out for [numbers] (the keyboard's width and height, the row \
                  height, and the horizontal and vertical key margins): the desired key's \
                  visible width and height, then for each listed key its touch and visible \
                  bounds (left, top, right, bottom), 1 if enabled else 0, and its code.",
            ..MEMBER
        },
    ],
    ..API
};
