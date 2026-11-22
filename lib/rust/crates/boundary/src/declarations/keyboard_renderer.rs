//! `KeyboardRenderer`, the Kotlin over `RenderNative` (render.rs).

use crate::{API, Api, Arg, ArgKind, HANDLE, Handle, MEMBER, Member, Shape, arg, default};

/// A member with only a doc, calling its method as it is.
const fn member(name: &'static str, doc: &'static str) -> Member {
    Member {
        name,
        doc,
        ..MEMBER
    }
}

/// A member whose `args` differ from its method's.
const fn with(name: &'static str, doc: &'static str, args: &'static [Arg]) -> Member {
    Member {
        name,
        doc,
        args,
        ..MEMBER
    }
}

pub const KEYBOARD_RENDERER: Api = Api {
    name: "KeyboardRenderer",
    doc: "The Rust renderer: the keyboard's scene, its panels and the settings screens, drawn \
          from plain packets. It has no Android dependency, so JVM tests use it too. Closing it \
          frees the renderer; a call after that throws.",
    module: "keyboard_renderer",
    native: "RenderNative",
    shape: Shape::Handle(Handle {
        params: &[arg("fontPaths", ArgKind::Same)],
        create: "create",
        free: "close",
        close: true,
        synchronized: true,
        ..HANDLE
    }),
    members: &[
        with(
            "settings",
            "The settings screens' reply to [operation] with its [input].",
            &[
                arg("operation", ArgKind::Code("SettingsUiOp")),
                default("input", "emptyArray()"),
            ],
        ),
        member(
            "settingsTouch",
            "The settings screens' reply to a MotionEvent, as [metadata], [times] and [coords].",
        ),
        member(
            "settingsImage",
            "The settings picture [name], [width] by [height] ARGB [pixels] at its intrinsic \
             size; 0×0 removes it. Gives back the screens' reply.",
        ),
        member(
            "theme",
            "The stylesheet [json], with the palette's [names] and [colors] and the [scale].",
        ),
        member(
            "update",
            "The keyboard's keys and candidates: its [dimensions], [metadata], [candidateFlags], \
             each key's [geometry] and [labels], and the [candidates].",
        ),
        member(
            "decorations",
            "The decorations' [labels] and [flags], [height] pixels tall.",
        ),
        with(
            "chrome",
            "A chrome view, sized by [config], with its [labels] and [icons], for the keyboard \
             mode and shift state in [metadata].",
            &[default(
                "metadata",
                "arrayOf(\"characters\", \"unshifted\")",
            )],
        ),
        member(
            "keyPreview",
            "A settings preview of the key [code] with its [labels], as ARGB pixels.",
        ),
        member(
            "chromeInfo",
            "The chrome view's field, six floats (`ime_chrome::Chrome::field`).",
        ),
        member("pixels", "The chrome view, drawn, as ARGB pixels."),
        with(
            "subtypes",
            "The subtype panel: the [selected] one of [ids], with its [labels], above a \
             [bottomInset].",
            &[default("bottomInset", "0f")],
        ),
        member(
            "search",
            "The search strip's [labels] at [density], as of [now].",
        ),
        member("gifSent", "The GIF being sent has gone."),
        member(
            "gifs",
            "The GIF panel: [config], [labels], its [chips] and the results' [ids].",
        ),
        member(
            "mediaRepeat",
            "Media [id]'s repeat count, as its decoder found it.",
        ),
        member("mediaRequests", "The media the panels want decoded."),
        member(
            "mediaUpload",
            "Frame [index] of media [id], [width] by [height] ARGB [pixels] shown for [delay], \
             the [last] if so; whether it was taken.",
        ),
        member(
            "clipboard",
            "The clipboard panel: its [config], [records] and [labels].",
        ),
        member(
            "emoji",
            "The emoji panel: its [config], [groups], [values], [history] and [labels].",
        ),
        member("panels", "The other panels: [config], [data] and [labels]."),
        member(
            "panelArrangement",
            "The smartbar arrangement being edited: the sticky slot, the dynamic slots and the \
             hidden ones.",
        ),
        member(
            "smartbar",
            "The smartbar: its [config], the actions' [codes] and their [labels].",
        ),
        member(
            "semanticConfig",
            "What accessibility reads: the resources' [names] and [values], the actions' [text], \
             which keys have [popups], which candidates are [removable], the [checked] codes and \
             the [editor] options.",
        ),
        member("semanticSnapshot", "The accessibility tree, as JSON."),
        member(
            "semanticAction",
            "Accessibility [action] on node [id]: the actions it makes.",
        ),
        member(
            "semanticCancel",
            "Accessibility dismissed what was open: the actions that makes.",
        ),
        with(
            "touch",
            "A touch: the actions it makes. With [trail], a key going down types the key this \
             user meant (see [TapPositions.retarget]): by which letters come next, and by where \
             they tap if [aimed].",
            &[
                Arg {
                    name: "trail",
                    kind: ArgKind::NullableHandle("TapPositions"),
                    default: "null",
                },
                default("aimed", "false"),
            ],
        ),
        member(
            "tooltipShown",
            "The platform showed, or hid, the tooltip for [pointer].",
        ),
        member(
            "candidateBindings",
            "The candidates' platform handles, [tokens]; gives back the smartbar's tokens.",
        ),
        member("gestures", "The gesture settings: [values] and [actions]."),
        member(
            "longPress",
            "Finger [pointer] held: whether it opens a [popup], and whether it was \
             [uninterrupted]. Gives the actions it makes.",
        ),
        member("tick", "The clock reached [time]: the actions that makes."),
        member(
            "nextDeadline",
            "When to call [tick] next, on the MotionEvent uptime clock, or -1 for never.",
        ),
        member(
            "barSettings",
            "The long-press [delay] and the smartbar slots that [changed]: the actions that \
             makes.",
        ),
        member("candidateLongPress", "Finger [pointer] held a candidate."),
        member(
            "popupGrid",
            "The open popup's grid: [prioritized] keys first, laid out for [landscape] if so.",
        ),
        member(
            "headroom",
            "The rows the surface keeps above the keyboard so that a key's popup is drawn whole, \
             with popups laid out for [landscape] if so. The surface is that much taller than \
             the scene and draws the scene that far down.",
        ),
        member(
            "popup",
            "A popup for key [key] with these [labels] and [codes].",
        ),
        with(
            "attach",
            "Draw to [surface], an Android `Surface`. [timeline]: present through a \
             SurfaceControl whose frames carry their vsync id (CPU path only).",
            &[default("hybrid", "false"), default("timeline", "false")],
        ),
        with(
            "frame",
            "Draw the frame for [time]; whether it is still animating. [vsync]: the Choreographer \
             vsync id this frame is timed by, or -1.",
            &[default("vsync", "-1")],
        ),
        member("detach", "Stop drawing to the surface."),
    ],
    ..API
};
