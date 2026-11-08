//! A fixed QWERTY layout for the decoder's spatial model, in abstract units (a key is 100 by 150).
//! Each typed letter is fed to the decoder as a touch on its key here, at the same spot in the key
//! as the real tap ([`touch_point`]), so the real layout on screen and its size don't matter. Was
//! Kotlin's `QwertyGeometry`.
//!
//! The proximity grid follows AOSP's `ProximityInfo.computeNearestNeighbors`: each grid cell lists
//! the keys whose edge lies within 1.2 key widths of the cell's centre.

use crate::java;
use crate::tap::TapOffset;

pub const KEY_WIDTH: i32 = 100;
pub const KEY_HEIGHT: i32 = 150;
pub const KEYBOARD_WIDTH: i32 = 10 * KEY_WIDTH;
pub const KEYBOARD_HEIGHT: i32 = 3 * KEY_HEIGHT;
pub const GRID_WIDTH: i32 = 32;
pub const GRID_HEIGHT: i32 = 16;
/// The native MAX_PROXIMITY_CHARS_SIZE.
pub const MAX_PROXIMITY_CHARS: usize = 16;

const SEARCH_DISTANCE: f32 = 1.2;
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

/// `GRID_WIDTH * GRID_HEIGHT * MAX_PROXIMITY_CHARS` code points near each cell, -1 padded.
pub fn proximity_grid() -> Vec<i32> {
    let cell_width = (KEYBOARD_WIDTH + GRID_WIDTH - 1) / GRID_WIDTH;
    let cell_height = (KEYBOARD_HEIGHT + GRID_HEIGHT - 1) / GRID_HEIGHT;
    let threshold = (KEY_WIDTH as f32 * SEARCH_DISTANCE) as i32;
    let cells = (GRID_WIDTH * GRID_HEIGHT) as usize;
    let mut grid = vec![-1; cells * MAX_PROXIMITY_CHARS];
    let (cells, _) = grid.as_chunks_mut::<MAX_PROXIMITY_CHARS>();
    for (cell, slots) in (0..).zip(cells) {
        let center_x = (cell % GRID_WIDTH) * cell_width + cell_width / 2;
        let center_y = (cell / GRID_WIDTH) * cell_height + cell_height / 2;
        let near = keys().filter(|key| {
            squared_distance_to_edge(key, center_x, center_y) < threshold * threshold
        });
        for (slot, key) in slots.iter_mut().zip(near) {
            *slot = key.code_point;
        }
    }
    grid
}

fn squared_distance_to_edge(key: &Key, x: i32, y: i32) -> i32 {
    let dx = 0.max(key.x - x).max(x - (key.x + KEY_WIDTH));
    let dy = 0.max(key.y - y).max(y - (key.y + KEY_HEIGHT));
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lays_out_three_staggered_rows() {
        let keys: Vec<Key> = keys().collect();
        assert_eq!(keys.len(), 26);
        assert_eq!(
            keys[0],
            Key {
                code_point: 'q' as i32,
                x: 0,
                y: 0
            }
        );
        assert_eq!(
            keys[10],
            Key {
                code_point: 'a' as i32,
                x: 50,
                y: 150
            }
        );
        assert_eq!(
            keys[25],
            Key {
                code_point: 'm' as i32,
                x: 750,
                y: 300
            }
        );
    }

    #[test]
    fn centres_either_case() {
        assert_eq!(key_center('q' as i32), Some((50, 75)));
        assert_eq!(key_center('Q' as i32), Some((50, 75)));
        assert_eq!(key_center('1' as i32), None);
        assert_eq!(key_center(-1), None);
    }

    #[test]
    fn moves_a_touch_by_the_tap_offset() {
        let offset = TapOffset { dx: 0.5, dy: -0.25 };
        assert_eq!(touch_point('p' as i32, Some(offset)), Ok(Some((999, 38))));
        assert_eq!(touch_point('q' as i32, None), Ok(Some((50, 75))));
        let left = TapOffset { dx: -0.6, dy: 0.6 };
        assert_eq!(touch_point('z' as i32, Some(left)), Ok(Some((140, 449))));
        assert_eq!(touch_point('!' as i32, Some(offset)), Ok(None));
        let nan = TapOffset {
            dx: f32::NAN,
            dy: 0.0,
        };
        assert_eq!(touch_point('a' as i32, Some(nan)), Err(CannotRoundNaN));
    }

    #[test]
    fn every_cell_lists_the_keys_near_it() {
        let grid = proximity_grid();
        assert_eq!(grid.len(), 32 * 16 * 16);
        // The first cell's centre is (16, 14): within 120 of the edges of q and w only.
        assert_eq!(grid[..3], ['q' as i32, 'w' as i32, -1]);
        for cell in grid.chunks(MAX_PROXIMITY_CHARS) {
            let keys = cell.iter().take_while(|&&c| c >= 0).count();
            assert!(
                cell[keys..].iter().all(|&c| c == -1),
                "padding only after the keys"
            );
        }
    }

    #[test]
    fn the_grid_lists_keys_within_one_point_two_keys() {
        // 2,649 entries at 1.2 keys, worked out apart from this code; 1.1 would give 2,416.
        let listed = proximity_grid().iter().filter(|&&cp| cp != -1).count();
        assert_eq!(listed, 2649);
    }
}
