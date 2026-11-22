//! Where keys were tapped (`suggest::tap` and `suggest::qwerty`), for `SuggestNative`, whose
//! generated entry points call these. A `TapPositions` is a handle to the trail and the learned
//! tap aim (`suggest::tap_learning::Taps`, see tap_aim.rs) behind a lock: the keyboard writes it on
//! the main thread and reads it while suggesting.

use std::sync::{Mutex, MutexGuard};

use suggest::qwerty;
use suggest::tap::{TapOffset, TapTrail};
use suggest::tap_learning::Taps;

use crate::handle::{into_handle, locked};
use crate::jni::{Result, illegal};

/// How far beside a key a tap still counts for it (`TapOffset::LIMIT`).
pub fn limit() -> f32 {
    TapOffset::LIMIT
}

/// How many characters a trail keeps unless told otherwise (`TapTrail::CAPACITY`).
pub fn capacity() -> i32 {
    TapTrail::CAPACITY as i32
}

/// The offset of a tap on a key (`TapOffset::of`): dx and dy, or null for a key with no size.
pub fn offset(
    x: f32,
    y: f32,
    center_x: f32,
    center_y: f32,
    width: f32,
    height: f32,
) -> Option<Vec<f32>> {
    TapOffset::of(x, y, center_x, center_y, width, height).map(|offset| vec![offset.dx, offset.dy])
}

/// Three values per character: 1 if it was tapped and 0 if not, then dx and dy.
pub(super) fn packed(offsets: &[Option<TapOffset>]) -> Vec<f32> {
    offsets
        .iter()
        .flat_map(|offset| match offset {
            Some(offset) => [1.0, offset.dx, offset.dy],
            None => [0.0, 0.0, 0.0],
        })
        .collect()
}

/// The offset (`dx`, `dy`) if `has_offset`.
fn tapped_at(has_offset: bool, dx: f32, dy: f32) -> Option<TapOffset> {
    has_offset.then_some(TapOffset { dx, dy })
}

/// A new trail of the last `capacity` characters. A negative capacity throws, on purpose: Kotlin's
/// `TapTrail` took one and then threw on the first character typed into it.
pub fn new_trail(capacity: i32) -> Result<i64> {
    let capacity = usize::try_from(capacity).map_err(|_| illegal("capacity is negative"))?;
    Ok(into_handle(Mutex::new(Taps::new(capacity))))
}

pub fn free_trail(trail: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each trail once, when nothing can reach it.
    unsafe { crate::handle::free::<Mutex<Taps>>(trail) };
    Ok(())
}

/// The trail behind `handle`, locked.
pub(crate) fn trail<'a>(handle: i64) -> Result<MutexGuard<'a, Taps>> {
    // SAFETY: Kotlin holds the trail while it calls, so it hasn't been freed.
    unsafe { locked(handle, "tap trail") }
}

/// A key for `code_point` went down, at (`dx`, `dy`) if `has_offset`.
pub fn pressed(handle: i64, code_point: i32, has_offset: bool, dx: f32, dy: f32) -> Result<()> {
    trail(handle)?
        .trail
        .pressed(code_point, tapped_at(has_offset, dx, dy));
    Ok(())
}

/// `text` went into the editor.
pub fn typed(handle: i64, text: &[u16]) -> Result<()> {
    trail(handle)?.typed(text);
    Ok(())
}

/// The last typed character was deleted.
pub fn deleted(handle: i64) -> Result<()> {
    trail(handle)?.deleted();
    Ok(())
}

/// Everything typed is forgotten, and the letter odds with it (a new field started).
pub fn cleared(handle: i64) -> Result<()> {
    trail(handle)?.clear();
    super::letter_odds::forget();
    Ok(())
}

/// Where each character of `word` was tapped (`TapTrail::offsets_for`): three values per
/// character, 1 if it was tapped and 0 if not, then dx and dy; empty if the trail doesn't spell it.
pub fn offsets_for(handle: i64, word: &[u16]) -> Result<Vec<f32>> {
    Ok(packed(&trail(handle)?.trail.offsets_for(word)))
}

/// The QWERTY layout's sizes: a key's width and height, the keyboard's, the proximity grid's
/// columns and rows, and how many keys a grid cell lists at most.
pub fn qwerty_sizes() -> Vec<i32> {
    vec![
        qwerty::KEY_WIDTH,
        qwerty::KEY_HEIGHT,
        qwerty::KEYBOARD_WIDTH,
        qwerty::KEYBOARD_HEIGHT,
        qwerty::GRID_WIDTH,
        qwerty::GRID_HEIGHT,
        qwerty::MAX_PROXIMITY_CHARS as i32,
    ]
}

/// The QWERTY layout's keys: code point, left and top for each.
pub fn qwerty_keys() -> Vec<i32> {
    qwerty::keys()
        .flat_map(|key| [key.code_point, key.x, key.y])
        .collect()
}

/// The centre of the key for `code_point` (x, y), or null.
pub fn key_center(code_point: i32) -> Option<Vec<i32>> {
    qwerty::key_center(code_point).map(|(x, y)| vec![x, y])
}

/// Where a tap at (`dx`, `dy`), if `has_offset`, on the key for `code_point` lands (x, y), or
/// null. An offset of NaN throws, as Kotlin's `roundToInt` did.
pub fn touch_point(
    code_point: i32,
    has_offset: bool,
    dx: f32,
    dy: f32,
) -> Result<Option<Vec<i32>>> {
    let point = qwerty::touch_point(code_point, tapped_at(has_offset, dx, dy))
        .map_err(|_| illegal("Cannot round NaN value."))?;
    Ok(point.map(|(x, y)| vec![x, y]))
}

/// The proximity grid (`qwerty::proximity_grid`).
pub fn proximity_grid() -> Vec<i32> {
    qwerty::proximity_grid()
}
