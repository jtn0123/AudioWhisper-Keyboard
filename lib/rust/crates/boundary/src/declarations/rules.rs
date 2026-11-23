//! Small rule objects whose `external fun`s are themselves the Kotlin API: geometry the renderer
//! and its tests share, and the input method window's visibility rule. Each calls the awnative
//! module it names.

use crate::{CLASS, Class, Element, METHOD, Method, Param, Return};

pub const IME_WINDOW_RULES: Class = Class {
    name: "ImeWindowRules",
    doc: "What the keyboard's views are told as Android shows the input method window \
          (awnative ime_window.rs).",
    module: "ime_window",
    file: "rules",
    public: true,
    methods: &[
        Method {
            name: "retellVisible",
            doc: "When the window is shown: whether a view last [told] some window visibility \
                  must be told again that it is visible, given the [window]'s own (both \
                  View.VISIBLE, INVISIBLE or GONE).",
            params: &[("told", Param::Int), ("window", Param::Int)],
            returns: Return::Boolean,
            ..METHOD
        },
        Method {
            name: "layout",
            doc: "Where the plain-View root puts everything, an [ImeWindowPlan] row then the \
                  Smartbar config drawn, for [facts]: an [ImeWindowFacts] row then the \
                  Smartbar config.",
            params: &[("facts", Param::Array(Element::Float))],
            returns: Return::Array(Element::Float),
            ..METHOD
        },
    ],
    ..CLASS
};

pub const GLIDE_TRAIL_RULES: Class = Class {
    name: "GlideTrailRules",
    doc: "Font-independent geometry comparison against Compose's test-only circle drawing \
          reference.",
    module: "render_trail",
    file: "rules",
    public: true,
    methods: &[Method {
        name: "circles",
        params: &[
            ("times", Param::Array(Element::Long)),
            ("points", Param::Array(Element::Float)),
            ("finish", Param::Long),
            ("now", Param::Long),
            ("duration", Param::Long),
            ("strip", Param::Float),
        ],
        returns: Return::Array(Element::Float),
        instance: true,
        ..METHOD
    }],
    ..CLASS
};

pub const RENDER_CONTENT_RULES: Class = Class {
    name: "RenderContentRules",
    doc: "Font-independent geometry oracle; drawing calls the same functions inside Rust.",
    module: "render_content",
    file: "rules",
    public: true,
    methods: &[Method {
        name: "phoneLabel",
        params: &[
            ("mode", Param::String),
            ("kind", Param::String),
            ("boundsAndMetrics", Param::Array(Element::Float)),
            ("hint", Param::Boolean),
        ],
        returns: Return::Array(Element::Float),
        instance: true,
        ..METHOD
    }],
    ..CLASS
};

pub const SUBTYPE_RULES: Class = Class {
    name: "SubtypeRules",
    doc: "The subtype sheet's geometry and rows, as the renderer lays them out.",
    module: "render_subtypes",
    file: "rules",
    public: true,
    methods: &[
        Method {
            name: "geometry",
            params: &[("config", Param::Array(Element::Float))],
            returns: Return::Array(Element::Float),
            instance: true,
            ..METHOD
        },
        Method {
            name: "rows",
            params: &[
                ("selected", Param::Long),
                ("ids", Param::Array(Element::Long)),
                ("labels", Param::Strings),
            ],
            returns: Return::Strings,
            instance: true,
            ..METHOD
        },
    ],
    ..CLASS
};

pub const MEDIA_RULES: Class = Class {
    name: "MediaRules",
    doc: "The media panel's chips, GIF grid and frame cache, as the live surface runs them.",
    module: "media_oracle",
    file: "rules",
    public: true,
    methods: &[
        Method {
            name: "scrollChips",
            params: &[("config", Param::Array(Element::Float))],
            returns: Return::Array(Element::Float),
            instance: true,
            ..METHOD
        },
        Method {
            name: "grid",
            params: &[("config", Param::Array(Element::Float))],
            returns: Return::Array(Element::Float),
            instance: true,
            ..METHOD
        },
        Method {
            name: "trace",
            params: &[
                ("config", Param::Array(Element::Int)),
                ("operations", Param::Array(Element::Long)),
            ],
            returns: Return::Array(Element::Long),
            instance: true,
            ..METHOD
        },
    ],
    ..CLASS
};

pub const IME_CHROME_RULES: Class = Class {
    name: "ImeChromeRules",
    doc: "The keyboard chrome's sizes: fonts, rows, touch targets, one-handed mode, previews, \
          labels and the extract and side buttons.",
    module: "render_chrome",
    file: "rules",
    public: true,
    methods: &[
        Method {
            name: "fontMultiplier",
            params: &[("config", Param::Array(Element::Int))],
            returns: Return::Float,
            instance: true,
            ..METHOD
        },
        Method {
            name: "rowHeights",
            params: &[
                ("base", Param::Float),
                ("config", Param::Array(Element::Int)),
            ],
            returns: Return::Array(Element::Float),
            instance: true,
            ..METHOD
        },
        Method {
            name: "minimumTarget",
            params: &[("config", Param::Array(Element::Float))],
            returns: Return::Array(Element::Float),
            instance: true,
            ..METHOD
        },
        Method {
            name: "oneHanded",
            params: &[("config", Param::Array(Element::Int))],
            returns: Return::Array(Element::Float),
            instance: true,
            ..METHOD
        },
        Method {
            name: "watermark",
            params: &[("flags", Param::Array(Element::Int))],
            returns: Return::Boolean,
            instance: true,
            rust: "crate::render_decorations::watermark",
            ..METHOD
        },
        Method {
            name: "previewSize",
            params: &[("config", Param::Array(Element::Float))],
            returns: Return::Array(Element::Int),
            instance: true,
            ..METHOD
        },
        Method {
            name: "startEllipsis",
            params: &[
                ("labels", Param::Strings),
                ("config", Param::Array(Element::Float)),
            ],
            returns: Return::Strings,
            instance: true,
            ..METHOD
        },
        Method {
            name: "extract",
            params: &[("config", Param::Array(Element::Float))],
            returns: Return::Array(Element::Float),
            instance: true,
            ..METHOD
        },
        Method {
            name: "sideButtons",
            params: &[("config", Param::Array(Element::Float))],
            returns: Return::Array(Element::Float),
            instance: true,
            ..METHOD
        },
    ],
    ..CLASS
};
