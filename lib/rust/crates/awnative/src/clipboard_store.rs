//! The clipboard store (`stores::clips` in `stores::store::Store`), for `ClipboardStoreNative`,
//! whose generated entry points call these.
//!
//! A store is a handle to the store behind a lock: the keyboard changes the history on one
//! thread while its content provider reads the media files' records on others. Rows cross as
//! strings, numbers in decimal, with `wire::stores::clip_flag` saying which texts a clip has.

use std::io;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use stores::clips::{Clip, Clips, Expiry, FileInfo, Limit};
use stores::disk::RealDisk;
use stores::legacy::{self, ClipRows};
use stores::store::Store;
use wire::stores::clip_flag::{HAS_TEXT, HAS_URI, PINNED, REMOTE, SENSITIVE};

use crate::boundary::args::{
    ClipboardStoreNativeClipboardAdd as Add, ClipboardStoreNativeClipboardExpire as Expire,
    ClipboardStoreNativeClipboardImport as Import, ClipboardStoreNativeClipboardRestore as Restore,
};
use crate::handle::{free, into_handle, locked};
use crate::jni::{Error, Result, decimal, illegal};

type Rows = Vec<Vec<Vec<u16>>>;

/// A file's path from Kotlin.
pub(crate) fn path(units: &[u16]) -> Result<PathBuf> {
    String::from_utf16(units)
        .map(PathBuf::from)
        .map_err(|_| illegal("the path is not text"))
}

/// An error of the store's file, as an `IOException`.
pub(crate) fn io(error: io::Error) -> Error {
    Error::Io(error.to_string())
}

pub fn open_clipboard(path_units: &[u16]) -> Result<i64> {
    let store = Store::<Clips>::open(&path(path_units)?, Box::new(RealDisk));
    Ok(into_handle(Mutex::new(store)))
}

pub fn free_clipboard(store: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each store once, when nothing can reach it.
    unsafe { free::<Mutex<Store<Clips>>>(store) };
    Ok(())
}

fn clips<'a>(store: i64) -> Result<MutexGuard<'a, Store<Clips>>> {
    // SAFETY: Kotlin holds the store while it calls, so it hasn't been freed.
    unsafe { locked(store, "clipboard store") }
}

/// Makes `change` to the clipboard and saves it.
fn change<R>(store: i64, change: impl FnOnce(&mut Clips) -> R) -> Result<R> {
    clips(store)?.change(change).map_err(io)
}

fn limit(now: i64, enabled: bool, maximum: i32) -> Limit {
    Limit {
        now,
        enabled,
        maximum,
    }
}

/// How the store's file was found, as a `StoreOpened` code.
pub fn clipboard_opened(store: i64) -> Result<i32> {
    Ok(clips(store)?.opened() as i32)
}

/// The history as shown, a row a clip: id, type, time, flags, text, URI and MIME types.
pub fn clipboard_items(store: i64) -> Result<Rows> {
    let store = clips(store)?;
    Ok(store.value().history().into_iter().map(row).collect())
}

fn row(clip: &Clip) -> Vec<Vec<u16>> {
    let mut flags = stored_flags(clip);
    flags |= if clip.text.is_some() { HAS_TEXT } else { 0 };
    flags |= if clip.uri.is_some() { HAS_URI } else { 0 };
    vec![
        decimal(clip.id),
        decimal(clip.kind),
        decimal(clip.created),
        decimal(flags),
        clip.text.clone().unwrap_or_default(),
        clip.uri.clone().unwrap_or_default(),
        clip.mime_types.clone(),
    ]
}

fn stored_flags(clip: &Clip) -> i32 {
    let bit = |on: bool, bit: i32| if on { bit } else { 0 };
    bit(clip.pinned, PINNED) | bit(clip.sensitive, SENSITIVE) | bit(clip.remote, REMOTE)
}

/// A clip from Kotlin's fields, refusing flags it doesn't know.
fn clip(
    id: i64,
    (kind, created, flags): (i32, i64, i32),
    texts: [Option<&[u16]>; 2],
    mime: &[u16],
) -> Result<Clip> {
    if flags & !(PINNED | SENSITIVE | REMOTE) != 0 {
        return Err(illegal("unknown clip flags"));
    }
    Ok(Clip {
        id,
        kind,
        created,
        pinned: flags & PINNED != 0,
        sensitive: flags & SENSITIVE != 0,
        remote: flags & REMOTE != 0,
        text: texts[0].map(<[u16]>::to_vec),
        uri: texts[1].map(<[u16]>::to_vec),
        mime_types: mime.to_vec(),
    })
}

/// Clips from Kotlin's columns, row i of each, with `ids` if given.
fn columns(
    ids: Option<&[i64]>,
    numbers: (&[i32], &[i64], &[i32]),
    texts: [&[Option<Vec<u16>>]; 2],
    mimes: &[Vec<u16>],
) -> Result<Vec<Clip>> {
    let (kinds, created, flags) = numbers;
    let n = kinds.len();
    let lengths = [
        created.len(),
        flags.len(),
        texts[0].len(),
        texts[1].len(),
        mimes.len(),
    ];
    if lengths.iter().any(|&len| len != n) || ids.is_some_and(|ids| ids.len() != n) {
        return Err(illegal("the clips' columns differ in length"));
    }
    (0..n)
        .map(|i| {
            let id = ids.map_or(0, |ids| ids[i]);
            let texts = [texts[0][i].as_deref(), texts[1][i].as_deref()];
            clip(id, (kinds[i], created[i], flags[i]), texts, &mimes[i])
        })
        .collect()
}

/// Copies a clip into the history; its new row's id.
pub fn clipboard_add(args: Add) -> Result<i64> {
    let new = clip(
        0,
        (args.kind, args.created, args.flags),
        [args.text, args.uri],
        args.mime_types,
    )?;
    let limit = limit(args.now, args.limit_enabled, args.limit);
    change(args.store, |clips| clips.add(new, limit))
}

/// Adds each clip the history doesn't have yet; how many it added.
pub fn clipboard_restore(args: Restore) -> Result<i32> {
    let numbers = (args.kinds, args.created, args.flags);
    let new = columns(None, numbers, [args.texts, args.uris], args.mime_types)?;
    let limit = limit(args.now, args.limit_enabled, args.limit);
    let added = change(args.store, |clips| clips.restore(new, limit))?;
    Ok(i32::try_from(added).unwrap_or(i32::MAX))
}

pub fn clipboard_pin(
    store: i64,
    id: i64,
    pinned: bool,
    now: i64,
    limit_enabled: bool,
    maximum: i32,
) -> Result<()> {
    change(store, |clips| {
        clips.set_pinned(id, pinned, limit(now, limit_enabled, maximum))
    })
}

/// Removes the clips `ids`; the media no clip refers to any more.
pub fn clipboard_delete(
    store: i64,
    ids: &[i64],
    only_if_unpinned: bool,
    now: i64,
    limit_enabled: bool,
    maximum: i32,
) -> Result<Vec<Vec<u16>>> {
    change(store, |clips| {
        clips.delete(ids, only_if_unpinned, limit(now, limit_enabled, maximum))
    })
}

/// Removes every clip, or every unpinned one; the media no clip refers to any more.
pub fn clipboard_clear(
    store: i64,
    unpinned_only: bool,
    now: i64,
    limit_enabled: bool,
    maximum: i32,
) -> Result<Vec<Vec<u16>>> {
    change(store, |clips| {
        clips.clear(unpinned_only, limit(now, limit_enabled, maximum))
    })
}

pub fn clipboard_expire(args: Expire) -> Result<()> {
    let expiry = Expiry {
        old: args.old,
        minutes: args.minutes,
        sensitive: args.sensitive,
        seconds: args.seconds,
    };
    let limit = limit(args.now, args.limit_enabled, args.limit);
    change(args.store, |clips| clips.expire(expiry, limit))
}

pub fn clipboard_enforce(store: i64, now: i64, limit_enabled: bool, maximum: i32) -> Result<()> {
    change(store, |clips| {
        clips.enforce(limit(now, limit_enabled, maximum))
    })
}

/// The record of the media file `id`, as one row or none: id, name, size, orientation and MIME
/// types.
pub fn clipboard_file(store: i64, id: i64) -> Result<Rows> {
    let store = clips(store)?;
    let file = store.value().file(id);
    Ok(file
        .map(|file| {
            vec![
                decimal(file.id),
                file.name.clone(),
                decimal(file.size),
                decimal(file.orientation),
                file.mime_types.clone(),
            ]
        })
        .into_iter()
        .collect())
}

pub fn clipboard_add_file(
    store: i64,
    id: i64,
    name: &[u16],
    size: i64,
    orientation: i32,
    mime_types: &[u16],
) -> Result<()> {
    let file = FileInfo {
        id,
        name: name.to_vec(),
        size,
        orientation,
        mime_types: mime_types.to_vec(),
    };
    change(store, |clips| clips.add_file(file))
}

pub fn clipboard_remove_file(store: i64, id: i64) -> Result<()> {
    change(store, |clips| clips.remove_file(id))
}

/// Imports the old databases' rows once and deletes their files: a `StoreImport` code.
pub fn clipboard_import(args: Import) -> Result<i32> {
    let numbers = (args.kinds, args.created, args.flags);
    let items = columns(
        Some(args.ids),
        numbers,
        [args.texts, args.uris],
        args.mime_types,
    )?;
    let n = args.file_ids.len();
    let lengths = [
        args.file_names.len(),
        args.file_sizes.len(),
        args.orientations.len(),
        args.file_mime_types.len(),
    ];
    if lengths.iter().any(|&len| len != n) {
        return Err(illegal("the files' columns differ in length"));
    }
    let files = (0..n)
        .map(|i| FileInfo {
            id: args.file_ids[i],
            name: args.file_names[i].clone(),
            size: args.file_sizes[i],
            orientation: args.orientations[i],
            mime_types: args.file_mime_types[i].clone(),
        })
        .collect();
    let rows = ClipRows {
        items,
        files,
        sequence: args.sequence,
    };
    let paths: Vec<PathBuf> = args.paths.iter().map(|p| path(p)).collect::<Result<_>>()?;
    let mut store = clips(args.store)?;
    let imported = legacy::import(&mut store, rows, &paths).map_err(io)?;
    Ok(imported as i32)
}
