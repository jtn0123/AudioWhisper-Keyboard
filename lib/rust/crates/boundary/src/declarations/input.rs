//! `InputRules` and `InputNative`, the key lifecycle and the rules around it, answered by
//! awnative's `input` module, and `InputDispatch`, the Kotlin over the lifecycle.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, HANDLE, Handle, MEMBER, METHOD, Member, Method,
    Param, Return, Shape, arg,
};

const LONGS: Param = Param::Array(Element::Long);
const FIELDS: (&str, Param) = ("fields", LONGS);
const HANDLE_PARAM: (&str, Param) = ("handle", Param::Long);

/// A method of one of these classes.
const fn method(
    name: &'static str,
    doc: &'static str,
    params: &'static [(&'static str, Param)],
    returns: Return,
) -> Method {
    Method {
        name,
        doc,
        params,
        returns,
        ..METHOD
    }
}

pub const INPUT_RULES: Class = Class {
    name: "InputRules",
    doc: "The rules around a key press, in Rust (lib/rust/crates/input), each taking and giving \
          a packet of numbers that crates/input documents. Answered by \
          lib/rust/crates/awnative/src/input.rs.",
    module: "input",
    public: true,
    methods: &[
        Method {
            defaults: &[("text", "\"\"")],
            ..method(
                "control",
                "A state, subtype or space decision around an Android callback, given its [fields] \
                 and the editor's [text].",
                &[FIELDS, ("text", Param::String)],
                Return::Array(Element::Long),
            )
        },
        method(
            "route",
            "Where a key's event goes.",
            &[FIELDS],
            Return::Array(Element::Long),
        ),
        method(
            "feedback",
            "The sound and haptic feedback for a key.",
            &[FIELDS],
            Return::Array(Element::Long),
        ),
        method(
            "visual",
            "A key's label (3 fields) or icon (5 fields).",
            &[FIELDS],
            Return::Int,
        ),
        method(
            "hold",
            "What holding the key [code] does.",
            &[
                ("code", Param::Int),
                ("popup", Param::Boolean),
                ("uninterrupted", Param::Boolean),
                ("spaceAction", Param::Boolean),
            ],
            Return::Int,
        ),
        method(
            "pointer",
            "A pointer rule: whether a key lets go when another finger goes down (its code), \
             whether a held key rolls (its code and whether it is still), or whether a pointer \
             slid out of a key (x, y and the key's left, top, right and bottom).",
            &[("fields", Param::Array(Element::Float))],
            Return::Boolean,
        ),
    ],
    ..CLASS
};

pub const INPUT_NATIVE: Class = Class {
    name: "InputNative",
    doc: "The key lifecycle's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/input.rs. [InputDispatch] wraps them.",
    module: "input",
    methods: &[
        method("create", "", &[], Return::Long),
        method("free", "", &[HANDLE_PARAM], Return::Unit),
        method(
            "call",
            "",
            &[HANDLE_PARAM, ("operation", Param::Int), FIELDS],
            Return::Array(Element::Long),
        ),
    ],
    ..CLASS
};

pub const INPUT_DISPATCH: Api = Api {
    name: "InputDispatch",
    doc: "A shared key lifecycle. Native calls finish before any Android work or receiver \
          callback.",
    module: "input",
    native: "InputNative",
    shape: Shape::Handle(Handle {
        create: "create",
        free: "free",
        close: true,
        synchronized: true,
        ..HANDLE
    }),
    members: &[Member {
        name: "call",
        doc: "The lifecycle's answer to [operation] with [fields], as crates/input's \
              lifecycle documents them.",
        args: &[arg("fields", ArgKind::As("vararg fields: Long", "fields"))],
        ..MEMBER
    }],
    ..API
};
