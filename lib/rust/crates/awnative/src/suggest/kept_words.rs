//! JNI entry points for the words kept as typed (`suggest::kept_words`), called by
//! `WordStoresNative`. A store is a handle to the words behind a lock: an undo adds one on the main
//! thread, suggesting reads them on another, and loading and saving happen on a third.

use std::ptr;
use std::sync::Mutex;

use jni_sys::{JNIEnv, jboolean, jclass, jint, jlong, jstring};
use suggest::kept_words::{self, KeptWords};

use super::boolean;
use crate::handle::{free, into_handle, locked, with_locked};
use crate::jni::{self, Env, Result};

/// How many words a store remembers unless told otherwise (`kept_words::CAPACITY`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_keptCapacity(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    kept_words::CAPACITY
}

/// A new store remembering at most `capacity` words.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_newKeptWords(
    _: *mut JNIEnv,
    _: jclass,
    capacity: jint,
) -> jlong {
    into_handle(Mutex::new(KeptWords::new(capacity)))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_freeKeptWords(
    _: *mut JNIEnv,
    _: jclass,
    store: jlong,
) {
    // SAFETY: the Kotlin side frees each store once, when nothing can reach it.
    unsafe { free::<Mutex<KeptWords>>(store) }
}

/// Runs `body` on the store behind `handle`, locked.
///
/// # Safety
/// `env` must be the environment the JVM passed to this call, and `handle` a store that hasn't been
/// freed: its Kotlin owner is still reachable.
unsafe fn with_store<T>(
    env: *mut JNIEnv,
    handle: jlong,
    fallback: T,
    body: impl FnOnce(&mut KeptWords, &Env) -> Result<T>,
) -> T {
    // SAFETY: as the caller promises.
    unsafe { with_locked(env, handle, "kept words", fallback, body) }
}

/// Whether `word` is kept, in any case.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_keptContains(
    env: *mut JNIEnv,
    _: jclass,
    store: jlong,
    word: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment; Kotlin holds the store while it calls.
    unsafe {
        with_store(env, store, 0, |store, env| {
            Ok(boolean(store.contains(&env.read_string(word, "word")?)))
        })
    }
}

/// Keeps `word`, saving it if `remember`; true when the saved text changed.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_keptAdd(
    env: *mut JNIEnv,
    _: jclass,
    store: jlong,
    word: jstring,
    remember: jboolean,
) -> jboolean {
    // SAFETY: the JVM passes its environment; Kotlin holds the store while it calls.
    unsafe {
        with_store(env, store, 0, |store, env| {
            let word = env.read_string(word, "word")?;
            Ok(boolean(store.add(&word, remember != 0)))
        })
    }
}

/// Forgets every kept word.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_keptClear(
    env: *mut JNIEnv,
    _: jclass,
    store: jlong,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the store while it calls.
    unsafe {
        with_store(env, store, (), |store, _| {
            store.clear();
            Ok(())
        })
    }
}

/// Takes in a saved file's `text`, read before taking the lock.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_keptMerge(
    env: *mut JNIEnv,
    _: jclass,
    store: jlong,
    text: jstring,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the store while it calls.
    unsafe {
        jni::run(env, (), |env| {
            let saved = KeptWords::parse(&env.read_string(text, "text")?);
            locked::<KeptWords>(store, "kept words")?.merge(saved);
            Ok(())
        })
    }
}

/// The text to save.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_keptSavedText(
    env: *mut JNIEnv,
    _: jclass,
    store: jlong,
) -> jstring {
    // SAFETY: the JVM passes its environment; Kotlin holds the store while it calls.
    unsafe {
        with_store(env, store, ptr::null_mut(), |store, env| {
            env.make_string(&store.saved_text())
        })
    }
}
