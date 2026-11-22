//! The NLP session (the `nlp` crate) and its word checks, for `NlpNative`, whose generated entry
//! points call these. JNI transport only: no NLP semantics and no retained JVM references.
use std::sync::{Mutex, MutexGuard};

use suggest::{personal_words::PersonalWords, session::Session};

use crate::handle::{into_handle, locked, object};
use crate::jni::{Error, Result, illegal};

/// A new session.
pub fn create() -> i64 {
    into_handle(Mutex::new(nlp::Session::default()))
}

pub fn free(handle: i64) -> Result<()> {
    // SAFETY: the wrapper serializes close with calls, zeros its handle and frees exactly once.
    unsafe { crate::handle::free::<Mutex<nlp::Session>>(handle) };
    Ok(())
}

/// `operation`'s answer to `fields`. A closed session throws an `IllegalStateException`, as
/// Kotlin's `check` did.
pub fn call(handle: i64, operation: &[u16], fields: &[Vec<u16>]) -> Result<Vec<Vec<u16>>> {
    if handle == 0 {
        return Err(Error::IllegalState("the NLP session is closed".into()));
    }
    let operation = String::from_utf16(operation).map_err(|_| illegal("invalid NLP operation"))?;
    // SAFETY: the wrapper keeps the handle live and serializes close.
    let mut session = unsafe { locked::<nlp::Session>(handle, "NLP session")? };
    session.checked_call(&operation, fields).map_err(illegal)
}

fn dictionary<'a>(handle: i64) -> Result<MutexGuard<'a, Session>> {
    // SAFETY: the Latin session's wrapper holds its live handle under the same monitor as close.
    unsafe { locked(handle, "session") }
}

fn personal<'a>(handle: i64) -> Result<&'a PersonalWords> {
    // SAFETY: Kotlin keeps the personal snapshot reachable while it calls.
    unsafe { object(handle, "personal words") }
}

/// Whether `word` is a name, by the dictionary behind `dictionary` and the personal words.
/// A closed session throws an `IllegalStateException`, as Kotlin's `check` did.
pub fn is_name(dictionary: i64, personal: i64, word: &[u16]) -> Result<bool> {
    if dictionary == 0 {
        return Err(Error::IllegalState("the session is closed".into()));
    }
    let personal = self::personal(personal)?;
    let dictionary = self::dictionary(dictionary)?;
    Ok(nlp::stores::is_name(word, personal, Some(&*dictionary)))
}

/// Whether `word` is a name by the personal words alone, before a dictionary has loaded.
pub fn is_name_without_dictionary(personal: i64, word: &[u16]) -> Result<bool> {
    Ok(nlp::stores::is_name(word, self::personal(personal)?, None))
}

/// Which of `words` the dictionary doesn't know.
pub fn unknown_words(dictionary: i64, words: &[Vec<u16>]) -> Result<Vec<Vec<u16>>> {
    Ok(nlp::stores::unknown_words(
        &*self::dictionary(dictionary)?,
        words.to_vec(),
    ))
}
