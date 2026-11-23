//! The GIF panel's decisions (`gif::session`, `gif::previews`, `gif::send`) for
//! `GifSessionNative`, whose generated entry points call these. Effects run after returning to
//! Kotlin, outside the lock.

use std::sync::{Mutex, MutexGuard};

use gif::previews::Previews;
use gif::rows::{self, Fields};
use gif::session::Session;
use gif::{reply, send};

use crate::handle::{free as free_handle, into_handle, locked};
use crate::jni::{Result, illegal};

/// A new session.
pub fn create() -> i64 {
    into_handle(Mutex::new(Session::new()))
}

pub fn free(session: i64) -> Result<()> {
    // SAFETY: the synchronized wrapper frees its handle once, then sets it to zero.
    unsafe { free_handle::<Mutex<Session>>(session) };
    Ok(())
}

fn session<'a>(handle: i64) -> Result<MutexGuard<'a, Session>> {
    // SAFETY: Kotlin holds the session while it calls, so it hasn't been freed.
    unsafe { locked(handle, "GIF session") }
}

fn previews<'a>(handle: i64) -> Result<MutexGuard<'a, Previews>> {
    // SAFETY: Kotlin holds the previews while it calls, so they haven't been freed.
    unsafe { locked(handle, "GIF previews") }
}

/// The plan for a `GifOp` at `now` (`Session::call`).
pub fn call(
    handle: i64,
    operation: &[u16],
    now: i64,
    fields: &[Vec<u16>],
) -> Result<Vec<Vec<u16>>> {
    let operation = String::from_utf16(operation).map_err(|_| illegal("invalid GIF operation"))?;
    session(handle)?
        .call(&operation, now, fields)
        .map_err(illegal)
}

/// Recents `generation`'s file held these GIF `rows`, unless it wasn't `present` (`Session::read`).
pub fn read(handle: i64, generation: i32, present: bool, rows: &[Vec<u16>]) -> Result<()> {
    let saved = if present {
        Some(Fields::new(rows).gifs().map_err(illegal)?)
    } else {
        None
    };
    session(handle)?.read(generation, saved);
    Ok(())
}

/// What a save of recents `generation` writes: the file, then GIF rows (`Session::rows`).
pub fn rows(handle: i64, generation: i32) -> Result<Vec<Vec<u16>>> {
    let session = session(handle)?;
    let Some((path, gifs)) = session.rows(generation) else {
        return Ok(Vec::new());
    };
    Ok(std::iter::once(path).chain(rows::gifs(gifs)).collect())
}

/// How to fetch a `GifMedia` (`reply::request`).
pub fn request(media: i32) -> Result<Vec<i32>> {
    reply::request(media)
        .map(Vec::from)
        .ok_or_else(|| illegal(format!("no GIF media {media}")))
}

pub fn previews_create() -> i64 {
    into_handle(Mutex::new(Previews::default()))
}

pub fn previews_free(handle: i64) -> Result<()> {
    // SAFETY: the synchronized wrapper frees its handle once, then sets it to zero.
    unsafe { free_handle::<Mutex<Previews>>(handle) };
    Ok(())
}

/// Whether `url`'s preview is kept (`Previews::get`).
pub fn previews_get(handle: i64, url: &[u16]) -> Result<bool> {
    Ok(previews(handle)?.get(url))
}

/// `url`'s preview of `bytes` is kept: the URLs dropped (`Previews::put`).
pub fn previews_put(handle: i64, url: &[u16], bytes: i32) -> Result<Vec<Vec<u16>>> {
    let bytes =
        usize::try_from(bytes).map_err(|_| illegal(format!("a preview of {bytes} bytes")))?;
    Ok(previews(handle)?.put(url, bytes))
}

pub fn previews_clear(handle: i64) -> Result<()> {
    previews(handle)?.clear();
    Ok(())
}

/// What to do after a step of sending a GIF (`send::next`).
pub fn send_next(step: &[u16], ok: bool, linkable: bool) -> Result<Vec<Vec<u16>>> {
    let step = String::from_utf16(step).map_err(|_| illegal("invalid GIF send step"))?;
    let actions = send::next(&step, ok, linkable).map_err(illegal)?;
    Ok(actions
        .iter()
        .map(|action| action.encode_utf16().collect())
        .collect())
}

/// The file a GIF with `id` is sent from, then its part file (`send::file_name`).
pub fn send_files(id: &[u16]) -> Vec<Vec<u16>> {
    let file = send::file_name(id);
    let mut part = file.clone();
    part.extend(send::PART.encode_utf16());
    vec![file, part]
}

/// The indexes of the files to delete at `now` (`send::stale`).
pub fn send_stale(names: &[Vec<u16>], modified: &[i64], now: i64) -> Result<Vec<i32>> {
    if names.len() != modified.len() {
        return Err(illegal("each file needs a time"));
    }
    let stale = names.iter().zip(modified).enumerate();
    Ok(stale
        .filter(|(_, (name, at))| send::stale(name, now.wrapping_sub(**at)))
        .map(|(index, _)| index as i32)
        .collect())
}
