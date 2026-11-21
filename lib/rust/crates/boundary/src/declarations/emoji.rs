//! `EmojiNative`, emoji data and suggestions, answered by awnative's `emoji` module, and
//! `EmojiTable` and `EmojiRules`, the Kotlin over it, with the records they read.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, HANDLE, Handle, Kind, MEMBER, METHOD, Member, Method,
    Module, Out, Param, Return, Shape, Type, arg,
};

const DATA: (&str, Param) = ("data", Param::Long);

pub const EMOJI_NATIVE: Class = Class {
    name: "EmojiNative",
    doc: "Emoji data's and suggestions' entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/emoji.rs. [EmojiTable] and [EmojiRules] wrap them.",
    module: "emoji",
    methods: &[
        Method {
            name: "parse",
            params: &[("text", Param::String), ("categoryIds", Param::Strings)],
            returns: Return::Long,
            ..METHOD
        },
        Method {
            name: "free",
            params: &[DATA],
            ..METHOD
        },
        Method {
            name: "entries",
            params: &[DATA],
            returns: Return::StringArrays,
            ..METHOD
        },
        Method {
            name: "sets",
            params: &[DATA],
            returns: Return::Array(Element::Int),
            ..METHOD
        },
        Method {
            name: "bySkinTone",
            params: &[DATA],
            returns: Return::IntArrays,
            ..METHOD
        },
        Method {
            name: "validateQuery",
            params: &[
                ("composing", Param::String),
                ("prefix", Param::String),
                ("minLength", Param::Int),
            ],
            returns: Return::NullableString,
            ..METHOD
        },
        Method {
            name: "suggest",
            params: &[
                DATA,
                ("query", Param::String),
                ("skinTone", Param::Int),
                ("max", Param::Int),
            ],
            returns: Return::Array(Element::Int),
            ..METHOD
        },
        Method {
            name: "tones",
            params: &[("value", Param::String)],
            returns: Return::Array(Element::Int),
            ..METHOD
        },
        Method {
            name: "resolveAsset",
            params: &[
                ("assets", Param::Strings),
                ("language", Param::String),
                ("country", Param::String),
                ("variant", Param::String),
            ],
            returns: Return::Int,
            ..METHOD
        },
    ],
    ..CLASS
};

/// The records emoji data is read into.
pub const EMOJI_TYPES: [(&str, Module, Type); 3] = [
    (
        "emoji",
        Module::Rust,
        Type::Record {
            name: "EmojiEntry",
            doc: "One emoji of a file [EmojiTable.parse] read: its value, its name (its set's \
                  first name, for a variant), its skin tone and hair style as ordinals of the \
                  keyboard's `EmojiSkinTone` and `EmojiHairStyle` (see [EmojiRules.tones]), and \
                  its keywords.",
            data: false,
            fields: &[
                ("value", "String"),
                ("name", "String"),
                ("skinTone", "Int"),
                ("hairStyle", "Int"),
                ("keywords", "List<String>"),
            ],
        },
    ),
    (
        "emoji",
        Module::Rust,
        Type::Record {
            name: "EmojiEntrySet",
            doc: "A set of emoji, a base and its variants: [size] entries from [start], in \
                  category [category].",
            data: false,
            fields: &[("category", "Int"), ("start", "Int"), ("size", "Int")],
        },
    ),
    (
        "emoji",
        Module::Rust,
        Type::Record {
            name: "EmojiTones",
            doc: "An emoji's skin tone and hair style, as ordinals of `EmojiSkinTone` and \
                  `EmojiHairStyle`.",
            data: true,
            fields: &[("skinTone", "Int"), ("hairStyle", "Int")],
        },
    ),
];

pub const EMOJI_TABLE: Api = Api {
    name: "EmojiTable",
    doc: "An emoji file (one of the app's assets in ime/media/emoji) parsed in Rust \
          (lib/rust/crates/emoji), which suggestions then search without its text crossing \
          again. Read-only, so any number of threads may use it; its native memory is freed once \
          nothing can reach it.",
    module: "emoji",
    native: "EmojiNative",
    shape: Shape::Handle(Handle {
        free: "free",
        ..HANDLE
    }),
    members: &[
        Member {
            name: "entries",
            doc: "Every emoji, in the file's order.",
            kind: Kind::Val,
            result: Out::Records("EmojiEntry"),
            ..MEMBER
        },
        Member {
            name: "sets",
            doc: "Every set, in the file's order.",
            kind: Kind::Val,
            result: Out::Records("EmojiEntrySet"),
            ..MEMBER
        },
        Member {
            name: "bySkinTone",
            doc: "For each skin tone, by ordinal, the entries to show with it chosen.",
            kind: Kind::Val,
            result: Out::List,
            ..MEMBER
        },
        Member {
            name: "suggest",
            doc: "The emoji for [query] among those shown for [skinTone], best first: positions \
                  in [bySkinTone]'s list for it, at most [max]. A name holding the query scores \
                  the share of it the query is, times 0.7, and a keyword holding it 0.3 (both \
                  ignoring case as Kotlin's `contains` does); ties keep the list's order, and \
                  emoji scoring 0 are left out. Throws [IllegalArgumentException] for a negative \
                  [max], as Java's `Stream.limit` does.",
            ..MEMBER
        },
        Member {
            name: "parse",
            doc: "Parses an emoji file's [text], with [categoryIds] those of `EmojiCategory`, in its \
                  order.",
            args: &[arg("categoryIds", ArgKind::List)],
            result: Out::Wrap("EmojiTable"),
            companion: true,
            ..MEMBER
        },
    ],
    ..API
};

pub const EMOJI_RULES: Api = Api {
    name: "EmojiRules",
    doc: "Emoji rules that need no parsed file.",
    module: "emoji",
    native: "EmojiNative",
    constants: &[(
        "NO_ASSET",
        "",
        "-1",
        "What [resolveAsset] gives when no file fits.",
    )],
    members: &[
        Member {
            name: "validateQuery",
            doc: "The query in the [composing] text: what follows [prefix] (`:`, or nothing), if \
                  the text starts with it, is at least [minLength] plus the prefix long, and is \
                  only ASCII letters. Null otherwise.",
            ..MEMBER
        },
        Member {
            name: "tones",
            doc: "The skin tone and hair style of the emoji [value]: the first of each enum's \
                  entries whose code point it holds, or `DEFAULT`.",
            result: Out::Record("EmojiTones"),
            ..MEMBER
        },
        Member {
            name: "resolveAsset",
            doc: "Which of the emoji files [assets] a locale reads: \
                  `{language}_{country}_{variant}.txt`, then `{language}_{country}.txt`, then \
                  `{language}.txt` (the language in lower case, and a blank country or variant \
                  left out), as its position, or [NO_ASSET].",
            args: &[arg("assets", ArgKind::List)],
            ..MEMBER
        },
    ],
    ..API
};
