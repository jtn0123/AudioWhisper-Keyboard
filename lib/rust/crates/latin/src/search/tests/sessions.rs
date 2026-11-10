//! One session on each side across calls, as the keyboard keeps one: a word typed letter by
//! letter and deleted back, and unrelated queries one after another. Each run of rounds
//! [`in_chunks`] gives a thread has its own pair of sessions, kept across its rounds.

use std::time::Instant;

use latin_oracle::queries::{self, Random};

use super::support::{Pair, in_chunks, new_session};

/// The query for the first `length` taps of `query`.
fn prefix(query: &latin_oracle::Query, length: usize) -> latin_oracle::Query {
    latin_oracle::Query {
        code_points: query.code_points[..length].to_vec(),
        points: query.points[..length].to_vec(),
        ..query.clone()
    }
}

/// A word typed letter by letter then deleted back, on one session per side. Sometimes a tap in
/// the middle changes or the previous words do, so the cache must not continue.
#[test]
fn words_typed_and_deleted_match_the_cpp() {
    let pair = Pair::shared();
    let mut random = Random::new(21);
    let started = Instant::now();
    let inputs: Vec<Vec<latin_oracle::Query>> = (0..2_500)
        .map(|_| {
            let query = queries::query(&mut random, &pair.words);
            let size = query.code_points.len();
            let mut lengths: Vec<usize> = (1..=size).collect();
            lengths.extend((1..size).rev());
            lengths
                .into_iter()
                .map(|length| {
                    let mut input = prefix(&query, length);
                    if length > 2 && random.chance(6) {
                        let at = random.below(length);
                        match random.below(3) {
                            0 => input.code_points[at] = i32::from(b'q'),
                            1 => input.points[at] = input.points[at].map(|it| (it.0 + 1, it.1)),
                            _ => input.previous.clear(),
                        }
                        if input.previous.is_empty() {
                            input.beginning_of_sentence.clear();
                        }
                    }
                    input
                })
                .collect()
        })
        .collect();
    let counts = in_chunks(
        &inputs,
        || (pair.cpp_session(), new_session()),
        |(cpp_session, session), round, steps| {
            let mut continued = 0;
            for (step, input) in steps.iter().enumerate() {
                let before = session.dic_nodes_cache.continued_searches;
                pair.compare(
                    (session, cpp_session),
                    input,
                    &format!("round {round} step {step}"),
                );
                continued += session.dic_nodes_cache.continued_searches - before;
            }
            (steps.len(), continued)
        },
    );
    let calls: usize = counts.iter().map(|it| it.0).sum();
    let continued: usize = counts.iter().map(|it| it.1).sum();
    eprintln!(
        "{calls} calls, {continued} continuing, {:?}",
        started.elapsed()
    );
    // The search really continued from the cache (not merely could have) on a good share of the
    // calls: a broken continuation would show in the answers of those calls.
    assert!(
        continued > calls / 4,
        "only {continued} of {calls} calls continued"
    );
}

/// Unrelated queries one after another on one session: whatever the cache holds from the last
/// must never leak into the next answer.
#[test]
fn unrelated_queries_on_one_session_match_the_cpp() {
    let pair = Pair::shared();
    let mut random = Random::new(23);
    let inputs: Vec<_> = (0..8_000)
        .map(|_| queries::query(&mut random, &pair.words))
        .collect();
    let calls = in_chunks(
        &inputs,
        || (pair.cpp_session(), new_session()),
        |(cpp_session, session), round, query| {
            pair.compare((session, cpp_session), query, &format!("round {round}"));
            if round % 5 == 0 {
                // The same again, as a re-decode of the same word does.
                pair.compare((session, cpp_session), query, &format!("repeat {round}"));
            }
            1 + usize::from(round % 5 == 0)
        },
    );
    assert_eq!(calls.iter().sum::<usize>(), 9_600);
}

/// A word typed to the end, then predictions, then the next word: the prediction call must not
/// disturb the cache a later call continues.
#[test]
fn predictions_between_typing_match_the_cpp() {
    let pair = Pair::shared();
    let mut random = Random::new(29);
    let inputs: Vec<_> = (0..800)
        .map(|_| queries::query(&mut random, &pair.words))
        .collect();
    let predictions = in_chunks(
        &inputs,
        || (pair.cpp_session(), new_session()),
        |(cpp_session, session), round, query| {
            let mut predictions = 0;
            for length in 1..=query.code_points.len() {
                pair.compare(
                    (session, cpp_session),
                    &prefix(query, length),
                    &format!("typing {round}/{length}"),
                );
                if length % 3 == 0 {
                    pair.compare(
                        (session, cpp_session),
                        &prefix(query, 0),
                        &format!("prediction {round}/{length}"),
                    );
                    predictions += 1;
                }
            }
            predictions
        },
    );
    let predictions: usize = predictions.iter().sum();
    assert!(
        predictions > 100,
        "only {predictions} predictions between taps"
    );
}
