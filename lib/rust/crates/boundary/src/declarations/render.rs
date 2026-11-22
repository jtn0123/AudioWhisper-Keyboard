//! `RenderNative`, the Rust renderer, answered by awnative's `render` module and the render_*
//! modules beside it. `KeyboardRenderer`, the Kotlin over it, is in keyboard_renderer.rs.

use crate::{CLASS, Class, Element, METHOD, Method, Param, Return};

const HANDLE_PARAM: (&str, Param) = ("handle", Param::Long);
const STRINGS: Param = Param::Strings;
const INTS: Param = Param::Array(Element::Int);
const LONGS: Param = Param::Array(Element::Long);
const FLOATS: Param = Param::Array(Element::Float);
const LABELS: (&str, Param) = ("labels", STRINGS);
const POINTER: (&str, Param) = ("pointer", Param::Int);
const ACTIONS: Return = Return::Array(Element::Int);
const REPLY: Return = Return::Strings;

/// A method of `RenderNative` taking the renderer's handle and then `params`, answered by `rust`,
/// a function of awnative's `render` module or, starting `crate::`, of another.
const fn method(
    name: &'static str,
    rust: &'static str,
    params: &'static [(&'static str, Param)],
    returns: Return,
) -> Method {
    Method {
        name,
        params,
        returns,
        rust,
        ..METHOD
    }
}

pub const RENDER_NATIVE: Class = Class {
    name: "RenderNative",
    doc: "The renderer's entry points in libawnative: lib/rust/crates/awnative/src/render.rs and \
          the render_*.rs modules beside it, which document the packets each takes. \
          [KeyboardRenderer] wraps them.",
    module: "render",
    methods: &[
        method("create", "", &[("fontPaths", STRINGS)], Return::Long),
        method("close", "", &[HANDLE_PARAM], Return::Unit),
        // The settings screens (render_settings.rs): each call gives back the screens' reply.
        method(
            "settings",
            "crate::render_settings::settings",
            &[HANDLE_PARAM, ("operation", Param::Int), ("input", STRINGS)],
            REPLY,
        ),
        method(
            "settingsTouch",
            "crate::render_settings::settings_touch",
            &[
                HANDLE_PARAM,
                ("metadata", INTS),
                ("times", LONGS),
                ("coords", FLOATS),
            ],
            REPLY,
        ),
        method(
            "settingsImage",
            "crate::render_settings::settings_image",
            &[
                HANDLE_PARAM,
                ("name", Param::String),
                ("width", Param::Int),
                ("height", Param::Int),
                ("pixels", INTS),
            ],
            REPLY,
        ),
        // The keyboard's scene (render.rs).
        method(
            "theme",
            "",
            &[
                HANDLE_PARAM,
                ("json", Param::String),
                ("names", STRINGS),
                ("colors", INTS),
                ("scale", FLOATS),
            ],
            Return::Unit,
        ),
        method(
            "update",
            "",
            &[
                HANDLE_PARAM,
                ("dimensions", FLOATS),
                ("metadata", STRINGS),
                ("candidateFlags", INTS),
                ("geometry", FLOATS),
                LABELS,
                ("candidates", STRINGS),
            ],
            Return::Unit,
        ),
        method(
            "popup",
            "",
            &[HANDLE_PARAM, ("key", Param::Int), LABELS, ("codes", INTS)],
            Return::Unit,
        ),
        method(
            "popupGrid",
            "",
            &[
                HANDLE_PARAM,
                ("prioritized", Param::Int),
                ("landscape", Param::Boolean),
            ],
            Return::Unit,
        ),
        method(
            "attach",
            "",
            &[
                HANDLE_PARAM,
                ("surface", Param::Object("Any")),
                ("hybrid", Param::Boolean),
                ("timeline", Param::Boolean),
            ],
            Return::Unit,
        ),
        method(
            "frame",
            "",
            &[HANDLE_PARAM, ("time", Param::Long), ("vsync", Param::Long)],
            Return::Boolean,
        ),
        method("detach", "", &[HANDLE_PARAM], Return::Unit),
        // The views drawn beside the keys.
        method(
            "decorations",
            "crate::render_decorations::decorations",
            &[
                HANDLE_PARAM,
                LABELS,
                ("height", Param::Float),
                ("flags", INTS),
            ],
            Return::Unit,
        ),
        method(
            "chrome",
            "crate::render_chrome::chrome",
            &[
                HANDLE_PARAM,
                ("config", FLOATS),
                LABELS,
                ("icons", STRINGS),
                ("metadata", STRINGS),
            ],
            Return::Unit,
        ),
        method(
            "chromeInfo",
            "crate::render_chrome::chrome_info",
            &[HANDLE_PARAM],
            Return::Array(Element::Float),
        ),
        method(
            "pixels",
            "crate::render_chrome::pixels",
            &[HANDLE_PARAM],
            Return::Array(Element::Int),
        ),
        method(
            "keyPreview",
            "crate::render_key_preview::key_preview",
            &[HANDLE_PARAM, ("code", Param::Int), LABELS],
            Return::Array(Element::Int),
        ),
        // The panels.
        method(
            "subtypes",
            "crate::render_subtypes::subtypes",
            &[
                HANDLE_PARAM,
                ("selected", Param::Long),
                ("ids", LONGS),
                LABELS,
                ("bottomInset", Param::Float),
            ],
            Return::Unit,
        ),
        method(
            "search",
            "crate::render_search::search",
            &[
                HANDLE_PARAM,
                ("density", Param::Float),
                LABELS,
                ("now", Param::Long),
            ],
            Return::Unit,
        ),
        method(
            "gifSent",
            "crate::render_gif::gif_sent",
            &[HANDLE_PARAM],
            Return::Unit,
        ),
        method(
            "gifs",
            "crate::render_gif::gifs",
            &[
                HANDLE_PARAM,
                ("config", FLOATS),
                LABELS,
                ("chips", Param::Int),
                ("ids", LONGS),
            ],
            Return::Unit,
        ),
        method(
            "mediaRepeat",
            "crate::render_media::media_repeat",
            &[HANDLE_PARAM, ("id", Param::Long), ("count", Param::Int)],
            Return::Unit,
        ),
        method(
            "mediaRequests",
            "crate::render_media::media_requests",
            &[HANDLE_PARAM],
            Return::Array(Element::Long),
        ),
        method(
            "mediaUpload",
            "crate::render_media::media_upload",
            &[
                HANDLE_PARAM,
                ("id", Param::Long),
                ("index", Param::Int),
                ("width", Param::Int),
                ("height", Param::Int),
                ("delay", Param::Long),
                ("last", Param::Boolean),
                ("pixels", INTS),
            ],
            Return::Boolean,
        ),
        method(
            "clipboard",
            "crate::render_clipboard::clipboard",
            &[HANDLE_PARAM, ("config", LONGS), ("records", LONGS), LABELS],
            Return::Unit,
        ),
        method(
            "emoji",
            "crate::render_emoji::emoji",
            &[
                HANDLE_PARAM,
                ("config", INTS),
                ("groups", INTS),
                ("values", STRINGS),
                ("history", INTS),
                LABELS,
            ],
            Return::Unit,
        ),
        method(
            "panels",
            "crate::render_panels::panels",
            &[HANDLE_PARAM, ("config", FLOATS), ("data", INTS), LABELS],
            Return::Unit,
        ),
        method(
            "panelArrangement",
            "crate::render_panels::panel_arrangement",
            &[HANDLE_PARAM],
            Return::IntArrays,
        ),
        // The smartbar (render_smartbar.rs).
        method(
            "smartbar",
            "crate::render_smartbar::smartbar",
            &[HANDLE_PARAM, ("config", FLOATS), ("codes", INTS), LABELS],
            Return::Unit,
        ),
        method(
            "candidateBindings",
            "crate::render_smartbar::candidate_bindings",
            &[HANDLE_PARAM, ("tokens", INTS)],
            ACTIONS,
        ),
        method(
            "candidateLongPress",
            "crate::render_smartbar::candidate_long_press",
            &[HANDLE_PARAM, POINTER],
            Return::Unit,
        ),
        method(
            "tooltipShown",
            "crate::render_smartbar::tooltip_shown",
            &[HANDLE_PARAM, POINTER, ("shown", Param::Boolean)],
            Return::Unit,
        ),
        method(
            "tick",
            "crate::render_smartbar::tick",
            &[HANDLE_PARAM, ("time", Param::Long)],
            ACTIONS,
        ),
        method(
            "nextDeadline",
            "crate::render_smartbar::next_deadline",
            &[HANDLE_PARAM],
            Return::Long,
        ),
        method(
            "barSettings",
            "crate::render_smartbar::bar_settings",
            &[HANDLE_PARAM, ("delay", Param::Long), ("changed", INTS)],
            ACTIONS,
        ),
        // Accessibility (render_accessibility.rs).
        method(
            "semanticConfig",
            "crate::render_accessibility::semantic_config",
            &[
                HANDLE_PARAM,
                ("names", STRINGS),
                ("values", STRINGS),
                ("text", STRINGS),
                ("popups", INTS),
                ("removable", INTS),
                ("checked", INTS),
                ("editor", INTS),
            ],
            Return::Unit,
        ),
        method(
            "semanticSnapshot",
            "crate::render_accessibility::semantic_snapshot",
            &[HANDLE_PARAM],
            Return::String,
        ),
        method(
            "semanticAction",
            "crate::render_accessibility::semantic_action",
            &[HANDLE_PARAM, ("id", Param::Int), ("action", Param::Int)],
            ACTIONS,
        ),
        method(
            "semanticCancel",
            "crate::render_accessibility::semantic_cancel",
            &[HANDLE_PARAM],
            ACTIONS,
        ),
        // Touch (render_input.rs).
        method(
            "touch",
            "crate::render_input::touch",
            &[
                HANDLE_PARAM,
                ("metadata", INTS),
                ("times", LONGS),
                ("coords", FLOATS),
                ("trail", Param::Long),
                ("aimed", Param::Boolean),
            ],
            ACTIONS,
        ),
        method(
            "gestures",
            "crate::render_input::gestures",
            &[HANDLE_PARAM, ("values", FLOATS), ("actions", STRINGS)],
            Return::Unit,
        ),
        method(
            "longPress",
            "crate::render_input::long_press",
            &[
                HANDLE_PARAM,
                POINTER,
                ("popup", Param::Boolean),
                ("uninterrupted", Param::Boolean),
            ],
            ACTIONS,
        ),
    ],
    ..CLASS
};
