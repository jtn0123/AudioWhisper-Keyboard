//! JNI entry points for the personal dictionaries' words (`suggest::personal_words`), called by
//! `WordStoresNative`. A snapshot never changes once made, so its handle needs no lock.

use std::ptr;

use jni_sys::{JNIEnv, jboolean, jclass, jint, jintArray, jlong, jobjectArray, jstring};
use suggest::personal_words::{self, Entry, PersonalWords};

use super::boolean;
use crate::handle::{free, into_handle, object};
use crate::jni::{self, Env, Error, Result, illegal};

/// Whether a row tagged `tag` (null for every language) applies to `language`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_appliesTo(
    env: *mut JNIEnv,
    _: jclass,
    tag: jstring,
    language: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            let tag = env.read_nullable_string(tag)?;
            let language = env.read_string(language, "language")?;
            Ok(boolean(personal_words::applies_to(
                tag.as_deref(),
                &language,
            )))
        })
    }
}

/// A snapshot of the rows `words`, `frequencies` and `shortcuts` (null where a row has none),
/// which line up.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_newPersonalWords(
    env: *mut JNIEnv,
    _: jclass,
    words: jobjectArray,
    frequencies: jintArray,
    shortcuts: jobjectArray,
) -> jlong {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            let words = env.read_strings(words, "words")?;
            let frequencies = env.read::<i32>(frequencies, "frequencies")?;
            let shortcuts = env.read_strings(shortcuts, "shortcuts")?;
            if words.len() != frequencies.len() || words.len() != shortcuts.len() {
                return Err(illegal("words, frequencies and shortcuts differ in length"));
            }
            let entries = words.into_iter().zip(frequencies).zip(shortcuts);
            let entries = entries
                .map(|((word, frequency), shortcut)| {
                    Ok(Entry {
                        word: word.ok_or(Error::NullPointer("a word"))?,
                        frequency,
                        shortcut,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(into_handle(PersonalWords::new(entries)))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_freePersonalWords(
    _: *mut JNIEnv,
    _: jclass,
    words: jlong,
) {
    // SAFETY: the Kotlin side frees each snapshot once, when nothing can reach it.
    unsafe { free::<PersonalWords>(words) }
}

/// Runs `body` on the snapshot behind `handle`.
///
/// # Safety
/// `env` must be the environment the JVM passed to this call, and `handle` a snapshot that hasn't
/// been freed: its Kotlin owner is still reachable.
unsafe fn with_words<T>(
    env: *mut JNIEnv,
    handle: jlong,
    fallback: T,
    body: impl FnOnce(&PersonalWords, &Env) -> Result<T>,
) -> T {
    // SAFETY: as the caller promises.
    unsafe {
        jni::run(env, fallback, |env| {
            body(object(handle, "personal words")?, env)
        })
    }
}

/// Where each row kept, most frequent first, was among those it was made from.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_personalOrder(
    env: *mut JNIEnv,
    _: jclass,
    words: jlong,
) -> jintArray {
    // SAFETY: the JVM passes its environment; Kotlin holds the snapshot while it calls.
    unsafe {
        with_words(env, words, ptr::null_mut(), |words, env| {
            // Each is an index into a Java array, so it fits.
            let order: Vec<i32> = words.order().iter().map(|&i| i as i32).collect();
            env.make(&order)
        })
    }
}

/// How the user spells `word`, or null.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_personalSpellingOf(
    env: *mut JNIEnv,
    _: jclass,
    words: jlong,
    word: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment; Kotlin holds the snapshot while it calls.
    unsafe {
        with_words(env, words, ptr::null_mut(), |words, env| {
            let word = env.read_string(word, "word")?;
            env.make_nullable_string(words.spelling_of(&word))
        })
    }
}

/// At most `maxCount` completions of `typed`, best first.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_personalCompletions(
    env: *mut JNIEnv,
    _: jclass,
    words: jlong,
    typed: jstring,
    max_count: jint,
) -> jobjectArray {
    // SAFETY: the JVM passes its environment; Kotlin holds the snapshot while it calls.
    unsafe {
        with_words(env, words, ptr::null_mut(), |words, env| {
            let typed = env.read_string(typed, "typed")?;
            env.make_strings(&words.completions(&typed, max_count))
        })
    }
}
