//! Android supplies integers; this module interprets their public bit layout.
use crate::{Field, integer, number};
use wire::editor::{attribute as ops, ime_options, input_type, keyboard, keyboard_request};
use wire::flag;

/// Whether `fields` has a true flag at `at`.
fn flag_at(fields: &[Field], at: usize) -> bool {
    fields.get(at).is_some_and(|value| flag::decode(value))
}

pub fn caps(flags: i32) -> i32 {
    if flags & 0x1000 != 0 {
        1
    } else if flags & 0x2000 != 0 {
        2
    } else if flags & 0x4000 != 0 {
        3
    } else {
        0
    }
}

pub fn input(raw: i32) -> Vec<Field> {
    let kind = match raw & 15 {
        n @ 1..=4 => n,
        _ => 0,
    };
    let variation = match (kind, raw & 0xff0) {
        (4, 0x10) => 1,
        (4, 0x20) => 11,
        (2, 0x10) => 6,
        (1, 0x10) => 12,
        (1, 0x20) => 2,
        (1, 0x30) => 3,
        (1, 0x40) => 10,
        (1, 0x50) => 5,
        (1, 0x60) => 7,
        (1, 0x70) => 9,
        (1, 0x80) => 6,
        (1, 0x90) => 13,
        (1, 0xa0) => 14,
        (1, 0xb0) => 4,
        (1, 0xc0) => 8,
        (1, 0xd0) => 15,
        (1, 0xe0) => 16,
        _ => 0,
    };
    let mut row = vec![vec![]; input_type::SIZE];
    row[input_type::TYPE] = number(kind);
    row[input_type::VARIATION] = number(variation);
    row[input_type::CAPS_MODE] = number(if kind == 1 { caps(raw) } else { 0 });
    for (field, class, mask) in [
        (input_type::NUMBER_DECIMAL, 2, 0x2000),
        (input_type::NUMBER_SIGNED, 2, 0x1000),
        (input_type::TEXT_AUTO_COMPLETE, 1, 0x10000),
        (input_type::TEXT_AUTO_CORRECT, 1, 0x8000),
        (input_type::TEXT_IME_MULTI_LINE, 1, 0x40000),
        (input_type::TEXT_MULTI_LINE, 1, 0x20000),
        (input_type::TEXT_NO_SUGGESTIONS, 1, 0x80000),
    ] {
        row[field] = number(i32::from(kind == class && raw & mask != 0));
    }
    row
}

pub fn options(raw: i32) -> Vec<Field> {
    let action = match raw & 255 {
        n @ 0..=7 => n,
        _ => 1,
    };
    let mut row = vec![vec![]; ime_options::SIZE];
    row[ime_options::ACTION] = number(action);
    for (field, mask) in [
        (ime_options::FORCE_ASCII, 0x80000000u32),
        (ime_options::NAVIGATE_NEXT, 0x8000000),
        (ime_options::NAVIGATE_PREVIOUS, 0x4000000),
        (ime_options::NO_ACCESSORY_ACTION, 0x20000000),
        (ime_options::NO_ENTER_ACTION, 0x40000000),
        (ime_options::NO_EXTRACT_UI, 0x10000000),
        (ime_options::NO_FULLSCREEN, 0x2000000),
        (ime_options::NO_PERSONALIZED_LEARNING, 0x1000000),
    ] {
        row[field] = number(i32::from(raw as u32 & mask != 0));
    }
    row
}

/// An [`ops`] operation, then its input.
pub fn call(args: &[Field]) -> Vec<Field> {
    let raw = integer(args, 1);
    match integer(args, 0) {
        ops::INPUT => input(raw),
        ops::OPTIONS => options(raw),
        ops::CAPS => vec![number(caps(raw))],
        ops::CAPS_FLAGS => vec![number(match raw {
            1 => 0x1000,
            2 => 0x2000,
            3 => 0x4000,
            _ => 0,
        })],
        // Rich-editor classification includes apps supplying a selection with TYPE_NULL.
        ops::RICH => vec![number(i32::from(
            raw & 15 != 0 && raw & 15 <= 4 || integer(args, 2) >= 0 && integer(args, 3) >= 0,
        ))],
        ops::KEYBOARD => {
            use keyboard_request::{FORCE_INCOGNITO, IME_OPTIONS, INCOGNITO_MODE, SUGGESTIONS};
            let request = args.get(1..).unwrap_or_default();
            let decoded = input(integer(request, keyboard_request::INPUT_TYPE));
            let kind = integer(&decoded, input_type::TYPE);
            let variation = integer(&decoded, input_type::VARIATION);
            let mode = match kind {
                2 => 4,
                3 => 6,
                _ => 0,
            };
            let key = if kind == 1 {
                match variation {
                    2 | 15 => 1,
                    6 | 13 | 16 => 3,
                    12 => 4,
                    _ => 2,
                }
            } else {
                2
            };
            let compose = mode != 4 && mode != 6 && key != 3 && flag_at(request, SUGGESTIONS);
            let incognito = match integer(request, INCOGNITO_MODE) {
                0 => false,
                1 => true,
                _ => {
                    integer(request, IME_OPTIONS) & 0x1000000 != 0
                        || flag_at(request, FORCE_INCOGNITO)
                }
            };
            let mut row = vec![vec![]; keyboard::SIZE];
            row[keyboard::MODE] = number(mode);
            row[keyboard::KEY_VARIATION] = number(key);
            row[keyboard::COMPOSING] = number(i32::from(compose));
            row[keyboard::INCOGNITO] = number(i32::from(incognito));
            row
        }
        ops::CAPS_AT => vec![number(crate::caps::flags(&args[2], raw, integer(args, 3)))],
        ops::ENTER_ACTION => {
            let action = integer(args, 2) & 255;
            vec![number(
                if integer(args, 2) & 0x40000000 != 0 || raw & 0x20000 != 0 && flag_at(args, 3) {
                    -1
                } else {
                    match action {
                        2..=7 => action,
                        _ => -1,
                    }
                },
            )]
        }
        _ => Vec::new(),
    }
}
