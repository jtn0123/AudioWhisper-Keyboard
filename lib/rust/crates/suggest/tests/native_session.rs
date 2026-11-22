//! The production session against LatinIME's C++ decoder, on real dictionary questions: the
//! C++'s answers to four thousand seeded queries per dictionary, kept as golden files in
//! `tests/golden/` (latin-golden's `Tape`), which `latin-oracle`'s copy of this test wrote.

use std::sync::Arc;

use latin_golden::queries::{self, Random};
use latin_golden::{Suggestions, Tape};

#[test]
fn native_session_matches_the_goldens_for_four_thousand_seeded_questions() {
    for (name, path) in latin_golden::dictionaries() {
        let bytes = std::fs::read(&path).expect("a dictionary");
        let mut native = suggest::session::Session::open(Arc::from(bytes)).expect("native session");
        let mut tape = Tape::open(
            env!("CARGO_MANIFEST_DIR"),
            &format!("session.native.{name}"),
        );
        let native_words = native.words();
        let joined: Vec<u16> = native_words
            .iter()
            .flat_map(|w| w.iter().copied().chain([10]))
            .collect();
        tape.record("the dictionary's words", &joined);
        let words: Vec<Vec<char>> = native_words
            .iter()
            .map(|w| {
                String::from_utf16(w)
                    .expect("UTF-16 word")
                    .chars()
                    .collect()
            })
            .collect();
        let mut random = Random::new(20261113);
        for _ in 0..4000 {
            let q = queries::query(&mut random, &words);
            let actual = native.query(&latin::search::Query {
                code_points: q.code_points,
                points: q.points,
                block_offensive_words: q.block_offensive_words,
                previous: q.previous,
                beginning_of_sentence: q.beginning_of_sentence,
            });
            let suggestions = Suggestions {
                count: actual.count,
                code_points: &actual.code_points,
                scores: &actual.scores,
                types: &actual.types,
                space_indices: &actual.space_indices,
                auto_commit_confidence: actual.auto_commit_confidence,
                weight: actual.weight,
            };
            tape.record("native.query(q)", &suggestions);
        }
        tape.finish();
    }
}
