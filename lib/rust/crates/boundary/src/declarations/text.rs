//! `JavaTextNative`, Kotlin's character rules as the `chars` crate has them, answered by
//! awnative's `text` module and the crate, and `JavaText`, the Kotlin over it.

use crate::{
    API, Api, ArgKind, BIT, Bit, BitKind, CLASS, Class, Element, MEMBER, METHOD, Member, Method,
    Out, Param, Return, Type, arg,
};

const TEXT: (&str, Param) = ("text", Param::String);

/// A method of `JavaTextNative`, answered by `rust` if it isn't empty.
const fn method(
    name: &'static str,
    params: &'static [(&'static str, Param)],
    returns: Return,
    rust: &'static str,
) -> Method {
    Method {
        name,
        params,
        returns,
        rust,
        ..METHOD
    }
}

pub const JAVA_TEXT_NATIVE: Class = Class {
    name: "JavaTextNative",
    doc: "The `chars` crate's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/text.rs and the crate. [JavaText] wraps them.",
    module: "text",
    methods: &[
        method("unitFacts", &[("unit", Param::Char)], Return::Long, ""),
        method(
            "codePointFacts",
            &[("codePoint", Param::Int)],
            Return::Array(Element::Int),
            "",
        ),
        method("lowercase", &[TEXT], Return::String, "::chars::lowercase"),
        method("uppercase", &[TEXT], Return::String, "::chars::uppercase"),
        method("trim", &[TEXT], Return::String, ""),
        method(
            "equalsIgnoreCase",
            &[("a", Param::String), ("b", Param::String)],
            Return::Boolean,
            "::chars::equals_ignore_case",
        ),
        method(
            "startsWithIgnoreCase",
            &[TEXT, ("prefix", Param::String)],
            Return::Boolean,
            "::chars::starts_with_ignore_case",
        ),
        method("isBlank", &[TEXT], Return::Boolean, "::chars::is_blank"),
        method("codePointCount", &[TEXT], Return::Int, ""),
        method("toIntOrNull", &[TEXT], Return::Long, ""),
    ],
    ..CLASS
};

/// A predicate of `JavaText.Unit`, at `shift`.
const fn flag(name: &'static str, shift: u32) -> Bit {
    Bit { name, shift, ..BIT }
}

/// A function of `JavaText` that passes its arguments as they are.
const fn same(name: &'static str) -> Member {
    Member { name, ..MEMBER }
}

pub const JAVA_TEXT: Api = Api {
    name: "JavaText",
    doc: "Kotlin's character and string rules as the Rust (lib/rust/crates/chars) has them, for \
          tests that hold them up against `java.lang.Character` and `String`: on a computer, in \
          lib/rust's JVM tests, and on a phone, whose Java answers some characters differently. \
          The keyboard itself never calls these; its Rust uses the crate directly.",
    module: "text",
    native: "JavaTextNative",
    nested: &[
        Type::Bits {
            name: "Unit",
            doc: "What Rust says of one `Char`: its predicates, its single-char cases and its \
                  NFD.",
            long: true,
            internal: true,
            fields: &[
                flag("isLetter", 0),
                flag("isDigit", 1),
                flag("isLetterOrDigit", 2),
                flag("isUpperCase", 3),
                flag("isLowerCase", 4),
                flag("isTitleCase", 5),
                flag("isWhitespace", 6),
                Bit {
                    doc: "Whether `\\s` in a `Regex` matches it.",
                    ..flag("isRegexSpace", 7)
                },
                Bit {
                    kind: BitKind::Char,
                    ..flag("uppercase", 8)
                },
                Bit {
                    kind: BitKind::Char,
                    ..flag("lowercase", 24)
                },
                Bit {
                    kind: BitKind::Int(21),
                    ..flag("decomposedFirst", 40)
                },
            ],
        },
        Type::Record {
            name: "CodePoint",
            doc: "What Rust says of one code point: `Character`'s functions that take an `Int`.",
            data: false,
            fields: &[
                ("isDigit", "Boolean"),
                ("digit", "Int"),
                ("upperCase", "Int"),
                ("lowerCase", "Int"),
            ],
        },
    ],
    members: &[
        Member {
            call: "unitFacts",
            args: &[arg("unit", ArgKind::As("char: Char", "char"))],
            result: Out::Wrap("Unit"),
            ..same("unit")
        },
        Member {
            call: "codePointFacts",
            result: Out::Record("CodePoint"),
            ..same("codePoint")
        },
        same("lowercase"),
        same("uppercase"),
        same("trim"),
        same("equalsIgnoreCase"),
        same("startsWithIgnoreCase"),
        same("isBlank"),
        same("codePointCount"),
        Member {
            result: Out::Then("Int?", ".takeIf { it != Long.MIN_VALUE }?.toInt()"),
            ..same("toIntOrNull")
        },
    ],
    ..API
};
