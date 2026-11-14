//! JNI entry points for what the keyboard learns from each finished word
//! (`suggest::word_learner`), called by `SuggesterNative`. A learner is a handle to its state
//! behind a lock; each call names the store of learned words it feeds, which it locks only to add
//! or remove a word, and the dictionary to ask (a `WordProbabilities`, or null while none is open).

use std::sync::Mutex;

use jni_sys::{JNIEnv, jboolean, jclass, jint, jlong, jstring};
use suggest::learned_words::LearnedWords;
use suggest::word_learner::WordLearner;

use super::boolean;
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
use super::legacy_suggester::JavaLookup;
use crate::handle::{free, into_handle, object, with_locked};
use crate::jni::{Env, Result};
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
use jni_sys::jobject;
use suggest::{dictionary::Lookup, session::Session};

/// The managed session monitor stays held for this whole exchange. Lookup remains lazy: a
/// closed session fails only if the same spelling rule would have asked the old callback.
struct NativeLookup(jlong);
impl Lookup for NativeLookup {
    type Error = crate::jni::Error;
    fn probability(&mut self, word: &[u16]) -> Result<i32> {
        // SAFETY: the synchronized session owner keeps this handle live, or passes zero after close.
        unsafe { Ok(crate::handle::locked::<Session>(self.0, "session")?.probability(word)) }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_NativeLearnerNative_call(
    env: *mut JNIEnv,
    _: jclass,
    learner: jlong,
    store: jlong,
    session: jlong,
    word: jstring,
    text_before: jstring,
    start: jint,
    swiped: jboolean,
) -> jboolean {
    // SAFETY: the learner/store owners stay reachable and the session monitor serializes close.
    unsafe {
        with_learner(env, learner, store, 0, |learner, store, env| {
            let word = env.read_string(word, "word")?;
            let text_before = env.read_string(text_before, "textBefore")?;
            let mut lookup = (session != -1).then_some(NativeLookup(session));
            let changed = if swiped != 0 {
                learner.swiped(store, lookup.as_mut(), &word, &text_before)?
            } else {
                learner.finished(store, lookup.as_mut(), &word, &text_before, start)?
            };
            Ok(boolean(changed))
        })
    }
}

/// A new learner, which has learned nothing yet.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggesterNative_newLearner(
    _: *mut JNIEnv,
    _: jclass,
) -> jlong {
    into_handle(Mutex::new(WordLearner::default()))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggesterNative_freeLearner(
    _: *mut JNIEnv,
    _: jclass,
    learner: jlong,
) {
    // SAFETY: the Kotlin side frees each learner once, when nothing can reach it.
    unsafe { free::<Mutex<WordLearner>>(learner) }
}

/// Runs `body` on the learner behind `learner`, locked, with the store behind `store`.
///
/// # Safety
/// `env` must be the environment the JVM passed to this call, `learner` a learner and `store` a
/// store of learned words, neither freed: their Kotlin owners are still reachable.
unsafe fn with_learner<T>(
    env: *mut JNIEnv,
    learner: jlong,
    store: jlong,
    fallback: T,
    body: impl FnOnce(&mut WordLearner, &Mutex<LearnedWords>, &Env) -> Result<T>,
) -> T {
    // SAFETY: as the caller promises.
    unsafe {
        with_locked(env, learner, "word learner", fallback, |learner, env| {
            let store = object::<Mutex<LearnedWords>>(store, "learned words")?;
            body(learner, store, env)
        })
    }
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
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggesterNative_learnerFinished(
    env: *mut JNIEnv,
    _: jclass,
    learner: jlong,
    store: jlong,
    dictionary: jobject,
    word: jstring,
    text_before: jstring,
    start: jint,
) -> jboolean {
    // SAFETY: the JVM passes its environment; Kotlin holds the learner and its store.
    unsafe {
        with_learner(env, learner, store, 0, |learner, store, env| {
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
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggesterNative_learnerSwiped(
    env: *mut JNIEnv,
    _: jclass,
    learner: jlong,
    store: jlong,
    dictionary: jobject,
    word: jstring,
    text_before: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment; Kotlin holds the learner and its store.
    unsafe {
        with_learner(env, learner, store, 0, |learner, store, env| {
            let word = env.read_string(word, "word")?;
            let text_before = env.read_string(text_before, "textBefore")?;
            let mut lookup = lookup(env, dictionary)?;
            let changed = learner.swiped(store, lookup.as_mut(), &word, &text_before)?;
            Ok(boolean(changed))
        })
    }
}

/// The swiped word waiting to be kept was replaced or deleted (`WordLearner::drop_swiped`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggesterNative_learnerDropSwiped(
    env: *mut JNIEnv,
    _: jclass,
    learner: jlong,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the learner.
    unsafe {
        with_locked(
            env,
            learner,
            "word learner",
            (),
            |learner: &mut WordLearner, _| {
                learner.drop_swiped();
                Ok(())
            },
        )
    }
}

/// The auto-correction that starts at `start` was undone (`WordLearner::undo`). Returns whether
/// the store changed.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggesterNative_learnerUndo(
    env: *mut JNIEnv,
    _: jclass,
    learner: jlong,
    store: jlong,
    start: jint,
) -> jboolean {
    // SAFETY: the JVM passes its environment; Kotlin holds the learner and its store.
    unsafe {
        with_learner(env, learner, store, 0, |learner, store, _| {
            Ok(boolean(learner.undo(store, start)))
        })
    }
}

/// Another field (`WordLearner::reset`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggesterNative_learnerReset(
    env: *mut JNIEnv,
    _: jclass,
    learner: jlong,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the learner.
    unsafe {
        with_locked(
            env,
            learner,
            "word learner",
            (),
            |learner: &mut WordLearner, _| {
                learner.reset();
                Ok(())
            },
        )
    }
}
