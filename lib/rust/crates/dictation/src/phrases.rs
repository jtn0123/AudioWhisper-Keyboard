//! Joining phrases that were transcribed separately.
//!
//! Each pause is transcribed on its own, so the model treats every phrase as a whole sentence: it
//! capitalises the first word and usually ends with a full stop, even where the speaker only
//! paused ("I need to talk to." "John about it.").

use chars::{
    is_blank, is_letter, is_letter_or_digit, is_lower_case, is_upper_case, is_whitespace,
    lower_equals_ascii, to_lower, to_upper,
};

use crate::unit;

/// Delete `replacing` characters before the cursor (a full stop this dictation typed), then type
/// `text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Typing {
    pub replacing: usize,
    pub text: Vec<u16>,
}

/// Punctuation the model ends a phrase with, which a spoken mark replaces.
const MARKS: [u16; 6] = [
    unit('.'),
    unit(','),
    unit(';'),
    unit(':'),
    unit('!'),
    unit('?'),
];

const SENTENCE_ENDS: [u16; 3] = [unit('.'), unit('!'), unit('?')];

/// Words a spoken sentence practically never ends on, so a full stop after one is a pause.
const OPEN_ENDINGS: [&str; 26] = [
    "a", "an", "the", "and", "or", "but", "nor", "of", "to", "for", "with", "about", "from",
    "into", "onto", "at", "as", "than", "because", "if", "my", "your", "our", "their", "its", "i",
];

/// First words that join on to the sentence before even when the model capitalised them.
const JOINERS: [&str; 5] = ["and", "but", "or", "nor", "because"];

/// How to type `phrase` after `before`, the text up to the cursor. `last_phrase` is what this
/// dictation typed last: when the cursor is still right after it, its full stop is taken back if
/// `phrase` carries the sentence on. `is_name` says a capitalised word is a name ("John"), which
/// keeps its capital mid-sentence; it is asked at most once, and only when that decides.
pub fn join(
    before: &[u16],
    last_phrase: Option<&[u16]>,
    phrase: &[u16],
    is_name: impl FnOnce(&[u16]) -> bool,
) -> Typing {
    // A full stop said out loud ("period") has no words with it, and is never taken back.
    let takes_back_stop = last_phrase.is_some_and(|last| {
        last.last() == Some(&unit('.'))
            && last.iter().any(|&u| is_letter_or_digit(u))
            && is_right_after(before, last)
            && carries_on(last, phrase)
    });
    // Taking a full stop back means the cursor is right after it, so `before` isn't empty.
    let kept = if takes_back_stop {
        &before[..before.len() - 1]
    } else {
        before
    };
    let body = if starts_line(kept) || ends_sentence(kept) {
        replace_first(phrase, to_upper)
    } else if continues_sentence(kept) {
        lower_first(phrase, is_name)
    } else {
        phrase.to_vec()
    };
    let mut text = Vec::with_capacity(body.len() + 1);
    if kept.last().is_some_and(|&u| !is_whitespace(u)) {
        text.push(unit(' '));
    }
    text.extend(body);
    Typing {
        replacing: usize::from(takes_back_stop),
        text,
    }
}

/// How to type a spoken punctuation `mark` ("question mark"): in place of the punctuation that
/// ends `last_phrase` if the cursor is still right after it ("See you soon." → "See you soon?"),
/// otherwise straight after the text.
pub fn punctuate(before: &[u16], last_phrase: Option<&[u16]>, mark: u16) -> Typing {
    let replaces = last_phrase.is_some_and(|last| {
        last.last().is_some_and(|u| MARKS.contains(u)) && is_right_after(before, last)
    });
    Typing {
        replacing: usize::from(replaces),
        text: vec![mark],
    }
}

/// Only what the editor shows before the cursor can be compared, so match as much of `phrase`
/// as that.
fn is_right_after(text: &[u16], phrase: &[u16]) -> bool {
    !text.is_empty() && text.ends_with(&phrase[phrase.len().saturating_sub(text.len())..])
}

/// The speaker paused mid-sentence: `last` stops on a word a sentence can't end on ("talk to."),
/// the model wrote `next` in lower case (it heard a fragment), or `next` opens with "and" or the
/// like. `last` ends with a full stop.
fn carries_on(last: &[u16], next: &[u16]) -> bool {
    let last_word = after_last(&last[..last.len() - 1], unit(' '));
    let first_word = first_word(next);
    OPEN_ENDINGS
        .iter()
        .any(|word| lower_equals_ascii(last_word, word))
        || first_word.first().is_some_and(|&u| is_lower_case(u))
        || JOINERS
            .iter()
            .any(|word| lower_equals_ascii(first_word, word))
}

/// Kotlin's `substringAfterLast(delimiter)`: all of `text` if it has none.
fn after_last(text: &[u16], delimiter: u16) -> &[u16] {
    text.iter()
        .rposition(|&u| u == delimiter)
        .map_or(text, |i| &text[i + 1..])
}

/// The cursor is at the start of a line below text ("new line"); the empty field is left to the
/// model.
fn starts_line(text: &[u16]) -> bool {
    text.contains(&unit('\n')) && is_blank(after_last(text, unit('\n')))
}

fn last_visible(text: &[u16]) -> Option<u16> {
    text.iter().rev().copied().find(|&u| !is_whitespace(u))
}

fn continues_sentence(text: &[u16]) -> bool {
    last_visible(text).is_some_and(|u| is_letter_or_digit(u) || u == unit(',') || u == unit(';'))
}

fn ends_sentence(text: &[u16]) -> bool {
    last_visible(text).is_some_and(|u| SENTENCE_ENDS.contains(&u))
}

/// Lower-cases the first word unless it is "I", an acronym, or a name.
fn lower_first(phrase: &[u16], is_name: impl FnOnce(&[u16]) -> bool) -> Vec<u16> {
    let word = first_word(phrase);
    let keeps_capital = word.is_empty()
        || word == [unit('I')]
        || word.starts_with(&[unit('I'), unit('\'')])
        || (word.len() > 1 && is_upper_case(word[1]))
        || is_name(word);
    if keeps_capital {
        phrase.to_vec()
    } else {
        replace_first(phrase, to_lower)
    }
}

/// The letters and apostrophes `text` starts with.
fn first_word(text: &[u16]) -> &[u16] {
    let length = text
        .iter()
        .take_while(|&&u| is_letter(u) || u == unit('\'') || u == RIGHT_QUOTE)
        .count();
    &text[..length]
}

/// ’, the apostrophe as typography writes it.
const RIGHT_QUOTE: u16 = 0x2019;

/// Kotlin's `replaceFirstChar(transform)`.
fn replace_first(text: &[u16], transform: fn(u16) -> u16) -> Vec<u16> {
    let mut replaced = text.to_vec();
    if let Some(first) = replaced.first_mut() {
        *first = transform(*first);
    }
    replaced
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utf16;

    fn typing(replacing: usize, text: &str) -> Typing {
        Typing {
            replacing,
            text: utf16(text),
        }
    }

    fn joined(before: &str, last: Option<&str>, phrase: &str, names: &[&str]) -> Typing {
        let last = last.map(utf16);
        join(&utf16(before), last.as_deref(), &utf16(phrase), |word| {
            names.iter().any(|name| utf16(name) == word)
        })
    }

    #[test]
    fn a_pause_mid_sentence_takes_the_full_stop_back() {
        let last = "I need to talk to.";
        assert_eq!(
            joined(last, Some(last), "John about it.", &["John"]),
            typing(1, " John about it.")
        );
        assert_eq!(
            joined(last, Some(last), "Bob about it.", &[]),
            typing(1, " bob about it.")
        );
    }

    #[test]
    fn a_new_sentence_keeps_the_full_stop_and_its_capital() {
        let last = "It rained.";
        assert_eq!(
            joined(last, Some(last), "Then it stopped.", &[]),
            typing(0, " Then it stopped.")
        );
    }

    #[test]
    fn a_lower_case_or_joining_word_carries_on() {
        let last = "It rained.";
        assert_eq!(
            joined(last, Some(last), "and then", &[]),
            typing(1, " and then")
        );
        assert_eq!(
            joined(last, Some(last), "But then", &[]),
            typing(1, " but then")
        );
    }

    #[test]
    fn a_spoken_full_stop_is_never_taken_back() {
        assert_eq!(
            joined("Hi.", Some("."), "and then", &[]),
            typing(0, " And then")
        );
    }

    #[test]
    fn the_cursor_elsewhere_keeps_the_full_stop() {
        assert_eq!(
            joined("Moved on.", Some("talk to."), "and then", &[]),
            typing(0, " And then")
        );
    }

    #[test]
    fn i_acronyms_and_names_keep_their_capitals() {
        assert_eq!(joined("so", None, "I think", &[]), typing(0, " I think"));
        assert_eq!(joined("so", None, "I'm sure", &[]), typing(0, " I'm sure"));
        assert_eq!(joined("so", None, "NASA did", &[]), typing(0, " NASA did"));
        assert_eq!(
            joined("so,", None, "Paris is", &["Paris"]),
            typing(0, " Paris is")
        );
        assert_eq!(joined("so;", None, "The end", &[]), typing(0, " the end"));
    }

    #[test]
    fn a_new_line_starts_with_a_capital_and_no_space() {
        assert_eq!(joined("Hi\n", None, "there", &[]), typing(0, "There"));
        assert_eq!(joined("", None, "there", &[]), typing(0, "there"));
        assert_eq!(joined("Hi ", None, "there", &[]), typing(0, "there"));
    }

    #[test]
    fn is_name_is_asked_only_when_it_decides() {
        let mut asked = Vec::new();
        join(&utf16("so"), None, &utf16("Paris is"), |word| {
            asked.push(word.to_vec());
            false
        });
        assert_eq!(asked, [utf16("Paris")]);
        join(&utf16("Hi."), None, &utf16("Paris is"), |_| {
            panic!("asked after a full stop")
        });
        join(&utf16("so"), None, &utf16("I am"), |_| {
            panic!("asked about I")
        });
    }

    #[test]
    fn a_mark_replaces_the_one_just_typed() {
        let last = "See you soon.";
        assert_eq!(
            punctuate(&utf16(last), Some(&utf16(last)), unit('?')),
            typing(1, "?")
        );
        assert_eq!(punctuate(&utf16(last), None, unit('?')), typing(0, "?"));
        assert_eq!(
            punctuate(&utf16("See you"), Some(&utf16("See you")), unit('?')),
            typing(0, "?")
        );
        assert_eq!(
            punctuate(&utf16("x"), Some(&utf16("soon.")), unit(',')),
            typing(0, ",")
        );
    }

    #[test]
    fn only_the_text_the_editor_shows_is_compared() {
        // The editor shows only "to." of a longer phrase, which still counts as right after it.
        assert_eq!(
            punctuate(&utf16("to."), Some(&utf16("talk to.")), unit('?')),
            typing(1, "?")
        );
        assert_eq!(
            punctuate(&utf16(""), Some(&utf16("to.")), unit('?')),
            typing(0, "?")
        );
    }
}
