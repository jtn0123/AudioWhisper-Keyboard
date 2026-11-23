//! A store's value and the file it lives in.
//!
//! Every change is made to a copy, which is saved before it replaces the value, so a change that
//! could not be saved changes nothing. A store opened from a file it could not read, or one a
//! newer version wrote, keeps that file as it is: it reads as empty and refuses every change.
//! Between [`Store::begin`] and [`Store::commit`] changes are kept in memory and saved once, or
//! dropped by [`Store::rollback`], as a database transaction would.

use std::io;
use std::path::{Path, PathBuf};

use crate::codec::{self, Error, Reader, Writer};
use crate::disk::{self, Disk};

/// What a store keeps, and how it is written.
pub trait Payload: Clone + Default + PartialEq + Send {
    /// The kind in the file's header.
    const KIND: [u8; 4];
    /// The version this build writes, and the newest it reads.
    const VERSION: u32;
    fn encode(&self, writer: &mut Writer);
    /// The payload of a file at `version`, at most [`Payload::VERSION`].
    fn decode(reader: &mut Reader<'_>, version: u32) -> Result<Self, Error>;
}

/// How the file was when the store was opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opened {
    /// There was none; the first change writes it.
    Missing = 0,
    Loaded = 1,
    /// It was not a whole store of this kind: the store starts empty and its first change
    /// replaces it.
    Damaged = 2,
    /// A newer version wrote it: the store reads as empty and changes nothing.
    Newer = 3,
    /// It could not be read: the store reads as empty and changes nothing.
    Unreadable = 4,
}

/// The bytes of `value` as a file.
pub fn encode<T: Payload>(value: &T) -> Vec<u8> {
    let mut writer = Writer::default();
    value.encode(&mut writer);
    codec::frame(T::KIND, T::VERSION, &writer.into_bytes())
}

/// The value in a file's `bytes`.
pub fn decode<T: Payload>(bytes: &[u8]) -> Result<T, Error> {
    let (version, payload) = codec::unframe(T::KIND, T::VERSION, bytes)?;
    let mut reader = Reader::new(payload);
    let value = T::decode(&mut reader, version)?;
    reader.finish()?;
    Ok(value)
}

/// A store's value, kept in the file at its path.
pub struct Store<T> {
    path: PathBuf,
    disk: Box<dyn Disk>,
    value: T,
    opened: Opened,
    /// The value before the open transaction, if one is open.
    before: Option<T>,
}

impl<T> std::fmt::Debug for Store<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never the value: it is the user's clipboard or words.
        f.debug_struct("Store")
            .field("opened", &self.opened)
            .finish_non_exhaustive()
    }
}

impl<T: Payload> Store<T> {
    /// The store in the file at `path`.
    pub fn open(path: &Path, mut disk: Box<dyn Disk>) -> Self {
        let (value, opened) = match disk::load(disk.as_mut(), path) {
            Ok(None) => (T::default(), Opened::Missing),
            Ok(Some(bytes)) => match decode(&bytes) {
                Ok(value) => (value, Opened::Loaded),
                Err(Error::Newer(_)) => (T::default(), Opened::Newer),
                Err(_) => (T::default(), Opened::Damaged),
            },
            Err(_) => (T::default(), Opened::Unreadable),
        };
        Store {
            path: path.to_owned(),
            disk,
            value,
            opened,
            before: None,
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn opened(&self) -> Opened {
        self.opened
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether changes are refused, to keep a file this build could not read.
    pub fn read_only(&self) -> bool {
        matches!(self.opened, Opened::Newer | Opened::Unreadable)
    }

    /// Makes `change` to a copy of the value and, if that changed anything, saves it and keeps
    /// it. Nothing changes if the copy cannot be saved.
    pub fn change<R>(&mut self, change: impl FnOnce(&mut T) -> R) -> io::Result<R> {
        if self.read_only() {
            return Err(io::Error::other("the store's file is kept as it was found"));
        }
        if self.before.is_some() {
            return Ok(change(&mut self.value));
        }
        let mut next = self.value.clone();
        let result = change(&mut next);
        if next != self.value {
            disk::save(self.disk.as_mut(), &self.path, &encode(&next))?;
            self.value = next;
        }
        Ok(result)
    }

    /// Starts a transaction, unless one is open.
    pub fn begin(&mut self) {
        if self.before.is_none() {
            self.before = Some(self.value.clone());
        }
    }

    /// Saves the transaction's changes, if any; if they cannot be saved, they are dropped.
    pub fn commit(&mut self) -> io::Result<()> {
        let Some(before) = self.before.take() else {
            return Ok(());
        };
        if before == self.value {
            return Ok(());
        }
        let saved = disk::save(self.disk.as_mut(), &self.path, &encode(&self.value));
        if saved.is_err() {
            self.value = before;
        }
        saved
    }

    /// Drops the transaction's changes.
    pub fn rollback(&mut self) {
        if let Some(before) = self.before.take() {
            self.value = before;
        }
    }

    /// Whether the file now holds exactly the value: read back after a change that must not be
    /// lost before anything it replaces is deleted.
    pub fn verify(&mut self) -> io::Result<bool> {
        let bytes = self.disk.read(&self.path)?;
        Ok(bytes.is_some_and(|bytes| decode::<T>(&bytes).is_ok_and(|value| value == self.value)))
    }

    /// The disk, to remove the files an import replaced.
    pub fn disk(&mut self) -> &mut dyn Disk {
        self.disk.as_mut()
    }
}
