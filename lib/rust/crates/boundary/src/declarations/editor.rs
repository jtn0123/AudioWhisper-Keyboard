//! `EditorNative`, the editor's entry points, answered by awnative's `editor` module.
//! `EditorSession` and `EditorAttributes` over it stay hand-written: the session holds a
//! coroutine lock across suspending transactions, and each attribute query packs its own row.

use crate::{CLASS, Class, METHOD, Method, Param, Return};

const HANDLE_PARAM: (&str, Param) = ("handle", Param::Long);

/// A method of `EditorNative`.
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

pub const EDITOR_NATIVE: Class = Class {
    name: "EditorNative",
    doc: "The editor's entry points in libawnative: lib/rust/crates/awnative/src/editor.rs. \
          [EditorSession] and [EditorAttributes] wrap them.",
    module: "editor",
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
        method("attributes", &[("fields", Param::Strings)], Return::Strings),
    ],
    ..CLASS
};
