//! The keyboard's word n-gram language model: a trigram model with interpolated Kneser-Ney
//! smoothing, pruned and quantised, built on the host by `crates/lexicon` and read here straight
//! from the file's bytes. It says how likely a word is after the one or two words before it
//! (a sentence start counts as a word), and which words are likeliest there.
//!
//! The file stores each kept n-gram's full interpolated probability and each context's backoff,
//! so a missing trigram costs the bigram's probability times the backoff, and so on down.
//!
//! Reading never panics: a damaged file gives no answers, not a crash.

#![forbid(unsafe_code)]

mod context;
pub mod format;

pub use context::{Context, Spellings, context, resolve};
use format::{Header, Layout, Unigram, ln, unpack};
use std::cmp::Ordering;
use std::fmt;

/// The likeliest words on their own, kept for predicting after a word the model doesn't know.
const TOP_WORDS: usize = 64;

/// An open model file.
pub struct Model {
    bytes: Box<[u8]>,
    header: Header,
    layout: Layout,
    /// The words with the highest lowest-order probability, best first.
    top: Vec<u32>,
}

impl fmt::Debug for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Model")
            .field("bytes", &self.bytes.len())
            .field("header", &self.header)
            .finish()
    }
}

impl Spellings for Model {
    fn exact(&self, word: &[u16]) -> Option<u32> {
        let (mut low, mut high) = (0, self.header.words);
        while low < high {
            let mid = low + (high - low) / 2;
            match compare(self.units(mid)?, word) {
                Ordering::Less => low = mid + 1,
                Ordering::Greater => high = mid,
                Ordering::Equal => return Some(mid),
            }
        }
        None
    }

    fn prefers_lower_at_start(&self, id: u32) -> bool {
        self.unigram(id)
            .is_some_and(|u| u.flags & format::PREFERS_LOWER_AT_START != 0)
    }

    fn sentence_start(&self) -> u32 {
        self.header.words
    }
}

/// A stored spelling (UTF-16 little-endian bytes) against `word`.
fn compare(stored: &[u8], word: &[u16]) -> Ordering {
    let units = stored
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c));
    units.cmp(word.iter().copied())
}

impl Model {
    /// The model in `bytes`, if they are a whole model file this reader knows. The model keeps
    /// the bytes it is given (a `Vec` read to its length becomes them without a copy).
    pub fn open(bytes: impl Into<Box<[u8]>>) -> Option<Model> {
        let bytes = bytes.into();
        let header = Header::read(&bytes)?;
        let layout = Layout::of(&header)?;
        if layout.end != bytes.len() {
            return None;
        }
        let mut model = Model {
            bytes,
            header,
            layout,
            top: Vec::new(),
        };
        let starts_end = model.u32_at(layout.bigram_starts, header.words as usize + 1)?;
        let trigrams_end = model.u32_at(layout.trigram_starts, header.contexts as usize)?;
        if starts_end != header.bigrams || trigrams_end != header.trigrams {
            return None;
        }
        model.top = model.top_words();
        Some(model)
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    /// The vocabulary's size: ids are 0 to this, less one.
    pub fn words(&self) -> u32 {
        self.header.words
    }

    /// The id of `word` as the keyboard looks a word up: as written, else lower-cased.
    pub fn word(&self, word: &[u16]) -> Option<u32> {
        resolve(self, word, false)
    }

    /// Word `id`'s spelling.
    pub fn spelling(&self, id: u32) -> Option<Vec<u16>> {
        let units = self.units(id)?;
        Some(
            units
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u16::from_le_bytes(*c))
                .collect(),
        )
    }

    /// The context after `newest_first` (see [`context`]).
    pub fn context(&self, newest_first: &[Vec<u16>], sentence_start: bool) -> Context {
        context(self, newest_first, sentence_start)
    }

    /// The natural log of the probability of word `id` after `context`.
    pub fn ln_probability(&self, context: Context, id: u32) -> Option<f32> {
        let unigram = self.unigram(id).filter(|_| id < self.header.words)?;
        let Some(newest) = context.newest else {
            return Some(self.ln_p(unigram.probability));
        };
        let mut backoff = 0.0;
        if let Some(bigram) = context.older.and_then(|older| self.bigram(older, newest)) {
            if let Some(q) = self.trigram(bigram, id) {
                return Some(self.ln_p(q));
            }
            backoff += self.ln_backoff(self.bigram_backoff(bigram)?);
        }
        if let Some(q) = self.bigram(newest, id).and_then(|at| self.bigram_q(at)) {
            return Some(backoff + self.ln_p(q));
        }
        let newest = self.unigram(newest)?;
        Some(backoff + self.ln_backoff(newest.backoff) + self.ln_p(unigram.probability))
    }

    /// The natural log of how often the corpus writes word `id`.
    pub fn ln_frequency(&self, id: u32) -> Option<f32> {
        let unigram = self.unigram(id).filter(|_| id < self.header.words)?;
        Some(self.ln_p(unigram.frequency))
    }

    /// How much likelier word `id` is after `context` than anywhere, in nats; none without a
    /// context.
    pub fn lift(&self, context: Context, id: u32) -> Option<f32> {
        context.newest?;
        Some(self.ln_probability(context, id)? - self.ln_frequency(id)?)
    }

    /// The `count` likeliest words after `context`, best first, with their log probabilities.
    /// Ties go to the lower id, so the order is the same every time.
    pub fn predict(&self, context: Context, count: usize) -> Vec<(u32, f32)> {
        let mut best = Best::new(count);
        let Some(newest) = context.newest else {
            for &id in &self.top {
                if let Some(p) = self.unigram(id) {
                    best.offer(id, self.ln_p(p.probability));
                }
            }
            return best.into_vec();
        };
        let bigram = context.older.and_then(|older| self.bigram(older, newest));
        let trigrams = bigram.map_or(&[][..], |b| self.trigram_entries(b));
        let mut backoff = 0.0;
        if let Some(b) = bigram {
            for entry in trigrams.as_chunks::<4>().0 {
                let (id, q) = unpack(u32::from_le_bytes(*entry));
                best.offer(id, self.ln_p(q));
            }
            backoff += self.bigram_backoff(b).map_or(0.0, |q| self.ln_backoff(q));
        }
        let bigrams = self.bigram_entries(newest);
        let mut walk = Walk::new(trigrams);
        for entry in bigrams.as_chunks::<4>().0 {
            let (id, q) = unpack(u32::from_le_bytes(*entry));
            if !walk.contains(id) {
                best.offer(id, backoff + self.ln_p(q));
            }
        }
        let below = backoff
            + self
                .unigram(newest)
                .map_or(0.0, |u| self.ln_backoff(u.backoff));
        for &id in &self.top {
            let stored = |entries: &[u8]| find(entries, id).is_some();
            if !stored(trigrams) && !stored(bigrams) {
                let p = self.unigram(id).map_or(0, |u| u.probability);
                best.offer(id, below + self.ln_p(p));
            }
        }
        best.into_vec()
    }

    fn ln_p(&self, q: u8) -> f32 {
        ln(q, self.header.step)
    }

    fn ln_backoff(&self, q: u8) -> f32 {
        ln(q, self.header.backoff_step)
    }

    fn u32_at(&self, section: usize, index: usize) -> Option<u32> {
        let at = section.checked_add(index.checked_mul(4)?)?;
        self.bytes.get(at..at.checked_add(4)?).map(le)
    }

    fn units(&self, id: u32) -> Option<&[u8]> {
        let start = self.u32_at(self.layout.spelling_starts, id as usize)? as usize;
        let end = self.u32_at(self.layout.spelling_starts, id as usize + 1)? as usize;
        let text = self.bytes.get(self.layout.text..self.layout.unigrams)?;
        text.get(start.checked_mul(2)?..end.checked_mul(2)?)
    }

    fn unigram(&self, id: u32) -> Option<Unigram> {
        let at = self
            .layout
            .unigrams
            .checked_add((id as usize).checked_mul(4)?)?;
        let b = self.bytes.get(at..at + 4)?;
        (id <= self.header.words).then(|| Unigram::from_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// The entries in `section` (each `size` bytes) from index `starts[at]` to `starts[at + 1]`.
    fn range(&self, starts: usize, at: usize, section: usize, size: usize) -> &[u8] {
        let bounds = (|| {
            let start = self.u32_at(starts, at)? as usize;
            let end = self.u32_at(starts, at.checked_add(1)?)? as usize;
            let from = section.checked_add(start.checked_mul(size)?)?;
            Some((from, section.checked_add(end.checked_mul(size)?)?))
        })();
        bounds
            .and_then(|(from, to)| self.bytes.get(from..to))
            .unwrap_or(&[])
    }

    fn bigram_entries(&self, context: u32) -> &[u8] {
        if context > self.header.words {
            return &[];
        }
        let l = &self.layout;
        self.range(l.bigram_starts, context as usize, l.bigrams, 4)
    }

    fn trigram_entries(&self, bigram: usize) -> &[u8] {
        let l = &self.layout;
        let contexts = self.bytes.get(l.contexts..l.trigram_starts).unwrap_or(&[]);
        let (mut low, mut high) = (0, contexts.len() / 4);
        while low < high {
            let mid = low + (high - low) / 2;
            match (le(&contexts[mid * 4..mid * 4 + 4]) as usize).cmp(&bigram) {
                Ordering::Less => low = mid + 1,
                Ordering::Greater => high = mid,
                Ordering::Equal => return self.range(l.trigram_starts, mid, l.trigrams, 4),
            }
        }
        &[]
    }

    /// The index of bigram (`previous`, `id`) among all bigrams.
    fn bigram(&self, previous: u32, id: u32) -> Option<usize> {
        let first = self.u32_at(self.layout.bigram_starts, previous as usize)? as usize;
        find(self.bigram_entries(previous), id).map(|i| first + i)
    }

    fn bigram_q(&self, bigram: usize) -> Option<u8> {
        self.u32_at(self.layout.bigrams, bigram)
            .map(|e| unpack(e).1)
    }

    fn bigram_backoff(&self, bigram: usize) -> Option<u8> {
        let at = self.layout.backoffs.checked_add(bigram)?;
        (bigram < self.header.bigrams as usize)
            .then(|| self.bytes.get(at).copied())
            .flatten()
    }

    fn trigram(&self, bigram: usize, id: u32) -> Option<u8> {
        let entries = self.trigram_entries(bigram);
        find(entries, id).map(|i| unpack(le(&entries[i * 4..i * 4 + 4])).1)
    }

    fn top_words(&self) -> Vec<u32> {
        let mut best = Best::new(TOP_WORDS);
        for id in 0..self.header.words {
            if let Some(u) = self.unigram(id) {
                best.offer(id, self.ln_p(u.probability));
            }
        }
        best.into_vec().into_iter().map(|(id, _)| id).collect()
    }
}

fn le(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

/// The index of `id` among packed `entries` sorted by id.
fn find(entries: &[u8], id: u32) -> Option<usize> {
    let count = entries.len() / 4;
    let (mut low, mut high) = (0, count);
    while low < high {
        let mid = low + (high - low) / 2;
        let (at, _) = unpack(le(&entries[mid * 4..mid * 4 + 4]));
        match at.cmp(&id) {
            Ordering::Less => low = mid + 1,
            Ordering::Greater => high = mid,
            Ordering::Equal => return Some(mid),
        }
    }
    None
}

/// Walks packed entries sorted by id alongside increasing ids.
struct Walk<'a> {
    entries: &'a [u8],
    at: usize,
}

impl<'a> Walk<'a> {
    fn new(entries: &'a [u8]) -> Self {
        Walk { entries, at: 0 }
    }

    /// Whether `id` is among the entries; ids must be asked in increasing order.
    fn contains(&mut self, id: u32) -> bool {
        while let Some(entry) = self.entries.get(self.at * 4..self.at * 4 + 4) {
            let (at, _) = unpack(le(entry));
            match at.cmp(&id) {
                Ordering::Less => self.at += 1,
                Ordering::Equal => return true,
                Ordering::Greater => return false,
            }
        }
        false
    }
}

/// The best few (id, score) pairs offered, highest score first, ties to the lower id.
struct Best {
    count: usize,
    items: Vec<(u32, f32)>,
}

impl Best {
    fn new(count: usize) -> Self {
        Best {
            count,
            items: Vec::with_capacity(count + 1),
        }
    }

    fn offer(&mut self, id: u32, score: f32) {
        let better = |a: &(u32, f32)| a.1 > score || (a.1 == score && a.0 < id);
        let at = self.items.partition_point(better);
        if at < self.count {
            self.items.insert(at, (id, score));
            self.items.truncate(self.count);
        }
    }

    fn into_vec(self) -> Vec<(u32, f32)> {
        self.items
    }
}

#[cfg(test)]
mod tests;
