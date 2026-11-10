//! `com_android_inputmethod_latin_BinaryDictionary`'s `getWords` and its `WordCollector`: every word
//! of a dictionary as one UTF-16 string, each word followed by a newline. A word with a control
//! character, or a code point outside Unicode, is left out.

use super::dictionary::Dictionary;
use super::interface::WordListener;
use crate::chars::is_in_unicode_space;

/// `WordCollector`: collects the words it is given as UTF-16, one line per word.
#[derive(Debug, Default)]
pub struct WordCollector {
    chars: Vec<u16>,
}

impl WordCollector {
    /// `WordCollector()`: nothing collected.
    pub fn new() -> Self {
        Self::default()
    }

    /// The text collected so far.
    pub fn utf16(&self) -> &[u16] {
        &self.chars
    }
}

impl WordListener for WordCollector {
    /// `WordCollector::onVisitWord`. A control character (below the space) or a code point outside
    /// Unicode drops the word. A code point of 0x10000 or more is written as a surrogate pair.
    fn on_visit_word(&mut self, word: &[i32]) {
        if word
            .iter()
            .any(|&code_point| code_point < i32::from(b' ') || !is_in_unicode_space(code_point))
        {
            return;
        }
        for &code_point in word {
            if code_point < 0x10000 {
                self.chars.push(code_point as u16);
            } else {
                let offset = code_point - 0x10000;
                self.chars.push((0xD800 + (offset >> 10)) as u16);
                self.chars.push((0xDC00 + (offset & 0x3FF)) as u16);
            }
        }
        self.chars.push(u16::from(b'\n'));
    }
}

/// `latinime_BinaryDictionary_getWords`: every word of `dictionary`, as UTF-16 with a newline after
/// each.
pub fn get_words(dictionary: &Dictionary) -> Vec<u16> {
    let mut collector = WordCollector::new();
    dictionary.iterate_words(&mut collector);
    collector.chars
}
