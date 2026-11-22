//! The layout port against LatinIME's answers (the goldens): the app's QWERTY keyboard
//! exhaustively, random keyboards, and typed inputs from `latin_golden`'s query generator, every
//! value to the bit.

mod robustness;
mod support;
mod typed;

use super::*;
use crate::defines::{MAX_PROXIMITY_CHARS_SIZE, MAX_WORD_LENGTH};
use crate::golden::{Tape, record, tape};

use latin_golden::queries::{self, Random};
use support::{Board, Keyboard, random_keyboard, record_state};

/// `en_US`, as the bundled dictionary's header spells its locale.
const EN_US: [i32; 5] = ['e' as i32, 'n' as i32, '_' as i32, 'U' as i32, 'S' as i32];

/// The locales to try: English (which adds vowels), others, and ones too short to be either.
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

/// Every value the keyboard gives, for `points` and the keys, recorded on `tape`.
fn record_keyboard(tape: &mut Tape, board: &Board, random: &mut Random, stride: usize) {
    let (keyboard, ours) = (&board.keyboard, &board.ours);
    let key_count = ours.get_key_count();
    record!(
        tape,
        (
            key_count,
            ours.get_cell_width(),
            ours.get_cell_height(),
            ours.get_grid_width(),
            ours.get_keyboard_height(),
            ours.get_most_common_key_width()
        )
    );
    for (cell, row) in keyboard
        .proximity_chars
        .chunks(MAX_PROXIMITY_CHARS_SIZE)
        .enumerate()
    {
        let start = (cell * MAX_PROXIMITY_CHARS_SIZE) as i32;
        assert_eq!(ours.proximity_chars_at(start), Some(row));
    }
    for key in -2..key_count + 2 {
        record!(
            tape,
            (
                ours.get_code_point_of(key),
                ours.get_original_code_point_of(key)
            )
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
        record!(
            tape,
            (ours.get_key_index_of(c), ours.is_code_point_on_keyboard(c))
        );
    }
    for key in -2..key_count {
        let mut references = vec![-1, 0, 1, keyboard.keyboard_width, i32::MAX, i32::MIN];
        references.extend((0..6).map(|_| random.below(1500) as i32 - 100));
        for reference in references {
            record!(
                tape,
                (
                    ours.get_key_center_x_of_key_id_g(key, reference),
                    ours.get_key_center_y_of_key_id_g(key, reference)
                )
            );
        }
    }
    let (xs, ys) = (keyboard.x_range(40), keyboard.y_range(40));
    for y in ys.clone().step_by(stride) {
        for x in xs.clone().step_by(stride) {
            record!(tape, ours.has_space_proximity(x, y));
            for key in -1..key_count {
                record!(
                    tape,
                    ours.get_normalized_squared_distance_from_center_float_g(key, x, y)
                        .to_bits()
                );
            }
        }
    }
    record_proximities(tape, board, random, stride);
}

/// The letters near each tap, for taps over the keyboard (48 to a call, any primary code), in
/// every locale.
fn record_proximities(tape: &mut Tape, board: &Board, random: &mut Random, stride: usize) {
    let (keyboard, ours) = (&board.keyboard, &board.ours);
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
        let input = TypingInput {
            codes: &codes,
            xs: &tap_xs,
            ys: &tap_ys,
            times: None,
            pointer_ids: None,
        };
        let mut actual = vec![0; codes.len() * MAX_PROXIMITY_CHARS_SIZE];
        ours.initialize_proximities(&input, locale, &mut actual);
        record!(tape, actual);
    }
}

#[test]
fn qwerty_keyboard_matches_the_goldens() {
    let mut tape = tape("layout.qwerty_keyboard");
    record_keyboard(&mut tape, &Board::qwerty(), &mut Random::new(1), 1);
    tape.finish();
}

#[test]
fn random_keyboards_match_the_goldens() {
    let mut tape = tape("layout.random_keyboards");
    let mut random = Random::new(20_261_110);
    for round in 0..200 {
        let board = Board::of(random_keyboard(&mut random));
        record_keyboard(&mut tape, &board, &mut random, 3 + round % 5);
    }
    tape.finish();
}

#[test]
fn sweet_spot_factor_matches_the_goldens() {
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
    let mut tape = tape("layout.sweet_spot_factor");
    for x in values {
        for enabled in [false, true] {
            record!(
                tape,
                touch_position_correction_utils::get_sweet_spot_factor(enabled, x).to_bits()
            );
        }
    }
    tape.finish();
}
