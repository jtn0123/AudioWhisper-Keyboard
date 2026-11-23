//! `DictationEdgesNative`, the microphone's audio rules and a model's file checks, answered by
//! awnative's `dictation_edges` module and the dictation crate's `audio` and `model_check`, and
//! `MicRules` and `ModelCheckRules`, the Kotlin over it.

use crate::{API, Api, CLASS, Class, Element, MEMBER, METHOD, Member, Method, Param, Return};

/// A method of `DictationEdgesNative`.
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

/// A member with the method's own name and signature.
const fn rule(name: &'static str, doc: &'static str) -> Member {
    Member {
        name,
        doc,
        ..MEMBER
    }
}

const LONGS: Param = Param::Array(Element::Long);
const FLAGS: Param = Param::Array(Element::Boolean);

pub const DICTATION_EDGES_NATIVE: Class = Class {
    name: "DictationEdgesNative",
    doc: "The microphone's audio rules and a model's file checks in libawnative: \
          lib/rust/crates/awnative/src/dictation_edges.rs and the dictation crate. [MicRules] \
          and [ModelCheckRules] wrap them.",
    module: "dictation_edges",
    methods: &[
        method(
            "level",
            &[
                ("samples", Param::Array(Element::Float)),
                ("count", Param::Int),
            ],
            Return::Float,
        ),
        method(
            "read",
            &[("count", Param::Int), ("running", Param::Boolean)],
            Return::Int,
        ),
        method(
            "pcm16",
            &[("wav", Param::Array(Element::Byte))],
            Return::Array(Element::Float),
        ),
        method("bufferBytes", &[("minimum", Param::Int)], Return::Int),
        method(
            "quick",
            &[("expected", LONGS), ("present", FLAGS), ("lengths", LONGS)],
            Return::Array(Element::Int),
        ),
        method(
            "keys",
            &[
                ("paths", Param::Strings),
                ("lengths", LONGS),
                ("modified", LONGS),
            ],
            Return::Strings,
        ),
        method("toHash", &[("known", FLAGS)], Return::Array(Element::Int)),
        method(
            "verdict",
            &[
                ("known", FLAGS),
                ("expected", Param::Strings),
                ("sums", Param::NullableStrings),
            ],
            Return::Array(Element::Int),
        ),
    ],
    ..CLASS
};

pub const MIC_RULES: Api = Api {
    name: "MicRules",
    doc: "The microphone's audio rules, in Rust (dictation::audio): how loud a chunk is, what a \
          read's count means, the PCM of a WAV file played in place of the mic, and the mic's \
          buffer.",
    module: "dictation_edges",
    native: "DictationEdgesNative",
    members: &[
        rule(
            "level",
            "How loud the first [count] [samples] are, 0 to 1: their root mean square times 8.",
        ),
        rule(
            "read",
            "What the recording thread does with a read that returned [count], while it is \
             still [running] or not: a `MicRead` code.",
        ),
        rule(
            "pcm16",
            "The samples of a 16 kHz mono 16-bit PCM [wav] file, each over 32,768. Throws \
             IllegalArgumentException if it is not a WAV file or holds no audio.",
        ),
        rule(
            "bufferBytes",
            "The bytes the mic buffers: half a second of float samples, or Android's [minimum] \
             if that is more.",
        ),
    ],
    ..API
};

pub const MODEL_CHECK_RULES: Api = Api {
    name: "ModelCheckRules",
    doc: "Whether a model's files are all there and whole, in Rust (dictation::model_check). \
          Kotlin looks at the files, hashes the ones [toHash] names, and remembers the keys \
          [verdict] says passed. A state answer is a `ModelCheckState` code, the count of the \
          files that make it missing or damaged, then their indexes.",
    module: "dictation_edges",
    native: "DictationEdgesNative",
    members: &[
        rule(
            "quick",
            "From whether each file is [present] and its [lengths], against the [expected] \
             sizes: a state answer. Missing files win over wrong sizes.",
        ),
        rule(
            "keys",
            "What each file, at these [paths], [lengths] and [modified] times, is remembered \
             by once it passed.",
        ),
        rule(
            "toHash",
            "The indexes of the files to hash: those not [known] to have passed.",
        ),
        rule(
            "verdict",
            "A full check's verdict from which files were [known], their [expected] SHA-256 and \
             the [sums] of those hashed (null where the read failed): a state answer, then the \
             indexes of the files to remember as passed.",
        ),
    ],
    ..API
};
