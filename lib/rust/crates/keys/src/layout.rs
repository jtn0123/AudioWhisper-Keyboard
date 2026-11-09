//! Laying a text keyboard's rows out at a size, and finding the key a touch lands on: FlorisBoard's
//! `TextKeyboard.layout` and `getKeyForPos`.

use crate::Rect;

/// A key's flex factors as its `flayWidthFactor`, `flayShrink` and `flayGrow` hold them, whatever
/// set them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct KeyFlex {
    pub width_factor: f32,
    pub shrink: f32,
    pub grow: f32,
}

/// A laid-out key's bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bounds {
    pub touch: Rect,
    pub visible: Rect,
}

/// Where each key of `rows` goes on a keyboard `keyboard_width` by `keyboard_height`, in row order,
/// given the desired key's touch and visible bounds; with `extend_down`, the last row's touch
/// bounds reach down another row. `None` if the Kotlin laid nothing out (an empty desired key or
/// a NaN size), leaving each key's bounds as they were.
pub fn layout(
    rows: &[&[KeyFlex]],
    desired_touch: Rect,
    desired_visible: Rect,
    keyboard_width: f32,
    keyboard_height: f32,
    extend_down: bool,
) -> Option<Vec<Bounds>> {
    // Kotlin also returned first without rows, which lays nothing out either way.
    if desired_touch.is_empty() || desired_visible.is_empty() {
        return None;
    }
    if keyboard_width.is_nan() || keyboard_height.is_nan() {
        return None;
    }
    let row_count = rows.len() as i32;
    let row_margin_h = (desired_touch.width() - desired_visible.width()).abs();
    let row_margin_v = (keyboard_height - desired_touch.height() * row_count as f32)
        / (row_count - 1).max(1) as f32;
    let row = Row {
        desired_touch,
        insets: Rect {
            left: (desired_touch.left - desired_visible.left).abs(),
            top: (desired_touch.top - desired_visible.top).abs(),
            right: (desired_touch.right - desired_visible.right).abs(),
            bottom: (desired_touch.bottom - desired_visible.bottom).abs(),
        },
        row_margin_h,
        keyboard_width,
        extend_down,
    };
    let mut bounds = Vec::with_capacity(rows.iter().map(|row| row.len()).sum());
    for (r, keys) in rows.iter().enumerate() {
        let pos_y = (desired_touch.height() + row_margin_v) * r as f32;
        let last = r + 1 == rows.len();
        row.lay_out(keys, pos_y, last, &mut bounds);
    }
    Some(bounds)
}

/// What every row is laid out with.
struct Row {
    desired_touch: Rect,
    /// How far in from each edge of its touch bounds a key's visible bounds are.
    insets: Rect,
    row_margin_h: f32,
    keyboard_width: f32,
    extend_down: bool,
}

impl Row {
    fn lay_out(&self, keys: &[KeyFlex], pos_y: f32, last: bool, out: &mut Vec<Bounds>) {
        let width = self.desired_touch.width();
        let available_width = (self.keyboard_width - self.row_margin_h) / width;
        let mut requested_width = 0.0f32;
        let mut shrink_sum = 0.0f32;
        let mut grow_sum = 0.0f32;
        for key in keys {
            requested_width += key.width_factor;
            shrink_sum += key.shrink;
            grow_sum += key.grow;
        }
        let mut pos_x = self.row_margin_h / 2.0;
        let end = keys.len().wrapping_sub(1);
        // Requested width at most what's available: the keys can grow. Otherwise (or if NaN)
        // they must shrink.
        let grows = requested_width <= available_width;
        let additional_width = available_width - requested_width;
        let clipping_width = requested_width - available_width;
        for (k, key) in keys.iter().enumerate() {
            let factor = if grows {
                if grow_sum == 0.0 {
                    if k == 0 || k == end {
                        key.width_factor + additional_width / 2.0
                    } else {
                        key.width_factor
                    }
                } else {
                    key.width_factor + additional_width * (key.grow / grow_sum)
                }
            } else if key.shrink == 0.0 {
                key.width_factor
            } else {
                key.width_factor - clipping_width * (key.shrink / shrink_sum)
            };
            let key_width = width * factor;
            let mut touch = Rect {
                left: pos_x,
                top: pos_y,
                right: pos_x + key_width,
                bottom: pos_y + self.desired_touch.height(),
            };
            let mut visible = Rect {
                left: touch.left + self.insets.left,
                top: touch.top + self.insets.top,
                right: touch.right - self.insets.right,
                bottom: touch.bottom - self.insets.bottom,
            };
            // Growing evenly, the end keys' visible bounds keep their width; their touch bounds
            // take the extra.
            if grows && grow_sum == 0.0 {
                let extra = (additional_width / 2.0) * width;
                if k == 0 {
                    visible.left += extra;
                }
                if k == end {
                    visible.right -= extra;
                }
            }
            pos_x += key_width;
            // The end keys' touch bounds reach the keyboard's edges, over the row margin.
            if k == 0 {
                touch.left = 0.0;
            } else if k == end {
                touch.right = self.keyboard_width;
            }
            if self.extend_down && last {
                touch.bottom += touch.height();
            }
            out.push(Bounds { touch, visible });
        }
    }
}

/// The index, counting every key of every row in order, of the first key whose touch bounds hold
/// (x, y). As `TextKeyboard.keys()` does, the search stops at the first row without keys.
pub fn key_at(rows: &[&[Rect]], x: f32, y: f32) -> Option<usize> {
    let mut index = 0;
    for keys in rows {
        if keys.is_empty() {
            return None;
        }
        for touch in *keys {
            if touch.contains(x, y) {
                return Some(index);
            }
            index += 1;
        }
    }
    None
}

#[cfg(test)]
mod tests;
