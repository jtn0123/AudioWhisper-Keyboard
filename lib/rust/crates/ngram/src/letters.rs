//! What comes next in a word being typed: the model's probability, after a context, of the words
//! that start with what was typed so far, split by the unit that follows. The keyboard's hit test
//! leans on it near a key's border (`keys::resize`).
//!
//! Words are stored in UTF-16 order, so the words that start with a prefix are one run of ids,
//! and so are the words that continue it with each next unit. A run's probability is exact for
//! the stored model: the explicit trigrams and bigrams in the run, plus the backoffs times the
//! rest of the run's lower-order probability, which a running sum of the unigrams gives at once.

use super::{Model, le, unpack};
use crate::Context;
use std::cmp::Ordering;
use std::ops::Range;

/// What the next-unit sums need, made once per model when first asked.
pub(crate) struct Sums {
    /// `below[id]` is the sum of the unigram probabilities of the words before `id`.
    below: Box<[f64]>,
    /// A quantised probability and backoff byte as a probability.
    probability: [f64; 256],
    backoff: [f64; 256],
}

impl Sums {
    pub(crate) fn of(model: &Model) -> Sums {
        let probability = std::array::from_fn(|q| f64::from(model.ln_p(q as u8)).exp());
        let backoff = std::array::from_fn(|q| f64::from(model.ln_backoff(q as u8)).exp());
        let words = model.header.words as usize;
        let mut below = Vec::with_capacity(words + 1);
        let mut sum = 0.0;
        below.push(sum);
        for id in 0..model.header.words {
            sum += model
                .unigram(id)
                .map_or(0.0, |u| probability[u.probability as usize]);
            below.push(sum);
        }
        Sums {
            below: below.into(),
            probability,
            backoff,
        }
    }
}

/// The stored n-grams after one context, and the backoffs past them.
struct Levels<'a> {
    trigrams: &'a [u8],
    /// The backoff after the two context words, when the model has them as a pair.
    trigram_backoff: f64,
    bigrams: &'a [u8],
    bigram_backoff: f64,
    /// No context: the unigrams alone.
    unigram_only: bool,
}

impl Model {
    /// For each unit that comes next in the model's words starting with `prefix` (exactly as
    /// written), that unit and the probability, after `context`, of the words it continues
    /// with, in UTF-16 order. A word that is the prefix itself continues with nothing and is left
    /// out. Empty when no word starts with `prefix`.
    pub fn next_units(&self, context: Context, prefix: &[u16]) -> Vec<(u16, f64)> {
        let range = self.starting_with(prefix, 0..self.header.words);
        let n = prefix.len();
        let mut at = range.start;
        if self.units_of(at).is_some_and(|word| word.len() == n) {
            at += 1;
        }
        let sums = self.sums.get_or_init(|| Sums::of(self));
        let levels = self.levels(context);
        let mut next = Vec::new();
        while at < range.end {
            let Some(unit) = self.units_of(at).and_then(|word| word.get(n).copied()) else {
                // A damaged spelling: no answer rather than a wrong one.
                return Vec::new();
            };
            let mut longer = prefix.to_vec();
            longer.push(unit);
            let run = self.starting_with(&longer, at..range.end);
            if run.end <= at {
                return Vec::new();
            }
            next.push((unit, self.mass(&levels, sums, at..run.end)));
            at = run.end;
        }
        next
    }

    /// Word `id`'s spelling as UTF-16 units.
    fn units_of(&self, id: u32) -> Option<Vec<u16>> {
        (id < self.header.words)
            .then(|| self.spelling(id))
            .flatten()
    }

    /// The ids in `within` of the words that start with `prefix`.
    fn starting_with(&self, prefix: &[u16], within: Range<u32>) -> Range<u32> {
        let order = |id: u32| {
            let stored = self.units(id).unwrap_or(&[]);
            let units = stored
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u16::from_le_bytes(*c));
            units.take(prefix.len()).cmp(prefix.iter().copied())
        };
        let first = |stop: &dyn Fn(Ordering) -> bool| {
            let (mut low, mut high) = (within.start, within.end);
            while low < high {
                let mid = low + (high - low) / 2;
                if stop(order(mid)) {
                    high = mid;
                } else {
                    low = mid + 1;
                }
            }
            low
        };
        let start = first(&|o| o != Ordering::Less);
        let end = first(&|o| o == Ordering::Greater);
        start..end.max(start)
    }

    fn levels(&self, context: Context) -> Levels<'_> {
        let sums = self.sums.get_or_init(|| Sums::of(self));
        let Some(newest) = context.newest else {
            return Levels {
                trigrams: &[],
                trigram_backoff: 1.0,
                bigrams: &[],
                bigram_backoff: 1.0,
                unigram_only: true,
            };
        };
        let pair = context.older.and_then(|older| self.bigram(older, newest));
        let (trigrams, trigram_backoff) = match pair {
            Some(b) => (
                self.trigram_entries(b),
                self.bigram_backoff(b)
                    .map_or(1.0, |q| sums.backoff[q as usize]),
            ),
            None => (&[][..], 1.0),
        };
        let bigram_backoff = self
            .unigram(newest)
            .map_or(1.0, |u| sums.backoff[u.backoff as usize]);
        Levels {
            trigrams,
            trigram_backoff,
            bigrams: self.bigram_entries(newest),
            bigram_backoff,
            unigram_only: false,
        }
    }

    /// The probability after the levels' context of the words with ids in `ids`.
    fn mass(&self, levels: &Levels, sums: &Sums, ids: Range<u32>) -> f64 {
        let unigram = |id: u32| {
            self.unigram(id)
                .map_or(0.0, |u| sums.probability[u.probability as usize])
        };
        let all = sums.below.get(ids.end as usize).copied().unwrap_or(0.0)
            - sums.below.get(ids.start as usize).copied().unwrap_or(0.0);
        if levels.unigram_only {
            return all.max(0.0);
        }
        let trigrams = within(levels.trigrams, &ids);
        let bigrams = within(levels.bigrams, &ids);
        let (mut explicit3, mut explicit2, mut stored_below) = (0.0, 0.0, 0.0);
        for (id, q) in trigrams.clone() {
            explicit3 += sums.probability[q as usize];
            stored_below += unigram(id);
        }
        let mut walk = trigrams.peekable();
        for (id, q) in bigrams {
            while walk.next_if(|&(t, _)| t < id).is_some() {}
            if walk.peek().is_some_and(|&(t, _)| t == id) {
                continue;
            }
            explicit2 += sums.probability[q as usize];
            stored_below += unigram(id);
        }
        let rest = (all - stored_below).max(0.0);
        explicit3 + levels.trigram_backoff * (explicit2 + levels.bigram_backoff * rest)
    }
}

/// The packed entries (sorted by id) whose ids are in `ids`, as (id, quantised probability).
fn within<'a>(entries: &'a [u8], ids: &Range<u32>) -> impl Iterator<Item = (u32, u8)> + Clone + 'a {
    let chunks = entries.as_chunks::<4>().0;
    let id = |c: &[u8; 4]| unpack(le(c)).0;
    let start = chunks.partition_point(|c| id(c) < ids.start);
    let end = chunks.partition_point(|c| id(c) < ids.end).max(start);
    chunks[start..end].iter().map(|c| unpack(le(c)))
}
