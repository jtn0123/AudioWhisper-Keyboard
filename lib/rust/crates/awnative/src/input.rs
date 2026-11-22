//! The key lifecycle and the rules around it (`input`), for `InputRules` and `InputNative`, whose
//! generated entry points call these. No Java object survives a call and no callback holds a
//! native lock.

use std::sync::{Mutex, MutexGuard};

use input::lifecycle::Dispatcher;

use crate::handle::{into_handle, locked};
use crate::jni::{Error, Result, illegal};

/// A new key lifecycle.
pub fn create() -> i64 {
    into_handle(Mutex::new(Dispatcher::default()))
}

pub fn free(handle: i64) -> Result<()> {
    // SAFETY: the synchronized wrapper zeros its handle on close and frees it exactly once.
    unsafe { crate::handle::free::<Mutex<Dispatcher>>(handle) };
    Ok(())
}

/// The lifecycle behind `handle`, locked. A closed one throws an `IllegalStateException`, as
/// Kotlin's `check` did.
fn dispatcher<'a>(handle: i64) -> Result<MutexGuard<'a, Dispatcher>> {
    if handle == 0 {
        return Err(Error::IllegalState("the input dispatcher is closed".into()));
    }
    // SAFETY: the synchronized wrapper keeps the handle live and serializes close.
    unsafe { locked(handle, "input dispatcher") }
}

/// The lifecycle's answer to `operation` with `fields`.
pub fn call(handle: i64, operation: i32, fields: &[i64]) -> Result<Vec<i64>> {
    dispatcher(handle)?.call(operation, fields).map_err(illegal)
}

/// A state, subtype or space decision (`input::control::plan`).
pub fn control(fields: &[i64], text: &[u16]) -> Result<Vec<i64>> {
    input::control::plan(fields, text).map_err(illegal)
}

/// Where a key's event goes (`input::routing::route`).
pub fn route(fields: &[i64]) -> Result<Vec<i64>> {
    input::routing::route(fields).map_err(illegal)
}

/// A key's sound and haptic feedback (`input::feedback::plan`).
pub fn feedback(fields: &[i64]) -> Result<Vec<i64>> {
    input::feedback::plan(fields).map_err(illegal)
}

/// A key's label, from its kind, code and mode, or its icon, from its code, raw input type, input
/// mode, options and mode.
pub fn visual(fields: &[i64]) -> Result<i32> {
    match *fields {
        [kind, code, mode] => Ok(input::visual::label(kind as i32, code as i32, mode as i32)),
        [code, raw, input, options, mode] => Ok(input::visual::icon(
            code as i32,
            raw as u64,
            input as i32,
            options as i32,
            mode as i32,
        )),
        _ => Err(illegal("wrong visual field count")),
    }
}

/// What holding the key `code` does (`input::pointer::hold`).
pub fn hold(code: i32, popup: bool, uninterrupted: bool, space_action: bool) -> i32 {
    input::pointer::hold(code, popup, uninterrupted, space_action)
}

/// A pointer rule, chosen by how many `fields` there are.
pub fn pointer(fields: &[f32]) -> Result<bool> {
    Ok(match *fields {
        [code] => input::pointer::roll_release(code as i32),
        [code, still] => input::pointer::rolls(code as i32, still != 0.0),
        [x, y, left, top, right, bottom] => input::pointer::slide(x, y, left, top, right, bottom),
        _ => return Err(illegal("wrong pointer field count")),
    })
}
