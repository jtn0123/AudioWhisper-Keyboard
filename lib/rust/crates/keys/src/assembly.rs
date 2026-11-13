//! Layout selection, arrangement merging, hint alignment and computed-keyboard cache keys.
//! Keys remain opaque indices: decoding, casing and Android objects stay with the caller.

use crate::flex::mode;

// LayoutType ordinals, retained by the Kotlin serialization model.
const CHARACTERS: i32 = 0;
const CHARACTERS_MOD: i32 = 1;
const NUMERIC: i32 = 3;
const NUMERIC_ADVANCED: i32 = 4;
const NUMERIC_ROW: i32 = 5;
const PHONE: i32 = 6;
const PHONE2: i32 = 7;
const SYMBOLS: i32 = 8;
const SYMBOLS_MOD: i32 = 9;
const SYMBOLS2: i32 = 10;
const SYMBOLS2_MOD: i32 = 11;

/// Main, modifier and extension types. -1 means no layout, including unknown/unused modes.
pub fn plan(mode: i32, number_row: bool) -> [i32; 3] {
    match mode {
        mode::CHARACTERS => [
            CHARACTERS,
            CHARACTERS_MOD,
            if number_row { NUMERIC_ROW } else { -1 },
        ],
        mode::SYMBOLS => [SYMBOLS, SYMBOLS_MOD, NUMERIC_ROW],
        mode::SYMBOLS2 => [SYMBOLS2, SYMBOLS2_MOD, -1],
        mode::NUMERIC => [NUMERIC, -1, -1],
        mode::NUMERIC_ADVANCED => [NUMERIC_ADVANCED, -1, -1],
        mode::PHONE => [PHONE, -1, -1],
        mode::PHONE2 => [PHONE2, -1, -1],
        _ => [-1; 3],
    }
}

/// A main layout's explicit modifier overrides the subtype's normal one.
pub fn modifier_type(main_type: i32) -> i32 {
    match main_type {
        SYMBOLS => SYMBOLS_MOD,
        SYMBOLS2 => SYMBOLS2_MOD,
        _ => CHARACTERS_MOD,
    }
}

/// Row sizes become ranges into the concatenated main/modifier/extension key list.
/// [-1] is an unavailable layout; [] is a successfully decoded empty layout.
fn rows(sizes: &[i32], offset: &mut i32) -> Result<Option<Vec<Vec<i32>>>, &'static str> {
    if sizes == [-1] {
        return Ok(None);
    }
    let mut rows = Vec::with_capacity(sizes.len());
    for &size in sizes {
        if size < 0 {
            return Err("negative row size");
        }
        let end = offset.checked_add(size).ok_or("too many keys")?;
        rows.push((*offset..end).collect());
        *offset = end;
    }
    Ok(Some(rows))
}

/// The extension precedes main; the first modifier row replaces the last main row,
/// inserting that row at every plain TextKeyData with code 0 (not at computed selectors).
/// Return rows packed as [size, key indices..., size, ...], retaining empty rows.
pub fn merge(
    main: &[i32],
    modifier: &[i32],
    extension: &[i32],
    mod_codes: &[i32],
) -> Result<Vec<i32>, &'static str> {
    let mut offset = 0;
    let main = rows(main, &mut offset)?;
    let mod_start = offset;
    let modifier = rows(modifier, &mut offset)?;
    if offset - mod_start != mod_codes.len() as i32 {
        return Err("modifier codes do not cover its keys");
    }
    let extension = rows(extension, &mut offset)?;
    let mut out = extension.unwrap_or_default();
    match (main, modifier) {
        (Some(main), Some(modifier)) => {
            for (i, row) in main.iter().enumerate() {
                if i + 1 < main.len() {
                    out.push(row.clone());
                } else {
                    let mut merged = Vec::new();
                    for &key in modifier.first().into_iter().flatten() {
                        if mod_codes[(key - mod_start) as usize] == 0 {
                            merged.extend(row);
                        } else {
                            merged.push(key);
                        }
                    }
                    out.push(merged);
                }
            }
            out.extend(modifier.into_iter().skip(1));
        }
        (Some(rows), None) | (None, Some(rows)) => out.extend(rows),
        (None, None) => {}
    }
    let mut packed = Vec::new();
    for row in out {
        packed.push(row.len() as i32);
        packed.extend(row);
    }
    Ok(packed)
}

/// Two symbol-key indices per output key: number hint, then symbol hint; -1 is absent.
/// Number hints come from the first symbol row; character hints are bottom-aligned.
/// Types are those computed with the default evaluator, not the live shift/subtype state.
pub fn hints(
    mode: i32,
    number_hints: bool,
    main: &[i32],
    symbols: &[i32],
    types: &[i32],
) -> Result<Vec<i32>, &'static str> {
    let main = rows(main, &mut 0)?.ok_or("missing assembled layout")?;
    let symbols = rows(symbols, &mut 0)?.ok_or("missing assembled symbols")?;
    if symbols.iter().map(Vec::len).sum::<usize>() != types.len() {
        return Err("hint types do not cover symbol keys");
    }
    let mut out = Vec::new();
    let offset = main.len() as i64 - symbols.len() as i64;
    for (r, row) in main.iter().enumerate() {
        for k in 0..row.len() {
            let at = |r: usize, kind| {
                symbols
                    .get(r)
                    .and_then(|row| row.get(k))
                    .copied()
                    .filter(|&key| types[key as usize] == kind)
                    .unwrap_or(-1)
            };
            let enabled = mode == mode::CHARACTERS;
            out.push(if enabled && number_hints && r == 0 {
                at(0, 7)
            } else {
                -1
            });
            let symbol_row = r as i64 - offset;
            out.push(if enabled && symbol_row >= 0 {
                at(symbol_row as usize, 0)
            } else {
                -1
            });
        }
    }
    Ok(out)
}

/// Preserve the old per-mode Int-hash cache, including intentional hash collisions.
/// The subtype's Kotlin data-class hash is an opaque identity; Rust owns key packing/filtering.
pub fn cache_key(mode: i32, subtype_hash: i32) -> i64 {
    ((mode as i64) << 32) | (subtype_hash as u32 as i64)
}

pub fn invalidated(keys: &[i64], mode: Option<i32>, subtype_hash: Option<i32>) -> Vec<i64> {
    keys.iter()
        .copied()
        .filter(|&key| {
            mode.is_none_or(|mode| (key >> 32) as i32 == mode)
                && subtype_hash.is_none_or(|hash| key as i32 == hash)
        })
        .collect()
}

#[cfg(test)]
mod tests;
