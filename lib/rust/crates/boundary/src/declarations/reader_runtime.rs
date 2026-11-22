//! `ReaderWalk` and `PlaybackNative`, the reader's screen walk and play queue, answered by
//! awnative's `reader_runtime` module, and `ReaderTree`, `ReaderTiming` and `PlaybackQueue`, the
//! Kotlin around them.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, HANDLE, Handle, Kind, MEMBER, METHOD, Member, Method,
    Module, Out, Param, Return, Shape, Type, arg, default,
};

const HANDLE_PARAM: (&str, Param) = ("handle", Param::Long);
const OP: (&str, Param) = ("op", Param::Int);
const ID: (&str, Param) = ("id", Param::Int);
const LONGS: Return = Return::Array(Element::Long);

/// A method of one of these classes.
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

/// What the walk asks of the screen, as a top-level interface the app's accessibility tree and
/// the tests implement.
pub const READER_TREE: (&str, Module, Type) = (
    "reader_runtime",
    Module::Rust,
    Type::Interface {
        name: "ReaderTree",
        doc: "Only platform facts. Rust requests children in reading order and decides which text \
              to read.",
        members: &[
            (
                "fun facts(id: Int): IntArray",
                "Node [id]'s bounds, flags, grid and child count, 13 numbers in the order \
                 lib/rust/crates/awnative/src/reader_runtime.rs reads them.",
            ),
            ("fun className(id: Int): String", ""),
            ("fun text(id: Int): String", ""),
            (
                "fun child(id: Int, index: Int): Int",
                "Node [id]'s child at [index], or a negative number if there is none.",
            ),
        ],
    },
);

pub const READER_WALK: Class = Class {
    name: "ReaderWalk",
    doc: "The reader's walk over the screen and its timing, answered by \
          lib/rust/crates/awnative/src/reader_runtime.rs. [ReaderTiming] reads the timing.",
    module: "reader_runtime",
    public: true,
    methods: &[
        Method {
            doc: "What to read of [tree] within the [visible] bounds (left, top, right, bottom), \
                  as the reader crate's walk packet.",
            ..method(
                "walk",
                &[
                    ("tree", Param::Object("ReaderTree")),
                    ("visible", Param::Array(Element::Int)),
                ],
                Return::Array(Element::Int),
            )
        },
        Method {
            doc: "The most page turns, the wait for each in ms, the play queue's lookahead and \
                  the follow-along delay in ms.",
            ..method("followConfig", &[], LONGS)
        },
    ],
    ..CLASS
};

pub const PLAYBACK_NATIVE: Class = Class {
    name: "PlaybackNative",
    doc: "The play queue's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/reader_runtime.rs. [PlaybackQueue] wraps them.",
    module: "reader_runtime",
    methods: &[
        method("create", &[], Return::Long),
        method(
            "call",
            &[HANDLE_PARAM, OP, ID, ("frame", Param::Long)],
            LONGS,
        ),
        method(
            "system",
            &[HANDLE_PARAM, OP, ID, ("text", Param::String)],
            LONGS,
        ),
        method("free", &[HANDLE_PARAM], Return::Unit),
    ],
    ..CLASS
};

/// A value of `ReaderTiming`: `followConfig`'s number `then` picks.
const fn timing(name: &'static str, ty: &'static str, then: &'static str) -> Member {
    Member {
        name,
        kind: Kind::Val,
        call: "followConfig",
        result: Out::Then(ty, then),
        ..MEMBER
    }
}

pub const READER_TIMING: Api = Api {
    name: "ReaderTiming",
    doc: "Reader policy constants; Android only schedules the requested waits and queue \
          capacity.",
    module: "reader_runtime",
    native: "ReaderWalk",
    members: &[
        timing("maxTurns", "Int", "[0].toInt()"),
        timing("turnMs", "Long", "[1]"),
        timing("lookahead", "Int", "[2].toInt()"),
        timing("followMs", "Long", "[3]"),
    ],
    ..API
};

pub const PLAYBACK_QUEUE: Api = Api {
    name: "PlaybackQueue",
    doc: "The reader's play queue, in Rust (lib/rust/crates/reader's play_queue.rs). Calls are \
          synchronized. Closing it frees the queue; a call after that throws.",
    module: "reader_runtime",
    native: "PlaybackNative",
    shape: Shape::Handle(Handle {
        create: "create",
        free: "free",
        close: true,
        synchronized: true,
        ..HANDLE
    }),
    members: &[
        Member {
            name: "call",
            doc: "The queue's answer to [op] for utterance [id] at [frame].",
            args: &[
                arg("op", ArgKind::Code("PlaybackOp")),
                default("id", "0"),
                default("frame", "0"),
            ],
            ..MEMBER
        },
        Member {
            name: "system",
            doc: "The queue's answer to the system voice's [op] for utterance [id], whose \
                  system utterance id is [text].",
            args: &[
                arg("op", ArgKind::Code("PlaybackSpeechOp")),
                default("id", "0"),
                default("text", "\"\""),
            ],
            ..MEMBER
        },
    ],
    ..API
};
