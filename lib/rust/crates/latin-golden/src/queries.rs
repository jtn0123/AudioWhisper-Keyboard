//! Seeded decoder questions shaped like the keyboard's: real words typed cleanly, with typos or
//! half-finished, gibberish, odd case and characters, empty input (next-word prediction), taps off
//! the key centre, and up to three previous words with the sentence-start slot the app adds.

use crate::qwerty::{self, TapOffset};

/// The native MAX_WORD_LENGTH: the longest word, and the input arrays' size.
pub const MAX_WORD_LENGTH: usize = 48;
/// The native MAX_PREV_WORD_COUNT_FOR_N_GRAM: the previous-word slots.
pub const MAX_PREVIOUS_WORDS: usize = 3;

/// A decoder question, as `LatinDictionaryEngine` shapes one.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Query {
    /// The typed code points, at most [`MAX_WORD_LENGTH`].
    pub code_points: Vec<i32>,
    /// Where each was touched on the QWERTY model, or `None` for no coordinate.
    pub points: Vec<Option<(i32, i32)>>,
    pub block_offensive_words: bool,
    /// The previous words, most recent first, at most [`MAX_PREVIOUS_WORDS`].
    pub previous: Vec<Vec<i32>>,
    /// One flag per previous word: the slot is the start of a sentence.
    pub beginning_of_sentence: Vec<bool>,
}

/// SplitMix64: small, seeded, and the same on every computer.
#[derive(Debug)]
pub struct Random(u64);

impl Random {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `0..bound`; `bound` isn't 0.
    pub fn below(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }

    pub fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

const LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";
const ODD: [char; 10] = ['\'', '-', '.', 'é', 'ß', 'İ', '1', ' ', 'ñ', '😀'];

/// The neighbours of a letter on the QWERTY rows, for slips of the finger.
fn neighbour(random: &mut Random, letter: char) -> char {
    const ROWS: [&str; 3] = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];
    for row in ROWS {
        let keys: Vec<char> = row.chars().collect();
        if let Some(at) = keys.iter().position(|&it| it == letter) {
            let mut near = Vec::new();
            if at > 0 {
                near.push(keys[at - 1]);
            }
            if at + 1 < keys.len() {
                near.push(keys[at + 1]);
            }
            return *random.pick(&near);
        }
    }
    random_letter(random)
}

fn random_letter(random: &mut Random) -> char {
    let letters: Vec<char> = LETTERS.chars().collect();
    *random.pick(&letters)
}

/// One typo: a neighbouring key, a letter dropped, doubled or added, or two swapped.
fn typo(random: &mut Random, word: &mut Vec<char>) {
    if word.is_empty() {
        word.push(random_letter(random));
        return;
    }
    let at = random.below(word.len());
    match random.below(5) {
        0 => word[at] = neighbour(random, word[at]),
        1 if word.len() > 1 => {
            word.remove(at);
        }
        2 => word.insert(at, word[at]),
        3 if at + 1 < word.len() => word.swap(at, at + 1),
        _ => word.insert(at, random_letter(random)),
    }
}

/// What the user typed, from the dictionary's `words`.
fn typed(random: &mut Random, words: &[Vec<char>]) -> Vec<char> {
    let mut word = random.pick(words).clone();
    match random.below(100) {
        0..25 => {}
        25..55 => typo(random, &mut word),
        55..65 => {
            typo(random, &mut word);
            typo(random, &mut word);
        }
        65..80 => word.truncate(1 + random.below(word.len().max(1))),
        80..88 => {
            word = (0..1 + random.below(12))
                .map(|_| random_letter(random))
                .collect()
        }
        88..93 => {
            let at = random.below(word.len() + 1);
            word.insert(at, *random.pick(&ODD));
        }
        93..96 => word.clear(),
        _ => {
            word = (0..random.below(60))
                .map(|_| random_letter(random))
                .collect()
        }
    }
    if random.chance(8) {
        if let Some(first) = word.first_mut() {
            *first = first.to_uppercase().next().unwrap_or(*first);
        }
    } else if random.chance(3) {
        word = word.iter().flat_map(|it| it.to_uppercase()).collect();
    }
    word
}

/// Where each code point was touched: the app's QWERTY model with the tap's offset, if any.
fn points(random: &mut Random, code_points: &[i32]) -> Vec<Option<(i32, i32)>> {
    let offset = |random: &mut Random| -> f32 { (random.below(1001) as f32 - 500.0) / 1000.0 };
    code_points
        .iter()
        .map(|&it| {
            let tap = (!random.chance(15)).then(|| TapOffset {
                dx: offset(random),
                dy: offset(random),
            });
            qwerty::touch_point(it, tap).ok().flatten()
        })
        .collect()
}

/// A question for the decoder, drawing words from `words`.
pub fn query(random: &mut Random, words: &[Vec<char>]) -> Query {
    let mut code_points: Vec<i32> = typed(random, words).iter().map(|&it| it as i32).collect();
    code_points.truncate(MAX_WORD_LENGTH);
    let points = points(random, &code_points);
    // As LatinDictionaryEngine.previousWordSlots: the words, then empty start-of-sentence slots.
    let slots = random.below(MAX_PREVIOUS_WORDS + 1);
    let known = random.below(slots + 1);
    let previous: Vec<Vec<i32>> = (0..slots)
        .map(|slot| {
            if slot < known {
                typed(random, words).iter().map(|&it| it as i32).collect()
            } else {
                Vec::new()
            }
        })
        .collect();
    let beginning_of_sentence = (0..slots).map(|slot| slot >= known).collect();
    Query {
        code_points,
        points,
        block_offensive_words: random.chance(50),
        previous,
        beginning_of_sentence,
    }
}
