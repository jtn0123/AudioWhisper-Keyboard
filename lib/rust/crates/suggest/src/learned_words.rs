//! What the user has typed: how often each word was used and which word came before it. The
//! bundled dictionary knows at most three next words for any word and none of the user's names or
//! slang, so this is where the keyboard picks up their habits. At most a capacity of words and of
//! pairs; the least recently used go first. Was Kotlin's `LearnedWords`, which still reads and
//! writes the file: this parses and writes its text.
//!
//! Counts are Kotlin `Int`s, which wrap past the largest; so do these.

use std::cmp::Reverse;

use chars::lowercase;
use chars::text::to_int_or_null;

use crate::lines::lines;
use crate::ordered::Ordered;
use crate::word_characters::is_word;

/// The previous "word" of the first word in a sentence.
pub const SENTENCE_START: &[u16] = &[];

/// A word typed this many times is one of the user's words (Kotlin's `TypingHistory.KNOWN_AFTER`).
pub const KNOWN_AFTER: i32 = 2;

/// Enough for everyday vocabulary; the oldest go first.
pub const WORD_CAPACITY: i32 = 5000;
pub const PAIR_CAPACITY: i32 = 20_000;

/// Longer than any real word; longer runs of letters are not worth keeping.
pub const MAX_WORD_LENGTH: usize = 48;

const HEADER: &str = "# AudioWhisper learned words, v1";
const TAB: u16 = b'\t' as u16;
const WORD_LINE: &[u16] = &[b'w' as u16];
const PAIR_LINE: &[u16] = &[b'p' as u16];

/// A word ([`is_word`]: letters, with apostrophes or hyphens inside) of at most
/// [`MAX_WORD_LENGTH`].
pub fn is_storable(word: &[u16]) -> bool {
    (1..=MAX_WORD_LENGTH).contains(&word.len()) && is_word(word)
}

/// The word before the next one: the last word typed (`first_word`, the context's most recent),
/// else [`SENTENCE_START`] where a sentence starts, else `None`, as what came before is unknown.
pub fn previous_key(first_word: Option<&[u16]>, is_beginning_of_sentence: bool) -> Option<&[u16]> {
    first_word.or(is_beginning_of_sentence.then_some(SENTENCE_START))
}

/// How a word is filed: lower-cased, so any case finds it.
fn key(word: &[u16]) -> Vec<u16> {
    lowercase(word)
}

/// How a pair is filed: "previous\tword", lower-cased.
fn pair_key(previous: &[u16], word: &[u16]) -> Vec<u16> {
    let mut pair = key(previous);
    pair.push(TAB);
    pair.extend(key(word));
    pair
}

#[derive(Clone, Debug)]
struct Entry {
    spelling: Vec<u16>,
    count: i32,
}

/// Kotlin's `take` of a negative count, which throws: [`LearnedWords::followers`] was asked for
/// fewer than none.
#[derive(Debug, PartialEq, Eq)]
pub struct NegativeCount(pub i32);

/// What a saved file held ([`Saved::parse`]), for [`LearnedWords::merge`].
#[derive(Debug, Default)]
pub struct Saved {
    words: Ordered<Vec<u16>, Entry>,
    pairs: Ordered<Vec<u16>, i32>,
}

impl Saved {
    /// A saved file's `text`: a line "w, count, spelling" or "p, count, previous, word",
    /// tab-separated, for each word and pair. Lines that are neither are skipped; a later line
    /// for the same word or pair replaces an earlier one's count but not its place.
    pub fn parse(text: &[u16]) -> Saved {
        let mut saved = Saved::default();
        for line in lines(text) {
            saved.parse_line(line);
        }
        saved
    }

    fn parse_line(&mut self, line: &[u16]) {
        let fields: Vec<&[u16]> = line.split(|&u| u == TAB).collect();
        let Some(count) = fields
            .get(1)
            .and_then(|count| to_int_or_null(count))
            .filter(|&count| count > 0)
        else {
            return;
        };
        match fields[..] {
            [WORD_LINE, _, spelling] if is_storable(spelling) => {
                let spelling = spelling.to_vec();
                self.words.put(key(&spelling), Entry { spelling, count });
            }
            [PAIR_LINE, _, previous, word]
                if is_storable(word) && (previous == SENTENCE_START || is_storable(previous)) =>
            {
                self.pairs.put(pair_key(previous, word), count);
            }
            _ => {}
        }
    }
}

/// The user's words and word pairs, with how often each was used.
#[derive(Debug)]
pub struct LearnedWords {
    word_capacity: i32,
    pair_capacity: i32,
    /// Lower-cased word -> its spelling and use count, least recently used first.
    words: Ordered<Vec<u16>, Entry>,
    /// "previous\tword", lower-cased -> use count, least recently used first.
    pairs: Ordered<Vec<u16>, i32>,
}

impl LearnedWords {
    /// No words yet, keeping at most `word_capacity` words and `pair_capacity` pairs (none, if
    /// negative).
    pub fn new(word_capacity: i32, pair_capacity: i32) -> Self {
        LearnedWords {
            word_capacity,
            pair_capacity,
            words: Ordered::default(),
            pairs: Ordered::default(),
        }
    }

    /// Counts one use of `word` after `previous`: a word, [`SENTENCE_START`], or `None` when what
    /// came before is unknown. With `keep_spelling` an earlier spelling stands (a capital that may
    /// only be there because the word starts a sentence).
    pub fn add(&mut self, word: &[u16], previous: Option<&[u16]>, keep_spelling: bool) {
        if !is_storable(word) {
            return;
        }
        let key = key(word);
        let entry = match self.words.remove(&key) {
            Some(mut entry) => {
                entry.count = entry.count.wrapping_add(1);
                if !keep_spelling {
                    entry.spelling = word.to_vec();
                }
                entry
            }
            None => Entry {
                spelling: word.to_vec(),
                count: 1,
            },
        };
        self.words.put(key, entry);
        self.words.trim(self.word_capacity);
        if let Some(previous) = previous
            && (previous == SENTENCE_START || is_storable(previous))
        {
            let pair = pair_key(previous, word);
            let count = self.pairs.remove(&pair).unwrap_or(0).wrapping_add(1);
            self.pairs.put(pair, count);
            self.pairs.trim(self.pair_capacity);
        }
    }

    /// Takes back one [`LearnedWords::add`] of `word` after `previous`.
    pub fn remove(&mut self, word: &[u16], previous: Option<&[u16]>) {
        let key = key(word);
        if let Some(entry) = self.words.get_mut(&key) {
            entry.count = entry.count.wrapping_sub(1);
            if entry.count <= 0 {
                self.words.remove(&key);
            }
        }
        let Some(previous) = previous else {
            return;
        };
        let pair = pair_key(previous, word);
        match self.pairs.get(&pair) {
            Some(&count) if count <= 1 => {
                self.pairs.remove(&pair);
            }
            Some(&count) => self.pairs.put(pair, count - 1),
            None => {}
        }
    }

    pub fn clear(&mut self) {
        self.words.clear();
        self.pairs.clear();
    }

    /// How the user spells each word they have used [`KNOWN_AFTER`] times, least recently used
    /// first.
    pub fn known_spellings(&self) -> Vec<&[u16]> {
        self.words
            .iter()
            .filter(|(_, entry)| entry.count >= KNOWN_AFTER)
            .map(|(_, entry)| &entry.spelling[..])
            .collect()
    }

    /// How the user spells `word` (ignoring case), once they have used it [`KNOWN_AFTER`] times.
    pub fn spelling_of(&self, word: &[u16]) -> Option<&[u16]> {
        self.words
            .get(&key(word))
            .filter(|entry| entry.count >= KNOWN_AFTER)
            .map(|entry| &entry.spelling[..])
    }

    /// The words used [`KNOWN_AFTER`] times that start with `typed` (ignoring case) and are
    /// longer, most used first and, among those used as often, most recently used first. None if
    /// `typed` is empty or `max_count` isn't positive; otherwise all of them, for the caller to
    /// keep the first `max_count` it accepts.
    pub fn completions(&self, typed: &[u16], max_count: i32) -> Vec<&[u16]> {
        if typed.is_empty() || max_count <= 0 {
            return Vec::new();
        }
        let prefix = key(typed);
        let matching = self.words.iter().filter(|(key, entry)| {
            entry.count >= KNOWN_AFTER && key.len() > prefix.len() && key.starts_with(&prefix)
        });
        most_used_first(matching.map(|(_, entry)| (&entry.spelling[..], entry.count)))
    }

    /// Up to `max_count` words typed right after the previous word (see [`previous_key`]) at least
    /// `min_count` times, most used first and then most recent, in the user's spelling.
    pub fn followers(
        &self,
        first_word: Option<&[u16]>,
        is_beginning_of_sentence: bool,
        min_count: i32,
        max_count: i32,
    ) -> Result<Vec<&[u16]>, NegativeCount> {
        let Some(previous) = previous_key(first_word, is_beginning_of_sentence) else {
            return Ok(Vec::new());
        };
        let max_count = usize::try_from(max_count).map_err(|_| NegativeCount(max_count))?;
        let mut prefix = key(previous);
        prefix.push(TAB);
        let matching = self
            .pairs
            .iter()
            .filter(|&(pair, &count)| count >= min_count && pair.starts_with(&prefix))
            .map(|(pair, &count)| (&pair[prefix.len()..], count));
        let mut followers = most_used_first(matching);
        followers.truncate(max_count);
        for word in &mut followers {
            if let Some(entry) = self.words.get(*word) {
                *word = &entry.spelling[..];
            }
        }
        Ok(followers)
    }

    /// How many times `word` was typed right after the previous word (see [`previous_key`]).
    pub fn pair_count(
        &self,
        first_word: Option<&[u16]>,
        is_beginning_of_sentence: bool,
        word: &[u16],
    ) -> i32 {
        previous_key(first_word, is_beginning_of_sentence)
            .and_then(|previous| self.pairs.get(&pair_key(previous, word)))
            .copied()
            .unwrap_or(0)
    }

    /// Takes in what a file held, keeping what was learned before it was read: a word or pair in
    /// both adds the counts, keeps the spelling learned since, and counts as most recently used.
    pub fn merge(&mut self, saved: Saved) {
        let Saved {
            mut words,
            mut pairs,
        } = saved;
        for (key, mut entry) in std::mem::take(&mut self.words).into_entries() {
            if let Some(saved) = words.remove(&key) {
                entry.count = entry.count.wrapping_add(saved.count);
            }
            words.put(key, entry);
        }
        for (pair, count) in std::mem::take(&mut self.pairs).into_entries() {
            let saved = pairs.remove(&pair).unwrap_or(0);
            pairs.put(pair, saved.wrapping_add(count));
        }
        self.words = words;
        self.pairs = pairs;
        self.words.trim(self.word_capacity);
        self.pairs.trim(self.pair_capacity);
    }

    /// The text to save: a header, then every word and every pair (see [`Saved::parse`]).
    pub fn saved_text(&self) -> Vec<u16> {
        let mut text: Vec<u16> = HEADER.encode_utf16().collect();
        text.push(u16::from(b'\n'));
        for (_, entry) in self.words.iter() {
            push_line(&mut text, WORD_LINE, entry.count, &entry.spelling);
        }
        for (pair, &count) in self.pairs.iter() {
            push_line(&mut text, PAIR_LINE, count, pair);
        }
        text
    }
}

/// `matching`, oldest first, ordered most used first and then most recent first, as Kotlin's
/// stable `asReversed().sortedByDescending { count }` orders it.
fn most_used_first<'a>(
    matching: impl DoubleEndedIterator<Item = (&'a [u16], i32)>,
) -> Vec<&'a [u16]> {
    let mut matching: Vec<_> = matching.rev().collect();
    matching.sort_by_key(|&(_, count)| Reverse(count));
    matching.into_iter().map(|(word, _)| word).collect()
}

fn push_line(text: &mut Vec<u16>, kind: &[u16], count: i32, value: &[u16]) {
    text.extend_from_slice(kind);
    text.push(TAB);
    text.extend(count.to_string().encode_utf16());
    text.push(TAB);
    text.extend_from_slice(value);
    text.push(u16::from(b'\n'));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    fn strings(words: &[&[u16]]) -> Vec<String> {
        words.iter().map(|w| String::from_utf16_lossy(w)).collect()
    }

    fn add(learned: &mut LearnedWords, word: &str, previous: Option<&str>, times: usize) {
        let previous = previous.map(utf16);
        for _ in 0..times {
            learned.add(&utf16(word), previous.as_deref(), false);
        }
    }

    fn after(word: &str) -> Vec<u16> {
        utf16(word)
    }

    #[test]
    fn a_word_becomes_the_users_once_used_twice() {
        let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
        add(&mut learned, "Zorblax", None, 1);
        assert_eq!(learned.spelling_of(&utf16("zorblax")), None);
        assert!(learned.completions(&utf16("zor"), 2).is_empty());
        add(&mut learned, "Zorblax", None, 1);
        assert_eq!(strings(&learned.known_spellings()), ["Zorblax"]);
        assert_eq!(
            learned.spelling_of(&utf16("ZORBLAX")),
            Some(&utf16("Zorblax")[..])
        );
        assert_eq!(strings(&learned.completions(&utf16("ZOR"), 2)), ["Zorblax"]);
        assert!(learned.completions(&utf16("zorblax"), 2).is_empty());
        assert!(learned.completions(&utf16("zor"), 0).is_empty());
        assert!(learned.completions(&[], 2).is_empty());
    }

    #[test]
    fn ranks_most_used_then_most_recent() {
        let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
        add(&mut learned, "formica", None, 3);
        add(&mut learned, "formal", None, 2);
        add(&mut learned, "formula", None, 2);
        assert_eq!(
            strings(&learned.completions(&utf16("form"), 1)),
            ["formica", "formula", "formal"]
        );
        add(&mut learned, "later", Some("you"), 2);
        add(&mut learned, "there", Some("you"), 1);
        add(&mut learned, "Hey", Some(""), 1);
        let you = after("You");
        assert_eq!(
            strings(&learned.followers(Some(&you), false, 1, 3).unwrap()),
            ["later", "there"]
        );
        assert_eq!(
            strings(&learned.followers(Some(&you), false, 1, 1).unwrap()),
            ["later"]
        );
        assert_eq!(
            strings(&learned.followers(None, true, 1, 3).unwrap()),
            ["Hey"]
        );
        assert!(learned.followers(None, false, 1, -1).unwrap().is_empty());
        assert_eq!(
            learned.followers(Some(&you), false, 1, -1),
            Err(NegativeCount(-1))
        );
        assert_eq!(
            learned.pair_count(Some(&after("YOU")), true, &utf16("Later")),
            2
        );
        assert_eq!(learned.pair_count(None, false, &utf16("later")), 0);
    }

    #[test]
    fn remove_takes_back_one_use() {
        let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
        add(&mut learned, "later", Some("you"), 2);
        let you = after("you");
        learned.remove(&utf16("later"), Some(&you));
        assert_eq!(learned.spelling_of(&utf16("later")), None);
        assert_eq!(learned.pair_count(Some(&you), false, &utf16("later")), 1);
        learned.remove(&utf16("later"), Some(&you));
        assert_eq!(learned.pair_count(Some(&you), false, &utf16("later")), 0);
        learned.remove(&utf16("never"), Some(&utf16("seen")));
        // Nothing is left at zero.
        assert_eq!(
            String::from_utf16_lossy(&learned.saved_text()),
            "# AudioWhisper learned words, v1\n"
        );
    }

    #[test]
    fn keeping_the_spelling_keeps_the_earlier_one() {
        let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
        add(&mut learned, "iPhone", None, 1);
        learned.add(&utf16("IPhone"), None, true);
        assert_eq!(
            learned.spelling_of(&utf16("iphone")),
            Some(&utf16("iPhone")[..])
        );
        learned.add(&utf16("IPhone"), None, false);
        assert_eq!(
            learned.spelling_of(&utf16("iphone")),
            Some(&utf16("IPhone")[..])
        );
    }

    #[test]
    fn stores_only_words() {
        let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
        let too_long = "x".repeat(MAX_WORD_LENGTH + 1);
        for word in ["mp3", "a b", "tab\there", &too_long, "--", ""] {
            add(&mut learned, word, Some("previous"), 2);
            assert_eq!(learned.spelling_of(&utf16(word)), None, "{word}");
        }
        assert!(is_storable(&utf16(&"x".repeat(MAX_WORD_LENGTH))));
        assert!(is_storable(&utf16("rock’n’roll")));
        add(&mut learned, "don't", Some("mp3"), 2);
        assert_eq!(
            learned.pair_count(Some(&utf16("mp3")), false, &utf16("don't")),
            0
        );
    }

    #[test]
    fn the_least_recently_used_go_first() {
        let mut learned = LearnedWords::new(2, 1);
        add(&mut learned, "alpha", Some("one"), 2);
        add(&mut learned, "bravo", Some("two"), 2);
        add(&mut learned, "alpha", None, 2);
        add(&mut learned, "charlie", None, 2);
        assert_eq!(strings(&learned.known_spellings()), ["alpha", "charlie"]);
        assert_eq!(
            learned.pair_count(Some(&utf16("one")), false, &utf16("alpha")),
            0
        );
        assert_eq!(
            learned.pair_count(Some(&utf16("two")), false, &utf16("bravo")),
            2
        );
    }

    #[test]
    fn counts_wrap_as_kotlin_ints_do() {
        let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
        learned.merge(Saved::parse(&utf16(
            "w\t2147483647\tbig\np\t2147483647\t\tbig\n",
        )));
        add(&mut learned, "big", Some(""), 1);
        assert_eq!(learned.spelling_of(&utf16("big")), None);
        assert_eq!(learned.pair_count(None, true, &utf16("big")), i32::MIN);
        // One fewer than the smallest Int is the largest: still used, and plenty.
        learned.remove(&utf16("big"), None);
        assert_eq!(learned.spelling_of(&utf16("big")), Some(&utf16("big")[..]));
    }

    #[test]
    fn saves_and_parses_everything() {
        let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
        add(&mut learned, "Zorblax", Some(""), 2);
        add(&mut learned, "later", Some("You"), 3);
        let text = learned.saved_text();
        assert_eq!(
            String::from_utf16_lossy(&text),
            "# AudioWhisper learned words, v1\nw\t2\tZorblax\nw\t3\tlater\np\t2\t\tzorblax\np\t3\tyou\tlater\n"
        );
        let mut reloaded = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
        reloaded.merge(Saved::parse(&text));
        assert_eq!(reloaded.saved_text(), text);
    }

    #[test]
    fn merging_keeps_what_was_learned_since() {
        let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
        add(&mut learned, "Later", Some("you"), 1);
        add(&mut learned, "new", None, 1);
        learned.merge(Saved::parse(&utf16(
            "w\t1\tlater\nw\t5\told\nw\t2\tLATER\np\t1\tyou\tlater\n",
        )));
        assert_eq!(
            String::from_utf16_lossy(&learned.saved_text()),
            "# AudioWhisper learned words, v1\nw\t5\told\nw\t3\tLater\nw\t1\tnew\np\t2\tyou\tlater\n"
        );
    }

    #[test]
    fn merging_keeps_to_the_capacity() {
        let mut learned = LearnedWords::new(1, 1);
        add(&mut learned, "new", Some("brand"), 1);
        learned.merge(Saved::parse(&utf16("w\t1\told\np\t1\tvery\told\n")));
        assert_eq!(
            String::from_utf16_lossy(&learned.saved_text()),
            "# AudioWhisper learned words, v1\nw\t1\tnew\np\t1\tbrand\tnew\n"
        );
    }

    #[test]
    fn skips_damaged_lines() {
        let saved = Saved::parse(&utf16(
            "# header\nw\tx\tword\nw\t2\tgood\np\t1\tonly-three\nq\t1\ta\tb\nw\t0\tzero\nw\t-1\tminus\nw\t+3\tplus\nw\t٣\tarabic\nw\t2\tgood\textra\np\t1\tmp3\tword\n\n",
        ));
        let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
        learned.merge(saved);
        assert_eq!(
            String::from_utf16_lossy(&learned.saved_text()),
            "# AudioWhisper learned words, v1\nw\t2\tgood\nw\t3\tplus\nw\t3\tarabic\n"
        );
    }

    #[test]
    fn the_previous_word_is_the_last_one_or_a_sentence_start() {
        let word = utf16("you");
        assert_eq!(previous_key(Some(&word), true), Some(&word[..]));
        assert_eq!(previous_key(None, true), Some(SENTENCE_START));
        assert_eq!(previous_key(None, false), None);
    }
}
