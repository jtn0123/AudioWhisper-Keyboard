//! JNI entry points for the words the keyboard learns (`suggest::learned_words`), called by
//! `WordStoresNative`. A store is a handle to the words behind a lock: the keyboard learns on the
//! main thread, suggests from another, and loads and saves on a third.

use std::ptr;
use std::sync::Mutex;

use jni_sys::{JNIEnv, jboolean, jclass, jint, jlong, jobjectArray, jstring};
use suggest::learned_words::{self, LearnedWords, NegativeCount, Saved};

use super::boolean;
use crate::handle::{free, into_handle, locked, with_locked};
use crate::jni::{self, Env, Result, illegal};

/// The previous "word" of a sentence's first word (`learned_words::SENTENCE_START`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_sentenceStart(
    env: *mut JNIEnv,
    _: jclass,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            env.make_string(learned_words::SENTENCE_START)
        })
    }
}

/// How many uses make a word the user's (`learned_words::KNOWN_AFTER`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_knownAfter(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    learned_words::KNOWN_AFTER
}

/// How many words a store keeps unless told otherwise (`learned_words::WORD_CAPACITY`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_learnedWordCapacity(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    learned_words::WORD_CAPACITY
}

/// How many word pairs a store keeps unless told otherwise (`learned_words::PAIR_CAPACITY`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_learnedPairCapacity(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    learned_words::PAIR_CAPACITY
}

/// The longest word a store keeps (`learned_words::MAX_WORD_LENGTH`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_maxWordLength(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    learned_words::MAX_WORD_LENGTH as jint
}

/// Whether `word` can be learned (`learned_words::is_storable`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_isStorable(
    env: *mut JNIEnv,
    _: jclass,
    word: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            Ok(boolean(learned_words::is_storable(
                &env.read_string(word, "word")?,
            )))
        })
    }
}

/// The word before the next one, given the context's most recent word, or null when unknown
/// (`learned_words::previous_key`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_previousKey(
    env: *mut JNIEnv,
    _: jclass,
    first_word: jstring,
    is_beginning_of_sentence: jboolean,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let first_word = env.read_nullable_string(first_word)?;
            env.make_nullable_string(learned_words::previous_key(
                first_word.as_deref(),
                is_beginning_of_sentence != 0,
            ))
        })
    }
}

/// A new store keeping at most `wordCapacity` words and `pairCapacity` pairs.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_newLearnedWords(
    _: *mut JNIEnv,
    _: jclass,
    word_capacity: jint,
    pair_capacity: jint,
) -> jlong {
    into_handle(Mutex::new(LearnedWords::new(word_capacity, pair_capacity)))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_freeLearnedWords(
    _: *mut JNIEnv,
    _: jclass,
    store: jlong,
) {
    // SAFETY: the Kotlin side frees each store once, when nothing can reach it.
    unsafe { free::<Mutex<LearnedWords>>(store) }
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
    body: impl FnOnce(&mut LearnedWords, &Env) -> Result<T>,
) -> T {
    // SAFETY: as the caller promises.
    unsafe { with_locked(env, handle, "learned words", fallback, body) }
}

/// One use of `word` after `previous` (a word, the sentence start, or null for unknown).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_learnedAdd(
    env: *mut JNIEnv,
    _: jclass,
    store: jlong,
    word: jstring,
    previous: jstring,
    keep_spelling: jboolean,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the store while it calls.
    unsafe {
        with_store(env, store, (), |store, env| {
            let word = env.read_string(word, "word")?;
            let previous = env.read_nullable_string(previous)?;
            store.add(&word, previous.as_deref(), keep_spelling != 0);
            Ok(())
        })
    }
}

/// Takes back one use of `word` after `previous`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_learnedRemove(
    env: *mut JNIEnv,
    _: jclass,
    store: jlong,
    word: jstring,
    previous: jstring,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the store while it calls.
    unsafe {
        with_store(env, store, (), |store, env| {
            let word = env.read_string(word, "word")?;
            let previous = env.read_nullable_string(previous)?;
            store.remove(&word, previous.as_deref());
            Ok(())
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_learnedClear(
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

/// How the user spells each of their words.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_learnedKnownSpellings(
    env: *mut JNIEnv,
    _: jclass,
    store: jlong,
) -> jobjectArray {
    // SAFETY: the JVM passes its environment; Kotlin holds the store while it calls.
    unsafe {
        with_store(env, store, ptr::null_mut(), |store, env| {
            env.make_strings(&store.known_spellings())
        })
    }
}

/// How the user spells `word`, or null.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_learnedSpellingOf(
    env: *mut JNIEnv,
    _: jclass,
    store: jlong,
    word: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment; Kotlin holds the store while it calls.
    unsafe {
        with_store(env, store, ptr::null_mut(), |store, env| {
            let word = env.read_string(word, "word")?;
            env.make_nullable_string(store.spelling_of(&word))
        })
    }
}

/// Every completion of `typed`, best first, for the caller to keep the first `maxCount` it wants.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_learnedCompletions(
    env: *mut JNIEnv,
    _: jclass,
    store: jlong,
    typed: jstring,
    max_count: jint,
) -> jobjectArray {
    // SAFETY: the JVM passes its environment; Kotlin holds the store while it calls.
    unsafe {
        with_store(env, store, ptr::null_mut(), |store, env| {
            let typed = env.read_string(typed, "typed")?;
            env.make_strings(&store.completions(&typed, max_count))
        })
    }
}

/// Up to `maxCount` words typed after the context's most recent word (`firstWord`, or the
/// sentence start), at least `minCount` times. A negative `maxCount` throws, as Kotlin's `take` did.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_learnedFollowers(
    env: *mut JNIEnv,
    _: jclass,
    store: jlong,
    first_word: jstring,
    is_beginning_of_sentence: jboolean,
    min_count: jint,
    max_count: jint,
) -> jobjectArray {
    // SAFETY: the JVM passes its environment; Kotlin holds the store while it calls.
    unsafe {
        with_store(env, store, ptr::null_mut(), |store, env| {
            let first_word = env.read_nullable_string(first_word)?;
            let followers = store
                .followers(
                    first_word.as_deref(),
                    is_beginning_of_sentence != 0,
                    min_count,
                    max_count,
                )
                .map_err(|NegativeCount(count)| {
                    illegal(format!(
                        "Requested element count {count} is less than zero."
                    ))
                })?;
            env.make_strings(&followers)
        })
    }
}

/// How many times `word` was typed after the context's most recent word or sentence start.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_learnedPairCount(
    env: *mut JNIEnv,
    _: jclass,
    store: jlong,
    first_word: jstring,
    is_beginning_of_sentence: jboolean,
    word: jstring,
) -> jint {
    // SAFETY: the JVM passes its environment; Kotlin holds the store while it calls.
    unsafe {
        with_store(env, store, 0, |store, env| {
            let first_word = env.read_nullable_string(first_word)?;
            let word = env.read_string(word, "word")?;
            Ok(store.pair_count(first_word.as_deref(), is_beginning_of_sentence != 0, &word))
        })
    }
}

/// Takes in a saved file's `text`, read before taking the lock so learning needn't wait for it.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_learnedMerge(
    env: *mut JNIEnv,
    _: jclass,
    store: jlong,
    text: jstring,
) {
    // SAFETY: the JVM passes its environment; Kotlin holds the store while it calls.
    unsafe {
        jni::run(env, (), |env| {
            let saved = Saved::parse(&env.read_string(text, "text")?);
            locked::<LearnedWords>(store, "learned words")?.merge(saved);
            Ok(())
        })
    }
}

/// The text to save.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_WordStoresNative_learnedSavedText(
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
