//! JNI entry points for the text keyboard's geometry, called by `KeysNative` in lib/rust's
//! Kotlin.
//!
//! Rects cross as four Floats each (left, top, right, bottom), and a keyboard's keys as one flat
//! array with the number of keys in each row beside it.

use std::ptr;

use jni_sys::{JNIEnv, jboolean, jclass, jfloat, jfloatArray, jint, jintArray};
use keys::{Bounds, KeyFlex, Rect};

use crate::jni::{self, Env, Result, illegal};

pub(crate) fn rect(&[left, top, right, bottom]: &[f32; 4]) -> Rect {
    Rect {
        left,
        top,
        right,
        bottom,
    }
}

fn floats(rect: Rect) -> [f32; 4] {
    [rect.left, rect.top, rect.right, rect.bottom]
}

/// `items` cut into rows of `sizes` items each, which must cover them exactly.
pub(crate) fn rows<'a, T>(items: &'a [T], sizes: &[i32]) -> Result<Vec<&'a [T]>> {
    let mut rest = items;
    let mut rows = Vec::with_capacity(sizes.len());
    for &size in sizes {
        let size = usize::try_from(size).map_err(|_| illegal("a row has a negative size"))?;
        let (row, after) = rest
            .split_at_checked(size)
            .ok_or_else(|| illegal("the rows have more keys than given"))?;
        rows.push(row);
        rest = after;
    }
    if !rest.is_empty() {
        return Err(illegal("the rows have fewer keys than given"));
    }
    Ok(rows)
}

/// The flex factors of a visible key (see `keys::flex`): shrink, grow and width factor.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeysNative_flex(
    env: *mut JNIEnv,
    _: jclass,
    mode: jint,
    code: jint,
    numeric: jboolean,
) -> jfloatArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let flex = keys::flex(mode, code, numeric != 0);
            env.make(&[flex.shrink, flex.grow, flex.width_factor])
        })
    }
}

/// The desired key's touch and visible bounds (see `keys::desired_key`), eight Floats.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeysNative_desiredKey(
    env: *mut JNIEnv,
    _: jclass,
    mode: jint,
    keyboard_width: jfloat,
    keyboard_height: jfloat,
    row_count: jint,
    row_base_height: jfloat,
    key_margin_h: jfloat,
    key_margin_v: jfloat,
) -> jfloatArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let (touch, visible) = keys::desired_key(
                mode,
                keyboard_width,
                keyboard_height,
                row_count,
                row_base_height,
                key_margin_h,
                key_margin_v,
            );
            env.make(&[floats(touch), floats(visible)].concat())
        })
    }
}

/// Lays out rows of `row_sizes` keys, whose `flex` is three Floats each (width factor, shrink,
/// grow), with the `desired` key's touch and visible bounds (eight Floats): eight Floats a key,
/// its touch and visible bounds, or null if nothing is laid out (see `keys::layout`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeysNative_layout(
    env: *mut JNIEnv,
    _: jclass,
    row_sizes: jintArray,
    flex: jfloatArray,
    desired: jfloatArray,
    keyboard_width: jfloat,
    keyboard_height: jfloat,
    extend_down: jboolean,
) -> jfloatArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let sizes = env.read::<i32>(row_sizes, "rowSizes")?;
            let flex = env.read::<f32>(flex, "flex")?;
            let (flex, rest) = flex.as_chunks::<3>();
            if !rest.is_empty() {
                return Err(illegal("flex needs three values per key"));
            }
            let flex: Vec<KeyFlex> = flex
                .iter()
                .map(|&[width_factor, shrink, grow]| KeyFlex {
                    width_factor,
                    shrink,
                    grow,
                })
                .collect();
            let (touch, visible) = desired_bounds(env, desired)?;
            let laid_out = keys::layout(
                &rows(&flex, &sizes)?,
                touch,
                visible,
                keyboard_width,
                keyboard_height,
                extend_down != 0,
            );
            match laid_out {
                None => Ok(ptr::null_mut()),
                Some(bounds) => {
                    let values: Vec<f32> = bounds
                        .iter()
                        .flat_map(|&Bounds { touch, visible }| [floats(touch), floats(visible)])
                        .flatten()
                        .collect();
                    env.make(&values)
                }
            }
        })
    }
}

fn desired_bounds(env: &Env, desired: jfloatArray) -> Result<(Rect, Rect)> {
    let desired = env.read::<f32>(desired, "desired")?;
    match desired.as_chunks::<4>() {
        ([touch, visible], []) => Ok((rect(touch), rect(visible))),
        _ => Err(illegal("desired needs eight values")),
    }
}

/// The index of the key whose touch bounds (four Floats a key, in rows of `row_sizes`) hold
/// (x, y), or -1 (see `keys::key_at`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeysNative_keyAt(
    env: *mut JNIEnv,
    _: jclass,
    row_sizes: jintArray,
    touch_bounds: jfloatArray,
    x: jfloat,
    y: jfloat,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, -1, |env| {
            let sizes = env.read::<i32>(row_sizes, "rowSizes")?;
            let bounds = env.read::<f32>(touch_bounds, "touchBounds")?;
            let (bounds, rest) = bounds.as_chunks::<4>();
            if !rest.is_empty() {
                return Err(illegal("touchBounds needs four values per key"));
            }
            let bounds: Vec<Rect> = bounds.iter().map(rect).collect();
            let index = keys::key_at(&rows(&bounds, &sizes)?, x, y);
            // The index fits in an Int: Kotlin's array held that many keys.
            Ok(index.map_or(-1, |index| index as jint))
        })
    }
}
