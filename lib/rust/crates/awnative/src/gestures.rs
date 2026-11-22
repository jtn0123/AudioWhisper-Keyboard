//! Swipes and the start of a glide (crates/gestures), for `GesturesNative`, whose generated
//! entry points call these. Both detectors are handles, called on every touch move.
//!
//! A swipe comes back as an `IntArray`, or null for none: its direction, as `SwipeDirection`
//! orders them, then its units moved since the finger went down and since the last swipe (x, y,
//! x, y). A glide's touch comes back as one `Long` of bits, which `GlideOutcome` reads:
//! [`glide_outcome`] lays them out.

use std::sync::{Mutex, MutexGuard};

use gestures::glide_start::{GlideStart, Outcome, Touch};
use gestures::swipe::{Direction, Swipe, Swipes};

use crate::boundary::args::{GesturesNativeGlideTouch, GesturesNativeSwipesUp};
use crate::handle::{into_handle, locked};
use crate::jni::{Result, illegal};

/// A direction's number, as `SwipeDirection` orders them.
fn direction(direction: Direction) -> i32 {
    match direction {
        Direction::UpLeft => 0,
        Direction::Up => 1,
        Direction::UpRight => 2,
        Direction::Right => 3,
        Direction::DownRight => 4,
        Direction::Down => 5,
        Direction::DownLeft => 6,
        Direction::Left => 7,
    }
}

/// A swipe as `Swipe` reads it, if there was one.
fn swipe(swipe: Option<Swipe>) -> Option<Vec<i32>> {
    swipe.map(|swipe| {
        vec![
            direction(swipe.direction),
            swipe.abs_x,
            swipe.abs_y,
            swipe.rel_x,
            swipe.rel_y,
        ]
    })
}

/// A new swipe detector, following no finger.
pub fn new_swipes() -> i64 {
    into_handle(Mutex::new(Swipes::default()))
}

pub fn free_swipes(swipes: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each detector once, when nothing can reach it.
    unsafe { crate::handle::free::<Mutex<Swipes>>(swipes) };
    Ok(())
}

fn swipes<'a>(swipes: i64) -> Result<MutexGuard<'a, Swipes>> {
    // SAFETY: Kotlin holds the detector while it calls, so it hasn't been freed.
    unsafe { locked(swipes, "swipe detector") }
}

/// A new touch began: forget every finger (`Swipes::reset`).
pub fn swipes_reset(swipes: i64) -> Result<()> {
    self::swipes(swipes)?.reset();
    Ok(())
}

/// Finger `id` went down at (`x`, `y`) pixels (`Swipes::down`).
pub fn swipes_down(swipes: i64, id: i32, x: f32, y: f32, density: f32) -> Result<()> {
    self::swipes(swipes)?.down(id, x, y, density);
    Ok(())
}

/// Finger `id` moved to (`x`, `y`) pixels (`Swipes::moved`): the swipe that makes, if any.
pub fn swipes_move(
    swipes: i64,
    id: i32,
    x: f32,
    y: f32,
    density: f32,
    distance_threshold: i32,
    always: bool,
) -> Result<Option<Vec<i32>>> {
    let found = self::swipes(swipes)?.moved(id, x, y, density, distance_threshold, always);
    Ok(swipe(found))
}

/// Finger `id` was lifted at (`x`, `y`) pixels, moving at (`velocity_x`, `velocity_y`) pixels a
/// second (`Swipes::up`): the swipe that makes, if any.
pub fn swipes_up(args: GesturesNativeSwipesUp) -> Result<Option<Vec<i32>>> {
    let found = self::swipes(args.swipes)?.up(
        args.id,
        (args.x, args.y),
        (args.velocity_x, args.velocity_y),
        args.density,
        args.distance_threshold,
        args.velocity_threshold,
    );
    Ok(swipe(found))
}

/// Finger `id`'s touch was cancelled (`Swipes::cancel`).
pub fn swipes_cancel(swipes: i64, id: i32) -> Result<()> {
    self::swipes(swipes)?.cancel(id);
    Ok(())
}

/// A new glide detector for keys `key_size` dp wide.
pub fn new_glide_start(key_size: f32) -> i64 {
    into_handle(Mutex::new(GlideStart::new(key_size)))
}

pub fn free_glide_start(glide: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each detector once, when nothing can reach it.
    unsafe { crate::handle::free::<Mutex<GlideStart>>(glide) };
    Ok(())
}

/// An outcome as `GlideOutcome` reads it: bit 0 a glide, 1 cleared before, 2 took the points,
/// 3 started the clock, 4 complete, 5 cancelled, 6 cleared after; bits 8 and 9 the decision
/// (0 none, 1 a glide, 2 not one); and the high 32 bits the first point to hand on, or -1 for
/// none.
fn glide_outcome(outcome: Outcome) -> i64 {
    let flags = [
        outcome.gesture,
        outcome.cleared_before,
        outcome.took,
        outcome.started,
        outcome.complete,
        outcome.cancelled,
        outcome.cleared_after,
    ];
    let mut bits = flags
        .iter()
        .enumerate()
        .fold(0i64, |bits, (i, &flag)| bits | (i64::from(flag) << i));
    bits |= match outcome.decision {
        None => 0,
        Some(true) => 1 << 8,
        Some(false) => 2 << 8,
    };
    // A point's index fits in an Int: it numbers a Java array.
    let emit = outcome.emit_from.map_or(-1, |from| from as i32);
    bits | (i64::from(emit) << 32)
}

/// One touch event (`Touch::of`, `GlideStart::touch`): its masked `action`, the pointer `id` at
/// its action index, and that pointer's first `count` floats of `points` (x, y, …). The touch
/// began on the key with `key_code` if `has_key`.
pub fn glide_touch(args: GesturesNativeGlideTouch) -> Result<i64> {
    let points = usize::try_from(args.count)
        .ok()
        .and_then(|count| args.points.get(..count))
        .ok_or_else(|| illegal("count"))?;
    let touch = Touch::of(args.action, args.id, points).map_err(illegal)?;
    let key = args.has_key.then_some(args.key_code);
    // SAFETY: Kotlin holds the detector while it calls, so it hasn't been freed.
    let mut glide = unsafe { locked::<GlideStart>(args.glide, "glide detector")? };
    Ok(glide_outcome(glide.touch(
        touch,
        args.now,
        args.density,
        key,
    )))
}
