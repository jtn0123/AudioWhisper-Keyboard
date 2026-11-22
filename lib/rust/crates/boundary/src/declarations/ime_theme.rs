//! `ImeThemeNative`, the keyboard's theme, answered by awnative's `ime_theme` module, and
//! `ImeTheme`, the Kotlin over it.

use crate::{
    API, Api, ArgKind, CLASS, Class, HANDLE, Handle, MEMBER, METHOD, Member, Method, Param, Return,
    Shape, arg,
};

const HANDLE_PARAM: (&str, Param) = ("handle", Param::Long);
const FIELDS: (&str, Param) = ("fields", Param::StringVararg);

/// A method of `ImeThemeNative`.
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

pub const IME_THEME_NATIVE: Class = Class {
    name: "ImeThemeNative",
    doc: "The theme's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/ime_theme.rs. [ImeTheme] wraps them.",
    module: "ime_theme",
    methods: &[
        method("create", &[], Return::Long),
        method("free", &[HANDLE_PARAM], Return::Unit),
        method(
            "call",
            &[HANDLE_PARAM, ("operation", Param::String), FIELDS],
            Return::Strings,
        ),
        method("rules", &[("rule", Param::String), FIELDS], Return::Strings),
    ],
    ..CLASS
};

pub const IME_THEME: Api = Api {
    name: "ImeTheme",
    doc: "The keyboard's theme: which theme the settings and the system's night mode make \
          active, what Android loads to show it, and which loaded theme shows. Android reads \
          the archives and stylesheets it names. Its native memory is freed once nothing can \
          reach it.",
    module: "ime_theme",
    native: "ImeThemeNative",
    shape: Shape::Handle(Handle {
        create: "create",
        free: "free",
        synchronized: true,
        ..HANDLE
    }),
    members: &[
        Member {
            name: "call",
            doc: "[operation]'s answer to [fields]: an [ImeThemeStep] row, as [ImeThemeOp] \
                  describes each.",
            args: &[
                arg("operation", ArgKind::Code("ImeThemeOp")),
                arg("fields", ArgKind::As("vararg fields: String", "fields")),
            ],
            ..MEMBER
        },
        Member {
            name: "rules",
            doc: "[rule]'s answer to [fields], as [ImeThemeRule] describes each.",
            companion: true,
            args: &[
                arg("rule", ArgKind::Code("ImeThemeRule")),
                arg("fields", ArgKind::As("vararg fields: String", "fields")),
            ],
            ..MEMBER
        },
    ],
    ..API
};
