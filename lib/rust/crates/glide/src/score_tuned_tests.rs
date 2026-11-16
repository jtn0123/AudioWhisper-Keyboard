//! [`Tuning`]'s additions as the scorer applies them: the original adds nothing, and each
//! addition moves the words it should, the way it should.

use super::tests::{qwerty, swipe_through};
use super::*;
use crate::index::tests::{ascii_table, index};

/// Each of `words`' weight for `swipe` (drawn at `times`) under `tuning`, at `frequencies`;
/// infinite for a word not ranked.
fn weights(
    words: &[&str],
    frequencies: &[f32],
    swipe: &Gesture,
    times: &[f32],
    tuning: &Tuning,
) -> Vec<f32> {
    let keys = qwerty();
    let index = index(words, &ascii_table(&[]));
    let layout = Layout::new(&keys, index.codes());
    let candidates = index.candidates(swipe, &layout);
    let candidate_frequencies: Vec<f32> = candidates
        .iter()
        .map(|&w| frequencies[w as usize])
        .collect();
    let swipe = Swipe {
        gesture: swipe,
        times,
    };
    let ranking = index
        .score_with(
            &candidates,
            &candidate_frequencies,
            swipe,
            &layout,
            8,
            tuning,
        )
        .unwrap();
    (0..words.len() as u32)
        .map(|w| {
            let at = ranking.words().iter().position(|&r| r == w);
            at.map_or(f32::INFINITY, |at| ranking.weights()[at])
        })
        .collect()
}

/// One of the additions alone, at the strength the keyboard uses.
fn only(addition: fn(&mut Tuning, &Tuning)) -> Tuning {
    let mut tuning = Tuning::ORIGINAL;
    addition(&mut tuning, &Tuning::DEFAULT);
    tuning
}

/// A clock for `swipe`: a point takes `slow` ms after the last within 60 px of (`x`, `y`), else
/// 1 ms.
fn clock(swipe: &Gesture, (x, y): (f32, f32), slow: f32) -> Vec<f32> {
    let mut t = 0.0;
    (0..swipe.len())
        .map(|i| {
            if i > 0 {
                t += if distance(swipe.x(i), swipe.y(i), x, y) < 60.0 {
                    slow
                } else {
                    1.0
                };
            }
            t
        })
        .collect()
}

#[test]
fn the_original_tuning_is_the_port_whatever_the_times() {
    let words = [
        "hello", "help", "hell", "jello", "world", "word", "would", "hold", "pool", "pol",
    ];
    let keys = qwerty();
    let index = index(&words, &ascii_table(&[]));
    let layout = Layout::new(&keys, index.codes());
    for swiped in ["hello", "word", "pool", "hold"] {
        let swipe = swipe_through(swiped, &keys);
        let candidates = index.candidates(&swipe, &layout);
        let frequencies: Vec<f32> = (0..candidates.len())
            .map(|i| 1.0 / (i + 1) as f32)
            .collect();
        let times = clock(&swipe, (450.0, 75.0), 9.0);
        let swipe_at = Swipe {
            gesture: &swipe,
            times: &times,
        };
        let port = index.score(&candidates, &frequencies, &swipe, &layout, 8);
        let original = index.score_with(
            &candidates,
            &frequencies,
            swipe_at,
            &layout,
            8,
            &Tuning::ORIGINAL,
        );
        assert_eq!(port, original, "{swiped}");
    }
}

#[test]
fn the_keyboard_uses_every_addition() {
    let tuning = Tuning::default();
    assert!(tuning.ends > 0.0 && tuning.dwell > 0.0 && tuning.reach > 0.0);
    assert!(!tuning.is_original());
}

#[test]
fn ends_favour_the_word_whose_end_key_the_swipe_stops_on() {
    // From "a" to 35 px past "s", towards "d": "ad" is the commoner word and the classifier's
    // choice, but the swipe stops on "s".
    let mut swipe = Gesture::new();
    for i in 0..=27 {
        swipe.push(100.0 + 5.0 * i as f32, 225.0);
    }
    let words = ["as", "ad"];
    let frequencies = [0.9, 1.0];
    let original = weights(&words, &frequencies, &swipe, &[], &Tuning::ORIGINAL);
    assert!(original[1] < original[0], "{original:?}");
    let ends = weights(
        &words,
        &frequencies,
        &swipe,
        &[],
        &only(|t, d| t.ends = d.ends),
    );
    assert!(ends[0] < ends[1], "{ends:?}");
}

#[test]
fn dwell_favours_the_word_through_the_keys_the_finger_slows_at() {
    // Through "w" and "e" on the way from "a" to "d": "awd" and "aed" are mirror images, so the
    // classifier can't tell them apart; where the finger slows can.
    let keys = qwerty();
    let swipe = swipe_through("awed", &keys);
    let words = ["awd", "aed"];
    let frequencies = [1.0, 1.0];
    let dwell = only(|t, d| t.dwell = d.dwell);
    let at_w = weights(
        &words,
        &frequencies,
        &swipe,
        &clock(&swipe, (150.0, 75.0), 8.0),
        &dwell,
    );
    assert!(at_w[0] < at_w[1], "{at_w:?}");
    let at_e = weights(
        &words,
        &frequencies,
        &swipe,
        &clock(&swipe, (250.0, 75.0), 8.0),
        &dwell,
    );
    assert!(at_e[1] < at_e[0], "{at_e:?}");
    // Without times there is nothing to go on.
    let plain = weights(&words, &frequencies, &swipe, &[], &dwell);
    let original = weights(&words, &frequencies, &swipe, &[], &Tuning::ORIGINAL);
    let (plain, original) = (plain[0] / plain[1], original[0] / original[1]);
    assert!((plain - original).abs() < 1e-4, "{plain} {original}");
}

#[test]
fn reach_holds_a_word_to_the_keys_its_swipe_goes_out_of_its_way_for() {
    let keys = qwerty();
    let words = ["ad", "aed"];
    let frequencies = [1.0, 1.0];
    let reach = only(|t, d| t.reach = d.reach);
    // A swipe up to "e" on the way counts against "ad", which has no "e"...
    let aed = swipe_through("aed", &keys);
    let original = weights(&words, &frequencies, &aed, &[], &Tuning::ORIGINAL);
    let reached = weights(&words, &frequencies, &aed, &[], &reach);
    assert!(reached[0] / reached[1] > 1.2 * original[0] / original[1]);
    // ...and one straight across counts against "aed", whose "e" it never nears.
    let ad = swipe_through("ad", &keys);
    let original = weights(&words, &frequencies, &ad, &[], &Tuning::ORIGINAL);
    let reached = weights(&words, &frequencies, &ad, &[], &reach);
    assert!(reached[1] / reached[0] > 1.2 * original[1] / original[0]);
    // A swipe that keeps to the word's path costs it nothing.
    assert_eq!(reached[0], original[0]);
}

#[test]
fn a_shorter_ranking_keeps_the_same_words_first() {
    let words = [
        "hello", "hero", "halo", "hippo", "hobo", "hullo", "hydro", "hugo", "helio", "hollo",
    ];
    let keys = qwerty();
    let index = index(&words, &ascii_table(&[]));
    let layout = Layout::new(&keys, index.codes());
    let swipe = swipe_through("hello", &keys);
    let candidates = index.candidates(&swipe, &layout);
    let frequencies = vec![0.5; candidates.len()];
    let times = clock(&swipe, (450.0, 75.0), 4.0);
    let swipe = Swipe {
        gesture: &swipe,
        times: &times,
    };
    let rank = |max| {
        let tuning = Tuning::DEFAULT;
        index
            .score_with(&candidates, &frequencies, swipe, &layout, max, &tuning)
            .unwrap()
    };
    let all = rank(words.len());
    assert!(all.words().len() > 3);
    for max in 1..=3 {
        let short = rank(max);
        assert_eq!(short.words(), &all.words()[..max]);
        assert_eq!(short.weights(), &all.weights()[..max]);
    }
}
