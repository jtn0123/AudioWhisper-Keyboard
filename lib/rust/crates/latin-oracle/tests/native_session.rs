//! The production session against the independent C++ decoder, using real dictionary questions.
//! The C++'s answers go on a golden tape in `crates/suggest/tests/golden/` (`LATIN_GOLDEN=write`
//! writes it), which `suggest`'s own copy of this test checks the session against.
use latin_golden::queries::{self, Random};
use latin_golden::{Suggestions, Tape};
use latin_oracle::{Dictionary, Proximity, Session};
use std::sync::Arc;

/// A generated question as the C++ side takes it.
fn oracle_query(query: &queries::Query) -> latin_oracle::Query {
    latin_oracle::Query {
        code_points: query.code_points.clone(),
        points: query.points.clone(),
        block_offensive_words: query.block_offensive_words,
        previous: query.previous.clone(),
        beginning_of_sentence: query.beginning_of_sentence.clone(),
    }
}

#[test]
fn native_session_matches_four_thousand_seeded_cpp_questions() {
    let goldens = concat!(env!("CARGO_MANIFEST_DIR"), "/../suggest");
    for (name, path) in latin_golden::dictionaries() {
        let mut tape = Tape::open(goldens, &format!("session.native.{name}"));
        let bytes = std::fs::read(&path).expect("a dictionary");
        let mut native = suggest::session::Session::open(Arc::from(bytes)).expect("native session");
        let dictionary = Dictionary::at(&path);
        let proximity = Proximity::qwerty();
        let session = Session::new("en_US", &dictionary);
        let text = String::from_utf16(&dictionary.words()).expect("UTF-16 dictionary");
        let words: Vec<Vec<char>> = text.lines().map(|w| w.chars().collect()).collect();
        let native_words: Vec<u16> = native
            .words()
            .iter()
            .flat_map(|w| w.iter().copied().chain([10]))
            .collect();
        assert_eq!(native_words, dictionary.words());
        tape.record("the dictionary's words", &dictionary.words());
        let mut random = Random::new(20261113);
        for index in 0..4000 {
            let q = queries::query(&mut random, &words);
            let expected =
                latin_oracle::suggest(&dictionary, &proximity, &session, &oracle_query(&q));
            let theirs = Suggestions {
                count: expected.count,
                code_points: &expected.code_points,
                scores: &expected.scores,
                types: &expected.types,
                space_indices: &expected.space_indices,
                auto_commit_confidence: expected.auto_commit_confidence,
                weight: expected.weight,
            };
            tape.record("latin_oracle::suggest(q)", &theirs);
            let actual = native.query(&latin::search::Query {
                code_points: q.code_points,
                points: q.points,
                block_offensive_words: q.block_offensive_words,
                previous: q.previous,
                beginning_of_sentence: q.beginning_of_sentence,
            });
            assert_eq!(actual.count, expected.count, "count {index}");
            let count = expected.count as usize;
            for slot in 0..count {
                let range = slot * 48..(slot + 1) * 48;
                let a = actual.code_points[range.clone()]
                    .iter()
                    .take_while(|&&cp| cp != 0)
                    .collect::<Vec<_>>();
                let b = expected.code_points[range]
                    .iter()
                    .take_while(|&&cp| cp != 0)
                    .collect::<Vec<_>>();
                assert_eq!(a, b, "code points {index}/{slot}");
            }
            assert_eq!(
                actual.scores[..count],
                expected.scores[..count],
                "scores {index}"
            );
            assert_eq!(
                actual.types[..count],
                expected.types[..count],
                "types {index}"
            );
            assert_eq!(
                actual.space_indices[..count],
                expected.space_indices[..count],
                "space {index}"
            );
            if count > 0 {
                assert_eq!(
                    actual.auto_commit_confidence, expected.auto_commit_confidence,
                    "confidence {index}"
                );
            }
            assert_eq!(
                actual.weight.to_bits(),
                expected.weight.to_bits(),
                "weight {index}"
            );
        }
        tape.finish();
    }
}
