//! `GlideNative`, swipe typing's scorer, answered by awnative's `glide` module, and
//! `GlideWords`, `GlideIndex`, `GlideQuery` and the records they take and give, the Kotlin over
//! it.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, HANDLE, Handle, Kind, MEMBER, METHOD, Member, Method,
    Module, OBJECTS, Out, Param, Return, Shape, Type, arg,
};

const WORDS: (&str, Param) = ("words", Param::Long);
const INDEX: (&str, Param) = ("index", Param::Long);
const QUERY: (&str, Param) = ("query", Param::Long);
const FLOATS: Param = Param::Array(Element::Float);
const INTS: Param = Param::Array(Element::Int);

/// A method of `GlideNative`.
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

/// A scoring method: the query, the candidates, their frequencies and how many to keep.
const fn score(name: &'static str) -> Method {
    method(
        name,
        &[
            QUERY,
            ("candidates", INTS),
            ("frequencies", FLOATS),
            ("max", Param::Int),
        ],
        OBJECTS,
    )
}

pub const GLIDE_NATIVE: Class = Class {
    name: "GlideNative",
    doc: "The swipe scorer's entry points in libawnative: lib/rust/crates/awnative/src/glide.rs, \
          which documents each. [GlideWords], [GlideIndex] and [GlideQuery] own its handles and \
          free them.",
    module: "glide",
    methods: &[
        method(
            "newWords",
            &[("text", Param::String), ("lengths", INTS)],
            Return::Long,
        ),
        method("freeWords", &[WORDS], Return::Unit),
        method("wordCount", &[WORDS], Return::Int),
        method("distinctChars", &[WORDS], Return::Array(Element::Char)),
        method("build", &[WORDS, ("table", INTS)], Return::Long),
        method("free", &[INDEX], Return::Unit),
        method(
            "query",
            &[
                INDEX,
                ("xs", FLOATS),
                ("ys", FLOATS),
                ("keyCodes", INTS),
                ("keyBounds", FLOATS),
            ],
            Return::Long,
        ),
        method(
            "queryTimed",
            &[
                INDEX,
                ("xs", FLOATS),
                ("ys", FLOATS),
                ("times", FLOATS),
                ("keyCodes", INTS),
                ("keyBounds", FLOATS),
            ],
            Return::Long,
        ),
        method("freeQuery", &[QUERY], Return::Unit),
        method("candidates", &[QUERY], Return::Array(Element::Int)),
        score("score"),
        score("scoreTuned"),
    ],
    ..CLASS
};

/// A top-level record of this file.
const fn record(
    name: &'static str,
    doc: &'static str,
    fields: &'static [(&'static str, &'static str)],
) -> (&'static str, Module, Type) {
    (
        "glide",
        Module::Rust,
        Type::Record {
            name,
            doc,
            data: false,
            fields,
            constants: &[],
        },
    )
}

pub const GLIDE_CHAR: (&str, Module, Type) = (
    "glide",
    Module::Rust,
    Type::Record {
        name: "GlideChar",
        doc: "What a character means on the layout being indexed. A word's path visits [pathKey] \
              for each of its characters that has one; a word is filed under the [endKey] of its \
              first and last characters, and left out if either has none. A letter repeats the \
              one before (its path loops) when their [lower] cases match.",
        data: false,
        fields: &[("lower", "Char"), ("pathKey", "Int"), ("endKey", "Int")],
        constants: &[(
            "NO_KEY",
            "",
            "-1",
            "A [GlideChar.pathKey] or [GlideChar.endKey] for a character with no key.",
        )],
    },
);

pub const GLIDE_KEYS: (&str, Module, Type) = record(
    "GlideKeys",
    "A layout's keys as drawn, in layout order: each key's code, and its visible bounds as four \
     values in [bounds]: centre x, centre y, width and height. A query refuses any other number \
     of bounds.",
    &[("codes", "IntArray"), ("bounds", "FloatArray")],
);

pub const GLIDE_SCORES: (&str, Module, Type) = record(
    "GlideScores",
    "A swipe's likeliest words, most likely first, by number in the [GlideWords], with weights.",
    &[("words", "IntArray"), ("weights", "FloatArray")],
);

pub const GLIDE_WORDS: Api = Api {
    name: "GlideWords",
    doc: "Swipe typing's words, held in Rust for the scorer. They cross as one string and their \
          lengths, as a dictionary's words one by one would take longer in JNI calls. Its native \
          memory is freed once nothing can reach it.",
    module: "glide",
    native: "GlideNative",
    shape: Shape::Handle(Handle {
        params: &[
            arg(
                "text",
                ArgKind::As("words: List<String>", "words.joinToString(\"\")"),
            ),
            arg(
                "lengths",
                ArgKind::Fixed("IntArray(words.size) { words[it].length }"),
            ),
        ],
        create: "newWords",
        free: "freeWords",
        ..HANDLE
    }),
    members: &[
        Member {
            name: "size",
            kind: Kind::Val,
            call: "wordCount",
            ..MEMBER
        },
        Member {
            name: "distinctChars",
            doc: "Every character the words use, once each, ascending.",
            ..MEMBER
        },
    ],
    ..API
};

/// The keys a query is drawn on, as its two arrays.
const KEYS: [crate::Arg; 2] = [
    arg("keyCodes", ArgKind::As("keys: GlideKeys", "keys.codes")),
    arg("keyBounds", ArgKind::Fixed("keys.bounds")),
];

pub const GLIDE_INDEX: Api = Api {
    name: "GlideIndex",
    doc: "Swipe typing's words indexed for one layout by the Rust scorer, lib/rust/crates/glide, \
          a port of FlorisBoard's statistical glide classifier. Read-only, so any number of \
          threads can query it. Its native memory is freed once nothing can reach it and no \
          query uses it.",
    module: "glide",
    native: "GlideNative",
    shape: Shape::Handle(Handle {
        free: "free",
        ..HANDLE
    }),
    members: &[
        Member {
            name: "query",
            doc: "Opens a query for the swipe through [xs] and [ys], drawn on [keys]. Close it \
                  when done.",
            args: &KEYS,
            result: Out::Wrap("GlideQuery"),
            ..MEMBER
        },
        Member {
            name: "query",
            doc: "As [query], with when each point was drawn: [times] in milliseconds, one per \
                  point, or none. [GlideQuery.scoreTuned] weighs the points the finger slowed at.",
            call: "queryTimed",
            args: &KEYS,
            result: Out::Wrap("GlideQuery"),
            ..MEMBER
        },
        Member {
            name: "build",
            doc: "Indexes [words], asking [describe] about each character they use.",
            companion: true,
            args: &[
                arg("words", ArgKind::Handle("GlideWords")),
                arg(
                    "table",
                    ArgKind::As(
                        "describe: (Char) -> GlideChar",
                        "words.distinctChars().flatMap { describe(it).run { listOf(it.code, \
                         lower.code, pathKey, endKey) } }.toIntArray()",
                    ),
                ),
            ],
            result: Out::Wrap("GlideIndex"),
            ..MEMBER
        },
    ],
    ..API
};

/// A scoring member of `GlideQuery`.
const fn scoring(name: &'static str, doc: &'static str) -> Member {
    Member {
        name,
        doc,
        result: Out::Record("GlideScores"),
        ..MEMBER
    }
}

pub const GLIDE_QUERY: Api = Api {
    name: "GlideQuery",
    doc: "One swipe against a [GlideIndex]: [candidates] gives the words worth scoring, and \
          [score] ranks them given their frequencies, which the keyboard looks up in between. \
          It keeps its index alive in Rust.",
    module: "glide",
    native: "GlideNative",
    shape: Shape::Handle(Handle {
        free: "freeQuery",
        close: true,
        synchronized: true,
        ..HANDLE
    }),
    members: &[
        Member {
            name: "candidates",
            doc: "The words worth scoring, by number, in the order to score them.",
            ..MEMBER
        },
        scoring(
            "score",
            "The [max] likeliest of [candidates], each at its frequency in [frequencies] (0 to \
             1, the same length), with weights: the lower, the likelier.",
        ),
        scoring(
            "scoreTuned",
            "As [score], as the keyboard scores: FlorisBoard's classifier, which [score] keeps \
             exactly, plus what measured better (lib/rust/crates/glide/src/tuning.rs): how near \
             the swipe's ends are to the word's, how far it strays from the word's path, and \
             where the finger slowed.",
        ),
    ],
    ..API
};
