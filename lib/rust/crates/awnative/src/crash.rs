//! What a crash leaves behind and the crash screen's report (`utilities::crash`) for
//! `CrashNative`, whose generated entry points call these.

use utilities::crash;

use crate::jni::{Result, illegal};

/// `crash::after`.
pub fn after(last: i64, now: i64) -> Result<i32> {
    Ok(crash::after(last, now))
}

/// `crash::same`.
pub fn same(last: &[u16], current: &[u16]) -> Result<bool> {
    Ok(crash::same(last, current))
}

/// `crash::report`; a row of the wrong length is refused.
pub fn report(
    facts: &[Vec<u16>],
    debug: bool,
    names: &[Vec<u16>],
    details: &[Vec<u16>],
) -> Result<Vec<u16>> {
    crash::report(facts, debug, names, details).ok_or_else(|| illegal("a crash report's rows"))
}
