//! `ImeSubtypesNative`, the keyboard's subtypes, answered by awnative's `ime_subtype` module, and
//! `ImeSubtypes`, the Kotlin over it.

use crate::{
    API, Api, ArgKind, CLASS, Class, HANDLE, Handle, MEMBER, METHOD, Member, Method, Param, Return,
    Shape, arg,
};

const HANDLE_PARAM: (&str, Param) = ("handle", Param::Long);

/// A method of `ImeSubtypesNative`.
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

pub const IME_SUBTYPES_NATIVE: Class = Class {
    name: "ImeSubtypesNative",
    doc: "The subtypes' entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/ime_subtype.rs. [ImeSubtypes] wraps them.",
    module: "ime_subtype",
    methods: &[
        method("create", &[], Return::Long),
        method("free", &[HANDLE_PARAM], Return::Unit),
        method(
            "call",
            &[
                HANDLE_PARAM,
                ("operation", Param::String),
                ("fields", Param::StringVararg),
            ],
            Return::Strings,
        ),
    ],
    ..CLASS
};

pub const IME_SUBTYPES: Api = Api {
    name: "ImeSubtypes",
    doc: "The keyboard's subtypes: the stored list, decoded and repaired, the active subtype, the \
          language key's next and previous, a switch by id, and what each subtype names. Android \
          writes the settings it is asked to, names locale tags and reports the currency sets it \
          loaded. Its native memory is freed once nothing can reach it.",
    module: "ime_subtype",
    native: "ImeSubtypesNative",
    shape: Shape::Handle(Handle {
        create: "create",
        free: "free",
        synchronized: true,
        ..HANDLE
    }),
    members: &[Member {
        name: "call",
        doc: "[operation]'s answer to [fields], as [ImeSubtypeOp] describes each.",
        args: &[
            arg("operation", ArgKind::Code("ImeSubtypeOp")),
            arg("fields", ArgKind::As("vararg fields: String", "fields")),
        ],
        ..MEMBER
    }],
    ..API
};
