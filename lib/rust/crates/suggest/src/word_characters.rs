//! Which typed characters end a word, and so may trigger auto-correcting it. Was Kotlin's
//! `WordCharacters`.

use chars::{code_point, is_letter};

/// Kotlin's first UTF-16 unit after the cursor, as the former TypingState factory judged it.
pub fn cursor_mid_word(text_after_cursor: &[u16]) -> bool {
    text_after_cursor
        .first()
        .is_some_and(|&u| chars::is_letter_or_digit(u))
}

/// AOSP's word connectors (`'` and `-`, plus the typographic apostrophe) and `@`, which continue
/// the word being typed: "don't" and "e-mail" aren't corrected halfway through, and the name part
/// of an e-mail address is left alone.
const CONTINUING_PUNCTUATION: [u32; 4] = ['\'' as u32, '’' as u32, '-' as u32, '@' as u32];

/// The apostrophes and the hyphen, which a word may have inside it.
const CONNECTORS: [u16; 3] = [b'\'' as u16, 0x2019, b'-' as u16];

/// Whether `word` is letters, with apostrophes or hyphens inside: no digits, spaces, tabs or
/// symbols. Such a word can be learned, and spelled by a swipe, which skips over the connectors.
pub fn is_word(word: &[u16]) -> bool {
    word.iter().any(|&u| is_letter(u))
        && word
            .iter()
            .all(|&u| is_letter(u) || CONNECTORS.contains(&u))
}

/// Whether typing `code_point` right after a word continues it rather than ending it: a digit of
/// any script (so "mp3" isn't corrected), or one of [`CONTINUING_PUNCTUATION`].
pub fn continues_word(code_point: i32) -> bool {
    u32::try_from(code_point)
        .is_ok_and(|cp| code_point::is_digit(cp) || CONTINUING_PUNCTUATION.contains(&cp))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_and_connectors_continue_a_word() {
        for c in ['3', '٣', '\'', '’', '-', '@'] {
            assert!(continues_word(c as i32), "{c}");
        }
        assert!(continues_word(0x1D7CE)); // a mathematical digit
        for c in [' ', '.', ',', '!', 'a', '²', '"'] {
            assert!(!continues_word(c as i32), "{c}");
        }
        assert!(!continues_word(-1));
    }
}
