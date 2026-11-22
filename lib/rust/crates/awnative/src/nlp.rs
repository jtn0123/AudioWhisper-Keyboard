//! The NLP session (the `nlp` crate) and its word checks, for `NlpNative`, whose generated entry
//! points call these. JNI transport only: no NLP semantics and no retained JVM references.
use std::sync::{Mutex, MutexGuard};

use suggest::{personal_words::PersonalWords, session::Session};

use wire::typing::event as event_kind;

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

/// The plan for one typing event (`wire::typing`), which the session answers itself, changing
/// the `learner` feeding `store`, the `taps` trail and the `kept` words, and asking `dictionary`
/// (a lent session, or -1 when none is lent) while it learns a word. A closed session throws an
/// `IllegalStateException`, as `call` does.
///
/// Each lock is taken in the order the session's own steps take them: session, learner, taps,
/// kept words, then the learned words and the dictionary only for the moment of each change or
/// lookup. Nothing else locks the dictionary first and then any of these.
pub fn event(
    handle: i64,
    learner: i64,
    store: i64,
    taps: i64,
    kept: i64,
    dictionary: i64,
    fields: &[Vec<u16>],
) -> Result<Vec<Vec<u16>>> {
    use crate::suggest::{tap, word_learner, word_stores};
    if handle == 0 {
        return Err(Error::IllegalState("the NLP session is closed".into()));
    }
    let field = fields
        .first()
        .is_some_and(|kind| *kind == number(event_kind::FIELD));
    let plan = {
        // SAFETY: the wrapper keeps the handle live and serializes close.
        let mut session = unsafe { locked::<nlp::Session>(handle, "NLP session")? };
        let mut learner = word_learner::learner_of(learner)?;
        let (mut taps, mut kept) = (tap::trail(taps)?, word_stores::kept(kept)?);
        let mut lookup = (dictionary != -1).then_some(word_learner::NativeLookup(dictionary));
        let stores = nlp::typing::Stores {
            learner: &mut learner,
            history: word_learner::store_of(store)?,
            taps: &mut taps,
            kept: &mut kept,
            lookup: lookup.as_mut(),
        };
        session.event(fields, stores).map_err(|error| match error {
            nlp::typing::EventError::Wire(message) => illegal(message),
            nlp::typing::EventError::Lookup(error) => error,
        })?
    };
    if field {
        // Another field: what the suggester worked out for the last one no longer holds.
        crate::suggest::letter_odds::forget();
    }
    Ok(plan)
}

/// `value` as a wire number.
fn number(value: i32) -> Vec<u16> {
    value.to_string().encode_utf16().collect()
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
