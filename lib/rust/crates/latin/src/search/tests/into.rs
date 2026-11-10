//! `suggest_into`, which writes into arrays the caller keeps, as the JNI method writes Java's: it
//! must write what `suggest_with` does and leave the rest as it was.

use super::support::{new_session, rust_side};
use crate::defines::{MAX_RESULTS, MAX_WORD_LENGTH, NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL};
use crate::search::{Output, Query, suggest_into, suggest_with};

const OPTIONS: [i32; 5] = [0, 0, 0, 0, 1000];
const STALE: i32 = 7_777;

/// Arrays as a previous call might have left them: every value `STALE`.
fn stale() -> Output {
    Output {
        count: STALE,
        code_points: vec![STALE; MAX_WORD_LENGTH * MAX_RESULTS],
        scores: vec![STALE; MAX_RESULTS],
        space_indices: vec![STALE; MAX_RESULTS],
        types: vec![STALE; MAX_RESULTS],
        auto_commit_confidence: STALE,
        weight: NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL,
    }
}

fn typed(word: &str) -> Query {
    let code_points: Vec<i32> = word.chars().map(|c| c as i32).collect();
    Query {
        points: vec![None; code_points.len()],
        code_points,
        ..Query::default()
    }
}

/// Each result's slot up to its terminator, and its score, type and space index, match a fresh
/// call; everything past them keeps its stale value.
fn assert_written_like_fresh(fresh: &Output, into: &Output) {
    let count = usize::try_from(fresh.count).unwrap_or(0);
    assert_eq!(into.count, fresh.count);
    assert_eq!(into.weight.to_bits(), fresh.weight.to_bits());
    for index in 0..MAX_RESULTS {
        let slot = index * MAX_WORD_LENGTH..(index + 1) * MAX_WORD_LENGTH;
        let (fresh_slot, into_slot) = (&fresh.code_points[slot.clone()], &into.code_points[slot]);
        if index < count {
            let written = fresh_slot
                .iter()
                .position(|&c| c == 0)
                .map_or(MAX_WORD_LENGTH, |end| end + 1);
            assert_eq!(into_slot[..written], fresh_slot[..written]);
            assert!(into_slot[written..].iter().all(|&c| c == STALE));
            assert_eq!(into.scores[index], fresh.scores[index]);
            assert_eq!(into.types[index], fresh.types[index]);
            assert_eq!(into.space_indices[index], fresh.space_indices[index]);
        } else {
            assert!(into_slot.iter().all(|&c| c == STALE));
            assert_eq!(
                (
                    into.scores[index],
                    into.types[index],
                    into.space_indices[index]
                ),
                (STALE, STALE, STALE)
            );
        }
    }
    let confidence = if count > 0 {
        fresh.auto_commit_confidence
    } else {
        STALE
    };
    assert_eq!(into.auto_commit_confidence, confidence);
}

#[test]
fn writes_what_suggest_with_does_and_keeps_the_rest() {
    let (dictionary, info) = rust_side();
    let previous = Query {
        previous: vec![typed("the").code_points],
        beginning_of_sentence: vec![false],
        ..Query::default()
    };
    for query in [typed("helo"), typed("qzxj"), typed("t"), previous] {
        let weight = NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL;
        let fresh = suggest_with(
            &dictionary,
            &info,
            &mut new_session(),
            &query,
            &OPTIONS,
            weight,
        );
        let mut into = stale();
        suggest_into(
            &dictionary,
            Some(&info),
            &mut new_session(),
            &query,
            &OPTIONS,
            &mut into,
        );
        assert_written_like_fresh(&fresh, &into);
    }
}

/// With no keyboard, typed input finds nothing and writes only the count; predictions need none.
#[test]
fn without_a_keyboard_only_predictions_run() {
    let (dictionary, info) = rust_side();
    let mut into = stale();
    suggest_into(
        &dictionary,
        None,
        &mut new_session(),
        &typed("helo"),
        &OPTIONS,
        &mut into,
    );
    let untouched = Output {
        count: 0,
        ..stale()
    };
    assert_eq!(into, untouched);

    let query = Query {
        previous: vec![typed("the").code_points],
        beginning_of_sentence: vec![false],
        ..Query::default()
    };
    let weight = NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL;
    let fresh = suggest_with(
        &dictionary,
        &info,
        &mut new_session(),
        &query,
        &OPTIONS,
        weight,
    );
    assert!(fresh.count > 0);
    let mut into = stale();
    suggest_into(
        &dictionary,
        None,
        &mut new_session(),
        &query,
        &OPTIONS,
        &mut into,
    );
    assert_written_like_fresh(&fresh, &into);
}
