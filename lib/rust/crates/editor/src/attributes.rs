//! Android supplies integers; this module interprets their public bit layout.
use crate::{Field, integer, number};

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
    let mut result = vec![
        number(kind),
        number(variation),
        number(if kind == 1 { caps(raw) } else { 0 }),
    ];
    for (class, mask) in [
        (2, 0x2000),
        (2, 0x1000),
        (1, 0x10000),
        (1, 0x8000),
        (1, 0x40000),
        (1, 0x20000),
        (1, 0x80000),
    ] {
        result.push(number(i32::from(kind == class && raw & mask != 0)));
    }
    result
}

pub fn options(raw: i32) -> Vec<Field> {
    let action = match raw & 255 {
        n @ 0..=7 => n,
        _ => 1,
    };
    let mut result = vec![number(action)];
    for mask in [
        0x80000000u32,
        0x8000000,
        0x4000000,
        0x20000000,
        0x40000000,
        0x10000000,
        0x2000000,
        0x1000000,
    ] {
        result.push(number(i32::from(raw as u32 & mask != 0)));
    }
    result
}

pub fn call(args: &[Field]) -> Vec<Field> {
    let raw = integer(args, 1);
    match integer(args, 0) {
        0 => input(raw),
        1 => options(raw),
        2 => vec![number(caps(raw))],
        3 => vec![number(match raw {
            1 => 0x1000,
            2 => 0x2000,
            3 => 0x4000,
            _ => 0,
        })],
        // Rich-editor classification includes apps supplying a selection with TYPE_NULL.
        4 => vec![number(i32::from(
            raw & 15 != 0 && raw & 15 <= 4 || integer(args, 2) >= 0 && integer(args, 3) >= 0,
        ))],
        5 => {
            let decoded = input(raw);
            let kind = integer(&decoded, 0);
            let variation = integer(&decoded, 1);
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
            let compose = mode != 4 && mode != 6 && key != 3 && integer(args, 3) != 0;
            let incognito = match integer(args, 4) {
                0 => false,
                1 => true,
                _ => integer(args, 2) & 0x1000000 != 0 || integer(args, 5) != 0,
            };
            vec![
                number(mode),
                number(key),
                number(i32::from(compose)),
                number(i32::from(incognito)),
            ]
        }
        6 => vec![number(crate::caps::flags(&args[2], raw, integer(args, 3)))],
        7 => {
            let action = integer(args, 2) & 255;
            vec![number(
                if integer(args, 2) & 0x40000000 != 0 || raw & 0x20000 != 0 && integer(args, 3) != 0
                {
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
