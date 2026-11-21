//! The text keyboard's geometry (the `keys` crate), for `KeysNative`, whose generated entry points
//! call these.
//!
//! Rects cross as four Floats each (left, top, right, bottom), and a keyboard's keys as one flat
//! array with the number of keys in each row beside it.

use keys::{Bounds, KeyFlex, Rect};

use crate::jni::{Result, illegal};

pub(crate) fn rect(&[left, top, right, bottom]: &[f32; 4]) -> Rect {
    Rect {
        left,
        top,
        right,
        bottom,
    }
}

fn floats(rect: Rect) -> [f32; 4] {
    [rect.left, rect.top, rect.right, rect.bottom]
}

/// `items` cut into rows of `sizes` items each, which must cover them exactly.
pub(crate) fn rows<'a, T>(items: &'a [T], sizes: &[i32]) -> Result<Vec<&'a [T]>> {
    let mut rest = items;
    let mut rows = Vec::with_capacity(sizes.len());
    for &size in sizes {
        let size = usize::try_from(size).map_err(|_| illegal("a row has a negative size"))?;
        let (row, after) = rest
            .split_at_checked(size)
            .ok_or_else(|| illegal("the rows have more keys than given"))?;
        rows.push(row);
        rest = after;
    }
    if !rest.is_empty() {
        return Err(illegal("the rows have fewer keys than given"));
    }
    Ok(rows)
}

/// The flex factors of a visible key (see `keys::flex`): shrink, grow and width factor.
pub fn flex(mode: i32, code: i32, numeric: bool) -> Vec<f32> {
    let flex = keys::flex(mode, code, numeric);
    vec![flex.shrink, flex.grow, flex.width_factor]
}

/// The desired key's touch and visible bounds (see `keys::desired_key`), eight Floats.
pub fn desired_key(
    mode: i32,
    keyboard_width: f32,
    keyboard_height: f32,
    row_count: i32,
    row_base_height: f32,
    key_margin_h: f32,
    key_margin_v: f32,
) -> Vec<f32> {
    let (touch, visible) = keys::desired_key(
        mode,
        keyboard_width,
        keyboard_height,
        row_count,
        row_base_height,
        key_margin_h,
        key_margin_v,
    );
    [floats(touch), floats(visible)].concat()
}

/// Lays out rows of `row_sizes` keys, whose `flex` is three Floats each (width factor, shrink,
/// grow), with the `desired` key's touch and visible bounds (eight Floats): eight Floats a key,
/// its touch and visible bounds, or null if nothing is laid out (see `keys::layout`).
pub fn layout(
    row_sizes: &[i32],
    flex: &[f32],
    desired: &[f32],
    keyboard_width: f32,
    keyboard_height: f32,
    extend_down: bool,
) -> Result<Option<Vec<f32>>> {
    let (flex, rest) = flex.as_chunks::<3>();
    if !rest.is_empty() {
        return Err(illegal("flex needs three values per key"));
    }
    let flex: Vec<KeyFlex> = flex
        .iter()
        .map(|&[width_factor, shrink, grow]| KeyFlex {
            width_factor,
            shrink,
            grow,
        })
        .collect();
    let (touch, visible) = desired_bounds(desired)?;
    let laid_out = keys::layout(
        &rows(&flex, row_sizes)?,
        touch,
        visible,
        keyboard_width,
        keyboard_height,
        extend_down,
    );
    Ok(laid_out.map(|bounds| {
        bounds
            .iter()
            .flat_map(|&Bounds { touch, visible }| [floats(touch), floats(visible)])
            .flatten()
            .collect()
    }))
}

fn desired_bounds(desired: &[f32]) -> Result<(Rect, Rect)> {
    match desired.as_chunks::<4>() {
        ([touch, visible], []) => Ok((rect(touch), rect(visible))),
        _ => Err(illegal("desired needs eight values")),
    }
}

/// The index of the key whose touch bounds (four Floats a key, in rows of `row_sizes`) hold
/// (x, y), or -1 (see `keys::key_at`).
pub fn key_at(row_sizes: &[i32], touch_bounds: &[f32], x: f32, y: f32) -> Result<i32> {
    let (bounds, rest) = touch_bounds.as_chunks::<4>();
    if !rest.is_empty() {
        return Err(illegal("touchBounds needs four values per key"));
    }
    let bounds: Vec<Rect> = bounds.iter().map(rect).collect();
    let index = keys::key_at(&rows(&bounds, row_sizes)?, x, y);
    // The index fits in an Int: Kotlin's array held that many keys.
    Ok(index.map_or(-1, |index| index as i32))
}
