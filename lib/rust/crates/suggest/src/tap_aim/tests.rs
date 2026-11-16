use super::*;

const P: i32 = 'p' as i32;
const O: i32 = 'o' as i32;
const PORTRAIT: Orientation = Orientation::Portrait;
const LANDSCAPE: Orientation = Orientation::Landscape;

fn at(dx: f32, dy: f32) -> TapOffset {
    TapOffset { dx, dy }
}

fn learn(aim: &mut TapAim, orientation: Orientation, code: i32, offset: TapOffset, times: usize) {
    for _ in 0..times {
        aim.learn(orientation, &[(code, offset)]);
    }
}

/// Taps alternating either side of `offset`, `spread` away, so their spread is `spread`.
fn learn_spread(aim: &mut TapAim, code: i32, offset: TapOffset, spread: f32, times: usize) {
    for i in 0..times {
        let side = if i % 2 == 0 { spread } else { -spread };
        aim.learn(PORTRAIT, &[(code, at(offset.dx + side, offset.dy - side))]);
    }
}

#[test]
fn a_key_applies_only_once_it_has_enough_samples() {
    let mut aim = TapAim::default();
    learn(&mut aim, PORTRAIT, P, at(-0.2, 0.1), 15);
    assert_eq!(aim.key(PORTRAIT, P).unwrap().weight, 15.0);
    assert_eq!(aim.bias(PORTRAIT, P), None);
    assert_eq!(aim.aimed(PORTRAIT, P, at(-0.3, 0.0)), at(-0.3, 0.0));
    learn(&mut aim, PORTRAIT, P, at(-0.2, 0.1), 1);
    let bias = aim.bias(PORTRAIT, P).unwrap();
    assert!(
        (bias.dx + 0.2).abs() < 1e-5 && (bias.dy - 0.1).abs() < 1e-5,
        "{bias:?}"
    );
}

#[test]
fn a_bias_moves_a_tap_at_most_the_cap() {
    let mut aim = TapAim::default();
    learn(&mut aim, PORTRAIT, P, at(-0.6, 0.6), 40);
    assert_eq!(aim.bias(PORTRAIT, P), Some(at(-CAP, CAP)));
    let aimed = aim.aimed(PORTRAIT, P, at(-0.5, 0.5));
    assert!((aimed.dx + 0.25).abs() < 1e-6 && (aimed.dy - 0.25).abs() < 1e-6);
    // The decoder never hears a tap further out than a tap can be.
    assert_eq!(aim.aimed(PORTRAIT, P, at(0.5, -0.5)), at(0.6, -0.6));
}

#[test]
fn a_sample_is_held_near_its_key() {
    let mut aim = TapAim::default();
    learn(&mut aim, PORTRAIT, P, at(-3.0, 9.0), 1);
    let key = aim.key(PORTRAIT, P).unwrap();
    assert_eq!((key.dx, key.dy), (-SAMPLE_LIMIT, SAMPLE_LIMIT));
}

#[test]
fn scatter_around_the_centre_is_not_a_bias() {
    let mut aim = TapAim::default();
    // A neutral typist's taps: wide, centred. An odd count leaves the mean a little off centre.
    learn_spread(&mut aim, P, at(0.0, 0.0), 0.3, 33);
    let key = *aim.key(PORTRAIT, P).unwrap();
    assert!(key.dx.abs() > 0.0, "{key:?}");
    assert_eq!(aim.bias(PORTRAIT, P), Some(at(0.0, 0.0)));
    assert_eq!(aim.aimed(PORTRAIT, P, at(0.31, 0.2)), at(0.31, 0.2));
}

#[test]
fn a_bias_too_small_to_matter_is_not_applied() {
    let mut aim = TapAim::default();
    // Every tap exactly 0.02 left: no noise to take off, but under MIN_BIAS.
    learn(&mut aim, PORTRAIT, P, at(-0.02, MIN_BIAS + 0.01), 50);
    let bias = aim.bias(PORTRAIT, P).unwrap();
    assert_eq!(bias.dx, 0.0);
    assert!((bias.dy - (MIN_BIAS + 0.01)).abs() < 1e-5, "{bias:?}");
    assert_eq!(aim.aimed(PORTRAIT, P, at(0.3, 0.0)).dx, 0.3);
}

#[test]
fn only_what_is_beyond_the_noise_applies() {
    let mut aim = TapAim::default();
    learn_spread(&mut aim, P, at(-0.2, 0.0), 0.3, 100);
    let key = *aim.key(PORTRAIT, P).unwrap();
    let bias = aim.bias(PORTRAIT, P).unwrap();
    let error = (key.var_x / key.weight).sqrt();
    assert!((key.var_x - 0.09).abs() < 1e-3, "{key:?}");
    assert!(
        (bias.dx - (key.dx + error)).abs() < 1e-5,
        "{bias:?} {key:?}"
    );
    assert!(bias.dx < -0.15 && bias.dx > key.dx);
}

#[test]
fn a_new_grip_replaces_the_old_one() {
    let mut aim = TapAim::default();
    learn(&mut aim, PORTRAIT, P, at(-0.2, 0.0), 1000);
    assert_eq!(aim.key(PORTRAIT, P).unwrap().weight, WINDOW);
    learn(&mut aim, PORTRAIT, P, at(0.2, 0.0), 1000);
    let bias = aim.bias(PORTRAIT, P).unwrap();
    assert!(bias.dx > 0.15, "{bias:?}");
}

#[test]
fn a_key_left_alone_fades_out() {
    let mut aim = TapAim::default();
    learn(&mut aim, PORTRAIT, P, at(-0.2, 0.0), 20);
    learn(&mut aim, PORTRAIT, O, at(0.0, 0.0), 4000);
    assert!(aim.bias(PORTRAIT, P).is_some());
    learn(&mut aim, PORTRAIT, O, at(0.0, 0.0), 1000);
    let weight = aim.key(PORTRAIT, P).unwrap().weight;
    assert!(weight < MIN_SAMPLES, "{weight}");
    assert_eq!(aim.bias(PORTRAIT, P), None);
}

#[test]
fn portrait_and_landscape_are_learned_apart() {
    let mut aim = TapAim::default();
    learn(&mut aim, LANDSCAPE, P, at(0.2, 0.0), 20);
    assert_eq!(aim.bias(PORTRAIT, P), None);
    assert!(aim.bias(LANDSCAPE, P).is_some());
    learn(&mut aim, PORTRAIT, P, at(-0.2, 0.0), 20);
    assert!(aim.bias(PORTRAIT, P).unwrap().dx < 0.0);
    assert!(aim.bias(LANDSCAPE, P).unwrap().dx > 0.0);
}

#[test]
fn letters_are_learned_in_either_case_and_nothing_else_is() {
    let mut aim = TapAim::default();
    learn(&mut aim, PORTRAIT, 'P' as i32, at(-0.2, 0.0), 10);
    learn(&mut aim, PORTRAIT, P, at(-0.2, 0.0), 10);
    assert_eq!(aim.key(PORTRAIT, 'P' as i32).unwrap().weight, 20.0);
    for code in ['1' as i32, ' ' as i32, '\'' as i32, -5, 0x1F600] {
        learn(&mut aim, PORTRAIT, code, at(0.1, 0.1), 20);
        assert_eq!(aim.key(PORTRAIT, code), None);
    }
    learn(&mut aim, PORTRAIT, O, at(f32::NAN, 0.0), 20);
    learn(&mut aim, PORTRAIT, O, at(0.0, f32::INFINITY), 20);
    assert_eq!(aim.key(PORTRAIT, O), None);
    assert_eq!(aim.aimed(PORTRAIT, '1' as i32, at(0.4, 0.0)), at(0.4, 0.0));
}

#[test]
fn holds_a_bounded_number_of_keys() {
    let mut aim = TapAim::default();
    // Cyrillic letters are letters too: 26 Latin and 48 Cyrillic.
    let letters: Vec<i32> = ('a' as i32..='z' as i32).chain(0x430..0x460).collect();
    assert_eq!(letters.len(), MAX_KEYS + 10);
    for &code in &letters {
        learn(&mut aim, PORTRAIT, code, at(0.1, 0.0), 1);
    }
    assert!(aim.key(PORTRAIT, letters[MAX_KEYS - 1]).is_some());
    assert_eq!(aim.key(PORTRAIT, letters[MAX_KEYS]), None);
    learn(&mut aim, PORTRAIT, letters[0], at(0.1, 0.0), 1);
    assert_eq!(aim.key(PORTRAIT, letters[0]).unwrap().weight.round(), 2.0);
}

#[test]
fn clears_every_key() {
    let mut aim = TapAim::default();
    assert!(aim.is_empty());
    learn(&mut aim, PORTRAIT, P, at(-0.2, 0.0), 20);
    learn(&mut aim, LANDSCAPE, O, at(-0.2, 0.0), 20);
    assert!(!aim.is_empty());
    aim.clear();
    assert!(aim.is_empty());
    assert_eq!(aim.bias(PORTRAIT, P), None);
}

#[test]
fn saves_numbers_only_and_reads_them_back() {
    let mut aim = TapAim::default();
    learn_spread(&mut aim, P, at(-0.2, 0.1), 0.1, 30);
    learn(&mut aim, LANDSCAPE, 'k' as i32, at(0.05, -0.1), 17);
    let text = aim.text();
    let saved = String::from_utf16(&text).unwrap();
    let mut lines = saved.lines();
    assert_eq!(lines.next(), Some(HEADER));
    for line in lines {
        let fields: Vec<&str> = line.split(' ').collect();
        assert_eq!(fields.len(), 7, "{line}");
        assert!(
            fields.iter().all(|field| field.parse::<f64>().is_ok()),
            "{line}"
        );
    }
    assert_eq!(TapAim::parse(&text), aim);
}

#[test]
fn reading_skips_what_is_not_a_key() {
    let lines = [
        HEADER,
        "0 112 20 -0.2 0.1 0.01 0.01",
        "0 112 20 -0.2 0.1 0.01",        // too short
        "0 112 20 -0.2 0.1 0.01 0.01 4", // too long
        "2 111 20 -0.2 0.1 0.01 0.01",   // no such orientation
        "0 80 20 -0.2 0.1 0.01 0.01",    // upper case
        "0 49 20 -0.2 0.1 0.01 0.01",    // not a letter
        "0 111 0 -0.2 0.1 0.01 0.01",    // no weight
        "0 111 999 -0.2 0.1 0.01 0.01",  // over the window
        "0 111 20 -0.9 0.1 0.01 0.01",   // too far out
        "0 111 20 -0.2 0.1 -1 0.01",     // a negative spread
        "0 111 20 NaN 0.1 0.01 0.01",    // not a number
        "1 107 16 0.1 0.0 0.0 0.0",
        "",
    ];
    let text: Vec<u16> = lines.join("\n").encode_utf16().collect();
    let aim = TapAim::parse(&text);
    assert_eq!(aim.key(PORTRAIT, P).unwrap().weight, 20.0);
    assert_eq!(aim.key(PORTRAIT, O), None);
    assert_eq!(aim.key(LANDSCAPE, 'k' as i32).unwrap().dx, 0.1);
    let other: Vec<u16> = "tap-aim 2\n0 112 20 -0.2 0.1 0.01 0.01\n"
        .encode_utf16()
        .collect();
    assert!(TapAim::parse(&other).is_empty());
    assert!(TapAim::parse(&[]).is_empty());
}

/// A row of three keys, O P and a key that is no letter, ten wide and fifteen high.
fn row() -> Vec<AimKey> {
    [O, P, '1' as i32]
        .map(|code| AimKey {
            code,
            width: 10.0,
            height: 15.0,
        })
        .to_vec()
}

fn key_at(x: f32, y: f32) -> Option<usize> {
    let column = (x / 10.0).floor();
    ((0.0..3.0).contains(&column) && (0.0..15.0).contains(&y)).then_some(column as usize)
}

/// An aim for this user's thumb, which falls `short` keys left of every key.
fn falls_short(short: f32) -> TapAim {
    let mut aim = TapAim::default();
    learn(&mut aim, PORTRAIT, O, at(-short, 0.0), 40);
    learn(&mut aim, PORTRAIT, P, at(-short, 0.0), 40);
    aim
}

#[test]
fn a_tap_on_the_border_goes_where_this_user_aims() {
    let aim = falls_short(0.2);
    let keys = row();
    // The right edge of O, where this user's taps for P land.
    assert_eq!(aim.retarget(PORTRAIT, &keys, 0, (9.0, 7.0), key_at), 1);
    // The middle of O stays an O, and so does its left edge.
    assert_eq!(aim.retarget(PORTRAIT, &keys, 0, (5.0, 7.0), key_at), 0);
    assert_eq!(aim.retarget(PORTRAIT, &keys, 0, (1.0, 7.0), key_at), 0);
    // A tap on P never moves back to O: this thumb falls short, it doesn't overshoot.
    assert_eq!(aim.retarget(PORTRAIT, &keys, 1, (10.5, 7.0), key_at), 1);
    // The other orientation has learned nothing.
    assert_eq!(aim.retarget(LANDSCAPE, &keys, 0, (9.0, 7.0), key_at), 0);
}

#[test]
fn a_tap_moves_only_when_both_keys_agree() {
    let keys = row();
    // P's taps land on its centre: nothing says a tap on O meant P.
    let mut aim = TapAim::default();
    learn(&mut aim, PORTRAIT, O, at(-0.2, 0.0), 40);
    learn(&mut aim, PORTRAIT, P, at(0.0, 0.0), 40);
    assert_eq!(aim.retarget(PORTRAIT, &keys, 0, (9.0, 7.0), key_at), 0);
    // P has too few samples to say.
    let mut aim = TapAim::default();
    learn(&mut aim, PORTRAIT, O, at(-0.2, 0.0), 40);
    learn(&mut aim, PORTRAIT, P, at(-0.2, 0.0), 5);
    assert_eq!(aim.retarget(PORTRAIT, &keys, 0, (9.0, 7.0), key_at), 0);
    // The neighbour is not a letter.
    let aim = falls_short(0.2);
    assert_eq!(aim.retarget(PORTRAIT, &keys, 1, (19.5, 7.0), key_at), 1);
}

#[test]
fn an_unbiased_typist_is_never_moved() {
    let keys = row();
    let mut aim = TapAim::default();
    learn_spread(&mut aim, O, at(0.0, 0.0), 0.25, 101);
    learn_spread(&mut aim, P, at(0.0, 0.0), 0.25, 101);
    for x in [0.5, 4.0, 9.0, 9.99, 10.0, 10.5, 15.0, 19.9] {
        let hit = key_at(x, 7.0).unwrap();
        assert_eq!(
            aim.retarget(PORTRAIT, &keys, hit, (x, 7.0), key_at),
            hit,
            "{x}"
        );
    }
    assert_eq!(
        TapAim::default().retarget(PORTRAIT, &keys, 0, (9.9, 7.0), key_at),
        0
    );
    // A hit outside the keys given is left alone.
    assert_eq!(aim.retarget(PORTRAIT, &keys, 7, (9.9, 7.0), key_at), 7);
}
