//! The GIF panel's rules (the `gif` crate) for `GifNative`, whose generated entry points call
//! these or the crate directly. A GIF in recents' order is the index of one before, or `-1 - i`
//! for the `i`th sent.

use std::sync::Mutex;

use gif::cache::{self, ResultCache};
use gif::keys::{self, Route, Swipe};
use gif::problem::{self, Failure};
use gif::recents::{self, From};
use gif::{chips, search_text};

use crate::handle::{free, into_handle, locked};
use crate::jni::{Result, illegal};

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

/// A count of GIFs or searches, which can't be below 0.
fn capacity(value: i32) -> Result<usize> {
    usize::try_from(value).map_err(|_| illegal(format!("capacity {value} is below 0")))
}

/// The longest search, in UTF-16 units (`search_text::MAX_LENGTH`).
pub fn max_length() -> i32 {
    search_text::MAX_LENGTH as i32
}

/// The quick searches, trending first (`chips::CATEGORIES`).
pub fn categories() -> Vec<Vec<u16>> {
    chips::CATEGORIES.map(utf16).to_vec()
}

/// The chip for the GIFs sent before (`chips::RECENTS`).
pub fn recents() -> Vec<u16> {
    utf16(chips::RECENTS)
}

/// How long the result cache keeps a search, in milliseconds (`cache::TTL_MS`).
pub fn cache_ttl_ms() -> i64 {
    cache::TTL_MS
}

/// How many searches the result cache keeps (`cache::CAPACITY`).
pub fn cache_capacity() -> i32 {
    cache::CAPACITY as i32
}

/// How many GIFs recents keeps (`recents::CAPACITY`).
pub fn recents_capacity() -> i32 {
    recents::CAPACITY as i32
}

/// `query` without its last `count` characters (`search_text::delete_last`).
pub fn delete_last(query: &[u16], count: i32) -> Vec<u16> {
    search_text::delete_last(query, count).to_vec()
}

/// `query` without its last `count` words (`search_text::delete_words`).
pub fn delete_words(query: &[u16], count: i32) -> Vec<u16> {
    search_text::delete_words(query, count).to_vec()
}

/// The word just typed at the end of `text`, or null (`search_text::last_word`).
pub fn last_word(text: Option<&[u16]>) -> Option<Vec<u16>> {
    text.and_then(search_text::last_word).map(<[u16]>::to_vec)
}

/// Where a key goes while typing into the search (`keys::route`), as `GifRules.Route` orders it.
pub fn route(code: i32, typing: bool) -> i32 {
    match keys::route(code, typing) {
        Route::Search => 0,
        Route::Keyboard => 1,
        Route::Ignore => 2,
    }
}

/// How far a delete-key swipe reaches back (`keys::swipe_delete`): `swipe` is 0 to delete
/// characters, 1 words and 2 to select; the count comes back.
pub fn swipe_delete(swipe: i32, units_x: i32) -> Result<i32> {
    let swipe = match swipe {
        0 => Swipe::DeleteCharacters,
        1 => Swipe::DeleteWords,
        2 => Swipe::Select,
        _ => return Err(illegal(format!("no swipe {swipe}"))),
    };
    Ok(keys::swipe_delete(swipe, units_x).count)
}

/// What a failed GIPHY call tells the user (`problem::problem`): `failure` is 0 for an
/// unreadable answer, 1 for an HTTP `status` and 2 for no answer.
pub fn problem(failure: i32, status: i32) -> Result<Vec<u16>> {
    let failure = match failure {
        0 => Failure::Unreadable,
        1 => Failure::Status(status),
        2 => Failure::Unreachable,
        _ => return Err(illegal(format!("no failure {failure}"))),
    };
    Ok(utf16(problem::problem(failure)))
}

/// A new, empty result cache keeping `capacity` searches for `ttl_ms`.
pub fn new_cache(ttl_ms: i64, capacity: i32) -> Result<i64> {
    let cache = ResultCache::new(ttl_ms, self::capacity(capacity)?);
    Ok(into_handle(Mutex::new(cache)))
}

pub fn free_cache(cache: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each cache once, when nothing can reach it.
    unsafe { free::<Mutex<ResultCache>>(cache) };
    Ok(())
}

/// Whether the cache behind `cache` still keeps `key` at `now` (`ResultCache::get`).
pub fn cache_get(cache: i64, key: &[u16], now: i64) -> Result<bool> {
    // SAFETY: Kotlin holds the cache while it calls, so it hasn't been freed.
    let mut cache = unsafe { locked::<ResultCache>(cache, "result cache")? };
    Ok(cache.get(key, now))
}

/// `key` was stored at `now` in the cache behind `cache` (`ResultCache::put`): the keys dropped.
pub fn cache_put(cache: i64, key: &[u16], now: i64) -> Result<Vec<Vec<u16>>> {
    // SAFETY: Kotlin holds the cache while it calls, so it hasn't been freed.
    let mut cache = unsafe { locked::<ResultCache>(cache, "result cache")? };
    Ok(cache.put(key, now))
}

/// A GIF's place in recents' new order, as the module's doc says.
fn place(from: From) -> i32 {
    match from {
        From::Before(at) => at as i32,
        From::Sent(at) => -1 - at as i32,
    }
}

/// Recents' order after the GIF with `id` was sent (`recents::add`).
pub fn recents_add(ids: &[Vec<u16>], id: &[u16], capacity: i32) -> Result<Vec<i32>> {
    let order = recents::add(ids, id, self::capacity(capacity)?);
    Ok(order.into_iter().map(place).collect())
}

/// Recents' order once the `saved` ids are read, keeping those `sent` meanwhile (`recents::load`).
pub fn recents_load(saved: &[Vec<u16>], sent: &[Vec<u16>], capacity: i32) -> Result<Vec<i32>> {
    let order = recents::load(saved, sent, self::capacity(capacity)?);
    Ok(order.into_iter().map(place).collect())
}
