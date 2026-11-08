//! Words whose auto-correction the user undid, so they are not corrected again: lower-cased, newest
//! last, at most a capacity of them. A word kept without remembering it (incognito) lasts only as
//! long as the store and is never saved. Was Kotlin's `KeptWords`, which still reads and writes the
//! file, one word per line: this parses and writes its text.

use std::collections::HashSet;

use chars::{is_blank, lowercase, trim};

use crate::lines::lines;
use crate::ordered::Ordered;

/// Enough for years of undos; the oldest go first.
pub const CAPACITY: i32 = 1000;

const LINE_FEED: u16 = b'\n' as u16;

#[derive(Debug)]
pub struct KeptWords {
    capacity: i32,
    remembered: Ordered<Vec<u16>, ()>,
    session_only: HashSet<Vec<u16>>,
}

/// The words a saved file held ([`KeptWords::parse`]), oldest first.
#[derive(Debug)]
pub struct Saved(Vec<Vec<u16>>);

impl KeptWords {
    /// None kept yet, remembering at most `capacity` (none, if negative).
    pub fn new(capacity: i32) -> Self {
        KeptWords {
            capacity,
            remembered: Ordered::default(),
            session_only: HashSet::new(),
        }
    }

    /// Whether `word` is kept, in any case.
    pub fn contains(&self, word: &[u16]) -> bool {
        let key = lowercase(word);
        self.remembered.contains_key(&key) || self.session_only.contains(&key)
    }

    /// Keeps `word`, saving it if `remember`; true when that changed what [`KeptWords::saved_text`]
    /// writes. A blank word, or one with a line break, isn't kept.
    pub fn add(&mut self, word: &[u16], remember: bool) -> bool {
        let key = lowercase(word);
        if is_blank(&key) || key.contains(&LINE_FEED) {
            return false;
        }
        if !remember {
            self.session_only.insert(key);
            return false;
        }
        self.session_only.remove(&key);
        self.remembered.remove(&key);
        self.remembered.put(key, ());
        self.remembered.trim(self.capacity);
        true
    }

    /// A saved file's `text`: a word per line, around which space doesn't count.
    pub fn parse(text: &[u16]) -> Saved {
        let words = lines(text)
            .into_iter()
            .map(trim)
            .filter(|word| !word.is_empty())
            .map(lowercase);
        Saved(words.collect())
    }

    /// Takes in what a file held, ahead of the words remembered before it was read.
    pub fn merge(&mut self, saved: Saved) {
        let added: Vec<Vec<u16>> = std::mem::take(&mut self.remembered)
            .into_entries()
            .map(|(word, ())| word)
            .collect();
        for word in saved.0 {
            self.remembered.put(word, ());
        }
        for word in &added {
            self.remembered.remove(word);
        }
        for word in added {
            self.remembered.put(word, ());
        }
        self.remembered.trim(self.capacity);
    }

    /// The text to save: each remembered word on a line of its own, oldest first.
    pub fn saved_text(&self) -> Vec<u16> {
        let mut text = Vec::new();
        for (index, (word, ())) in self.remembered.iter().enumerate() {
            if index > 0 {
                text.push(LINE_FEED);
            }
            text.extend_from_slice(word);
        }
        text.push(LINE_FEED);
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    fn saved(kept: &KeptWords) -> String {
        String::from_utf16_lossy(&kept.saved_text())
    }

    #[test]
    fn keeps_a_word_in_any_case() {
        let mut kept = KeptWords::new(CAPACITY);
        assert!(kept.add(&utf16("Teh"), true));
        assert!(kept.contains(&utf16("TEH")));
        assert!(!kept.contains(&utf16("the")));
        assert_eq!(saved(&kept), "teh\n");
    }

    #[test]
    fn never_saves_an_incognito_word() {
        let mut kept = KeptWords::new(CAPACITY);
        assert!(!kept.add(&utf16("secret"), false));
        assert!(kept.contains(&utf16("Secret")));
        assert_eq!(saved(&kept), "\n");
        // Remembering it later moves it to the saved words.
        assert!(kept.add(&utf16("secret"), true));
        assert_eq!(saved(&kept), "secret\n");
    }

    #[test]
    fn keeps_no_blank_word_or_line_break() {
        let mut kept = KeptWords::new(CAPACITY);
        for word in ["", " \t", "a\nb"] {
            assert!(!kept.add(&utf16(word), true), "{word:?}");
        }
        assert!(kept.add(&utf16("a\rb"), true));
    }

    #[test]
    fn keeps_the_newest_when_full() {
        let mut kept = KeptWords::new(2);
        for word in ["one", "two", "one", "three"] {
            kept.add(&utf16(word), true);
        }
        assert_eq!(saved(&kept), "one\nthree\n");
        let mut none = KeptWords::new(-1);
        assert!(none.add(&utf16("one"), true));
        assert!(!none.contains(&utf16("one")));
    }

    #[test]
    fn merging_puts_the_saved_words_first() {
        let mut kept = KeptWords::new(CAPACITY);
        kept.add(&utf16("early"), true);
        kept.add(&utf16("both"), true);
        kept.merge(KeptWords::parse(&utf16(
            "  Saved \r\n\nboth\nSAVED\n\u{00A0}\nlast",
        )));
        assert_eq!(saved(&kept), "saved\nlast\nearly\nboth\n");
    }
}
