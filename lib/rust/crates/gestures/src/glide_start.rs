//! Whether a touch is the start of a glide across the keys, as FlorisBoard's
//! `GlideTypingGesture.Detector` decided: the first finger down is followed, and once it has gone
//! further than a key's width, faster than [`VELOCITY_THRESHOLD`], from a key that has swipes of
//! its own, its points are a glide; if it hasn't by [`MAX_DETECT_TIME`], they never are.
//!
//! Kotlin still reads the touch events, keeps the points for the glide's listeners and calls them;
//! this says which points to keep, which to hand on and when. Positions are in pixels, the key's
//! size in dp, and the clock in milliseconds.

/// After this long, in milliseconds, a touch that isn't a glide yet never will be.
pub const MAX_DETECT_TIME: i64 = 500;

/// A glide moves faster than this, in dp a millisecond.
pub const VELOCITY_THRESHOLD: f64 = 0.10;

/// Touches that start on these keys aren't glides: delete, shift, space and the CJK space have
/// swipes of their own (`KeyCode.DELETE`, `SHIFT`, `SPACE`, `CJK_SPACE`).
pub const SWIPE_GESTURE_KEYS: [i32; 4] = [-7, -11, 32, 12288];

/// No pointer followed (`Pointer.UNUSED_P`).
const NO_POINTER: i32 = -1;

/// Android's `MotionEvent` actions, masked, that the detector tells apart.
pub mod action {
    pub const DOWN: i32 = 0;
    pub const UP: i32 = 1;
    pub const MOVE: i32 = 2;
    pub const CANCEL: i32 = 3;
    pub const POINTER_DOWN: i32 = 5;
    pub const POINTER_UP: i32 = 6;
}

/// A touch event, as the detector reads it.
#[derive(Clone, Copy, Debug)]
pub enum Touch<'a> {
    /// A finger went down at (x, y); `first` if it is the first of the touch (`ACTION_DOWN`).
    Down {
        first: bool,
        id: i32,
        at: (f32, f32),
    },
    /// The finger at the event's action index moved, through `points` (x, y, x, y, …): its
    /// history, oldest first, then where it is now.
    Move { id: i32, points: &'a [f32] },
    /// A finger was lifted.
    Up { id: i32 },
    /// The touch was cancelled.
    Cancel,
    /// Anything else, such as hovering.
    Other,
}

impl<'a> Touch<'a> {
    /// The touch of an event with this masked `action`, whose action index has pointer `id`, and
    /// whose points for that pointer are `points` (x, y, …): for a down, where it went down; for a
    /// move, its history then where it is now. Fails for a down without a point or an odd count.
    pub fn of(action: i32, id: i32, points: &'a [f32]) -> Result<Touch<'a>, &'static str> {
        if !points.len().is_multiple_of(2) {
            return Err("points must be x, y pairs");
        }
        Ok(match action {
            action::DOWN | action::POINTER_DOWN => match points {
                [x, y, ..] => Touch::Down {
                    first: action == action::DOWN,
                    id,
                    at: (*x, *y),
                },
                _ => return Err("a down needs its point"),
            },
            action::MOVE => Touch::Move { id, points },
            action::UP | action::POINTER_UP => Touch::Up { id },
            action::CANCEL => Touch::Cancel,
            _ => Touch::Other,
        })
    }
}

/// What a touch event made of the glide, in the order Kotlin acts on it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    /// The event is part of a glide (what `onTouchEvent` returned).
    pub gesture: bool,
    /// The kept points were cleared first: a new touch began.
    pub cleared_before: bool,
    /// The event's points were added to those kept.
    pub took: bool,
    /// The glide's clock started now.
    pub started: bool,
    /// The kept points from this index on go to the listeners (`onGlideAddPoint`), in order.
    pub emit_from: Option<usize>,
    /// The glide is complete (`onGlideComplete`, with the kept points).
    pub complete: bool,
    /// The glide was cancelled (`onGlideCancelled`).
    pub cancelled: bool,
    /// The kept points were cleared last.
    pub cleared_after: bool,
    /// Whether the touch is a glide, once decided (`PointerData.isActuallyGesture`).
    pub decision: Option<bool>,
}

/// The detector, between events.
#[derive(Debug)]
pub struct GlideStart {
    /// A key's width, in dp.
    key_size: f32,
    /// The pointer followed, or [`NO_POINTER`].
    pointer_id: i32,
    /// The first point kept, in pixels.
    first: (f32, f32),
    /// How many points are kept.
    count: usize,
    start_time: i64,
    decision: Option<bool>,
}

/// Kotlin's `Position.dist`: `sqrt((dx).pow(2) + (dy).pow(2))`, each step through a `Double` and
/// back. A `Float` squared is exact in a `Double`, so `pow(2)` is the product.
fn distance((x1, y1): (f32, f32), (x2, y2): (f32, f32)) -> f32 {
    let square = |d: f32| (d as f64 * d as f64) as f32;
    ((square(x2 - x1) + square(y2 - y1)) as f64).sqrt() as f32
}

impl GlideStart {
    /// A detector for keys `key_size` dp wide.
    pub fn new(key_size: f32) -> GlideStart {
        GlideStart {
            key_size,
            pointer_id: NO_POINTER,
            first: (0.0, 0.0),
            count: 0,
            start_time: 0,
            decision: None,
        }
    }

    fn reset(&mut self) {
        *self = GlideStart::new(self.key_size);
    }

    /// Keeps a point.
    fn take(&mut self, point: (f32, f32)) {
        if self.count == 0 {
            self.first = point;
        }
        self.count += 1;
    }

    /// One touch event at `now`, with the screen's `density`, from a touch that began on the key
    /// with code `initial_key`, if any.
    pub fn touch(
        &mut self,
        touch: Touch,
        now: i64,
        density: f32,
        initial_key: Option<i32>,
    ) -> Outcome {
        let mut outcome = Outcome::default();
        match touch {
            Touch::Down { first, id, at } => {
                if first {
                    self.reset();
                    outcome.cleared_before = true;
                }
                // A second finger is ignored.
                if self.pointer_id == NO_POINTER {
                    self.pointer_id = id;
                    self.take(at);
                    self.start_time = now;
                    outcome.took = true;
                    outcome.started = true;
                }
            }
            Touch::Move { id, points } if id == self.pointer_id => {
                outcome.took = true;
                outcome.emit_from = self.moved(points, now, density, initial_key);
                outcome.gesture = self.decision == Some(true);
            }
            Touch::Up { id } if id == self.pointer_id => {
                outcome.complete = self.decision == Some(true);
                self.reset();
                outcome.cleared_after = true;
            }
            Touch::Cancel => {
                outcome.cancelled = self.decision == Some(true);
                self.reset();
                outcome.cleared_after = true;
            }
            Touch::Move { .. } | Touch::Up { .. } | Touch::Other => {}
        }
        outcome.decision = self.decision;
        outcome
    }

    /// The followed finger moved through `points`: keeps them, decides whether this is a glide if
    /// it isn't decided, and says from which kept point on the listeners hear of them.
    fn moved(
        &mut self,
        points: &[f32],
        now: i64,
        density: f32,
        initial_key: Option<i32>,
    ) -> Option<usize> {
        let mut emit_from = None;
        for &[x, y] in points.as_chunks::<2>().0 {
            let point = (x, y);
            self.take(point);
            if self.decision.is_none() {
                // `ViewUtils.px2dp`.
                let dist = distance(self.first, point) / density;
                // Kotlin's Long arithmetic wraps; `Float / Long` divides by the Long as a Float.
                let time = now.wrapping_sub(self.start_time).wrapping_add(1);
                let swipe_key = initial_key.is_some_and(|code| SWIPE_GESTURE_KEYS.contains(&code));
                if dist > self.key_size
                    && (dist / time as f32) as f64 > VELOCITY_THRESHOLD
                    && !swipe_key
                {
                    self.decision = Some(true);
                    // Every point kept so far, this one included.
                    emit_from = Some(0);
                } else if time > MAX_DETECT_TIME {
                    self.decision = Some(false);
                }
            }
            if self.decision == Some(true) && emit_from.is_none() {
                emit_from = Some(self.count - 1);
            }
        }
        emit_from
    }
}

#[cfg(test)]
mod tests;
