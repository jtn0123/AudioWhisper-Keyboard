//! `GifSessionNative`, the GIF panel's decisions, answered by awnative's `gif_session` module and
//! the gif crate's session, previews and send steps, and `GifSessionPlan` and `GifPreviewKeys`,
//! the Kotlin over it.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, HANDLE, Handle, MEMBER, METHOD, Member, Method, Out,
    Param, Return, Shape, arg,
};

const HANDLE_PARAM: (&str, Param) = ("handle", Param::Long);

/// A method of `GifSessionNative`.
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

pub const GIF_SESSION_NATIVE: Class = Class {
    name: "GifSessionNative",
    doc: "The GIF panel's decisions in libawnative: lib/rust/crates/awnative/src/gif_session.rs \
          and the gif crate. [GifSessionPlan] and [GifPreviewKeys] wrap them.",
    module: "gif_session",
    methods: &[
        method("create", &[], Return::Long),
        method("free", &[HANDLE_PARAM], Return::Unit),
        method(
            "call",
            &[
                HANDLE_PARAM,
                ("operation", Param::String),
                ("now", Param::Long),
                ("fields", Param::StringVararg),
            ],
            Return::Strings,
        ),
        method(
            "read",
            &[
                HANDLE_PARAM,
                ("generation", Param::Int),
                ("present", Param::Boolean),
                ("rows", Param::Strings),
            ],
            Return::Unit,
        ),
        method(
            "rows",
            &[HANDLE_PARAM, ("generation", Param::Int)],
            Return::Strings,
        ),
        method(
            "request",
            &[("media", Param::Int)],
            Return::Array(Element::Int),
        ),
        method("previewsCreate", &[], Return::Long),
        method("previewsFree", &[HANDLE_PARAM], Return::Unit),
        method(
            "previewsGet",
            &[HANDLE_PARAM, ("url", Param::String)],
            Return::Boolean,
        ),
        method(
            "previewsPut",
            &[HANDLE_PARAM, ("url", Param::String), ("bytes", Param::Int)],
            Return::Strings,
        ),
        method("previewsClear", &[HANDLE_PARAM], Return::Unit),
        method(
            "sendNext",
            &[
                ("step", Param::String),
                ("ok", Param::Boolean),
                ("linkable", Param::Boolean),
            ],
            Return::Strings,
        ),
        method("sendFiles", &[("id", Param::String)], Return::Strings),
        method(
            "sendStale",
            &[
                ("names", Param::Strings),
                ("modified", Param::Array(Element::Long)),
                ("now", Param::Long),
            ],
            Return::Array(Element::Int),
        ),
    ],
    ..CLASS
};

pub const GIF_SESSION_PLAN: Api = Api {
    name: "GifSessionPlan",
    doc: "The GIF panel's session: each call answers with its state and the effects for Kotlin \
          to run in order, as crates/gif's session documents them. Owns no Android resources and \
          never calls back while locked, so any thread may call it. Closing it frees the \
          session; a call after that throws.",
    module: "gif_session",
    native: "GifSessionNative",
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
            doc: "The state and effects of [operation] at [now], with these [fields].",
            args: &[
                arg("operation", ArgKind::Code("GifOp")),
                arg("fields", ArgKind::As("vararg fields: String", "fields")),
            ],
            ..MEMBER
        },
        Member {
            name: "read",
            doc: "Recents [generation]'s file held these GIF [rows], or was missing or damaged \
                  (not [present]). Called on the thread that read it.",
            ..MEMBER
        },
        Member {
            name: "rows",
            doc: "What a save of recents [generation] writes: the file, then GIF rows; empty if \
                  there is nothing to write.",
            ..MEMBER
        },
        Member {
            name: "request",
            doc: "How to fetch a `GifMedia` [media]: a `GifRequest` row.",
            companion: true,
            ..MEMBER
        },
        Member {
            name: "sendNext",
            doc: "What to do, in order, after sending a GIF reached [step], which worked or not \
                  ([ok]); [linkable] says it has a page link. `GifSendAction`s.",
            args: &[arg("step", ArgKind::Code("GifSendStep"))],
            companion: true,
            ..MEMBER
        },
        Member {
            name: "sendFiles",
            doc: "The file a GIF with [id] is sent from, then the part file it downloads into.",
            companion: true,
            ..MEMBER
        },
        Member {
            name: "sendStale",
            doc: "The indexes of the sent-GIF folder's files to delete at [now], given their \
                  [names] and when each was [modified].",
            companion: true,
            ..MEMBER
        },
    ],
    ..API
};

pub const GIF_PREVIEW_KEYS: Api = Api {
    name: "GifPreviewKeys",
    doc: "Which previews the GIF panel keeps in memory, in Rust: up to 24 MB, the least \
          recently used going first. The bytes stay in Kotlin under the URLs this keeps. Its \
          native memory is freed once nothing can reach it.",
    module: "gif_session",
    native: "GifSessionNative",
    shape: Shape::Handle(Handle {
        create: "previewsCreate",
        free: "previewsFree",
        synchronized: true,
        ..HANDLE
    }),
    members: &[
        Member {
            name: "get",
            doc: "Whether [url]'s preview is kept; if so it becomes the most recently used.",
            call: "previewsGet",
            ..MEMBER
        },
        Member {
            name: "put",
            doc: "[url]'s preview of [bytes] is kept, replacing any before: the URLs dropped to \
                  make room, which may include [url].",
            call: "previewsPut",
            result: Out::List,
            ..MEMBER
        },
        Member {
            name: "clear",
            doc: "Drops every preview.",
            call: "previewsClear",
            ..MEMBER
        },
    ],
    ..API
};
