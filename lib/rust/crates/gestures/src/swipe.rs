//! Swipes, as FlorisBoard's `SwipeGesture.Detector` found them: which way each finger moved and by
//! how many units of a quarter of the swipe distance, both while it moves (the space bar's cursor
//! and the delete key's swipes) and when it is lifted (the swipe actions). Kotlin still reads the
//! touch events and Android's `VelocityTracker`, and acts on each swipe.
//!
//! Positions and velocities come in pixels with the screen's density, as Kotlin had them before
//! `ViewUtils.px2dp`; the thresholds are the preferences' `Int`s.

use crate::java_math::{atan2, to_degrees};

/// How many fingers are followed at once, as Kotlin's `PointerMap` held: a fifth is ignored.
pub const POINTERS: usize = 4;

/// The direction of a swipe, ordered as Kotlin's `SwipeGesture.Direction`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    UpLeft,
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
}

/// A swipe found, without its pointer id and its type, which the caller knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Swipe {
    pub direction: Direction,
    /// Units moved since the finger went down.
    pub abs_x: i32,
    pub abs_y: i32,
    /// Units moved since the last swipe reported while moving; on lifting, the same as `abs_*`.
    pub rel_x: i32,
    pub rel_y: i32,
}

/// One of `PointerMap`'s slots: a `GesturePointer`. Unused while its id is negative.
#[derive(Clone, Copy, Debug)]
struct Pointer {
    id: i32,
    first_x: f32,
    first_y: f32,
    last_x: f32,
    last_y: f32,
    abs_x: i32,
    abs_y: i32,
}

impl Pointer {
    const UNUSED: Pointer = Pointer {
        id: -1,
        first_x: 0.0,
        first_y: 0.0,
        last_x: 0.0,
        last_y: 0.0,
        abs_x: 0,
        abs_y: 0,
    };
}

/// The swipe detector: the fingers it follows, in `PointerMap`'s slots.
///
/// It keeps `PointerMap`'s quirks, since Kotlin's results depend on them: a slot is taken without
/// clearing what it held, an id is looked up among unused slots too (so a negative id finds one),
/// and lifting a finger writes its unit counts into the slot after freeing it, so the next finger
/// to take that slot, before all are lifted, counts its first units from them.
#[derive(Debug)]
pub struct Swipes {
    pointers: [Pointer; POINTERS],
}

impl Default for Swipes {
    fn default() -> Self {
        Swipes {
            pointers: [Pointer::UNUSED; POINTERS],
        }
    }
}

/// `ViewUtils.px2dp`.
fn dp(px: f32, density: f32) -> f32 {
    px / density
}

/// The swipe distance preference in dp, as `Int.dp.value.toDouble()` reads it: through a `Float`.
fn threshold_width(distance_threshold: i32) -> f64 {
    distance_threshold as f32 as f64
}

/// Kotlin's `Double.toInt()`: NaN is 0, and out-of-range values are clamped, as Rust's `as` does.
fn units(diff: f32, unit_width: f64) -> i32 {
    (diff as f64 / unit_width) as i32
}

impl Swipes {
    /// A new touch began (`ACTION_DOWN`): forget every finger.
    pub fn reset(&mut self) {
        self.pointers = [Pointer::UNUSED; POINTERS];
    }

    /// `PointerMap.findById`: the first slot with this id, used or not.
    fn find(&mut self, id: i32) -> Option<&mut Pointer> {
        self.pointers.iter_mut().find(|pointer| pointer.id == id)
    }

    /// `PointerMap.removeById`: frees the first slot with this id.
    fn remove(&mut self, id: i32) {
        if let Some(pointer) = self.find(id) {
            *pointer = Pointer::UNUSED;
        }
    }

    /// Finger `id` went down at (`x`, `y`) pixels. Ignored if four are already followed.
    pub fn down(&mut self, id: i32, x: f32, y: f32, density: f32) {
        // `PointerMap.add`: the first unused slot, whatever it still holds.
        if let Some(pointer) = self.pointers.iter_mut().find(|pointer| pointer.id < 0) {
            pointer.id = id;
            pointer.first_x = dp(x, density);
            pointer.first_y = dp(y, density);
            pointer.last_x = pointer.first_x;
            pointer.last_y = pointer.first_y;
        }
    }

    /// Finger `id` moved to (`x`, `y`) pixels: a swipe once it has moved more than half the swipe
    /// distance (dp) along either axis since the last one, or at every move if `always`.
    pub fn moved(
        &mut self,
        id: i32,
        x: f32,
        y: f32,
        density: f32,
        distance_threshold: i32,
        always: bool,
    ) -> Option<Swipe> {
        let pointer = self.find(id)?;
        let current_x = dp(x, density);
        let current_y = dp(y, density);
        let abs_diff_x = current_x - pointer.first_x;
        let abs_diff_y = current_y - pointer.first_y;
        let rel_diff_x = current_x - pointer.last_x;
        let rel_diff_y = current_y - pointer.last_y;
        let threshold_width = threshold_width(distance_threshold);
        let unit_width = threshold_width / 4.0;
        if !(always
            || rel_diff_x.abs() as f64 > threshold_width / 2.0
            || rel_diff_y.abs() as f64 > threshold_width / 2.0)
        {
            return None;
        }
        pointer.last_x = current_x;
        pointer.last_y = current_y;
        let direction = direction(rel_diff_x as f64, rel_diff_y as f64);
        let abs_x = units(abs_diff_x, unit_width);
        let abs_y = units(abs_diff_y, unit_width);
        // Kotlin's Int difference wraps.
        let rel_x = abs_x.wrapping_sub(pointer.abs_x);
        let rel_y = abs_y.wrapping_sub(pointer.abs_y);
        pointer.abs_x = abs_x;
        pointer.abs_y = abs_y;
        Some(Swipe {
            direction,
            abs_x,
            abs_y,
            rel_x,
            rel_y,
        })
    }

    /// Finger `id` was lifted at (x, y) pixels, moving at (x, y) pixels a second: a swipe if it
    /// went further than the swipe distance (dp) along either axis and faster than the swipe
    /// velocity (dp a second) along either. The finger is forgotten.
    pub fn up(
        &mut self,
        id: i32,
        (x, y): (f32, f32),
        (velocity_x, velocity_y): (f32, f32),
        density: f32,
        distance_threshold: i32,
        velocity_threshold: i32,
    ) -> Option<Swipe> {
        let pointer = self.find(id)?;
        let current_x = dp(x, density);
        let current_y = dp(y, density);
        let abs_diff_x = current_x - pointer.first_x;
        let abs_diff_y = current_y - pointer.first_y;
        let velocity_x = dp(velocity_x, density);
        let velocity_y = dp(velocity_y, density);
        // Freed before the counts are written into it, as Kotlin did.
        *pointer = Pointer::UNUSED;
        let threshold_speed = velocity_threshold as f64;
        let threshold_width = threshold_width(distance_threshold);
        let unit_width = threshold_width / 4.0;
        if !((abs_diff_x.abs() as f64 > threshold_width
            || abs_diff_y.abs() as f64 > threshold_width)
            && (velocity_x.abs() as f64 > threshold_speed
                || velocity_y.abs() as f64 > threshold_speed))
        {
            return None;
        }
        let direction = direction(abs_diff_x as f64, abs_diff_y as f64);
        pointer.abs_x = units(abs_diff_x, unit_width);
        pointer.abs_y = units(abs_diff_y, unit_width);
        Some(Swipe {
            direction,
            abs_x: pointer.abs_x,
            abs_y: pointer.abs_y,
            rel_x: pointer.abs_x,
            rel_y: pointer.abs_y,
        })
    }

    /// Finger `id`'s touch was cancelled: forget it.
    pub fn cancel(&mut self, id: i32) {
        self.remove(id);
    }
}

/// The angle of (`diff_x`, `diff_y`) in degrees from 0 to 360, clockwise from +x, since Android's
/// y grows downwards.
fn angle(diff_x: f64, diff_y: f64) -> f64 {
    (to_degrees(atan2(diff_y, diff_x)) + 360.0) % 360.0
}

/// Which of eight sectors, each an eighth of the circle centred on its direction, the angle of
/// (`diff_x`, `diff_y`) falls in. A sector takes its lower edge; NaN is [`Direction::Right`].
pub fn direction(diff_x: f64, diff_y: f64) -> Direction {
    let diff_angle = angle(diff_x, diff_y) / 360.0;
    const SECTORS: [Direction; 7] = [
        Direction::DownRight,
        Direction::Down,
        Direction::DownLeft,
        Direction::Left,
        Direction::UpLeft,
        Direction::Up,
        Direction::UpRight,
    ];
    for (i, direction) in SECTORS.into_iter().enumerate() {
        let low = (1 + 2 * i) as f64 / 16.0;
        let high = (3 + 2 * i) as f64 / 16.0;
        if diff_angle >= low && diff_angle < high {
            return direction;
        }
    }
    Direction::Right
}

#[cfg(test)]
mod tests;
