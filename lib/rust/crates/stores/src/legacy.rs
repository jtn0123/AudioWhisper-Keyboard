//! Importing the Room databases the keyboard kept before, exactly once, then deleting them.
//!
//! Kotlin reads the old rows (it has Android's SQLite; Rust takes no SQLite dependency) and
//! hands them over with the database's files, main file first. [`import`] then:
//!
//! 1. fingerprints the rows ([`codec::fnv64`] of their encoding);
//! 2. if the store already holds that fingerprint, the rows are in it already, from a run that
//!    stopped before the files were gone: it only deletes the files;
//! 3. otherwise it adds the rows and the fingerprint in one change, which [`Store::change`]
//!    saves crash-safely, then reads the file back and checks it holds exactly what was meant;
//! 4. and only then deletes the files, the main file first ([`disk::remove_all`]), so a crash
//!    between leaves journals with no database, which the next run deletes as rows of none.
//!
//! A crash before step 3's rename leaves the old store and every old file, and the next run
//! imports again; after it, the fingerprint is on disk and the next run only deletes. So no
//! crash loses a row or imports one twice. A store that keeps a file it could not read takes
//! nothing and deletes nothing.

use std::io;
use std::path::PathBuf;

use crate::clip_codec::{write_clip, write_file};
use crate::clips::{Clip, Clips, FileInfo};
use crate::codec::{self, Writer};
use crate::disk;
use crate::store::{Payload, Store};
use crate::word_codec::write_word;
use crate::words::{Word, Words};

/// What [`import`] did, as Kotlin reads its number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Imported {
    /// There were no rows; the files are gone.
    Nothing = 0,
    /// The rows are in the store, and the files are gone.
    Rows = 1,
    /// The store had the rows already; the files are gone.
    Already = 2,
    /// The store keeps a file it could not read, so the old files stay.
    Kept = 3,
}

/// A store that takes in an old database's rows.
pub trait Takes: Payload {
    type Rows;
    fn is_empty(rows: &Self::Rows) -> bool;
    fn fingerprint(rows: &Self::Rows) -> u64;
    fn imported(&self) -> &[u64];
    /// Adds `rows`, keeping their ids if the store is empty, and the fingerprint.
    fn take(&mut self, rows: Self::Rows, print: u64);
}

/// An old clipboard: the history's rows, by id, the files' records, and the history's
/// `AUTOINCREMENT` sequence (0 if it had none).
#[derive(Clone, Debug, Default)]
pub struct ClipRows {
    pub items: Vec<Clip>,
    pub files: Vec<FileInfo>,
    pub sequence: i64,
}

/// An old user dictionary: its rows, by id, and its `AUTOINCREMENT` sequence.
#[derive(Clone, Debug, Default)]
pub struct WordRows {
    pub words: Vec<Word>,
    pub sequence: i64,
}

/// Imports `rows` into `store` once, then deletes the old database's `files`, main file first.
pub fn import<T: Takes>(
    store: &mut Store<T>,
    rows: T::Rows,
    files: &[PathBuf],
) -> io::Result<Imported> {
    if store.read_only() {
        return Ok(Imported::Kept);
    }
    let outcome = if T::is_empty(&rows) {
        Imported::Nothing
    } else {
        let print = T::fingerprint(&rows);
        if store.value().imported().contains(&print) {
            Imported::Already
        } else {
            store.change(|value| value.take(rows, print))?;
            if !store.verify()? {
                return Err(io::Error::other("the store did not read back as written"));
            }
            Imported::Rows
        }
    };
    disk::remove_all(store.disk(), files)?;
    Ok(outcome)
}

/// The id after `ids` and `sequence`, as `AUTOINCREMENT` would give.
fn next_after(ids: impl Iterator<Item = i64>, sequence: i64, next: i64) -> i64 {
    let largest = ids.max().unwrap_or(0).max(sequence);
    next.max(largest.saturating_add(1))
}

impl Takes for Clips {
    type Rows = ClipRows;

    fn is_empty(rows: &ClipRows) -> bool {
        rows.items.is_empty() && rows.files.is_empty()
    }

    fn fingerprint(rows: &ClipRows) -> u64 {
        let mut writer = Writer::default();
        writer.i64(rows.sequence);
        writer.count(rows.items.len());
        rows.items
            .iter()
            .for_each(|clip| write_clip(&mut writer, clip));
        writer.count(rows.files.len());
        rows.files
            .iter()
            .for_each(|file| write_file(&mut writer, file));
        codec::fnv64(&writer.into_bytes())
    }

    fn imported(&self) -> &[u64] {
        &self.imported
    }

    fn take(&mut self, mut rows: ClipRows, print: u64) {
        rows.items.sort_by_key(|clip| clip.id);
        rows.items.dedup_by_key(|clip| clip.id);
        if self.items.is_empty() {
            self.next_id = next_after(rows.items.iter().map(|c| c.id), rows.sequence, self.next_id);
            self.items = rows.items;
        } else {
            for mut clip in rows.items {
                clip.id = self.next_id.max(1);
                self.next_id = clip.id + 1;
                self.items.push(clip);
            }
        }
        for file in rows.files {
            if self.file(file.id).is_none() {
                self.files.push(file);
            }
        }
        self.imported.push(print);
    }
}

impl Takes for Words {
    type Rows = WordRows;

    fn is_empty(rows: &WordRows) -> bool {
        rows.words.is_empty()
    }

    fn fingerprint(rows: &WordRows) -> u64 {
        let mut writer = Writer::default();
        writer.i64(rows.sequence);
        writer.count(rows.words.len());
        rows.words
            .iter()
            .for_each(|word| write_word(&mut writer, word));
        codec::fnv64(&writer.into_bytes())
    }

    fn imported(&self) -> &[u64] {
        &self.imported
    }

    fn take(&mut self, mut rows: WordRows, print: u64) {
        rows.words.sort_by_key(|word| word.id);
        rows.words.dedup_by_key(|word| word.id);
        if self.words.is_empty() {
            let ids = rows.words.iter().map(|w| w.id);
            self.next_id = next_after(ids, rows.sequence, self.next_id);
            self.words = rows.words;
        } else {
            for mut word in rows.words {
                word.id = self.next_id.max(1);
                self.next_id = word.id + 1;
                self.words.push(word);
            }
        }
        self.imported.push(print);
    }
}
