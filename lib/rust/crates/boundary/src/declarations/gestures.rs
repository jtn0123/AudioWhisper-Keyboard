//! `GesturesNative`, swipes and the start of a glide, answered by awnative's `gestures` module,
//! and `SwipeDetector` and `GlideDetector`, the Kotlin over it, with what they give back.

use crate::{
    API, Api, ArgKind, BIT, Bit, BitKind, CLASS, Class, Element, HANDLE, Handle, MEMBER, METHOD,
    Member, Method, Module, Out, Param, Return, Shape, Type, arg,
};

const SWIPES: (&str, Param) = ("swipes", Param::Long);
const ID: (&str, Param) = ("id", Param::Int);
const X: (&str, Param) = ("x", Param::Float);
const Y: (&str, Param) = ("y", Param::Float);
const DENSITY: (&str, Param) = ("density", Param::Float);
const DISTANCE: (&str, Param) = ("distanceThreshold", Param::Int);
const SWIPE: Return = Return::NullableArray(Element::Int);

/// A method of `GesturesNative`.
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

pub const GESTURES_NATIVE: Class = Class {
    name: "GesturesNative",
    doc: "The touch gestures' entry points in libawnative: lib/rust/crates/awnative/src/\
          gestures.rs, which documents each and how a swipe and a glide's outcome are packed. \
          [SwipeDetector] and [GlideDetector] wrap them.",
    module: "gestures",
    methods: &[
        method("newSwipes", &[], Return::Long),
        method("freeSwipes", &[SWIPES], Return::Unit),
        method("swipesReset", &[SWIPES], Return::Unit),
        method("swipesDown", &[SWIPES, ID, X, Y, DENSITY], Return::Unit),
        method(
            "swipesMove",
            &[
                SWIPES,
                ID,
                X,
                Y,
                DENSITY,
                DISTANCE,
                ("always", Param::Boolean),
            ],
            SWIPE,
        ),
        method(
            "swipesUp",
            &[
                SWIPES,
                ID,
                X,
                Y,
                ("velocityX", Param::Float),
                ("velocityY", Param::Float),
                DENSITY,
                DISTANCE,
                ("velocityThreshold", Param::Int),
            ],
            SWIPE,
        ),
        method("swipesCancel", &[SWIPES, ID], Return::Unit),
        method("newGlideStart", &[("keySize", Param::Float)], Return::Long),
        method("freeGlideStart", &[("glide", Param::Long)], Return::Unit),
        method(
            "glideTouch",
            &[
                ("glide", Param::Long),
                ("action", Param::Int),
                ID,
                ("points", Param::Array(Element::Float)),
                ("count", Param::Int),
                ("now", Param::Long),
                DENSITY,
                ("hasKey", Param::Boolean),
                ("keyCode", Param::Int),
            ],
            Return::Long,
        ),
    ],
    ..CLASS
};

pub const SWIPE_DIRECTION: (&str, Module, Type) = (
    "gestures",
    Module::Rust,
    Type::Enum {
        name: "SwipeDirection",
        doc: "The direction of a swipe: eighths of the circle, clockwise from up-left.",
        entries: &[
            "UP_LEFT",
            "UP",
            "UP_RIGHT",
            "RIGHT",
            "DOWN_RIGHT",
            "DOWN",
            "DOWN_LEFT",
            "LEFT",
        ],
    },
);

pub const SWIPE_RECORD: (&str, Module, Type) = (
    "gestures",
    Module::Rust,
    Type::Record {
        name: "Swipe",
        doc: "A swipe: its [direction], and how many units (a quarter of the swipe distance \
              each) the finger moved since it went down ([absX], [absY]) and since the last \
              swipe reported ([relX], [relY]).",
        data: true,
        fields: &[
            ("direction", "SwipeDirection"),
            ("absX", "Int"),
            ("absY", "Int"),
            ("relX", "Int"),
            ("relY", "Int"),
        ],
        constants: &[],
    },
);

/// A flag of `GlideOutcome` at bit `shift`.
const fn flag(name: &'static str, shift: u32) -> Bit {
    Bit { name, shift, ..BIT }
}

pub const GLIDE_OUTCOME: (&str, Module, Type) = (
    "gestures",
    Module::Rust,
    Type::Bits {
        name: "GlideOutcome",
        doc: "What a touch event made of a glide, in the order to act on it: clear the points \
              kept if [clearedBefore], add the event's if [took] (and start the clock if \
              [started]), hand the kept points from [emitFrom] on to the listeners, then report \
              the glide [complete] or [cancelled], and clear the points if [clearedAfter]. \
              [gesture] is whether the event is part of a glide, and [decision] whether the \
              touch is one, once decided.",
        long: true,
        internal: false,
        fields: &[
            flag("gesture", 0),
            flag("clearedBefore", 1),
            flag("took", 2),
            flag("started", 3),
            flag("complete", 4),
            flag("cancelled", 5),
            flag("clearedAfter", 6),
            Bit {
                kind: BitKind::Maybe,
                ..flag("decision", 8)
            },
            Bit {
                doc: "The first kept point to hand on, or -1 for none.",
                kind: BitKind::Signed,
                ..flag("emitFrom", 32)
            },
        ],
    },
);

pub const SWIPE_DETECTOR: Api = Api {
    name: "SwipeDetector",
    doc: "Finds swipes, in Rust (lib/rust/crates/gestures/src/swipe.rs): up to four fingers at \
          once, each from where it went down. Positions and velocities are in pixels, with the \
          screen's density; the swipe distance is in dp and the velocity in dp a second, as the \
          gesture preferences hold them. Not for more than one thread at a time. Its native \
          memory is freed once nothing can reach it.",
    module: "gestures",
    native: "GesturesNative",
    shape: Shape::Handle(Handle {
        create: "newSwipes",
        free: "freeSwipes",
        ..HANDLE
    }),
    members: &[
        Member {
            name: "reset",
            doc: "A new touch began: forget every finger.",
            call: "swipesReset",
            ..MEMBER
        },
        Member {
            name: "down",
            doc: "Finger [id] went down at ([x], [y]). A fifth finger is ignored.",
            call: "swipesDown",
            ..MEMBER
        },
        Member {
            name: "move",
            doc: "Finger [id] moved to ([x], [y]): a swipe once it has moved more than half \
                  [distanceThreshold] along either axis since the last one, or at every move if \
                  [always].",
            call: "swipesMove",
            result: Out::Record("Swipe"),
            ..MEMBER
        },
        Member {
            name: "up",
            doc: "Finger [id] was lifted at ([x], [y]), moving at ([velocityX], [velocityY]): a \
                  swipe if it went further than [distanceThreshold] and faster than \
                  [velocityThreshold] along either axis.",
            call: "swipesUp",
            result: Out::Record("Swipe"),
            ..MEMBER
        },
        Member {
            name: "cancel",
            doc: "Finger [id]'s touch was cancelled: forget it.",
            call: "swipesCancel",
            ..MEMBER
        },
    ],
    ..API
};

pub const GLIDE_DETECTOR: Api = Api {
    name: "GlideDetector",
    doc: "Decides whether a touch is the start of a glide across keys [keySize] dp wide, in Rust \
          (lib/rust/crates/gestures/src/glide_start.rs). Not for more than one thread at a time. \
          Its native memory is freed once nothing can reach it.",
    module: "gestures",
    native: "GesturesNative",
    shape: Shape::Handle(Handle {
        create: "newGlideStart",
        free: "freeGlideStart",
        ..HANDLE
    }),
    members: &[Member {
        name: "touch",
        doc: "A touch event at [now] milliseconds: its masked [action], the pointer [id] at its \
              action index, and that pointer's first [count] floats of [points] (x, y, … in \
              pixels): where it went down, or its history then where it is now. The touch began \
              on the key with [initialKeyCode], if any.",
        call: "glideTouch",
        args: &[
            arg("hasKey", ArgKind::Fixed("initialKeyCode != null")),
            arg(
                "keyCode",
                ArgKind::As("initialKeyCode: Int?", "initialKeyCode ?: 0"),
            ),
        ],
        result: Out::Wrap("GlideOutcome"),
        ..MEMBER
    }],
    ..API
};
