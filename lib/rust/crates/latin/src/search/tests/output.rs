//! `JniDataUtils::outputCodePoints`, which no dictionary word reaches with a control code or the
//! beginning-of-sentence marker, checked against what its C++ does by reading it.

use crate::defines::{CODE_POINT_BEGINNING_OF_SENTENCE, MAX_WORD_LENGTH};
use crate::search::jni::output_code_points;

const REPLACEMENT: i32 = 0xFFFD;

fn output(code_points: &[i32]) -> [i32; MAX_WORD_LENGTH] {
    let mut out = [7; MAX_WORD_LENGTH];
    output_code_points(&mut out, code_points);
    out
}

#[test]
fn plain_words_are_copied_and_terminated() {
    let out = output(&[104, 105]);
    assert_eq!(&out[..3], &[104, 105, 0]);
    assert_eq!(out[3], 7, "nothing is written past the terminator");
}

#[test]
fn control_codes_become_the_replacement_character() {
    for control in [0x01, 0x09, 0x1E, 0x1F] {
        assert_eq!(&output(&[97, control, 98])[..4], &[97, REPLACEMENT, 98, 0]);
    }
    // 0 and 0x20 are not control codes here.
    assert_eq!(&output(&[0x20, 97])[..3], &[0x20, 97, 0]);
    assert_eq!(&output(&[0, 97])[..3], &[0, 97, 0]);
}

#[test]
fn non_unicode_code_points_become_the_replacement_character() {
    for bad in [-1, i32::MIN, 0x11_0001, i32::MAX, 0xD800] {
        let out = output(&[97, bad]);
        let expect = if bad == 0xD800 { bad } else { REPLACEMENT };
        assert_eq!(&out[..3], &[97, expect, 0], "{bad:#x}");
    }
}

#[test]
fn the_beginning_of_sentence_marker_is_skipped() {
    let out = output(&[
        CODE_POINT_BEGINNING_OF_SENTENCE,
        97,
        CODE_POINT_BEGINNING_OF_SENTENCE,
    ]);
    assert_eq!(&out[..2], &[97, 0]);
}

#[test]
fn a_full_word_has_no_terminator_and_a_longer_one_is_cut() {
    let word: Vec<i32> = (0..MAX_WORD_LENGTH as i32 + 5)
        .map(|it| 97 + it % 26)
        .collect();
    let out = output(&word);
    assert_eq!(out[..], word[..MAX_WORD_LENGTH]);
}
