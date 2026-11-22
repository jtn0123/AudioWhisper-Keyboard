//! `SuggestNative`, the typing suggestions' rules, answered by awnative's `suggest` module and its
//! `tap` and `tap_aim` modules, and `SuggestRules`, `QwertyLayout` and `TapPositions`, the Kotlin
//! over it.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, HANDLE, Handle, Kind, MEMBER, METHOD, Member, Method,
    NULLABLE_OBJECTS, Out, Param, Return, Shape, Type, arg, default,
};

const S: Param = Param::String;
const I: Param = Param::Int;
const F: Param = Param::Float;
const B: Param = Param::Boolean;
const TRAIL: (&str, Param) = ("trail", Param::Long);
const FLOATS: Return = Return::Array(Element::Float);

/// A method of `SuggestNative` answered in awnative's `suggest` module.
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

/// A method answered by `rust`, a function of the `tap` or `tap_aim` module.
const fn tap(
    name: &'static str,
    rust: &'static str,
    params: &'static [(&'static str, Param)],
    returns: Return,
) -> Method {
    Method {
        rust,
        ..method(name, params, returns)
    }
}

pub const SUGGEST_NATIVE: Class = Class {
    name: "SuggestNative",
    doc: "The typing suggestions' entry points in libawnative: lib/rust/crates/awnative/src/\
          suggest.rs, suggest/tap.rs and suggest/tap_aim.rs, which document each. \
          [SuggestRules], [QwertyLayout] and [TapPositions] wrap them.",
    module: "suggest",
    methods: &[
        // The rules, in suggest.rs.
        method("continuesWord", &[("codePoint", I)], Return::Boolean),
        method("wordCase", &[("typed", S)], Return::Int),
        method("applyCase", &[("case", I), ("word", S)], Return::String),
        method("isMostlyCaps", &[("typed", S)], Return::Boolean),
        method("decoderInput", &[("typed", S)], Return::String),
        method("maxPreviousWords", &[], Return::Int),
        method(
            "previousSlotCount",
            &[("wordCount", I), ("sentenceStart", B)],
            Return::Int,
        ),
        method(
            "isCursorMidWord",
            &[("textAfterCursor", S)],
            Return::Boolean,
        ),
        method("previousWords", &[("textBeforeWord", S)], Return::Strings),
        method("modestThreshold", &[], Return::Float),
        method("defaultStrength", &[], Return::Int),
        method("strengthNames", &[], Return::Strings),
        method(
            "exceedsThreshold",
            &[
                ("word", S),
                ("score", I),
                ("kindAndFlags", I),
                ("typed", S),
                ("threshold", F),
            ],
            Return::Boolean,
        ),
        method(
            "normalizedScore",
            &[("before", S), ("after", S), ("score", I)],
            Return::Float,
        ),
        method(
            "editDistance",
            &[
                ("before", Param::Array(Element::Int)),
                ("after", Param::Array(Element::Int)),
            ],
            Return::Int,
        ),
        method(
            "revertAt",
            &[
                ("typed", S),
                ("corrected", S),
                ("start", I),
                ("cursor", I),
                ("textBeforeCursor", S),
            ],
            NULLABLE_OBJECTS,
        ),
        // Where keys were tapped and the QWERTY layout, in suggest/tap.rs.
        tap("tapLimit", "tap::limit", &[], Return::Float),
        tap("tapTrailCapacity", "tap::capacity", &[], Return::Int),
        tap(
            "tapOffset",
            "tap::offset",
            &[
                ("x", F),
                ("y", F),
                ("centerX", F),
                ("centerY", F),
                ("width", F),
                ("height", F),
            ],
            Return::NullableArray(Element::Float),
        ),
        tap(
            "newTapTrail",
            "tap::new_trail",
            &[("capacity", I)],
            Return::Long,
        ),
        tap("freeTapTrail", "tap::free_trail", &[TRAIL], Return::Unit),
        tap(
            "tapPressed",
            "tap::pressed",
            &[
                TRAIL,
                ("codePoint", I),
                ("hasOffset", B),
                ("dx", F),
                ("dy", F),
            ],
            Return::Unit,
        ),
        tap(
            "tapTyped",
            "tap::typed",
            &[TRAIL, ("text", S)],
            Return::Unit,
        ),
        tap("tapDeleted", "tap::deleted", &[TRAIL], Return::Unit),
        tap("tapCleared", "tap::cleared", &[TRAIL], Return::Unit),
        tap(
            "tapOffsetsFor",
            "tap::offsets_for",
            &[TRAIL, ("word", S)],
            FLOATS,
        ),
        tap(
            "qwertySizes",
            "tap::qwerty_sizes",
            &[],
            Return::Array(Element::Int),
        ),
        tap(
            "qwertyKeys",
            "tap::qwerty_keys",
            &[],
            Return::Array(Element::Int),
        ),
        tap(
            "keyCenter",
            "tap::key_center",
            &[("codePoint", I)],
            Return::NullableArray(Element::Int),
        ),
        tap(
            "touchPoint",
            "tap::touch_point",
            &[("codePoint", I), ("hasOffset", B), ("dx", F), ("dy", F)],
            Return::NullableArray(Element::Int),
        ),
        tap(
            "proximityGrid",
            "tap::proximity_grid",
            &[],
            Return::Array(Element::Int),
        ),
        // The learned tap aim, in suggest/tap_aim.rs.
        tap(
            "tapLandscape",
            "tap_aim::landscape",
            &[TRAIL, ("landscape", B)],
            Return::Unit,
        ),
        tap(
            "tapKept",
            "tap_aim::kept",
            &[TRAIL, ("word", S)],
            Return::Unit,
        ),
        tap(
            "tapEvent",
            "tap_aim::event",
            &[TRAIL, ("event", I)],
            Return::Unit,
        ),
        tap(
            "tapAimedOffsetsFor",
            "tap_aim::aimed_offsets_for",
            &[TRAIL, ("word", S), ("aimed", B)],
            FLOATS,
        ),
        tap(
            "tapRetarget",
            "tap_aim::retarget",
            &[
                TRAIL,
                ("layout", Param::Array(Element::Int)),
                ("bounds", Param::Array(Element::Float)),
                ("x", F),
                ("y", F),
                ("aimed", B),
            ],
            Return::Int,
        ),
        tap("tapAimText", "tap_aim::text", &[TRAIL], Return::String),
        tap(
            "tapAimLoad",
            "tap_aim::load",
            &[TRAIL, ("text", S)],
            Return::Unit,
        ),
        tap("tapAimForget", "tap_aim::forget", &[TRAIL], Return::Unit),
        tap(
            "tapAimChanged",
            "tap_aim::changed",
            &[TRAIL],
            Return::Boolean,
        ),
    ],
    ..CLASS
};

/// A `val` of `SuggestRules` or `QwertyLayout`, read once.
const fn value(name: &'static str, doc: &'static str) -> Member {
    Member {
        name,
        doc,
        kind: Kind::Val,
        ..MEMBER
    }
}

/// A function that passes its arguments as they are.
const fn same(name: &'static str, doc: &'static str) -> Member {
    Member {
        name,
        doc,
        ..MEMBER
    }
}

pub const SUGGEST_RULES: Api = Api {
    name: "SuggestRules",
    doc: "The typing suggestions' rules, in Rust (lib/rust/crates/suggest): how a typed word is \
          capitalized, the words before it, whether a correction is good enough to make by \
          itself, how to undo one, and where a tap lands on the decoder's QWERTY layout. The \
          keyboard's `suggest` package calls these; its classes document the rules. Text is cased \
          as Locale.US cases it.",
    module: "suggest",
    native: "SuggestNative",
    nested: &[
        Type::Enum {
            name: "Case",
            doc: "How a typed word is capitalized, as the keyboard's `WordCase` names them.",
            entries: &["AS_TYPED", "CAPITALIZED", "ALL_CAPS"],
        },
        Type::Record {
            name: "Context",
            doc: "Whether a sentence starts before a word, and the previous words before it, most \
                  recent first.",
            data: false,
            fields: &[
                ("isBeginningOfSentence", "Boolean"),
                ("words", "List<String>"),
            ],
            constants: &[],
        },
        Type::Record {
            name: "Revert",
            doc: "Replace the [length] characters left of the cursor with [text].",
            data: false,
            fields: &[
                ("length", "Int"),
                ("text", "String"),
                ("phantomSpace", "Boolean"),
            ],
            constants: &[],
        },
    ],
    members: &[
        value(
            "modestThreshold",
            "AOSP's \"modest\" auto-correction threshold.",
        ),
        Member {
            result: Out::List,
            ..value(
                "strengthNames",
                "The auto-correction strengths' names, in ordinal order: how \
                 [LatinSession.suggest] takes one.",
            )
        },
        value(
            "defaultStrength",
            "The default strength's ordinal: what the keyboard always did.",
        ),
        value("maxPreviousWords", "The most words a [Context] holds."),
        same("previousSlotCount", ""),
        same("isCursorMidWord", ""),
        value(
            "tapLimit",
            "How far beside a key, in key sizes from its centre, a tap still counts for it.",
        ),
        value(
            "tapTrailCapacity",
            "How many characters a [TapPositions] keeps unless told otherwise.",
        ),
        same(
            "continuesWord",
            "Whether typing [codePoint] right after a word continues the word.",
        ),
        Member {
            call: "wordCase",
            result: Out::Entry("Case"),
            ..same("case", "How [typed] is capitalized.")
        },
        Member {
            args: &[arg("case", ArgKind::Ordinal("Case"))],
            ..same("applyCase", "[word] capitalized as [case] says.")
        },
        same("isMostlyCaps", "Whether [typed] has more than one capital."),
        same("decoderInput", "[typed] as the decoder is given it."),
        Member {
            result: Out::Record("Context"),
            ..same(
                "previousWords",
                "The context for a word that starts right after [textBeforeWord].",
            )
        },
        same(
            "exceedsThreshold",
            "Whether auto-correcting [typed] to the decoder's [word], with its [score] and \
             [kindAndFlags], passes [threshold].",
        ),
        same(
            "normalizedScore",
            "The decoder's [score] for [after], weighted down by how far it is from [before].",
        ),
        same(
            "editDistance",
            "The edits, by code point, from [before] to [after].",
        ),
        Member {
            result: Out::Record("Revert"),
            ..same(
                "revertAt",
                "How to undo auto-correcting [typed] to [corrected] at [start], with the cursor \
                 at [cursor] after [textBeforeCursor], or null if that is no longer the \
                 correction and its separator.",
            )
        },
        same(
            "tapOffset",
            "Where a tap at ([x], [y]) landed on a key centred at ([centerX], [centerY]): dx and \
             dy in key sizes, or null for a key with no size.",
        ),
    ],
    ..API
};

/// One of the QWERTY layout's sizes, the one at `at` of `qwertySizes`.
const fn size(name: &'static str, then: &'static str) -> Member {
    Member {
        call: "qwertySizes",
        result: Out::Then("Int", then),
        ..value(name, "")
    }
}

pub const QWERTY_LAYOUT: Api = Api {
    name: "QwertyLayout",
    doc: "The fixed QWERTY layout the decoder's spatial model sees, from Rust.",
    module: "suggest",
    native: "SuggestNative",
    members: &[
        size("keyWidth", "[0]"),
        size("keyHeight", "[1]"),
        size("keyboardWidth", "[2]"),
        size("keyboardHeight", "[3]"),
        size("gridWidth", "[4]"),
        size("gridHeight", "[5]"),
        size("maxProximityChars", "[6]"),
        Member {
            call: "qwertyKeys",
            ..same(
                "keys",
                "The keys, three values each: code point, left and top.",
            )
        },
        same(
            "keyCenter",
            "The centre (x, y) of the key that types [codePoint] in either case, or null.",
        ),
        same(
            "touchPoint",
            "Where (x, y) a tap on the key for [codePoint] lands: at ([dx], [dy]) from its centre \
             if [hasOffset], else at the centre. Null if no key types it.",
        ),
        same(
            "proximityGrid",
            "The code points near each grid cell, [maxProximityChars] per cell, -1 padded.",
        ),
    ],
    ..API
};

/// A member of `TapPositions` calling `call`.
const fn trail(name: &'static str, call: &'static str, doc: &'static str) -> Member {
    Member {
        call,
        ..same(name, doc)
    }
}

pub const TAP_POSITIONS: Api = Api {
    name: "TapPositions",
    doc: "Where each of the last [capacity] typed characters was tapped, in Rust. Safe to use \
          from any thread. Its native memory is freed once nothing can reach it.",
    module: "suggest",
    native: "SuggestNative",
    shape: Shape::Handle(Handle {
        params: &[default("capacity", "SuggestRules.tapTrailCapacity")],
        create: "newTapTrail",
        free: "freeTapTrail",
        ..HANDLE
    }),
    constants: &[
        ("BACKSPACE", "", "0", ""),
        ("UNDO", "", "1", ""),
        ("PICKED", "", "2", ""),
        ("PICKED_FOR_WORD", "", "3", ""),
        ("SWIPED", "", "4", ""),
    ],
    members: &[
        trail(
            "pressed",
            "tapPressed",
            "A key for [codePoint] went down, at ([dx], [dy]) from its centre if [hasOffset].",
        ),
        trail("typed", "tapTyped", "[text] went into the editor."),
        trail(
            "deleted",
            "tapDeleted",
            "The last typed character was deleted.",
        ),
        trail("clear", "tapCleared", "Everything typed is forgotten."),
        trail(
            "offsetsFor",
            "tapOffsetsFor",
            "Where each character of [word] was tapped: three values per character, 1 if it was \
             tapped and 0 if not, then dx and dy. Empty if the last characters typed don't spell \
             [word].",
        ),
        trail(
            "landscape",
            "tapLandscape",
            "The screen is in landscape or not: where this user taps is learned and used per \
             orientation.",
        ),
        trail(
            "kept",
            "tapKept",
            "[word] was committed as typed: it may teach where this user taps, once nothing takes \
             it back.",
        ),
        trail(
            "event",
            "tapEvent",
            "One of the [BACKSPACE], [UNDO], [PICKED], [PICKED_FOR_WORD] or [SWIPED] events.",
        ),
        trail(
            "offsetsFor",
            "tapAimedOffsetsFor",
            "[offsetsFor], each offset moved by where this user is known to tap that key if \
             [aimed].",
        ),
        trail(
            "retarget",
            "tapRetarget",
            "Which key a tap at ([x], [y]) types, given which letters come next in the word being \
             typed and, if [aimed], where this user is known to tap: [layout] is the key the \
             plain hit test found, the row count, each row's size, then each key's code; [bounds] \
             is six values per key, its touch bounds (left, top, right, bottom) then its drawn \
             width and height. -1 if they don't fit together.",
        ),
        trail(
            "aimText",
            "tapAimText",
            "Where this user taps, as saved: numbers only.",
        ),
        trail(
            "loadAim",
            "tapAimLoad",
            "Replaces where this user taps with the saved [text].",
        ),
        trail("forgetAim", "tapAimForget", "Forgets where this user taps."),
        trail(
            "aimChanged",
            "tapAimChanged",
            "Whether where this user taps changed since this was last asked.",
        ),
    ],
    ..API
};
