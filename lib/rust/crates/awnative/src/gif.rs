//! JNI entry points for the GIF panel's rules (the `gif` crate), called by `GifNative` in
//! lib/rust's Kotlin. A GIF in recents' order is the index of one before, or `-1 - i` for the
//! `i`th sent.

use std::ptr;
use std::sync::Mutex;

use gif::cache::{self, ResultCache};
use gif::keys::{self, Route, Swipe};
use gif::problem::{self, Failure};
use gif::recents::{self, From};
use gif::{chips, search_text};
use jni_sys::{JNI_FALSE, JNIEnv, jboolean, jclass, jint, jintArray, jlong, jobjectArray, jstring};

use crate::handle::{free, into_handle, with_locked};
use crate::jni::{self, Env, Result, boolean, illegal};

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

/// A count of GIFs or searches, which can't be below 0.
fn capacity(value: jint) -> Result<usize> {
    usize::try_from(value).map_err(|_| illegal(format!("capacity {value} is below 0")))
}

/// The strings in `array`, none of them null.
fn strings(env: &Env, array: jobjectArray, name: &'static str) -> Result<Vec<Vec<u16>>> {
    (env.read_strings(array, name)?.into_iter())
        .map(|it| it.ok_or_else(|| illegal(format!("a null in {name}"))))
        .collect()
}

/// Runs `body` on a string argument and gives back the string it makes.
///
/// # Safety
/// `env` must be the environment the JVM passed to this call.
unsafe fn edit(
    env: *mut JNIEnv,
    text: jstring,
    name: &'static str,
    body: impl FnOnce(&[u16]) -> Vec<u16>,
) -> jstring {
    // SAFETY: as the caller promises.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            env.make_string(&body(&env.read_string(text, name)?))
        })
    }
}

/// The longest search, in UTF-16 units (`search_text::MAX_LENGTH`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_maxLength(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    search_text::MAX_LENGTH as jint
}

/// The quick searches, trending first (`chips::CATEGORIES`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_categories(
    env: *mut JNIEnv,
    _: jclass,
) -> jobjectArray {
    let categories: Vec<Vec<u16>> = chips::CATEGORIES.map(utf16).to_vec();
    // SAFETY: the JVM passes its environment for this call.
    unsafe { jni::run(env, ptr::null_mut(), |env| env.make_strings(&categories)) }
}

/// The chip for the GIFs sent before (`chips::RECENTS`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_recents(
    env: *mut JNIEnv,
    _: jclass,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            env.make_string(&utf16(chips::RECENTS))
        })
    }
}

/// How long the result cache keeps a search, in milliseconds (`cache::TTL_MS`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_cacheTtlMs(
    _: *mut JNIEnv,
    _: jclass,
) -> jlong {
    cache::TTL_MS
}

/// How many searches the result cache keeps (`cache::CAPACITY`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_cacheCapacity(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    cache::CAPACITY as jint
}

/// How many GIFs recents keeps (`recents::CAPACITY`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_recentsCapacity(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    recents::CAPACITY as jint
}

/// `query` with `text` typed after it (`search_text::type_text`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_type(
    env: *mut JNIEnv,
    _: jclass,
    query: jstring,
    text: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let query = env.read_string(query, "query")?;
            let text = env.read_string(text, "text")?;
            env.make_string(&search_text::type_text(&query, &text))
        })
    }
}

/// `query` with a glide-typed `word` after it (`search_text::type_word`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_typeWord(
    env: *mut JNIEnv,
    _: jclass,
    query: jstring,
    word: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let query = env.read_string(query, "query")?;
            let word = env.read_string(word, "word")?;
            env.make_string(&search_text::type_word(&query, &word))
        })
    }
}

/// `query` without its last `count` characters (`search_text::delete_last`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_deleteLast(
    env: *mut JNIEnv,
    _: jclass,
    query: jstring,
    count: jint,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        edit(env, query, "query", |it| {
            search_text::delete_last(it, count).to_vec()
        })
    }
}

/// `query` without its last `count` words (`search_text::delete_words`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_deleteWords(
    env: *mut JNIEnv,
    _: jclass,
    query: jstring,
    count: jint,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        edit(env, query, "query", |it| {
            search_text::delete_words(it, count).to_vec()
        })
    }
}

/// The word just typed at the end of `text`, or null (`search_text::last_word`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_lastWord(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let text = env.read_nullable_string(text)?;
            env.make_nullable_string(text.as_deref().and_then(search_text::last_word))
        })
    }
}

/// The chips to offer after `typed_word`, which may be null (`chips::chips`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_chips(
    env: *mut JNIEnv,
    _: jclass,
    typed_word: jstring,
    has_recents: jboolean,
) -> jobjectArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let typed = env.read_nullable_string(typed_word)?;
            env.make_strings(&chips::chips(typed.as_deref(), has_recents != 0))
        })
    }
}

/// Where a key goes while typing into the search (`keys::route`), as `GifRules.Route` orders it.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_route(
    _: *mut JNIEnv,
    _: jclass,
    code: jint,
    typing: jboolean,
) -> jint {
    match keys::route(code, typing != 0) {
        Route::Search => 0,
        Route::Keyboard => 1,
        Route::Ignore => 2,
    }
}

/// How far a delete-key swipe reaches back (`keys::swipe_delete`): `swipe` is 0 to delete
/// characters, 1 words and 2 to select; the count comes back.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_swipeDelete(
    env: *mut JNIEnv,
    _: jclass,
    swipe: jint,
    units_x: jint,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |_| {
            let swipe = match swipe {
                0 => Swipe::DeleteCharacters,
                1 => Swipe::DeleteWords,
                2 => Swipe::Select,
                _ => return Err(illegal(format!("no swipe {swipe}"))),
            };
            Ok(keys::swipe_delete(swipe, units_x).count)
        })
    }
}

/// What a failed GIPHY call tells the user (`problem::problem`): `failure` is 0 for an
/// unreadable answer, 1 for an HTTP `status` and 2 for no answer.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_problem(
    env: *mut JNIEnv,
    _: jclass,
    failure: jint,
    status: jint,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let failure = match failure {
                0 => Failure::Unreadable,
                1 => Failure::Status(status),
                2 => Failure::Unreachable,
                _ => return Err(illegal(format!("no failure {failure}"))),
            };
            env.make_string(&utf16(problem::problem(failure)))
        })
    }
}

/// The key `search` is cached under (`cache::key`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_cacheKey(
    env: *mut JNIEnv,
    _: jclass,
    search: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe { edit(env, search, "search", cache::key) }
}

/// A new, empty result cache keeping `capacity` searches for `ttl_ms`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_newCache(
    env: *mut JNIEnv,
    _: jclass,
    ttl_ms: jlong,
    capacity: jint,
) -> jlong {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |_| {
            let cache = ResultCache::new(ttl_ms, self::capacity(capacity)?);
            Ok(into_handle(Mutex::new(cache)))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_freeCache(
    _: *mut JNIEnv,
    _: jclass,
    cache: jlong,
) {
    // SAFETY: the Kotlin side frees each cache once, when nothing can reach it.
    unsafe { free::<Mutex<ResultCache>>(cache) }
}

/// Whether the cache behind `cache` still keeps `key` at `now` (`ResultCache::get`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_cacheGet(
    env: *mut JNIEnv,
    _: jclass,
    cache: jlong,
    key: jstring,
    now: jlong,
) -> jboolean {
    // SAFETY: the JVM passes its environment; Kotlin holds the cache while it calls.
    unsafe {
        with_locked(
            env,
            cache,
            "result cache",
            JNI_FALSE,
            |cache: &mut ResultCache, env| {
                Ok(boolean(cache.get(&env.read_string(key, "key")?, now)))
            },
        )
    }
}

/// `key` was stored at `now` in the cache behind `cache` (`ResultCache::put`): the keys dropped.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_cachePut(
    env: *mut JNIEnv,
    _: jclass,
    cache: jlong,
    key: jstring,
    now: jlong,
) -> jobjectArray {
    // SAFETY: the JVM passes its environment; Kotlin holds the cache while it calls.
    unsafe {
        with_locked(
            env,
            cache,
            "result cache",
            ptr::null_mut(),
            |cache: &mut ResultCache, env| {
                let dropped = cache.put(&env.read_string(key, "key")?, now);
                env.make_strings(&dropped)
            },
        )
    }
}

/// A GIF's place in recents' new order, as the module's doc says.
fn place(from: From) -> jint {
    match from {
        From::Before(at) => at as jint,
        From::Sent(at) => -1 - at as jint,
    }
}

/// Recents' order after the GIF with `id` was sent (`recents::add`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_recentsAdd(
    env: *mut JNIEnv,
    _: jclass,
    ids: jobjectArray,
    id: jstring,
    capacity: jint,
) -> jintArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let ids = strings(env, ids, "ids")?;
            let id = env.read_string(id, "id")?;
            let order = recents::add(&ids, &id, self::capacity(capacity)?);
            env.make(&order.into_iter().map(place).collect::<Vec<_>>())
        })
    }
}

/// Recents' order once the `saved` ids are read, keeping those `sent` meanwhile (`recents::load`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GifNative_recentsLoad(
    env: *mut JNIEnv,
    _: jclass,
    saved: jobjectArray,
    sent: jobjectArray,
    capacity: jint,
) -> jintArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let saved = strings(env, saved, "saved")?;
            let sent = strings(env, sent, "sent")?;
            let order = recents::load(&saved, &sent, self::capacity(capacity)?);
            env.make(&order.into_iter().map(place).collect::<Vec<_>>())
        })
    }
}
