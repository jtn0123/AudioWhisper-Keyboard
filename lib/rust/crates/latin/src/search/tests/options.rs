//! The options array and the weight the app never changes: the C++ and the Rust given the same
//! ones. A weight other than 1.0 reaches `getCompoundDistance`, the first of the two expressions
//! the compiler fuses into a multiply-add (see the module docs).

use latin_oracle::hooks::search::suggest_with;
use latin_oracle::queries::{self, Random};

use super::support::{Pair, assert_same, convert, in_chunks, new_session};
use crate::search::jni;

/// Weights from the app's NOT_A_WEIGHT through ones that move every score.
const WEIGHTS: [f32; 9] = [-1.0, 0.0, 0.25, 0.5, 1.0, 1.3, 2.0, 7.7, 1.0e6];

#[test]
fn weights_and_options_match_the_cpp() {
    let pair = Pair::shared();
    let mut random = Random::new(31);
    let locale_weights = [0, 1, 500, 989, 990, 1000, 1500, 100_000];
    let inputs: Vec<_> = (0..9_000_usize)
        .map(|round| {
            let query = queries::query(&mut random, &pair.words);
            let options = [
                0,
                (round % 2) as i32,
                i32::from(query.block_offensive_words),
                (round % 3 == 0) as i32,
                locale_weights[random.below(locale_weights.len())],
            ];
            (query, options)
        })
        .collect();
    let rounds = in_chunks(
        &inputs,
        || (),
        |(), round, (query, options)| {
            let weight = WEIGHTS[round % WEIGHTS.len()];
            let cpp_session = pair.cpp_session();
            let mut session = new_session();
            let theirs = suggest_with(
                &pair.cpp_dictionary,
                &pair.cpp_proximity,
                &cpp_session,
                query,
                options,
                weight,
            );
            let ours = jni::suggest_with(
                &pair.dictionary,
                &pair.info,
                &mut session,
                &convert(query),
                options,
                weight,
            );
            assert_same(
                &ours,
                &theirs,
                &format!("round {round} weight {weight} {options:?}"),
            );
        },
    );
    assert_eq!(rounds.len(), 9_000);
}

/// The weight and options on one session across a word typed letter by letter.
#[test]
fn weights_while_typing_match_the_cpp() {
    let pair = Pair::shared();
    let mut random = Random::new(37);
    let inputs: Vec<_> = (0..600)
        .map(|_| queries::query(&mut random, &pair.words))
        .collect();
    let calls = in_chunks(
        &inputs,
        || (),
        |(), round, query| {
            let weight = WEIGHTS[round % WEIGHTS.len()];
            let options = [0, 0, 0, 0, 1000];
            let cpp_session = pair.cpp_session();
            let mut session = new_session();
            for length in 1..=query.code_points.len() {
                let input = latin_oracle::Query {
                    code_points: query.code_points[..length].to_vec(),
                    points: query.points[..length].to_vec(),
                    ..query.clone()
                };
                let theirs = suggest_with(
                    &pair.cpp_dictionary,
                    &pair.cpp_proximity,
                    &cpp_session,
                    &input,
                    &options,
                    weight,
                );
                let ours = jni::suggest_with(
                    &pair.dictionary,
                    &pair.info,
                    &mut session,
                    &convert(&input),
                    &options,
                    weight,
                );
                assert_same(
                    &ours,
                    &theirs,
                    &format!("round {round} length {length} weight {weight}"),
                );
            }
            query.code_points.len()
        },
    );
    assert!(calls.iter().sum::<usize>() > 600, "the words were typed");
}
