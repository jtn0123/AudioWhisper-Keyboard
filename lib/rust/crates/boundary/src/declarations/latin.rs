//! `LatinSessionNative` and `NlpNative`, the in-process dictionary session and the NLP session,
//! answered by awnative's `suggest::session` and `nlp` modules, and `LatinSession`,
//! `NlpSession` and `NlpNames`, the Kotlin over them.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, HANDLE, Handle, MEMBER, METHOD, Member, Method, Out,
    Param, Return, Shape, Type, arg,
};

const HANDLE_PARAM: (&str, Param) = ("handle", Param::Long);
const PREVIOUS: (&str, Param) = ("previous", Param::Strings);
const SENTENCE_START: (&str, Param) = ("sentenceStart", Param::Boolean);
const BLOCK: (&str, Param) = ("block", Param::Boolean);
const TAPS: (&str, Param) = ("taps", Param::Array(Element::Float));
const PERSONAL: (&str, Param) = ("personal", Param::Long);
const LEARNED: (&str, Param) = ("learned", Param::Long);

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

/// The stores a strip or a spell check reads.
const STORES: [crate::Arg; 3] = [
    arg("personal", ArgKind::Handle("PersonalWordStore")),
    arg("learned", ArgKind::NullableHandle("LearnedWordStore")),
    arg("previous", ArgKind::List),
];

pub const LATIN_SESSION_NATIVE: Class = Class {
    name: "LatinSessionNative",
    doc: "The dictionary session's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/suggest/session.rs. [LatinSession] wraps them.",
    module: "suggest::session",
    file: "latin",
    methods: &[
        method(
            "open",
            &[("bytes", Param::Array(Element::Byte))],
            Return::Long,
        ),
        method("free", &[HANDLE_PARAM], Return::Unit),
        method(
            "attachModel",
            &[
                HANDLE_PARAM,
                ("path", Param::String),
                ("offset", Param::Long),
                ("length", Param::Long),
            ],
            Return::Boolean,
        ),
        method(
            "probability",
            &[HANDLE_PARAM, ("word", Param::String)],
            Return::Int,
        ),
        method(
            "ngram",
            &[
                HANDLE_PARAM,
                ("word", Param::String),
                PREVIOUS,
                SENTENCE_START,
            ],
            Return::Int,
        ),
        method("words", &[HANDLE_PARAM], Return::Strings),
        method(
            "decode",
            &[
                HANDLE_PARAM,
                ("input", Param::String),
                PREVIOUS,
                SENTENCE_START,
                BLOCK,
                TAPS,
            ],
            Return::Strings,
        ),
        method(
            "suggest",
            &[
                HANDLE_PARAM,
                PERSONAL,
                LEARNED,
                ("typed", Param::String),
                PREVIOUS,
                SENTENCE_START,
                ("midWord", Param::Boolean),
                BLOCK,
                TAPS,
                ("maxCount", Param::Int),
                ("strength", Param::Int),
            ],
            Return::Strings,
        ),
        method(
            "spell",
            &[
                HANDLE_PARAM,
                PERSONAL,
                LEARNED,
                ("word", Param::String),
                BLOCK,
                ("maxCount", Param::Int),
                ("threshold", Param::Float),
            ],
            Return::Strings,
        ),
    ],
    ..CLASS
};

pub const NLP_NATIVE: Class = Class {
    name: "NlpNative",
    doc: "The NLP session's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/nlp.rs. [NlpSession], [LatinSession] and [NlpNames] \
          wrap them.",
    module: "nlp",
    file: "latin",
    methods: &[
        method(
            "isName",
            &[
                ("dictionary", Param::Long),
                PERSONAL,
                ("word", Param::String),
            ],
            Return::Boolean,
        ),
        method(
            "isNameWithoutDictionary",
            &[PERSONAL, ("word", Param::String)],
            Return::Boolean,
        ),
        method(
            "unknownWords",
            &[("dictionary", Param::Long), ("words", Param::Strings)],
            Return::Strings,
        ),
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
        method(
            "event",
            &[
                HANDLE_PARAM,
                ("learner", Param::Long),
                ("store", Param::Long),
                ("taps", Param::Long),
                ("kept", Param::Long),
                ("dictionary", Param::Long),
                ("fields", Param::StringVararg),
            ],
            Return::Strings,
        ),
    ],
    ..CLASS
};

pub const LATIN_SESSION: Api = Api {
    name: "LatinSession",
    doc: "Owns one native dictionary, search cache and QWERTY model. Only transfers data across \
          JNI. Closing it frees them; a call after that throws.",
    module: "latin",
    native: "LatinSessionNative",
    shape: Shape::Handle(Handle {
        free: "free",
        close: true,
        synchronized: true,
        lend: true,
        ..HANDLE
    }),
    nested: &[
        Type::Record {
            name: "Candidate",
            doc: "A word the decoder found, with its score and its kind and flags.",
            data: false,
            constants: &[],
            fields: &[
                ("word", "String"),
                ("score", "Int"),
                ("kindAndFlags", "Int"),
            ],
        },
        Type::Record {
            name: "Strip",
            doc: "The candidate strip: the indexes of its auto-correction and of the typed word, \
                  or -1, and its words.",
            data: false,
            constants: &[],
            fields: &[
                ("autoCorrection", "Int"),
                ("typedWord", "Int"),
                ("words", "List<String>"),
            ],
        },
        Type::Record {
            name: "Verdict",
            doc: "A spell check: whether the word is valid, whether that is confident, and the \
                  suggestions.",
            data: false,
            constants: &[],
            fields: &[
                ("isValid", "Boolean"),
                ("isHighConfidence", "Boolean"),
                ("suggestions", "List<String>"),
            ],
        },
    ],
    members: &[
        Member {
            name: "open",
            doc: "A session over the dictionary [bytes], or null if they aren't one.",
            companion: true,
            result: Out::NullableWrap("LatinSession"),
            ..MEMBER
        },
        Member {
            name: "isName",
            doc: "Whether [word] is a name, by the dictionary and the [personal] words.",
            native: "NlpNative",
            args: &[arg("personal", ArgKind::Handle("PersonalWordStore"))],
            ..MEMBER
        },
        Member {
            name: "unknownWords",
            doc: "Which of [words] the dictionary doesn't know.",
            native: "NlpNative",
            args: &[arg("words", ArgKind::List)],
            result: Out::List,
            ..MEMBER
        },
        Member {
            name: "attachModel",
            doc: "Reads the n-gram model at [offset] of [path] (an uncompressed APK entry) into \
                  the session.",
            ..MEMBER
        },
        Member {
            name: "probability",
            ..MEMBER
        },
        Member {
            name: "ngram",
            args: &[STORES[2]],
            ..MEMBER
        },
        Member {
            name: "words",
            result: Out::List,
            ..MEMBER
        },
        Member {
            name: "decode",
            args: &[STORES[2]],
            result: Out::Records("Candidate"),
            ..MEMBER
        },
        Member {
            name: "suggest",
            doc: "The strip for [typed]; [strength] is the auto-correction strength's ordinal \
                  ([SuggestRules.strengthNames]).",
            args: &STORES,
            result: Out::Record("Strip"),
            ..MEMBER
        },
        Member {
            name: "spell",
            args: &[STORES[0], STORES[1]],
            result: Out::Record("Verdict"),
            ..MEMBER
        },
    ],
    ..API
};

pub const NLP_SESSION: Api = Api {
    name: "NlpSession",
    doc: "One orchestration owner shared by both renderers. Calls copy UTF-16; no native \
          callback.",
    module: "latin",
    native: "NlpNative",
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
            doc: "[operation]'s answer to [fields], each as its [NlpOp] describes.",
            args: &[
                arg("operation", ArgKind::Code("NlpOp")),
                arg("fields", ArgKind::As("vararg fields: String", "fields")),
            ],
            ..MEMBER
        },
        Member {
            name: "event",
            doc: "The plan for the typing event in [fields] (an [NlpEvent] row, then an \
                  [NlpContext] row for the kinds that take one), which changes [learner], \
                  [store], [taps] and [kept] itself, asking [dictionary] when one is lent.",
            args: &[
                arg("learner", ArgKind::Handle("Learner")),
                arg("store", ArgKind::Handle("LearnedWordStore")),
                arg("taps", ArgKind::Handle("TapPositions")),
                arg("kept", ArgKind::Handle("KeptWordStore")),
                arg("dictionary", ArgKind::Lent("LatinSession")),
                arg("fields", ArgKind::As("vararg fields: String", "fields")),
            ],
            ..MEMBER
        },
    ],
    ..API
};

pub const NLP_NAMES: Api = Api {
    name: "NlpNames",
    doc: "The same name check as [LatinSession.isName] when the dictionary has not loaded yet.",
    module: "latin",
    native: "NlpNative",
    members: &[Member {
        name: "withoutDictionary",
        doc: "Whether [word] is a name by the [personal] words alone.",
        call: "isNameWithoutDictionary",
        args: &[arg("personal", ArgKind::Handle("PersonalWordStore"))],
        ..MEMBER
    }],
    ..API
};
