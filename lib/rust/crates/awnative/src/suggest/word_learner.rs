//! What the keyboard learns from each finished word (`suggest::word_learner`), for
//! `SuggesterNative` and `NativeLearnerNative`, whose generated entry points call these. A learner
//! is a handle to its state behind a lock; each call names the store of learned words it feeds,
//! which it locks only to add or remove a word, and the dictionary to ask (a session handle, or -1
//! while none is open).

use std::sync::{Mutex, MutexGuard};

use suggest::dictionary::Lookup;
use suggest::learned_words::LearnedWords;
use suggest::session::Session;
use suggest::word_learner::WordLearner;

use crate::handle::{free, into_handle, locked, object};
use crate::jni::Result;
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
use {
    super::{boolean, legacy_suggester::JavaLookup},
    crate::jni::{self, Env},
    jni_sys::{JNIEnv, jboolean, jclass, jint, jlong, jobject, jstring},
};

/// The session's lock stays held for this whole exchange. Lookup remains lazy: a closed session
/// fails only if the same spelling rule would have asked the old callback.
struct NativeLookup(i64);

impl Lookup for NativeLookup {
    type Error = crate::jni::Error;
    fn probability(&mut self, word: &[u16]) -> Result<i32> {
        // SAFETY: the session's owner lends this handle under its lock, or 0 after close.
        unsafe { Ok(locked::<Session>(self.0, "session")?.probability(word)) }
    }
}

fn learner_of<'a>(learner: i64) -> Result<MutexGuard<'a, WordLearner>> {
    // SAFETY: Kotlin holds the learner while it calls, so it hasn't been freed.
    unsafe { locked(learner, "word learner") }
}

fn store_of<'a>(store: i64) -> Result<&'a Mutex<LearnedWords>> {
    // SAFETY: the learner's Kotlin owner keeps its store reachable while it calls.
    unsafe { object(store, "learned words") }
}

/// `word`, which starts at `start` right after `text_before`, is finished
/// (`WordLearner::finished`), or was swiped in (`WordLearner::swiped`); `dictionary` is the
/// session to ask, or -1 for none. Returns whether the store changed.
pub fn call(
    learner: i64,
    store: i64,
    word: &[u16],
    text_before: &[u16],
    start: i32,
    swiped: bool,
    dictionary: i64,
) -> Result<bool> {
    let (mut learner, store) = (learner_of(learner)?, store_of(store)?);
    let mut lookup = (dictionary != -1).then_some(NativeLookup(dictionary));
    if swiped {
        learner.swiped(store, lookup.as_mut(), word, text_before)
    } else {
        learner.finished(store, lookup.as_mut(), word, text_before, start)
    }
}

/// A new learner, which has learned nothing yet.
pub fn new_learner() -> i64 {
    into_handle(Mutex::new(WordLearner::default()))
}

pub fn free_learner(learner: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each learner once, when nothing can reach it.
    unsafe { free::<Mutex<WordLearner>>(learner) };
    Ok(())
}

/// The swiped word waiting to be kept was replaced or deleted (`WordLearner::drop_swiped`).
pub fn learner_drop_swiped(learner: i64) -> Result<()> {
    learner_of(learner)?.drop_swiped();
    Ok(())
}

/// The auto-correction that starts at `start` was undone (`WordLearner::undo`). Returns whether
/// the store changed.
pub fn learner_undo(learner: i64, store: i64, start: i32) -> Result<bool> {
    let (mut learner, store) = (learner_of(learner)?, store_of(store)?);
    Ok(learner.undo(store, start))
}

/// Another field (`WordLearner::reset`).
pub fn learner_reset(learner: i64) -> Result<()> {
    learner_of(learner)?.reset();
    Ok(())
}

/// The dictionary to ask, if one is open (`lookup` isn't null).
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
fn lookup(env: &Env, lookup: jobject) -> Result<Option<JavaLookup<'_>>> {
    if lookup.is_null() {
        return Ok(None);
    }
    JavaLookup::new(env, lookup, "lookup").map(Some)
}

/// `word`, which starts at `start` right after `textBefore`, is finished
/// (`WordLearner::finished`). Returns whether the store changed.
#[unsafe(no_mangle)]
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LegacyFixtureNative_learnerFinished(
    env: *mut JNIEnv,
    _: jclass,
    learner: jlong,
    store: jlong,
    dictionary: jobject,
    word: jstring,
    text_before: jstring,
    start: jint,
) -> jboolean {
    // SAFETY: the JVM passes its environment.
    unsafe {
        jni::run(env, 0, |env| {
            let (mut learner, store) = (learner_of(learner)?, store_of(store)?);
            let word = env.read_string(word, "word")?;
            let text_before = env.read_string(text_before, "textBefore")?;
            let mut lookup = lookup(env, dictionary)?;
            let changed = learner.finished(store, lookup.as_mut(), &word, &text_before, start)?;
            Ok(boolean(changed))
        })
    }
}

/// A swiped `word` went in after `textBefore` (`WordLearner::swiped`). Returns whether the store
/// changed.
#[unsafe(no_mangle)]
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LegacyFixtureNative_learnerSwiped(
    env: *mut JNIEnv,
    _: jclass,
    learner: jlong,
    store: jlong,
    dictionary: jobject,
    word: jstring,
    text_before: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment.
    unsafe {
        jni::run(env, 0, |env| {
            let (mut learner, store) = (learner_of(learner)?, store_of(store)?);
            let word = env.read_string(word, "word")?;
            let text_before = env.read_string(text_before, "textBefore")?;
            let mut lookup = lookup(env, dictionary)?;
            let changed = learner.swiped(store, lookup.as_mut(), &word, &text_before)?;
            Ok(boolean(changed))
        })
    }
}
