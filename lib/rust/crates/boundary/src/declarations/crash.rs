//! `CrashNative`, what a crash leaves behind and the report the crash screen shows, answered by
//! awnative's `crash` module and `utilities::crash`, and `CrashRules`, the Kotlin over it.

use crate::{API, Api, CLASS, Class, MEMBER, METHOD, Member, Method, Param, Return};

/// A method of `CrashNative`.
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
const fn member(name: &'static str, doc: &'static str) -> Member {
    Member {
        name,
        doc,
        ..MEMBER
    }
}

pub const CRASH_NATIVE: Class = Class {
    name: "CrashNative",
    doc: "What a crash leaves behind and the crash screen's report, in libawnative: \
          lib/rust/crates/awnative/src/crash.rs and utilities::crash. [CrashRules] wraps it.",
    module: "crash",
    methods: &[
        method(
            "after",
            &[("last", Param::Long), ("now", Param::Long)],
            Return::Int,
        ),
        method(
            "same",
            &[("last", Param::String), ("current", Param::String)],
            Return::Boolean,
        ),
        method(
            "report",
            &[
                ("facts", Param::Strings),
                ("debug", Param::Boolean),
                ("names", Param::Strings),
                ("details", Param::Strings),
            ],
            Return::String,
        ),
    ],
    ..CLASS
};

pub const CRASH_RULES: Api = Api {
    name: "CrashRules",
    doc: "What a crash leaves behind and what the crash screen shows, in Rust \
          (utilities::crash). A caller that must work without the library catches \
          LinkageError: the crash handler does.",
    module: "crash",
    native: "CrashNative",
    members: &[
        member(
            "after",
            "What to do after a crash at [now], the last at [last] (0 for none): `CrashAfter` \
             flags.",
        ),
        member(
            "same",
            "Whether the [last] crash's report, as read back, is the [current] one.",
        ),
        member(
            "report",
            "The crash screen's report of these [facts] (a `CrashFact` row), linking the \
             version to its commit if [debug], with each stack trace's [names] and [details].",
        ),
    ],
    ..API
};
