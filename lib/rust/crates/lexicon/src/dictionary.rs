//! A new dictionary from the old one and corpus counts: the same words, flags and shortcuts, each
//! word's probability from how often the corpus writes it, and many more next words.
//!
//! Probabilities keep the old dictionary's scale. The decoder's weights and thresholds were tuned
//! on it, so the new dictionary hands out exactly the old set of values, only to different words:
//! the word the corpus writes most gets the old highest value, and so on down (a quantile
//! mapping). Words the corpus never writes keep their old order, below the rest.
//!
//! A next word's 4-bit value is its probability after the word, on the same scale, encoded as
//! AOSP's `BinaryDictEncoderUtils.getBigramFrequencyDiff` does.
//!
//! Possibly-offensive and not-a-word entries keep their old value and are never stored as a next
//! word, so the rebuild can't start suggesting them.

use crate::count::{Counts, unpair};
use crate::v202::{Dictionary, Entry};

/// Whether an entry keeps its old value and is never a next word.
pub(crate) fn fixed(entry: &Entry) -> bool {
    entry.possibly_offensive || entry.not_a_word
}

/// How a dictionary is built.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// Most next words stored for a word.
    pub next: usize,
    /// Fewest times a pair must be seen to be stored.
    pub min_pair: u32,
    /// The header's `date`, in seconds since 1970.
    pub date: u64,
}

/// The value scale, fitted from the new unigram values: value = `a + b ln p`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale {
    pub a: f64,
    pub b: f64,
}

impl Scale {
    pub fn value(&self, probability: f64) -> f64 {
        self.a + self.b * probability.ln()
    }
}

/// What the build did, to report.
#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub scale: Option<Scale>,
    pub bigrams: usize,
    pub words_with_bigrams: usize,
    pub words_seen: usize,
}

const MAX_VALUE: f64 = 255.0;
/// AOSP's MAX_BIGRAM_FREQUENCY.
const MAX_STEP: u8 = 15;
/// Words seen this often say where a probability sits on the scale.
const RELIABLE: f64 = 20.0;

/// The distance between a word's 4-bit steps: what is left above its unigram value, in 16.5 parts.
fn step_size(unigram: u8) -> f64 {
    (MAX_VALUE - f64::from(unigram)) / (1.5 + f64::from(MAX_STEP))
}

/// `BinaryDictEncoderUtils.getBigramFrequencyDiff`: the 4-bit step for an absolute bigram value
/// after a word of unigram value `unigram`.
pub fn step(unigram: u8, bigram: f64) -> u8 {
    let size = step_size(unigram);
    let first = 1.0 + f64::from(unigram) + size / 2.0;
    let step = ((bigram - first) / size).floor();
    step.clamp(0.0, f64::from(MAX_STEP)) as u8
}

/// The value a word scores after its previous word for a step: the keyboard's
/// `compute_probability_for_bigram`, without its rounding down.
pub fn combined(unigram: u8, step: u8) -> f64 {
    f64::from(unigram) + (f64::from(step) + 1.0) * step_size(unigram)
}

pub fn rebuild(old: &Dictionary, counts: &Counts, options: Options) -> (Dictionary, Summary) {
    let mut new = old.clone();
    rank(&mut new, counts);
    let scale = scale(&new, counts);
    for entry in &mut new.entries {
        entry.bigrams.clear();
    }
    if let Some(scale) = scale {
        next_words(&mut new, counts, options, scale);
    }
    for (key, value) in &mut new.attributes {
        match key.as_str() {
            "version" => *value = (value.parse::<u32>().unwrap_or(0) + 1).to_string(),
            "date" => *value = options.date.to_string(),
            _ => {}
        }
    }
    let summary = Summary {
        scale,
        bigrams: new.entries.iter().map(|e| e.bigrams.len()).sum(),
        words_with_bigrams: new.entries.iter().filter(|e| !e.bigrams.is_empty()).count(),
        words_seen: counts.words.iter().filter(|&&n| n > 0.0).count(),
    };
    (new, summary)
}

/// New unigram values: the old values, handed out again by corpus frequency.
fn rank(new: &mut Dictionary, counts: &Counts) {
    let mut order: Vec<usize> = (0..new.entries.len())
        .filter(|&i| !fixed(&new.entries[i]))
        .collect();
    let mut values: Vec<u8> = order.iter().map(|&i| new.entries[i].probability).collect();
    values.sort_unstable_by(|a, b| b.cmp(a));
    let old = |i: usize| new.entries[i].probability;
    order.sort_by(|&x, &y| {
        counts.words[y]
            .total_cmp(&counts.words[x])
            .then(old(y).cmp(&old(x)))
            .then(x.cmp(&y))
    });
    for (&i, &value) in order.iter().zip(&values) {
        new.entries[i].probability = value;
    }
}

/// The value scale of a rebuilt dictionary `new`: least squares of its values on the log
/// probability, over the words seen often enough.
pub fn scale(new: &Dictionary, counts: &Counts) -> Option<Scale> {
    let total: f64 = counts.words.iter().sum();
    let points: Vec<(f64, f64)> = new
        .entries
        .iter()
        .zip(&counts.words)
        .filter(|(e, n)| !fixed(e) && **n >= RELIABLE)
        .map(|(e, &n)| ((n / total).ln(), f64::from(e.probability)))
        .collect();
    let k = points.len() as f64;
    let (mx, my) = points
        .iter()
        .fold((0.0, 0.0), |(x, y), p| (x + p.0, y + p.1));
    let (mx, my) = (mx / k, my / k);
    let sxy: f64 = points.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    let sxx: f64 = points.iter().map(|p| (p.0 - mx).powi(2)).sum();
    let b = sxy / sxx;
    (k >= 2.0 && b.is_finite() && b > 0.0).then_some(Scale { a: my - b * mx, b })
}

/// Each word's most frequent followers, as many as `options.next`, that it makes likelier than
/// they are anyway: a pair the encoding can't put above the word's own value would only overstate
/// it, so it is left to the unigram.
fn next_words(new: &mut Dictionary, counts: &Counts, options: Options, scale: Scale) {
    let mut followers: Vec<Vec<(u32, u32)>> = vec![Vec::new(); new.entries.len()];
    for (&key, &n) in &counts.pairs {
        let (previous, next) = unpair(key);
        if n >= options.min_pair && !fixed(&new.entries[next as usize]) {
            followers[previous as usize].push((next, n));
        }
    }
    for (previous, mut list) in followers.into_iter().enumerate() {
        list.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        let context = counts.contexts[previous] as f64;
        let mut bigrams = Vec::new();
        for (next, n) in list {
            if bigrams.len() == options.next {
                break;
            }
            let unigram = new.entries[next as usize].probability;
            let value = scale.value(f64::from(n) / context).min(MAX_VALUE);
            if value < f64::from(unigram) + step_size(unigram) / 2.0 {
                continue;
            }
            bigrams.push((next as usize, step(unigram, value)));
        }
        new.entries[previous].bigrams = bigrams;
    }
}

#[cfg(test)]
mod tests;
