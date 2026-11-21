//! `ShiftNative`: the shift key's rules, wrapped by `ShiftRules` (lib/rust's Shift.kt) and
//! answered by awnative's `shift` module.

use crate::{CLASS, Class, METHOD, Method, Param, Return};

pub const SHIFT_NATIVE: Class = Class {
    name: "ShiftNative",
    doc: "The shift key's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/shift.rs. [ShiftRules] wraps them.",
    module: "shift",
    methods: &[
        Method {
            name: "next",
            doc: "The state `event` leaves `state` in (`shift::next`). States, events, \
                  behaviours and caps modes cross as their ordinals; `facts` holds the bits \
                  `ShiftRules` packs.",
            params: &[
                ("state", Param::Int),
                ("event", Param::Int),
                ("behavior", Param::Int),
                ("capsMode", Param::Int),
                ("facts", Param::Int),
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
            ..METHOD
        },
        Method {
            name: "isDoubleSpacePeriod",
            doc: "Whether a second space after `text` types a full stop \
                  (`shift::is_double_space_period`).",
            params: &[("text", Param::String)],
            returns: Return::Boolean,
            ..METHOD
        },
    ],
    ..CLASS
};
