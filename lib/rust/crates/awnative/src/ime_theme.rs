//! The keyboard's theme (the `imetheme` crate), for `ImeThemeNative`, whose generated entry
//! points call these. JNI transport only: rows in, rows out, no retained JVM references.
use std::sync::Mutex;

use imetheme::{calls, session::Session};

use crate::handle::{into_handle, locked};
use crate::jni::{Result, illegal};

/// A new theme session: no theme loaded, the base theme showing.
pub fn create() -> i64 {
    into_handle(Mutex::new(Session::new()))
}

pub fn free(handle: i64) -> Result<()> {
    // SAFETY: the wrapper frees exactly once, when nothing can reach it.
    unsafe { crate::handle::free::<Mutex<Session>>(handle) };
    Ok(())
}

/// `operation`'s answer to `fields` (`wire::ime_theme`).
pub fn call(handle: i64, operation: &[u16], fields: &[Vec<u16>]) -> Result<Vec<Vec<u16>>> {
    // SAFETY: the wrapper keeps the handle live while it calls.
    let mut session = unsafe { locked::<Session>(handle, "theme")? };
    calls::call(&mut session, operation, fields).map_err(|error| illegal(error.0))
}

/// `rule`'s answer to `fields` (`wire::ime_theme::RULES`).
pub fn rules(rule: &[u16], fields: &[Vec<u16>]) -> Result<Vec<Vec<u16>>> {
    calls::rule(rule, fields).map_err(|error| illegal(error.0))
}
