//! JNI entry points for the decisions a walk of the screen and following along make (the
//! `reader` crate's `screen`), called by `ScreenNative` in lib/rust's Kotlin. A scroller is its
//! index in the walk's list of them, and -1 is none. Two boxes cross as one `int[8]`: a box's
//! left, top, right and bottom, then the visible box's.

use std::ptr;

use jni_sys::{JNI_FALSE, JNIEnv, jboolean, jbooleanArray, jclass, jint, jintArray, jlong};
use jni_sys::{jlongArray, jstring};
use reader::screen::{self, AfterTurn, Bounds, Move};

use crate::jni::{self, Env, Result, boolean, illegal};

/// No scroller.
const NONE: jint = -1;

/// The scroller `value` names among `count`: `None` for -1, else one of them.
fn scroller(value: jint, count: usize, name: &str) -> Result<Option<usize>> {
    if value == NONE {
        return Ok(None);
    }
    (usize::try_from(value).ok().filter(|&it| it < count))
        .map(Some)
        .ok_or_else(|| illegal(format!("{name} {value} is not one of {count} scrollers")))
}

/// The scrollers `values` name among `count`.
fn scrollers(values: &[jint], count: usize, name: &str) -> Result<Vec<Option<usize>>> {
    values.iter().map(|&it| scroller(it, count, name)).collect()
}

/// Each scroller's parent, which must come before it, so a walk up its parents ends.
fn parents(env: &Env, parents: jintArray) -> Result<Vec<Option<usize>>> {
    let values = env.read::<i32>(parents, "parents")?;
    (values.iter().enumerate())
        .map(|(index, &it)| scroller(it, index, "a parent"))
        .collect()
}

fn index(value: Option<usize>) -> jint {
    value.map_or(NONE, |it| it as jint)
}

/// A box and the visible one, from `boxes`.
fn read_boxes(env: &Env, boxes: jintArray) -> Result<(Bounds, Bounds)> {
    let values = env.read::<i32>(boxes, "boxes")?;
    let [left, top, right, bottom, v_left, v_top, v_right, v_bottom] = values[..] else {
        return Err(illegal(format!("{} sides, not 8", values.len())));
    };
    let bounds = |left, top, right, bottom| Bounds {
        left,
        top,
        right,
        bottom,
    };
    Ok((
        bounds(left, top, right, bottom),
        bounds(v_left, v_top, v_right, v_bottom),
    ))
}

/// Whether a scrollable view goes sideways (`screen::is_sideways`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ScreenNative_isSideways(
    env: *mut JNIEnv,
    _: jclass,
    rows: jint,
    columns: jint,
    up_down: jboolean,
    across: jboolean,
    class_name: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, JNI_FALSE, |env| {
            let name = env.read_string(class_name, "className")?;
            let sideways = screen::is_sideways(rows, columns, up_down != 0, across != 0, &name);
            Ok(boolean(sideways))
        })
    }
}

/// How much of the first box shows in the second, in square pixels (`Bounds::visible_area`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ScreenNative_visibleArea(
    env: *mut JNIEnv,
    _: jclass,
    boxes: jintArray,
) -> jlong {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            let (bounds, visible) = read_boxes(env, boxes)?;
            Ok(bounds.visible_area(visible))
        })
    }
}

/// The part of the first box that shows in the second, or all of the second
/// (`Bounds::visible_part`): its left, top, right and bottom.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ScreenNative_visiblePart(
    env: *mut JNIEnv,
    _: jclass,
    boxes: jintArray,
) -> jintArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let (bounds, visible) = read_boxes(env, boxes)?;
            let part = bounds.visible_part(visible);
            env.make(&[part.left, part.top, part.right, part.bottom])
        })
    }
}

/// The list among scrollers that show `areas`, given the scroller each piece of text is directly
/// in (`screen::main_scroller`), or -1.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ScreenNative_mainScroller(
    env: *mut JNIEnv,
    _: jclass,
    piece_scrollers: jintArray,
    areas: jlongArray,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, NONE, |env| {
            let areas = env.read::<i64>(areas, "areas")?;
            let values = env.read::<i32>(piece_scrollers, "pieceScrollers")?;
            let pieces = scrollers(&values, areas.len(), "a piece's scroller")?;
            Ok(index(screen::main_scroller(&pieces, &areas)))
        })
    }
}

/// For each piece of text, whether the scroller it is directly in is `main` or inside it
/// (`screen::is_within`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ScreenNative_inList(
    env: *mut JNIEnv,
    _: jclass,
    piece_scrollers: jintArray,
    parents_of: jintArray,
    main: jint,
) -> jbooleanArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let parents = parents(env, parents_of)?;
            let main = scroller(main, parents.len(), "main")?;
            let values = env.read::<i32>(piece_scrollers, "pieceScrollers")?;
            let pieces = scrollers(&values, parents.len(), "a piece's scroller")?;
            let within: Vec<jboolean> = (pieces.iter())
                .map(|&it| boolean(screen::is_within(&parents, it, main)))
                .collect();
            env.make(&within)
        })
    }
}

/// How to bring a web page's piece into view (`screen::move_into_view`), as `Move`'s order in
/// lib/rust's Kotlin has it.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ScreenNative_moveIntoView(
    _: *mut JNIEnv,
    _: jclass,
    first: jint,
    last: jint,
    top: jint,
    bottom: jint,
    can_turn: jboolean,
) -> jint {
    match screen::move_into_view(first, last, top, bottom, can_turn != 0) {
        Move::Stay => 0,
        Move::ShowOnScreen => 1,
        Move::TurnPages => 2,
    }
}

/// What to do once a web page has turned (`screen::after_turn`), as `AfterTurn`'s order in
/// lib/rust's Kotlin has it.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ScreenNative_afterTurn(
    _: *mut JNIEnv,
    _: jclass,
    first: jint,
    last: jint,
    top: jint,
    bottom: jint,
) -> jint {
    match screen::after_turn(first, last, top, bottom) {
        AfterTurn::ShowOnScreen => 0,
        AfterTurn::Done => 1,
        AfterTurn::TurnAgain => 2,
    }
}

/// Whether an app's own view is well inside the view (`screen::is_inside`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ScreenNative_isInside(
    _: *mut JNIEnv,
    _: jclass,
    top: jint,
    bottom: jint,
    view_top: jint,
    view_bottom: jint,
) -> jboolean {
    boolean(screen::is_inside(top, bottom, view_top, view_bottom))
}
