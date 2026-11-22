//! Input the app never sends, which the C++ reads out of bounds or asserts on: the Rust must not
//! panic, and answers within bounds.

use super::support::{new_session, rust_side};
use crate::defines::{MAX_RESULTS, MAX_WORD_LENGTH};
use crate::search::{Query, jni};

/// SplitMix64, as `latin_golden`'s generator.
struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn code_point(&mut self) -> i32 {
        match self.below(12) {
            0 => -1,
            1 => 0,
            2 => 0x11_0000,
            3 => i32::MIN,
            4 => i32::MAX,
            5 => 0x1F600,
            6 => i32::from(b' '),
            _ => i32::from(b'a') + self.below(26) as i32,
        }
    }
}

fn hostile(random: &mut Random) -> Query {
    let size = random.below(MAX_WORD_LENGTH + 10);
    let code_points = (0..size).map(|_| random.code_point()).collect();
    let points = (0..size)
        .map(|_| match random.below(4) {
            0 => None,
            1 => Some((
                random.below(3000) as i32 - 1000,
                random.below(2000) as i32 - 800,
            )),
            _ => Some((random.below(1000) as i32, random.below(450) as i32)),
        })
        .collect();
    let slots = random.below(5);
    let previous: Vec<Vec<i32>> = (0..slots)
        .map(|_| (0..random.below(60)).map(|_| random.code_point()).collect())
        .collect();
    let beginning_of_sentence = (0..slots).map(|_| random.below(2) == 0).collect();
    Query {
        code_points,
        points,
        block_offensive_words: random.below(2) == 0,
        previous,
        beginning_of_sentence,
    }
}

#[test]
fn hostile_queries_do_not_panic() {
    let (dictionary, info) = rust_side();
    let mut session = new_session();
    let mut random = Random(5);
    for _ in 0..4_000 {
        let query = hostile(&mut random);
        let output = jni::suggest(&dictionary, &info, &mut session, &query);
        assert!((0..=MAX_RESULTS as i32).contains(&output.count));
    }
}

#[test]
fn hostile_options_and_weights_do_not_panic() {
    let (dictionary, info) = rust_side();
    let mut session = new_session();
    let mut random = Random(6);
    let weights = [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -5.0,
        0.0,
        f32::MAX,
    ];
    for round in 0..1_500 {
        let query = hostile(&mut random);
        let options = [
            0,
            random.below(3) as i32 - 1,
            random.below(3) as i32,
            0,
            random.code_point(),
        ];
        let output = jni::suggest_with(
            &dictionary,
            &info,
            &mut session,
            &query,
            &options,
            weights[round % weights.len()],
        );
        assert!((0..=MAX_RESULTS as i32).contains(&output.count));
    }
}

/// A gesture is not ported: no suggestions, and the weight given comes back.
#[test]
fn a_gesture_gives_nothing() {
    let (dictionary, info) = rust_side();
    let mut session = new_session();
    let query = Query {
        code_points: vec![i32::from(b'a'); 3],
        points: vec![Some((50, 50)); 3],
        ..Query::default()
    };
    let output = jni::suggest_with(
        &dictionary,
        &info,
        &mut session,
        &query,
        &[1, 0, 0, 0, 1000],
        0.5,
    );
    assert_eq!(output.count, 0);
    assert_eq!(output.weight, 0.5);
}

/// A query longer than a word is cut to the longest word.
#[test]
fn an_over_long_query_is_cut() {
    let (dictionary, info) = rust_side();
    let mut session = new_session();
    let long = Query {
        code_points: vec![i32::from(b'a'); MAX_WORD_LENGTH + 20],
        points: vec![Some((50, 50)); MAX_WORD_LENGTH + 20],
        ..Query::default()
    };
    let cut = Query {
        code_points: long.code_points[..MAX_WORD_LENGTH].to_vec(),
        points: long.points[..MAX_WORD_LENGTH].to_vec(),
        ..Query::default()
    };
    let a = jni::suggest(&dictionary, &info, &mut session, &long);
    let mut other = new_session();
    let b = jni::suggest(&dictionary, &info, &mut other, &cut);
    assert_eq!(a, b);
}

/// Once warm, a call grows none of the storage the session reuses.
#[test]
fn a_warm_session_does_not_grow() {
    let (dictionary, info) = rust_side();
    let mut session = new_session();
    let mut random = Random(9);
    let queries: Vec<Query> = (0..300).map(|_| hostile(&mut random)).collect();
    for query in &queries {
        jni::suggest(&dictionary, &info, &mut session, query);
    }
    let capacity = |session: &crate::search::DicTraverseSession| {
        (
            session.scratch.terminals.capacity(),
            session.scratch.children.capacity(),
            session.scratch.grand_children.capacity(),
            session.scratch.second_grand_children.capacity(),
        )
    };
    let warm = capacity(&session);
    for query in &queries {
        jni::suggest(&dictionary, &info, &mut session, query);
    }
    assert_eq!(capacity(&session), warm);
}
