//! Counting the dictionary's words, and the words that follow them, in a corpus.
//!
//! Each sentence is read the way the keyboard reads what was typed: the words are
//! `text::Sentence`'s letter words, and the word before each one is what
//! `PreviousWords::from` finds there, looked up as the keyboard looks it up (as it is, then
//! lower-cased). So a pair is counted exactly when the keyboard, typing that sentence, would ask
//! the dictionary for it.

use crate::text::Sentence;
use crate::v202::Dictionary;
use std::collections::HashMap;
use suggest::previous_words::PreviousWords;

/// The dictionary's words by spelling, in UTF-16 as the keyboard sees them.
#[derive(Debug)]
pub struct Vocabulary {
    index: HashMap<Vec<u16>, u32>,
}

impl Vocabulary {
    pub fn new(dictionary: &Dictionary) -> Self {
        let mut index = HashMap::with_capacity(dictionary.entries.len());
        for (i, entry) in dictionary.entries.iter().enumerate() {
            let mut word = Vec::with_capacity(entry.word.len());
            for &c in &entry.word {
                chars::code_point::push_utf16(&mut word, c);
            }
            index.insert(word, i as u32);
        }
        Vocabulary { index }
    }

    pub fn exact(&self, word: &[u16]) -> Option<u32> {
        self.index.get(word).copied()
    }

    /// The word as it is, else lower-cased: how the keyboard finds a previous word (`word_id`
    /// with `try_lower_case`), and how a word mid-sentence is counted.
    pub fn find(&self, word: &[u16]) -> Option<u32> {
        self.exact(word).or_else(|| self.lower(word))
    }

    fn lower(&self, word: &[u16]) -> Option<u32> {
        let lower = chars::lowercase(word);
        (lower != word).then(|| self.exact(&lower)).flatten()
    }
}

/// What the corpus says about the dictionary's words.
#[derive(Debug, Default)]
pub struct Counts {
    /// Times each entry was written, by index; fractional where a capital at a sentence's start
    /// left the case open (see [`Counts::settle_case`]).
    pub words: Vec<f64>,
    /// Times each entry was the keyboard's previous word for a following letter word.
    pub contexts: Vec<u64>,
    /// Times each entry followed another: `previous << 32 | next`.
    pub pairs: HashMap<u64, u32>,
    /// Letter words read, in the dictionary or not.
    pub tokens: u64,
    /// A capitalised word starting a sentence, spelled both ways in the dictionary ("Will" and
    /// "will"): `exact << 32 | lower`, settled once every sentence is counted.
    open_case: HashMap<u64, u32>,
}

pub fn pair(previous: u32, next: u32) -> u64 {
    u64::from(previous) << 32 | u64::from(next)
}

pub fn unpair(key: u64) -> (u32, u32) {
    ((key >> 32) as u32, key as u32)
}

impl Counts {
    fn new(entries: usize) -> Self {
        Counts {
            words: vec![0.0; entries],
            contexts: vec![0; entries],
            ..Counts::default()
        }
    }

    fn sentence(&mut self, vocabulary: &Vocabulary, line: &str) {
        let sentence = Sentence::new(line);
        for i in 0..sentence.words.len() {
            self.tokens += 1;
            let word = sentence.word(i);
            let previous = PreviousWords::from(sentence.before(i));
            let starts = previous.words.is_empty() && previous.is_beginning_of_sentence;
            let Some(next) = self.word(vocabulary, word, starts) else {
                if let Some(p) = previous.words.first().and_then(|w| vocabulary.find(w)) {
                    self.contexts[p as usize] += 1;
                }
                continue;
            };
            if let Some(p) = previous.words.first().and_then(|w| vocabulary.find(w)) {
                self.contexts[p as usize] += 1;
                *self.pairs.entry(pair(p, next)).or_default() += 1;
            }
        }
    }

    /// Counts one word and returns its entry. At a sentence's start a capital says nothing, so a
    /// word in the dictionary both ways is set aside for [`Counts::settle_case`].
    fn word(&mut self, vocabulary: &Vocabulary, word: &[u16], starts: bool) -> Option<u32> {
        let exact = vocabulary.exact(word);
        let lower = vocabulary.lower(word);
        let found = match (exact, lower) {
            (Some(e), Some(l)) if starts => {
                *self.open_case.entry(pair(e, l)).or_default() += 1;
                return Some(e);
            }
            (Some(e), _) => e,
            (None, Some(l)) => l,
            (None, None) => return None,
        };
        self.words[found as usize] += 1.0;
        Some(found)
    }

    fn merge(&mut self, other: Counts) {
        for (mine, theirs) in self.words.iter_mut().zip(other.words) {
            *mine += theirs;
        }
        for (mine, theirs) in self.contexts.iter_mut().zip(other.contexts) {
            *mine += theirs;
        }
        for (key, n) in other.pairs {
            *self.pairs.entry(key).or_default() += n;
        }
        for (key, n) in other.open_case {
            *self.open_case.entry(key).or_default() += n;
        }
        self.tokens += other.tokens;
    }

    /// Shares each sentence-initial capital between its two spellings as the rest of the corpus
    /// writes them mid-sentence ("Will" the name against "will"), half each when it never does.
    /// In a fixed order, from the mid-sentence counts alone, so every run writes the same bytes.
    fn settle_case(&mut self) {
        let mid_sentence = self.words.clone();
        let mut open: Vec<(u64, u32)> = std::mem::take(&mut self.open_case).into_iter().collect();
        open.sort_unstable();
        for (key, n) in open {
            let (exact, lower) = unpair(key);
            let (e, l) = (mid_sentence[exact as usize], mid_sentence[lower as usize]);
            let share = (e + 0.5) / (e + l + 1.0);
            self.words[exact as usize] += f64::from(n) * share;
            self.words[lower as usize] += f64::from(n) * (1.0 - share);
        }
    }
}

/// Counts `lines` (one sentence each) on `threads` threads.
pub fn count(dictionary: &Dictionary, lines: &[&str], threads: usize) -> Counts {
    let vocabulary = Vocabulary::new(dictionary);
    let entries = dictionary.entries.len();
    let threads = threads.max(1);
    let chunk = lines.len().div_ceil(threads).max(1);
    let mut total = Counts::new(entries);
    std::thread::scope(|scope| {
        let workers: Vec<_> = lines
            .chunks(chunk)
            .map(|part| {
                let vocabulary = &vocabulary;
                scope.spawn(move || {
                    let mut counts = Counts::new(entries);
                    for line in part {
                        counts.sentence(vocabulary, line);
                    }
                    counts
                })
            })
            .collect();
        for worker in workers {
            total.merge(worker.join().expect("a counting thread panicked"));
        }
    });
    total.settle_case();
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v202::{Entry, code_points};

    fn dictionary(words: &[&str]) -> Dictionary {
        let entries = words
            .iter()
            .map(|w| Entry {
                word: code_points(w),
                probability: 100,
                not_a_word: false,
                possibly_offensive: false,
                shortcuts: Vec::new(),
                bigrams: Vec::new(),
            })
            .collect();
        Dictionary {
            flags: 0,
            attributes: Vec::new(),
            entries,
        }
    }

    #[test]
    fn counts_what_the_keyboard_would_look_up() {
        let d = dictionary(&["the", "cat", "sat", "on", "mat", "Tom", "I"]);
        let lines = [
            "The cat sat on the mat.",
            "Tom sat. The cat, the mat",
            "I sat on 2 mats and \"the\" cat.",
        ];
        let counts = count(&d, &lines, 2);
        let n = |w: &str| {
            counts.words[d
                .entries
                .iter()
                .position(|e| e.word == code_points(w))
                .unwrap()]
        };
        let pairs = |a: u32, b: u32| counts.pairs.get(&pair(a, b)).copied().unwrap_or(0);
        assert_eq!(counts.tokens, 19);
        assert_eq!((n("the"), n("cat"), n("Tom"), n("I")), (5.0, 3.0, 1.0, 1.0));
        // "The cat" twice: a capital is looked up lower-cased, as the keyboard does.
        assert_eq!(pairs(0, 1), 2);
        // After "Tom sat." a new sentence starts, and after "cat," the context is unknown.
        assert_eq!(pairs(2, 0) + pairs(1, 0), 0);
        assert_eq!(pairs(3, 0), 1);
        // "on 2 mats": the word before "mats" is "2", so "on" was a context once, not twice.
        assert_eq!(counts.contexts[3], 1);
        // "\"the\" cat" adds no pair: a word ending in a quote leaves the context unknown.
        assert_eq!(counts.contexts[0], 4);
    }

    #[test]
    fn a_capital_at_a_sentence_start_follows_the_case_used_mid_sentence() {
        let d = dictionary(&["Will", "will", "go"]);
        let lines = [
            "Will go.",
            "I will go.",
            "you will go.",
            "we will go.",
            "Will Will go.",
        ];
        let counts = count(&d, &lines, 1);
        // Mid-sentence: "will" three times, "Will" once. Twice at a start, shared 1.5 to 3.5.
        let share = 1.5 / 5.0;
        assert!((counts.words[0] - (1.0 + 2.0 * share)).abs() < 1e-9);
        assert!((counts.words[1] - (3.0 + 2.0 * (1.0 - share))).abs() < 1e-9);
    }
}
