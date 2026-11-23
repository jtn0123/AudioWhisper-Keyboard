//! `GifNative`, the GIF panel's rules, answered by awnative's `gif` module and the gif crate,
//! and `GifRules`, the Kotlin over it.

use crate::{
    API, Api, ArgKind, CLASS, Class, Kind, MEMBER, METHOD, Member, Method, Out, Param, Return,
    Type, arg, default,
};

/// A method of `GifNative` that takes nothing.
const fn getter(name: &'static str, returns: Return) -> Method {
    Method {
        name,
        returns,
        ..METHOD
    }
}

/// A method of `GifNative` that edits a search.
const fn edit(name: &'static str, params: &'static [(&'static str, Param)]) -> Method {
    Method {
        name,
        params,
        returns: Return::String,
        ..METHOD
    }
}

pub const GIF_NATIVE: Class = Class {
    name: "GifNative",
    doc: "The GIF panel's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/gif.rs and the gif crate. [GifRules] wraps them.",
    module: "gif",
    methods: &[
        getter("maxLength", Return::Int),
        getter("categories", Return::Strings),
        getter("recents", Return::String),
        Method {
            rust: "::gif::search_text::type_text",
            ..edit("type", &[("query", Param::String), ("text", Param::String)])
        },
        Method {
            rust: "::gif::search_text::type_word",
            ..edit(
                "typeWord",
                &[("query", Param::String), ("word", Param::String)],
            )
        },
        edit(
            "deleteLast",
            &[("query", Param::String), ("count", Param::Int)],
        ),
        edit(
            "deleteWords",
            &[("query", Param::String), ("count", Param::Int)],
        ),
        Method {
            name: "lastWord",
            params: &[("text", Param::NullableString)],
            returns: Return::NullableString,
            ..METHOD
        },
        Method {
            name: "chips",
            params: &[
                ("typedWord", Param::NullableString),
                ("hasRecents", Param::Boolean),
            ],
            returns: Return::Strings,
            rust: "::gif::chips::chips",
            ..METHOD
        },
        Method {
            name: "route",
            params: &[("code", Param::Int), ("typing", Param::Boolean)],
            returns: Return::Int,
            ..METHOD
        },
        Method {
            name: "swipeDelete",
            params: &[("swipe", Param::Int), ("unitsX", Param::Int)],
            returns: Return::Int,
            ..METHOD
        },
        Method {
            name: "problem",
            params: &[("failure", Param::Int), ("status", Param::Int)],
            returns: Return::String,
            ..METHOD
        },
    ],
    ..CLASS
};

pub const GIF_RULES: Api = Api {
    name: "GifRules",
    doc: "The GIF panel's rules, in Rust (lib/rust/crates/gif): editing the search field, the \
          chips under it, which keys reach it, and what a failed GIPHY call tells the user. \
          The panel's session (`GifSessionPlan`) runs them; the keyboard reads the chips, \
          Trending and Recents and the key routes here, and its tests hold each one to the \
          Kotlin it replaced.",
    module: "gif",
    native: "GifNative",
    constants: &[(
        "trending",
        "String",
        "GifNative.categories()[0]",
        "GIPHY's trending list's chip.",
    )],
    nested: &[
        Type::Enum {
            name: "Route",
            doc: "Where a key goes while the user types into the search field.",
            entries: &["SEARCH", "KEYBOARD", "IGNORE"],
        },
        Type::Enum {
            name: "Swipe",
            doc: "The delete key's swipes the search field acts on.",
            entries: &["DELETE_CHARACTERS", "DELETE_WORDS", "SELECT"],
        },
        Type::Enum {
            name: "Failure",
            doc: "How a GIPHY call failed: an answer that isn't the JSON expected, an HTTP \
                  status, or none.",
            entries: &["UNREADABLE", "STATUS", "UNREACHABLE"],
        },
    ],
    members: &[
        Member {
            name: "maxLength",
            doc: "The longest search, in UTF-16 units: GIPHY's limit.",
            kind: Kind::Val,
            ..MEMBER
        },
        Member {
            name: "categories",
            doc: "The quick searches, [trending] first.",
            kind: Kind::Val,
            result: Out::List,
            ..MEMBER
        },
        Member {
            name: "recents",
            doc: "The chip for the GIFs sent before.",
            kind: Kind::Val,
            ..MEMBER
        },
        Member {
            name: "type",
            doc: "[query] with [text] typed after it.",
            ..MEMBER
        },
        Member {
            name: "typeWord",
            doc: "[query] with a glide-typed [word] after it.",
            ..MEMBER
        },
        Member {
            name: "deleteLast",
            doc: "[query] without its last [count] characters.",
            ..MEMBER
        },
        Member {
            name: "deleteWords",
            doc: "[query] without its last [count] words.",
            ..MEMBER
        },
        Member {
            name: "lastWord",
            doc: "The word just typed at the end of [text], or null.",
            args: &[arg("text", ArgKind::Text)],
            ..MEMBER
        },
        Member {
            name: "chips",
            doc: "The chips to offer after [typedWord].",
            result: Out::List,
            ..MEMBER
        },
        Member {
            name: "route",
            doc: "Where the key with [code] goes; [typing] says whether it is a character or \
                  numeric key.",
            result: Out::Entry("Route"),
            ..MEMBER
        },
        Member {
            name: "swipeDelete",
            doc: "How many characters, or words, a delete-key [swipe] [unitsX] key widths along \
                  reaches back.",
            args: &[arg("swipe", ArgKind::Ordinal("Swipe"))],
            ..MEMBER
        },
        Member {
            name: "problem",
            doc: "What a [failure], with this HTTP [status] if it had one, tells the user.",
            args: &[
                arg("failure", ArgKind::Ordinal("Failure")),
                default("status", "0"),
            ],
            ..MEMBER
        },
    ],
    ..API
};
