//! The options array and the weight the app never changes: the search given other
//! ones. A weight other than 1.0 reaches `getCompoundDistance`, the first of the two expressions
//! the compiler fuses into a multiply-add (see the module docs).

use latin_golden::queries::{self, Random};

use super::support::{Fixture, in_chunks, prefix, record};

/// Weights from the app's NOT_A_WEIGHT through ones that move every score.
const WEIGHTS: [f32; 9] = [-1.0, 0.0, 0.25, 0.5, 1.0, 1.3, 2.0, 7.7, 1.0e6];

#[test]
fn weights_and_options_match_the_goldens() {
    for fixture in Fixture::each() {
        let mut random = Random::new(31);
        let locale_weights = [0, 1, 500, 989, 990, 1000, 1500, 100_000];
        let inputs: Vec<_> = (0..9_000_usize)
            .map(|round| {
                let query = queries::query(&mut random, &fixture.words);
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
        let outputs = in_chunks(
            &inputs,
            || (),
            |(), round, (query, options)| {
                let weight = WEIGHTS[round % WEIGHTS.len()];
                fixture.ask_with(&mut fixture.session(), query, (options, weight))
            },
        );
        assert_eq!(outputs.len(), 9_000);
        let mut tape = fixture.tape("weights_and_options");
        record(&mut tape, &outputs);
        tape.finish();
    }
}

/// The weight and options on one session across a word typed letter by letter.
#[test]
fn weights_while_typing_match_the_goldens() {
    for fixture in Fixture::each() {
        let mut random = Random::new(37);
        let inputs: Vec<_> = (0..600)
            .map(|_| queries::query(&mut random, &fixture.words))
            .collect();
        let outputs = in_chunks(
            &inputs,
            || (),
            |(), round, query| {
                let weight = WEIGHTS[round % WEIGHTS.len()];
                let options = [0, 0, 0, 0, 1000];
                let mut session = fixture.session();
                (1..=query.code_points.len())
                    .map(|length| {
                        let input = prefix(query, length);
                        fixture.ask_with(&mut session, &input, (&options, weight))
                    })
                    .collect::<Vec<_>>()
            },
        );
        let calls = outputs.iter().map(Vec::len).sum::<usize>();
        assert!(calls > 600, "the words were typed");
        let mut tape = fixture.tape("weights_while_typing");
        record(&mut tape, outputs.iter().flatten());
        tape.finish();
    }
}
