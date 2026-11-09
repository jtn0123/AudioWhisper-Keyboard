//! JNI entry points for swipes and the start of a glide (crates/gestures), called by
//! `GesturesNative` in lib/rust's Kotlin. Both detectors are handles, called on every touch move.
//!
//! A swipe comes back in an `IntArray` of [`SWIPE_INTS`]: its direction, as `SwipeDirection`
//! orders them, then its units moved since the finger went down and since the last swipe (x, y,
//! x, y). A glide's touch comes back as one `Long` of bits, which `GlideOutcome` reads:
//! [`glide_outcome`] lays them out.

use std::sync::Mutex;

use gestures::glide_start::{GlideStart, Outcome, Touch};
use gestures::swipe::{Direction, Swipe, Swipes};
use jni_sys::{JNI_FALSE, JNIEnv, jboolean, jclass, jfloat, jfloatArray, jint, jintArray, jlong};

use crate::handle::{free, into_handle, with_locked};
use crate::jni::{self, Env, Result};

/// How many `Int`s a swipe takes in its array.
const SWIPE_INTS: usize = 5;

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

/// Writes a swipe, if there was one, into `out`: whether there was.
fn swipe(env: &Env, swipe: Option<Swipe>, out: jintArray) -> Result<jboolean> {
    let Some(swipe) = swipe else {
        return Ok(JNI_FALSE);
    };
    let ints: [i32; SWIPE_INTS] = [
        direction(swipe.direction),
        swipe.abs_x,
        swipe.abs_y,
        swipe.rel_x,
        swipe.rel_y,
    ];
    env.write(out, &ints, "out")?;
    Ok(jni::boolean(true))
}

/// A new swipe detector, following no finger.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GesturesNative_newSwipes(
    _: *mut JNIEnv,
    _: jclass,
) -> jlong {
    into_handle(Mutex::new(Swipes::default()))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GesturesNative_freeSwipes(
    _: *mut JNIEnv,
    _: jclass,
    swipes: jlong,
) {
    // SAFETY: the Kotlin side frees each detector once, when nothing can reach it.
    unsafe { free::<Mutex<Swipes>>(swipes) }
}

/// A new touch began: forget every finger (`Swipes::reset`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GesturesNative_swipesReset(
    env: *mut JNIEnv,
    _: jclass,
    swipes: jlong,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the detector while it calls.
    unsafe {
        with_locked(
            env,
            swipes,
            "swipe detector",
            (),
            |swipes: &mut Swipes, _| {
                swipes.reset();
                Ok(())
            },
        )
    }
}

/// Finger `id` went down at (`x`, `y`) pixels (`Swipes::down`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GesturesNative_swipesDown(
    env: *mut JNIEnv,
    _: jclass,
    swipes: jlong,
    id: jint,
    x: jfloat,
    y: jfloat,
    density: jfloat,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the detector while it calls.
    unsafe {
        with_locked(
            env,
            swipes,
            "swipe detector",
            (),
            |swipes: &mut Swipes, _| {
                swipes.down(id, x, y, density);
                Ok(())
            },
        )
    }
}

/// Finger `id` moved to (`x`, `y`) pixels (`Swipes::moved`): whether that is a swipe, written
/// into `out`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GesturesNative_swipesMove(
    env: *mut JNIEnv,
    _: jclass,
    swipes: jlong,
    id: jint,
    x: jfloat,
    y: jfloat,
    density: jfloat,
    distance_threshold: jint,
    always: jboolean,
    out: jintArray,
) -> jboolean {
    // SAFETY: the JVM passes its environment; Kotlin holds the detector while it calls.
    unsafe {
        with_locked(
            env,
            swipes,
            "swipe detector",
            JNI_FALSE,
            |swipes: &mut Swipes, env| {
                let found =
                    swipes.moved(id, x, y, density, distance_threshold, always != JNI_FALSE);
                swipe(env, found, out)
            },
        )
    }
}

/// Finger `id` was lifted at (`x`, `y`) pixels, moving at (`velocity_x`, `velocity_y`) pixels a
/// second (`Swipes::up`): whether that is a swipe, written into `out`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GesturesNative_swipesUp(
    env: *mut JNIEnv,
    _: jclass,
    swipes: jlong,
    id: jint,
    x: jfloat,
    y: jfloat,
    velocity_x: jfloat,
    velocity_y: jfloat,
    density: jfloat,
    distance_threshold: jint,
    velocity_threshold: jint,
    out: jintArray,
) -> jboolean {
    // SAFETY: the JVM passes its environment; Kotlin holds the detector while it calls.
    unsafe {
        with_locked(
            env,
            swipes,
            "swipe detector",
            JNI_FALSE,
            |swipes: &mut Swipes, env| {
                let found = swipes.up(
                    id,
                    (x, y),
                    (velocity_x, velocity_y),
                    density,
                    distance_threshold,
                    velocity_threshold,
                );
                swipe(env, found, out)
            },
        )
    }
}

/// Finger `id`'s touch was cancelled (`Swipes::cancel`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GesturesNative_swipesCancel(
    env: *mut JNIEnv,
    _: jclass,
    swipes: jlong,
    id: jint,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the detector while it calls.
    unsafe {
        with_locked(
            env,
            swipes,
            "swipe detector",
            (),
            |swipes: &mut Swipes, _| {
                swipes.cancel(id);
                Ok(())
            },
        )
    }
}

/// A new glide detector for keys `key_size` dp wide.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GesturesNative_newGlideStart(
    _: *mut JNIEnv,
    _: jclass,
    key_size: jfloat,
) -> jlong {
    into_handle(Mutex::new(GlideStart::new(key_size)))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GesturesNative_freeGlideStart(
    _: *mut JNIEnv,
    _: jclass,
    glide: jlong,
) {
    // SAFETY: the Kotlin side frees each detector once, when nothing can reach it.
    unsafe { free::<Mutex<GlideStart>>(glide) }
}

/// An outcome as `GlideOutcome` reads it: bit 0 a glide, 1 cleared before, 2 took the points,
/// 3 started the clock, 4 complete, 5 cancelled, 6 cleared after; bits 8 and 9 the decision
/// (0 none, 1 a glide, 2 not one); and the high 32 bits one more than the first point to hand on,
/// or 0 for none.
fn glide_outcome(outcome: Outcome) -> jlong {
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
    let emit = outcome.emit_from.map_or(0, |from| from as i64 + 1);
    bits | (emit << 32)
}

/// One touch event (`Touch::of`, `GlideStart::touch`): its masked `action`, the pointer `id` at
/// its action index, and that pointer's first `count` floats of `points` (x, y, …). The touch
/// began on the key with `key_code` if `has_key`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GesturesNative_glideTouch(
    env: *mut JNIEnv,
    _: jclass,
    glide: jlong,
    action: jint,
    id: jint,
    points: jfloatArray,
    count: jint,
    now: jlong,
    density: jfloat,
    has_key: jboolean,
    key_code: jint,
) -> jlong {
    // SAFETY: the JVM passes its environment; Kotlin holds the detector while it calls.
    unsafe {
        with_locked(
            env,
            glide,
            "glide detector",
            0,
            |glide: &mut GlideStart, env| {
                let points = env.read::<f32>(points, "points")?;
                let points = usize::try_from(count)
                    .ok()
                    .and_then(|count| points.get(..count))
                    .ok_or_else(|| jni::illegal("count"))?;
                let touch = Touch::of(action, id, points).map_err(jni::illegal)?;
                let key = (has_key != JNI_FALSE).then_some(key_code);
                Ok(glide_outcome(glide.touch(touch, now, density, key)))
            },
        )
    }
}
