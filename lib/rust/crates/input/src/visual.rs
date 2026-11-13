//! Labels/icons are resource tokens, resolved to Android strings/vectors by the host.
use crate::protocol::*;
use keys::{codes::*, models};
pub fn label(kind: i32, code: i32, mode: i32) -> i32 {
    if (kind == 0 && !matches!(code, SPACE | CJK_SPACE | HALF_SPACE | KESHIDA)) || kind == 7 {
        return 1;
    }
    match code {
        SPACE | CJK_SPACE => {
            if mode == 0 {
                2
            } else {
                0
            }
        }
        PHONE_PAUSE
        | PHONE_WAIT
        | IME_UI_MODE_TEXT
        | VIEW_CHARACTERS
        | VIEW_NUMERIC
        | VIEW_NUMERIC_ADVANCED
        | VIEW_PHONE
        | VIEW_PHONE2
        | VIEW_SYMBOLS
        | VIEW_SYMBOLS2
        | HALF_SPACE
        | KESHIDA => code,
        _ => 0,
    }
}
pub fn icon(code: i32, raw: u64, input: i32, options: i32, mode: i32) -> i32 {
    match code {
        ENTER => {
            if options & 0x40000000 != 0 || input & 0x20000 != 0 {
                ICON_RETURN
            } else {
                match options & 255 {
                    6 => ICON_DONE,
                    2 | 5 | 7 => ICON_ARROW,
                    3 => ICON_SEARCH,
                    4 => ICON_SEND,
                    _ => ICON_RETURN,
                }
            }
        }
        SHIFT => {
            if models::read(raw, 2) == Some(0) {
                SHIFT
            } else {
                ICON_CAPS
            }
        }
        CAPS_LOCK => 0,
        SPACE | CJK_SPACE => {
            if matches!(mode, 4..=7) {
                SPACE
            } else {
                0
            }
        }
        TOGGLE_INCOGNITO_MODE => {
            if models::read(raw, 9) == Some(0) {
                ICON_INCOGNITO_OFF
            } else {
                TOGGLE_INCOGNITO_MODE
            }
        }
        CHAR_WIDTH_SWITCHER => {
            if models::read(raw, 14) == Some(0) {
                CHAR_WIDTH_HALF
            } else {
                CHAR_WIDTH_FULL
            }
        }
        _ => code,
    }
}
