//! One dictation's ordered effects (`dictation::session`), for `DictationSessionNative`, whose
//! generated entry points call these. Effects run after returning to Kotlin, outside the lock.

use std::sync::{Mutex, MutexGuard};

use dictation::session::Session;

use crate::handle::{into_handle, locked};
use crate::jni::{Result, illegal};

/// A new session.
pub fn create() -> i64 {
    into_handle(Mutex::new(Session::new()))
}

pub fn free(session: i64) -> Result<()> {
    // SAFETY: the synchronized wrapper frees its handle once, then sets it to zero.
    unsafe { crate::handle::free::<Mutex<Session>>(session) };
    Ok(())
}

/// The session behind `handle`, locked.
fn session<'a>(handle: i64) -> Result<MutexGuard<'a, Session>> {
    // SAFETY: Kotlin holds the session while it calls, so it hasn't been freed.
    unsafe { locked(handle, "dictation session") }
}

/// The effects of `operation` at `now` for listening generation `id` (`Session::call`).
pub fn call(
    handle: i64,
    operation: &[u16],
    now: i64,
    id: i32,
    fields: &[Vec<u16>],
) -> Result<Vec<Vec<u16>>> {
    let operation = String::from_utf16(operation).map_err(|_| illegal("invalid operation"))?;
    session(handle)?
        .call(&operation, now, id, fields)
        .map_err(illegal)
}

/// Listening generation `id` detected speech, or lost it (`Session::speech`).
pub fn speech(handle: i64, id: i32, detected: bool) -> Result<()> {
    session(handle)?.speech(id, detected);
    Ok(())
}

/// The model cache's answer to a `DictationModelOp` (`Session::model`).
pub fn model(handle: i64, operation: &[u16]) -> Result<i32> {
    let operation =
        String::from_utf16(operation).map_err(|_| illegal("invalid model operation"))?;
    session(handle)?.model(&operation).map_err(illegal)
}

/// Whether the caller may run the live preview now (`Session::claim_preview`).
pub fn claim_preview(handle: i64) -> Result<bool> {
    Ok(session(handle)?.claim_preview())
}

/// The claimed live preview has finished (`Session::complete_preview`).
pub fn complete_preview(handle: i64) -> Result<()> {
    session(handle)?.complete_preview();
    Ok(())
}

/// When to release an idle model (`dictation::session::release_at`).
pub fn release_at(idle: i64, now: i64, shown: bool) -> i64 {
    dictation::session::release_at(idle, now, shown)
}

/// The problem to report as a dictation ends (`dictation::session::ending`).
pub fn ending(typed: bool, failures: i32) -> Vec<u16> {
    dictation::session::ending(typed, failures)
        .encode_utf16()
        .collect()
}
