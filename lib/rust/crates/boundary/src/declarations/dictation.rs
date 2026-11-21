//! `DictationNative`, dictation's text rules and mic key, answered by awnative's `dictation`
//! module, and `DictationRules` and `MicKeyState`, the Kotlin over it.

use crate::{
    API, Api, ArgKind, CLASS, Class, HANDLE, Handle, Kind, MEMBER, METHOD, Member, Method, Out,
    Param, Return, Shape, Type, arg,
};

pub const DICTATION_NATIVE: Class = Class {
    name: "DictationNative",
    doc: "Dictation's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/dictation.rs. [DictationRules] reads their rows, and \
          [MicKeyState] wraps the mic key's.",
    module: "dictation",
    methods: &[
        Method {
            name: "parse",
            params: &[("phrase", Param::String)],
            returns: Return::Strings,
            ..METHOD
        },
        Method {
            name: "punctuate",
            params: &[
                ("before", Param::String),
                ("lastPhrase", Param::NullableString),
                ("mark", Param::Char),
            ],
            returns: Return::Strings,
            ..METHOD
        },
        Method {
            name: "revertsAt",
            params: &[
                ("typed", Param::String),
                ("start", Param::Int),
                ("cursor", Param::Int),
                ("textBeforeCursor", Param::String),
            ],
            returns: Return::Boolean,
            rust: "::dictation::reverts_at",
            ..METHOD
        },
        Method {
            name: "format",
            params: &[
                ("phrase", Param::String),
                ("vocabulary", Param::String),
                ("removeFillers", Param::Boolean),
            ],
            returns: Return::String,
            ..METHOD
        },
        Method {
            name: "holdMs",
            returns: Return::Long,
            ..METHOD
        },
        Method {
            name: "newMicKey",
            returns: Return::Long,
            ..METHOD
        },
        Method {
            name: "freeMicKey",
            params: &[("key", Param::Long)],
            ..METHOD
        },
        Method {
            name: "micKeyPress",
            params: &[
                ("key", Param::Long),
                ("status", Param::Int),
                ("now", Param::Long),
            ],
            returns: Return::Int,
            ..METHOD
        },
        Method {
            name: "micKeyRelease",
            params: &[("key", Param::Long), ("now", Param::Long)],
            returns: Return::Int,
            ..METHOD
        },
    ],
    ..CLASS
};

pub const DICTATION_RULES: Api = Api {
    name: "DictationRules",
    doc: "Dictation's text rules, in Rust (lib/rust/crates/dictation): finding spoken commands in \
          a transcribed phrase, joining phrases that were transcribed separately, and whether a \
          backspace can still take back the phrase typed last. The keyboard's `Spoken`, \
          `Phrases` and `DictationUndo` call these; their documentation explains the rules.",
    module: "dictation",
    native: "DictationNative",
    nested: &[
        Type::Enum {
            name: "Kind",
            doc: "What a part of a phrase asks for.",
            entries: &["WORDS", "MARK", "BREAK", "SCRATCH_THAT"],
        },
        Type::Record {
            name: "Part",
            doc: "A part of a phrase: its [kind], and its words, its mark or its line breaks as \
                  [text].",
            data: false,
            fields: &[("kind", "Kind"), ("text", "String")],
        },
        Type::Record {
            name: "Typing",
            doc: "Delete [replacing] characters before the cursor, then type [text].",
            data: false,
            fields: &[("replacing", "Int"), ("text", "String")],
        },
    ],
    members: &[
        Member {
            name: "format",
            doc: "[phrase] tidied by the Mac app's rules, shared through AudioWhisper-Core: \
                  filler words (\"uh\", \"um,\") removed if [removeFillers], then [vocabulary] \
                  applied. [vocabulary] is one entry per line, as on the Mac: a word to spell \
                  exactly (\"Kubernetes\"), or `heard -> written`.",
            ..MEMBER
        },
        Member {
            name: "parse",
            doc: "[phrase]'s words and commands, in order.",
            result: Out::Records("Part"),
            ..MEMBER
        },
        Member {
            name: "punctuate",
            doc: "How to type the spoken punctuation [mark] after [before].",
            result: Out::Record("Typing"),
            ..MEMBER
        },
        Member {
            name: "revertsAt",
            doc: "Whether a backspace with the cursor at [cursor], after [textBeforeCursor], can \
                  still take back [typed], the phrase dictation typed at [start].",
            ..MEMBER
        },
    ],
    ..API
};

pub const MIC_KEY_STATE: Api = Api {
    name: "MicKeyState",
    doc: "The mic key between a press and its release, in Rust: a tap toggles dictation, and a \
          hold of [holdMs] or more dictates until it is let go. Each press and release says when \
          it came, in milliseconds. Safe to use from any thread. Its native memory is freed once \
          nothing can reach it.",
    module: "dictation",
    native: "DictationNative",
    shape: Shape::Handle(Handle {
        create: "newMicKey",
        free: "freeMicKey",
        ..HANDLE
    }),
    nested: &[
        Type::Enum {
            name: "Status",
            doc: "What dictation is doing when the key goes down.",
            entries: &["IDLE", "LISTENING", "FINISHING"],
        },
        Type::Enum {
            name: "Action",
            doc: "What a press or a release asks dictation to do.",
            entries: &["NONE", "START", "FINISH", "END_HOLD", "TOGGLE"],
        },
    ],
    members: &[
        Member {
            name: "press",
            doc: "The press, at [now], while dictation is [status]: [Action.START] if it was \
                  idle.",
            call: "micKeyPress",
            args: &[arg("status", ArgKind::Ordinal("Status"))],
            result: Out::Entry("Action"),
            ..MEMBER
        },
        Member {
            name: "release",
            doc: "The release, at [now]; [Action.TOGGLE] if no press was seen.",
            call: "micKeyRelease",
            result: Out::Entry("Action"),
            ..MEMBER
        },
        Member {
            name: "holdMs",
            doc: "A press this long, in milliseconds, is a hold.",
            kind: Kind::Val,
            companion: true,
            ..MEMBER
        },
    ],
    ..API
};
