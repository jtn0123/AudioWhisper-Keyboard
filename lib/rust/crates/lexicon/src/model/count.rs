//! Counting a corpus's word triples for the n-gram model: every letter word the model knows, with
//! the context the keyboard would have before it (`PreviousWords::from`, resolved by
//! `ngram::context`), so the model is counted exactly as it will be asked.

use super::Vocabulary;
use crate::text::Sentence;
use suggest::previous_words::PreviousWords;

const BITS: u32 = 21;
const MASK: u64 = (1 << BITS) - 1;

/// One word in context, packed: the previous word, the word, then the word before the previous.
/// The sentence start is `words`; an unknown or missing word is `words + 1`.
fn key(previous: u32, word: u32, older: u32) -> u64 {
    u64::from(previous) << (2 * BITS) | u64::from(word) << BITS | u64::from(older)
}

fn unkey(key: u64) -> (u32, u32, u32) {
    (
        (key >> (2 * BITS)) as u32,
        (key >> BITS & MASK) as u32,
        (key & MASK) as u32,
    )
}

/// The largest vocabulary [`key`] can hold.
pub const MAX_WORDS: usize = (1 << BITS) - 2;

/// Every word of `line` the vocabulary knows, in context.
fn sentence(vocabulary: &Vocabulary, line: &str, out: &mut Vec<u64>) {
    let unknown = vocabulary.unknown();
    let sentence = Sentence::new(line);
    for i in 0..sentence.words.len() {
        let previous = PreviousWords::from(sentence.before(i));
        let initial = previous.words.is_empty() && previous.is_beginning_of_sentence;
        let Some(word) = ngram::resolve(vocabulary, sentence.word(i), initial) else {
            continue;
        };
        let context = ngram::context(
            vocabulary,
            &previous.words,
            previous.is_beginning_of_sentence,
        );
        let newest = context.newest.unwrap_or(unknown);
        let older = context.older.unwrap_or(unknown);
        out.push(key(newest, word, older));
    }
}

/// The sorted keys of every word in `lines`, counted on `threads` threads.
pub fn keys(vocabulary: &Vocabulary, lines: &[&str], threads: usize) -> Vec<u64> {
    let chunk = lines.len().div_ceil(threads.max(1)).max(1);
    let mut all = std::thread::scope(|scope| {
        let workers: Vec<_> = lines
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || {
                    let mut keys = Vec::new();
                    for line in part {
                        sentence(vocabulary, line, &mut keys);
                    }
                    keys.sort_unstable();
                    keys
                })
            })
            .collect();
        let parts: Vec<Vec<u64>> = workers
            .into_iter()
            .map(|w| w.join().expect("a counting thread panicked"))
            .collect();
        parts
    });
    let mut keys = Vec::with_capacity(all.iter().map(Vec::len).sum());
    for part in &mut all {
        keys.append(part);
    }
    keys.sort_unstable();
    keys
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bigram {
    pub previous: u32,
    pub word: u32,
    /// Times the word followed `previous`.
    pub count: u32,
    /// How many different words (or none) came before the pair: Kneser-Ney's continuation count.
    pub continuation: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trigram {
    pub older: u32,
    pub previous: u32,
    pub word: u32,
    pub count: u32,
}

/// The counts the model is smoothed from.
#[derive(Debug, Default)]
pub struct Grams {
    /// By previous word (the sentence start last), then word.
    pub bigrams: Vec<Bigram>,
    /// By older word, previous word, then word; both context words known.
    pub trigrams: Vec<Trigram>,
    /// For each word, how many different words (or none) came right before it.
    pub continuation: Vec<u32>,
    /// For each word, how many times the corpus writes it.
    pub frequency: Vec<u64>,
}

/// The counts in sorted `keys` over a vocabulary of `words`.
pub fn grams(keys: &[u64], words: u32) -> Grams {
    let (start, unknown) = (words, words + 1);
    let mut grams = Grams {
        continuation: vec![0; words as usize],
        frequency: vec![0; words as usize],
        ..Grams::default()
    };
    let mut last_pair = None;
    for run in keys.chunk_by(|a, b| a == b) {
        let n = run.len() as u32;
        let (previous, word, older) = unkey(run[0]);
        grams.frequency[word as usize] += u64::from(n);
        if last_pair != Some((previous, word)) {
            last_pair = Some((previous, word));
            grams.continuation[word as usize] += 1;
            if previous != unknown {
                grams.bigrams.push(Bigram {
                    previous,
                    word,
                    count: 0,
                    continuation: 0,
                });
            }
        }
        if previous == unknown {
            continue;
        }
        if let Some(bigram) = grams.bigrams.last_mut() {
            bigram.count += n;
            bigram.continuation += 1;
        }
        if previous != start && older != unknown {
            grams.trigrams.push(Trigram {
                older,
                previous,
                word,
                count: n,
            });
        }
    }
    grams
        .trigrams
        .sort_unstable_by_key(|t| (t.older, t.previous, t.word));
    grams
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_round_trip() {
        assert_eq!(
            unkey(key(5, 1_000_000, 2_097_150)),
            (5, 1_000_000, 2_097_150)
        );
    }

    #[test]
    fn counts_pairs_triples_and_continuations() {
        // Vocabulary of 3; the sentence start is 3 and unknown 4.
        let keys = {
            let mut k = vec![
                key(0, 1, 3),
                key(0, 1, 3),
                key(0, 1, 2),
                key(0, 1, 4),
                key(3, 0, 4),
                key(4, 1, 4),
                key(2, 0, 1),
            ];
            k.sort_unstable();
            k
        };
        let g = grams(&keys, 3);
        let b = |previous, word, count, continuation| Bigram {
            previous,
            word,
            count,
            continuation,
        };
        assert_eq!(g.bigrams, vec![b(0, 1, 4, 3), b(2, 0, 1, 1), b(3, 0, 1, 1)]);
        let t = |older, previous, word, count| Trigram {
            older,
            previous,
            word,
            count,
        };
        assert_eq!(
            g.trigrams,
            vec![t(1, 2, 0, 1), t(2, 0, 1, 1), t(3, 0, 1, 2)]
        );
        // Word 1 follows 0 and an unknown word; word 0 follows 2 and the sentence start.
        assert_eq!(g.continuation, vec![2, 2, 0]);
        assert_eq!(g.frequency, vec![2, 5, 0]);
    }
}
