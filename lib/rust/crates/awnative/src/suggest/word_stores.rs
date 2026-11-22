//! The words the keyboard keeps for the user (`suggest::learned_words`, `kept_words` and
//! `personal_words`), for `WordStoresNative`, whose generated entry points call these.
//!
//! A learned or kept store is a handle to the words behind a lock: the keyboard learns on the
//! main thread, suggests from another, and loads and saves on a third. A personal snapshot never
//! changes once made, so its handle needs no lock.

use std::sync::{Mutex, MutexGuard};

use suggest::kept_words::{self, KeptWords};
use suggest::learned_words::{self, LearnedWords, Saved};
use suggest::personal_words::{Entry, PersonalWords};
use suggest::swipe_words::Boost;

use crate::handle::{into_handle, locked, object};
use crate::jni::{Result, illegal};

fn owned(words: Vec<&[u16]>) -> Vec<Vec<u16>> {
    words.into_iter().map(<[u16]>::to_vec).collect()
}

/// The previous "word" of a sentence's first word (`learned_words::SENTENCE_START`).
pub fn sentence_start() -> Vec<u16> {
    learned_words::SENTENCE_START.to_vec()
}

/// How many uses make a word the user's (`learned_words::KNOWN_AFTER`).
pub fn known_after() -> i32 {
    learned_words::KNOWN_AFTER
}

/// How many words a store keeps unless told otherwise (`learned_words::WORD_CAPACITY`).
pub fn learned_word_capacity() -> i32 {
    learned_words::WORD_CAPACITY
}

/// How many word pairs a store keeps unless told otherwise (`learned_words::PAIR_CAPACITY`).
pub fn learned_pair_capacity() -> i32 {
    learned_words::PAIR_CAPACITY
}

/// The longest word a store keeps (`learned_words::MAX_WORD_LENGTH`).
pub fn max_word_length() -> i32 {
    learned_words::MAX_WORD_LENGTH as i32
}

/// How many words a kept store remembers unless told otherwise (`kept_words::CAPACITY`).
pub fn kept_capacity() -> i32 {
    kept_words::CAPACITY
}

/// The word before the next one, given the context's most recent word, or null when unknown
/// (`learned_words::previous_key`).
pub fn previous_key(
    first_word: Option<&[u16]>,
    is_beginning_of_sentence: bool,
) -> Option<Vec<u16>> {
    learned_words::previous_key(first_word, is_beginning_of_sentence).map(<[u16]>::to_vec)
}

/// A new learned store keeping at most `word_capacity` words and `pair_capacity` pairs.
pub fn new_learned_words(word_capacity: i32, pair_capacity: i32) -> i64 {
    into_handle(Mutex::new(LearnedWords::new(word_capacity, pair_capacity)))
}

pub fn free_learned_words(store: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each store once, when nothing can reach it.
    unsafe { crate::handle::free::<Mutex<LearnedWords>>(store) };
    Ok(())
}

fn learned<'a>(store: i64) -> Result<MutexGuard<'a, LearnedWords>> {
    // SAFETY: Kotlin holds the store while it calls, so it hasn't been freed.
    unsafe { locked(store, "learned words") }
}

/// One use of `word` after `previous` (a word, the sentence start, or null for unknown).
pub fn learned_add(
    store: i64,
    word: &[u16],
    previous: Option<&[u16]>,
    keep_spelling: bool,
) -> Result<()> {
    learned(store)?.add(word, previous, keep_spelling);
    Ok(())
}

/// Takes back one use of `word` after `previous`.
pub fn learned_remove(store: i64, word: &[u16], previous: Option<&[u16]>) -> Result<()> {
    learned(store)?.remove(word, previous);
    Ok(())
}

pub fn learned_clear(store: i64) -> Result<()> {
    learned(store)?.clear();
    Ok(())
}

/// How the user spells each of their words.
pub fn learned_known_spellings(store: i64) -> Result<Vec<Vec<u16>>> {
    Ok(owned(learned(store)?.known_spellings()))
}

/// How many steps the user's use of each of `words` raises it for swipe typing
/// (`swipe_words::Boost::DEFAULT`): 0 for a word that isn't theirs as spelt.
pub fn learned_swipe_boosts(store: i64, words: &[Vec<u16>]) -> Result<Vec<i32>> {
    let store = learned(store)?;
    Ok(words
        .iter()
        .map(|word| Boost::DEFAULT.of(&store, word))
        .collect())
}

/// How the user spells `word`, or null.
pub fn learned_spelling_of(store: i64, word: &[u16]) -> Result<Option<Vec<u16>>> {
    Ok(learned(store)?.spelling_of(word).map(<[u16]>::to_vec))
}

/// Every completion of `typed`, best first, for the caller to keep the first `max_count` it wants.
pub fn learned_completions(store: i64, typed: &[u16], max_count: i32) -> Result<Vec<Vec<u16>>> {
    Ok(owned(learned(store)?.completions(typed, max_count)))
}

/// Up to `max_count` words typed after the context's most recent word (`first_word`, or the
/// sentence start), at least `min_count` times. A negative `max_count` throws, as Kotlin's `take`
/// did.
pub fn learned_followers(
    store: i64,
    first_word: Option<&[u16]>,
    is_beginning_of_sentence: bool,
    min_count: i32,
    max_count: i32,
) -> Result<Vec<Vec<u16>>> {
    let store = learned(store)?;
    let followers = store.followers(first_word, is_beginning_of_sentence, min_count, max_count)?;
    Ok(owned(followers))
}

/// How many times `word` was typed after the context's most recent word or sentence start.
pub fn learned_pair_count(
    store: i64,
    first_word: Option<&[u16]>,
    is_beginning_of_sentence: bool,
    word: &[u16],
) -> Result<i32> {
    Ok(learned(store)?.pair_count(first_word, is_beginning_of_sentence, word))
}

/// How many times the words have changed, as a bit pattern: only equality means anything.
pub fn learned_changes(store: i64) -> Result<i64> {
    Ok(learned(store)?.changes() as i64)
}

/// Takes in a saved file's `text`, read before taking the lock so learning needn't wait for it.
pub fn learned_merge(store: i64, text: &[u16]) -> Result<()> {
    let saved = Saved::parse(text);
    learned(store)?.merge(saved);
    Ok(())
}

/// The text to save.
pub fn learned_saved_text(store: i64) -> Result<Vec<u16>> {
    Ok(learned(store)?.saved_text())
}

/// A new kept store remembering at most `capacity` words.
pub fn new_kept_words(capacity: i32) -> i64 {
    into_handle(Mutex::new(KeptWords::new(capacity)))
}

pub fn free_kept_words(store: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each store once, when nothing can reach it.
    unsafe { crate::handle::free::<Mutex<KeptWords>>(store) };
    Ok(())
}

fn kept<'a>(store: i64) -> Result<MutexGuard<'a, KeptWords>> {
    // SAFETY: Kotlin holds the store while it calls, so it hasn't been freed.
    unsafe { locked(store, "kept words") }
}

/// Whether `word` is kept, in any case.
pub fn kept_contains(store: i64, word: &[u16]) -> Result<bool> {
    Ok(kept(store)?.contains(word))
}

/// Keeps `word`, saving it if `remember`; true when the saved text changed.
pub fn kept_add(store: i64, word: &[u16], remember: bool) -> Result<bool> {
    Ok(kept(store)?.add(word, remember))
}

pub fn kept_clear(store: i64) -> Result<()> {
    kept(store)?.clear();
    Ok(())
}

/// Takes in a saved file's `text`, read before taking the lock.
pub fn kept_merge(store: i64, text: &[u16]) -> Result<()> {
    let saved = KeptWords::parse(text);
    kept(store)?.merge(saved);
    Ok(())
}

/// The text to save.
pub fn kept_saved_text(store: i64) -> Result<Vec<u16>> {
    Ok(kept(store)?.saved_text())
}

/// A snapshot of the rows `words`, `frequencies` and `shortcuts` (null where a row has none),
/// which line up.
pub fn new_personal_words(
    words: &[Vec<u16>],
    frequencies: &[i32],
    shortcuts: &[Option<Vec<u16>>],
) -> Result<i64> {
    if words.len() != frequencies.len() || words.len() != shortcuts.len() {
        return Err(illegal("words, frequencies and shortcuts differ in length"));
    }
    let rows = words.iter().zip(frequencies).zip(shortcuts);
    let entries = rows
        .map(|((word, &frequency), shortcut)| Entry {
            word: word.clone(),
            frequency,
            shortcut: shortcut.clone(),
        })
        .collect();
    Ok(into_handle(PersonalWords::new(entries)))
}

pub fn free_personal_words(words: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each snapshot once, when nothing can reach it.
    unsafe { crate::handle::free::<PersonalWords>(words) };
    Ok(())
}

fn personal<'a>(words: i64) -> Result<&'a PersonalWords> {
    // SAFETY: Kotlin holds the snapshot while it calls, so it hasn't been freed.
    unsafe { object(words, "personal words") }
}

/// Where each row kept, most frequent first, was among those it was made from.
pub fn personal_order(words: i64) -> Result<Vec<i32>> {
    // Each is an index into a Java array, so it fits.
    Ok(personal(words)?.order().iter().map(|&i| i as i32).collect())
}

/// How the user spells `word`, or null.
pub fn personal_spelling_of(words: i64, word: &[u16]) -> Result<Option<Vec<u16>>> {
    Ok(personal(words)?.spelling_of(word).map(<[u16]>::to_vec))
}

/// At most `max_count` completions of `typed`, best first.
pub fn personal_completions(words: i64, typed: &[u16], max_count: i32) -> Result<Vec<Vec<u16>>> {
    Ok(owned(personal(words)?.completions(typed, max_count)))
}
