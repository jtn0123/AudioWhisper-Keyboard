//! The en-US keyboard as a Galaxy S24 Ultra draws it in portrait, laid out by the keyboard's own
//! code: the layout assets the app ships, merged as FlorisBoard merges them, then
//! [`keys::layout`] at the phone's size with the default settings.
//!
//! The phone is 1440 px wide at 3.75 px per dp (384 dp). Its base row height is 199.2 px (what
//! `ImeChromeRules.rowHeights` gives on that screen at the default height factor), the default key
//! spacing is 2 dp across and 5 dp down, and the emoji key sits beside the space bar, so the
//! language key is hidden.

use keys::{KeyFlex, Rect};
use serde_json::Value;
use std::path::Path;

/// The phone's screen width, in pixels.
pub const WIDTH: f32 = 1440.0;
/// Pixels per dp.
pub const DENSITY: f32 = 3.75;
/// The base row height, in pixels.
pub const ROW_BASE_HEIGHT: f32 = 199.2;
/// `keyboard__key_spacing_horizontal` and `keyboard__key_spacing_vertical`, in dp.
const KEY_SPACING_H: f32 = 2.0;
const KEY_SPACING_V: f32 = 5.0;

const PLACEHOLDER: i32 = 0;
const LANGUAGE_SWITCH: i32 = -227;

/// A laid-out key: its code, and where it takes touches and where it is drawn.
#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub code: i32,
    pub touch: Rect,
    pub visible: Rect,
}

impl Key {
    pub fn center(&self) -> (f32, f32) {
        let v = &self.visible;
        ((v.left + v.right) / 2.0, (v.top + v.bottom) / 2.0)
    }

    /// A key that types a letter.
    pub fn is_letter(&self) -> bool {
        u16::try_from(self.code).is_ok_and(chars::is_letter)
    }
}

/// The character keyboard, laid out.
#[derive(Debug)]
pub struct Keyboard {
    /// Every visible key, row by row.
    pub keys: Vec<Key>,
    /// How many keys each row has.
    row_lengths: Vec<usize>,
    pub width: f32,
    pub height: f32,
}

impl Keyboard {
    /// The en-US QWERTY layout from the app's `assets` folder at the phone's size.
    pub fn phone(assets: &Path) -> Result<Keyboard, String> {
        let layouts = assets.join("ime/keyboard/org.florisboard.layouts/layouts");
        let read = |name: &str| {
            std::fs::read_to_string(layouts.join(name)).map_err(|error| format!("{name}: {error}"))
        };
        let rows = merged_rows(
            &read("characters/qwerty.json")?,
            &read("charactersMod/default.json")?,
        )?;
        Keyboard::lay_out(&rows, WIDTH)
    }

    /// `rows` of key codes laid out `width` pixels wide, at the phone's row height and spacing.
    pub fn lay_out(rows: &[Vec<i32>], width: f32) -> Result<Keyboard, String> {
        let row_count = rows.len();
        let height = ROW_BASE_HEIGHT * row_count as f32;
        let (touch, visible) = keys::desired_key(
            keys::mode::CHARACTERS,
            width,
            height,
            row_count as i32,
            ROW_BASE_HEIGHT,
            KEY_SPACING_H * DENSITY,
            KEY_SPACING_V * DENSITY,
        );
        let flexes: Vec<Vec<KeyFlex>> = rows
            .iter()
            .map(|row| row.iter().map(|&code| flex(code)).collect())
            .collect();
        let flex_rows: Vec<&[KeyFlex]> = flexes.iter().map(Vec::as_slice).collect();
        let bounds = keys::layout(&flex_rows, touch, visible, width, height, true)
            .ok_or("the keyboard did not lay out")?;
        let keys = rows
            .iter()
            .flatten()
            .zip(bounds)
            .map(|(&code, b)| Key {
                code,
                touch: b.touch,
                visible: b.visible,
            })
            .collect();
        Ok(Keyboard {
            keys,
            row_lengths: rows.iter().map(Vec::len).collect(),
            width,
            height,
        })
    }

    /// The distance from one letter key to the next, across and down: the width and height of a
    /// top-row letter key's touch bounds away from the edges. (The middle row's nine keys are a
    /// little wider.)
    pub fn pitch(&self) -> (f32, f32) {
        self.key_for(i32::from(b'w'))
            .map_or((0.0, 0.0), |key| (key.touch.width(), key.touch.height()))
    }

    /// The key that types `code`.
    pub fn key_for(&self, code: i32) -> Option<&Key> {
        self.index_for(code).map(|index| &self.keys[index])
    }

    /// Where in [`Keyboard::keys`] the key that types `code` is.
    pub fn index_for(&self, code: i32) -> Option<usize> {
        self.keys.iter().position(|key| key.code == code)
    }

    /// The key a touch at (x, y) lands on, as the keyboard finds it.
    pub fn key_at(&self, x: f32, y: f32) -> Option<&Key> {
        self.index_at(x, y).map(|index| &self.keys[index])
    }

    /// Where in [`Keyboard::keys`] the key a touch at (x, y) lands on is.
    pub fn index_at(&self, x: f32, y: f32) -> Option<usize> {
        let mut rows = Vec::with_capacity(self.row_lengths.len());
        let mut start = 0;
        let touches: Vec<Rect> = self.keys.iter().map(|key| key.touch).collect();
        for &length in &self.row_lengths {
            rows.push(&touches[start..start + length]);
            start += length;
        }
        keys::key_at(&rows, x, y)
    }

    /// The letter keys' centres and sizes, as glide typing reads every key.
    pub fn glide_keys(&self) -> Vec<glide::Key> {
        self.keys
            .iter()
            .map(|key| {
                let (center_x, center_y) = key.center();
                glide::Key {
                    code: key.code,
                    center_x,
                    center_y,
                    width: key.visible.width(),
                    height: key.visible.height(),
                }
            })
            .collect()
    }
}

fn flex(code: i32) -> KeyFlex {
    let f = keys::flex(keys::mode::CHARACTERS, code, false);
    KeyFlex {
        width_factor: f.width_factor,
        shrink: f.shrink,
        grow: f.grow,
    }
}

/// The rows of the main layout merged into the mod layout: the mod's placeholder takes the main
/// layout's last row, whose other rows go on top. Hidden keys are left out, and the variation
/// selector is its default key, the comma.
pub fn merged_rows(main: &str, modifier: &str) -> Result<Vec<Vec<i32>>, String> {
    let main = codes(main)?;
    let modifier = codes(modifier)?;
    let (last, top) = main.split_last().ok_or("the main layout has no rows")?;
    let mut rows: Vec<Vec<i32>> = top.to_vec();
    for row in modifier {
        let mut merged = Vec::new();
        for code in row {
            match code {
                PLACEHOLDER => merged.extend_from_slice(last),
                LANGUAGE_SWITCH => {}
                code => merged.push(code),
            }
        }
        rows.push(merged);
    }
    Ok(rows)
}

/// Each key's code, row by row.
fn codes(json: &str) -> Result<Vec<Vec<i32>>, String> {
    let value: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    let rows = value.as_array().ok_or("a layout is an array of rows")?;
    rows.iter()
        .map(|row| {
            row.as_array()
                .ok_or("a row is an array of keys")?
                .iter()
                .map(code)
                .collect()
        })
        .collect()
}

fn code(key: &Value) -> Result<i32, String> {
    let key = match key.get("$").and_then(Value::as_str) {
        Some("variation_selector") => key.get("default").ok_or("no default variation")?,
        _ => key,
    };
    let code = key
        .get("code")
        .and_then(Value::as_i64)
        .ok_or("a key without a code")?;
    i32::try_from(code).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assets() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../app/src/main/assets")
    }

    #[test]
    fn merges_the_shipped_layout_into_four_rows() {
        let keyboard = Keyboard::phone(&assets()).unwrap();
        let codes: Vec<i32> = keyboard.keys.iter().map(|key| key.code).collect();
        let letters: String = codes
            .iter()
            .filter_map(|&c| char::from_u32(c as u32).filter(char::is_ascii_lowercase))
            .collect();
        assert_eq!(letters, "qwertyuiopasdfghjklzxcvbnm");
        assert_eq!(keyboard.row_lengths, [10, 9, 9, 6]);
        assert_eq!(
            &codes[19..],
            [
                -11, 122, 120, 99, 118, 98, 110, 109, -7, -202, 44, -212, 32, 46, 10
            ]
        );
    }

    #[test]
    fn keys_are_a_tenth_of_the_phone_wide_and_taps_find_them() {
        let keyboard = Keyboard::phone(&assets()).unwrap();
        let w = keyboard.key_for('w' as i32).unwrap();
        assert!((w.touch.width() - 142.5).abs() < 0.01);
        assert!((w.visible.width() - 127.5).abs() < 0.01);
        assert_eq!(keyboard.pitch(), (142.5, 199.2));
        // The keys at the edges take the touches beside them.
        assert_eq!(keyboard.key_at(1.0, 300.0).unwrap().code, 'a' as i32);
        let a = keyboard.key_for('a' as i32).unwrap();
        // The middle row is indented half a key.
        assert!((a.center().0 - 144.0).abs() < 0.01, "{:?}", a.visible);
        for key in &keyboard.keys {
            let (x, y) = key.center();
            assert_eq!(keyboard.key_at(x, y).unwrap().code, key.code);
        }
        let space = keyboard.key_for(32).unwrap();
        assert!(space.touch.width() > 3.0 * 144.0);
    }
}
