//! JNI entry points for the learned tap aim (`suggest::tap_aim` and `suggest::tap_learning`) on a
//! `TapTrail` handle (see tap.rs), called by `SuggestNative`: the events it learns from, the aimed
//! offsets the decoder hears, the aimed hit test, and the saved text. The text holds numbers only.

use std::ptr;

use jni_sys::{JNIEnv, jboolean, jclass, jfloat, jfloatArray, jint, jintArray, jlong, jstring};
use keys::Rect;
use suggest::tap_aim::{AimKey, Orientation};
use suggest::tap_learning::Taps;

use super::tap::{packed, with_trail};
use crate::handle;
use crate::jni::{boolean, illegal};
use crate::keys::{rect, rows};

/// The screen is in landscape (`true`) or portrait: taps are learned and aimed per orientation.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapLandscape(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
    landscape: jboolean,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe {
        with_trail(env, trail, (), |taps, _| {
            taps.set_orientation(Orientation::of(landscape != 0));
            Ok(())
        })
    }
}

/// `word` was committed as typed (`Taps::kept`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapKept(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
    word: jstring,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe {
        with_trail(env, trail, (), |taps, env| {
            taps.kept(&env.read_string(word, "word")?);
            Ok(())
        })
    }
}

/// One of the events that confirm or throw away the word waiting to teach: 0 a backspace, 1 an
/// undo, 2 a suggestion tapped with nothing being typed, 3 one tapped for a word being typed, 4 a
/// swiped word.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapEvent(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
    event: jint,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe {
        with_trail(env, trail, (), |taps, _| {
            match event {
                0 => taps.backspace(),
                1 => taps.rejected(),
                2 => taps.picked(false),
                3 => taps.picked(true),
                4 => taps.swiped(),
                _ => return Err(illegal("unknown tap event")),
            }
            Ok(())
        })
    }
}

/// Where each character of `word` was tapped, moved by the learned aim if `aimed`
/// (`Taps::offsets_for`), packed as `tapOffsetsFor` packs it.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapAimedOffsetsFor(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
    word: jstring,
    aimed: jboolean,
) -> jfloatArray {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe {
        with_trail(env, trail, ptr::null_mut(), |taps, env| {
            let offsets = taps.offsets_for(&env.read_string(word, "word")?, aimed != 0);
            env.make(&packed(&offsets))
        })
    }
}

/// Which key a tap at (`x`, `y`) types (`TapAim::retarget`). `layout` holds the key the plain hit
/// test found, the number of rows, each row's size, then each key's code; `bounds` holds six values
/// per key: its touch bounds (left, top, right, bottom) and its drawn width and height.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapRetarget(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
    layout: jintArray,
    bounds: jfloatArray,
    x: jfloat,
    y: jfloat,
) -> jint {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe {
        with_trail(env, trail, -1, |taps, env| {
            let layout = env.read::<i32>(layout, "layout")?;
            let bounds = env.read::<f32>(bounds, "bounds")?;
            let (&hit, rest) = layout.split_first().ok_or_else(|| illegal("no hit"))?;
            let (&row_count, rest) = rest.split_first().ok_or_else(|| illegal("no rows"))?;
            let row_count = usize::try_from(row_count).map_err(|_| illegal("negative rows"))?;
            let (row_sizes, codes) = rest
                .split_at_checked(row_count)
                .ok_or_else(|| illegal("fewer row sizes than rows"))?;
            let (bounds, []) = bounds.as_chunks::<6>() else {
                return Err(illegal("bounds needs six values per key"));
            };
            if codes.len() != bounds.len() {
                return Err(illegal("every key needs a code and bounds"));
            }
            let Ok(hit) = usize::try_from(hit) else {
                return Ok(hit);
            };
            let touch: Vec<Rect> = bounds
                .iter()
                .map(|&[left, top, right, bottom, ..]| rect(&[left, top, right, bottom]))
                .collect();
            let rows = rows(&touch, row_sizes)?;
            let keys: Vec<AimKey> = codes
                .iter()
                .zip(bounds)
                .map(|(&code, &[.., width, height])| AimKey {
                    code,
                    width,
                    height,
                })
                .collect();
            let key_at = |x, y| keys::key_at(&rows, x, y);
            // The index fits in an Int: Kotlin's arrays held that many keys.
            Ok(taps.retarget(&keys, hit, (x, y), key_at) as jint)
        })
    }
}

/// How far, in pixels, this user's taps for each of `keys` land from where they mean, if that is
/// known: what the Rust renderer's hit test moves a tap back by (see `keys::aim::retarget`). None
/// for every key without a trail (`trail` 0: the setting is off).
///
/// # Safety
/// `trail` must be 0 or a live trail handle: its Kotlin owner is still reachable.
pub(crate) unsafe fn key_shifts(
    trail: jlong,
    keys: &[render::scene::Key],
) -> crate::jni::Result<Vec<Option<(f32, f32)>>> {
    if trail == 0 {
        return Ok(Vec::new());
    }
    // SAFETY: as the caller promises.
    let taps = unsafe { handle::locked::<Taps>(trail, "tap trail")? };
    let orientation = taps.orientation();
    Ok(keys
        .iter()
        .map(|key| {
            let bias = taps.aim.bias(orientation, key.code)?;
            let size = key.bounds.visible;
            Some((bias.dx * size.width(), bias.dy * size.height()))
        })
        .collect())
}

/// The learned aim as saved text (`TapAim::text`): numbers only.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapAimText(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
) -> jstring {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe {
        with_trail(env, trail, ptr::null_mut(), |taps, env| {
            env.make_string(&taps.aim.text())
        })
    }
}

/// Replaces the learned aim with saved `text` (`Taps::load`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapAimLoad(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
    text: jstring,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe {
        with_trail(env, trail, (), |taps, env| {
            taps.load(&env.read_string(text, "text")?);
            Ok(())
        })
    }
}

/// Erases the learned aim (`Taps::forget`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapAimForget(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe {
        with_trail(env, trail, (), |taps, _| {
            taps.forget();
            Ok(())
        })
    }
}

/// Whether the aim changed since this was last asked (`Taps::take_changed`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_tapAimChanged(
    env: *mut JNIEnv,
    _: jclass,
    trail: jlong,
) -> jboolean {
    // SAFETY: the JVM passes its environment; Kotlin holds the trail while it calls.
    unsafe { with_trail(env, trail, 0, |taps, _| Ok(boolean(taps.take_changed()))) }
}
