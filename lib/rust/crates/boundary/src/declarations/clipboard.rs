//! `ClipboardRules`, the clipboard history's policy, answered by awnative's `clipboard` module and
//! stores::clipboard_policy.

use crate::{CLASS, Class, Element, METHOD, Method, Param, Return};

pub const CLIPBOARD_RULES: Class = Class {
    name: "ClipboardRules",
    doc: "The clipboard history's policy, in Rust (stores::clipboard_policy). Texts compare as \
          lossless UTF-16.",
    module: "clipboard",
    public: true,
    methods: &[
        Method {
            name: "plan",
            doc: "The history's order, its three time groups, the clips over the limit, the \
                  expired ones, the last text's duplicate (index and whether pinned, or -1), and \
                  the unpinned clips: five longs a clip in [records], one text a clip in [texts] \
                  and the new text last, and ten settings in [config].",
            params: &[
                ("records", Param::Array(Element::Long)),
                ("texts", Param::Strings),
                ("config", Param::Array(Element::Long)),
            ],
            returns: Return::IntArrays,
            instance: true,
            ..METHOD
        },
        Method {
            name: "mimeMatches",
            doc: "Whether the [concrete] MIME type matches [desired], which may have wildcards.",
            params: &[("concrete", Param::String), ("desired", Param::String)],
            returns: Return::Boolean,
            instance: true,
            rust: "::stores::clipboard_policy::mime_matches",
            ..METHOD
        },
        Method {
            name: "eligible",
            doc: "Whether a clip of the MIME [types] can go into an editor that takes [editor].",
            params: &[("types", Param::Strings), ("editor", Param::Strings)],
            returns: Return::Boolean,
            instance: true,
            rust: "::stores::clipboard_policy::eligible",
            ..METHOD
        },
    ],
    ..CLASS
};
