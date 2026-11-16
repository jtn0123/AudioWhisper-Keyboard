//! Interpolated Kneser-Ney smoothing of the counted n-grams, then pruning and quantising them
//! into the tables the model file holds.
//!
//! - A word alone: its continuation count (how many different words came before it) over all
//!   of them, so a word that only ever follows one word ("Francisco") is not thought common.
//! - After a word: the pair's continuation count, or its plain count after a sentence start
//!   (nothing comes before one), less a discount, plus the discounted mass times the word alone.
//! - After two words: the triple's count less a discount, plus that mass times the pair.
//!
//! The discount at each order is `n1 / (n1 + 2 n2)` from its counts of counts. Each kept n-gram
//! stores its full interpolated probability and each context its backoff (the mass it hands
//! down), so the reader multiplies its way down for an n-gram that isn't kept.

use super::count::{Bigram, Grams};
use crate::dictionary::Scale;
use ngram::format::{Unigram, pack, quantise};

/// Which n-grams the file keeps.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// Fewest times a pair must be seen to be kept.
    pub min_bigram: u32,
    /// Fewest times a triple must be seen to be kept.
    pub min_trigram: u32,
}

/// The model file's contents, before they are written out.
#[derive(Clone, Debug, PartialEq)]
pub struct Tables {
    pub words: Vec<Vec<u16>>,
    /// Each word's, then the sentence start's.
    pub unigrams: Vec<Unigram>,
    pub bigram_starts: Vec<u32>,
    pub bigrams: Vec<u32>,
    pub backoffs: Vec<u8>,
    /// The bigrams with trigrams, by index.
    pub contexts: Vec<u32>,
    pub trigram_starts: Vec<u32>,
    pub trigrams: Vec<u32>,
    pub step: f32,
    pub backoff_step: f32,
    pub scale: Scale,
}

/// What smoothing found, to report.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Discounts {
    pub bigram: f64,
    pub trigram: f64,
}

fn discount(counts: impl Iterator<Item = u32>) -> f64 {
    let (mut n1, mut n2) = (0.0f64, 0.0f64);
    for c in counts {
        match c {
            1 => n1 += 1.0,
            2 => n2 += 1.0,
            _ => {}
        }
    }
    if n1 + 2.0 * n2 > 0.0 {
        (n1 / (n1 + 2.0 * n2)).clamp(0.05, 0.95)
    } else {
        0.5
    }
}

/// The count a pair is smoothed with: its plain count after a sentence start, else its
/// continuation count.
fn smoothing_count(bigram: &Bigram, start: u32) -> u32 {
    if bigram.previous == start {
        bigram.count
    } else {
        bigram.continuation
    }
}

/// Probabilities in nats, unquantised.
struct Smoothed {
    unigram: Vec<f64>,
    /// After each word, then the sentence start: the backoff to the word alone.
    unigram_backoff: Vec<f64>,
    bigram: Vec<f64>,
    trigram: Vec<f64>,
    /// For each trigram context, where its trigrams start and its backoff.
    contexts: Vec<(usize, f64)>,
    discounts: Discounts,
}

fn smooth(grams: &Grams, words: u32) -> Smoothed {
    let start = words;
    let floor = |n: f64| n.max(0.5);
    let total: f64 = grams
        .continuation
        .iter()
        .map(|&n| floor(f64::from(n)))
        .sum();
    let unigram: Vec<f64> = (grams.continuation.iter())
        .map(|&n| floor(f64::from(n)) / total)
        .collect();
    let d2 = discount(grams.bigrams.iter().map(|b| smoothing_count(b, start)));
    let d3 = discount(grams.trigrams.iter().map(|t| t.count));
    let mut unigram_backoff = vec![1.0; words as usize + 1];
    let mut bigram = vec![0.0; grams.bigrams.len()];
    let mut at = 0;
    for group in grams.bigrams.chunk_by(|a, b| a.previous == b.previous) {
        let sum: f64 = group
            .iter()
            .map(|b| f64::from(smoothing_count(b, start)))
            .sum();
        let backoff = d2 * group.len() as f64 / sum;
        unigram_backoff[group[0].previous as usize] = backoff;
        for b in group {
            let k = f64::from(smoothing_count(b, start));
            bigram[at] = (k - d2).max(0.0) / sum + backoff * unigram[b.word as usize];
            at += 1;
        }
    }
    let find = |previous: u32, word: u32| {
        grams
            .bigrams
            .binary_search_by_key(&(previous, word), |b| (b.previous, b.word))
            .ok()
    };
    let mut trigram = vec![0.0; grams.trigrams.len()];
    let mut contexts = Vec::new();
    let mut at = 0;
    for group in grams
        .trigrams
        .chunk_by(|a, b| (a.older, a.previous) == (b.older, b.previous))
    {
        let sum: f64 = group.iter().map(|t| f64::from(t.count)).sum();
        let backoff = d3 * group.len() as f64 / sum;
        contexts.push((at, backoff));
        for t in group {
            let lower = find(t.previous, t.word).map_or(0.0, |i| bigram[i]);
            trigram[at] = (f64::from(t.count) - d3).max(0.0) / sum + backoff * lower;
            at += 1;
        }
    }
    Smoothed {
        unigram,
        unigram_backoff,
        bigram,
        trigram,
        contexts,
        discounts: Discounts {
            bigram: d2,
            trigram: d3,
        },
    }
}

/// The step that fits the largest `-ln` in a byte.
fn step(lns: impl Iterator<Item = f64>) -> f64 {
    let most = lns.map(|ln| -ln).fold(0.0, f64::max);
    (most / 255.0).max(1e-3)
}

/// Smooths, prunes and quantises `grams` over `words` (their spellings, in id order) with
/// `prefers_lower` flags.
pub fn tables(
    grams: &Grams,
    words: &[Vec<u16>],
    prefers_lower: &[bool],
    scale: Scale,
    options: Options,
) -> (Tables, Discounts) {
    let n = words.len() as u32;
    let s = smooth(grams, n);
    let kept: Vec<bool> = (grams.bigrams.iter())
        .map(|b| b.count >= options.min_bigram)
        .collect();
    let mut stored_index = vec![None; grams.bigrams.len()];
    let mut stored = 0usize;
    for (i, &keep) in kept.iter().enumerate() {
        if keep {
            stored_index[i] = Some(stored);
            stored += 1;
        }
    }
    let frequency_total: f64 = grams.frequency.iter().map(|&f| f as f64).sum();
    let frequency: Vec<f64> = (grams.frequency.iter())
        .map(|&f| (f as f64).max(0.5) / frequency_total.max(1.0))
        .collect();
    let mut backoff_ln = vec![0.0; stored];
    let mut trigram_lists: Vec<Vec<(u32, f64)>> = vec![Vec::new(); stored];
    let find = |older: u32, previous: u32| {
        grams
            .bigrams
            .binary_search_by_key(&(older, previous), |b| (b.previous, b.word))
            .ok()
            .and_then(|i| stored_index[i])
    };
    for (c, &(first, backoff)) in s.contexts.iter().enumerate() {
        let end = s.contexts.get(c + 1).map_or(s.trigram.len(), |next| next.0);
        let head = grams.trigrams[first];
        let Some(context) = find(head.older, head.previous) else {
            continue;
        };
        backoff_ln[context] = backoff.ln();
        for i in first..end {
            let t = grams.trigrams[i];
            if t.count >= options.min_trigram {
                trigram_lists[context].push((t.word, s.trigram[i].ln()));
            }
        }
    }
    let kept_bigrams = (grams.bigrams.iter().zip(&s.bigram))
        .zip(&kept)
        .filter(|(_, keep)| **keep)
        .map(|((b, &p), _)| (b.previous, b.word, p.ln()));
    let kept_bigrams: Vec<(u32, u32, f64)> = kept_bigrams.collect();
    let lns = (s.unigram.iter().chain(&frequency).map(|p| p.ln()))
        .chain(kept_bigrams.iter().map(|b| b.2))
        .chain(trigram_lists.iter().flatten().map(|t| t.1));
    let p_step = step(lns);
    let backoff_lns = s.unigram_backoff.iter().map(|b| b.ln());
    let b_step = step(backoff_lns.chain(backoff_ln.iter().copied()));
    let q = |ln: f64| quantise(ln, p_step);
    let qb = |ln: f64| quantise(ln, b_step);
    let mut unigrams: Vec<Unigram> = (0..n as usize)
        .map(|w| Unigram {
            probability: q(s.unigram[w].ln()),
            backoff: qb(s.unigram_backoff[w].ln()),
            frequency: q(frequency[w].ln()),
            flags: if prefers_lower[w] {
                ngram::format::PREFERS_LOWER_AT_START
            } else {
                0
            },
        })
        .collect();
    unigrams.push(Unigram {
        probability: u8::MAX,
        backoff: qb(s.unigram_backoff[n as usize].ln()),
        frequency: u8::MAX,
        flags: 0,
    });
    let mut bigram_starts = vec![0u32; n as usize + 2];
    for b in &kept_bigrams {
        bigram_starts[b.0 as usize + 1] += 1;
    }
    for i in 1..bigram_starts.len() {
        bigram_starts[i] += bigram_starts[i - 1];
    }
    let bigrams = kept_bigrams.iter().map(|b| pack(b.1, q(b.2))).collect();
    let backoffs = backoff_ln.iter().map(|&ln| qb(ln)).collect();
    let mut contexts = Vec::new();
    let mut trigram_starts = vec![0];
    let mut trigrams = Vec::new();
    for (bigram, list) in trigram_lists.iter().enumerate() {
        if !list.is_empty() {
            contexts.push(bigram as u32);
            trigrams.extend(list.iter().map(|&(w, ln)| pack(w, q(ln))));
            trigram_starts.push(trigrams.len() as u32);
        }
    }
    let tables = Tables {
        words: words.to_vec(),
        unigrams,
        bigram_starts,
        bigrams,
        backoffs,
        contexts,
        trigram_starts,
        trigrams,
        step: p_step as f32,
        backoff_step: b_step as f32,
        scale,
    };
    (tables, s.discounts)
}
