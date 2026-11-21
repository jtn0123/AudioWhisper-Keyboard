//! The renderer's codes, which Kotlin sees as `RenderProtocol`. `render::protocol` re-exports
//! them for the crates that draw and route touches.

crate::declare! {
    /// The actions a touch, a timer or an accessibility request yields: a kind and two integers.
    /// Kotlin reads them in ranges (the smartbar's lie between `BAR_DOWN` and `GLIDE_POINT`, the
    /// panels' from `PANEL_DOWN` to `PANEL_REPEAT`), so keep each family together.
    pub const ACTIONS: object RenderProtocol(i32) {
        KEY_DOWN = 0,
        KEY_UP = 1,
        KEY_CANCEL = 2,
        POPUP_SELECT = 3,
        CANDIDATE_COMMIT = 4,
        BAR_DOWN = 5,
        BAR_UP = 6,
        BAR_CANCEL = 7,
        CANDIDATE_DOWN = 8,
        CANDIDATE_UP = 9,
        CANDIDATE_CANCEL = 10,
        TOOLTIP_SHOW = 11,
        BAR_TOGGLE = 12,
        CANDIDATE_HOLD = 13,
        TOOLTIP_UP = 14,
        TOOLTIP_CLEAR = 15,
        BAR_HELD_UP = 16,
        GLIDE_POINT = 20,
        GLIDE_COMPLETE = 21,
        GLIDE_CANCEL = 22,
        SWIPE_ACTION = 23,
        CURSOR_LEFT = 24,
        CURSOR_RIGHT = 25,
        MASS_BEGIN = 26,
        MASS_END = 27,
        SELECT_CHARACTERS = 28,
        SELECT_WORDS = 29,
        DELETE_SELECTION = 30,
        GIF_DELETE_CHARACTERS = 31,
        GIF_DELETE_WORDS = 32,
        GESTURE_FEEDBACK = 33,
        LONG_PRESS_BLOCK = 34,
        OPEN_POPUP = 35,
        CAPS_LOCK = 36,
        INPUT_METHOD_PICKER = 37,
        KEY_FEEDBACK = 38,
        LONG_PRESS_FEEDBACK = 39,
        KEY_CANCEL_CODE = 40,
        ACCESS_READ_TOP = 41,
        /// Before each GLIDE_POINT when the gesture config asks for it: the point's time, in
        /// microseconds after the touch went down.
        GLIDE_TIME = 42,
        PANEL_DOWN = 50,
        PANEL_UP = 51,
        PANEL_CANCEL = 52,
        PANEL_MODE = 53,
        PANEL_REPEAT = 54,
        PANEL_EMOJI = 55,
        PANEL_HISTORY = 56,
        PANEL_CHANGED = 57,
        PANEL_CLIPBOARD = 58,
        PANEL_GIF = 59,
        PANEL_SUBTYPE = 60,
        IME_CHROME = 61,
    }
}

crate::declare! {
    /// The bits of a smartbar action's flags word.
    pub const BAR_FLAGS: object RenderProtocol(i32) {
        BAR_ENABLED = 1,
        BAR_DICTATING = 2,
        BAR_READING = 4,
        BAR_WARMING = 8,
        BAR_FINISHING = 16,
    }
}

crate::declare! {
    /// Where the microphone level sits in that word.
    pub const BAR_LEVEL: object RenderProtocol(i32) {
        BAR_LEVEL_SHIFT = 8,
        BAR_LEVEL_MASK = 255,
    }
}

crate::declare! {
    /// The smartbar's action packet: a code and its flags word for each action.
    pub const BAR_ACTION: object RenderProtocol(i32) {
        BAR_ACTION_STRIDE = 2,
        BAR_FLAGS_OFFSET = 1,
    }
}

crate::declare! {
    /// The smartbar's measurement packet: a header, then a slot of six floats each.
    pub const BAR_SLOTS: object RenderProtocol(i32) {
        BAR_MEASUREMENT_HEADER = 8,
        BAR_SLOT_STRIDE = 6,
        BAR_SLOT_ID = 1,
        BAR_SLOT_LEFT = 2,
        BAR_SLOT_TOP = 3,
        BAR_SLOT_RIGHT = 4,
        BAR_SLOT_BOTTOM = 5,
    }
}

crate::declare! {
    /// A slot's kind, its first float, and the id of the slot that toggles the actions.
    pub const BAR_SLOT_KINDS: object RenderProtocol(i32) {
        BAR_SLOT_ACTION = 1,
        BAR_TOGGLE_INDEX = 4,
    }
}

crate::declare! {
    /// The bits of a candidate's flags.
    pub const CANDIDATE_FLAGS: object RenderProtocol(i32) {
        CANDIDATE_AUTO_COMMIT = 1,
        CANDIDATE_CLIPBOARD = 2,
    }
}

crate::declare! {
    /// The swipe and space-bar preferences, by their place in the gesture config's actions.
    pub const PREFS: object RenderProtocol(i32) {
        PREF_SWIPE_UP = 0,
        PREF_SWIPE_DOWN = 1,
        PREF_SWIPE_LEFT = 2,
        PREF_SWIPE_RIGHT = 3,
        PREF_SPACE_UP = 4,
        PREF_SPACE_LEFT = 5,
        PREF_SPACE_RIGHT = 6,
        PREF_SPACE_HOLD = 7,
        PREF_DELETE_LEFT = 8,
    }
}

crate::declare! {
    /// The accessibility actions a virtual view supports and TalkBack asks for.
    pub const ACCESS: object RenderProtocol(i32) {
        ACCESS_CLICK = 1,
        ACCESS_LONG_CLICK = 2,
        ACCESS_DISMISS = 3,
        ACCESS_SCROLL_FORWARD = 4,
        ACCESS_SCROLL_BACKWARD = 5,
    }
}

/// The groups the generator writes as Kotlin's `RenderProtocol`.
pub const GROUPS: &[&crate::Group] = &[
    &ACTIONS,
    &BAR_FLAGS,
    &BAR_LEVEL,
    &BAR_ACTION,
    &BAR_SLOTS,
    &BAR_SLOT_KINDS,
    &CANDIDATE_FLAGS,
    &PREFS,
    &ACCESS,
];
