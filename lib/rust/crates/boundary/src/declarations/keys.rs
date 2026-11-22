//! `KeysNative`, the text keyboard's geometry, answered by awnative's `keys` module, and
//! `KeyGeometry`, the Kotlin over it.

use crate::{
    API, Api, CLASS, Class, Element, MEMBER, METHOD, Member, Method, Out, Param, Return, Type,
};

const FLOATS: Param = Param::Array(Element::Float);
const ROW_SIZES: (&str, Param) = ("rowSizes", Param::Array(Element::Int));
const KEYBOARD: [(&str, Param); 2] = [
    ("keyboardWidth", Param::Float),
    ("keyboardHeight", Param::Float),
];

pub const KEYS_NATIVE: Class = Class {
    name: "KeysNative",
    doc: "The text keyboard's geometry's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/keys.rs, which documents how rects are packed into arrays. \
          [KeyGeometry] wraps them.",
    module: "keys",
    methods: &[
        Method {
            name: "flex",
            params: &[
                ("mode", Param::Int),
                ("code", Param::Int),
                ("numeric", Param::Boolean),
            ],
            returns: Return::Array(Element::Float),
            ..METHOD
        },
        Method {
            name: "desiredKey",
            params: &[
                ("mode", Param::Int),
                KEYBOARD[0],
                KEYBOARD[1],
                ("rowCount", Param::Int),
                ("rowBaseHeight", Param::Float),
                ("keyMarginH", Param::Float),
                ("keyMarginV", Param::Float),
            ],
            returns: Return::Array(Element::Float),
            ..METHOD
        },
        Method {
            name: "layout",
            params: &[
                ROW_SIZES,
                ("flex", FLOATS),
                ("desired", FLOATS),
                KEYBOARD[0],
                KEYBOARD[1],
                ("extendDown", Param::Boolean),
            ],
            returns: Return::NullableArray(Element::Float),
            ..METHOD
        },
        Method {
            name: "keyAt",
            params: &[
                ROW_SIZES,
                ("touchBounds", FLOATS),
                ("x", Param::Float),
                ("y", Param::Float),
            ],
            returns: Return::Int,
            ..METHOD
        },
    ],
    ..CLASS
};

pub const KEY_GEOMETRY: Api = Api {
    name: "KeyGeometry",
    doc: "The text keyboard's geometry, in Rust (lib/rust/crates/keys): how wide each key wants to \
          be, the desired key a keyboard is laid out with, where each key goes, and which key a \
          touch lands on. Keyboard modes are `KeyboardMode.value`s. A rect is four Floats (left, \
          top, right, bottom), and a keyboard's keys are one flat array in row order with \
          [rowSizes][layout] beside it.",
    module: "keys",
    native: "KeysNative",
    nested: &[Type::Record {
        name: "Flex",
        doc: "A visible key's flex factors: how much it shrinks and grows, and how wide it wants \
              to be.",
        data: false,
        constants: &[],
        fields: &[
            ("shrink", "Float"),
            ("grow", "Float"),
            ("widthFactor", "Float"),
        ],
    }],
    members: &[
        Member {
            name: "flex",
            doc: "The factors of a visible key with this [code] on a keyboard of this [mode]; \
                  [numeric] if its type is.",
            result: Out::Record("Flex"),
            ..MEMBER
        },
        Member {
            name: "desiredKey",
            doc: "The desired key's touch bounds and then its visible bounds, eight Floats: a \
                  tenth of the keyboard wide, a row high (at most 1.12 base rows, for character \
                  and symbol keyboards) and the visible bounds inside the margins.",
            ..MEMBER
        },
        Member {
            name: "layout",
            doc: "Lays out rows of [rowSizes] keys, whose [flex] is three Floats each (width \
                  factor, shrink, grow), with the [desired] key's touch and visible bounds: each \
                  key's touch and visible bounds, eight Floats a key, or null if nothing is to be \
                  laid out and each key keeps its bounds.",
            ..MEMBER
        },
        Member {
            name: "keyAt",
            doc: "The index of the first key whose [touchBounds] (four Floats a key) hold ([x], \
                  [y]), or -1. The search stops at the first row without keys.",
            ..MEMBER
        },
    ],
    ..API
};
