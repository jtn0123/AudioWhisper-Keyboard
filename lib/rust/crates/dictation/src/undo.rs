//! Taking back the phrase dictation typed last with one backspace.

/// Whether a backspace with the cursor at `cursor`, after `text_before_cursor`, can still take
/// back `typed`, the phrase dictation typed at `start`: it must be what is right before the
/// cursor, and the cursor where typing it left it. `start` and `cursor` are the editor's Ints,
/// whose sum wraps as Kotlin's does.
pub fn reverts_at(typed: &[u16], start: i32, cursor: i32, text_before_cursor: &[u16]) -> bool {
    // A Kotlin String's length is an Int.
    let length = i32::try_from(typed.len()).unwrap_or(i32::MAX);
    !typed.is_empty() && cursor == start.wrapping_add(length) && text_before_cursor.ends_with(typed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utf16;

    #[test]
    fn reverts_only_right_after_the_phrase() {
        let typed = utf16(" to John.");
        assert!(reverts_at(&typed, 10, 19, &utf16("I talked to John.")));
        assert!(!reverts_at(&typed, 10, 18, &utf16("I talked to John.")));
        assert!(!reverts_at(&typed, 10, 19, &utf16("I talked to Joan.")));
        assert!(!reverts_at(&[], 10, 10, &utf16("anything")));
    }

    #[test]
    fn the_cursor_sum_wraps_as_in_kotlin() {
        assert!(reverts_at(
            &utf16("ab"),
            i32::MAX,
            i32::MIN + 1,
            &utf16("ab")
        ));
    }
}
