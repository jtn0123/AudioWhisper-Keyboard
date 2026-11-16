use super::*;
use crate::tap_aim::CAP;

const SHORT: TapOffset = TapOffset { dx: -0.3, dy: 0.1 };
const PORTRAIT: Orientation = Orientation::Portrait;

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn taps() -> Taps {
    Taps::new(TapTrail::CAPACITY)
}

/// Taps every character of `word` at `offset`.
fn tap(taps: &mut Taps, word: &str, offset: TapOffset) {
    for c in word.chars() {
        taps.trail.pressed(c as i32, Some(offset));
        taps.typed(&utf16(&c.to_string()));
    }
}

/// Taps `word` and commits it as typed.
fn keep(taps: &mut Taps, word: &str) {
    tap(taps, word, SHORT);
    taps.kept(&utf16(word));
}

/// A key's weight, to two places: other keys' samples fade it a little.
fn weight(taps: &Taps, letter: char) -> f32 {
    taps.aim
        .key(PORTRAIT, letter as i32)
        .map_or(0.0, |aim| (aim.weight * 100.0).round() / 100.0)
}

#[test]
fn a_kept_word_teaches_once_typing_goes_on() {
    let mut taps = taps();
    keep(&mut taps, "pop");
    assert!(taps.aim.is_empty());
    tap(&mut taps, "a", SHORT);
    assert!(taps.aim.is_empty(), "one character is not enough");
    tap(&mut taps, "n", SHORT);
    assert_eq!((weight(&taps, 'p'), weight(&taps, 'o')), (2.0, 1.0));
    // The next word's taps teach nothing until it is kept and confirmed too.
    assert_eq!(weight(&taps, 'a'), 0.0);
    assert!(taps.take_changed());
    assert!(!taps.take_changed());
}

#[test]
fn the_next_word_kept_confirms_the_one_before() {
    let mut taps = taps();
    keep(&mut taps, "pop");
    keep(&mut taps, "a");
    assert_eq!(weight(&taps, 'p'), 2.0);
    assert_eq!(weight(&taps, 'a'), 0.0);
}

#[test]
fn a_backspace_or_an_undo_throws_the_word_away() {
    let mut taps = taps();
    keep(&mut taps, "pop");
    taps.backspace();
    tap(&mut taps, "an", SHORT);
    assert!(taps.aim.is_empty());
    keep(&mut taps, "pop");
    taps.rejected();
    tap(&mut taps, "an", SHORT);
    assert!(taps.aim.is_empty());
    keep(&mut taps, "pop");
    taps.clear();
    tap(&mut taps, "an", SHORT);
    assert!(taps.aim.is_empty());
    assert!(!taps.take_changed());
}

#[test]
fn a_word_fixed_with_a_backspace_teaches_nothing() {
    let mut taps = taps();
    tap(&mut taps, "pi", SHORT);
    taps.deleted();
    tap(&mut taps, "op", SHORT);
    taps.kept(&utf16("pop"));
    tap(&mut taps, "an", SHORT);
    assert!(taps.aim.is_empty());
    // Retyped whole after the backspace, it does.
    tap(&mut taps, "x", SHORT);
    taps.deleted();
    keep(&mut taps, "pop");
    tap(&mut taps, "an", SHORT);
    assert_eq!(weight(&taps, 'p'), 2.0);
}

#[test]
fn a_word_the_trail_does_not_end_with_teaches_nothing() {
    let mut taps = taps();
    tap(&mut taps, "pop", SHORT);
    taps.kept(&utf16("hello"));
    taps.kept(&utf16(""));
    tap(&mut taps, "an", SHORT);
    assert!(taps.aim.is_empty());
}

#[test]
fn only_tapped_characters_teach() {
    let mut taps = taps();
    taps.trail.pressed('p' as i32, Some(SHORT));
    taps.typed(&utf16("p"));
    taps.typed(&utf16("o")); // a long-press popup: no position
    taps.trail.pressed('p' as i32, Some(SHORT));
    taps.typed(&utf16("P"));
    taps.kept(&utf16("poP"));
    tap(&mut taps, "an", SHORT);
    assert_eq!((weight(&taps, 'p'), weight(&taps, 'o')), (2.0, 0.0));
    // A word with no tapped character waits for nothing.
    taps.typed(&utf16("ok"));
    taps.kept(&utf16("ok"));
    tap(&mut taps, "an", SHORT);
    assert_eq!(weight(&taps, 'o'), 0.0);
}

#[test]
fn a_prediction_keeps_the_word_and_a_suggestion_for_it_replaces_it() {
    let mut taps = taps();
    keep(&mut taps, "pop");
    taps.picked(false);
    assert_eq!(weight(&taps, 'p'), 2.0);
    // The cursor went back into the word and a suggestion replaced it.
    keep(&mut taps, "pop");
    taps.picked(true);
    tap(&mut taps, "an", SHORT);
    assert_eq!(weight(&taps, 'p'), 2.0);
    // A suggestion tapped for the next word, one character in.
    keep(&mut taps, "pop");
    tap(&mut taps, "a", SHORT);
    taps.picked(true);
    assert_eq!(weight(&taps, 'p'), 4.0);
}

#[test]
fn a_swipe_after_the_word_keeps_it() {
    let mut taps = taps();
    keep(&mut taps, "pop");
    taps.swiped();
    assert_eq!(weight(&taps, 'p'), 2.0);
}

#[test]
fn a_word_is_learned_in_the_orientation_it_was_typed_in() {
    let mut taps = taps();
    taps.set_orientation(Orientation::Landscape);
    keep(&mut taps, "pop");
    taps.set_orientation(PORTRAIT);
    tap(&mut taps, "an", SHORT);
    assert_eq!(weight(&taps, 'p'), 0.0);
    let landscape = taps.aim.key(Orientation::Landscape, 'p' as i32);
    assert_eq!(landscape.map(|aim| aim.weight.round()), Some(2.0));
    assert_eq!(taps.orientation(), PORTRAIT);
}

#[test]
fn the_decoder_hears_aimed_taps_only_when_asked() {
    let mut taps = taps();
    while taps.aim.bias(PORTRAIT, 'p' as i32).is_none() {
        keep(&mut taps, "pop");
        taps.picked(false);
    }
    let word = utf16("pop");
    tap(&mut taps, "pop", SHORT);
    assert_eq!(taps.offsets_for(&word, false), [Some(SHORT); 3]);
    let aimed = taps.offsets_for(&word, true);
    // P's taps all land 0.3 short: the aim takes back all but what is over the cap.
    let p = aimed[0].unwrap();
    assert!(
        (p.dx - (SHORT.dx + CAP)).abs() < 1e-5 && p.dy.abs() < 1e-5,
        "{p:?}"
    );
    // O has fewer samples than it needs.
    assert_eq!(aimed[1], Some(SHORT));
    assert!(taps.offsets_for(&utf16("xyz"), true).is_empty());
}

#[test]
fn retargets_in_the_current_orientation() {
    let mut taps = taps();
    for _ in 0..20 {
        keep(&mut taps, "op");
        taps.picked(false);
    }
    let keys = ['o', 'p'].map(|c| AimKey {
        code: c as i32,
        width: 10.0,
        height: 10.0,
    });
    let key_at = |x: f32, _: f32| Some(usize::from(x >= 10.0));
    let none = |_| None;
    assert_eq!(
        taps.key_for(&keys, (0, (8.0, 5.0)), true, &[], none, key_at),
        1
    );
    assert_eq!(
        taps.key_for(&keys, (0, (8.0, 5.0)), false, &[], none, key_at),
        0,
        "the aim isn't used"
    );
    taps.set_orientation(Orientation::Landscape);
    assert_eq!(
        taps.key_for(&keys, (0, (8.0, 5.0)), true, &[], none, key_at),
        0
    );
}

#[test]
fn the_letter_odds_move_a_tap_the_aim_leaves() {
    let taps = taps();
    let keys = ['o', 'p'].map(|c| AimKey {
        code: c as i32,
        width: 10.0,
        height: 10.0,
    });
    let key_at = |x: f32, _: f32| Some(usize::from(x >= 10.0));
    let bounds = |i: usize| {
        let left = i as f32 * 10.0;
        Some(keys::Rect {
            left,
            top: 0.0,
            right: left + 10.0,
            bottom: 10.0,
        })
    };
    // P far likelier than O: a tap 0.1 key inside O types P; one 0.3 inside stays an O.
    let priors = [Some(0.01f32.ln()), Some(0.9f32.ln())];
    for aimed in [false, true] {
        let key_for = |x| taps.key_for(&keys, (0, (x, 5.0)), aimed, &priors, bounds, key_at);
        assert_eq!((key_for(9.0), key_for(7.0)), (1, 0));
    }
    assert_eq!(
        taps.key_for(&keys, (0, (9.0, 5.0)), false, &[], bounds, key_at),
        0
    );
}

#[test]
fn forgets_and_loads_the_aim() {
    let mut taps = taps();
    keep(&mut taps, "pop");
    taps.picked(false);
    let saved = taps.aim.text();
    keep(&mut taps, "pop");
    taps.forget();
    assert!(taps.aim.is_empty());
    assert!(!taps.take_changed());
    tap(&mut taps, "an", SHORT);
    assert!(taps.aim.is_empty(), "the word waiting is forgotten too");
    taps.load(&saved);
    assert_eq!(weight(&taps, 'p'), 2.0);
    assert!(!taps.take_changed());
}
