//! Shared pointer predicates. The renderer and Compose retain their platform bindings.
use keys::codes::*;
pub fn suitable(code: i32) -> bool {
    (code > SPACE && code != CJK_SPACE)
        || matches!(
            code,
            ENTER
                | LANGUAGE_SWITCH
                | IME_UI_MODE_TEXT
                | IME_UI_MODE_MEDIA
                | IME_UI_MODE_CLIPBOARD
                | CHAR_WIDTH_SWITCHER
        )
}
pub fn roll_release(code: i32) -> bool {
    suitable(code)
}
pub fn slide(x: f32, y: f32, left: f32, top: f32, right: f32, bottom: f32) -> bool {
    x < left - 0.1 * (right - left)
        || x > right + 0.1 * (right - left)
        || y < top - 0.35 * (bottom - top)
        || y > bottom + 0.35 * (bottom - top)
}

pub fn hold(code: i32, popup: bool, uninterrupted: bool, space_action: bool) -> i32 {
    use crate::protocol::*;
    match code {
        SPACE | CJK_SPACE => HOLD_BLOCK | if space_action { HOLD_SPACE } else { 0 },
        SHIFT if uninterrupted => HOLD_CAPS | HOLD_FEEDBACK,
        LANGUAGE_SWITCH => HOLD_PICKER | HOLD_BLOCK,
        code if suitable(code) && popup => HOLD_POPUP | HOLD_FEEDBACK | HOLD_BLOCK,
        _ => 0,
    }
}
