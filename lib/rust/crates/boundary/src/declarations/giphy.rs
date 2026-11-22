//! `GiphyNative`, GIPHY's URLs and answers, answered by awnative's `giphy` module and the gif
//! crate, and `GiphyRules`, the Kotlin over it.

use crate::{API, Api, ArgKind, CLASS, Class, MEMBER, METHOD, Member, Method, Param, Return, arg};

pub const GIPHY_NATIVE: Class = Class {
    name: "GiphyNative",
    doc: "GIPHY's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/giphy.rs and the gif crate. [GiphyRules] wraps them.",
    module: "giphy",
    methods: &[
        Method {
            name: "parse",
            params: &[("body", Param::String)],
            returns: Return::Strings,
            ..METHOD
        },
        Method {
            name: "url",
            params: &[
                ("apiKey", Param::String),
                ("endpoint", Param::String),
                ("names", Param::Strings),
                ("values", Param::Strings),
            ],
            returns: Return::String,
            ..METHOD
        },
        Method {
            name: "trendingUrl",
            params: &[("apiKey", Param::String)],
            returns: Return::String,
            rust: "::gif::giphy::trending_url",
            ..METHOD
        },
        Method {
            name: "searchUrl",
            params: &[("apiKey", Param::String), ("query", Param::String)],
            returns: Return::String,
            rust: "::gif::giphy::search_url",
            ..METHOD
        },
    ],
    ..CLASS
};

pub const GIPHY_RULES: Api = Api {
    name: "GiphyRules",
    doc: "GIPHY's API apart from the call itself, in Rust (lib/rust/crates/gif/src/giphy.rs): the \
          URLs to ask it, and its answer read into GIFs exactly as the keyboard's \
          kotlinx.serialization classes read it (`parse`, in Giphy.kt). The keyboard's `Giphy` \
          wraps these and documents them.",
    module: "giphy",
    native: "GiphyNative",
    members: &[
        Member {
            name: "url",
            doc: "The URL that asks [endpoint] with [params], then [apiKey] and GIPHY's settings.",
            args: &[
                arg(
                    "names",
                    ArgKind::As("params: Map<String, String>", "params.keys.toTypedArray()"),
                ),
                arg("values", ArgKind::Fixed("params.values.toTypedArray()")),
            ],
            ..MEMBER
        },
        Member {
            name: "trendingUrl",
            doc: "The URL that asks for trending GIFs.",
            ..MEMBER
        },
        Member {
            name: "searchUrl",
            doc: "The URL that searches for [query], cut to GIPHY's longest search.",
            ..MEMBER
        },
    ],
    ..API
};
