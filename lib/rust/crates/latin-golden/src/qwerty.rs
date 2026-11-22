//! Frozen pre-A1 QWERTY model, as the C++ oracle's questions placed their taps.
//! A fixed QWERTY layout for the decoder's spatial model, in abstract units (a key is 100 by 150).
//! Each typed letter is fed to the decoder as a touch on its key here, at the same spot in the key
//! as the real tap ([`touch_point`]), so the real layout on screen and its size don't matter. Was
//! Kotlin's `QwertyGeometry`.

use crate::math as java;
#[derive(Clone, Copy, Debug)]
pub struct TapOffset {
    pub dx: f32,
    pub dy: f32,
}

pub const KEY_WIDTH: i32 = 100;
pub const KEY_HEIGHT: i32 = 150;
pub const KEYBOARD_WIDTH: i32 = 10 * KEY_WIDTH;
pub const KEYBOARD_HEIGHT: i32 = 3 * KEY_HEIGHT;

const ROWS: [&str; 3] = ["qwertyuiop", "asdfghjkl", "zxcvbnm"];
const ROW_OFFSETS: [i32; 3] = [0, KEY_WIDTH / 2, KEY_WIDTH * 3 / 2];

/// A key's code point and its top-left corner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub code_point: i32,
    pub x: i32,
    pub y: i32,
}

/// The keys, row by row.
pub fn keys() -> impl Iterator<Item = Key> {
    (0..).zip(ROWS).flat_map(|(row, letters)| {
        (0..).zip(letters.chars()).map(move |(column, letter)| Key {
            code_point: letter as i32,
            x: ROW_OFFSETS[row as usize] + column * KEY_WIDTH,
            y: row * KEY_HEIGHT,
        })
    })
}

/// The centre of the key that types `code_point` (either case), or `None` if there is none.
pub fn key_center(code_point: i32) -> Option<(i32, i32)> {
    let lower = java::to_lower(code_point);
    let key = keys().find(|key| key.code_point == lower)?;
    Some((key.x + KEY_WIDTH / 2, key.y + KEY_HEIGHT / 2))
}

/// Kotlin's `roundToInt` was given NaN, which it throws for.
#[derive(Debug, PartialEq, Eq)]
pub struct CannotRoundNaN;

/// Where a tap on the key for `code_point` lands here: `offset` from its centre, or the centre
/// when the tap's position is unknown. `None` if no key types `code_point`.
pub fn touch_point(
    code_point: i32,
    offset: Option<TapOffset>,
) -> Result<Option<(i32, i32)>, CannotRoundNaN> {
    let Some((x, y)) = key_center(code_point) else {
        return Ok(None);
    };
    let Some(offset) = offset else {
        return Ok(Some((x, y)));
    };
    let along = |centre: i32, delta: f32, size: i32, limit: i32| {
        let moved =
            java::round_to_int(centre as f32 + delta * size as f32).ok_or(CannotRoundNaN)?;
        Ok(moved.clamp(0, limit - 1))
    };
    Ok(Some((
        along(x, offset.dx, KEY_WIDTH, KEYBOARD_WIDTH)?,
        along(y, offset.dy, KEY_HEIGHT, KEYBOARD_HEIGHT)?,
    )))
}
