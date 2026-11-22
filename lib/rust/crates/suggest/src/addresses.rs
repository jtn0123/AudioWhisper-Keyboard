//! E-mail addresses and URLs typed into ordinary text ("write to `bob@example.com`", "see
//! `https://example.com/news`"). Their pieces are letters like any word, so the word learner would
//! keep "example", "com" or "news"; none of an address is ever learned. The letters before the
//! first mark are let through, as nothing yet shows they start an address.

use chars::{is_letter_or_digit, is_whitespace};

const DOT: u16 = b'.' as u16;

/// The marks that put a word inside an address or URL when they are glued to it.
const MARKS: [u16; 3] = [b'@' as u16, b'/' as u16, b':' as u16];

/// The text glued to the word typed after `text_before`: what follows its last whitespace.
pub fn glued(text_before: &[u16]) -> &[u16] {
    let start = text_before
        .iter()
        .rposition(|&unit| is_whitespace(unit))
        .map_or(0, |at| at + 1);
    &text_before[start..]
}

/// Whether `word`, typed right after `text_before`, is a piece of an address or URL: the text
/// glued to it has an '@', '/' or ':' in it, or a '.' between two letters or digits, the word's
/// own first letter included ("example." before "com").
pub fn is_inside(text_before: &[u16], word: &[u16]) -> bool {
    let glued = glued(text_before);
    if glued.iter().any(|unit| MARKS.contains(unit)) {
        return true;
    }
    let token: Vec<u16> = glued.iter().chain(word.first()).copied().collect();
    token
        .windows(3)
        .any(|w| w[1] == DOT && is_letter_or_digit(w[0]) && is_letter_or_digit(w[2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    fn inside(text_before: &str, word: &str) -> bool {
        is_inside(&utf16(text_before), &utf16(word))
    }

    #[test]
    fn the_pieces_of_an_address_or_url_are_inside_it() {
        for (before, word) in [
            ("write to bob@", "example"),
            ("write to bob@example.", "com"),
            ("see https:", "example"),
            ("see https://", "example"),
            ("see https://example.com/", "news"),
            ("see www.", "example"),
            ("see example.", "com"),
            ("v1.", "two"),
            ("(see example.", "com"),
        ] {
            assert!(inside(before, word), "{before:?} {word:?}");
        }
    }

    #[test]
    fn ordinary_words_are_not() {
        for (before, word) in [
            ("", "hello"),
            ("see you ", "later"),
            ("it ended. ", "Then"),
            ("bob@example.com ", "wrote"),
            ("(", "see"),
            ("\"", "quoted"),
            ("wait...", "what"),
            ("well, ", "yes"),
            ("line\n", "two"),
        ] {
            assert!(!inside(before, word), "{before:?} {word:?}");
        }
    }

    #[test]
    fn the_glued_text_starts_after_the_last_whitespace() {
        assert_eq!(glued(&utf16("see bob@")), utf16("bob@"));
        assert_eq!(glued(&utf16("see ")), utf16(""));
        assert_eq!(glued(&utf16("a\tb")), utf16("b"));
        assert_eq!(glued(&utf16("ab")), utf16("ab"));
    }
}
