//! Sentences as the keyboard sees text (UTF-16), and the words in them: the words the scoreboard
//! types, and the words the dictionary's statistics are counted over.
//!
//! A sentence is typed as it is written, punctuation included, so the keyboard's own rules decide
//! what context each word has (`PreviousWords::from`). The words scored are the letter words:
//! letters, with apostrophes or hyphens inside ("don't", "e-mail"). Numbers, symbols and the like
//! are typed as context but not scored.

/// One sentence in UTF-16, as the keyboard sees text.
#[derive(Clone, Debug)]
pub struct Sentence {
    pub text: Vec<u16>,
    /// The scored words, as UTF-16 ranges of `text`.
    pub words: Vec<(usize, usize)>,
}

impl Sentence {
    pub fn new(line: &str) -> Self {
        let text: Vec<u16> = line.trim().encode_utf16().collect();
        let mut words = Vec::new();
        let mut start = 0;
        while start < text.len() {
            if is_space(text[start]) {
                start += 1;
                continue;
            }
            let mut end = start;
            while end < text.len() && !is_space(text[end]) {
                end += 1;
            }
            if let Some(core) = word_core(&text[start..end]) {
                words.push((start + core.0, start + core.1));
            }
            start = end;
        }
        Sentence { text, words }
    }

    pub fn word(&self, index: usize) -> &[u16] {
        let (start, end) = self.words[index];
        &self.text[start..end]
    }

    /// The text before word `index`, all of it: earlier words, punctuation and spaces.
    pub fn before(&self, index: usize) -> &[u16] {
        &self.text[..self.words[index].0]
    }
}

fn is_space(unit: u16) -> bool {
    unit == u16::from(b' ') || unit == u16::from(b'\t')
}

/// Connectors that may sit inside a word.
fn is_connector(unit: u16) -> bool {
    unit == u16::from(b'\'') || unit == 0x2019 || unit == u16::from(b'-')
}

/// The letter word inside a whitespace token, without the punctuation around it, if the token
/// holds one: letters, with connectors only between letters.
fn word_core(token: &[u16]) -> Option<(usize, usize)> {
    let start = token.iter().position(|&u| chars::is_letter(u))?;
    let end = token.iter().rposition(|&u| chars::is_letter(u))? + 1;
    let core = &token[start..end];
    // "9pm" and "MP3" are not words to type with letters alone.
    let outside = token[..start].iter().chain(&token[end..]);
    if outside.clone().any(|&u| chars::is_letter_or_digit(u)) {
        return None;
    }
    let valid = core.iter().all(|&u| chars::is_letter(u) || is_connector(u))
        && !core
            .windows(2)
            .any(|w| is_connector(w[0]) && is_connector(w[1]));
    // A surrogate pair is no letter of ours: emoji and rare scripts stay context.
    (valid && core.iter().all(|&u| !(0xD800..0xE000).contains(&u))).then_some((start, end))
}

/// The sentences of a test file, one per line; blank lines are skipped.
pub fn sentences(contents: &str) -> Vec<Sentence> {
    contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(Sentence::new)
        .filter(|sentence| !sentence.words.is_empty())
        .collect()
}

pub fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

pub fn string(text: &[u16]) -> String {
    String::from_utf16_lossy(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(line: &str) -> Vec<String> {
        let sentence = Sentence::new(line);
        (0..sentence.words.len())
            .map(|i| string(sentence.word(i)))
            .collect()
    }

    #[test]
    fn scores_letter_words_without_their_punctuation() {
        assert_eq!(
            words("\"Don't go,\" she said (twice) at 9pm -- e-mail me!"),
            ["Don't", "go", "she", "said", "twice", "at", "e-mail", "me"]
        );
        assert!(words("42 1,000 :-)").is_empty());
        assert_eq!(words("café naïve"), ["café", "naïve"]);
    }

    #[test]
    fn the_text_before_a_word_is_everything_typed_so_far() {
        let sentence = Sentence::new("Well, I think so.");
        assert_eq!(string(sentence.before(0)), "");
        assert_eq!(string(sentence.before(1)), "Well, ");
        assert_eq!(string(sentence.before(3)), "Well, I think ");
    }

    #[test]
    fn reads_one_sentence_a_line() {
        let all = sentences("One two.\n\n  \n42\nThree.\n");
        assert_eq!(all.len(), 2);
        assert_eq!(all[1].words.len(), 1);
    }
}
