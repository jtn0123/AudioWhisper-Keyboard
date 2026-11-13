//! The editing tiles use InputEventDispatcher's exact hold/repeat and overflow-close policy.
use keys::codes;
pub fn repeatable(code: i32) -> bool {
    [
        codes::ARROW_DOWN,
        codes::ARROW_LEFT,
        codes::ARROW_RIGHT,
        codes::ARROW_UP,
        codes::DELETE,
        codes::FORWARD_DELETE,
        codes::UNDO,
        codes::REDO,
    ]
    .contains(&code)
}
pub fn repeat_code(code: i32, words: bool) -> i32 {
    match code {
        codes::DELETE if words => codes::DELETE_WORD,
        codes::FORWARD_DELETE if words => codes::FORWARD_DELETE_WORD,
        _ => code,
    }
}
pub fn repeat_delay(code: i32, base: i64) -> i64 {
    base * if [
        codes::DELETE_WORD,
        codes::FORWARD_DELETE_WORD,
        codes::UNDO,
        codes::REDO,
    ]
    .contains(&code)
    {
        5
    } else {
        1
    }
}
pub fn hold_delay(code: i32, base: i64) -> i64 {
    (base as f32
        * match code {
            codes::SPACE | codes::CJK_SPACE | codes::SHIFT => 2.5,
            codes::LANGUAGE_SWITCH => 2.0,
            _ => 1.0,
        }) as i64
}
pub fn closes(code: i32) -> bool {
    !repeatable(code)
        && code != codes::TOGGLE_ACTIONS_OVERFLOW
        && code != codes::CLIPBOARD_SELECT_ALL
}

/// manual mode, its start/end flags, editor range, requested select/arrow command.
/// Returns updated flags and an optional collapsed editor position (-1 means no edit).
pub fn selection(c: [i32; 6]) -> [i32; 4] {
    let [manual, start, end, left, right, code] = c;
    let selected = left >= 0 && right >= 0 && left != right;
    if code == codes::CLIPBOARD_SELECT {
        if selected {
            [
                0,
                start,
                end,
                if manual != 0 && start != 0 {
                    left
                } else {
                    right
                },
            ]
        } else {
            [i32::from(manual == 0), start, end, -1]
        }
    } else if manual != 0 && !selected {
        let beginning = [
            codes::ARROW_LEFT,
            codes::ARROW_UP,
            codes::MOVE_START_OF_LINE,
            codes::MOVE_START_OF_PAGE,
        ]
        .contains(&code);
        [manual, i32::from(beginning), i32::from(!beginning), -1]
    } else {
        [manual, start, end, -1]
    }
}
