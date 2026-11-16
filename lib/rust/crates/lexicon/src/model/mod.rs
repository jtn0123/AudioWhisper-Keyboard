//! The keyboard's n-gram language model (`crates/ngram`), built from corpus sentences over the
//! dictionary's words: a trigram model with interpolated Kneser-Ney smoothing
//! ([`smooth`]), pruned by count and quantised to a byte per probability.
//!
//! The vocabulary is the dictionary's words the corpus writes, less its possibly-offensive and
//! not-a-word entries, so the model can never predict one. A word outside the vocabulary ends a
//! context, as it does for the keyboard.

mod count;
pub mod smooth;
mod write;

use crate::count::Counts;
use crate::dictionary::{Scale, fixed};
use crate::v202::Dictionary;
use ngram::Spellings;
use std::collections::HashMap;

pub use smooth::{Discounts, Options, Tables};
pub use write::write;

/// The model's words, in UTF-16 order: their ids.
#[derive(Debug)]
pub struct Vocabulary {
    words: Vec<Vec<u16>>,
    index: HashMap<Vec<u16>, u32>,
    prefers_lower: Vec<bool>,
}

impl Vocabulary {
    /// The dictionary's entries the corpus writes, less the fixed ones. A capitalised word whose
    /// lower-case spelling the corpus writes at least as often is taken as that spelling at a
    /// sentence start.
    pub fn new(dictionary: &Dictionary, counts: &Counts) -> Vocabulary {
        let mut chosen: Vec<(Vec<u16>, f64)> = (dictionary.entries.iter())
            .zip(&counts.words)
            .filter(|(entry, n)| !fixed(entry) && **n > 0.0)
            .map(|(entry, &n)| {
                let mut word = Vec::with_capacity(entry.word.len());
                for &c in &entry.word {
                    chars::code_point::push_utf16(&mut word, c);
                }
                (word, n)
            })
            .collect();
        chosen.sort_by(|a, b| a.0.cmp(&b.0));
        chosen.dedup_by(|a, b| a.0 == b.0);
        let index: HashMap<Vec<u16>, u32> = (chosen.iter().enumerate())
            .map(|(i, (w, _))| (w.clone(), i as u32))
            .collect();
        let prefers_lower = (chosen.iter())
            .map(|(word, n)| {
                let lower = chars::lowercase(word);
                lower != *word
                    && index
                        .get(&lower)
                        .is_some_and(|&l| chosen[l as usize].1 >= *n)
            })
            .collect();
        Vocabulary {
            words: chosen.into_iter().map(|(w, _)| w).collect(),
            index,
            prefers_lower,
        }
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// The id that stands for an unknown word, or none, in the counts.
    fn unknown(&self) -> u32 {
        self.words.len() as u32 + 1
    }
}

impl Spellings for Vocabulary {
    fn exact(&self, word: &[u16]) -> Option<u32> {
        self.index.get(word).copied()
    }

    fn prefers_lower_at_start(&self, id: u32) -> bool {
        self.prefers_lower
            .get(id as usize)
            .copied()
            .unwrap_or(false)
    }

    fn sentence_start(&self) -> u32 {
        self.words.len() as u32
    }
}

/// What a build made, to report.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Summary {
    pub words: usize,
    pub counted: usize,
    pub bigrams: usize,
    pub trigrams: usize,
    pub discounts: Discounts,
}

/// The counts a model is built from, kept to build it with different options.
#[derive(Debug)]
pub struct Counted {
    vocabulary: Vocabulary,
    grams: count::Grams,
    counted: usize,
}

/// Counts `lines` (a sentence each) over `dictionary`'s words on `threads` threads; `counts` are
/// the dictionary's own counts of the same lines (`crate::count`).
pub fn count(
    dictionary: &Dictionary,
    counts: &Counts,
    lines: &[&str],
    threads: usize,
) -> Result<Counted, String> {
    let vocabulary = Vocabulary::new(dictionary, counts);
    if vocabulary.len() > count::MAX_WORDS {
        return Err(format!("{} words is too many to count", vocabulary.len()));
    }
    let keys = count::keys(&vocabulary, lines, threads);
    let grams = count::grams(&keys, vocabulary.len() as u32);
    Ok(Counted {
        counted: keys.len(),
        vocabulary,
        grams,
    })
}

/// The model file for `counted` with `options`, on the dictionary's value `scale`.
pub fn build(
    counted: &Counted,
    scale: Scale,
    options: Options,
) -> Result<(Vec<u8>, Summary), String> {
    let v = &counted.vocabulary;
    let (tables, discounts) =
        smooth::tables(&counted.grams, &v.words, &v.prefers_lower, scale, options);
    let bytes = write(&tables)?;
    let summary = Summary {
        words: v.len(),
        counted: counted.counted,
        bigrams: tables.bigrams.len(),
        trigrams: tables.trigrams.len(),
        discounts,
    };
    Ok((bytes, summary))
}

#[cfg(test)]
mod tests;
