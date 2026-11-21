//! `ShiftNative`, the shift key's rules, answered by awnative's `shift` module, and `ShiftRules`,
//! the Kotlin over it.

use crate::{
    API, Api, Arg, ArgKind, CLASS, Class, MEMBER, METHOD, Member, Method, Out, Param, Return, Type,
    arg, default,
};

pub const SHIFT_NATIVE: Class = Class {
    name: "ShiftNative",
    doc: "The shift key's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/shift.rs. [ShiftRules] wraps them.",
    module: "shift",
    methods: &[
        Method {
            name: "next",
            doc: "The state `event` leaves `state` in (`shift::next`). States, events, \
                  behaviours and caps modes cross as their ordinals.",
            params: &[
                ("state", Param::Int),
                ("event", Param::Int),
                ("behavior", Param::Int),
                ("capsMode", Param::Int),
                ("shiftPressed", Param::Boolean),
                ("anyPressed", Param::Boolean),
                ("consecutiveDown", Param::Boolean),
                ("uninterrupted", Param::Boolean),
                ("autoCapitalization", Param::Boolean),
                ("localeCapitalizes", Param::Boolean),
            ],
            returns: Return::Int,
            ..METHOD
        },
        Method {
            name: "isConsecutive",
            doc: "Whether a key event follows the last within the timeout \
                  (`shift::is_consecutive`).",
            params: &[
                ("lastCode", Param::Int),
                ("lastTime", Param::Long),
                ("code", Param::Int),
                ("now", Param::Long),
                ("timeout", Param::Long),
            ],
            returns: Return::Boolean,
            rust: "::shift::is_consecutive",
            ..METHOD
        },
        Method {
            name: "isDoubleSpacePeriod",
            doc: "Whether a second space after `text` types a full stop \
                  (`shift::is_double_space_period`).",
            params: &[("text", Param::String)],
            returns: Return::Boolean,
            rust: "::shift::is_double_space_period",
            ..METHOD
        },
    ],
    ..CLASS
};

pub const SHIFT_RULES: Api = Api {
    name: "ShiftRules",
    doc: "The shift key and auto-capitalisation, in Rust (lib/rust/crates/shift): the shift \
          state each event leaves, whether a key's press or release is a double tap, and \
          whether a second space types a full stop.",
    module: "shift",
    native: "ShiftNative",
    nested: &[
        Type::Enum {
            name: "State",
            doc: "The keyboard's shift state, in `InputShiftState`'s order.",
            entries: &["UNSHIFTED", "SHIFTED_MANUAL", "SHIFTED_AUTOMATIC", "CAPS_LOCK"],
        },
        Type::Enum {
            name: "Event",
            doc: "What happened. [REEVALUATE] is the cursor or editor changing; [KEY_TYPED], a \
                  typing key's release.",
            entries: &[
                "REEVALUATE",
                "SHIFT_DOWN",
                "SHIFT_UP",
                "CAPS_LOCK",
                "SHIFT_CANCEL",
                "KEY_TYPED",
            ],
        },
        Type::Enum {
            name: "Behavior",
            doc: "How the shift key reaches caps lock, in `CapitalizationBehavior`'s order.",
            entries: &["CAPSLOCK_BY_DOUBLE_TAP", "CAPSLOCK_BY_CYCLE"],
        },
        Type::Enum {
            name: "CapsMode",
            doc: "What the editor capitalises at the cursor, in `InputAttributes.CapsMode`'s \
                  order.",
            entries: &["NONE", "ALL", "WORDS", "SENTENCES"],
        },
    ],
    members: &[
        Member {
            name: "next",
            doc: "The state [event] leaves [state] in, given what Kotlin knew: [shiftPressed] and \
                  [anyPressed] keys down, [consecutiveDown] (a shift press soon after the last), \
                  [uninterrupted] (shift was the last key down), and whether the user \
                  ([autoCapitalization]) and language ([localeCapitalizes]) want capitals. Each \
                  event reads only some of them.",
            args: &[
                arg("state", ArgKind::Ordinal("State")),
                arg("event", ArgKind::Ordinal("Event")),
                Arg {
                    default: "Behavior.CAPSLOCK_BY_DOUBLE_TAP",
                    ..arg("behavior", ArgKind::Ordinal("Behavior"))
                },
                Arg {
                    default: "CapsMode.NONE",
                    ..arg("capsMode", ArgKind::Ordinal("CapsMode"))
                },
                default("shiftPressed", "false"),
                default("anyPressed", "false"),
                default("consecutiveDown", "false"),
                default("uninterrupted", "false"),
                default("autoCapitalization", "false"),
                default("localeCapitalizes", "false"),
            ],
            result: Out::Entry("State"),
            ..MEMBER
        },
        Member {
            name: "isConsecutive",
            doc: "Whether a key [code] at [now] follows the last such event, [lastCode] at \
                  [lastTime], within [timeout] ms.",
            ..MEMBER
        },
        Member {
            name: "isDoubleSpacePeriod",
            doc: "Whether the two characters before the cursor, [text], are a space a second \
                  space turns into \". \".",
            ..MEMBER
        },
    ],
    ..API
};
