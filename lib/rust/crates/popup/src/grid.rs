//! The extended popup a long press opens, as `PopupUiController.extend` laid it out and
//! `propagateMotionEvent` followed the finger across it.
//!
//! With `n` keys: up to 5 sit in one row (row 0); more go in two, row 0 holding the extra one
//! when `n` is odd. Row 1 is drawn above row 0, and the keys are numbered row 1 first, left to
//! right, then row 0. The popup is anchored at the key's left if the key is in the keyboard's
//! left half, else at its right, and reaches past the key by up to half a row towards the
//! keyboard's middle, as far as the keyboard leaves room for.

/// A rectangle in the keyboard's coordinates, as a `FlorisRect`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Bounds {
    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }
}

/// The first, second and third of a popup's prioritized keys, as `PopupKeys` numbers them: a
/// popup index that isn't one of these is a place in its other keys.
pub const FIRST_PRIORITIZED: i32 = -1;
pub const SECOND_PRIORITIZED: i32 = -2;
pub const THIRD_PRIORITIZED: i32 = -3;

/// How the extended popup's keys are arranged, which is all following the finger needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    /// The popup is anchored at the key's left edge, else at its right.
    pub anchor_left: bool,
    /// How many keys the popup reaches past the key, towards the keyboard's middle.
    pub anchor_offset: i32,
    pub row0_count: i32,
    pub row1_count: i32,
}

/// An extended popup, laid out.
#[derive(Clone, Debug, PartialEq)]
pub struct Extended {
    pub grid: Grid,
    /// The key under the finger when the popup opens, as numbered in the module's doc.
    pub initial_index: i32,
    /// For each key in the popup's order, which of `PopupKeys` it shows (see
    /// [`FIRST_PRIORITIZED`]): the prioritized keys sit nearest the finger.
    pub popup_indices: Vec<i32>,
    pub bounds: Bounds,
}

impl Extended {
    /// The row the key at `ui_index` is in.
    pub fn row(&self, ui_index: i32) -> i32 {
        let row1_count = self.grid.row1_count;
        if ui_index < row1_count && row1_count > 0 {
            1
        } else {
            0
        }
    }
}

/// The extended popup of `count` keys, `prioritized` of them prioritized, for the key at `key`
/// whose preview popup is `base`, on a keyboard `keyboard_width` wide. None if there are no keys.
pub fn extend(
    count: i32,
    prioritized: i32,
    key: Bounds,
    base: Bounds,
    keyboard_width: f32,
) -> Option<Extended> {
    let n = count;
    let anchor_left = key.left < keyboard_width / 2.0;
    let (row1_count, row0_count) = match n {
        ..=0 => return None,
        1..=5 => (0, n),
        _ if n % 2 == 1 => ((n - 1) / 2, (n + 1) / 2),
        _ => (n / 2, n / 2),
    };
    let anchor_offset = anchor_offset(row0_count, anchor_left, key, base, keyboard_width);
    let initial_index = if anchor_left {
        anchor_offset.wrapping_add(row1_count)
    } else {
        (row0_count - 1)
            .wrapping_sub(anchor_offset)
            .wrapping_add(row1_count)
    };
    let popup_indices = popup_indices(n, prioritized, initial_index);
    let bounds = ext_bounds(
        row0_count,
        row1_count,
        anchor_left,
        anchor_offset,
        key,
        base,
    );
    Some(Extended {
        grid: Grid {
            anchor_left,
            anchor_offset,
            row0_count,
            row1_count,
        },
        initial_index,
        popup_indices,
        bounds,
    })
}

/// How many keys the popup reaches past the key: half a row, less what the keyboard has no room
/// for.
fn anchor_offset(
    row0_count: i32,
    anchor_left: bool,
    key: Bounds,
    base: Bounds,
    keyboard_width: f32,
) -> i32 {
    if row0_count <= 1 {
        return 0;
    }
    let mut offset = if row0_count % 2 == 1 {
        (row0_count - 1) / 2
    } else {
        (row0_count / 2) - 1
    };
    let key_popup_diff_x = (key.width() - base.width()) / 2.0;
    let available_space = if anchor_left {
        key.left + key_popup_diff_x
    } else {
        keyboard_width - (key.left + key_popup_diff_x + base.width())
    };
    while offset > 0 {
        if available_space >= offset as f32 * base.width() {
            break;
        }
        offset -= 1;
    }
    offset
}

/// Which of `PopupKeys` each place shows: the prioritized keys at and around the initial place,
/// the others in order around them.
fn popup_indices(n: i32, prioritized: i32, initial: i32) -> Vec<i32> {
    let len = usize::try_from(n).unwrap_or(0);
    let mut indices = vec![0; len];
    let mut set = |at: i32, value: i32| indices[at as usize] = value;
    match prioritized {
        // only one key: use the initial place
        1 => set(initial, FIRST_PRIORITIZED),
        // two keys: the initial place and the one to its right if there is one, else its left
        2 => {
            set(initial, FIRST_PRIORITIZED);
            if initial + 1 < n {
                set(initial + 1, SECOND_PRIORITIZED);
            } else if initial >= 1 {
                set(initial - 1, SECOND_PRIORITIZED);
            }
        }
        // three keys: the initial place and one either side if there is room, else two to the
        // right or two to the left
        3 => {
            set(initial, FIRST_PRIORITIZED);
            if initial + 1 < n && initial >= 1 {
                set(initial + 1, SECOND_PRIORITIZED);
                set(initial - 1, THIRD_PRIORITIZED);
            } else if initial + 2 < n {
                set(initial + 1, SECOND_PRIORITIZED);
                set(initial + 2, THIRD_PRIORITIZED);
            } else if initial >= 2 {
                set(initial - 1, SECOND_PRIORITIZED);
                set(initial - 2, THIRD_PRIORITIZED);
            }
        }
        _ => {}
    }
    let mut offset = 0;
    for (ui_index, index) in (0..).zip(indices.iter_mut()) {
        if *index < 0 {
            offset += 1;
        } else {
            *index = ui_index - offset;
        }
    }
    indices
}

/// Where the popup is drawn: above the key, row 1 above row 0.
fn ext_bounds(
    row0_count: i32,
    row1_count: i32,
    anchor_left: bool,
    anchor_offset: i32,
    key: Bounds,
    base: Bounds,
) -> Bounds {
    let ext_width = row0_count as f32 * base.width();
    let ext_height = if row1_count > 0 {
        base.height() * 0.4 * 2.0
    } else {
        base.height() * 0.4
    };
    let shift = if anchor_left {
        // Kotlin negates the Int before it becomes a Float: no -0.0 for an offset of 0.
        anchor_offset.wrapping_neg() as f32 * base.width()
    } else {
        -ext_width + base.width() + anchor_offset as f32 * base.width()
    };
    let x = ((key.width() - base.width()) / 2.0) + shift + key.left;
    // Kotlin truncates row 1's height to an Int, then takes it from a Float.
    let row1_height = if row1_count > 0 {
        (base.height() * 0.4) as i32
    } else {
        0
    };
    let y = -base.height() - row1_height as f32 + key.bottom;
    Bounds {
        left: x,
        top: y,
        right: x + ext_width,
        bottom: y + ext_height,
    }
}

/// Which key of the popup laid out as `grid` the finger at (`x`, `y`) is on, for the key at
/// `key` whose preview popup is `base`; None if the finger has left the popup.
pub fn pointer(key: Bounds, base: Bounds, grid: Grid, x_event: f32, y_event: f32) -> Option<i32> {
    let base_width = base.width();
    let base_height = base.height();
    let key_popup_diff_x = (key.width() - base_width) / 2.0;
    let x = x_event - key.left;
    let y = y_event - key.top;
    let k_x = x / base_width;
    if y < -base_height || y > 0.9 * base_height {
        return None;
    }
    let Grid {
        anchor_left,
        anchor_offset: ao,
        row0_count: row0,
        row1_count: row1,
    } = grid;
    let k = k_x as i32;
    if anchor_left {
        if x < key_popup_diff_x - ao.wrapping_add(1) as f32 * base_width
            || x > (key_popup_diff_x + row0.wrapping_add(1).wrapping_sub(ao) as f32 * base_width)
        {
            return None;
        }
        Some(if y < 0.0 && row1 > 0 {
            if k_x >= row1.wrapping_sub(ao) as f32 {
                row1.wrapping_sub(1)
            } else if k_x < ao.wrapping_neg() as f32 {
                0
            } else if k_x < 0.0 {
                k.wrapping_sub(1).wrapping_add(ao)
            } else {
                k.wrapping_add(ao)
            }
        } else if k_x >= row0.wrapping_sub(ao) as f32 {
            row1.wrapping_add(row0).wrapping_sub(1)
        } else if k_x < ao.wrapping_neg() as f32 {
            row1
        } else if k_x < 0.0 {
            row1.wrapping_add(k).wrapping_sub(1).wrapping_add(ao)
        } else {
            row1.wrapping_add(k).wrapping_add(ao)
        })
    } else {
        let key_width = key.width();
        if x > key_width - key_popup_diff_x + ao.wrapping_add(1) as f32 * base_width
            || x < (key_width
                - key_popup_diff_x
                - row0.wrapping_add(1).wrapping_sub(ao) as f32 * base_width)
        {
            return None;
        }
        Some(if y < 0.0 && row1 > 0 {
            if k_x >= ao as f32 {
                row1.wrapping_sub(1)
            } else if k_x < row1.wrapping_sub(1).wrapping_sub(ao).wrapping_neg() as f32 {
                0
            } else if k_x < 0.0 {
                row1.wrapping_sub(2).wrapping_add(k).wrapping_sub(ao)
            } else {
                row1.wrapping_sub(1).wrapping_add(k).wrapping_sub(ao)
            }
        } else if k_x >= ao as f32 {
            row1.wrapping_add(row0).wrapping_sub(1)
        } else if k_x < row0.wrapping_sub(1).wrapping_sub(ao).wrapping_neg() as f32 {
            row1
        } else if k_x < 0.0 {
            row1.wrapping_add(row0)
                .wrapping_sub(2)
                .wrapping_add(k)
                .wrapping_sub(ao)
        } else {
            row1.wrapping_add(row0)
                .wrapping_sub(1)
                .wrapping_add(k)
                .wrapping_sub(ao)
        })
    }
}

/// Where the key numbered `index` is in rows of these `sizes`, numbered from the last row back
/// (`getElementOrNull`): its row and its place in it, or None.
pub fn element_at(sizes: &[i32], index: i32) -> Option<(usize, i32)> {
    if index < 0 {
        return None;
    }
    let mut cached = index;
    for (row, &size) in sizes.iter().enumerate().rev() {
        if cached >= size {
            cached = cached.wrapping_sub(size);
        } else {
            return Some((row, cached));
        }
    }
    None
}

#[cfg(test)]
mod tests;
