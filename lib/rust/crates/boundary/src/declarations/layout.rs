//! The layout's rules: what each key code is and shows (`KeyModels`), how rows of keys merge
//! (`LayoutRules`), and the panels' geometry and policies (`PanelRules`). Integers are the
//! layout's existing wire values, and keys themselves stay in Kotlin.

use crate::{CLASS, Class, Element, METHOD, Method, Param, Return};

const INTS: Param = Param::Array(Element::Int);
const FLOATS: Param = Param::Array(Element::Float);

pub const KEY_MODELS: Class = Class {
    name: "KeyModels",
    doc: "Key and domain policy in crates/keys. Integers are the existing layout wire values. \
          State fields: mode, variation, shift, UI mode, RTL, selection, manual selection, start \
          and end, incognito, actions overflow and editor, subtype dialog, composing, half width. \
          Selector kinds: case, shift, variation, direction, width; children keep Kotlin's order.",
    module: "key_models",
    file: "layout",
    public: true,
    methods: &[
        Method {
            name: "codes",
            returns: Return::Array(Element::Int),
            ..METHOD
        },
        Method {
            name: "read",
            params: &[("raw", Param::Long), ("field", Param::Int)],
            returns: Return::Int,
            ..METHOD
        },
        Method {
            name: "write",
            params: &[
                ("raw", Param::Long),
                ("field", Param::Int),
                ("value", Param::Int),
            ],
            returns: Return::Long,
            ..METHOD
        },
        Method {
            name: "select",
            params: &[
                ("kind", Param::Int),
                ("raw", Param::Long),
                ("present", Param::Int),
            ],
            returns: Return::Int,
            ..METHOD
        },
        Method {
            name: "isCurrency",
            params: &[("code", Param::Int)],
            returns: Return::Boolean,
            rust: "::keys::models::is_currency",
            ..METHOD
        },
        Method {
            name: "currencySlot",
            params: &[("code", Param::Int)],
            returns: Return::Int,
            rust: "::keys::models::currency_slot",
            ..METHOD
        },
        Method {
            name: "currencyDefaults",
            returns: Return::Array(Element::Int),
            ..METHOD
        },
        Method {
            name: "isSpace",
            params: &[("type", Param::Int), ("code", Param::Int)],
            returns: Return::Boolean,
            rust: "::keys::models::is_space",
            ..METHOD
        },
        Method {
            name: "predefined",
            returns: Return::Strings,
            ..METHOD
        },
        Method {
            name: "codeKind",
            params: &[("code", Param::Int)],
            returns: Return::Int,
            rust: "::keys::models::code_kind",
            ..METHOD
        },
        Method {
            name: "text",
            params: &[
                ("code", Param::Int),
                ("label", Param::String),
                ("display", Param::Boolean),
                ("nonSpacing", Param::Boolean),
            ],
            returns: Return::String,
            rust: "::keys::models::text",
            ..METHOD
        },
        Method {
            name: "multiText",
            params: &[
                ("codes", INTS),
                ("label", Param::String),
                ("display", Param::Boolean),
            ],
            returns: Return::String,
            rust: "::keys::models::multi_text",
            ..METHOD
        },
        Method {
            name: "popupMode",
            params: &[("mode", Param::Int)],
            returns: Return::Boolean,
            rust: "::keys::models::popup_mode",
            ..METHOD
        },
        Method {
            name: "popupGroup",
            params: &[("group", Param::Int)],
            returns: Return::NullableString,
            ..METHOD
        },
        Method {
            name: "popupSource",
            params: &[("variation", Param::Int), ("present", Param::Int)],
            returns: Return::Int,
            ..METHOD
        },
        Method {
            name: "enabled",
            params: &[("code", Param::Int), ("a", Param::Int), ("b", Param::Int)],
            returns: Return::Boolean,
            rust: "::keys::models::enabled",
            ..METHOD
        },
        Method {
            name: "visible",
            params: &[
                ("code", Param::Int),
                ("utility", Param::Boolean),
                ("action", Param::Int),
                ("language", Param::Boolean),
            ],
            returns: Return::Boolean,
            rust: "::keys::models::visible",
            ..METHOD
        },
        Method {
            name: "phoneHint",
            params: &[("code", Param::Int)],
            returns: Return::NullableString,
            ..METHOD
        },
    ],
    ..CLASS
};

pub const LAYOUT_RULES: Class = Class {
    name: "LayoutRules",
    doc: "Layout rules in keys::assembly. Kotlin keeps the decoded keys as opaque objects: \
          `merge` and `sizes`, by hand in LayoutRules.kt, pass row sizes and take back indices.",
    module: "layout_rules",
    file: "layout",
    public: true,
    methods: &[
        Method {
            name: "plan",
            params: &[("mode", Param::Int), ("numberRow", Param::Boolean)],
            returns: Return::Array(Element::Int),
            ..METHOD
        },
        Method {
            name: "modifierType",
            params: &[("mainType", Param::Int)],
            returns: Return::Int,
            rust: "::keys::assembly::modifier_type",
            ..METHOD
        },
        Method {
            name: "mergeRows",
            params: &[
                ("main", INTS),
                ("modifier", INTS),
                ("extension", INTS),
                ("modCodes", INTS),
            ],
            returns: Return::Array(Element::Int),
            internal: true,
            ..METHOD
        },
        Method {
            name: "hints",
            params: &[
                ("mode", Param::Int),
                ("numberHints", Param::Boolean),
                ("main", INTS),
                ("symbols", INTS),
                ("types", INTS),
            ],
            returns: Return::Array(Element::Int),
            ..METHOD
        },
        Method {
            name: "cacheKey",
            params: &[("mode", Param::Int), ("subtypeHash", Param::Int)],
            returns: Return::Long,
            rust: "::keys::assembly::cache_key",
            ..METHOD
        },
        Method {
            name: "invalidated",
            params: &[
                ("keys", Param::Array(Element::Long)),
                ("mode", Param::Int),
                ("subtypeHash", Param::Int),
                ("filter", Param::Int),
            ],
            returns: Return::Array(Element::Long),
            ..METHOD
        },
    ],
    ..CLASS
};

/// A `PanelRules` method that takes an array of float settings and returns floats.
const fn floats(name: &'static str) -> Method {
    Method {
        name,
        params: &[("config", FLOATS)],
        returns: Return::Array(Element::Float),
        ..METHOD
    }
}

/// A `PanelRules` method that takes an array of int settings and returns ints.
const fn ints(name: &'static str) -> Method {
    Method {
        name,
        params: &[("config", INTS)],
        returns: Return::Array(Element::Int),
        ..METHOD
    }
}

pub const PANEL_RULES: Class = Class {
    name: "PanelRules",
    doc: "The media and clipboard panels' geometry and policies. Opaque value ids keep equality \
          and preference serialisation at the Android storage edge.",
    module: "panel_oracle",
    file: "layout",
    public: true,
    methods: &[
        floats("clipboardDialog"),
        floats("clipboardTargets"),
        ints("clipboardMenu"),
        floats("clipboardGrid"),
        ints("overflowAndHistoryMoves"),
        floats("scroll"),
        ints("selection"),
        Method {
            name: "editing",
            params: &[
                ("codes", INTS),
                ("hold", Param::Long),
                ("repeat", Param::Long),
                ("words", Param::Long),
            ],
            returns: Return::Array(Element::Int),
            ..METHOD
        },
        floats("cells"),
        Method {
            name: "emojiChoices",
            params: &[("tones", INTS), ("tone", Param::Int)],
            returns: Return::Array(Element::Int),
            ..METHOD
        },
        Method {
            name: "history",
            doc: "The emoji history after one edit: its [pinned] and [recent] ids, as two rows.",
            params: &[("pinned", INTS), ("recent", INTS), ("config", INTS)],
            returns: Return::IntArrays,
            rust: "crate::panel_rules::history",
            ..METHOD
        },
        Method {
            name: "arrange",
            doc: "The quick actions after the edits in [config]: the sticky action, the dynamic \
                  ones and the hidden ones, as three rows.",
            params: &[("config", INTS), ("dynamic", INTS), ("hidden", INTS)],
            returns: Return::IntArrays,
            rust: "crate::panel_rules::arrange",
            ..METHOD
        },
    ],
    ..CLASS
};
