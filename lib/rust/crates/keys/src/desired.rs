//! The desired key `TextKeyboardLayout` lays a text keyboard out with: a tenth of the keyboard
//! wide and a row high, its visible bounds inside the key margins.

use crate::Rect;
use crate::flex::mode;

/// How much taller than the base row height a character or symbol row may be.
const ROW_HEIGHT_CAP: f32 = 1.12;

/// The desired key's touch and visible bounds on a keyboard of this `mode` (`KeyboardMode.value`)
/// and size with `row_count` rows, given the base row height and the key margins, in pixels.
pub fn desired_key(
    mode: i32,
    keyboard_width: f32,
    keyboard_height: f32,
    row_count: i32,
    row_base_height: f32,
    key_margin_h: f32,
    key_margin_v: f32,
) -> (Rect, Rect) {
    let height = match mode {
        mode::CHARACTERS | mode::NUMERIC_ADVANCED | mode::SYMBOLS | mode::SYMBOLS2 => {
            coerce_at_most(
                keyboard_height / row_count as f32,
                row_base_height * ROW_HEIGHT_CAP,
            )
        }
        _ => row_base_height,
    };
    let mut touch = Rect::default();
    touch.right = touch.left + keyboard_width / 10.0;
    touch.bottom = touch.top + height;
    // `deflateBy` is `inflateBy` of the margins' negatives.
    let (dx, dy) = (-key_margin_h, -key_margin_v);
    let visible = Rect {
        left: touch.left - dx,
        top: touch.top - dy,
        right: touch.right + dx,
        bottom: touch.bottom + dy,
    };
    (touch, visible)
}

/// Kotlin's `coerceAtMost`: `value` unless it is greater than `max`, so NaN stays.
fn coerce_at_most(value: f32, max: f32) -> f32 {
    if value > max { max } else { value }
}

#[cfg(test)]
mod tests;
