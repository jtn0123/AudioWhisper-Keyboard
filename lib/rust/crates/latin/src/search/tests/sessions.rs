//! One session across calls, as the keyboard keeps one: a word typed letter by
//! letter and deleted back, and unrelated queries one after another. Each run of rounds
//! [`in_chunks`] gives a thread has its own session, kept across its rounds.

use std::time::Instant;

use latin_golden::queries::{self, Query, Random};

use super::support::{Fixture, in_chunks, prefix, record};

/// A word typed letter by letter then deleted back, on one session. Sometimes a tap in
/// the middle changes or the previous words do, so the cache must not continue.
#[test]
fn words_typed_and_deleted_match_the_goldens() {
    for fixture in Fixture::each() {
        let mut random = Random::new(21);
        let started = Instant::now();
        let inputs: Vec<Vec<Query>> = (0..2_500)
            .map(|_| {
                let query = queries::query(&mut random, &fixture.words);
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
        let rounds = in_chunks(
            &inputs,
            || fixture.session(),
            |session, _, steps| {
                let mut continued = 0;
                let outputs: Vec<_> = steps
                    .iter()
                    .map(|input| {
                        let before = session.dic_nodes_cache.continued_searches;
                        let output = fixture.ask(session, input);
                        continued += session.dic_nodes_cache.continued_searches - before;
                        output
                    })
                    .collect();
                (outputs, continued)
            },
        );
        let mut tape = fixture.tape("words_typed_and_deleted");
        record(&mut tape, rounds.iter().flat_map(|it| &it.0));
        tape.finish();
        let calls: usize = rounds.iter().map(|it| it.0.len()).sum();
        let continued: usize = rounds.iter().map(|it| it.1).sum();
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
}

/// Unrelated queries one after another on one session: whatever the cache holds from the last
/// must never leak into the next answer.
#[test]
fn unrelated_queries_on_one_session_match_the_goldens() {
    for fixture in Fixture::each() {
        let mut random = Random::new(23);
        let inputs: Vec<_> = (0..8_000)
            .map(|_| queries::query(&mut random, &fixture.words))
            .collect();
        let outputs = in_chunks(
            &inputs,
            || fixture.session(),
            |session, round, query| {
                let mut outputs = vec![fixture.ask(session, query)];
                if round % 5 == 0 {
                    // The same again, as a re-decode of the same word does.
                    outputs.push(fixture.ask(session, query));
                }
                outputs
            },
        );
        assert_eq!(outputs.iter().map(Vec::len).sum::<usize>(), 9_600);
        let mut tape = fixture.tape("unrelated_queries_on_one_session");
        record(&mut tape, outputs.iter().flatten());
        tape.finish();
    }
}

/// A word typed to the end, then predictions, then the next word: the prediction call must not
/// disturb the cache a later call continues.
#[test]
fn predictions_between_typing_match_the_goldens() {
    for fixture in Fixture::each() {
        let mut random = Random::new(29);
        let inputs: Vec<_> = (0..800)
            .map(|_| queries::query(&mut random, &fixture.words))
            .collect();
        let rounds = in_chunks(
            &inputs,
            || fixture.session(),
            |session, _, query| {
                let mut outputs = Vec::new();
                let mut predictions = 0;
                for length in 1..=query.code_points.len() {
                    outputs.push(fixture.ask(session, &prefix(query, length)));
                    if length % 3 == 0 {
                        outputs.push(fixture.ask(session, &prefix(query, 0)));
                        predictions += 1;
                    }
                }
                (outputs, predictions)
            },
        );
        let mut tape = fixture.tape("predictions_between_typing");
        record(&mut tape, rounds.iter().flat_map(|it| &it.0));
        tape.finish();
        let predictions: usize = rounds.iter().map(|it| it.1).sum();
        assert!(
            predictions > 100,
            "only {predictions} predictions between taps"
        );
    }
}
