//! A [`Disk`] in memory that can stop the process at any step, to test every crash point.
//!
//! It keeps two views of the files: as the running process sees them, and as they would be
//! after a power cut, which only takes renames and removals once their directory is synced. A
//! file's synced contents survive either way. After a crash, the next "run" starts a new disk
//! from either view.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::disk::Disk;

type Files = BTreeMap<PathBuf, Vec<u8>>;

#[derive(Debug, Default)]
struct State {
    seen: Files,
    synced: Files,
    steps: usize,
    fail_at: Option<usize>,
}

/// How the process stopped.
#[derive(Clone, Copy, Debug)]
pub enum Crash {
    /// Killed: what it wrote stays.
    Death,
    /// The power went: renames and removals not yet synced are undone.
    PowerCut,
}

pub const CRASHES: [Crash; 2] = [Crash::Death, Crash::PowerCut];

/// A handle to the files, shared with the disk a store owns.
#[derive(Clone, Debug, Default)]
pub struct FaultDisk(Arc<Mutex<State>>);

impl FaultDisk {
    /// A disk holding `files`, all of them synced.
    pub fn with(files: &[(&str, &[u8])]) -> Self {
        let files: Files = files
            .iter()
            .map(|(path, bytes)| (PathBuf::from(path), bytes.to_vec()))
            .collect();
        FaultDisk(Arc::new(Mutex::new(State {
            seen: files.clone(),
            synced: files,
            ..State::default()
        })))
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.0.lock().expect("unpoisoned")
    }

    /// Fails the `at`th step from now (0 for the next), and every step after.
    pub fn fail_at(&self, at: usize) {
        let mut state = self.state();
        state.fail_at = Some(state.steps + at);
    }

    /// How many steps it has taken.
    pub fn steps(&self) -> usize {
        self.state().steps
    }

    /// The disk the next run starts with, after `crash`.
    pub fn after(&self, crash: Crash) -> FaultDisk {
        let state = self.state();
        let files = match crash {
            Crash::Death => state.seen.clone(),
            Crash::PowerCut => state.synced.clone(),
        };
        FaultDisk(Arc::new(Mutex::new(State {
            seen: files.clone(),
            synced: files,
            ..State::default()
        })))
    }

    pub fn exists(&self, path: &str) -> bool {
        self.state().seen.contains_key(Path::new(path))
    }

    pub fn file(&self, path: &str) -> Option<Vec<u8>> {
        self.state().seen.get(Path::new(path)).cloned()
    }

    pub fn names(&self) -> Vec<PathBuf> {
        self.state().seen.keys().cloned().collect()
    }

    /// A box of this disk, for a store to own.
    pub fn boxed(&self) -> Box<dyn Disk> {
        Box::new(self.clone())
    }

    fn step(&self) -> io::Result<std::sync::MutexGuard<'_, State>> {
        let mut state = self.state();
        state.steps += 1;
        match state.fail_at {
            Some(at) if state.steps > at => Err(io::Error::other("the process stopped")),
            _ => Ok(state),
        }
    }
}

impl Disk for FaultDisk {
    fn read(&mut self, path: &Path) -> io::Result<Option<Vec<u8>>> {
        Ok(self.step()?.seen.get(path).cloned())
    }

    fn write_synced(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let mut state = self.step()?;
        if let Some(synced) = state.synced.get_mut(path) {
            *synced = bytes.to_vec();
        }
        state.seen.insert(path.to_owned(), bytes.to_vec());
        Ok(())
    }

    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        let mut state = self.step()?;
        let bytes = state.seen.remove(from).ok_or(io::ErrorKind::NotFound)?;
        state.seen.insert(to.to_owned(), bytes);
        Ok(())
    }

    fn sync_dir(&mut self, _path: &Path) -> io::Result<()> {
        let mut state = self.step()?;
        state.synced = state.seen.clone();
        Ok(())
    }

    fn remove(&mut self, path: &Path) -> io::Result<()> {
        self.step()?.seen.remove(path);
        Ok(())
    }
}
