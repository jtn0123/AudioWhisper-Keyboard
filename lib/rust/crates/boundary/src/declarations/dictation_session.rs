//! `DictationSessionNative`, one dictation's ordered effects, answered by awnative's
//! `dictation_session` module, and `DictationSessionPlan`, the Kotlin over it.

use crate::{
    API, Api, ArgKind, CLASS, Class, HANDLE, Handle, MEMBER, METHOD, Member, Method, Param, Return,
    Shape, arg, default,
};

const HANDLE_PARAM: (&str, Param) = ("handle", Param::Long);

/// A method of `DictationSessionNative`.
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

pub const DICTATION_SESSION_NATIVE: Class = Class {
    name: "DictationSessionNative",
    doc: "A dictation session's entry points in libawnative: lib/rust/crates/awnative/src/\
          dictation_session.rs. [DictationSessionPlan] wraps them.",
    module: "dictation_session",
    methods: &[
        method("create", &[], Return::Long),
        method("free", &[HANDLE_PARAM], Return::Unit),
        method(
            "call",
            &[
                HANDLE_PARAM,
                ("operation", Param::String),
                ("now", Param::Long),
                ("generation", Param::Int),
                ("fields", Param::StringVararg),
            ],
            Return::Strings,
        ),
        method(
            "speech",
            &[
                HANDLE_PARAM,
                ("generation", Param::Int),
                ("detected", Param::Boolean),
            ],
            Return::Unit,
        ),
        method(
            "model",
            &[HANDLE_PARAM, ("operation", Param::String)],
            Return::Int,
        ),
        method("claimPreview", &[HANDLE_PARAM], Return::Boolean),
        method("completePreview", &[HANDLE_PARAM], Return::Unit),
        method(
            "releaseAt",
            &[
                ("idle", Param::Long),
                ("now", Param::Long),
                ("shown", Param::Boolean),
            ],
            Return::Long,
        ),
        method(
            "ending",
            &[("typed", Param::Boolean), ("failures", Param::Int)],
            Return::String,
        ),
    ],
    ..CLASS
};

pub const DICTATION_SESSION_PLAN: Api = Api {
    name: "DictationSessionPlan",
    doc: "Ordered session effects; owns no Android resources and never calls back while locked. \
          Closing it frees the session; a call after that throws.",
    module: "dictation_session",
    native: "DictationSessionNative",
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
            doc: "The effects of [operation] at [now], for listening [generation] with these \
                  [fields], as crates/dictation's session documents them.",
            args: &[
                arg("operation", ArgKind::Code("DictationEvent")),
                default("generation", "0"),
                arg("fields", ArgKind::As("vararg fields: String", "fields")),
            ],
            ..MEMBER
        },
        Member {
            name: "speech",
            doc: "Listening [generation] [detected] speech, or lost it.",
            ..MEMBER
        },
        Member {
            name: "model",
            doc: "The model cache's answer to [operation].",
            args: &[arg("operation", ArgKind::Code("DictationModelOp"))],
            ..MEMBER
        },
        Member {
            name: "claimPreview",
            doc: "Whether this caller may run the live preview now; if so, it calls \
                  [completePreview] after.",
            ..MEMBER
        },
        Member {
            name: "completePreview",
            doc: "The live preview claimed with [claimPreview] has finished.",
            ..MEMBER
        },
        Member {
            name: "releaseAt",
            doc: "When to release a model idle since [idle], at [now], with the keyboard \
                  [shown] or not.",
            companion: true,
            ..MEMBER
        },
        Member {
            name: "ending",
            doc: "The problem to report as a dictation ends: empty if it [typed] anything, \
                  else one for [failures] or for nothing caught.",
            companion: true,
            ..MEMBER
        },
    ],
    ..API
};
