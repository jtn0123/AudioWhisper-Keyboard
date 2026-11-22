//! `ReaderNative`, reading a page aloud, answered by awnative's `reader` module, and
//! `ReaderRules` and `SpeakerKeyState`, the Kotlin over it.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, HANDLE, Handle, Kind, MEMBER, METHOD, Member, Method,
    Out, Param, Return, Shape, Type, arg,
};

const INTS: Param = Param::Array(Element::Int);
const EDGES: (&str, Param) = ("edges", INTS);
const TEXTS: (&str, Param) = ("texts", Param::Strings);
const VISIBLE: [(&str, Param); 2] = [("visibleTop", Param::Int), ("visibleBottom", Param::Int)];
const KEY: (&str, Param) = ("key", Param::Long);
const NOW: (&str, Param) = ("now", Param::Long);

/// A method of `ReaderNative`.
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

pub const READER_NATIVE: Class = Class {
    name: "ReaderNative",
    doc: "Reading a page aloud's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/reader.rs. [ReaderRules] and [SpeakerKeyState] wrap them.",
    module: "reader",
    methods: &[
        method("maxUtterance", &[], Return::Int),
        method("recent", &[], Return::Int),
        method("holdMs", &[], Return::Long),
        method(
            "firstOnScreen",
            &[EDGES, VISIBLE[0], VISIBLE[1]],
            Return::Int,
        ),
        method(
            "allShown",
            &[EDGES, VISIBLE[0], VISIBLE[1]],
            Return::Boolean,
        ),
        method(
            "join",
            &[TEXTS, EDGES, ("scrolls", Param::Array(Element::Boolean))],
            Return::Strings,
        ),
        method(
            "find",
            &[
                TEXTS,
                EDGES,
                ("selected", Param::String),
                VISIBLE[0],
                VISIBLE[1],
            ],
            Return::NullableArray(Element::Int),
        ),
        method(
            "resumeAfter",
            &[TEXTS, ("read", Param::Strings)],
            Return::Int,
        ),
        method(
            "utterances",
            &[
                ("text", Param::String),
                ("sentenceLengths", INTS),
                ("max", Param::Int),
            ],
            Return::Strings,
        ),
        method(
            "fits",
            &[
                ("first", Param::Int),
                ("last", Param::Int),
                ("top", Param::Int),
                ("bottom", Param::Int),
            ],
            Return::Boolean,
        ),
        method(
            "offlineVoice",
            &[
                ("locales", Param::Strings),
                ("numbers", INTS),
                ("language", Param::String),
                ("country", Param::String),
                ("preferred", Param::Int),
            ],
            Return::Int,
        ),
        method("newSpeakerKey", &[], Return::Long),
        method("freeSpeakerKey", &[KEY], Return::Unit),
        method(
            "speakerKeyDown",
            &[KEY, ("reading", Param::Boolean), NOW],
            Return::Int,
        ),
        method("speakerKeyUp", &[KEY, NOW], Return::Int),
        method("speakerKeyCancel", &[KEY], Return::Unit),
    ],
    ..CLASS
};

/// A member of `ReaderRules`.
const fn rule(name: &'static str, doc: &'static str) -> Member {
    Member {
        name,
        doc,
        ..MEMBER
    }
}

const TEXT_LIST: crate::Arg = arg("texts", ArgKind::List);

pub const READER_RULES: Api = Api {
    name: "ReaderRules",
    doc: "Reading a page aloud's rules, in Rust (lib/rust/crates/reader): where reading starts, \
          joining the pieces of text a walk of the screen found into blocks, finding a selection, \
          carrying on after a scroll, cutting text for the speech engine, and keeping the text \
          read in sight. The keyboard's `reader` package wraps these and documents them.\n\n\
          Blocks' places on the screen go in as one array of edges: each block's top, then its \
          bottom, in screen pixels.",
    module: "reader",
    native: "ReaderNative",
    nested: &[
        Type::Record {
            name: "Joined",
            doc: "A block made of pieces from the one at [first] on: their [text], [top] and \
                  [bottom].",
            data: false,
            constants: &[],
            fields: &[
                ("first", "Int"),
                ("text", "String"),
                ("top", "Int"),
                ("bottom", "Int"),
            ],
        },
        Type::Record {
            name: "VoiceFacts",
            doc: "A text-to-speech voice as Android describes it: its locale's [language] and \
                  [country], whether it [needsNetwork], whether it is [installed] (no \
                  \"notInstalled\" feature), and its [quality] and [latency] (Voice.QUALITY_* \
                  and LATENCY_*).",
            data: false,
            constants: &[],
            fields: &[
                ("language", "String"),
                ("country", "String"),
                ("needsNetwork", "Boolean"),
                ("installed", "Boolean"),
                ("quality", "Int"),
                ("latency", "Int"),
            ],
        },
    ],
    members: &[
        Member {
            kind: Kind::Val,
            ..rule(
                "maxUtterance",
                "The longest piece of text the speech engine is given, in UTF-16 units.",
            )
        },
        Member {
            kind: Kind::Val,
            ..rule(
                "recent",
                "How many of the last blocks read [resumeAfter] looks back over.",
            )
        },
        rule(
            "firstOnScreen",
            "The first block whose [edges] show between [visibleTop] and [visibleBottom], or -1.",
        ),
        rule(
            "allShown",
            "Whether every block whose [edges] these are shows between [visibleTop] and \
             [visibleBottom].",
        ),
        Member {
            args: &[TEXT_LIST],
            result: Out::Records("Joined"),
            ..rule(
                "join",
                "The pieces with these [texts], [edges] and whether each [scrolls], made into \
                 blocks to read.",
            )
        },
        Member {
            args: &[TEXT_LIST],
            ..rule(
                "find",
                "Where [selected] starts among the blocks with these [texts] and [edges]: the \
                 block, then the offset in its text; null if it isn't there.",
            )
        },
        Member {
            args: &[TEXT_LIST, arg("read", ArgKind::List)],
            ..rule(
                "resumeAfter",
                "Where to carry on among the blocks with these [texts] after a scroll, given the \
                 texts [read] so far.",
            )
        },
        Member {
            result: Out::List,
            ..rule(
                "utterances",
                "[text], whose sentences are [sentenceLengths] long, cut into pieces of at most \
                 [max] UTF-16 units for the speech engine. [max] must be positive.",
            )
        },
        rule(
            "fits",
            "Whether a piece from [first] down to [last] fits in a view from [top] to [bottom].",
        ),
        Member {
            args: &[
                arg(
                    "locales",
                    ArgKind::As(
                        "voices: List<VoiceFacts>",
                        "voices.flatMap { listOf(it.language, it.country) }.toTypedArray()",
                    ),
                ),
                arg(
                    "numbers",
                    ArgKind::Fixed(
                        "voices.flatMap { listOf(if (it.needsNetwork) 1 else 0, \
                         if (it.installed) 1 else 0, it.quality, it.latency) }.toIntArray()",
                    ),
                ),
            ],
            ..rule(
                "offlineVoice",
                "Which of [voices] may read text in [language]: the [preferred] one (an index, \
                 or -1) if it works offline, else the offline one for [language] that best fits \
                 [country], quality and latency; -1 if every voice for [language] needs the \
                 network or isn't installed.",
            )
        },
    ],
    ..API
};

pub const SPEAKER_KEY_STATE: Api = Api {
    name: "SpeakerKeyState",
    doc: "The suggestion bar's speaker key between a press and its lift, in Rust: a tap reads \
          from the screen, a hold of [holdMs] or more from the top of the page, and a press while \
          reading stops it. Each press and lift says when it came, in milliseconds. Safe to use \
          from any thread. Its native memory is freed once nothing can reach it.",
    module: "reader",
    native: "ReaderNative",
    shape: Shape::Handle(Handle {
        create: "newSpeakerKey",
        free: "freeSpeakerKey",
        ..HANDLE
    }),
    nested: &[Type::Enum {
        name: "Action",
        doc: "What a press or a lift asks for.",
        entries: &["STOP", "FROM_SCREEN", "FROM_TOP"],
    }],
    members: &[
        Member {
            call: "speakerKeyDown",
            result: Out::NullableEntry("Action"),
            ..rule(
                "down",
                "The press, at [now]; [Action.STOP] if [reading], and the lift that follows then \
                 asks nothing.",
            )
        },
        Member {
            call: "speakerKeyUp",
            result: Out::NullableEntry("Action"),
            ..rule("up", "The lift, at [now].")
        },
        Member {
            call: "speakerKeyCancel",
            ..rule("cancel", "The finger slid off: the lift asks nothing.")
        },
        Member {
            kind: Kind::Val,
            companion: true,
            ..rule("holdMs", "A press this long, in milliseconds, is a hold.")
        },
    ],
    ..API
};
