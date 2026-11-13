//! Key-domain decisions ported from KeyData, CurrencySet, KeyboardState and TextKey.
//! State is a value, not a JNI handle: Kotlin owns observation/batching and Android objects.

use crate::{codes, model_data::PREDEFINED};

/// Region mask and offset, followed by the flags in Kotlin's state register.
const FIELDS: &[(u64, u32)] = &[
    (15, 0),
    (15, 4),
    (3, 8),
    (7, 24),
    (1, 27),
    (1, 10),
    (1, 11),
    (1, 12),
    (1, 13),
    (1, 15),
    (1, 16),
    (1, 17),
    (1, 32),
    (1, 20),
    (1, 21),
];

pub fn read(raw: u64, field: usize) -> Option<i32> {
    let &(mask, offset) = FIELDS.get(field)?;
    let value = ((raw >> offset) & mask) as i32;
    Some(match field {
        0 if value > 10 => 0, // KeyboardMode.fromInt's fallback
        1 if value > 4 => 0,  // KeyVariation.ALL
        3 if value > 2 => 0,  // ImeUiMode.TEXT
        _ => value,
    })
}

pub fn write(raw: u64, field: usize, value: i32) -> Option<u64> {
    let &(mask, offset) = FIELDS.get(field)?;
    Some((raw & !(mask << offset)) | (((value as u64) & mask) << offset))
}

/// Index into the selector's Kotlin children; -1 means none. Presence bits distinguish null
/// children from keys whose evaluation later returns null (which must NOT trigger fallback).
pub fn select(kind: i32, raw: u64, present: u32) -> i32 {
    let variation = read(raw, 1).unwrap_or(0);
    let shift = read(raw, 2).unwrap_or(0);
    let choices: &[i32] = match kind {
        0 if shift == 0 => &[0],
        0 => &[1],
        1 => match shift {
            0 => &[0, 5],
            1 => &[2, 1, 5],
            2 => &[3, 1, 5],
            _ => &[4, 5],
        },
        2 => match variation {
            1 => &[1, 0],
            2 => &[3, 0],
            3 => &[4, 0],
            4 => &[2, 0],
            _ => &[0],
        },
        3 if raw & (1 << 27) != 0 => &[1],
        3 => &[0],
        4 if raw & (1 << 21) != 0 => &[1],
        4 => &[0],
        _ => &[],
    };
    choices
        .iter()
        .copied()
        .find(|&index| present & (1 << index) != 0)
        .unwrap_or(-1)
}

pub fn is_currency(code: i32) -> bool {
    (codes::CURRENCY_SLOT_6..=codes::CURRENCY_SLOT_1).contains(&code)
}

/// Preserve abs(Int.MIN_VALUE) and subtraction wrapping, including positive code lookup.
pub fn currency_slot(code: i32) -> i32 {
    code.wrapping_abs()
        .wrapping_sub(codes::CURRENCY_SLOT_1.wrapping_abs())
}

pub const CURRENCY_DEFAULTS: &[i32] = &[36, 162, 8364, 163, 165, 8369];

pub fn is_space(kind: i32, code: i32) -> bool {
    kind == 0
        && matches!(
            code,
            codes::SPACE | codes::CJK_SPACE | codes::HALF_SPACE | codes::KESHIDA
        )
}

pub fn code_kind(code: i32) -> i32 {
    if code > 0 {
        2
    } else if PREDEFINED
        .iter()
        .any(|key| key.code == code && key.discoverable)
    {
        1
    } else {
        0
    }
}

/// Java StringBuilder.appendCodePoint permits lone surrogate code points, ignores invalid
/// ones only because the original caller caught its exception. Do not use Rust char here.
fn append_code_point(out: &mut Vec<u16>, code: i32) {
    match code {
        0..=0xffff => out.push(code as u16),
        0x10000..=0x10ffff => {
            let cp = code - 0x10000;
            out.push(0xd800 | (cp >> 10) as u16);
            out.push(0xdc00 | (cp & 1023) as u16);
        }
        _ => {}
    }
}

/// ICU's mark predicate is supplied by the Android edge, just like locale casing.
pub fn text(code: i32, label: &[u16], display: bool, non_spacing: bool) -> Vec<u16> {
    let mut out = Vec::new();
    if display || code == codes::URI_COMPONENT_TLD || code < codes::SPACE {
        if non_spacing && !label.starts_with(&[0x25cc]) {
            out.push(0x25cc);
        }
        out.extend_from_slice(label);
    } else {
        append_code_point(&mut out, code);
    }
    out
}

pub fn multi_text(codes: &[i32], label: &[u16], display: bool) -> Vec<u16> {
    if display {
        return label.to_vec();
    }
    let mut out = Vec::new();
    for &code in codes {
        append_code_point(&mut out, code);
    }
    out
}

pub fn popup_mode(mode: i32) -> bool {
    matches!(mode, 0 | 2 | 3 | 5)
}

pub fn popup_group(group: i32) -> Option<&'static str> {
    match group {
        1 => Some("~left"),
        2 => Some("~right"),
        3 => Some("~enter"),
        _ => None,
    }
}

/// Source index in pairs [custom, default] for ALL, EMAIL, NORMAL, PASSWORD, URI.
pub fn popup_source(variation: i32, present: u32) -> i32 {
    let order: &[i32] = match variation {
        3 => &[6, 7, 4, 5, 0, 1],
        2 => &[4, 5, 0, 1],
        1 => &[2, 3, 8, 9, 0, 1],
        4 => &[8, 9, 0, 1],
        _ => &[0, 1],
    };
    order
        .iter()
        .copied()
        .find(|&i| present & (1 << i) != 0)
        .unwrap_or(-1)
}

pub fn phone_hint(code: i32) -> Option<&'static str> {
    match code {
        48 => Some("+"),
        49 => Some(""),
        50 => Some("ABC"),
        51 => Some("DEF"),
        52 => Some("GHI"),
        53 => Some("JKL"),
        54 => Some("MNO"),
        55 => Some("PQRS"),
        56 => Some("TUV"),
        57 => Some("WXYZ"),
        _ => None,
    }
}

/// Android gathers only the two facts this code needs. Clipboard access stays lazy: unrelated
/// keys do not ask Android for its primary clip. a/b are boolean 0/1, except incognito's mode
/// ordinal (a) and language switch's subtype count (a).
pub fn enabled(code: i32, a: i32, b: i32) -> bool {
    match code {
        codes::CLIPBOARD_COPY | codes::CLIPBOARD_CUT => a != 0 && b != 0,
        codes::CLIPBOARD_PASTE => a == 0 && b != 0,
        codes::CLIPBOARD_CLEAR_PRIMARY_CLIP | codes::CLIPBOARD_SELECT_ALL | codes::VOICE_INPUT => {
            a != 0
        }
        codes::TOGGLE_INCOGNITO_MODE => a == 2 && b == 0,
        codes::LANGUAGE_SWITCH => a > 1,
        _ => true,
    }
}

/// Utility action ordinals match the settings enum: emoji, language, keyboard, dynamic, off.
pub fn visible(code: i32, utility: bool, action: i32, language: bool) -> bool {
    let action = if utility { action } else { 4 };
    match code {
        codes::IME_UI_MODE_TEXT | codes::IME_UI_MODE_MEDIA => {
            action == 0 || (action == 3 && !language)
        }
        codes::LANGUAGE_SWITCH => matches!(action, 1 | 2) || (action == 3 && language),
        _ => true,
    }
}
