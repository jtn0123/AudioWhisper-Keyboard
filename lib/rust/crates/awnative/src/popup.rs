//! JNI entry points for a key's long-press popup (the `popup` crate), called by `PopupNative` in
//! lib/rust's Kotlin.
//!
//! Rectangles cross as their four floats, left, top, right and bottom. Results come back packed:
//!
//! - an extended popup as an `int[]`: whether it is anchored left (1 or 0), its anchor offset,
//!   row 0's and row 1's counts, the initial index, its bounds' four floats as their bits, then
//!   each place's popup index and then each place's row; empty if there is no popup;
//! - the key under the finger as a `long`: the index, or `Long.MIN_VALUE` once the finger is off
//!   the popup;
//! - an element's place as a `long`: its row in the high half and its place in the row in the
//!   low, or -1 if there is none;
//! - a set's order as an `int[]`: the hint, how many keys are prioritized, how many others (or
//!   -1, -2 and -3 for the set's own list of relevant keys, symbol popups or number popups), then
//!   the prioritized keys and the others. A key is 0 for main, 1 and 2 for the symbol and number
//!   hints, then from 3 the relevant keys, the symbol hint's popup keys and the number hint's,
//!   one after the other; the hint is -1 if none;
//! - a merge as an `int[]` of steps: the index of a key to add (the number of relevant keys for
//!   the merged set's main), or -1 where its main becomes the set's.

use std::ptr;

use jni_sys::jlong;
use jni_sys::{JNI_FALSE, JNIEnv, jboolean, jbooleanArray, jclass, jfloat, jint, jintArray};
use popup::grid::{self, Bounds, Grid};
use popup::order::{self, Contents, HintConfig, HintMode, Key, Other, Step};

use crate::jni::{self, Result};

/// What `pointer` gives once the finger is off the popup.
const OFF_THE_POPUP: jlong = jlong::MIN;

/// A count from Kotlin, which can't be negative.
fn count(count: jint, name: &str) -> Result<usize> {
    usize::try_from(count).map_err(|_| jni::illegal(format!("{name} is negative")))
}

/// The hint mode numbered as `KeyHintMode` orders them.
fn mode(number: jint) -> Result<HintMode> {
    match number {
        0 => Ok(HintMode::Disabled),
        1 => Ok(HintMode::HintPriority),
        2 => Ok(HintMode::AccentPriority),
        3 => Ok(HintMode::SmartPriority),
        _ => Err(jni::illegal("hint mode")),
    }
}

/// A rectangle from its four sides.
fn bounds(left: jfloat, top: jfloat, right: jfloat, bottom: jfloat) -> Bounds {
    Bounds {
        left,
        top,
        right,
        bottom,
    }
}

/// The extended popup of `count` keys, `prioritized` of them prioritized, for the key at `key_*`
/// whose preview popup is `base_*` (`popup::grid::extend`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PopupNative_extend(
    env: *mut JNIEnv,
    _: jclass,
    count: jint,
    prioritized: jint,
    key_left: jfloat,
    key_top: jfloat,
    key_right: jfloat,
    key_bottom: jfloat,
    base_left: jfloat,
    base_top: jfloat,
    base_right: jfloat,
    base_bottom: jfloat,
    keyboard_width: jfloat,
) -> jintArray {
    let key = bounds(key_left, key_top, key_right, key_bottom);
    let base = bounds(base_left, base_top, base_right, base_bottom);
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let Some(ext) = grid::extend(count, prioritized, key, base, keyboard_width) else {
                return env.make::<i32>(&[]);
            };
            let g = ext.grid;
            let b = ext.bounds;
            let mut packed = vec![
                i32::from(g.anchor_left),
                g.anchor_offset,
                g.row0_count,
                g.row1_count,
                ext.initial_index,
            ];
            packed.extend([b.left, b.top, b.right, b.bottom].map(|f| f.to_bits() as i32));
            packed.extend(&ext.popup_indices);
            packed.extend((0..count).map(|ui_index| ext.row(ui_index)));
            env.make(&packed)
        })
    }
}

/// Which key of the popup laid out as `anchor_left`, `anchor_offset`, `row0_count` and
/// `row1_count` the finger at (`x`, `y`) is on (`popup::grid::pointer`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PopupNative_pointer(
    _: *mut JNIEnv,
    _: jclass,
    key_left: jfloat,
    key_top: jfloat,
    key_right: jfloat,
    key_bottom: jfloat,
    base_left: jfloat,
    base_top: jfloat,
    base_right: jfloat,
    base_bottom: jfloat,
    anchor_left: jboolean,
    anchor_offset: jint,
    row0_count: jint,
    row1_count: jint,
    x: jfloat,
    y: jfloat,
) -> jlong {
    let key = bounds(key_left, key_top, key_right, key_bottom);
    let base = bounds(base_left, base_top, base_right, base_bottom);
    let grid = Grid {
        anchor_left: anchor_left != JNI_FALSE,
        anchor_offset,
        row0_count,
        row1_count,
    };
    grid::pointer(key, base, grid, x, y).map_or(OFF_THE_POPUP, jlong::from)
}

/// Where element `index` is in rows of these `sizes` (`popup::grid::element_at`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PopupNative_elementAt(
    env: *mut JNIEnv,
    _: jclass,
    sizes: jintArray,
    index: jint,
) -> jlong {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, -1, |env| {
            let sizes: Vec<i32> = env.read(sizes, "sizes")?;
            Ok(grid::element_at(&sizes, index).map_or(-1, |(row, at)| {
                ((row as jlong) << 32) | jlong::from(at as u32)
            }))
        })
    }
}

/// A key's number, as the module's doc says.
fn key_number(key: Key, contents: &Contents) -> i32 {
    let r = contents.relevant;
    let s = contents.symbol_popups;
    let number = match key {
        Key::Main => 0,
        Key::SymbolHint => 1,
        Key::NumberHint => 2,
        Key::Relevant(at) => 3 + at,
        Key::SymbolPopup(at) => 3 + r + at,
        Key::NumberPopup(at) => 3 + r + s + at,
    };
    number as i32
}

/// The order of a set's keys for the hint settings (`popup::order::order`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PopupNative_order(
    env: *mut JNIEnv,
    _: jclass,
    main: jboolean,
    relevant: jint,
    symbol_hint: jboolean,
    number_hint: jboolean,
    symbol_popups: jint,
    number_popups: jint,
    symbol_mode: jint,
    number_mode: jint,
    merge_hint_popups: jboolean,
) -> jintArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let contents = Contents {
                main: main != JNI_FALSE,
                relevant: count(relevant, "relevant")?,
                symbol_hint: symbol_hint != JNI_FALSE,
                number_hint: number_hint != JNI_FALSE,
                symbol_popups: count(symbol_popups, "symbolPopups")?,
                number_popups: count(number_popups, "numberPopups")?,
            };
            let config = HintConfig {
                symbol: mode(symbol_mode)?,
                number: mode(number_mode)?,
                merge_hint_popups: merge_hint_popups != JNI_FALSE,
            };
            let order = order::order(&contents, config);
            let number = |key: &Key| key_number(*key, &contents);
            let (other_count, others): (i32, &[Key]) = match &order.other {
                Other::Relevant => (-1, &[]),
                Other::SymbolPopups => (-2, &[]),
                Other::NumberPopups => (-3, &[]),
                Other::Keys(keys) => (keys.len() as i32, keys),
            };
            let mut packed = vec![
                order.hint.as_ref().map_or(-1, number),
                order.prioritized.len() as i32,
                other_count,
            ];
            packed.extend(order.prioritized.iter().map(number));
            packed.extend(others.iter().map(number));
            env.make(&packed)
        })
    }
}

/// Where a merged set's keys go (`popup::order::merge`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PopupNative_merge(
    env: *mut JNIEnv,
    _: jclass,
    computed: jbooleanArray,
    main_computed: jboolean,
    use_main: jboolean,
    has_main: jboolean,
) -> jintArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let computed: Vec<jboolean> = env.read(computed, "computed")?;
            let computed: Vec<bool> = computed.iter().map(|&c| c != JNI_FALSE).collect();
            let steps = order::merge(
                &computed,
                main_computed != JNI_FALSE,
                use_main != JNI_FALSE,
                has_main != JNI_FALSE,
            );
            let packed: Vec<i32> = steps
                .iter()
                .map(|step| match step {
                    Step::Add(at) => *at as i32,
                    Step::SetMain => -1,
                })
                .collect();
            env.make(&packed)
        })
    }
}
