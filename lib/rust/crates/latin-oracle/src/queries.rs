//! Seeded decoder questions shaped like the keyboard's: real words typed cleanly, with typos or
//! half-finished, gibberish, odd case and characters, empty input (next-word prediction), taps off
//! the key centre, and up to three previous words with the sentence-start slot the app adds.
//!
//! Also a line format for queries and outputs, so the same questions can be put to the decoder on
//! the emulator and the answers compared with these.

use crate::reference_qwerty as qwerty;
use crate::reference_qwerty::TapOffset;

use crate::{MAX_PREVIOUS_WORDS, MAX_WORD_LENGTH, Output, Query};

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

fn ints(values: impl IntoIterator<Item = i32>) -> String {
    values
        .into_iter()
        .map(|it| it.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

fn parse_ints(text: &str) -> Vec<i32> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split('.')
        .map(|it| it.parse().expect("an int"))
        .collect()
}

/// A query as one line: `codes|xs|ys|block|previous words (comma-separated)|start flags`.
pub fn query_line(query: &Query) -> String {
    let xs = query.points.iter().map(|it| it.map_or(-1, |(x, _)| x));
    let ys = query.points.iter().map(|it| it.map_or(-1, |(_, y)| y));
    let previous: Vec<String> = query
        .previous
        .iter()
        .map(|it| ints(it.iter().copied()))
        .collect();
    let flags = ints(query.beginning_of_sentence.iter().map(|&it| i32::from(it)));
    format!(
        "{}|{}|{}|{}|{}|{}",
        ints(query.code_points.iter().copied()),
        ints(xs),
        ints(ys),
        i32::from(query.block_offensive_words),
        previous.join(","),
        flags
    )
}

/// [`query_line`]'s query back.
pub fn parse_query(line: &str) -> Query {
    let fields: Vec<&str> = line.split('|').collect();
    assert_eq!(fields.len(), 6, "six fields");
    let code_points = parse_ints(fields[0]);
    let (xs, ys) = (parse_ints(fields[1]), parse_ints(fields[2]));
    let points = xs
        .iter()
        .zip(&ys)
        .map(|(&x, &y)| (x != -1 || y != -1).then_some((x, y)))
        .collect();
    let flags: Vec<bool> = parse_ints(fields[5]).iter().map(|&it| it != 0).collect();
    let previous = if flags.is_empty() {
        Vec::new()
    } else {
        fields[4].split(',').map(parse_ints).collect()
    };
    Query {
        code_points,
        points,
        block_offensive_words: fields[3] == "1",
        previous,
        beginning_of_sentence: flags,
    }
}

/// What the app reads from an output, as one line: `count|word:score:type,...|confidence|weight`,
/// the weight as its bits.
pub fn output_line(output: &Output) -> String {
    let suggestions: Vec<String> = output
        .suggestions()
        .iter()
        .map(|(word, score, kind)| format!("{}:{score}:{kind}", ints(word.iter().copied())))
        .collect();
    format!(
        "{}|{}|{}|{}",
        output.count,
        suggestions.join(","),
        output.auto_commit_confidence,
        output.weight.to_bits()
    )
}
