//! The last auto-correction, kept so that a backspace straight after it puts back what was typed,
//! as AOSP's keyboard does ("teh " becomes "the ", and a backspace makes it "teh"). Was Kotlin's
//! `AutoCorrectionUndo.revertAt`.

/// A space, a punctuation mark, or a punctuation mark and the space added after it.
const MAX_SEPARATOR_LENGTH: i32 = 2;
const SPACE: u16 = b' ' as u16;

/// Replace the `length` characters left of the cursor with `text`, and leave a space pending if
/// `phantom_space`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Revert {
    pub length: i32,
    pub text: Vec<u16>,
    pub phantom_space: bool,
}

/// How to undo correcting `typed` to `corrected` at `start` with the cursor at `cursor` and
/// `text_before_cursor` left of it, or `None` if that text is no longer the correction followed by
/// the separator that triggered it.
///
/// A space separator is dropped and left pending as a phantom space, so the user can fix the word,
/// add punctuation to it, or just type on and get the space back. Punctuation is kept.
pub fn revert_at(
    (typed, corrected, start): (&[u16], &[u16], i32),
    cursor: i32,
    text_before_cursor: &[u16],
) -> Option<Revert> {
    // A string's length is an Int; the arithmetic wraps as Kotlin's does.
    let corrected_length = corrected.len() as i32;
    let separator_length = cursor.wrapping_sub(start).wrapping_sub(corrected_length);
    if !(1..=MAX_SEPARATOR_LENGTH).contains(&separator_length) {
        return None;
    }
    let length = corrected_length.wrapping_add(separator_length);
    let from = usize::try_from(length)
        .ok()
        .and_then(|length| text_before_cursor.len().checked_sub(length))?;
    let replaced = &text_before_cursor[from..];
    let separator = replaced.strip_prefix(corrected)?;
    if separator
        .iter()
        .any(|&unit| chars::is_letter_or_digit(unit))
    {
        return None;
    }
    Some(if separator == [SPACE] {
        Revert {
            length,
            text: typed.to_vec(),
            phantom_space: true,
        }
    } else {
        Revert {
            length,
            text: [typed, separator].concat(),
            phantom_space: false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    fn revert(cursor: i32, text: &str) -> Option<(i32, String, bool)> {
        let (typed, corrected) = (utf16("teh"), utf16("the"));
        let revert = revert_at((&typed, &corrected, 4), cursor, &utf16(text))?;
        Some((
            revert.length,
            String::from_utf16(&revert.text).unwrap(),
            revert.phantom_space,
        ))
    }

    #[test]
    fn a_space_comes_back_as_a_phantom_space() {
        assert_eq!(revert(8, "say the "), Some((4, "teh".into(), true)));
    }

    #[test]
    fn punctuation_is_kept() {
        assert_eq!(revert(8, "say the."), Some((4, "teh.".into(), false)));
        assert_eq!(revert(9, "say the. "), Some((5, "teh. ".into(), false)));
    }

    /// Only the text's end is looked at, not where it starts.
    #[test]
    fn the_text_needs_only_to_end_in_the_correction() {
        assert_eq!(revert(8, "the "), Some((4, "teh".into(), true)));
    }

    #[test]
    fn anything_else_means_no_undo() {
        assert_eq!(revert(7, "say the"), None);
        assert_eq!(revert(10, "say the.  "), None);
        assert_eq!(revert(8, "say thy "), None);
        assert_eq!(revert(8, "say thex"), None);
        assert_eq!(revert(8, "th "), None);
    }

    #[test]
    fn only_a_lone_space_becomes_a_phantom_space() {
        assert_eq!(revert(9, "say the ."), Some((5, "teh .".into(), false)));
    }
}
