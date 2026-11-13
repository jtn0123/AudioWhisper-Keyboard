//! The layout port against LatinIME's C++: the app's QWERTY keyboard exhaustively, random
//! keyboards, and typed inputs from the oracle's query generator, every value to the bit.

mod robustness;
#[cfg(all(feature = "oracle", not(target_os = "android")))]
mod support;
#[cfg(all(feature = "oracle", not(target_os = "android")))]
mod typed;

#[cfg(all(feature = "oracle", not(target_os = "android")))]
use super::*;
#[cfg(all(feature = "oracle", not(target_os = "android")))]
use crate::defines::{MAX_PROXIMITY_CHARS_SIZE, MAX_WORD_LENGTH};

#[cfg(all(feature = "oracle", not(target_os = "android")))]
use latin_oracle::hooks::layout as cpp;
#[cfg(all(feature = "oracle", not(target_os = "android")))]
use latin_oracle::queries::{self, Random};
#[cfg(all(feature = "oracle", not(target_os = "android")))]
use support::{Keyboard, assert_bits, assert_states_match, qwerty, random_keyboard};

/// `en_US`, as the bundled dictionary's header spells its locale.
#[cfg(all(feature = "oracle", not(target_os = "android")))]
const EN_US: [i32; 5] = ['e' as i32, 'n' as i32, '_' as i32, 'U' as i32, 'S' as i32];

/// The locales to try: English (which adds vowels), others, and ones too short to be either.
#[cfg(all(feature = "oracle", not(target_os = "android")))]
fn locales() -> Vec<Vec<i32>> {
    let text = |it: &str| it.chars().map(|c| c as i32).collect::<Vec<i32>>();
    vec![
        EN_US.to_vec(),
        text("en"),
        text("en_GB"),
        text("de_DE"),
        text("e"),
        Vec::new(),
        text("ne"),
    ]
}

/// Every value the keyboard gives, for `points` and the keys, the same as the C++'s.
#[cfg(all(feature = "oracle", not(target_os = "android")))]
fn assert_keyboard_matches(
    keyboard: &Keyboard,
    ours: &ProximityInfo,
    theirs: cpp::Info,
    random: &mut Random,
    stride: usize,
) {
    let key_count = ours.get_key_count();
    let params = theirs.params();
    assert_eq!(
        (
            key_count,
            ours.get_cell_width(),
            ours.get_cell_height(),
            ours.get_grid_width(),
            ours.get_keyboard_height(),
            ours.get_most_common_key_width()
        ),
        (
            params.key_count,
            params.cell_width,
            params.cell_height,
            params.grid_width,
            params.keyboard_height,
            params.most_common_key_width
        ),
        "{keyboard:?}"
    );
    assert_eq!(theirs.grid(), keyboard.proximity_chars);
    for (cell, row) in keyboard
        .proximity_chars
        .chunks(MAX_PROXIMITY_CHARS_SIZE)
        .enumerate()
    {
        let start = (cell * MAX_PROXIMITY_CHARS_SIZE) as i32;
        assert_eq!(ours.proximity_chars_at(start), Some(row));
    }
    for key in -2..key_count + 2 {
        assert_eq!(
            (
                ours.get_code_point_of(key),
                ours.get_original_code_point_of(key)
            ),
            theirs.key(key),
            "key {key}"
        );
    }
    let mut codes: Vec<i32> = (-3..0x600).collect();
    codes.extend([
        0x110000,
        0x10FFFF,
        0x1F600,
        0xFFFF,
        0x10041,
        i32::MAX,
        i32::MIN,
        0x7FFF_FF41,
    ]);
    codes.extend(keyboard.key_codes.iter().copied());
    for &c in &codes {
        assert_eq!(
            ours.get_key_index_of(c),
            theirs.key_index_of(c),
            "index of {c:#x}"
        );
        assert_eq!(ours.is_code_point_on_keyboard(c), theirs.on_keyboard(c));
    }
    for key in -2..key_count {
        let mut references = vec![-1, 0, 1, keyboard_extent(keyboard), i32::MAX, i32::MIN];
        references.extend((0..6).map(|_| random.below(1500) as i32 - 100));
        for reference in references {
            assert_eq!(
                ours.get_key_center_x_of_key_id_g(key, reference),
                theirs.key_center_x(key, reference),
                "center x of key {key} at {reference}"
            );
            assert_eq!(
                ours.get_key_center_y_of_key_id_g(key, reference),
                theirs.key_center_y(key, reference),
                "center y of key {key} at {reference}"
            );
        }
    }
    let (xs, ys) = (keyboard.x_range(40), keyboard.y_range(40));
    for y in ys.clone().step_by(stride) {
        for x in xs.clone().step_by(stride) {
            assert_eq!(
                ours.has_space_proximity(x, y),
                theirs.has_space_proximity(x, y),
                "space proximity at ({x}, {y})"
            );
            for key in -1..key_count {
                assert_bits(
                    ours.get_normalized_squared_distance_from_center_float_g(key, x, y),
                    theirs.normalized_squared_distance(key, x, y),
                    &(key, x, y),
                );
            }
        }
    }
    assert_proximities_match(keyboard, ours, theirs, random, stride);
}

#[cfg(all(feature = "oracle", not(target_os = "android")))]
fn keyboard_extent(keyboard: &Keyboard) -> i32 {
    keyboard.keyboard_width
}

/// The letters near each tap, for taps over the keyboard (48 to a call, any primary code), in
/// every locale.
#[cfg(all(feature = "oracle", not(target_os = "android")))]
fn assert_proximities_match(
    keyboard: &Keyboard,
    ours: &ProximityInfo,
    theirs: cpp::Info,
    random: &mut Random,
    stride: usize,
) {
    let (xs, ys) = (keyboard.x_range(40), keyboard.y_range(40));
    let points: Vec<(i32, i32)> = ys
        .step_by(stride)
        .flat_map(|y| xs.clone().step_by(stride).map(move |x| (x, y)))
        .collect();
    let locales = locales();
    for (batch, chunk) in points.chunks(MAX_WORD_LENGTH).enumerate() {
        let codes: Vec<i32> = chunk.iter().map(|_| support::pool_code(random)).collect();
        let mut tap_xs: Vec<i32> = chunk.iter().map(|it| it.0).collect();
        let mut tap_ys: Vec<i32> = chunk.iter().map(|it| it.1).collect();
        if batch % 7 == 0 {
            tap_xs[0] = -1;
        }
        if batch % 11 == 0 {
            tap_ys[chunk.len() - 1] = -1;
        }
        let locale = &locales[batch % locales.len()];
        let mut expected = theirs.initialize_proximities(&codes, &tap_xs, &tap_ys, locale);
        let input = TypingInput {
            codes: &codes,
            xs: &tap_xs,
            ys: &tap_ys,
            times: None,
            pointer_ids: None,
        };
        let mut actual = vec![0; codes.len() * MAX_PROXIMITY_CHARS_SIZE];
        ours.initialize_proximities(&input, locale, &mut actual);
        assert_eq!(
            actual,
            std::mem::take(&mut expected),
            "{codes:?} at {tap_xs:?} {tap_ys:?}"
        );
    }
}

#[cfg(all(feature = "oracle", not(target_os = "android")))]
#[test]
fn qwerty_keyboard_matches_the_cpp() {
    let keyboard = qwerty();
    let proximity = latin_oracle::Proximity::qwerty();
    let theirs = cpp::Info::of(&proximity);
    // The arrays above are the ones the oracle gave the C++.
    assert_eq!(theirs.grid(), keyboard.proximity_chars);
    let ours = ProximityInfo::new(&keyboard.geometry()).expect("a valid keyboard");
    assert_keyboard_matches(&keyboard, &ours, theirs, &mut Random::new(1), 1);
}

#[cfg(all(feature = "oracle", not(target_os = "android")))]
#[test]
fn random_keyboards_match_the_cpp() {
    let mut random = Random::new(20_261_110);
    for round in 0..200 {
        let keyboard = random_keyboard(&mut random);
        let theirs = cpp::Keyboard::new(&keyboard.raw());
        let ours = ProximityInfo::new(&keyboard.geometry()).expect("a valid keyboard");
        assert_keyboard_matches(&keyboard, &ours, theirs.info(), &mut random, 3 + round % 5);
    }
}

#[cfg(all(feature = "oracle", not(target_os = "android")))]
#[test]
fn sweet_spot_factor_matches_the_cpp() {
    let mut values: Vec<f32> = (-300..=1500).map(|it| it as f32 / 400.0).collect();
    values.extend([
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -0.0,
        0.25,
        1.0,
        1.2,
        0.24,
        1e30,
    ]);
    let mut random = Random::new(7);
    values.extend((0..20_000).map(|_| f32::from_bits(random.next_u64() as u32)));
    for x in values {
        for enabled in [false, true] {
            assert_bits(
                touch_position_correction_utils::get_sweet_spot_factor(enabled, x),
                cpp::sweet_spot_factor(enabled, x),
                &(enabled, x),
            );
        }
    }
}
