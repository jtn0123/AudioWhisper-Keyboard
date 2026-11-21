//! The input dispatcher's codes, which Kotlin sees as `InputProtocol`. `input::protocol`
//! re-exports them for the crates that route keys.

crate::declare! {
    /// The operations `InputDispatch.call` takes.
    pub const CALLS: object InputProtocol(i32) {
        INIT = 0,
        DOWN = 1,
        UP = 2,
        DOWN_UP = 3,
        CANCEL = 4,
        HOLD_RESULT = 5,
        REPEAT_RESULT = 6,
        FACTS = 7,
        ACK_DOWN = 8,
        ACK_UP = 9,
        DELAYS = 10,
    }
}

crate::declare! {
    /// The effects the dispatcher and its plans hand back: a code and its argument.
    pub const EFFECTS: object InputProtocol(i32) {
        EMIT_DOWN = 100,
        EMIT_UP = 101,
        EMIT_CANCEL = 102,
        EMIT_DOWN_UP = 103,
        EMIT_REPEAT = 104,
        STATE = 105,
        MASS_BEGIN = 106,
        MASS_END = 107,
        ARROW = 108,
        STOP_READING = 109,
        MIC_DOWN = 110,
        MIC_UP = 111,
        STOP_DICTATION = 112,
        READ_DOWN = 113,
        READ_UP = 114,
        READ_CANCEL = 115,
        CLEAR_UNDO = 116,
        FORGET_DICTATION = 117,
        GIF_STOP = 119,
        CUT = 120,
        COPY = 121,
        PASTE = 122,
        SELECT = 123,
        SELECT_ALL = 124,
        CLEAR_HISTORY = 125,
        CLEAR_FULL_HISTORY = 126,
        CLEAR_CLIP = 127,
        ONE_HANDED = 128,
        DELETE_FORWARD = 129,
        DELETE_BACK = 130,
        ENTER_KEY = 131,
        SHOW_UI = 132,
        HIDE_UI = 133,
        SUBTYPE = 134,
        NEXT_IME = 135,
        PREV_IME = 136,
        UNDO_KEY = 137,
        REDO_KEY = 138,
        SETTINGS_UI = 139,
        SPACE_KEY = 140,
        IME_PICKER = 141,
        SMARTBAR = 142,
        INCOGNITO = 143,
        AUTOCORRECT_TOGGLE = 144,
        AUTO_CORRECT = 145,
        COMMIT_TEXT = 146,
        TAP_TYPED = 147,
        COMMIT_CHAR = 148,
        UNKNOWN_KEY = 149,
        FEEDBACK_REPEAT = 150,
        DELETE_DROP = 300,
        DELETE_UNDO_AUTO = 301,
        DELETE_UNDO_DICTATION = 302,
        DELETE_NOTIFY = 303,
        DELETE_TAP = 304,
        DELETE_BACKWARD_EDITOR = 305,
        DELETE_FORWARD_EDITOR = 306,
        DELETE_STATE = 307,
        GIF_DELETE_KEY = 400,
        GIF_DELETE_WORD_KEY = 401,
        GIF_SPACE = 402,
        GIF_SUBMIT = 403,
        GIF_TYPE = 404,
    }
}

crate::declare! {
    /// A key's phase, as `executeInput` passes it.
    pub const PHASES: object InputProtocol(i32) {
        PHASE_DOWN = 0,
        PHASE_UP = 1,
        PHASE_CANCEL = 2,
        PHASE_REPEAT = 3,
    }
}

crate::declare! {
    /// The bits of a long press's plan.
    pub const HOLDS: object InputProtocol(i32) {
        HOLD_SPACE = 1,
        HOLD_CAPS = 2,
        HOLD_PICKER = 4,
        HOLD_POPUP = 8,
        HOLD_FEEDBACK = 16,
        HOLD_BLOCK = 32,
    }
}

crate::declare! {
    /// The plans `InputRules.control` makes, by its first argument.
    pub const CONTROLS: object InputProtocol(i32) {
        CONTROL_EVALUATOR = 0,
        CONTROL_SUBTYPE = 1,
        CONTROL_CYCLE = 2,
        CONTROL_SPACE = 3,
        CONTROL_START = 4,
        CONTROL_FINISH = 5,
        CONTROL_FIND_SUBTYPE = 6,
        CONTROL_DELETE = 7,
    }
}

crate::declare! {
    /// The icons the evaluator picks for a key.
    pub const ICONS: object InputProtocol(i32) {
        ICON_RETURN = 1000,
        ICON_DONE = 1001,
        ICON_ARROW = 1002,
        ICON_SEARCH = 1003,
        ICON_SEND = 1004,
        ICON_INCOGNITO_OFF = 1005,
        ICON_CAPS = 1006,
    }
}

/// The groups the generator writes as Kotlin's `InputProtocol`.
pub const GROUPS: &[&crate::Group] = &[&CALLS, &EFFECTS, &PHASES, &HOLDS, &CONTROLS, &ICONS];
