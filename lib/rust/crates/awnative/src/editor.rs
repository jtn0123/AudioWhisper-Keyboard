//! The editor's session and attribute queries (crates/editor), for `EditorNative`, whose
//! generated entry points call these. No editor semantics here, and no JVM references kept.

use std::sync::Mutex;

use crate::handle::{into_handle, locked};
use crate::jni::{Result, illegal};

/// A new session.
pub fn create() -> i64 {
    into_handle(Mutex::new(editor::Session::new()))
}

pub fn free(session: i64) -> Result<()> {
    // SAFETY: the wrapper synchronizes close with calls, zeros its handle and frees exactly once.
    unsafe { crate::handle::free::<Mutex<editor::Session>>(session) };
    Ok(())
}

/// The answer to `operation` with these `fields` (`editor::Session::checked_call`).
pub fn call(handle: i64, operation: &[u16], fields: &[Vec<u16>]) -> Result<Vec<Vec<u16>>> {
    let operation =
        String::from_utf16(operation).map_err(|_| illegal("invalid editor operation"))?;
    // SAFETY: the wrapper keeps the handle live and serializes close with calls.
    let mut session = unsafe { locked::<editor::Session>(handle, "editor")? };
    session.checked_call(&operation, fields).map_err(illegal)
}

/// The answer to an attribute query: its `EditorAttributeOp` code, then its values.
pub fn attributes(fields: &[Vec<u16>]) -> Result<Vec<Vec<u16>>> {
    if fields.len() < 2 || (editor::integer(fields, 0) == 6 && fields.len() < 4) {
        return Err(illegal("wrong editor attribute field count"));
    }
    Ok(editor::attributes::call(fields))
}
