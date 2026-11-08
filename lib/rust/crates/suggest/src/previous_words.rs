//! The n-gram context before the word being typed. Mirrors AOSP's `NgramContext`; was Kotlin's
//! `PreviousWords`.

/// The native MAX_PREV_WORD_COUNT_FOR_N_GRAM.
pub const MAX_COUNT: usize = 3;

const SENTENCE_TERMINATORS: [u16; 3] = [b'.' as u16, b'?' as u16, b'!' as u16];
const WORD_CONNECTORS: [u16; 3] = [b'\'' as u16, 0x2019, b'-' as u16];
const NEWLINE: u16 = b'\n' as u16;

/// Up to [`MAX_COUNT`] previous words, most recent first, and whether a beginning-of-sentence
/// marker follows them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PreviousWords {
    pub words: Vec<Vec<u16>>,
    pub is_beginning_of_sentence: bool,
}

impl PreviousWords {
    /// No usable context (for example after a comma).
    pub const NONE: PreviousWords = PreviousWords {
        words: Vec::new(),
        is_beginning_of_sentence: false,
    };

    /// The native n-gram slots: each word, then the sentence-start marker if there is room.
    pub fn slot_count(&self) -> usize {
        (self.words.len() + usize::from(self.is_beginning_of_sentence)).min(MAX_COUNT)
    }

    /// The context for a word that starts right after `text_before_word`, by AOSP's
    /// `NgramContextUtils.getNgramContextFromNthPreviousWord`: only the current line counts, words
    /// are split on whitespace, the start of the line or a word ending in `.?!` is a sentence
    /// start, and any other punctuation at a word's end leaves the context unknown.
    pub fn from(text_before_word: &[u16]) -> PreviousWords {
        let line_start = text_before_word
            .iter()
            .rposition(|&u| u == NEWLINE)
            .map_or(0, |i| i + 1);
        // The word being typed is the empty last token, so a prefix glued to it ("(word") stays
        // with it.
        let tokens = split_on_whitespace(&text_before_word[line_start..]);
        let mut words = Vec::with_capacity(MAX_COUNT);
        let sentence_starts = |words| PreviousWords {
            words,
            is_beginning_of_sentence: true,
        };
        for i in 0..MAX_COUNT {
            // Kotlin's index, tokens.size - 2 - i, plus one: never negative, as the loop returns
            // as soon as the index is.
            let following = tokens[tokens.len() - 1 - i];
            if following
                .first()
                .is_some_and(|u| WORD_CONNECTORS.contains(u))
            {
                break;
            }
            let Some(index) = (tokens.len() - 1 - i).checked_sub(1) else {
                return sentence_starts(words);
            };
            let token = tokens[index];
            match token.last() {
                None => return sentence_starts(words),
                Some(last) if SENTENCE_TERMINATORS.contains(last) => return sentence_starts(words),
                Some(&last) if !chars::is_letter_or_digit(last) => break,
                Some(_) => words.push(token.to_vec()),
            }
        }
        PreviousWords {
            words,
            is_beginning_of_sentence: false,
        }
    }
}

/// Kotlin's `split(Regex("\\s+"))`, keeping the empty strings at either end.
fn split_on_whitespace(text: &[u16]) -> Vec<&[u16]> {
    let mut tokens = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < text.len() {
        if chars::is_regex_space(text[i]) {
            tokens.push(&text[start..i]);
            while i < text.len() && chars::is_regex_space(text[i]) {
                i += 1;
            }
            start = i;
        } else {
            i += 1;
        }
    }
    tokens.push(&text[start..]);
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(text: &str) -> (Vec<String>, bool) {
        let units: Vec<u16> = text.encode_utf16().collect();
        let previous = PreviousWords::from(&units);
        let words = previous
            .words
            .iter()
            .map(|w| String::from_utf16(w).unwrap())
            .collect();
        (words, previous.is_beginning_of_sentence)
    }

    fn words(list: &[&str]) -> Vec<String> {
        list.iter().map(|w| w.to_string()).collect()
    }

    #[test]
    fn takes_up_to_three_words_most_recent_first() {
        assert_eq!(
            context("I want to go "),
            (words(&["go", "to", "want"]), false)
        );
        assert_eq!(context("want to "), (words(&["to", "want"]), true));
        assert_eq!(context(""), (words(&[]), true));
        assert_eq!(context("hello"), (words(&[]), true));
        assert_eq!(context("don't "), (words(&["don't"]), true));
    }

    #[test]
    fn a_sentence_end_or_line_start_is_a_sentence_start() {
        assert_eq!(context("Done. Then "), (words(&["Then"]), true));
        assert_eq!(context("first line\nsecond "), (words(&["second"]), true));
        assert_eq!(context("Really?  so "), (words(&["so"]), true));
    }

    #[test]
    fn other_punctuation_leaves_the_context_unknown() {
        assert_eq!(context("well, "), (words(&[]), false));
        assert_eq!(context("say hi) "), (words(&[]), false));
        // Punctuation in front of a word is part of it.
        assert_eq!(context("say (hi "), (words(&["(hi", "say"]), true));
        assert_eq!(context("rock '"), (words(&[]), false));
        assert_eq!(context("one -"), (words(&[]), false));
    }

    #[test]
    fn counts_slots() {
        let three = PreviousWords {
            words: vec![vec![1], vec![2], vec![3]],
            is_beginning_of_sentence: true,
        };
        assert_eq!(three.slot_count(), 3);
        let one = PreviousWords {
            words: vec![vec![1]],
            is_beginning_of_sentence: true,
        };
        assert_eq!(one.slot_count(), 2);
        assert_eq!(PreviousWords::NONE.slot_count(), 0);
    }

    #[test]
    fn splits_like_kotlin() {
        let split = |text: &str| {
            let units: Vec<u16> = text.encode_utf16().collect();
            split_on_whitespace(&units)
                .iter()
                .map(|t| String::from_utf16(t).unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(split(" a  b "), ["", "a", "b", ""]);
        assert_eq!(split(""), [""]);
        assert_eq!(split("a"), ["a"]);
    }

    #[test]
    fn a_word_connector_before_the_word_leaves_the_context_unknown() {
        for connector in ["'", "’", "-"] {
            assert_eq!(context(&format!("one {connector}")), (words(&[]), false));
        }
    }

    #[test]
    fn splits_on_regex_whitespace_only() {
        // On the JVM, \s is ASCII whitespace; an ideographic space is part of a word.
        assert_eq!(context("a\u{3000}b "), (words(&["a\u{3000}b"]), true));
        // A carriage return splits words but doesn't start a line.
        assert_eq!(context("a\rb "), (words(&["b", "a"]), true));
        assert_eq!(context("mp3 "), (words(&["mp3"]), true));
    }
}
