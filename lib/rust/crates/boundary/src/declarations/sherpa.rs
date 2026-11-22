//! `SherpaNative`, the speech recognizer and voice detector, answered by awnative's `sherpa`
//! module, and `DictationAudio`, the audio dictation records. `SherpaAsr` and `SherpaVad` over
//! them stay hand-written: a recognizer is cancelled while it decodes, under a read-write lock no
//! generated class takes, and a voice batch reads phrases from an array of arrays.

use crate::{
    API, Api, CLASS, Class, Element, Kind, MEMBER, METHOD, Member, Method, Out, Param, Return,
};

const HANDLE_PARAM: (&str, Param) = ("handle", Param::Long);
const SAMPLES: (&str, Param) = ("samples", Param::Int);

/// A method of `SherpaNative`.
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

pub const SHERPA_NATIVE: Class = Class {
    name: "SherpaNative",
    doc: "The recognizer's and voice detector's entry points in libawnative: lib/rust/crates/\
          awnative/src/sherpa.rs. PCM crosses in a direct, native-order Float32 buffer, borrowed \
          only until the call returns. [SherpaAsr], [SherpaVad] and [DictationAudio] wrap them.",
    module: "sherpa",
    methods: &[
        method("audioConfig", &[], Return::Array(Element::Int)),
        method(
            "createRecognizer",
            &[("directory", Param::String)],
            Return::Long,
        ),
        method(
            "transcribe",
            &[
                HANDLE_PARAM,
                ("pcm", Param::Object("java.nio.ByteBuffer")),
                SAMPLES,
            ],
            Return::String,
        ),
        method("cancel", &[HANDLE_PARAM], Return::Unit),
        method("freeRecognizer", &[HANDLE_PARAM], Return::Unit),
        method("createVad", &[("model", Param::String)], Return::Long),
        method(
            "vad",
            &[
                HANDLE_PARAM,
                ("pcm", Param::Object("java.nio.ByteBuffer?")),
                SAMPLES,
                ("mode", Param::Int),
            ],
            Return::Object("Array<Any?>", "jobjectArray"),
        ),
        method("freeVad", &[HANDLE_PARAM], Return::Unit),
    ],
    ..CLASS
};

pub const DICTATION_AUDIO: Api = Api {
    name: "DictationAudio",
    doc: "The audio dictation records, as `dictation::segmentation` cuts it.",
    module: "sherpa",
    native: "SherpaNative",
    members: &[
        Member {
            name: "sampleRate",
            doc: "Samples a second.",
            kind: Kind::Val,
            call: "audioConfig",
            result: Out::Then("Int", "[0]"),
            ..MEMBER
        },
        Member {
            name: "chunkSamples",
            doc: "Samples in each chunk the microphone hands on.",
            kind: Kind::Val,
            call: "audioConfig",
            result: Out::Then("Int", "[1]"),
            ..MEMBER
        },
    ],
    ..API
};
