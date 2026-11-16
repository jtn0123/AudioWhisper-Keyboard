//! Canonical wire constants. Gradle generates RenderProtocol.java from these declarations.
pub const KEY_DOWN: i32 = 0;
pub const KEY_UP: i32 = 1;
pub const KEY_CANCEL: i32 = 2;
pub const POPUP_SELECT: i32 = 3;
pub const CANDIDATE_COMMIT: i32 = 4;
pub const BAR_DOWN: i32 = 5;
pub const BAR_UP: i32 = 6;
pub const BAR_CANCEL: i32 = 7;
pub const CANDIDATE_DOWN: i32 = 8;
pub const CANDIDATE_UP: i32 = 9;
pub const CANDIDATE_CANCEL: i32 = 10;
pub const TOOLTIP_SHOW: i32 = 11;
pub const BAR_TOGGLE: i32 = 12;
pub const CANDIDATE_HOLD: i32 = 13;
pub const TOOLTIP_UP: i32 = 14;
pub const TOOLTIP_CLEAR: i32 = 15;
pub const BAR_ENABLED: i32 = 1;
pub const BAR_DICTATING: i32 = 2;
pub const BAR_READING: i32 = 4;
pub const BAR_WARMING: i32 = 8;
pub const BAR_FINISHING: i32 = 16;
pub const BAR_LEVEL_SHIFT: i32 = 8;
pub const BAR_LEVEL_MASK: i32 = 255;
pub const BAR_TOGGLE_INDEX: i32 = 4;
pub const CANDIDATE_AUTO_COMMIT: i32 = 1;
pub const CANDIDATE_CLIPBOARD: i32 = 2;

pub const GLIDE_POINT: i32 = 20;
pub const GLIDE_COMPLETE: i32 = 21;
pub const GLIDE_CANCEL: i32 = 22;
pub const SWIPE_ACTION: i32 = 23;
pub const CURSOR_LEFT: i32 = 24;
pub const CURSOR_RIGHT: i32 = 25;
pub const MASS_BEGIN: i32 = 26;
pub const MASS_END: i32 = 27;
pub const SELECT_CHARACTERS: i32 = 28;
pub const SELECT_WORDS: i32 = 29;
pub const DELETE_SELECTION: i32 = 30;
pub const GIF_DELETE_CHARACTERS: i32 = 31;
pub const GIF_DELETE_WORDS: i32 = 32;
pub const GESTURE_FEEDBACK: i32 = 33;
pub const LONG_PRESS_BLOCK: i32 = 34;
pub const OPEN_POPUP: i32 = 35;
pub const CAPS_LOCK: i32 = 36;
pub const INPUT_METHOD_PICKER: i32 = 37;
pub const KEY_FEEDBACK: i32 = 38;
pub const LONG_PRESS_FEEDBACK: i32 = 39;
/// Before each GLIDE_POINT when the gesture config asks for it: the point's time, in microseconds
/// after the touch went down.
pub const GLIDE_TIME: i32 = 42;
pub const PREF_SWIPE_UP: i32 = 0;
pub const PREF_SWIPE_DOWN: i32 = 1;
pub const PREF_SWIPE_LEFT: i32 = 2;
pub const PREF_SWIPE_RIGHT: i32 = 3;
pub const PREF_SPACE_UP: i32 = 4;
pub const PREF_SPACE_LEFT: i32 = 5;
pub const PREF_SPACE_RIGHT: i32 = 6;
pub const PREF_SPACE_HOLD: i32 = 7;
pub const PREF_DELETE_LEFT: i32 = 8;
pub const KEY_CANCEL_CODE: i32 = 40;
pub const BAR_HELD_UP: i32 = 16;
pub const BAR_ACTION_STRIDE: i32 = 2;
pub const BAR_FLAGS_OFFSET: i32 = 1;
pub const BAR_SLOT_ACTION: i32 = 1;
pub const BAR_MEASUREMENT_HEADER: i32 = 8;
pub const BAR_SLOT_STRIDE: i32 = 6;
pub const BAR_SLOT_ID: i32 = 1;
pub const BAR_SLOT_LEFT: i32 = 2;
pub const BAR_SLOT_TOP: i32 = 3;
pub const BAR_SLOT_RIGHT: i32 = 4;
pub const BAR_SLOT_BOTTOM: i32 = 5;
pub const ACCESS_CLICK: i32 = 1;
pub const ACCESS_LONG_CLICK: i32 = 2;
pub const ACCESS_DISMISS: i32 = 3;
pub const ACCESS_READ_TOP: i32 = 41;

pub const PANEL_DOWN: i32 = 50;
pub const PANEL_UP: i32 = 51;
pub const PANEL_CANCEL: i32 = 52;
pub const PANEL_MODE: i32 = 53;

pub const PANEL_REPEAT: i32 = 54;

pub const PANEL_EMOJI: i32 = 55;
pub const PANEL_HISTORY: i32 = 56;

pub const PANEL_CHANGED: i32 = 57;
pub const PANEL_SUBTYPE: i32 = 60;
pub const IME_CHROME: i32 = 61;

pub const ACCESS_SCROLL_FORWARD: i32 = 4;
pub const ACCESS_SCROLL_BACKWARD: i32 = 5;

pub const PANEL_CLIPBOARD: i32 = 58;

pub const PANEL_GIF: i32 = 59;
