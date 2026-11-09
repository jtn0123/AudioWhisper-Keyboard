//! The words in the user's personal dictionaries (Floris's own and the system's): never
//! auto-corrected away, and offered as completions. A snapshot, small enough to scan. Was Kotlin's
//! `PersonalWords`, which still reads the dictionaries.

use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use chars::{equals_ignore_case, is_blank, lowercase, starts_with_ignore_case};

/// How many of the user's own words the candidate strip shows at most, so dictionary words still
/// get room.
pub const MAX_IN_STRIP: i32 = 2;

/// A personal dictionary row: `frequency` is 0-255, and `shortcut` types `word` when tapped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub word: Vec<u16>,
    pub frequency: i32,
    pub shortcut: Option<Vec<u16>>,
}

#[derive(Debug)]
pub struct PersonalWords {
    /// The entries with a word, most frequent first.
    entries: Vec<Entry>,
    /// Where each of `entries` was in the list it was made from.
    order: Vec<usize>,
    spellings: HashSet<Vec<u16>>,
    /// Each lower-cased word -> the most frequent of `entries` with it.
    by_lower_case: HashMap<Vec<u16>, usize>,
}

impl PersonalWords {
    /// A snapshot of `entries`, leaving out those whose word is blank.
    pub fn new(entries: Vec<Entry>) -> Self {
        let mut kept: Vec<(usize, Entry)> = entries
            .into_iter()
            .enumerate()
            .filter(|(_, entry)| !is_blank(&entry.word))
            .collect();
        // Stable, as Kotlin's sortedByDescending is: equally frequent words keep their order.
        kept.sort_by_key(|(_, entry)| Reverse(entry.frequency));
        let (order, entries): (Vec<usize>, Vec<Entry>) = kept.into_iter().unzip();
        let spellings = entries.iter().map(|entry| entry.word.clone()).collect();
        let mut by_lower_case = HashMap::new();
        for (index, entry) in entries.iter().enumerate() {
            by_lower_case.entry(lowercase(&entry.word)).or_insert(index);
        }
        PersonalWords {
            entries,
            order,
            spellings,
            by_lower_case,
        }
    }

    /// Where each entry kept, most frequent first, was in the list this was made from.
    pub fn order(&self) -> &[usize] {
        &self.order
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// How the user spells `word`: `word` itself if they wrote it that way, else their spelling
    /// ignoring case, else `None`.
    pub fn spelling_of(&self, word: &[u16]) -> Option<&[u16]> {
        if let Some(spelling) = self.spellings.get(word) {
            return Some(spelling);
        }
        let index = *self.by_lower_case.get(&lowercase(word))?;
        Some(&self.entries[index].word)
    }

    /// At most `max_count` words for `typed`, most frequent first: those whose shortcut it is,
    /// then those that start with it (ignoring case) and are longer. `typed` itself is never one.
    pub fn completions(&self, typed: &[u16], max_count: i32) -> Vec<&[u16]> {
        let Ok(max_count) = usize::try_from(max_count) else {
            return Vec::new();
        };
        if typed.is_empty() {
            return Vec::new();
        }
        let expansions = self.entries.iter().filter(|entry| {
            entry
                .shortcut
                .as_ref()
                .is_some_and(|shortcut| equals_ignore_case(shortcut, typed))
        });
        let longer = self.entries.iter().filter(|entry| {
            entry.word.len() > typed.len() && starts_with_ignore_case(&entry.word, typed)
        });
        let mut completions: Vec<&[u16]> = Vec::new();
        for entry in expansions.chain(longer) {
            if completions.len() == max_count {
                break;
            }
            if entry.word != typed && !completions.contains(&&entry.word[..]) {
                completions.push(&entry.word);
            }
        }
        completions
    }
}

/// Whether a row tagged `tag` ("en", "en_US", or none for every language) applies to `language`.
pub fn applies_to(tag: Option<&[u16]>, language: &[u16]) -> bool {
    match tag {
        None | Some([]) => true,
        Some(tag) => equals_ignore_case(before(before(tag, b'_'), b'-'), language),
    }
}

/// Kotlin's `substringBefore(delimiter)`: all of `text` if it has none.
fn before(text: &[u16], delimiter: u8) -> &[u16] {
    let end = text
        .iter()
        .position(|&u| u == u16::from(delimiter))
        .unwrap_or(text.len());
    &text[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    fn entry(word: &str, frequency: i32, shortcut: Option<&str>) -> Entry {
        Entry {
            word: utf16(word),
            frequency,
            shortcut: shortcut.map(utf16),
        }
    }

    fn words() -> PersonalWords {
        PersonalWords::new(vec![
            entry("Justin", 200, None),
            entry("iPhone", 250, None),
            entry("justinian", 100, None),
            entry("on my way", 150, Some("omw")),
            entry(" ", 255, None),
            entry("JUSTIN", 200, None),
        ])
    }

    fn strings(words: Vec<&[u16]>) -> Vec<String> {
        words.into_iter().map(String::from_utf16_lossy).collect()
    }

    #[test]
    fn keeps_the_words_most_frequent_first() {
        let words = words();
        assert_eq!(words.order(), [1, 0, 5, 3, 2]);
        assert!(!words.is_empty());
        assert!(PersonalWords::new(vec![entry("\t", 1, None)]).is_empty());
    }

    #[test]
    fn spells_a_word_the_users_way() {
        let words = words();
        let spelling = |word: &str| {
            words
                .spelling_of(&utf16(word))
                .map(String::from_utf16_lossy)
        };
        assert_eq!(spelling("JUSTIN").as_deref(), Some("JUSTIN"));
        // Either is the user's; the first of the most frequent wins.
        assert_eq!(spelling("justin").as_deref(), Some("Justin"));
        assert_eq!(spelling("IPHONE").as_deref(), Some("iPhone"));
        assert_eq!(spelling("just"), None);
    }

    #[test]
    fn completes_shortcuts_then_longer_words() {
        let words = words();
        let complete = |typed: &str, max: i32| strings(words.completions(&utf16(typed), max));
        assert_eq!(complete("ju", 5), ["Justin", "JUSTIN", "justinian"]);
        assert_eq!(complete("ju", 1), ["Justin"]);
        // Only longer words: not "Justin" itself, nor "JUSTIN".
        assert_eq!(complete("Justin", 5), ["justinian"]);
        assert_eq!(complete("OMW", 5), ["on my way"]);
        assert!(complete("", 5).is_empty());
        assert!(complete("ju", 0).is_empty());
        assert!(complete("ju", -1).is_empty());
    }

    #[test]
    fn a_shortcut_comes_first_but_never_gives_what_was_typed() {
        let words = PersonalWords::new(vec![
            entry("brbx", 250, None),
            entry("brb", 200, Some("brb")),
            entry("be right back", 100, Some("BRB")),
        ]);
        assert_eq!(
            strings(words.completions(&utf16("brb"), 5)),
            ["be right back", "brbx"]
        );
    }

    #[test]
    fn applies_to_its_language_or_to_all() {
        let en = utf16("en");
        assert!(applies_to(None, &en));
        assert!(applies_to(Some(&[]), &en));
        for (tag, applies) in [
            ("en_US", true),
            ("EN-GB", true),
            ("en", true),
            ("de_DE", false),
        ] {
            assert_eq!(applies_to(Some(&utf16(tag)), &en), applies, "{tag}");
        }
    }
}
