//! JNI entry points for where keys were tapped (`suggest::tap` and `suggest::qwerty`), called by
//! `SuggestNative`. A `TapTrail` is a handle to a trail behind a lock: the keyboard writes it on
//! the main thread and reads it while suggesting.

use std::ptr;
use std::sync::{Mutex, MutexGuard};

use jni_sys::{JNIEnv, jboolean, jclass, jfloat, jfloatArray, jint, jintArray, jlong, jstring};
use suggest::qwerty;
use suggest::tap::{TapOffset, TapTrail};

use crate::handle::{free, into_handle, object};
use crate::jni::{self, Env, Result, illegal};

/// How far beside a key a tap still counts for it (`TapOffset::LIMIT`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapLimit(
    _: *mut JNIEnv,
    _: jclass,
) -> jfloat {
    TapOffset::LIMIT
}

/// How many characters a trail keeps unless told otherwise (`TapTrail::CAPACITY`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapTrailCapacity(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    TapTrail::CAPACITY as jint
}

/// The offset of a tap on a key (`TapOffset::of`): dx and dy, or null for a key with no size.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapOffset(
    env: *mut JNIEnv,
    _: jclass,
    x: jfloat,
    y: jfloat,
    center_x: jfloat,
    center_y: jfloat,
    width: jfloat,
    height: jfloat,
) -> jfloatArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            match TapOffset::of(x, y, center_x, center_y, width, height) {
                Some(offset) => env.make(&[offset.dx, offset.dy]),
                None => Ok(ptr::null_mut()),
            }
        })
    }
}

fn offset(has_offset: jboolean, dx: jfloat, dy: jfloat) -> Option<TapOffset> {
    (has_offset != 0).then_some(TapOffset { dx, dy })
}

/// A new trail of the last `capacity` characters. A negative capacity throws, on purpose: Kotlin's
/// `TapTrail` took one and then threw on the first character typed into it.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_newTapTrail(
    env: *mut JNIEnv,
    _: jclass,
    capacity: jint,
) -> jlong {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |_| {
            let capacity =
                usize::try_from(capacity).map_err(|_| illegal("capacity is negative"))?;
            Ok(into_handle(Mutex::new(TapTrail::new(capacity))))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_freeTapTrail(
    _: *mut JNIEnv,
    _: jclass,
    trail: jlong,
) {
    // SAFETY: the Kotlin side frees each trail once, when nothing can reach it.
    unsafe { free::<Mutex<TapTrail>>(trail) }
}

/// The trail behind `handle`, locked.
///
/// # Safety
/// `handle` must be a trail that hasn't been freed: its Kotlin owner is still reachable.
unsafe fn locked<'a>(handle: jlong) -> Result<MutexGuard<'a, TapTrail>> {
    // SAFETY: as the caller promises.
    let trail = unsafe { object::<Mutex<TapTrail>>(handle, "tap trail")? };
    // A panic aborts, so no lock is ever poisoned; take it back if one somehow were.
    Ok(trail
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()))
}

/// Runs `body` on the trail behind `handle`, locked.
///
/// # Safety
/// `env` must be the environment the JVM passed to this call, and `handle` a trail that hasn't been
/// freed.
unsafe fn with_trail<T>(
    env: *mut JNIEnv,
    handle: jlong,
    fallback: T,
    body: impl FnOnce(&mut TapTrail, &Env) -> Result<T>,
) -> T {
    // SAFETY: as the caller promises.
    unsafe { jni::run(env, fallback, |env| body(&mut *locked(handle)?, env)) }
}

/// A key for `codePoint` went down, at (`dx`, `dy`) if `hasOffset`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapPressed(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
    code_point: jint,
    has_offset: jboolean,
    dx: jfloat,
    dy: jfloat,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe {
        with_trail(env, trail, (), |trail, _| {
            trail.pressed(code_point, offset(has_offset, dx, dy));
            Ok(())
        })
    }
}

/// `text` went into the editor.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapTyped(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
    text: jstring,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe {
        with_trail(env, trail, (), |trail, env| {
            trail.typed(&env.read_string(text, "text")?);
            Ok(())
        })
    }
}

/// The last typed character was deleted.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapDeleted(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe {
        with_trail(env, trail, (), |trail, _| {
            trail.deleted();
            Ok(())
        })
    }
}

/// Everything typed is forgotten.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapCleared(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe {
        with_trail(env, trail, (), |trail, _| {
            trail.clear();
            Ok(())
        })
    }
}

/// Where each character of `word` was tapped (`TapTrail::offsets_for`): three values per
/// character, 1 if it was tapped and 0 if not, then dx and dy; empty if the trail doesn't spell it.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapOffsetsFor(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
    word: jstring,
) -> jfloatArray {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe {
        with_trail(env, trail, ptr::null_mut(), |trail, env| {
            let offsets = trail.offsets_for(&env.read_string(word, "word")?);
            let packed: Vec<f32> = offsets
                .iter()
                .flat_map(|offset| match offset {
                    Some(offset) => [1.0, offset.dx, offset.dy],
                    None => [0.0, 0.0, 0.0],
                })
                .collect();
            env.make(&packed)
        })
    }
}

/// The QWERTY layout's sizes: a key's width and height, the keyboard's, the proximity grid's
/// columns and rows, and how many keys a grid cell lists at most.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_qwertySizes(
    env: *mut JNIEnv,
    _: jclass,
) -> jintArray {
    let sizes = [
        qwerty::KEY_WIDTH,
        qwerty::KEY_HEIGHT,
        qwerty::KEYBOARD_WIDTH,
        qwerty::KEYBOARD_HEIGHT,
        qwerty::GRID_WIDTH,
        qwerty::GRID_HEIGHT,
        qwerty::MAX_PROXIMITY_CHARS as jint,
    ];
    // SAFETY: the JVM passes its environment for this call.
    unsafe { jni::run(env, ptr::null_mut(), |env| env.make(&sizes)) }
}

/// The QWERTY layout's keys: code point, left and top for each.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_qwertyKeys(
    env: *mut JNIEnv,
    _: jclass,
) -> jintArray {
    let keys: Vec<i32> = qwerty::keys()
        .flat_map(|key| [key.code_point, key.x, key.y])
        .collect();
    // SAFETY: the JVM passes its environment for this call.
    unsafe { jni::run(env, ptr::null_mut(), |env| env.make(&keys)) }
}

/// The centre of the key for `codePoint` (x, y), or null.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_keyCenter(
    env: *mut JNIEnv,
    _: jclass,
    code_point: jint,
) -> jintArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            match qwerty::key_center(code_point) {
                Some((x, y)) => env.make(&[x, y]),
                None => Ok(ptr::null_mut()),
            }
        })
    }
}

/// Where a tap at (`dx`, `dy`), if `hasOffset`, on the key for `codePoint` lands (x, y), or null.
/// An offset of NaN throws, as Kotlin's `roundToInt` did.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_touchPoint(
    env: *mut JNIEnv,
    _: jclass,
    code_point: jint,
    has_offset: jboolean,
    dx: jfloat,
    dy: jfloat,
) -> jintArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let point = qwerty::touch_point(code_point, offset(has_offset, dx, dy))
                .map_err(|_| illegal("Cannot round NaN value."))?;
            match point {
                Some((x, y)) => env.make(&[x, y]),
                None => Ok(ptr::null_mut()),
            }
        })
    }
}

/// The proximity grid (`qwerty::proximity_grid`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_proximityGrid(
    env: *mut JNIEnv,
    _: jclass,
) -> jintArray {
    let grid = qwerty::proximity_grid();
    // SAFETY: the JVM passes its environment for this call.
    unsafe { jni::run(env, ptr::null_mut(), |env| env.make(&grid)) }
}
