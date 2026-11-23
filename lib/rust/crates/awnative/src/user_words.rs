//! The keyboard's own user dictionary (`stores::words` in `stores::store::Store`), for
//! `UserWordsNative`, whose generated entry points call these.
//!
//! A store is a handle to the store behind a lock: the settings screen changes words while the
//! suggestions read them. A transaction spans several calls, so the Kotlin over it holds its
//! own lock across one. Rows cross as strings, numbers in decimal, with
//! `wire::stores::word_flag` saying which texts a word has.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use stores::disk::RealDisk;
use stores::legacy::{self, WordRows};
use stores::store::Store;
use stores::words::{Word, Words};
use wire::stores::word_flag::{HAS_LOCALE, HAS_SHORTCUT};

use crate::boundary::args::UserWordsNativeUserWordsImport as Import;
use crate::clipboard_store::{io, path};
use crate::handle::{free, into_handle, locked};
use crate::jni::{Error, Result, decimal, illegal};

type Rows = Vec<Vec<Vec<u16>>>;

pub fn open_user_words(path_units: &[u16]) -> Result<i64> {
    let store = Store::<Words>::open(&path(path_units)?, Box::new(RealDisk));
    Ok(into_handle(Mutex::new(store)))
}

pub fn free_user_words(store: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each store once, when nothing can reach it.
    unsafe { free::<Mutex<Store<Words>>>(store) };
    Ok(())
}

fn words<'a>(store: i64) -> Result<MutexGuard<'a, Store<Words>>> {
    // SAFETY: Kotlin holds the store while it calls, so it hasn't been freed.
    unsafe { locked(store, "user dictionary store") }
}

/// Makes `change` to the dictionary and saves it, unless a transaction is open.
fn change<R>(store: i64, change: impl FnOnce(&mut Words) -> R) -> Result<R> {
    words(store)?.change(change).map_err(io)
}

fn row(word: &Word) -> Vec<Vec<u16>> {
    let mut flags = if word.locale.is_some() { HAS_LOCALE } else { 0 };
    flags |= if word.shortcut.is_some() {
        HAS_SHORTCUT
    } else {
        0
    };
    vec![
        decimal(word.id),
        word.word.clone(),
        decimal(word.frequency),
        decimal(flags),
        word.locale.clone().unwrap_or_default(),
        word.shortcut.clone().unwrap_or_default(),
    ]
}

fn word(
    id: i64,
    text: &[u16],
    frequency: i32,
    locale: Option<&[u16]>,
    shortcut: Option<&[u16]>,
) -> Word {
    Word {
        id,
        word: text.to_vec(),
        frequency,
        locale: locale.map(<[u16]>::to_vec),
        shortcut: shortcut.map(<[u16]>::to_vec),
    }
}

/// How the store's file was found, as a `StoreOpened` code.
pub fn user_words_opened(store: i64) -> Result<i32> {
    Ok(words(store)?.opened() as i32)
}

/// Every word, by id, a row each: id, word, frequency, flags, locale and shortcut.
pub fn user_words_all(store: i64) -> Result<Rows> {
    Ok(words(store)?.value().all().iter().map(row).collect())
}

/// The words that are `word` exactly in `locale` exactly, by id.
pub fn user_words_exact(store: i64, text: &[u16], locale: Option<&[u16]>) -> Result<Rows> {
    let store = words(store)?;
    Ok(store
        .value()
        .exact(text, locale)
        .into_iter()
        .map(row)
        .collect())
}

/// Adds a word with its id, or the next if it is 0; the id it got. A taken id throws, as
/// Room's constraint did.
pub fn user_words_insert(
    store: i64,
    id: i64,
    text: &[u16],
    frequency: i32,
    locale: Option<&[u16]>,
    shortcut: Option<&[u16]>,
) -> Result<i64> {
    let new = word(id, text, frequency, locale, shortcut);
    change(store, |words| words.insert(new))?
        .map_err(|taken| Error::IllegalState(format!("a word has the id {}", taken.0)))
}

pub fn user_words_update(
    store: i64,
    id: i64,
    text: &[u16],
    frequency: i32,
    locale: Option<&[u16]>,
    shortcut: Option<&[u16]>,
) -> Result<()> {
    let new = word(id, text, frequency, locale, shortcut);
    change(store, |words| words.update(new))
}

pub fn user_words_delete(store: i64, id: i64) -> Result<()> {
    change(store, |words| words.delete(id))
}

pub fn user_words_begin(store: i64) -> Result<()> {
    words(store)?.begin();
    Ok(())
}

pub fn user_words_commit(store: i64) -> Result<()> {
    words(store)?.commit().map_err(io)
}

pub fn user_words_rollback(store: i64) -> Result<()> {
    words(store)?.rollback();
    Ok(())
}

/// Imports the old database's rows once and deletes its files: a `StoreImport` code.
pub fn user_words_import(args: Import) -> Result<i32> {
    let n = args.ids.len();
    let lengths = [
        args.words.len(),
        args.frequencies.len(),
        args.locales.len(),
        args.shortcuts.len(),
    ];
    if lengths.iter().any(|&len| len != n) {
        return Err(illegal("the words' columns differ in length"));
    }
    let rows = (0..n)
        .map(|i| {
            let (locale, shortcut) = (args.locales[i].as_deref(), args.shortcuts[i].as_deref());
            word(
                args.ids[i],
                &args.words[i],
                args.frequencies[i],
                locale,
                shortcut,
            )
        })
        .collect();
    let rows = WordRows {
        words: rows,
        sequence: args.sequence,
    };
    let paths: Vec<PathBuf> = args.paths.iter().map(|p| path(p)).collect::<Result<_>>()?;
    let mut store = words(args.store)?;
    let imported = legacy::import(&mut store, rows, &paths).map_err(io)?;
    Ok(imported as i32)
}
