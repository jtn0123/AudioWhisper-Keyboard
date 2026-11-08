//! Spoken commands: "new line", "comma", "scratch that".

use chars::{is_blank, is_regex_space, lower_equals_ascii, trim};

use crate::unit;

/// What part of a dictated phrase asks for: words to type, or a spoken command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Spoken {
    /// Words to type, as transcribed.
    Words(Vec<u16>),
    /// "Comma", "question mark": put in place of the punctuation the dictation just typed.
    Mark(u16),
    /// "New line", "new paragraph": the line breaks to type.
    Break(&'static str),
    /// "Scratch that": take back the phrase typed last.
    ScratchThat,
}

/// The commands, in lower case with single spaces.
static COMMANDS: [(&str, Spoken); 14] = [
    ("new line", Spoken::Break("\n")),
    ("newline", Spoken::Break("\n")),
    ("next line", Spoken::Break("\n")),
    ("new paragraph", Spoken::Break("\n\n")),
    ("scratch that", Spoken::ScratchThat),
    ("delete that", Spoken::ScratchThat),
    ("period", Spoken::Mark(unit('.'))),
    ("full stop", Spoken::Mark(unit('.'))),
    ("comma", Spoken::Mark(unit(','))),
    ("question mark", Spoken::Mark(unit('?'))),
    ("exclamation mark", Spoken::Mark(unit('!'))),
    ("exclamation point", Spoken::Mark(unit('!'))),
    ("colon", Spoken::Mark(unit(':'))),
    ("semicolon", Spoken::Mark(unit(';'))),
];

/// Splits `phrase` into words and commands. A command counts only as a sentence of its own
/// ("Hello there. New line. How are you?"), never inside one ("I said new line"), because the
/// model only writes it that way when the speaker says it on its own.
pub fn parse(phrase: &[u16]) -> Vec<Spoken> {
    let mut parts = Vec::new();
    let mut words: Vec<&[u16]> = Vec::new();
    for sentence in sentences(trim(phrase)) {
        match command(sentence) {
            None if !is_blank(sentence) => words.push(sentence),
            None => {}
            Some(command) => {
                flush(&mut parts, &mut words);
                parts.push(command.clone());
            }
        }
    }
    flush(&mut parts, &mut words);
    parts
}

fn flush(parts: &mut Vec<Spoken>, words: &mut Vec<&[u16]>) {
    if !words.is_empty() {
        parts.push(Spoken::Words(words.join(&unit(' '))));
        words.clear();
    }
}

/// Kotlin's `split(Regex("(?<=[.!?])\\s+"))`: the text between runs of whitespace (the regex's
/// `\s`, which differs on Android) that follow a full stop, exclamation mark or question mark.
fn sentences(text: &[u16]) -> Vec<&[u16]> {
    let mut sentences = Vec::new();
    let (mut start, mut at) = (0, 1);
    while at < text.len() {
        if is_regex_space(text[at]) && [unit('.'), unit('!'), unit('?')].contains(&text[at - 1]) {
            let end = at
                + text[at..]
                    .iter()
                    .take_while(|&&u| is_regex_space(u))
                    .count();
            sentences.push(&text[start..at]);
            (start, at) = (end, end);
        } else {
            at += 1;
        }
    }
    sentences.push(&text[start..]);
    sentences
}

/// The command `sentence` says, if it is one. Kotlin looked it up by
/// `sentence.trim().trimEnd('.', '!', '?', ',').trim().lowercase().replace(Regex("\\s+"), " ")`.
/// Lower-casing never makes or removes `\s` whitespace, so the runs of it are squeezed first and
/// the result compared, lower-cased, with each command.
fn command(sentence: &[u16]) -> Option<&'static Spoken> {
    let marks = [unit('.'), unit('!'), unit('?'), unit(',')];
    let trimmed = trim(sentence);
    let end = trimmed
        .iter()
        .rposition(|u| !marks.contains(u))
        .map_or(0, |i| i + 1);
    let key = squeeze_regex_spaces(trim(&trimmed[..end]));
    COMMANDS
        .iter()
        .find(|(name, _)| lower_equals_ascii(&key, name))
        .map(|(_, command)| command)
}

/// Each run of `\s` whitespace as one space.
fn squeeze_regex_spaces(text: &[u16]) -> Vec<u16> {
    let mut squeezed = Vec::with_capacity(text.len());
    for &u in text {
        if !is_regex_space(u) {
            squeezed.push(u);
        } else if squeezed.last().is_none_or(|&last| !is_regex_space(last)) {
            squeezed.push(unit(' '));
        }
    }
    squeezed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utf16;

    fn words(text: &str) -> Spoken {
        Spoken::Words(utf16(text))
    }

    fn parsed(phrase: &str) -> Vec<Spoken> {
        parse(&utf16(phrase))
    }

    #[test]
    fn plain_words_are_one_part() {
        assert_eq!(
            parsed("  Hello there. How are you?  "),
            [words("Hello there. How are you?")]
        );
    }

    #[test]
    fn a_command_on_its_own_splits_the_words() {
        assert_eq!(
            parsed("Hello there. New line. How are you?"),
            [
                words("Hello there."),
                Spoken::Break("\n"),
                words("How are you?")
            ]
        );
    }

    #[test]
    fn a_command_inside_a_sentence_is_words() {
        assert_eq!(parsed("I said new line."), [words("I said new line.")]);
    }

    #[test]
    fn commands_ignore_case_spacing_and_end_punctuation() {
        assert_eq!(parsed("QUESTION \t MARK!?,"), [Spoken::Mark(unit('?'))]);
        assert_eq!(parsed("Scratch that."), [Spoken::ScratchThat]);
        assert_eq!(parsed("New paragraph"), [Spoken::Break("\n\n")]);
    }

    #[test]
    fn only_ascii_whitespace_after_a_stop_splits() {
        // On the JVM, whose tables cargo test uses, a no-break space isn't the regex's \s, so
        // this is one sentence and no command.
        assert_eq!(parsed("Hi.\u{A0}Comma."), [words("Hi.\u{A0}Comma.")]);
        assert_eq!(
            parsed("Hi.\n\tComma."),
            [words("Hi."), Spoken::Mark(unit(','))]
        );
    }

    #[test]
    fn blank_and_empty_give_nothing() {
        assert_eq!(parsed(""), []);
        assert_eq!(parsed(" \u{3000} "), []);
    }

    #[test]
    fn splits_keep_kotlins_pieces() {
        let pieces = |text: &str| sentences(&utf16(text)).len();
        assert_eq!(pieces("a. b! c? d"), 4);
        assert_eq!(pieces("a.b"), 1);
        assert_eq!(pieces(". a"), 2);
        assert_eq!(pieces(""), 1);
    }
}
