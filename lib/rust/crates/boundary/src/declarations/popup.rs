//! `PopupNative`, a key's long-press popup, answered by awnative's `popup` module, and
//! `PopupGrid` and `PopupOrder`, the Kotlin over it, with what they take and give back.
//! `PopupLayout`, which reads an extended popup's row by place, stays hand-written in Popup.kt.

use crate::{
    API, Api, Arg, ArgKind, CLASS, Class, Element, MEMBER, METHOD, Member, Method, Module, OBJECTS,
    Out, Param, Return, Type, arg,
};

const F: Param = Param::Float;
const I: Param = Param::Int;
const B: Param = Param::Boolean;

/// A method of `PopupNative`.
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

pub const POPUP_NATIVE: Class = Class {
    name: "PopupNative",
    doc: "A key's long-press popup's entry points in libawnative: lib/rust/crates/awnative/src/\
          popup.rs, which documents each and how results are packed. [PopupGrid] and \
          [PopupOrder] wrap them.",
    module: "popup",
    methods: &[
        method(
            "extend",
            &[
                ("count", I),
                ("prioritized", I),
                ("keyLeft", F),
                ("keyTop", F),
                ("keyRight", F),
                ("keyBottom", F),
                ("baseLeft", F),
                ("baseTop", F),
                ("baseRight", F),
                ("baseBottom", F),
                ("keyboardWidth", F),
            ],
            Return::NullableArray(Element::Int),
        ),
        method(
            "pointer",
            &[
                ("keyLeft", F),
                ("keyTop", F),
                ("keyRight", F),
                ("keyBottom", F),
                ("baseLeft", F),
                ("baseTop", F),
                ("baseRight", F),
                ("baseBottom", F),
                ("anchorLeft", B),
                ("anchorOffset", I),
                ("row0Count", I),
                ("row1Count", I),
                ("x", F),
                ("y", F),
            ],
            Return::Long,
        ),
        method(
            "elementAt",
            &[("sizes", Param::Array(Element::Int)), ("index", I)],
            Return::Long,
        ),
        method(
            "order",
            &[
                ("main", B),
                ("relevant", I),
                ("symbolHint", B),
                ("numberHint", B),
                ("symbolPopups", I),
                ("numberPopups", I),
                ("symbolMode", I),
                ("numberMode", I),
                ("mergeHintPopups", B),
            ],
            OBJECTS,
        ),
        method(
            "merge",
            &[
                ("computed", Param::Array(Element::Boolean)),
                ("mainComputed", B),
                ("useMain", B),
                ("hasMain", B),
            ],
            Return::Array(Element::Int),
        ),
    ],
    ..CLASS
};

pub const POPUP_RECT: (&str, Module, Type) = (
    "popup",
    Module::Rust,
    Type::Record {
        name: "PopupRect",
        doc: "A rectangle in the keyboard's coordinates, as upstream's `FlorisRect` holds it, for \
              crossing into Rust without the app's classes.",
        data: false,
        fields: &[
            ("left", "Float"),
            ("top", "Float"),
            ("right", "Float"),
            ("bottom", "Float"),
        ],
        constants: &[],
    },
);

/// The key's and its preview popup's rectangles, for the native method's sides.
const EXTEND: [Arg; 8] = [
    arg("keyLeft", ArgKind::As("key: PopupRect", "key.left")),
    arg("keyTop", ArgKind::Fixed("key.top")),
    arg("keyRight", ArgKind::Fixed("key.right")),
    arg("keyBottom", ArgKind::Fixed("key.bottom")),
    arg("baseLeft", ArgKind::As("base: PopupRect", "base.left")),
    arg("baseTop", ArgKind::Fixed("base.top")),
    arg("baseRight", ArgKind::Fixed("base.right")),
    arg("baseBottom", ArgKind::Fixed("base.bottom")),
];

pub const POPUP_GRID: Api = Api {
    name: "PopupGrid",
    doc: "The extended popup's grid and the key under the finger, in Rust \
          (lib/rust/crates/popup/src/grid.rs).",
    module: "popup",
    native: "PopupNative",
    constants: &[(
        "OFF_THE_POPUP",
        "",
        "Long.MIN_VALUE",
        "What [pointer] gives once the finger is off the popup.",
    )],
    members: &[
        Member {
            name: "extend",
            doc: "The extended popup of [count] keys, [prioritized] of them prioritized, for the \
                  key at [key] whose preview popup is [base], on a keyboard [keyboardWidth] wide; \
                  null if there are no keys.",
            args: &EXTEND,
            result: Out::NullableWrap("PopupLayout"),
            ..MEMBER
        },
        Member {
            name: "pointer",
            doc: "Which place of a popup laid out with these [anchorLeft], [anchorOffset], \
                  [row0Count] and [row1Count] the finger at ([x], [y]) is on, for the key at \
                  [keyLeft] to [keyBottom] whose preview popup is [baseLeft] to [baseBottom]; \
                  [OFF_THE_POPUP] once the finger has left it. Takes the sides as floats, not \
                  rectangles, as it runs on every move of the finger.",
            ..MEMBER
        },
        Member {
            name: "elementAt",
            doc: "Where the element numbered [index] is in rows of these [sizes], counting from \
                  the last row back: its row in the high half of the result and its place in the \
                  row in the low half; -1 if there is none.",
            ..MEMBER
        },
    ],
    ..API
};

pub const POPUP_ORDER: Api = Api {
    name: "PopupOrder",
    doc: "The order of a key's popup keys for the hint settings, in Rust \
          (lib/rust/crates/popup/src/order.rs).",
    module: "popup",
    native: "PopupNative",
    nested: &[
        Type::Record {
            name: "Order",
            doc: "The keys in order, numbered as above. Unless [ownList] is [NUMBERED], the other \
                  keys are that list of the set's itself, which keys merged later show in, and \
                  [other] is empty.",
            data: false,
            fields: &[
                ("hint", "Int"),
                ("prioritized", "IntArray"),
                ("other", "IntArray"),
                ("ownList", "Int"),
            ],
            constants: &[],
        },
        Type::Enum {
            name: "Mode",
            doc: "How a hint is offered, as `KeyHintMode` orders them.",
            entries: &[
                "DISABLED",
                "HINT_PRIORITY",
                "ACCENT_PRIORITY",
                "SMART_PRIORITY",
            ],
        },
    ],
    constants: &[
        (
            "MAIN",
            "",
            "0",
            "A set's keys, numbered: these three, then its relevant keys, the symbol hint's and \
             the number hint's popup keys.",
        ),
        ("SYMBOL_HINT", "", "1", ""),
        ("NUMBER_HINT", "", "2", ""),
        ("FIRST_LISTED", "", "3", ""),
        ("NO_HINT", "", "-1", "No hint, in [Order.hint]."),
        (
            "SET_MAIN",
            "",
            "-1",
            "In a merge's steps: the merged set's main key becomes the set's.",
        ),
        (
            "NUMBERED",
            "",
            "0",
            "In [Order.ownList]: the other keys are these keys of the set's, numbered, not one of \
             its own lists.",
        ),
        (
            "OWN_RELEVANT",
            "",
            "-1",
            "In [Order.ownList]: the other keys are the set's own list of relevant keys, symbol \
             popups or number popups.",
        ),
        ("OWN_SYMBOL_POPUPS", "", "-2", ""),
        ("OWN_NUMBER_POPUPS", "", "-3", ""),
    ],
    members: &[
        Member {
            name: "order",
            doc: "The order of a set's keys: whether it has a [main] key, a [symbolHint] and a \
                  [numberHint], and how many [relevant] keys and popup keys of each hint, for \
                  these settings.",
            args: &[
                arg("symbolMode", ArgKind::Ordinal("Mode")),
                arg("numberMode", ArgKind::Ordinal("Mode")),
            ],
            result: Out::Record("Order"),
            ..MEMBER
        },
        Member {
            name: "merge",
            doc: "Where a merged set's keys go: its relevant keys that [computed] and then its \
                  main key if it [mainComputed]. Each step is the index of a key to add (the \
                  number of relevant keys for its main) or [SET_MAIN], which the merge may do if \
                  it [useMain] and the set [hasMain] not.",
            ..MEMBER
        },
    ],
    ..API
};
