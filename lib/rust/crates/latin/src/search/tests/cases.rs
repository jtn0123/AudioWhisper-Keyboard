//! Single queries on a fresh session each: the seeded generator, predictions, sentence starts,
//! offensive blocking and capital letters.

use std::time::Instant;

use latin_golden::queries::{self, Query, Random};

use super::support::{Pair, in_chunks, prefix, record, tap};
use crate::defines::NOT_A_FIRST_WORD_CONFIDENCE;
use crate::search::dictionary::{KIND_PREDICTION, KIND_SHORTCUT, KIND_WHITELIST};

/// What the compared outputs covered, so a test can fail if it compared nothing interesting.
#[derive(Default)]
struct Coverage {
    non_empty: usize,
    predictions: usize,
    multi_word: usize,
    whitelist: usize,
    shortcuts: usize,
    flagged: usize,
    /// Answers whose best suggestion has an auto-commit confidence (a multi-word one).
    confident: usize,
}

impl Coverage {
    /// The coverage of every output.
    fn of<'a>(outputs: impl IntoIterator<Item = &'a crate::search::Output>) -> Self {
        let mut coverage = Self::default();
        outputs.into_iter().for_each(|it| coverage.add(it));
        coverage
    }

    fn add(&mut self, output: &crate::search::Output) {
        let count = output.count as usize;
        self.non_empty += usize::from(count > 0);
        self.confident +=
            usize::from(count > 0 && output.auto_commit_confidence != NOT_A_FIRST_WORD_CONFIDENCE);
        for index in 0..count {
            let kind = output.types[index];
            self.predictions += usize::from(kind & 0xFF == KIND_PREDICTION);
            self.whitelist += usize::from(kind & 0xFF == KIND_WHITELIST);
            self.shortcuts += usize::from(kind & 0xFF == KIND_SHORTCUT);
            self.flagged += usize::from(kind & !0xFF != 0);
            let word = &output.code_points[index * 48..(index + 1) * 48];
            self.multi_word += usize::from(word.contains(&i32::from(b' ')));
        }
    }
}

/// One query on a fresh session per side, compared; the Rust output.
fn fresh(pair: &Pair, query: &Query) -> crate::search::Output {
    pair.compare(&mut pair.sessions(), query)
}

/// Every query on a fresh session, compared, and the outputs recorded on `test`'s tape.
fn ask_fresh(pair: &Pair, test: &str, inputs: &[Query]) -> Vec<crate::search::Output> {
    let outputs = in_chunks(inputs, || (), |(), _, query| fresh(pair, query));
    let mut tape = pair.tape(test);
    record(&mut tape, &outputs);
    tape.finish();
    outputs
}

#[test]
fn seeded_queries_match_the_goldens() {
    for pair in Pair::each() {
        let mut random = Random::new(20_261_109);
        let started = Instant::now();
        let total = 20_000;
        let inputs: Vec<_> = (0..total)
            .map(|_| queries::query(&mut random, &pair.words))
            .collect();
        let outputs = ask_fresh(pair, "seeded_queries", &inputs);
        let coverage = Coverage::of(&outputs);
        eprintln!("{total} queries in {:?}", started.elapsed());
        assert_eq!(outputs.len(), total);
        assert!(coverage.non_empty > total / 2, "{}", coverage.non_empty);
        assert!(coverage.predictions > 20, "{}", coverage.predictions);
        assert!(coverage.multi_word > 100, "{}", coverage.multi_word);
        assert!(coverage.flagged > 100, "{}", coverage.flagged);
        assert!(coverage.confident > 20, "{}", coverage.confident);
        eprintln!(
            "whitelist {} shortcuts {} multi-word {} confident {}",
            coverage.whitelist, coverage.shortcuts, coverage.multi_word, coverage.confident
        );
    }
}

/// Empty input is next-word prediction: after one, two or three words, after a sentence start
/// alone, and after nothing.
#[test]
fn predictions_match_the_goldens() {
    for pair in Pair::each() {
        let mut random = Random::new(7);
        let inputs: Vec<_> = (0..3_000)
            .map(|round| {
                let mut query = queries::query(&mut random, &pair.words);
                query.code_points.clear();
                query.points.clear();
                if round % 7 == 0 {
                    query.previous.clear();
                    query.beginning_of_sentence.clear();
                }
                if round % 11 == 0 {
                    query.previous = vec![Vec::new()];
                    query.beginning_of_sentence = vec![true];
                }
                query
            })
            .collect();
        let outputs = ask_fresh(pair, "predictions", &inputs);
        let coverage = Coverage::of(&outputs);
        assert!(coverage.predictions > 500, "{}", coverage.predictions);
    }
}

/// The first word of a text: the previous slot is only the start of a sentence, with the word
/// typed or not.
#[test]
fn sentence_starts_match_the_goldens() {
    for pair in Pair::each() {
        let mut random = Random::new(11);
        let inputs: Vec<_> = (0..4_000)
            .map(|round| {
                let mut query = queries::query(&mut random, &pair.words);
                query.previous = vec![Vec::new(); 1 + round % 3];
                query.beginning_of_sentence = vec![true; 1 + round % 3];
                if round % 4 == 0 {
                    query.code_points.clear();
                    query.points.clear();
                }
                query
            })
            .collect();
        let outputs = ask_fresh(pair, "sentence_starts", &inputs);
        let coverage = Coverage::of(&outputs);
        assert!(coverage.non_empty > 2_000, "{}", coverage.non_empty);
    }
}

/// The same questions with offensive words blocked and not: each matches the C++, and blocking
/// changes some answers.
#[test]
fn offensive_blocking_matches_the_goldens() {
    for pair in Pair::each() {
        let mut random = Random::new(13);
        let inputs: Vec<_> = (0..6_000)
            .map(|_| queries::query(&mut random, &pair.words))
            .collect();
        let answers = in_chunks(
            &inputs,
            || (),
            |(), _, query| {
                let mut query = query.clone();
                query.block_offensive_words = false;
                let open = fresh(pair, &query);
                query.block_offensive_words = true;
                (open, fresh(pair, &query))
            },
        );
        let mut tape = pair.tape("offensive_blocking");
        record(
            &mut tape,
            answers.iter().flat_map(|(open, blocked)| [open, blocked]),
        );
        tape.finish();
        let differing = answers
            .iter()
            .filter(|(open, blocked)| open != blocked)
            .count();
        eprintln!("blocking changed {differing} answers");
        assert!(differing > 0, "no answer had an offensive word");
    }
}

/// Capital letters: the first letter, every letter, and a letter in the middle.
#[test]
fn capitalised_input_matches_the_goldens() {
    for pair in Pair::each() {
        let mut random = Random::new(17);
        let upper = |code: i32| {
            char::from_u32(code as u32)
                .and_then(|it| {
                    let mut upper = it.to_uppercase();
                    let first = upper.next()?;
                    upper.next().is_none().then_some(first as i32)
                })
                .unwrap_or(code)
        };
        let inputs: Vec<_> = (0..6_000)
            .map(|round| {
                let mut query = queries::query(&mut random, &pair.words);
                match round % 3 {
                    0 => {
                        if let Some(first) = query.code_points.first_mut() {
                            *first = upper(*first);
                        }
                    }
                    1 => query.code_points.iter_mut().for_each(|it| *it = upper(*it)),
                    _ => {
                        if !query.code_points.is_empty() {
                            let at = random.below(query.code_points.len());
                            query.code_points[at] = upper(query.code_points[at]);
                        }
                    }
                }
                query
            })
            .collect();
        let outputs = ask_fresh(pair, "capitalised_input", &inputs);
        let coverage = Coverage::of(&outputs);
        assert!(coverage.non_empty > 3_000, "{}", coverage.non_empty);
    }
}

/// Taps with no coordinates (the app sends none for a key it could not place): every tap
/// missing, or every other one, typed letter by letter on one session per side (one per run of
/// rounds; see [`in_chunks`]).
#[test]
fn missing_coordinates_match_the_goldens() {
    for pair in Pair::each() {
        let mut random = Random::new(19);
        let inputs: Vec<_> = (0..1_500)
            .map(|round| {
                let mut query = queries::query(&mut random, &pair.words);
                for (index, point) in query.points.iter_mut().enumerate() {
                    if round % 2 == 0 || index % 2 == 1 {
                        *point = None;
                    }
                }
                query
            })
            .collect();
        let outputs = in_chunks(
            &inputs,
            || pair.sessions(),
            |sessions, _, query| {
                (1..=query.code_points.len())
                    .map(|length| pair.compare(sessions, &prefix(query, length)))
                    .collect::<Vec<_>>()
            },
        );
        let mut tape = pair.tape("missing_coordinates");
        record(&mut tape, outputs.iter().flatten());
        tape.finish();
        let calls = outputs.iter().map(Vec::len).sum::<usize>();
        let coverage = Coverage::of(outputs.iter().flatten());
        assert!(
            coverage.non_empty > calls / 2,
            "{} of {calls}",
            coverage.non_empty
        );
    }
}

/// The longest input the app sends, 48 taps (it drops anything longer), and 47 beside it: words
/// run together until they fill it, half the time with a typo, with taps or without.
#[test]
fn the_longest_inputs_match_the_goldens() {
    for pair in Pair::each() {
        let mut random = Random::new(48);
        let inputs: Vec<_> = (0..400)
            .map(|round| {
                let length = 48 - round % 2;
                let mut code_points = Vec::with_capacity(length + 48);
                while code_points.len() < length {
                    let word = &pair.words[random.below(pair.words.len())];
                    code_points.extend(word.iter().map(|&c| c as i32));
                }
                code_points.truncate(length);
                if random.below(2) == 0 {
                    code_points[random.below(length)] = i32::from(b'a') + random.below(26) as i32;
                }
                let tapped = random.below(2) == 0;
                let points = code_points
                    .iter()
                    .map(|&c| if tapped { tap(&mut random, c) } else { None })
                    .collect();
                Query {
                    code_points,
                    points,
                    block_offensive_words: false,
                    previous: Vec::new(),
                    beginning_of_sentence: Vec::new(),
                }
            })
            .collect();
        let outputs = ask_fresh(pair, "the_longest_inputs", &inputs);
        assert_eq!(outputs.len(), 400);
    }
}
