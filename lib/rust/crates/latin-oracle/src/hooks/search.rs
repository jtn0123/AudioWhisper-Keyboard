//! The search area: `getSuggestionsNative` called with the suggest options and the in/out weight
//! given, which [`crate::suggest`] fixes at what the app passes, and a [`Proximity`] built from any
//! keyboard. The functions are ones `cpp/fake_jni.cpp` already defines, so this area needs no
//! `cpp/hooks` file.

use crate::hooks::layout::{MAX_PROXIMITY_CHARS_SIZE, RawKeyboard};
use crate::{
    Dictionary, MAX_PREVIOUS_WORDS, MAX_RESULTS, MAX_WORD_LENGTH, NOT_A_COORDINATE, Output,
    Proximity, Query, Session, lock, search_lock,
};

// SAFETY: cpp/fake_jni.cpp defines this with the same C signature (the same declaration as
// `crate`'s); each pointer is to an array the caller holds, valid for the length passed with it.
unsafe extern "C" {
    fn aw_oracle_suggest(
        dictionary: i64,
        proximity: i64,
        session: i64,
        xs: *const i32,
        ys: *const i32,
        times: *const i32,
        pointer_ids: *const i32,
        codes: *const i32,
        input_capacity: i32,
        input_size: i32,
        options: *const i32,
        option_count: i32,
        previous_flat: *const i32,
        previous_lengths: *const i32,
        beginning_of_sentence: *const u8,
        previous_slots: i32,
        previous_count: i32,
        out_count: *mut i32,
        out_code_points: *mut i32,
        code_point_capacity: i32,
        out_scores: *mut i32,
        out_space_indices: *mut i32,
        out_types: *mut i32,
        result_capacity: i32,
        out_auto_commit: *mut i32,
        in_out_weight: *mut f32,
    );
    fn aw_oracle_proximity(
        width: i32,
        height: i32,
        grid_width: i32,
        grid_height: i32,
        key_width: i32,
        key_height: i32,
        grid: *const i32,
        grid_length: i32,
        key_count: i32,
        xs: *const i32,
        ys: *const i32,
        widths: *const i32,
        heights: *const i32,
        codes: *const i32,
    ) -> i64;
}

/// The keyboard `setProximityInfoNative` makes from `raw`, as a [`Proximity`] the search can be
/// given, as [`Proximity::qwerty`] makes the app's (which has no space key). `raw` must be valid:
/// the grid array `grid_width * grid_height * 16` long, each key array `key_count` long.
pub fn proximity(raw: &RawKeyboard<'_>) -> Proximity {
    let grid_cells = i64::from(raw.grid_width) * i64::from(raw.grid_height);
    assert_eq!(
        raw.proximity_chars.len() as i64,
        grid_cells * MAX_PROXIMITY_CHARS_SIZE as i64
    );
    for array in [
        raw.key_xs,
        raw.key_ys,
        raw.key_widths,
        raw.key_heights,
        raw.key_codes,
    ] {
        assert_eq!(array.len() as i64, i64::from(raw.key_count));
    }
    let _lock = lock();
    // SAFETY: every array is valid for the length given and only read during the call.
    let handle = unsafe {
        aw_oracle_proximity(
            raw.keyboard_width,
            raw.keyboard_height,
            raw.grid_width,
            raw.grid_height,
            raw.most_common_key_width,
            raw.most_common_key_height,
            raw.proximity_chars.as_ptr(),
            i32::try_from(raw.proximity_chars.len()).expect("a small grid"),
            raw.key_count,
            raw.key_xs.as_ptr(),
            raw.key_ys.as_ptr(),
            raw.key_widths.as_ptr(),
            raw.key_heights.as_ptr(),
            raw.key_codes.as_ptr(),
        )
    };
    // `Proximity`'s drop releases it with `aw_oracle_proximity_release`.
    Proximity { handle }
}

/// `BinaryDictionary.getSuggestions` with `options` as the suggest options array and `weight` as
/// the in/out weight of the language model against the spatial model, going in. The query's own
/// `block_offensive_words` is ignored: `options` says it.
pub fn suggest_with(
    dictionary: &Dictionary,
    proximity: &Proximity,
    session: &Session,
    query: &Query,
    options: &[i32],
    weight: f32,
) -> Output {
    let size = query.code_points.len();
    assert!(
        size <= MAX_WORD_LENGTH,
        "the app never passes a longer word"
    );
    assert_eq!(query.points.len(), size, "one point per code point");
    assert!(
        query.previous.len() <= MAX_PREVIOUS_WORDS,
        "at most three previous words"
    );
    assert_eq!(query.previous.len(), query.beginning_of_sentence.len());
    let mut codes = [0; MAX_WORD_LENGTH];
    let mut xs = [NOT_A_COORDINATE; MAX_WORD_LENGTH];
    let mut ys = [NOT_A_COORDINATE; MAX_WORD_LENGTH];
    codes[..size].copy_from_slice(&query.code_points);
    for (index, point) in query.points.iter().enumerate() {
        if let Some((x, y)) = point {
            (xs[index], ys[index]) = (*x, *y);
        }
    }
    let zeros = [0; MAX_WORD_LENGTH];
    let flat: Vec<i32> = query.previous.concat();
    let lengths: Vec<i32> = (0..MAX_PREVIOUS_WORDS)
        .map(|slot| {
            query
                .previous
                .get(slot)
                .map_or(-1, |it| i32::try_from(it.len()).expect("short word"))
        })
        .collect();
    let mut beginning = [0u8; MAX_PREVIOUS_WORDS];
    for (slot, &flag) in query.beginning_of_sentence.iter().enumerate() {
        beginning[slot] = u8::from(flag);
    }
    let mut output = Output {
        count: 0,
        code_points: vec![0; MAX_WORD_LENGTH * MAX_RESULTS],
        scores: vec![0; MAX_RESULTS],
        space_indices: vec![0; MAX_RESULTS],
        types: vec![0; MAX_RESULTS],
        auto_commit_confidence: 0,
        weight,
    };
    let _lock = search_lock(session);
    // SAFETY: every array is valid for the length passed with it, the outputs are writable for
    // theirs, and the handles are open until their owners drop.
    unsafe {
        aw_oracle_suggest(
            dictionary.address(),
            proximity.address(),
            session.address(),
            xs.as_ptr(),
            ys.as_ptr(),
            zeros.as_ptr(),
            zeros.as_ptr(),
            codes.as_ptr(),
            MAX_WORD_LENGTH as i32,
            size as i32,
            options.as_ptr(),
            i32::try_from(options.len()).expect("few options"),
            flat.as_ptr(),
            lengths.as_ptr(),
            beginning.as_ptr(),
            MAX_PREVIOUS_WORDS as i32,
            i32::try_from(query.previous.len()).expect("three at most"),
            &raw mut output.count,
            output.code_points.as_mut_ptr(),
            i32::try_from(output.code_points.len()).expect("864"),
            output.scores.as_mut_ptr(),
            output.space_indices.as_mut_ptr(),
            output.types.as_mut_ptr(),
            MAX_RESULTS as i32,
            &raw mut output.auto_commit_confidence,
            &raw mut output.weight,
        );
    }
    output
}
