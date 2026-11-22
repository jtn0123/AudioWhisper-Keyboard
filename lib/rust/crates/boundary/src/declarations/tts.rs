//! `TtsNative`, the read-aloud voice, answered by awnative's `tts` module, and `SherpaTts`, the
//! Kotlin over it, with `TtsAudio`, the callback that plays what it makes.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, HANDLE, Handle, Kind, MEMBER, METHOD, Member, Method,
    Module, Param, Return, Shape, Type, arg,
};

const HANDLE_PARAM: (&str, Param) = ("handle", Param::Long);

/// A method of `TtsNative`.
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

pub const TTS_NATIVE: Class = Class {
    name: "TtsNative",
    doc: "The voice's entry points in libawnative: lib/rust/crates/awnative/src/tts.rs. A handle \
          is a token in Rust's registry, so a call racing `close` finds the voice gone instead of \
          freed memory. [SherpaTts] wraps them.",
    module: "tts",
    methods: &[
        method(
            "create",
            &[
                ("directory", Param::String),
                ("referenceWav", Param::Array(Element::Byte)),
            ],
            Return::Long,
        ),
        method("rate", &[HANDLE_PARAM], Return::Int),
        method(
            "generate",
            &[
                HANDLE_PARAM,
                ("text", Param::String),
                ("callback", Param::Object("TtsAudio")),
            ],
            Return::Unit,
        ),
        method("cancel", &[HANDLE_PARAM], Return::Unit),
        method("close", &[HANDLE_PARAM], Return::Unit),
    ],
    ..CLASS
};

pub const TTS_AUDIO: (&str, Module, Type) = (
    "tts",
    Module::Rust,
    Type::Interface {
        name: "TtsAudio",
        doc: "What the voice made, called synchronously while it speaks; return false to stop. The \
              buffer is borrowed only until it returns.",
        members: &[("fun audio(pcm: java.nio.ByteBuffer): Boolean", "")],
    },
);

pub const SHERPA_TTS: Api = Api {
    name: "SherpaTts",
    doc: "The read-aloud voice in [directory]. Rust owns generating, cancelling and the model; \
          Android owns the files and plays the audio. Speaking and cancelling take no lock, so \
          [cancel] stops a [speak] in progress.",
    module: "tts",
    native: "TtsNative",
    shape: Shape::Handle(Handle {
        params: &[arg(
            "referenceWav",
            ArgKind::Fixed("java.io.File(directory, \"loona.wav\").readBytes()"),
        )],
        create: "create",
        free: "close",
        close: true,
        ..HANDLE
    }),
    members: &[
        Member {
            name: "sampleRate",
            doc: "Samples a second in what it makes.",
            kind: Kind::Val,
            call: "rate",
            ..MEMBER
        },
        Member {
            name: "speak",
            doc: "Speaks [text], handing [onAudio] each piece of audio until it returns false. The \
                  buffer is read-only and borrowed only until [onAudio] returns; copy it to keep \
                  it.",
            call: "generate",
            args: &[arg(
                "callback",
                ArgKind::As(
                    "onAudio: (java.nio.ByteBuffer) -> Boolean",
                    "TtsAudio { onAudio(it.asReadOnlyBuffer().order(java.nio.ByteOrder.nativeOrder())) }",
                ),
            )],
            ..MEMBER
        },
        Member {
            name: "cancel",
            doc: "Stops what it is speaking.",
            ..MEMBER
        },
    ],
    ..API
};
